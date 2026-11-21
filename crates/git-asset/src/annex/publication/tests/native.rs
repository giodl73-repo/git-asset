use super::*;
use git_asset_store::{
    existing::{Layout, Place},
    metadata::{Limits as MetadataLimits, MetadataReader, MetadataStore},
    source_claims::{Command, admission::Policy},
};
use std::process::Command as Process;

struct Fixture {
    root: PathBuf,
    profile: Profile,
    plan: Plan,
    namespaces: Vec<Namespace>,
    metadata: MetadataReader,
    witness: LocalStore,
    ledger: CapacityLedger,
    _temp: Option<tempfile::TempDir>,
}
#[derive(Deserialize)]
struct Saved {
    profile: Profile,
    plan: Plan,
    metadata: git_asset_store::metadata::Checkpoint,
    namespaces: Vec<Checkpoint>,
    capacity: git_asset_store::ledger::Checkpoint,
}
impl Fixture {
    fn save(&self) {
        let p = &self.profile.pool.pool;
        let pool = serde_json::json!({"git":p.git,"annex":p.annex,"repository":p.repository,"bare":p.bare,
            "uuid":p.uuid,"configuration":p.configuration,"tools":p.tools,"empty_config":p.empty_config,
            "empty_hooks":p.empty_hooks,"temporary":p.temporary,"command_millis":p.command_millis,
            "output_bytes":p.output_bytes,"limits":{"object_bytes":p.limits.object_bytes,"total_bytes":p.limits.total_bytes,"requests":p.limits.requests,"verify_millis":p.limits.verify_millis,"live_readers":p.limits.live_readers}});
        let members: Vec<_> = self
            .profile
            .pool
            .members
            .iter()
            .map(|m| serde_json::json!({"root":m.root,"minimum":m.minimum}))
            .collect();
        let profile = serde_json::json!({"pool":{"pool":pool,"binding":self.profile.pool.binding,"members":members},
            "retained_place":self.profile.retained_place,"staging":self.profile.staging,"staging_marker":self.profile.staging_marker,
            "budget":self.profile.budget,"allow_publish":self.profile.allow_publish});
        let saved = serde_json::json!({"profile":profile,"plan":self.plan,"metadata":self.metadata.checkpoint(),
            "namespaces":self.namespaces.iter().map(|n|n.checkpoint()).collect::<Vec<_>>(),"capacity":self.ledger.checkpoint()});
        fs::write(
            self.root.join("fixture.json"),
            serde_json::to_vec(&saved).unwrap(),
        )
        .unwrap();
    }
    fn load(root: PathBuf) -> Self {
        let s: Saved =
            serde_json::from_slice(&fs::read(root.join("fixture.json")).unwrap()).unwrap();
        let namespaces = s
            .namespaces
            .iter()
            .enumerate()
            .map(|(i, c)| Namespace::open(&root.join(format!("namespace-{i}")), c).unwrap())
            .collect();
        let metadata = MetadataReader::open(&root.join("metadata"), &s.metadata).unwrap();
        let witness = LocalStore::open(&root.join("witness")).unwrap();
        let ledger = CapacityLedger::open(&root.join("ledger"), &s.capacity).unwrap();
        Self {
            root,
            profile: s.profile,
            plan: s.plan,
            namespaces,
            metadata,
            witness,
            ledger,
            _temp: None,
        }
    }
    fn new(bytes: &[u8]) -> Self {
        Self::build(bytes, None, false)
    }
    fn build(bytes: &[u8], tools: Option<(PathBuf, PathBuf, Vec<PathBuf>)>, bare: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        let mut p = profile(&root);
        p.pool.pool.repository = root.join("pool");
        p.pool.pool.bare = bare;
        let pool = &p.pool.pool.repository;
        fs::create_dir(pool).unwrap();
        fs::write(&p.pool.pool.empty_config, b"").unwrap();
        fs::create_dir(&p.pool.pool.empty_hooks).unwrap();
        fs::create_dir(&p.pool.pool.temporary).unwrap();
        let native = tools.is_some();
        if let Some((git, annex, path)) = tools {
            p.pool.pool.git = git;
            p.pool.pool.annex = annex;
            p.pool.pool.tools = path;
            p.pool.pool.command_millis = 120000;
        } else {
            p.pool.pool.git = fake_tool().into();
            p.pool.pool.annex = fake_tool().into();
            p.pool.pool.tools = vec![fake_tool().parent().unwrap().into()];
        }
        if native {
            let init = if bare {
                vec!["init", "--bare", "--quiet"]
            } else {
                vec!["init", "--quiet"]
            };
            process::fixture_command(&p.pool.pool, false, &init).unwrap();
            process::fixture_command(
                &p.pool.pool,
                false,
                &["config", "user.name", "Synthetic fixture"],
            )
            .unwrap();
            process::fixture_command(
                &p.pool.pool,
                false,
                &["config", "user.email", "fixture@example.invalid"],
            )
            .unwrap();
            process::fixture_command(&p.pool.pool, true, &["init", "synthetic-publication"])
                .unwrap();
            let uuid = process::fixture_command(
                &p.pool.pool,
                false,
                &["config", "--local", "--get", "annex.uuid"],
            )
            .unwrap();
            p.pool.pool.uuid = super::super::super::line(&uuid.stdout).unwrap().into();
        } else {
            fs::create_dir_all(pool.join(if bare {
                "annex/objects"
            } else {
                ".git/annex/objects"
            }))
            .unwrap();
            fs::write(
                pool.join(if bare { "config" } else { ".git/config" }),
                b"fixture configuration",
            )
            .unwrap();
        }
        let places = Places {
            current: vec![k("pool")],
            pending: vec![k("pool")],
            predecessor: vec![k("pool")],
        };
        let policy = Policy {
            prefer_active: true,
            classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                .into_iter()
                .map(|x| (x, places.clone()))
                .collect(),
        };
        let mut namespaces: Vec<_> = (0..2)
            .map(|i| {
                Namespace::create(
                    &root.join(format!("namespace-{i}")),
                    k("project"),
                    k("epoch"),
                    policy.clone(),
                )
                .unwrap()
            })
            .collect();
        let empty = MemoryNodes::default();
        let limits = Self::limits();
        for (op, source, class) in [
            ("enroll", "asset", Kind::Retention),
            ("creative-enroll", "creative", Kind::Branch),
        ] {
            namespaces[0]
                .apply(
                    k(op),
                    Command::Enroll {
                        source: k(source),
                        owner: k("owner"),
                        class,
                    },
                    &empty,
                    &limits,
                )
                .unwrap();
        }
        p.pool.binding = Binding::new(
            1,
            Digest::blob(b"synthetic participating boundary"),
            namespaces
                .iter()
                .map(|n| Member {
                    namespace: n.identity().clone(),
                    places: vec![k("pool")],
                })
                .collect(),
        )
        .unwrap();
        p.pool.members = namespaces
            .iter()
            .enumerate()
            .map(|(i, n)| managed::Member {
                root: root.join(format!("namespace-{i}")),
                minimum: n.checkpoint(),
            })
            .collect();
        let binding = p.pool.binding.id().unwrap();
        if native {
            process::fixture_command(
                &p.pool.pool,
                false,
                &[
                    "config",
                    "--local",
                    "tig.protectionbinding",
                    binding.as_str(),
                ],
            )
            .unwrap();
        } else {
            fs::write(pool.join("binding"), binding.as_str()).unwrap();
        }
        p.pool.pool.configuration = Object::from_bytes(
            &fs::read(pool.join(if bare { "config" } else { ".git/config" })).unwrap(),
        );
        fs::create_dir(&p.staging).unwrap();
        fs::write(p.staging.join(".git-asset-staging"), b"stage marker").unwrap();
        let mut metadata = MetadataStore::create(
            &root.join("metadata"),
            MetadataLimits {
                nodes: 10000,
                bytes: 4 * 1024 * 1024,
                database_bytes: 16 * 1024 * 1024,
                min_free_bytes: 65536,
            },
        )
        .unwrap();
        let mut req = request(&mut metadata);
        req.candidate.object = Object::from_bytes(bytes);
        req.expected_revision = namespaces[0].source(&k("asset")).unwrap().unwrap().revision;
        let frozen = freeze(&req, &mut metadata, TreeLimits::default()).unwrap();
        let cp = metadata.finish().unwrap();
        drop(metadata);
        let metadata = MetadataReader::open(&root.join("metadata"), &cp).unwrap();
        fs::create_dir_all(root.join("input/objects")).unwrap();
        fs::write(
            root.join("input/objects")
                .join(req.candidate.object.sha256.as_str()),
            bytes,
        )
        .unwrap();
        let witness = LocalStore::create(&root.join("witness"), k("witness")).unwrap();
        let mut ledger =
            CapacityLedger::create(&root.join("ledger"), k("capacity"), k("epoch"), 65536).unwrap();
        let probe = ledger.begin_probe(k("probe")).unwrap().reference().clone();
        ledger
            .observe(k("observe"), &probe, fs2::available_space(&root).unwrap())
            .unwrap();
        let job = ledger
            .reserve(
                k("job"),
                k("owner"),
                4 * bytes.len() as u64 + 16 * 1024 * 1024,
            )
            .unwrap()
            .reference()
            .clone();
        let plan = Plan::new(
            &p,
            req,
            frozen,
            Admission {
                namespace: namespaces[0].identity().clone(),
                witness: witness.identity().clone(),
                capacity: ledger.identity().clone(),
                job,
            },
        )
        .unwrap();
        Self {
            _temp: Some(temp),
            root,
            profile: p,
            plan,
            namespaces,
            metadata,
            witness,
            ledger,
        }
    }
    fn limits() -> git_asset_store::protection::Limits {
        git_asset_store::protection::Limits {
            millis: 300000,
            roots: 256,
            bindings: 10000,
            associations: 100000,
            reads: 100000,
            read_bytes: 32 * 1024 * 1024,
        }
    }
    fn resources(&mut self) -> Resources<'_> {
        Resources {
            namespace: &mut self.namespaces[0],
            metadata: &mut self.metadata,
            limits: Self::limits(),
            witness: &self.witness,
            ledger: &mut self.ledger,
        }
    }
    fn input(&self) -> ReadCache {
        ReadCache::new(existing::Profile {
            places: vec![Place {
                root: self.root.join("input"),
                layout: Layout::Flat,
            }],
            required: RequiredProtection::PreventMutation,
            limits: existing::Limits {
                object_bytes: 64 * 1024 * 1024,
                total_bytes: 1024 * 1024 * 1024,
                requests: 128,
                verify_millis: 300000,
                live_readers: 8,
            },
        })
        .unwrap()
    }
    fn publisher(&self) -> Publisher {
        Publisher::new(self.profile.clone()).unwrap()
    }
    fn refresh(&mut self) {
        if self
            .ledger
            .allocation_observation(&key(&self.plan.scope().unwrap(), "payload").unwrap())
            .unwrap()
            .is_none()
        {
            let n = self.ledger.checkpoint().sequence();
            let probe = self
                .ledger
                .begin_probe(k(&format!("probe-{n}")))
                .unwrap()
                .reference()
                .clone();
            self.ledger
                .observe(
                    k(&format!("observe-{n}")),
                    &probe,
                    fs2::available_space(&self.root).unwrap(),
                )
                .unwrap();
        }
    }
    fn output_path(&self) -> PathBuf {
        let key = format!(
            "SHA256-s{}--{}",
            self.plan.candidate().object.bytes,
            self.plan.candidate().object.sha256.as_str()
        );
        self.profile
            .pool
            .pool
            .repository
            .join(if self.profile.pool.pool.bare {
                "annex/objects/a/b"
            } else {
                ".git/annex/objects/a/b"
            })
            .join(&key)
            .join(key)
    }
}

#[test]
fn publication_crash_worker() {
    let Some(root) = std::env::var_os("GIT_ASSET_CANDIDATE_FIXTURE_ROOT") else {
        return;
    };
    let mut f = Fixture::load(root.into());
    let p = f.plan.clone();
    let mut input = f.input();
    let mode = std::env::var("GIT_ASSET_CANDIDATE_FIXTURE_MODE").unwrap();
    if mode == "recover" {
        f.refresh();
        let mut ready = f
            .publisher()
            .apply(&p, &mut f.resources(), &mut input)
            .unwrap();
        let mut bytes = vec![];
        ready.read_to_end(&mut bytes).unwrap();
        assert_eq!(Object::from_bytes(&bytes), p.candidate().object);
        fs::write(
            f.root.join("recovered-receipt.json"),
            serde_json::to_vec(ready.receipt()).unwrap(),
        )
        .unwrap();
        return;
    }
    f.refresh();
    f.publisher()
        .apply_hook(&p, &mut f.resources(), &mut input, |stage| {
            if format!("{stage:?}") == mode {
                std::process::exit(73);
            }
            Ok(())
        })
        .unwrap();
    panic!("requested crash boundary not reached");
}
fn worker(f: &Fixture, mode: &str) -> std::process::Output {
    Process::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "annex::publication::tests::native::publication_crash_worker",
            "--test-threads=1",
        ])
        .env("GIT_ASSET_CANDIDATE_FIXTURE_ROOT", &f.root)
        .env("GIT_ASSET_CANDIDATE_FIXTURE_MODE", mode)
        .output()
        .unwrap()
}
fn crash_and_recover(f: &mut Fixture, stage: Stage) {
    f.save();
    let original = fs::read(
        f.root
            .join("input/objects")
            .join(f.plan.candidate().object.sha256.as_str()),
    )
    .unwrap();
    let exit = worker(f, &format!("{stage:?}"));
    assert_eq!(
        exit.status.code(),
        Some(73),
        "{} {}",
        String::from_utf8_lossy(&exit.stdout),
        String::from_utf8_lossy(&exit.stderr)
    );
    let revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    assert_eq!(revision, f.plan.request.expected_revision + 2);
    assert_eq!(
        fs::read(
            f.root
                .join("input/objects")
                .join(f.plan.candidate().object.sha256.as_str())
        )
        .unwrap(),
        original
    );
    if matches!(
        stage,
        Stage::Staged | Stage::Imported | Stage::Completed | Stage::Settled
    ) {
        fs::rename(f.root.join("input"), f.root.join("input-offline")).unwrap();
    }
    let output = worker(f, "recover");
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        f.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .revision,
        revision
    );
    let receipt: Receipt =
        serde_json::from_slice(&fs::read(f.root.join("recovered-receipt.json")).unwrap()).unwrap();
    let p = f.plan.clone();
    let ready = f.publisher().reopen(&p, &mut f.resources()).unwrap();
    assert_eq!(ready.receipt(), &receipt);
}
#[test]
fn actual_process_exit_matrix_recovers_original_registration_stage_and_receipt() {
    for stage in [
        Stage::Registered,
        Stage::Intent,
        Stage::StagingChunk,
        Stage::Staged,
        Stage::Imported,
        Stage::Completed,
        Stage::Settled,
    ] {
        crash_and_recover(&mut Fixture::new(&vec![41u8; 150000]), stage);
    }
}
#[test]
fn competing_attempt_defers_without_global_lock_and_settled_status_is_read_only() {
    let mut f = Fixture::new(b"original result");
    let p = f.plan.clone();
    let mut input = f.input();
    f.refresh();
    let mut peer = Fixture::load({
        f.save();
        f.root.clone()
    });
    let mut saw_busy = false;
    let ready = f
        .publisher()
        .apply_hook(&p, &mut f.resources(), &mut input, |stage| {
            if stage == Stage::Intent {
                let mut source = peer.input();
                assert!(matches!(
                    peer.publisher()
                        .apply(&p, &mut peer.resources(), &mut source),
                    Err(Error::Busy)
                ));
                saw_busy = true;
                // Another allocation in the same domain still progresses; the held
                // fence belongs only to this publication attempt, not the cache.
                let alloc = peer
                    .ledger
                    .begin_allocation(k("unrelated"), &p.admission.job, k("other"), 4096)
                    .unwrap();
                peer.ledger
                    .complete_allocation(k("unrelated-done"), alloc.reference(), 0)
                    .unwrap();
            }
            Ok(())
        })
        .unwrap();
    drop(ready);
    assert!(saw_busy);
    let cap = f.ledger.checkpoint();
    let ns = f.namespaces[0].checkpoint();
    let status = f.publisher().status(&p, &mut f.resources()).unwrap();
    assert!(
        status.accounting_settled
            && status.historical_completion
            && status.protected
            && status.bytes_verified
    );
    assert_eq!(f.ledger.checkpoint(), cap);
    assert_eq!(f.namespaces[0].checkpoint(), ns);
    assert!(matches!(
        f.ledger
            .allocation_observation(&k("unrelated"))
            .unwrap()
            .unwrap()
            .charged_bytes,
        Some(0)
    ));
}
#[test]
fn native_candidate_roundtrip_preserves_source_and_reopens_after_scratch_retirement() {
    let mut f = Fixture::new(b"original result");
    let p = f.plan.clone();
    let mut input = f.input();
    let mut publisher = f.publisher();
    f.refresh();
    let creative = f.namespaces[0].source(&k("creative")).unwrap();
    let mut ready = publisher.apply(&p, &mut f.resources(), &mut input).unwrap();
    let receipt = ready.receipt().clone();
    let mut bytes = vec![];
    ready.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"original result");
    assert_eq!(ready.held_namespaces(), 2);
    assert!(
        ready
            .with_path(|path| fs::remove_file(path))
            .unwrap()
            .is_err()
    );
    for n in &f.namespaces {
        assert!(matches!(
            n.fence(&p.candidate().object),
            Err(admission::Error::Busy)
        ));
    }
    assert_eq!(f.namespaces[0].source(&k("creative")).unwrap(), creative);
    drop(ready);
    let revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    assert_eq!(
        fs::read(
            f.root
                .join("input/objects")
                .join(p.candidate().object.sha256.as_str())
        )
        .unwrap(),
        b"original result"
    );
    let again = f.publisher().resume(&p, &mut f.resources()).unwrap();
    assert_eq!(again.receipt(), &receipt);
    drop(again);
    f.ledger.finish(k("finish-job"), &p.admission.job).unwrap();
    f.ledger.retire_closed(&p.admission.job).unwrap();
    fs::rename(f.root.join("input"), f.root.join("input-retired")).unwrap();
    fs::rename(&f.profile.staging, f.root.join("staging-retired")).unwrap();
    let before = f.ledger.checkpoint();
    let reopened = f.publisher().reopen(&p, &mut f.resources()).unwrap();
    assert_eq!(reopened.receipt(), &receipt);
    drop(reopened);
    assert_eq!(f.ledger.checkpoint(), before);
    assert_eq!(
        f.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .revision,
        revision
    );
}
#[test]
fn unknown_native_outcome_recovers_original_operation_without_source_or_duplicate_registration() {
    let mut f = Fixture::new(b"original result");
    let p = f.plan.clone();
    let mut input = f.input();
    f.refresh();
    fs::write(f.profile.pool.pool.repository.join("mode"), b"lost-reply").unwrap();
    assert!(matches!(
        f.publisher().apply(&p, &mut f.resources(), &mut input),
        Err(Error::Uncertain)
    ));
    let revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    fs::rename(f.root.join("input"), f.root.join("input-offline")).unwrap();
    let ready = f.publisher().resume(&p, &mut f.resources()).unwrap();
    drop(ready);
    assert_eq!(
        f.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .revision,
        revision
    );
    let calls = fs::read_to_string(f.profile.pool.pool.repository.join("calls")).unwrap();
    assert_eq!(
        calls.lines().filter(|s| s.starts_with("reinject ")).count(),
        1
    );
}
#[test]
fn disabled_stale_corrupt_and_acknowledged_missing_never_republish() {
    let mut f = Fixture::new(b"original result");
    let p = f.plan.clone();
    let mut input = f.input();
    let mut disabled = f.profile.clone();
    disabled.allow_publish = false;
    assert!(matches!(
        Publisher::new(disabled)
            .unwrap()
            .apply(&p, &mut f.resources(), &mut input),
        Err(Error::Denied)
    ));
    assert!(!f.profile.pool.pool.repository.join("calls").exists());
    let mut bad = p.clone();
    bad.request.owner = k("other");
    assert!(matches!(
        f.publisher().apply(&bad, &mut f.resources(), &mut input),
        Err(Error::Stale)
    ));
    f.refresh();
    drop(
        f.publisher()
            .apply(&p, &mut f.resources(), &mut input)
            .unwrap(),
    );
    fs::write(f.output_path(), b"corrupt").unwrap();
    assert!(matches!(
        f.publisher().resume(&p, &mut f.resources()),
        Err(Error::Integrity)
    ));
    fs::remove_file(f.output_path()).unwrap();
    assert!(matches!(
        f.publisher().resume(&p, &mut f.resources()),
        Err(Error::MissingAcknowledged)
    ));
}
#[test]
fn explicitly_retired_candidate_is_not_reregistered_by_old_publication_retry() {
    let mut f = Fixture::new(b"original result");
    let p = f.plan.clone();
    let mut input = f.input();
    f.refresh();
    drop(
        f.publisher()
            .apply(&p, &mut f.resources(), &mut input)
            .unwrap(),
    );
    let source = f.namespaces[0].source(&k("asset")).unwrap().unwrap();
    let r = f.namespaces[0]
        .apply(
            k("withdraw"),
            Command::Withdraw {
                source: k("asset"),
                owner: k("owner"),
                expected_revision: source.revision,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    f.namespaces[0]
        .apply(
            k("retire"),
            Command::Retire {
                source: k("asset"),
                owner: k("owner"),
                expected_revision: r.source_revision,
                predecessor_revision: source.current.unwrap().revision,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    assert!(matches!(
        f.publisher().resume(&p, &mut f.resources()),
        Err(Error::NeedsRecovery) | Err(Error::ProtectionLost)
    ));
}
#[test]
fn pinned_staging_marker_and_owned_file_collision_refuse_without_native_write() {
    for wrong_marker in [true, false] {
        let mut f = Fixture::new(b"original result");
        let p = f.plan.clone();
        let mut input = f.input();
        if wrong_marker {
            fs::write(
                f.profile.staging.join(".git-asset-staging"),
                b"wrong marker",
            )
            .unwrap();
        } else {
            let path = staging::path(&f.profile, &p).unwrap();
            fs::create_dir(path.parent().unwrap()).unwrap();
            fs::write(path.parent().unwrap().join("unowned"), b"preserve").unwrap();
        }
        f.refresh();
        assert!(
            f.publisher()
                .apply(&p, &mut f.resources(), &mut input)
                .is_err()
        );
        let calls = fs::read_to_string(f.profile.pool.pool.repository.join("calls")).unwrap();
        assert!(!calls.lines().any(|s| s.starts_with("reinject ")));
        assert!(!f.output_path().exists());
        if !wrong_marker {
            assert_eq!(
                fs::read(
                    staging::path(&f.profile, &p)
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join("unowned")
                )
                .unwrap(),
                b"preserve"
            );
        }
    }
}
#[test]
fn native_namespace_is_held_and_changed_staging_location_is_stale() {
    let mut f = Fixture::new(b"original result");
    let p = f.plan.clone();
    let mut input = f.input();
    let mut changed = f.profile.clone();
    changed.staging = f.root.join("different-stage");
    assert!(matches!(
        Publisher::new(changed)
            .unwrap()
            .apply(&p, &mut f.resources(), &mut input),
        Err(Error::Stale)
    ));
    assert!(!f.profile.pool.pool.repository.join("calls").exists());
    let annex = f.profile.pool.pool.repository.join(".git/annex");
    let renamed = f.root.join("annex-moved");
    f.refresh();
    let mut checked = false;
    let ready = f
        .publisher()
        .apply_hook(&p, &mut f.resources(), &mut input, |stage| {
            if stage == Stage::Intent {
                assert!(fs::rename(&annex, &renamed).is_err());
                checked = true;
            }
            Ok(())
        })
        .unwrap();
    assert!(checked);
    drop(ready);
}
#[test]
fn same_operation_cannot_change_provenance_or_reuse_a_stale_baseline() {
    let mut f = Fixture::new(b"original result");
    let original = f.plan.clone();
    let mut input = f.input();
    f.refresh();
    drop(
        f.publisher()
            .apply(&original, &mut f.resources(), &mut input)
            .unwrap(),
    );
    let revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    let mut metadata =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let mut request = original.request.clone();
    request.candidate.provenance = metadata.put(b"different returned provenance").unwrap();
    let frozen = freeze(&request, &mut metadata, TreeLimits::default()).unwrap();
    let minimum = metadata.finish().unwrap();
    drop(metadata);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &minimum).unwrap();
    let changed = Plan::new(
        &f.profile,
        request.clone(),
        frozen.clone(),
        original.admission.clone(),
    )
    .unwrap();
    assert!(matches!(
        f.publisher()
            .apply(&changed, &mut f.resources(), &mut input),
        Err(Error::Conflict)
    ));
    request.operation = k("stale-new-operation");
    // Descriptor must match that different original operation before baseline admission.
    let mut metadata =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let frozen = freeze(&request, &mut metadata, TreeLimits::default()).unwrap();
    let minimum = metadata.finish().unwrap();
    drop(metadata);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &minimum).unwrap();
    let stale = Plan::new(&f.profile, request, frozen, original.admission.clone()).unwrap();
    assert!(matches!(
        f.publisher().apply(&stale, &mut f.resources(), &mut input),
        Err(Error::Stale)
    ));
    assert_eq!(
        f.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .revision,
        revision
    );
}
#[test]
#[ignore = "explicit Git/annex tools and owned synthetic native pools"]
fn actual_native_candidate_publication_ordinary_bare_and_large_binary() {
    let git: PathBuf = std::env::var_os("GIT_ASSET_TEST_GIT_EXE")
        .expect("git path")
        .into();
    let annex: PathBuf = std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE")
        .expect("annex path")
        .into();
    let path = std::env::split_paths(
        &std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").expect("runtime PATH"),
    )
    .collect::<Vec<_>>();
    let bytes = vec![29u8; 32 * 1024 * 1024];
    for bare in [false, true] {
        let mut f = Fixture::build(
            &bytes,
            Some((git.clone(), annex.clone(), path.clone())),
            bare,
        );
        let p = f.plan.clone();
        let mut input = f.input();
        let pool = &f.profile.pool.pool.repository;
        let head = fs::read(pool.join(if bare { "HEAD" } else { ".git/HEAD" })).unwrap();
        let config = fs::read(pool.join(if bare { "config" } else { ".git/config" })).unwrap();
        let objects = pool.join(if bare {
            "annex/objects"
        } else {
            ".git/annex/objects"
        });
        let had_objects = objects.exists();
        let cap = f.ledger.checkpoint();
        let status = f.publisher().status(&p, &mut f.resources()).unwrap();
        assert!(!status.registered && !status.bytes_verified);
        assert_eq!(objects.exists(), had_objects);
        assert_eq!(f.ledger.checkpoint(), cap);
        f.refresh();
        let mut ready = f
            .publisher()
            .apply(&p, &mut f.resources(), &mut input)
            .unwrap();
        let receipt = ready.receipt().clone();
        let mut actual = vec![];
        ready.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, bytes);
        drop(ready);
        let pool = &f.profile.pool.pool.repository;
        assert_eq!(
            fs::read(pool.join(if bare { "HEAD" } else { ".git/HEAD" })).unwrap(),
            head
        );
        assert_eq!(
            fs::read(pool.join(if bare { "config" } else { ".git/config" })).unwrap(),
            config
        );
        let ready = f.publisher().resume(&p, &mut f.resources()).unwrap();
        assert_eq!(ready.receipt(), &receipt);
    }
}

#[test]
#[ignore = "explicit native Git/annex tools and actual process-exit qualification"]
fn actual_native_process_exits_preserve_retained_outputs_and_reconcile_without_input() {
    let git: PathBuf = std::env::var_os("GIT_ASSET_TEST_GIT_EXE").unwrap().into();
    let annex: PathBuf = std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE").unwrap().into();
    let path = std::env::split_paths(&std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").unwrap())
        .collect::<Vec<_>>();
    for bare in [false, true] {
        for stage in [Stage::Imported, Stage::Completed] {
            let mut f = Fixture::build(
                &vec![17u8; 150000],
                Some((git.clone(), annex.clone(), path.clone())),
                bare,
            );
            crash_and_recover(&mut f, stage);
        }
    }
}

#[test]
fn previous_candidate_version_reopens_without_retargeting_current_registration() {
    let mut f = Fixture::new(b"original result");
    let old = f.plan.clone();
    let mut input = f.input();
    f.refresh();
    let ready = f
        .publisher()
        .apply(&old, &mut f.resources(), &mut input)
        .unwrap();
    let old_receipt = ready.receipt().clone();
    drop(ready);
    let mut metadata =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let mut request = old.request.clone();
    request.operation = k("next");
    request.expected_revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    request.candidate.object = Object::from_bytes(b"new version");
    request.candidate.provenance = metadata.put(b"new opaque provenance").unwrap();
    let frozen = freeze(&request, &mut metadata, TreeLimits::default()).unwrap();
    let cp = metadata.finish().unwrap();
    drop(metadata);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &cp).unwrap();
    fs::write(
        f.root
            .join("input/objects")
            .join(request.candidate.object.sha256.as_str()),
        b"new version",
    )
    .unwrap();
    let next = Plan::new(&f.profile, request, frozen, old.admission.clone()).unwrap();
    f.plan = next.clone();
    f.refresh();
    let mut new_input = f.input();
    let ready = f
        .publisher()
        .apply(&next, &mut f.resources(), &mut new_input)
        .unwrap();
    drop(ready);
    let current = f.namespaces[0].source(&k("asset")).unwrap().unwrap();
    assert_eq!(
        current.current.as_ref().unwrap().selection,
        next.selection.selection
    );
    assert!(
        current
            .predecessors
            .iter()
            .any(|v| v.selection == old.selection.selection)
    );
    let mut prior = f.publisher().reopen(&old, &mut f.resources()).unwrap();
    assert_eq!(prior.receipt(), &old_receipt);
    let mut bytes = vec![];
    prior.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"original result");
    drop(prior);
    assert_eq!(
        f.namespaces[0].source(&k("asset")).unwrap().unwrap(),
        current
    );
}
