//! Connected native controller/facade over fresh synthetic pools only.
mod cli;
mod pipeline;
mod process;
mod race;
#[cfg(feature = "github")]
mod trusted;
use super::*;
use crate::annex::retirement::Assessor;
use crate::annex::retirement::planning::{
    ActorBoundary, Controller, EffectProfile, MetadataNamespace, Nodes, Plan as NativePlan,
    Producer, Profile as OwnerProfile, Request as EvictionRequest, Runtime, Sources,
    execution::{self, Cut},
};

fn demand(f: &mut Fixture, old_native: &Plan) {
    f.namespaces[0]
        .apply(
            k("enroll-recovery-job"),
            Command::Enroll {
                source: k("recovery-job"),
                owner: k("owner"),
                class: Kind::Job,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    f.namespaces[0]
        .select(
            &Select {
                prepare: k("prepare-recovery-job"),
                adopt: k("adopt-recovery-job"),
                source: k("recovery-job"),
                owner: k("owner"),
                expected_revision: 1,
                selection: old_native.selection.selection.clone(),
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
}

#[test]
#[ignore = "explicit native tools and distinct volume roots; fresh synthetic pools only"]
fn actual_native_eviction_retained_restore_and_stale_replay() {
    matrix(false, false, false);
}
fn matrix(exits: bool, races: bool, copies: bool) {
    matrix_mode(exits, races, copies, false);
}
fn matrix_mode(exits: bool, races: bool, copies: bool, trusted: bool) {
    matrix_profile(exits, races, copies, trusted, 0);
}
fn matrix_profile(exits: bool, races: bool, copies: bool, trusted: bool, batch: u8) {
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
    for (bare, cut) in [
        (false, Cut::BeforeEffect),
        (true, Cut::BeforeEffect),
        (false, Cut::None),
        (true, Cut::None),
        (false, Cut::AfterCompletion),
        (true, Cut::AfterCompletion),
        (false, Cut::AfterRemoval),
        (true, Cut::AfterRemoval),
    ] {
        if trusted && (bare || cut != Cut::None) {
            continue;
        }
        if (exits && cut == Cut::None)
            || (races && cut != Cut::BeforeEffect)
            || (copies && cut != Cut::None)
        {
            continue;
        }
        eprintln!(
            "native eviction case: bare={bare}, abrupt_exit={exits}, boundary={}",
            match cut {
                Cut::BeforeEffect => "before_effect",
                Cut::AfterRemoval => "after_removal",
                Cut::AfterCompletion => "after_completion",
                _ => "fresh",
            }
        );
        let bytes = vec![91; 256 * 1024];
        let mut f = Fixture::build_setup(
            &bytes,
            Some((git.clone(), annex.clone(), path.clone())),
            bare,
            Some(&source_root),
            trusted,
        );
        let mut backup = Fixture::build_in(
            &bytes,
            Some((git.clone(), annex.clone(), path.clone())),
            !bare,
            Some(&keeper_root),
        );
        let keeper_control = Control::build(&mut backup, true);
        let source_native = crate::annex::Reader::new(f.profile.pool.pool.clone())
            .unwrap()
            .store();
        let keeper_native = crate::annex::Reader::new(backup.profile.pool.pool.clone())
            .unwrap()
            .store();
        let effect = EffectProfile {
            root: f.root.join("witness"),
            store: f.witness.identity().clone(),
            ledger: f.root.join("ledger"),
            minimum: f.ledger.checkpoint(),
            job: f.plan.admission.job.clone(),
            owner: k("owner"),
        };
        let metadata_namespace_root = f.root.join("control-namespace");
        let keeper_profile = backup.control_profile.clone().unwrap();
        let mut enrolled = None;
        let mut source_control =
            Control::build_enrolled(&mut f, true, true, |source, job, namespace| {
                let policy = OwnerProfile {
                    source: source.clone(),
                    keeper: keeper_profile,
                    source_domain: k("synthetic-metadata-source"),
                    keeper_domain: k("keeper-domain"),
                    allow_evict: true,
                    unshared_direct_backing: true,
                    actor_boundary: ActorBoundary::NamespaceOwned,
                    runtime: Some(Runtime {
                        effect,
                        job: job.clone(),
                        namespaces: vec![MetadataNamespace {
                            root: metadata_namespace_root,
                            minimum: namespace.checkpoint(),
                        }],
                    }),
                };
                let authority = policy.admit(&source_native, &keeper_native).unwrap();
                enrolled = Some(policy);
                Some(authority)
            });
        let policy = enrolled.unwrap();
        let mut keeper_control = keeper_control;
        super::super::retirement::publish(&mut f);
        if copies {
            pipeline::copy(&mut f, &mut backup, &policy, &bytes);
        } else {
            super::super::retirement::publish(&mut backup);
        }
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
        #[cfg(feature = "github")]
        let mut trusted_state = trusted.then(|| trusted::State::new(&mut f, &backup, &policy));
        super::super::retirement::settle(&mut f);
        f.refresh();
        let mut prior_assessment = Assessor::new(f.profile.pool.clone())
            .unwrap()
            .assess(&object, &f.metadata, &Fixture::limits())
            .unwrap();
        let prior_source_read = prior_assessment.read_for_plan().unwrap();
        drop(prior_assessment);
        f.namespaces[0]
            .apply(
                k("advance-plan-basis"),
                Command::Enroll {
                    source: k("unrelated-source"),
                    owner: k("owner"),
                    class: Kind::Branch,
                },
                &f.metadata,
                &Fixture::limits(),
            )
            .unwrap();
        let mut assessed = Assessor::new(f.profile.pool.clone())
            .unwrap()
            .assess(&object, &f.metadata, &Fixture::limits())
            .unwrap();
        let mut source_read = Some(assessed.read_for_plan().unwrap());
        let kept = assessed
            .open_keeper(
                backup.profile.pool.clone(),
                &pin,
                &mut backup.metadata,
                TreeLimits::default(),
            )
            .unwrap();
        let separation = kept.compare_source_storage().unwrap();
        let source_proof = old_native
            .producer_completion(
                &f.profile,
                f.control_profile.as_ref().unwrap(),
                &f.witness,
                &mut f.ledger,
            )
            .unwrap();
        let keeper_proof = backup
            .plan
            .producer_completion(
                &backup.profile,
                backup.control_profile.as_ref().unwrap(),
                &backup.witness,
                &mut backup.ledger,
            )
            .unwrap();
        let before = (
            source_control.ledger.checkpoint(),
            keeper_control.ledger.checkpoint(),
        );
        let mut denied = policy.clone();
        denied.allow_evict = false;
        assert_eq!(
            separation
                .plan(
                    &denied,
                    k("native-eviction"),
                    source_read.as_ref().unwrap(),
                    &source_proof,
                    &keeper_proof,
                    grant.clone()
                )
                .err()
                .unwrap()
                .code(),
            "denied"
        );
        denied = policy.clone();
        denied.actor_boundary = ActorBoundary::MixedLegacy;
        assert_eq!(
            separation
                .plan(
                    &denied,
                    k("native-eviction"),
                    source_read.as_ref().unwrap(),
                    &source_proof,
                    &keeper_proof,
                    grant.clone()
                )
                .err()
                .unwrap()
                .code(),
            "unsupported"
        );
        assert_eq!(
            separation
                .plan(
                    &policy,
                    k("native-eviction"),
                    &prior_source_read,
                    &source_proof,
                    &keeper_proof,
                    grant.clone()
                )
                .err()
                .unwrap()
                .code(),
            "stale"
        );
        drop(prior_source_read);
        assert_eq!(
            separation
                .plan(
                    &policy,
                    k("native-eviction"),
                    source_read.as_ref().unwrap(),
                    &keeper_proof,
                    &source_proof,
                    grant.clone()
                )
                .err()
                .unwrap()
                .code(),
            "stale"
        );
        let mut changed_runtime = policy.clone();
        changed_runtime.runtime.as_mut().unwrap().effect.root =
            f.root.join("unadmitted-effect-root");
        assert_eq!(
            separation
                .plan(
                    &changed_runtime,
                    k("native-eviction"),
                    source_read.as_ref().unwrap(),
                    &source_proof,
                    &keeper_proof,
                    grant.clone()
                )
                .err()
                .unwrap()
                .code(),
            "stale"
        );
        assert!(
            !changed_runtime
                .runtime
                .as_ref()
                .unwrap()
                .effect
                .root
                .exists()
        );
        let mut bad = serde_json::to_value(&grant).unwrap();
        bad["sha256"] = serde_json::json!(Digest::blob(b"corrupt grant"));
        let bad = serde_json::from_value(bad).unwrap();
        assert_eq!(
            separation
                .plan(
                    &policy,
                    k("native-eviction"),
                    source_read.as_ref().unwrap(),
                    &source_proof,
                    &keeper_proof,
                    bad
                )
                .err()
                .unwrap()
                .code(),
            "corrupt"
        );
        let planned = separation
            .plan(
                &policy,
                k("native-eviction"),
                source_read.as_ref().unwrap(),
                &source_proof,
                &keeper_proof,
                grant,
            )
            .unwrap();
        let encoded = planned.to_json().unwrap();
        assert_eq!(
            NativePlan::from_json(&encoded).unwrap().id().unwrap(),
            planned.id().unwrap()
        );
        let mut invalid = encoded.clone();
        invalid.push(b'\n');
        assert!(NativePlan::from_json(&invalid).is_err());
        assert_eq!(planned.object(), &object);
        assert_eq!(
            before,
            (
                source_control.ledger.checkpoint(),
                keeper_control.ledger.checkpoint()
            )
        );
        let retired = planned.metadata_plan().clone();
        let original_control = source_control.plan.clone();
        let nodes = MemoryNodes::default();
        // Public controller must assemble the same proofs without manual orchestration.
        drop(source_proof);
        drop(keeper_proof);
        drop(source_read.take());
        drop(separation);
        drop(kept);
        drop(assessed);
        let controller = Controller::new(&policy);
        macro_rules! sources {
            () => {
                Sources {
                    source: Producer {
                        profile: &f.profile,
                        plan: &old_native,
                        witness: &f.witness,
                        ledger: &mut f.ledger,
                    },
                    keeper: Producer {
                        profile: &backup.profile,
                        plan: &backup.plan,
                        witness: &backup.witness,
                        ledger: &mut backup.ledger,
                    },
                }
            };
        }
        let mut denied = policy.clone();
        denied.allow_evict = false;
        assert_eq!(
            Controller::new(&denied)
                .plan(
                    EvictionRequest {
                        operation: k("native-eviction"),
                        snapshot: pin.clone(),
                        grant: planned.metadata_plan().request().keepers[0].grant.clone()
                    },
                    sources!(),
                    Nodes {
                        source: &f.metadata,
                        keeper: &mut backup.metadata,
                        limits: &Fixture::limits(),
                        tree_limits: TreeLimits::default()
                    },
                )
                .err()
                .unwrap()
                .code(),
            "denied"
        );
        let public_preview = controller
            .plan(
                EvictionRequest {
                    operation: k("native-eviction"),
                    snapshot: pin.clone(),
                    grant: planned.metadata_plan().request().keepers[0].grant.clone(),
                },
                sources!(),
                Nodes {
                    source: &f.metadata,
                    keeper: &mut backup.metadata,
                    limits: &Fixture::limits(),
                    tree_limits: TreeLimits::default(),
                },
            )
            .unwrap();
        assert_eq!(public_preview.id().unwrap(), planned.id().unwrap());
        #[cfg(feature = "github")]
        if batch != 0 {
            trusted_state.as_mut().unwrap().execute_batch(
                &mut f,
                &mut backup,
                &planned,
                batch == 2,
                batch == 3,
            );
            continue;
        }
        #[cfg(not(feature = "github"))]
        let _ = batch;
        let cli = if trusted {
            None
        } else {
            cli::Client::new(&f, &backup, &policy, &planned, &pin)
        };
        let result = if exits {
            if races {
                cli.as_ref()
                    .unwrap()
                    .race(|| race::check(&mut f, &old_native, &mut source_control));
            } else {
                cli.as_ref().unwrap().exit(cut);
            }
            None
        } else {
            Some(if let (Cut::None, Some(cli)) = (cut, &cli) {
                Ok(cli.call("apply", true)["report"].clone())
            } else if cut == Cut::None && trusted {
                #[cfg(feature = "github")]
                {
                    Ok(
                        serde_json::to_value(trusted_state.as_mut().unwrap().execute(
                            &mut f,
                            &mut backup,
                            &planned,
                        ))
                        .unwrap(),
                    )
                }
                #[cfg(not(feature = "github"))]
                {
                    unreachable!("trusted fixture requires github")
                }
            } else if cut == Cut::None {
                controller
                    .apply(
                        &planned,
                        sources!(),
                        Nodes {
                            source: &f.metadata,
                            keeper: &mut backup.metadata,
                            limits: &Fixture::limits(),
                            tree_limits: TreeLimits::default(),
                        },
                    )
                    .map(|r| serde_json::to_value(r).unwrap())
            } else {
                controller
                    .apply_hook(
                        &planned,
                        sources!(),
                        Nodes {
                            source: &f.metadata,
                            keeper: &mut backup.metadata,
                            limits: &Fixture::limits(),
                            tree_limits: TreeLimits::default(),
                        },
                        cut,
                    )
                    .map(|r| serde_json::to_value(r).unwrap())
            })
        };
        if cut == Cut::None {
            let report = result.unwrap().unwrap();
            assert_eq!(report["historical"], false);
            assert_eq!(report["metadata"]["cancelled"], false);
            assert_eq!(report["logical_subject_bytes"], object.bytes);
            assert!(report["source_allocation_before"].is_u64());
        } else {
            if let Some(result) = result {
                assert_eq!(
                    result.err().unwrap().code(),
                    if cut == Cut::AfterRemoval {
                        "uncertain"
                    } else {
                        "needs_recovery"
                    }
                );
            }
            assert!(matches!(
                control::Profile::open(
                    f.control_profile.as_ref().unwrap(),
                    &source_native,
                    &object,
                    &original_control
                ),
                Err(Error::NeedsRecovery)
            ));
        }
        let before_status = (
            CapacityLedger::open(&policy.source.ledger, &policy.source.minimum)
                .unwrap()
                .checkpoint(),
            CapacityLedger::open(
                &policy.runtime.as_ref().unwrap().effect.ledger,
                &policy.runtime.as_ref().unwrap().effect.minimum,
            )
            .unwrap()
            .checkpoint(),
        );
        let status = controller
            .status(&planned, &f.profile, &old_native)
            .unwrap();
        assert_eq!(
            status.effect,
            match cut {
                Cut::BeforeEffect => "not_started",
                Cut::AfterRemoval => "uncertain",
                _ => "completed",
            }
        );
        if let Some(cli) = &cli {
            assert_eq!(
                cli.call("status", true)["report"],
                serde_json::to_value(&status).unwrap()
            );
        }
        assert!(!status.removal_authority);
        assert!(if cut == Cut::BeforeEffect {
            matches!(status.observed, execution::Observation::VerifiedPresent)
        } else {
            matches!(status.observed, execution::Observation::ObservedAbsent)
        });
        assert_eq!(
            before_status,
            (
                CapacityLedger::open(&policy.source.ledger, &policy.source.minimum)
                    .unwrap()
                    .checkpoint(),
                CapacityLedger::open(
                    &policy.runtime.as_ref().unwrap().effect.ledger,
                    &policy.runtime.as_ref().unwrap().effect.minimum
                )
                .unwrap()
                .checkpoint()
            )
        );
        if cut == Cut::BeforeEffect {
            demand(&mut f, &old_native);
            if let Some(cli) = &cli {
                assert_eq!(
                    cli.call("cancel", true)["report"]["metadata"]["cancelled"],
                    true
                );
            }
            let result = controller
                .cancel(
                    &planned,
                    &f.profile,
                    &old_native,
                    &nodes,
                    &Fixture::limits(),
                )
                .unwrap();
            assert!(result.metadata.cancelled);
            let again = controller
                .cancel(
                    &planned,
                    &f.profile,
                    &old_native,
                    &nodes,
                    &Fixture::limits(),
                )
                .unwrap();
            assert!(again.metadata.cancelled);
            let history = controller
                .resume(
                    &planned,
                    &f.profile,
                    &old_native,
                    &nodes,
                    &Fixture::limits(),
                )
                .unwrap();
            assert!(history.historical && history.metadata.cancelled);
            assert!(
                control::Profile::open(
                    f.control_profile.as_ref().unwrap(),
                    &source_native,
                    &object,
                    &original_control
                )
                .is_ok()
            );
            let mut kept = Vec::new();
            crate::annex::managed::Reader::new(f.profile.pool.clone())
                .unwrap()
                .open(&object)
                .unwrap()
                .read_to_end(&mut kept)
                .unwrap();
            assert_eq!(kept, bytes);
            continue;
        }
        assert_eq!(
            controller
                .cancel(
                    &planned,
                    &f.profile,
                    &old_native,
                    &nodes,
                    &Fixture::limits()
                )
                .err()
                .unwrap()
                .code(),
            "uncertain"
        );
        if let Some(cli) = &cli {
            let denied = cli.call("cancel", false);
            assert_eq!(denied["code"], "uncertain");
            assert_eq!(denied["retry"], "inspect_original_operation");
        }
        if cut == Cut::AfterRemoval {
            if let Some(cli) = &cli {
                assert_eq!(cli.call("resume", false)["code"], "uncertain");
            }
            for _ in 0..2 {
                assert_eq!(
                    controller
                        .resume(
                            &planned,
                            &f.profile,
                            &old_native,
                            &nodes,
                            &Fixture::limits()
                        )
                        .err()
                        .unwrap()
                        .code(),
                    "uncertain"
                );
            }
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
        // A new raw job demand after process-boundary guard release must not block
        // historical finalization or cause a repeated removal.
        demand(&mut f, &old_native);
        assert_eq!(
            Assessor::new(f.profile.pool.clone())
                .unwrap()
                .assess(&object, &f.metadata, &Fixture::limits())
                .err()
                .unwrap()
                .code(),
            "protected"
        );
        let report = controller
            .resume(
                &planned,
                &f.profile,
                &old_native,
                &nodes,
                &Fixture::limits(),
            )
            .unwrap();
        assert!(report.historical && !report.metadata.cancelled);
        let again = controller
            .resume(
                &planned,
                &f.profile,
                &old_native,
                &nodes,
                &Fixture::limits(),
            )
            .unwrap();
        assert!(again.metadata.historical_replay);
        if let Some(cli) = &cli {
            assert_eq!(cli.call("resume", true)["report"]["historical"], true);
        }
        source_control.ledger =
            CapacityLedger::open(&policy.source.ledger, &policy.source.minimum).unwrap();
        let mut retained_bytes = Vec::new();
        crate::annex::Reader::new(backup.profile.pool.pool.clone())
            .unwrap()
            .open(&object)
            .unwrap()
            .read_to_end(&mut retained_bytes)
            .unwrap();
        assert_eq!(retained_bytes, bytes);

        // Restore uses a new native request and an explicit new control incarnation.
        if copies {
            f.plan = pipeline::restore(&mut f, &mut backup, &policy, &planned, &old_native);
        } else {
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
            f.ledger = CapacityLedger::open(
                &f.root.join("ledger"),
                &policy.runtime.as_ref().unwrap().effect.minimum,
            )
            .unwrap();
            let mut publisher = f.publisher();
            let mut input =
                crate::annex::managed::Reader::new(backup.profile.pool.clone()).unwrap();
            let mut restored = publisher
                .apply(&plan, &mut f.resources(), &mut input)
                .unwrap();
            let mut actual = Vec::new();
            restored.read_to_end(&mut actual).unwrap();
            assert_eq!(actual, bytes);
            drop(restored);
        }
        // Old completion is explicitly historical, and cannot remove this same-hash birth.
        let historical = controller
            .resume(
                &planned,
                &f.profile,
                &old_native,
                &nodes,
                &Fixture::limits(),
            )
            .unwrap();
        assert!(
            historical.historical
                && matches!(historical.observed, execution::Observation::VerifiedPresent)
        );
        let mut current = Vec::new();
        crate::annex::Reader::new(f.profile.pool.pool.clone())
            .unwrap()
            .open(&object)
            .unwrap()
            .read_to_end(&mut current)
            .unwrap();
        assert_eq!(current, bytes);
    }
}

#[test]
#[ignore = "fresh facade executable plus native tools and separate volume roots"]
fn actual_native_storage_facade_ordinary_bare_lifecycle() {
    assert!(std::env::var_os("GIT_ASSET_TEST_FRONT_DOOR_EXE").is_some());
    actual_native_eviction_retained_restore_and_stale_replay();
}

#[test]
#[ignore = "fresh facade executable, native tools and distinct volumes; actual child exits"]
fn actual_native_executor_process_exits_preserve_original_operation() {
    assert!(std::env::var_os("GIT_ASSET_TEST_FRONT_DOOR_EXE").is_some());
    matrix(true, false, false);
}

#[test]
#[ignore = "fresh facade/native tools and separate volumes; actual paused actor races"]
fn actual_native_executor_fences_conflicts_and_allows_unrelated_metadata() {
    assert!(std::env::var_os("GIT_ASSET_TEST_FRONT_DOOR_EXE").is_some());
    matrix(true, true, false);
}

#[test]
#[ignore = "fresh native facade/tools and distinct volumes; controlled copy through restore"]
fn actual_native_controlled_copy_retained_eviction_and_restore_pipeline() {
    assert!(std::env::var_os("GIT_ASSET_TEST_FRONT_DOOR_EXE").is_some());
    matrix(false, false, true);
}

#[cfg(feature = "github")]
#[test]
#[ignore = "explicit native tools and distinct synthetic volumes only"]
fn actual_trusted_git_eviction_rechecks_expiry_after_custody() {
    matrix_mode(false, false, false, true);
}

#[cfg(feature = "github")]
#[test]
#[ignore = "explicit native tools and distinct synthetic volumes only"]
fn actual_registered_eviction_preview_protection_expiry_and_saved_replay() {
    matrix_profile(false, false, false, true, 1);
}

#[cfg(feature = "github")]
#[test]
#[ignore = "explicit native tools and distinct synthetic volumes only"]
fn actual_registered_eviction_uncertain_saved_intent_never_repeats_removal() {
    matrix_profile(false, false, false, true, 2);
}

#[cfg(feature = "github")]
#[test]
#[ignore = "explicit native tools and distinct synthetic volumes only"]
fn actual_named_producer_heads_eviction_protection_expiry_and_saved_replay() {
    matrix_profile(false, false, false, true, 3);
}
