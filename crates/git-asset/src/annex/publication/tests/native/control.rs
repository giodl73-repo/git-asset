use super::*;
use crate::annex::control;
use git_asset_store::{
    accounted, local_capacity::LocalCapacityProbe, replica::ReplicaEnrollment, retention,
    retirement,
};
struct Control {
    store: LocalStore,
    ledger: CapacityLedger,
    job: JournalRef,
    namespace: Namespace,
    backup: LocalStore,
    backup_ledger: CapacityLedger,
    keeper: retirement::Keeper,
    plan: control::Plan,
}
impl Control {
    fn new(f: &mut Fixture) -> Self {
        Self::build(f, false)
    }
    fn build(f: &mut Fixture, native_tools: bool) -> Self {
        Self::build_reacquirable(f, native_tools, false)
    }
    fn build_reacquirable(f: &mut Fixture, native_tools: bool, allow_reacquire: bool) -> Self {
        let native = crate::annex::Reader::new(f.profile.pool.pool.clone())
            .unwrap()
            .store();
        let places = Places {
            current: vec![k("metadata")],
            pending: vec![k("metadata")],
            predecessor: vec![k("metadata")],
        };
        let policy = Policy {
            prefer_active: true,
            classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                .into_iter()
                .map(|k| (k, places.clone()))
                .collect(),
        };
        let namespace = Namespace::create(
            &f.root.join("control-namespace"),
            k("metadata"),
            k("epoch"),
            policy,
        )
        .unwrap();
        let binding = Binding::new(
            1,
            Digest::blob(b"owned synthetic metadata actors"),
            vec![Member {
                namespace: namespace.identity().clone(),
                places: vec![k("metadata")],
            }],
        )
        .unwrap();
        let store = LocalStore::create_managed(
            &f.root.join("control-store"),
            control::place(&native).unwrap(),
            binding,
        )
        .unwrap();
        let mut ledger =
            CapacityLedger::create(&f.root.join("control-ledger"), k("metadata"), k("epoch"), 0)
                .unwrap();
        LocalCapacityProbe::bind(&store, &mut ledger)
            .unwrap()
            .observe(&store, &mut ledger, k("probe"), k("sample"))
            .unwrap();
        let job = ledger
            .reserve(k("job"), k("controller"), 64 * 1024 * 1024)
            .unwrap()
            .reference()
            .clone();
        let profile = control::Profile {
            root: f.root.join("control-store"),
            store: store.identity().clone(),
            ledger: f.root.join("control-ledger"),
            minimum: ledger.checkpoint(),
            owner: k("controller"),
            allow_reacquire,
        };
        let id = profile.binding(&native).unwrap();
        let config = f
            .profile
            .pool
            .pool
            .repository
            .join(if f.profile.pool.pool.bare {
                "config"
            } else {
                ".git/config"
            });
        if native_tools {
            crate::annex::process::fixture_command(
                &f.profile.pool.pool,
                false,
                &[
                    "config",
                    "--local",
                    "gitasset.lifecyclecontrol",
                    id.as_str(),
                ],
            )
            .unwrap();
        } else {
            fs::write(f.profile.pool.pool.repository.join("control"), id.as_str()).unwrap();
            let mut bytes = fs::read(&config).unwrap();
            bytes.extend_from_slice(id.as_str().as_bytes());
            fs::write(&config, bytes).unwrap();
        }
        f.profile.pool.pool.configuration = Object::from_bytes(&fs::read(config).unwrap());
        f.control_profile = Some(profile);
        let request =
            control::initial_request(&f.plan.candidate().object, k("controller")).unwrap();
        let publication = accounted::plan_bound(
            &store,
            &request,
            control::writer_context(&f.profile, &f.plan.request).unwrap(),
        )
        .unwrap();
        let document = control::document(&f.plan.candidate().object).unwrap();
        accounted::publish_accounted(
            &mut ledger,
            &store,
            &job,
            &publication,
            &mut Cursor::new(&document),
        )
        .unwrap();
        let plan = control::Plan {
            publication,
            job: job.clone(),
        };
        f.plan = Plan::controlled(
            &f.profile,
            f.plan.request.clone(),
            f.plan.selection.selection.clone(),
            f.plan.admission.clone(),
            f.control_profile.as_ref().unwrap(),
            plan.clone(),
        )
        .unwrap();
        let mut backup = LocalStore::create(&f.root.join("control-backup"), k("backup")).unwrap();
        git_asset_store::publish(&mut backup, &request, &mut Cursor::new(&document)).unwrap();
        let mut backup_ledger = CapacityLedger::create(
            &f.root.join("control-backup-ledger"),
            k("backup"),
            k("epoch"),
            0,
        )
        .unwrap();
        let probe = backup_ledger
            .begin_probe(k("probe"))
            .unwrap()
            .reference()
            .clone();
        backup_ledger
            .observe(k("sample"), &probe, 64 * 1024 * 1024)
            .unwrap();
        let retained = retention::plan(
            &backup,
            &retention::RetentionRequest {
                operation: k("retain"),
                owner: k("controller"),
                publication: request,
                minimum_ack: git_asset_store::AckLevel::ProcessRestart,
            },
        )
        .unwrap();
        let reserved = backup_ledger
            .reserve(k("job"), k("controller"), retained.required_peak())
            .unwrap()
            .reference()
            .clone();
        let grant = retention::retain(&mut backup_ledger, &backup, &reserved, &retained)
            .unwrap()
            .grant;
        let keeper = retirement::Keeper {
            grant,
            enrollment: ReplicaEnrollment {
                store: backup.identity().clone(),
                failure_domain: k("synthetic-metadata-backup"),
            },
        };
        Self {
            store,
            ledger,
            job,
            namespace,
            backup,
            backup_ledger,
            keeper,
            plan,
        }
    }
    fn retirement(&self, operation: &str) -> retirement::Plan {
        retirement::plan(
            &self.store,
            retirement::Request {
                operation: k(operation),
                owner: k("controller"),
                publication: self.plan.publication.request().clone(),
                context: Digest::blob(b"synthetic-metadata-operation"),
                source: self.enrollment(),
                keepers: vec![self.keeper.clone()],
                required_domains: 1,
                capacity: self.ledger.identity().clone(),
            },
        )
        .unwrap()
    }
    fn enrollment(&self) -> ReplicaEnrollment {
        ReplicaEnrollment {
            store: self.store.identity().clone(),
            failure_domain: k("synthetic-metadata-source"),
        }
    }
    fn execute(
        &mut self,
        plan: &retirement::Plan,
        pending: bool,
        cancel: bool,
    ) -> retirement::Result<retirement::Receipt> {
        let source = self.enrollment();
        let mut backups = [retirement::Backup {
            store: &self.backup,
            ledger: &mut self.backup_ledger,
            enrollment: &self.keeper.enrollment,
        }];
        let context = Digest::blob(b"synthetic-metadata-operation");
        let mut execution = retirement::Execution {
            store: &self.store,
            ledger: &mut self.ledger,
            job: &self.job,
            namespaces: std::slice::from_mut(&mut self.namespace),
            nodes: &MemoryNodes::default(),
            limits: &Fixture::limits(),
            backups: &mut backups,
            admitted_context: &context,
            source_enrollment: &source,
        };
        let auth = plan.authorize().unwrap();
        if cancel {
            execution.cancel(plan, &auth)
        } else if pending {
            execution.apply_with_effect(plan, &auth, |_| {
                Err(git_asset_store::Error::NeedsRecovery.into())
            })
        } else {
            execution.apply(plan, &auth)
        }
    }
}
#[test]
fn native_publication_holds_control_and_pending_metadata_fences_writers_across_child_reopen() {
    let mut f = Fixture::new(b"native controlled body");
    let mut control = Control::new(&mut f);
    let plan = f.plan.clone();
    f.refresh();
    let mut input = f.input();
    let ready = f
        .publisher()
        .apply(&plan, &mut f.resources(), &mut input)
        .unwrap();
    let retired = control.retirement("pending-native-control");
    assert!(matches!(
        control.execute(&retired, true, false),
        Err(retirement::Error::Protection(
            git_asset_store::store_protection::Error::Busy
        ))
    ));
    drop(ready);
    assert!(matches!(
        control.execute(&retired, true, false),
        Err(retirement::Error::Store(
            git_asset_store::Error::NeedsRecovery
        ))
    ));
    // An unrelated body can publish through the same pool/control store while A is pending.
    let other_bytes = b"unrelated controlled body";
    let mut metadata =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let mut other = plan.request.clone();
    other.operation = k("unrelated-body");
    other.candidate.object = Object::from_bytes(other_bytes);
    other.expected_revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    let frozen = freeze(&other, &mut metadata, TreeLimits::default()).unwrap();
    let cp = metadata.finish().unwrap();
    drop(metadata);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &cp).unwrap();
    fs::write(
        f.root
            .join("input/objects")
            .join(other.candidate.object.sha256.as_str()),
        other_bytes,
    )
    .unwrap();
    let request = control::initial_request(&other.candidate.object, k("controller")).unwrap();
    let metadata_plan = accounted::plan_bound(
        &control.store,
        &request,
        control::writer_context(&f.profile, &other).unwrap(),
    )
    .unwrap();
    accounted::publish_accounted(
        &mut control.ledger,
        &control.store,
        &control.job,
        &metadata_plan,
        &mut Cursor::new(control::document(&other.candidate.object).unwrap()),
    )
    .unwrap();
    let other = Plan::controlled(
        &f.profile,
        other,
        frozen,
        plan.admission.clone(),
        f.control_profile.as_ref().unwrap(),
        control::Plan {
            publication: metadata_plan,
            job: control.job.clone(),
        },
    )
    .unwrap();
    f.plan = other.clone();
    f.refresh();
    f.plan = plan.clone();
    let mut input = f.input();
    f.publisher()
        .apply(&other, &mut f.resources(), &mut input)
        .unwrap();
    let before = f.ledger.checkpoint();
    assert!(matches!(
        f.publisher().resume(&plan, &mut f.resources()),
        Err(Error::NeedsRecovery)
    ));
    assert_eq!(f.ledger.checkpoint(), before);
    f.save();
    let output = Process::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "annex::publication::tests::native::control::control_gate_worker",
            "--nocapture",
        ])
        .env("GIT_ASSET_CONTROL_FIXTURE_ROOT", &f.root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(control.execute(&retired, false, true).unwrap().cancelled);
    f.publisher().reopen(&plan, &mut f.resources()).unwrap();
    let retired = control.retirement("retire-control-only");
    control.execute(&retired, false, false).unwrap();
    assert!(matches!(
        f.publisher().resume(&plan, &mut f.resources()),
        Err(Error::Stale)
    ));
    // Metadata retirement is not native removal: body remains, no native drop API exists.
    assert_eq!(
        Object::from_bytes(&fs::read(f.output_path()).unwrap()),
        plan.candidate().object
    );
}
#[test]
fn control_gate_worker() {
    let Some(root) = std::env::var_os("GIT_ASSET_CONTROL_FIXTURE_ROOT") else {
        return;
    };
    let mut f = Fixture::load(root.into());
    let p = f.plan.clone();
    let before = f.ledger.checkpoint();
    assert!(matches!(
        f.publisher().resume(&p, &mut f.resources()),
        Err(Error::NeedsRecovery)
    ));
    assert_eq!(f.ledger.checkpoint(), before);
}
#[test]
fn configured_control_cannot_be_omitted_and_alias_cannot_repair_missing_native_bytes() {
    let mut f = Fixture::new(b"native same-hash alias");
    let control = Control::new(&mut f);
    let p = f.plan.clone();
    let profile = f.profile.clone();
    let unguarded = Plan::new(
        &profile,
        p.request.clone(),
        p.selection.selection.clone(),
        p.admission.clone(),
    )
    .unwrap();
    let before = f.ledger.checkpoint();
    let mut unguarded_input = f.input();
    assert!(matches!(
        Publisher::new(profile).unwrap().apply(
            &unguarded,
            &mut f.resources(),
            &mut unguarded_input
        ),
        Err(Error::Stale)
    ));
    assert_eq!(f.ledger.checkpoint(), before);
    let mut input = f.input();
    f.refresh();
    f.publisher()
        .apply(&p, &mut f.resources(), &mut input)
        .unwrap();
    let mut metadata =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let mut request = p.request.clone();
    request.operation = k("another-logical-registration");
    request.candidate.logical_id = "another logical asset".into();
    request.expected_revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    let frozen = freeze(&request, &mut metadata, TreeLimits::default()).unwrap();
    let cp = metadata.finish().unwrap();
    drop(metadata);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &cp).unwrap();
    let alias = Plan::controlled(
        &f.profile,
        request,
        frozen,
        p.admission.clone(),
        f.control_profile.as_ref().unwrap(),
        control.plan.clone(),
    )
    .unwrap();
    let mut input = f.input();
    f.plan = alias.clone();
    f.refresh();
    // An unacknowledged alias cannot become the missing body's producer.
    let native_path = f.output_path();
    let original = fs::read(&native_path).unwrap();
    fs::remove_file(&native_path).unwrap();
    let before = f.ledger.checkpoint();
    assert!(matches!(
        f.publisher().apply(&alias, &mut f.resources(), &mut input),
        Err(Error::NeedsRecovery)
    ));
    assert_eq!(f.ledger.checkpoint(), before);
    assert!(!native_path.exists());
    // Restore only the owned fake fixture, then exercise normal byte reuse.
    fs::write(&native_path, original).unwrap();
    f.publisher()
        .apply(&alias, &mut f.resources(), &mut input)
        .unwrap();
    assert!(matches!(
        alias.producer_completion(
            &f.profile,
            f.control_profile.as_ref().unwrap(),
            &f.witness,
            &mut f.ledger
        ),
        Err(Error::Denied)
    ));
    fs::remove_file(f.output_path()).unwrap();
    assert!(matches!(
        f.publisher().resume(&alias, &mut f.resources()),
        Err(Error::MissingAcknowledged)
    ));
}

#[test]
#[ignore = "explicit native Git/annex tools; fresh controlled ordinary and bare pools"]
fn actual_native_control_gate_blocks_pending_writes_without_removing_native_bytes() {
    let git: PathBuf = std::env::var_os("GIT_ASSET_TEST_GIT_EXE").unwrap().into();
    let annex: PathBuf = std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE").unwrap().into();
    let path = std::env::split_paths(&std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").unwrap())
        .collect::<Vec<_>>();
    for bare in [false, true] {
        let mut f = Fixture::build(
            &vec![31; 128 * 1024],
            Some((git.clone(), annex.clone(), path.clone())),
            bare,
        );
        let mut control = Control::build(&mut f, true);
        let p = f.plan.clone();
        let mut input = f.input();
        f.refresh();
        let ready = f
            .publisher()
            .apply(&p, &mut f.resources(), &mut input)
            .unwrap();
        let retired = control.retirement("pending-controlled-native");
        assert!(matches!(
            control.execute(&retired, true, false),
            Err(retirement::Error::Protection(
                git_asset_store::store_protection::Error::Busy
            ))
        ));
        drop(ready);
        assert!(matches!(
            control.execute(&retired, true, false),
            Err(retirement::Error::Store(
                git_asset_store::Error::NeedsRecovery
            ))
        ));
        assert!(matches!(
            f.publisher().resume(&p, &mut f.resources()),
            Err(Error::NeedsRecovery)
        ));
        assert!(control.execute(&retired, false, true).unwrap().cancelled);
        let mut read = f.publisher().reopen(&p, &mut f.resources()).unwrap();
        let mut bytes = vec![];
        read.read_to_end(&mut bytes).unwrap();
        assert_eq!(Object::from_bytes(&bytes), p.candidate().object);
    }
}

#[test]
fn controlled_copy_retains_both_metadata_guards_and_cannot_bypass_destination_pending() {
    use crate::annex::replication as copy;
    let mut source = Fixture::new(b"controlled copy");
    let mut target = Fixture::new(b"controlled copy");
    assert_ne!(source.profile.pool.pool.uuid, target.profile.pool.pool.uuid);
    let mut source_control = Control::new(&mut source);
    let mut target_control = Control::new(&mut target);
    let source_plan = source.plan.clone();
    let mut input = source.input();
    source.refresh();
    source
        .publisher()
        .apply(&source_plan, &mut source.resources(), &mut input)
        .unwrap();
    let profile = copy::Profile {
        source: source.profile.clone(),
        destination: target.profile.clone(),
        source_domain: k("one-synthetic-device"),
        destination_domain: k("one-synthetic-device"),
    };
    let plan = copy::Plan::new(&profile, source.plan.clone(), target.plan.clone()).unwrap();
    target.refresh();
    assert!(matches!(
        copy::Copier::new(profile.clone()).unwrap().apply(
            &plan,
            &mut source.resources(),
            &mut target.resources()
        ),
        Err(Error::Stale)
    ));
    let mut copier = copy::Copier::controlled(
        profile,
        source.control_profile.clone(),
        target.control_profile.clone(),
    )
    .unwrap();
    let mut ready = copier
        .apply(&plan, &mut source.resources(), &mut target.resources())
        .unwrap();
    let mut actual = vec![];
    ready.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, b"controlled copy");
    let source_retired = source_control.retirement("source-metadata");
    let target_retired = target_control.retirement("target-metadata");
    assert!(matches!(
        source_control.execute(&source_retired, true, false),
        Err(retirement::Error::Protection(
            git_asset_store::store_protection::Error::Busy
        ))
    ));
    assert!(matches!(
        target_control.execute(&target_retired, true, false),
        Err(retirement::Error::Protection(
            git_asset_store::store_protection::Error::Busy
        ))
    ));
    drop(ready);
    assert!(matches!(
        target_control.execute(&target_retired, true, false),
        Err(retirement::Error::Store(
            git_asset_store::Error::NeedsRecovery
        ))
    ));
    assert!(matches!(
        copier.resume(&plan, &mut target.resources()),
        Err(Error::NeedsRecovery)
    ));
    assert_eq!(
        Object::from_bytes(&fs::read(target.output_path()).unwrap()),
        plan.destination().candidate().object
    );
}

mod eviction;

#[test]
fn original_guarded_producer_completion_is_exact_readonly_and_holds_control_origin() {
    let mut f = Fixture::new(b"guarded producer proof");
    let mut c = Control::new(&mut f);
    let plan = f.plan.clone();
    let profile = f.control_profile.clone().unwrap();
    assert!(matches!(
        plan.producer_completion(&f.profile, &profile, &f.witness, &mut f.ledger),
        Err(Error::NeedsRecovery)
    ));
    let mut input = f.input();
    f.refresh();
    f.publisher()
        .apply(&plan, &mut f.resources(), &mut input)
        .unwrap();
    let before = f.ledger.checkpoint();
    let held = plan
        .producer_completion(&f.profile, &profile, &f.witness, &mut f.ledger)
        .unwrap();
    assert_eq!(f.ledger.checkpoint(), before);
    let retired = c.retirement("proof-busy");
    assert!(matches!(
        c.execute(&retired, true, false),
        Err(retirement::Error::Protection(
            git_asset_store::store_protection::Error::Busy
        ))
    ));
    drop(held);
    let mut changed = profile.clone();
    changed.root = f.root.join("missing-profile-root");
    assert!(matches!(
        plan.producer_completion(&f.profile, &changed, &f.witness, &mut f.ledger),
        Err(Error::Stale)
    ));
    let mut legacy = plan.clone();
    legacy.effect_protocol = None;
    assert!(matches!(
        legacy.producer_completion(&f.profile, &profile, &f.witness, &mut f.ledger),
        Err(Error::Unsupported)
    ));
    assert!(!changed.root.exists());
}
#[test]
fn incomplete_original_native_producer_cannot_supply_eviction_closure() {
    let mut f = Fixture::new(b"unknown producer proof");
    let _control = Control::new(&mut f);
    let plan = f.plan.clone();
    fs::write(f.profile.pool.pool.repository.join("mode"), b"lost-reply").unwrap();
    let mut input = f.input();
    f.refresh();
    assert!(matches!(
        f.publisher().apply(&plan, &mut f.resources(), &mut input),
        Err(Error::Uncertain)
    ));
    assert!(matches!(
        plan.producer_completion(
            &f.profile,
            f.control_profile.as_ref().unwrap(),
            &f.witness,
            &mut f.ledger
        ),
        Err(Error::Uncertain)
    ));
}
