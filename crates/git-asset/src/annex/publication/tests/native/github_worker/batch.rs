pub(super) mod backup;
use super::*;
use crate::annex::{producer_index::Record, replication, snapshot::Snapshot};
pub(in super::super) fn pending_namespace(f: &Fixture) -> Namespace {
    pending_namespace_at(&f.root.join("pending-actions"))
}
pub(in super::super) fn pending_namespace_at(root: &Path) -> Namespace {
    let places = Places {
        current: vec![k("metadata")],
        pending: vec![k("metadata")],
        predecessor: vec![k("metadata")],
    };
    let policy = admission::Policy {
        prefer_active: true,
        classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
            .into_iter()
            .map(|k| (k, places.clone()))
            .collect(),
    };
    Namespace::create_fenced(root, k("pending"), k("epoch"), policy).unwrap()
}
pub(in super::super) fn pending_scope(
    f: &Fixture,
    namespace: &Namespace,
) -> crate::annex::pending::Scope {
    crate::annex::pending::Scope {
        namespace: namespace.identity().clone(),
        metadata: f.metadata.checkpoint().identity().clone(),
        owner: k("owner"),
        prefix: k("pending"),
        place: k("metadata"),
        allow_store: true,
    }
}

pub(super) fn index(f: &mut Fixture, plan: Plan, snapshot: Option<Snapshot>) -> Object {
    let mut metadata =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let (path, object) = Record::new(plan, snapshot)
        .unwrap()
        .store(&mut metadata)
        .unwrap();
    let (tree, _) = Tree::build(
        &BTreeMap::from([(path, object)]),
        &mut metadata,
        TreeLimits::default(),
    )
    .unwrap();
    let checkpoint = metadata.finish().unwrap();
    drop(metadata);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &checkpoint).unwrap();
    tree.root().clone()
}
pub(super) fn configured<'a, R: lifecycle::CommittedSource + ?Sized>(
    worker: &'a mut Execution<'_, R>,
    policy: &'a WorkerPolicy,
    trust: &'a github::Trust,
    request: &'a github::Verified,
) -> Execution<'a, R> {
    Execution {
        trust,
        request,
        policy,
        enrollment: worker.enrollment,
        repository: worker.repository,
        namespace: worker.namespace,
        metadata: worker.metadata,
        limits: worker.limits,
    }
}
pub(super) fn exercise(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    source: &mut Fixture,
    target: &mut Fixture,
    plan: &replication::Plan,
    project: &Path,
    signer: &github::fixture::Fixture,
) {
    let budget = github::placement::Budget {
        pages: 1,
        objects: 1,
        stream_bytes: 1024 * 1024,
        millis: 60000,
    };
    let mut dry_policy = worker.policy.clone();
    dry_policy.allow_copy = false;
    let mut dry_trust = worker.trust.clone();
    dry_trust.placement_enrollment = dry_policy.id().unwrap();
    let request = github::verify(
        &dry_trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    let report = configured(worker, &dry_policy, &dry_trust, &request)
        .reconcile(
            None,
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(report.outcomes.len(), 1);
    assert_eq!(report.outcomes[0].state, "planned");
    assert!(report.outcomes[0].hold.is_none());
    assert_eq!(report.outcomes[0].plan, Some(plan.id().unwrap()));
    assert!(
        target.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .current
            .is_none()
    );
    let tiny = github::placement::Budget {
        stream_bytes: 1,
        ..budget.clone()
    };
    let report = worker
        .reconcile(
            None,
            &tiny,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(
        report.outcomes[0].hold.as_ref().unwrap().code,
        "admission_denied"
    );
    assert!(
        target.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .current
            .is_none()
    );
    let mut missing_policy = worker.policy.clone();
    missing_policy.producers.as_mut().unwrap().source =
        Object::from_bytes(b"unavailable metadata root");
    let unchanged = worker.namespace.checkpoint();
    let trust = worker.trust.clone();
    let request = worker.request;
    assert!(matches!(
        configured(worker, &missing_policy, &trust, request).reconcile(
            None,
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Admission(github::Error::Trust))
    ));
    assert_eq!(unchanged, worker.namespace.checkpoint());
    let mut missing_trust = trust.clone();
    missing_trust.placement_enrollment = missing_policy.id().unwrap();
    let missing_request = github::verify(
        &missing_trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    let missing = configured(worker, &missing_policy, &missing_trust, &missing_request)
        .reconcile(
            None,
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(missing.outcomes[0].state, "held");
    assert_eq!(missing.outcomes[0].hold.as_ref().unwrap().code, "missing");
    assert!(missing.outcomes[0].plan.is_none());
    let mut concurrent = Namespace::open(
        &target.root.join("namespace-0"),
        &worker.namespace.checkpoint(),
    )
    .unwrap();
    let nodes = worker.metadata.finished_reader().unwrap();
    let mut calls = 0;
    let report = worker.reconcile(
        None,
        &budget,
        &mut source.resources(),
        &mut target.resources(),
        || {
            calls += 1;
            if calls == 2 {
                let source = concurrent.source(&k("creative")).unwrap().unwrap();
                assert!(matches!(
                    concurrent.apply(
                        k("batch-concurrent-withdraw"),
                        Command::Withdraw {
                            source: k("creative"),
                            owner: k("owner"),
                            expected_revision: source.revision,
                        },
                        &nodes,
                        &Fixture::limits()
                    ),
                    Err(admission::Error::Journal(
                        git_asset_store::source_claims::Error::Busy
                    ))
                ));
            }
            Ok(1000)
        },
    );
    let report = report.unwrap();
    assert_eq!(
        report.outcomes[0].state,
        "copied",
        "{}",
        serde_json::to_string(&report).unwrap()
    );
    assert_eq!(report.outcomes[0].plan, Some(plan.id().unwrap()));
    assert!(report.post_effect_hold.is_none());
    assert_eq!(report.storage_converged, None);
    let cursor = git_asset_store::placement::Cursor {
        basis: report.basis,
        after: plan.source().candidate().object.sha256.clone(),
    };
    let mut stale = cursor.clone();
    stale.basis = Digest::blob(b"other basis");
    assert!(matches!(
        worker.reconcile(
            Some(&stale),
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Admission(github::Error::Stale))
    ));
    let mut absent = cursor.clone();
    absent.after = Digest::blob(b"absent cursor");
    assert!(matches!(
        worker.reconcile(
            Some(&absent),
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Admission(github::Error::Stale))
    ));
    let suffix = worker
        .reconcile(
            Some(&cursor),
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert!(suffix.outcomes.is_empty());
    assert_eq!(suffix.storage_converged, None);
    // Exact Main bytes survive release of the temporary per-object ingest claim.
    retirement::settle(target);
    assert!(matches!(
        target
            .publisher()
            .reopen(plan.destination(), &mut target.resources()),
        Err(Error::ProtectionLost)
    ));
    let checkpoint = worker.namespace.checkpoint();
    let duplicate = worker
        .reconcile(
            None,
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(
        duplicate.outcomes[0].state,
        "present",
        "{}",
        serde_json::to_string(&duplicate).unwrap()
    );
    assert_eq!(worker.namespace.checkpoint(), checkpoint);
    // Unavailable retained custody must not become working absence or a new copy.
    let config = source
        .profile
        .pool
        .pool
        .repository
        .join(if source.profile.pool.pool.bare {
            "config"
        } else {
            ".git/config"
        });
    let offline = config.with_extension("offline");
    fs::rename(&config, &offline).unwrap();
    let held = worker
        .reconcile(
            None,
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(held.outcomes[0].state, "held");
    assert!(held.outcomes[0].hold.is_some());
    fs::rename(offline, config).unwrap();
    // A new concurrent job changes the pass basis without advancing Main.
    let known = &plan.source().candidate().object;
    let unknown = (0..)
        .map(|n| Object::from_bytes(format!("unregistered job {n}").as_bytes()))
        .find(|o| o.sha256 > known.sha256)
        .unwrap();
    let (tree, _) = Tree::build(
        &BTreeMap::from([(LogicalPath::parse("job/input").unwrap(), unknown)]),
        worker.metadata,
        TreeLimits::default(),
    )
    .unwrap();
    worker.metadata.finish().unwrap();
    let nodes = worker.metadata.finished_reader().unwrap();
    worker
        .namespace
        .apply(
            k("batch-job-enroll"),
            Command::Enroll {
                source: k("batch-job"),
                owner: k("owner"),
                class: Kind::Job,
            },
            &nodes,
            &Fixture::limits(),
        )
        .unwrap();
    worker
        .namespace
        .select(
            &admission::Select {
                prepare: k("batch-job-prepare"),
                adopt: k("batch-job-adopt"),
                source: k("batch-job"),
                owner: k("owner"),
                expected_revision: 1,
                selection: git_asset_store::source_claims::Selection {
                    root: tree.root().clone(),
                    context: Digest::blob(b"batch job"),
                },
            },
            &nodes,
            &Fixture::limits(),
        )
        .unwrap();
    assert!(matches!(
        worker.reconcile(
            Some(&cursor),
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Admission(github::Error::Stale))
    ));
    let first = worker
        .reconcile(
            None,
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(first.outcomes.len(), 1);
    assert_eq!(first.outcomes[0].state, "present");
    let next = first.next.unwrap();
    let second = worker
        .reconcile(
            Some(&next),
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(second.outcomes.len(), 1);
    assert_eq!(second.outcomes[0].hold.as_ref().unwrap().code, "missing");
    assert_eq!(second.storage_converged, None);
    // Post-observation admission drift preserves the completed present result.
    let mut calls = 0;
    let drift = worker
        .reconcile(
            None,
            &budget,
            &mut source.resources(),
            &mut target.resources(),
            || {
                calls += 1;
                if calls == 3 {
                    git(
                        project,
                        &[
                            "commit",
                            "--allow-empty",
                            "-m",
                            "head advances after observation",
                        ],
                    );
                }
                Ok(1000)
            },
        )
        .unwrap();
    assert_eq!(drift.outcomes[0].state, "present");
    assert_eq!(drift.post_effect_hold, Some(github::Error::Stale));
    assert_eq!(drift.selection.observed_ref_current, Some(false));
}
#[test]
fn authenticated_indexed_copy_dry_run_retry_handoff_offline_and_drift() {
    matrix(false, true, true, false, 1);
}
pub(super) fn uncertain(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    source: &mut Fixture,
    target: &mut Fixture,
    plan: &replication::Plan,
) {
    let mut copier = replication::Copier::new(worker.policy.copy.clone()).unwrap();
    assert!(
        copier
            .apply_hook(
                plan,
                &mut source.resources(),
                &mut target.resources(),
                |stage| {
                    if stage == Stage::Imported {
                        Err(Error::NeedsRecovery)
                    } else {
                        Ok(())
                    }
                }
            )
            .is_err()
    );
    let budget = github::placement::Budget {
        pages: 1,
        objects: 1,
        stream_bytes: 1024 * 1024,
        millis: 60000,
    };
    let calls = target.profile.pool.pool.repository.join("calls");
    let imports = || {
        fs::read_to_string(&calls)
            .unwrap_or_default()
            .lines()
            .filter(|s| s.contains("reinject"))
            .count()
    };
    let row = crate::annex::producer_index::Record::new(target.plan.clone(), None).unwrap();
    assert_eq!(
        row.with_completed_publication(
            plan.destination().clone(),
            &target.witness,
            &mut target.ledger
        )
        .unwrap_err(),
        Error::Uncertain
    );
    let before = imports();
    assert!(
        before > 0,
        "fixture must reach native import before interruption"
    );
    for _ in 0..2 {
        let report = worker
            .reconcile(
                None,
                &budget,
                &mut source.resources(),
                &mut target.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert_eq!(
            report.outcomes[0].state,
            "held",
            "{}",
            serde_json::to_string(&report).unwrap()
        );
        assert!(matches!(
            report.outcomes[0].hold.as_ref().unwrap().code.as_str(),
            "uncertain" | "needs_recovery"
        ));
        assert_eq!(report.outcomes[0].plan, Some(plan.id().unwrap()));
        assert_eq!(report.storage_converged, None);
    }
    assert_eq!(imports(), before);
}
#[test]
fn indexed_present_bytes_never_hide_original_uncertain_publication() {
    matrix(false, true, true, false, 2);
}
pub(super) fn existing(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    source: &mut Fixture,
    target: &mut Fixture,
    plan: &replication::Plan,
) {
    let budget = github::placement::Budget {
        pages: 1,
        objects: 1,
        stream_bytes: 1024 * 1024,
        millis: 60000,
    };
    let original = target.plan.clone();
    assert_ne!(original.id().unwrap(), plan.destination().id().unwrap());
    for released in [false, true] {
        if released {
            retirement::settle(target);
        }
        let report = worker
            .reconcile(
                None,
                &budget,
                &mut source.resources(),
                &mut target.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert_eq!(
            report.outcomes[0].state,
            "present",
            "{}",
            serde_json::to_string(&report).unwrap()
        );
        assert_eq!(report.outcomes[0].publication, Some(original.id().unwrap()));
        assert_eq!(report.outcomes[0].plan, Some(plan.id().unwrap()));
        assert!(report.outcomes[0].hold.is_none());
        assert_eq!(report.storage_converged, None);
    }
    let mut copier = replication::Copier::new(worker.policy.copy.clone()).unwrap();
    assert!(
        copier.reopen(plan, &mut target.resources()).is_err(),
        "an observation must not invent a successful copy attempt"
    );
}
#[test]
fn indexed_original_working_ingress_is_verified_without_copy_republication() {
    matrix(false, true, true, false, 3);
}
#[test]
#[ignore = "explicit native tools; synthetic pools only"]
fn actual_indexed_original_working_ingress_is_verified_without_copy_republication() {
    matrix(true, true, true, false, 3);
}
#[test]
#[ignore = "explicit native tools; synthetic pools only"]
fn actual_authenticated_indexed_copy_dry_run_retry_handoff_offline_and_drift() {
    matrix(true, true, true, false, 1);
}
