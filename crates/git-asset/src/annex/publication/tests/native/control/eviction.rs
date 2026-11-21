//! Connected synthetic proof, deliberately not a public removal API.
use super::*;
use crate::annex::{process, retirement::Assessor};
use git_asset_store::upload_witness::{Document, Packet};

fn canonical(value: serde_json::Value) -> Vec<u8> {
    git_asset_core::identity::canonical_json(&value).unwrap()
}

/// Open the actual enrolled metadata root internally, never a supplied same-ID clone.
fn execute(
    f: &Fixture,
    source: &mut Control,
    keeper: &mut Control,
    plan: &retirement::Plan,
    effect: impl FnMut(&retirement::Plan) -> retirement::Result<()>,
) -> retirement::Result<retirement::Receipt> {
    let profile = f.control_profile.as_ref().unwrap();
    let native = crate::annex::Reader::new(f.profile.pool.pool.clone())
        .unwrap()
        .store();
    control::configuration(
        &f.profile.pool.pool,
        &f.profile.pool.binding.id().unwrap(),
        Some(&profile.binding(&native).unwrap()),
    )
    .unwrap();
    let store = LocalStore::open(&profile.root)?;
    assert_eq!(store.identity(), &profile.store);
    assert_eq!(store.place(), &control::place(&native).unwrap());
    let mut backups = [retirement::Backup {
        store: &keeper.store,
        ledger: &mut keeper.ledger,
        enrollment: &plan.request().keepers[0].enrollment,
    }];
    let mut execution = retirement::Execution {
        store: &store,
        ledger: &mut source.ledger,
        job: &source.job,
        namespaces: std::slice::from_mut(&mut source.namespace),
        nodes: &MemoryNodes::default(),
        limits: &Fixture::limits(),
        backups: &mut backups,
        admitted_context: &plan.request().context,
        source_enrollment: &plan.request().source,
    };
    execution.apply_with_effect(plan, &plan.authorize()?, effect)
}

#[test]
#[ignore = "explicit native tools and distinct volume roots; fresh synthetic pools only"]
fn actual_native_eviction_retained_restore_and_stale_replay() {
    let git: PathBuf = std::env::var_os("GIT_ASSET_TEST_GIT_EXE").unwrap().into();
    let annex: PathBuf = std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE").unwrap().into();
    let path = std::env::split_paths(&std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").unwrap())
        .collect::<Vec<_>>();
    let source_root: PathBuf = std::env::var_os("GIT_ASSET_TEST_SOURCE_VOLUME_ROOT")
        .unwrap()
        .into();
    let keeper_root: PathBuf = std::env::var_os("GIT_ASSET_TEST_KEEPER_VOLUME_ROOT")
        .unwrap()
        .into();
    for (bare, uncertain) in [(false, false), (true, false), (false, true), (true, true)] {
        let bytes = vec![91; 256 * 1024];
        let mut f = Fixture::build_in(
            &bytes,
            Some((git.clone(), annex.clone(), path.clone())),
            bare,
            Some(&source_root),
        );
        let mut backup = Fixture::build_in(
            &bytes,
            Some((git.clone(), annex.clone(), path.clone())),
            !bare,
            Some(&keeper_root),
        );
        let mut source_control = Control::build_reacquirable(&mut f, true, true);
        let mut keeper_control = Control::build(&mut backup, true);
        super::super::retirement::publish(&mut f);
        super::super::retirement::publish(&mut backup);
        let old_native = f.plan.clone();
        let object = old_native.candidate().object.clone();
        let pin = super::super::retirement::snapshot(&mut backup);

        // Real native bytes and genuine metadata documents have separate custody.
        let retained = retention::plan(
            &keeper_control.store,
            &retention::RetentionRequest {
                operation: k("retain-controller-copy"),
                owner: k("controller"),
                publication: source_control.plan.publication.request().clone(),
                minimum_ack: git_asset_store::AckLevel::ProcessRestart,
            },
        )
        .unwrap();
        let grant = retention::retain(
            &mut keeper_control.ledger,
            &keeper_control.store,
            &keeper_control.job,
            &retained,
        )
        .unwrap()
        .grant;
        super::super::retirement::settle(&mut f);
        f.refresh();
        let mut assessed = Assessor::new(f.profile.pool.clone())
            .unwrap()
            .assess(&object, &f.metadata, &Fixture::limits())
            .unwrap();
        let mut source_read = Some(assessed.read_source().unwrap());
        let kept = assessed
            .open_keeper(
                backup.profile.pool.clone(),
                &pin,
                &mut backup.metadata,
                TreeLimits::default(),
            )
            .unwrap();
        let separation = kept.compare_source_storage().unwrap();
        // Explicit synthetic namespace-owned/direct-backing admission is not owner approval.
        let context = Digest::blob(&canonical(serde_json::json!([
            "git-asset.synthetic-native-eviction.v1",
            assessed.replay_basis().unwrap(),
            separation.id().unwrap(),
            "owned-fresh-namespace-only",
            "unshared-direct-backing",
            "source-domain",
            "keeper-domain",
            f.control_profile
                .as_ref()
                .unwrap()
                .binding(&assessed.store())
                .unwrap()
        ])));
        let retired = retirement::plan(
            &source_control.store,
            retirement::Request {
                operation: k("native-eviction"),
                owner: k("controller"),
                publication: source_control.plan.publication.request().clone(),
                context,
                source: source_control.enrollment(),
                keepers: vec![retirement::Keeper {
                    grant,
                    enrollment: ReplicaEnrollment {
                        store: keeper_control.store.identity().clone(),
                        failure_domain: k("keeper-domain"),
                    },
                }],
                required_domains: 1,
                capacity: source_control.ledger.identity().clone(),
            },
        )
        .unwrap();
        // Plan identity includes the original metadata incarnation, not just the hash.
        let effect_id = retired.id().unwrap();
        let packet = Packet::new(&f.witness, &effect_id, &k("owner"), ["intent", "completion"].map(|phase| Document {
            operation: k(&format!("native-{phase}-{}", effect_id.as_str())),
            bytes: canonical(serde_json::json!({"schema":"git-asset.synthetic-native-effect.v1","plan":effect_id,"phase":phase,"object":object})),
        })).unwrap();
        let native_key = format!("SHA256-s{}--{}", object.bytes, object.sha256.as_str());
        let job = f.plan.admission.job.clone();
        let mut effects = 0;
        let witness = LocalStore::open(&f.root.join("witness")).unwrap();
        // Separate native witness ledger is already admitted by this owned fixture.
        let mut effect_ledger =
            CapacityLedger::open(&f.root.join("ledger"), &f.ledger.checkpoint()).unwrap();
        let original_control = source_control.plan.clone();
        assert!(
            control::Profile::open(
                f.control_profile.as_ref().unwrap(),
                &assessed.store(),
                &object,
                &original_control
            )
            .is_ok()
        );
        let interrupted = execute(
            &f,
            &mut source_control,
            &mut keeper_control,
            &retired,
            |_| {
                packet
                    .apply_once(&witness, &mut effect_ledger, &job, || {
                        separation.revalidate().unwrap();
                        // Pending original metadata denies native ingress before its data lookup.
                        assert!(matches!(
                            control::Profile::open(
                                f.control_profile.as_ref().unwrap(),
                                &assessed.store(),
                                &object,
                                &original_control
                            ),
                            Err(Error::NeedsRecovery)
                        ));
                        drop(source_read.take());
                        effects += 1;
                        let group = process::run_for_object(
                            &f.profile.pool.pool,
                            &object,
                            true,
                            &[
                                "dropkey",
                                "--force",
                                "--json",
                                "--json-error-messages",
                                &native_key,
                            ],
                        )
                        .unwrap();
                        let response = git_asset_core::identity::parse_json(
                            crate::annex::line(&group.stdout).unwrap().as_bytes(),
                        )
                        .unwrap();
                        assert_eq!(response["command"], "dropkey");
                        assert_eq!(response["key"], native_key);
                        assert_eq!(response["success"], true);
                        assert!(matches!(
                            process::run(
                                &f.profile.pool.pool,
                                true,
                                &["contentlocation", &native_key]
                            ),
                            Err(crate::annex::Error::Missing)
                        ));
                        separation.revalidate().unwrap();
                        if uncertain {
                            // Effect happened, but no durable completion: it must never be repeated.
                            return Err(git_asset_store::upload_witness::Error::NeedsRecovery);
                        }
                        Ok(group)
                    })
                    .map_err(|_| git_asset_store::Error::NeedsRecovery)?;
                // Simulate interruption after durable native completion, before metadata unlink.
                Err(git_asset_store::Error::NeedsRecovery.into())
            },
        );
        assert!(matches!(
            interrupted,
            Err(retirement::Error::Store(
                git_asset_store::Error::NeedsRecovery
            ))
        ));
        assert!(matches!(
            control::Profile::open(
                f.control_profile.as_ref().unwrap(),
                &assessed.store(),
                &object,
                &original_control
            ),
            Err(Error::NeedsRecovery)
        ));
        if uncertain {
            let state = packet.inspect(&witness, &mut effect_ledger, &job).unwrap();
            assert!(state.intent && !state.completion);
            let repeated = packet.apply_once(
                &witness,
                &mut effect_ledger,
                &job,
                || -> git_asset_store::upload_witness::Result<()> {
                    panic!("uncertain native deletion must never be retried")
                },
            );
            assert!(matches!(
                repeated,
                Err(git_asset_store::upload_witness::OnceError::Uncertain)
            ));
            assert_eq!(effects, 1);
            drop(separation);
            drop(kept);
            drop(assessed);
            let mut surviving = Vec::new();
            crate::annex::managed::Reader::new(backup.profile.pool.clone())
                .unwrap()
                .open(&object)
                .unwrap()
                .read_to_end(&mut surviving)
                .unwrap();
            assert_eq!(surviving, bytes);
            continue;
        }
        // Reopen the exact witnesses after the simulated interruption; never repeat dropkey.
        let receipt = execute(
            &f,
            &mut source_control,
            &mut keeper_control,
            &retired,
            |_| {
                packet
                    .apply_once(
                        &witness,
                        &mut effect_ledger,
                        &job,
                        || -> git_asset_store::upload_witness::Result<()> {
                            panic!("completed native command repeated on recovery")
                        },
                    )
                    .map_err(|_| git_asset_store::Error::NeedsRecovery)?;
                Ok(())
            },
        )
        .unwrap();
        assert!(!receipt.cancelled);
        assert_eq!(effects, 1);
        assert!(matches!(
            process::run(
                &f.profile.pool.pool,
                true,
                &["contentlocation", &native_key]
            ),
            Err(crate::annex::Error::Missing)
        ));
        packet.reopen(&witness, &mut effect_ledger, &job).unwrap();
        let historical = execute(
            &f,
            &mut source_control,
            &mut keeper_control,
            &retired,
            |_| panic!("completed native effect repeated"),
        )
        .unwrap();
        assert!(historical.historical_replay);
        // Reopen surviving media independently of the source metadata document.
        drop(separation);
        drop(kept);
        drop(assessed);
        let mut retained_bytes = Vec::new();
        crate::annex::Reader::new(backup.profile.pool.pool.clone())
            .unwrap()
            .open(&object)
            .unwrap()
            .read_to_end(&mut retained_bytes)
            .unwrap();
        assert_eq!(retained_bytes, bytes);

        // Restore uses a new native request and an explicit new control incarnation.
        let mut request = old_native.request.clone();
        request.operation = k("restore-native-version");
        request.expected_revision = f.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .revision;
        let mut metadata =
            MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
        let frozen = freeze(&request, &mut metadata, TreeLimits::default()).unwrap();
        let checkpoint = metadata.finish().unwrap();
        drop(metadata);
        f.metadata = MetadataReader::open(&f.root.join("metadata"), &checkpoint).unwrap();
        let mut control_request = source_control.plan.publication.request().clone();
        control_request.operation = k("restore-control-version");
        let reacquire = retirement::plan_reacquisition(
            &source_control.store,
            &retired,
            control_request,
            control::writer_context(&f.profile, &request).unwrap(),
        )
        .unwrap();
        retirement::Reacquisition {
            store: &source_control.store,
            ledger: &mut source_control.ledger,
            job: &source_control.job,
        }
        .apply(
            &reacquire,
            &reacquire.authorize().unwrap(),
            &mut Cursor::new(control::document(&object).unwrap()),
        )
        .unwrap();
        let restored_control = control::Plan {
            publication: reacquire.payload_plan().clone(),
            job: source_control.job.clone(),
        };
        f.plan = Plan::controlled(
            &f.profile,
            request,
            frozen,
            old_native.admission.clone(),
            f.control_profile.as_ref().unwrap(),
            restored_control,
        )
        .unwrap();
        // Make original input unavailable: restore consumes the retained native bytes.
        fs::remove_file(f.root.join("input/objects").join(object.sha256.as_str())).unwrap();
        let plan = f.plan.clone();
        f.refresh();
        f.ledger =
            CapacityLedger::open(&f.root.join("ledger"), &effect_ledger.checkpoint()).unwrap();
        let mut publisher = f.publisher();
        let mut input = crate::annex::managed::Reader::new(backup.profile.pool.clone()).unwrap();
        let mut restored = publisher
            .apply(&plan, &mut f.resources(), &mut input)
            .unwrap();
        let mut actual = Vec::new();
        restored.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, bytes);
        drop(restored);
        // Old completion is explicitly historical, and cannot remove this same-hash birth.
        let historical = execute(
            &f,
            &mut source_control,
            &mut keeper_control,
            &retired,
            |_| panic!("stale plan deleted restored bytes"),
        )
        .unwrap();
        assert!(historical.historical_replay);
        let mut current = Vec::new();
        crate::annex::Reader::new(f.profile.pool.pool.clone())
            .unwrap()
            .open(&object)
            .unwrap()
            .read_to_end(&mut current)
            .unwrap();
        assert_eq!(current, bytes);
        assert_eq!(effects, 1);
    }
}
