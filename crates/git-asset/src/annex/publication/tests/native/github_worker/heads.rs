mod backup;
use super::*;
use crate::annex::{
    pending::Scope,
    producer_index::{Index, Record, update as roots},
};
use crate::github::native::producers::{Head, Heads};

pub(in super::super) struct Registered {
    pub(in super::super) head: Head,
    profile: roots::Profile,
    namespace: Namespace,
    nodes: MetadataStore,
    row: Record,
    version: git_asset_store::source_claims::Version,
}
pub(in super::super) fn register(f: &mut Fixture, root: Object, name: &str) -> Registered {
    let row = Index::from_root(root)
        .get(
            &f.plan.candidate().object,
            &mut f.metadata,
            TreeLimits::default(),
        )
        .unwrap()
        .0
        .unwrap();
    let places = Places {
        current: vec![k("metadata")],
        pending: vec![k("metadata")],
        predecessor: vec![k("metadata")],
    };
    let path = f.root.join(name);
    let mut namespace = Namespace::create_fenced(
        &path,
        k(name),
        k("epoch"),
        admission::Policy {
            prefer_active: true,
            classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                .into_iter()
                .map(|c| (c, places.clone()))
                .collect(),
        },
    )
    .unwrap();
    let mut nodes = MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let profile = roots::Profile {
        scope: Scope {
            namespace: namespace.identity().clone(),
            metadata: nodes.checkpoint().identity().clone(),
            owner: k("owner"),
            prefix: k("producers"),
            place: k("metadata"),
            allow_store: true,
        },
        limits: Fixture::limits(),
    };
    namespace
        .apply(
            k("enroll-root"),
            Command::Enroll {
                source: k("producers"),
                owner: k("owner"),
                class: Kind::Job,
            },
            &nodes,
            &Fixture::limits(),
        )
        .unwrap();
    let plan = roots::prepare(
        &profile,
        None,
        &[roots::Change {
            expected_record: None,
            record: row.clone(),
        }],
        &mut namespace,
        &mut nodes,
    )
    .unwrap()
    .unwrap();
    let result = roots::apply(&profile, &plan, &mut namespace, &mut nodes, || Ok(())).unwrap();
    let mut readonly = profile.clone();
    readonly.scope.allow_store = false;
    let head = Head {
        root: path,
        minimum: namespace.checkpoint(),
        profile: readonly,
    };
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &nodes.checkpoint()).unwrap();
    Registered {
        head,
        profile,
        namespace,
        nodes,
        row,
        version: result.version,
    }
}
pub(super) fn exercise(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    m: &mut Fixture,
    f: &mut Fixture,
    signer: &github::fixture::Fixture,
) {
    let old = worker.policy.producers.as_ref().unwrap();
    let mut source = register(m, old.source.clone(), "retained-producers");
    let mut destination = register(f, old.destination.clone(), "working-producers");
    destination.head.profile.scope.allow_store = true;
    let mut pending = batch::pending_namespace_at(&f.root.join("working-publication-actions"));
    let mut policy = worker.policy.clone();
    policy.actions.as_mut().unwrap().registration = true;
    policy.actions.as_mut().unwrap().pending = Some(batch::pending_scope(f, &pending));
    policy.producers = None;
    policy.producer_heads = Some(Heads {
        source: source.head.clone(),
        destination: destination.head.clone(),
    });
    let mut ambiguous = policy.clone();
    ambiguous.producers = Some(old.clone());
    assert!(ambiguous.id().is_err());
    let budget = github::placement::Budget {
        pages: 1,
        objects: 1,
        stream_bytes: 1024 * 1024,
        millis: 60000,
    };
    let mut dry = policy.clone();
    dry.allow_copy = false;
    let mut trust = worker.trust.clone();
    trust.placement_enrollment = dry.id().unwrap();
    let request = github::verify(
        &trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    let first = batch::configured(worker, &dry, &trust, &request)
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(first.outcomes[0].state, "planned");
    assert_eq!(
        first.producer_heads.as_ref().unwrap().source.version,
        source.version
    );
    assert_eq!(
        first.producer_heads.as_ref().unwrap().destination.version,
        destination.version
    );
    let cursor = git_asset_store::placement::Cursor {
        basis: first.basis.clone(),
        after: source.row.object.sha256.clone(),
    };
    let id = dry.id().unwrap();
    // The new retained member changes an index Version, not trusted enrollment.
    let mut updated = source.row.clone();
    updated.retained = Some(retirement::snapshot(m));
    let plan = roots::prepare(
        &source.profile,
        Some(&source.version),
        &[roots::Change {
            expected_record: Some(source.row.metadata_object().unwrap()),
            record: updated.clone(),
        }],
        &mut source.namespace,
        &mut source.nodes,
    )
    .unwrap()
    .unwrap();
    let advanced = roots::apply(
        &source.profile,
        &plan,
        &mut source.namespace,
        &mut source.nodes,
        || Ok(()),
    )
    .unwrap();
    assert_eq!(id, dry.id().unwrap());
    source.version = advanced.version;
    m.metadata =
        MetadataReader::open(&m.root.join("metadata"), &source.nodes.checkpoint()).unwrap();
    assert!(
        batch::configured(worker, &dry, &trust, &request)
            .reconcile(
                Some(&cursor),
                &budget,
                &mut m.resources(),
                &mut f.resources(),
                || Ok(1000)
            )
            .is_err()
    );
    let fresh = batch::configured(worker, &dry, &trust, &request)
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(fresh.outcomes[0].state, "planned");
    assert_ne!(fresh.basis, first.basis);
    assert_eq!(
        fresh.producer_heads.as_ref().unwrap().source.version,
        source.version
    );
    // Explicit head enrollment never falls back to the old static index.
    let checkpoint = worker.namespace.checkpoint();
    let mut missing = dry.clone();
    missing.producer_heads.as_mut().unwrap().source.root = source.head.root.join("missing");
    let mut missing_trust = trust.clone();
    missing_trust.placement_enrollment = missing.id().unwrap();
    let missing_request = github::verify(
        &missing_trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    assert!(
        batch::configured(worker, &missing, &missing_trust, &missing_request)
            .reconcile(
                None,
                &budget,
                &mut m.resources(),
                &mut f.resources(),
                || Ok(1000)
            )
            .is_err()
    );
    assert_eq!(checkpoint, worker.namespace.checkpoint());
    let mut wrong = dry.clone();
    wrong
        .producer_heads
        .as_mut()
        .unwrap()
        .source
        .profile
        .scope
        .metadata = Digest::blob(b"another provider");
    let mut wrong_trust = trust.clone();
    wrong_trust.placement_enrollment = wrong.id().unwrap();
    let wrong_request = github::verify(
        &wrong_trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    assert!(
        batch::configured(worker, &wrong, &wrong_trust, &wrong_request)
            .reconcile(
                None,
                &budget,
                &mut m.resources(),
                &mut f.resources(),
                || Ok(1000)
            )
            .is_err()
    );
    assert_eq!(checkpoint, worker.namespace.checkpoint());
    // Prepare another legitimate metadata change, then race it against copy.
    let blocked = roots::prepare(
        &source.profile,
        Some(&source.version),
        &[roots::Change {
            expected_record: Some(updated.metadata_object().unwrap()),
            record: source.row.clone(),
        }],
        &mut source.namespace,
        &mut source.nodes,
    )
    .unwrap()
    .unwrap();
    let mut trust = trust.clone();
    trust.placement_enrollment = policy.id().unwrap();
    let request = github::verify(
        &trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    let copy = crate::annex::replication::Plan::from_snapshot(
        &policy.copy,
        source.row.producer.clone(),
        destination.row.producer.clone(),
        updated.retained.clone().unwrap(),
    )
    .unwrap();
    assert_eq!(
        destination
            .row
            .with_completed_publication(copy.destination().clone(), &f.witness, &mut f.ledger)
            .unwrap_err(),
        Error::NeedsRecovery
    );
    let mut calls = 0;
    let mut raced = false;
    let applied = batch::configured(worker, &policy, &trust, &request)
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || {
                calls += 1;
                if calls == 2 {
                    assert_eq!(
                        roots::apply(
                            &source.profile,
                            &blocked,
                            &mut source.namespace,
                            &mut source.nodes,
                            || Ok(())
                        )
                        .err(),
                        Some(Error::Busy)
                    );
                    raced = true;
                }
                Ok(1000)
            },
        )
        .unwrap();
    assert!(raced);
    assert_eq!(
        applied.outcomes[0].state,
        "copied",
        "{}",
        serde_json::to_string(&applied).unwrap()
    );
    assert_eq!(
        applied.producer_heads.as_ref().unwrap().source.version,
        source.version
    );
    assert_eq!(applied.outcomes[0].plan, Some(copy.id().unwrap()));
    // Preserve the actual immutable copy recipe before another history rebind.
    let published = destination
        .row
        .with_completed_publication(copy.destination().clone(), &f.witness, &mut f.ledger)
        .unwrap();
    assert_eq!(published.producer, destination.row.producer);
    assert_eq!(published.publication.as_ref(), Some(copy.destination()));
    assert_eq!(
        published
            .with_completed_publication(destination.row.producer.clone(), &f.witness, &mut f.ledger)
            .unwrap_err(),
        Error::Conflict
    );
    use crate::annex::pending::RegistrationSide;
    use crate::github::native::batch::registration;
    let scope = policy.actions.as_ref().unwrap().pending.as_ref().unwrap();
    let frozen = roots::prepare(
        &destination.profile,
        Some(&destination.version),
        &[roots::Change {
            expected_record: Some(destination.row.metadata_object().unwrap()),
            record: published.clone(),
        }],
        &mut destination.namespace,
        &mut destination.nodes,
    )
    .unwrap()
    .unwrap();
    let original = crate::annex::pending::save(
        scope,
        policy.id().unwrap(),
        request.commit().as_str(),
        &crate::annex::pending::Action {
            working: destination.row.producer.clone(),
            retained: source.row.producer.clone(),
            operation: crate::annex::pending::Operation::Registration(Box::new(
                crate::annex::pending::Registration {
                    side: RegistrationSide::Working,
                    index: frozen,
                    record: published.clone(),
                },
            )),
        },
        &mut pending,
        worker.metadata,
        &Fixture::limits(),
    )
    .unwrap();
    destination
        .namespace
        .select(
            &admission::Select {
                prepare: k("advance-working-index-prepare"),
                adopt: k("advance-working-index-adopt"),
                source: destination.profile.scope.prefix.clone(),
                owner: destination.profile.scope.owner.clone(),
                expected_revision: destination.version.revision,
                selection: destination.version.selection.clone(),
            },
            &destination.nodes,
            &Fixture::limits(),
        )
        .unwrap();
    let registered = batch::configured(worker, &policy, &trust, &request)
        .reconcile_registrations(
            registration::Pass {
                cursor: None,
                budget: &budget,
                pending: &mut pending,
                target: registration::Target {
                    side: RegistrationSide::Working,
                    namespace: &mut destination.namespace,
                    nodes: &mut destination.nodes,
                },
                apply: true,
                replan: true,
            },
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert!(
        registered.outcomes[0].hold.is_none(),
        "{}",
        serde_json::to_string(&registered).unwrap()
    );
    assert_eq!(registered.outcomes[0].state, "recorded");
    assert_ne!(registered.outcomes[0].saved.as_ref(), Some(&original));
    assert!(
        crate::annex::pending::reopen(
            scope,
            &original,
            &mut pending,
            worker.metadata,
            &Fixture::limits(),
        )
        .is_ok()
    );
    let result = registered
        .outcomes
        .into_iter()
        .next()
        .unwrap()
        .registration
        .unwrap();
    assert_eq!(
        Index::from_root(result.root)
            .get(
                &published.object,
                &mut destination.nodes,
                TreeLimits::default()
            )
            .unwrap()
            .0
            .as_ref(),
        Some(&published)
    );
    destination.version = result.version;
    f.metadata =
        MetadataReader::open(&f.root.join("metadata"), &destination.nodes.checkpoint()).unwrap();
    *worker.metadata =
        MetadataStore::open(&f.root.join("metadata"), &destination.nodes.checkpoint()).unwrap();
    // Ordinary root edits cannot erase a recorded recipe and hide its uncertainty.
    assert_eq!(
        roots::prepare(
            &destination.profile,
            Some(&destination.version),
            &[roots::Change {
                expected_record: Some(published.metadata_object().unwrap()),
                record: destination.row.clone(),
            }],
            &mut destination.namespace,
            &mut destination.nodes
        )
        .err(),
        Some(Error::Conflict)
    );
    let repeated = batch::configured(worker, &policy, &trust, &request)
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(
        repeated.outcomes[0].state,
        "present",
        "{}",
        serde_json::to_string(&repeated).unwrap()
    );
    assert_eq!(
        repeated.outcomes[0].publication,
        applied.outcomes[0].publication
    );
    // Ending the pass releases only the live head guard, not durable history.
    roots::apply(
        &source.profile,
        &blocked,
        &mut source.namespace,
        &mut source.nodes,
        || Ok(()),
    )
    .unwrap();
    m.metadata =
        MetadataReader::open(&m.root.join("metadata"), &source.nodes.checkpoint()).unwrap();
    let checkpoint = f.ledger.checkpoint();
    let rebound = batch::configured(worker, &policy, &trust, &request)
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(
        rebound.outcomes[0].state,
        "present",
        "{}",
        serde_json::to_string(&rebound).unwrap()
    );
    assert_eq!(
        rebound.outcomes[0].publication,
        applied.outcomes[0].publication
    );
    assert_eq!(checkpoint, f.ledger.checkpoint());
    assert!(
        source
            .namespace
            .source(&k("producers"))
            .unwrap()
            .unwrap()
            .predecessors
            .contains(&source.version)
    );
}

pub(super) fn exercise_backup(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    m: &mut Fixture,
    f: &mut Fixture,
    signer: &github::fixture::Fixture,
) {
    backup::exercise(worker, m, f, signer);
}
