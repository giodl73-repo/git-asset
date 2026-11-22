use super::*;
use crate::annex::{publication::handoff, snapshot::Snapshot};
use git_asset_store::source_claims::{Selection, Version, admission::Select};

pub(super) fn exercise(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    m: &mut Fixture,
    f: &mut Fixture,
    signer: &github::fixture::Fixture,
) {
    let old = worker.policy.producers.as_ref().unwrap();
    let mut source = register(m, old.source.clone(), "retained-producers");
    source.head.profile.scope.allow_store = true;
    let destination = register(f, old.destination.clone(), "working-producers");
    let mut pending = batch::pending_namespace_at(&f.root.join("publication-actions"));
    let mut policy = worker.policy.clone();
    policy.actions.as_mut().unwrap().registration = true;
    policy.actions.as_mut().unwrap().pending = Some(batch::pending_scope(f, &pending));
    policy.producers = None;
    policy.producer_heads = Some(Heads {
        source: source.head.clone(),
        destination: destination.head,
    });
    let mut trust = worker.trust.clone();
    trust.placement_enrollment = policy.id().unwrap();
    let request = github::verify(
        &trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    let budget = github::placement::Budget {
        pages: 1,
        objects: 1,
        stream_bytes: 1024 * 1024,
        millis: 60000,
    };
    let reverse = crate::annex::replication::Profile {
        source: policy.copy.destination.clone(),
        destination: policy.copy.source.clone(),
        source_domain: policy.copy.destination_domain.clone(),
        destination_domain: policy.copy.source_domain.clone(),
    };
    let backup =
        crate::annex::replication::Plan::new(&reverse, f.plan.clone(), m.plan.clone()).unwrap();
    let result = batch::configured(worker, &policy, &trust, &request)
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(
        result.outcomes[0].state,
        "backed_up",
        "{}",
        serde_json::to_string(&result).unwrap()
    );
    assert_eq!(
        result.outcomes[0].retained_publication,
        Some(backup.destination().id().unwrap())
    );

    use crate::annex::pending;
    use crate::github::native::batch::registration;
    macro_rules! registration_pass {
        ($apply:expr,$cursor:expr) => {
            registration::Pass {
                cursor: $cursor,
                budget: &budget,
                pending: &mut pending,
                target: registration::Target {
                    side: pending::RegistrationSide::Retained,
                    namespace: &mut source.namespace,
                    nodes: &mut source.nodes,
                },
                apply: $apply,
            }
        };
    }
    let target_checkpoint = source.namespace.checkpoint();
    let pending_checkpoint = pending.checkpoint();
    let native_checkpoint = m.namespaces[0].checkpoint();
    let preview = batch::configured(worker, &policy, &trust, &request)
        .reconcile_registrations(
            registration_pass!(false, None),
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(preview.outcomes[0].state, "eligible");
    assert!(preview.outcomes[0].saved.is_none());
    assert_eq!(target_checkpoint, source.namespace.checkpoint());
    assert_eq!(pending_checkpoint, pending.checkpoint());
    let mut denied_policy = policy.clone();
    denied_policy
        .producer_heads
        .as_mut()
        .unwrap()
        .source
        .profile
        .scope
        .allow_store = false;
    let mut denied_trust = trust.clone();
    denied_trust.placement_enrollment = denied_policy.id().unwrap();
    let denied_request = github::verify(
        &denied_trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    let denied = batch::configured(worker, &denied_policy, &denied_trust, &denied_request)
        .reconcile_registrations(
            registration_pass!(true, None),
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(
        denied.outcomes[0].hold.as_ref().map(|h| h.code.as_str()),
        Some("denied")
    );
    assert!(denied.outcomes[0].saved.is_none());
    assert_eq!(target_checkpoint, source.namespace.checkpoint());
    assert_eq!(pending_checkpoint, pending.checkpoint());
    let registered = batch::configured(worker, &policy, &trust, &request)
        .reconcile_registrations(
            registration_pass!(true, None),
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
    assert!(registered.fresh_pass_required && registered.next.is_none());
    assert_eq!(native_checkpoint, m.namespaces[0].checkpoint());
    let cursor = git_asset_store::placement::Cursor {
        basis: preview.basis,
        after: m.plan.candidate().object.sha256.clone(),
    };
    assert!(matches!(
        batch::configured(worker, &policy, &trust, &request).reconcile_registrations(
            registration_pass!(true, Some(&cursor)),
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Admission(github::Error::Stale))
    ));
    let checkpoint = source.namespace.checkpoint();
    let repeated = batch::configured(worker, &policy, &trust, &request)
        .reconcile_registrations(
            registration_pass!(true, None),
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(repeated.outcomes[0].state, "recorded");
    assert!(repeated.outcomes[0].hold.is_none());
    assert_eq!(checkpoint, source.namespace.checkpoint());
    let saved = registered.outcomes[0].saved.as_ref().unwrap();
    let scope = policy.actions.as_ref().unwrap().pending.as_ref().unwrap();
    let loaded = pending::reopen(
        scope,
        saved,
        &mut pending,
        worker.metadata,
        &Fixture::limits(),
    )
    .unwrap();
    let pending::Operation::Registration(body) = &loaded.action.operation else {
        panic!("wrong registration body")
    };
    let config = f.root.join("registration-reopen.json");
    fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({"scope":scope,"saved":saved,
        "namespace":f.root.join("publication-actions"),"minimum":pending.checkpoint(),
        "metadata":f.root.join("metadata"),"metadata_minimum":worker.metadata.checkpoint(),
        "plan":Digest::blob(&body.index.to_json().unwrap())}))
        .unwrap(),
    )
    .unwrap();
    drop(loaded);
    assert!(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "annex::publication::tests::native::pending::reopen_worker"
            ])
            .env("GIT_ASSET_PENDING_REOPEN_FIXTURE", config)
            .status()
            .unwrap()
            .success()
    );
    let result = registered
        .outcomes
        .into_iter()
        .next()
        .unwrap()
        .registration
        .unwrap();
    source.version = result.version;
    source.row = Index::from_root(result.root)
        .get(
            &m.plan.candidate().object,
            &mut source.nodes,
            TreeLimits::default(),
        )
        .unwrap()
        .0
        .unwrap();
    assert_eq!(source.row.publication.as_ref(), Some(backup.destination()));

    // A real retained root replaces both temporary ingest obligations.
    let path = LogicalPath::parse("assets/archived").unwrap();
    let object = m.plan.candidate().object.clone();
    let (tree, _) = Tree::build(
        &BTreeMap::from([(path.clone(), object.clone())]),
        &mut source.nodes,
        TreeLimits::default(),
    )
    .unwrap();
    source.nodes.finish().unwrap();
    m.namespaces[0]
        .apply(
            k("enroll-archived-members"),
            Command::Enroll {
                source: k("archived-members"),
                owner: k("owner"),
                class: Kind::Retention,
            },
            &source.nodes,
            &Fixture::limits(),
        )
        .unwrap();
    let selection = Selection {
        root: tree.root().clone(),
        context: Digest::blob(b"exact archived member tree"),
    };
    let receipt = m.namespaces[0]
        .select(
            &Select {
                prepare: k("archive-prepare"),
                adopt: k("archive-adopt"),
                source: k("archived-members"),
                owner: k("owner"),
                expected_revision: 1,
                selection: selection.clone(),
            },
            &source.nodes,
            &Fixture::limits(),
        )
        .unwrap();
    let snapshot = Snapshot {
        namespace: m.namespaces[0].identity().clone(),
        source: k("archived-members"),
        owner: k("owner"),
        version: Version {
            revision: receipt.source_revision,
            selection,
        },
        place: k("pool"),
        path,
        object: object.clone(),
    };
    let mut published = source
        .row
        .with_completed_publication(backup.destination().clone(), &m.witness, &mut m.ledger)
        .unwrap();
    published.retained = Some(snapshot.clone());
    let plan = roots::prepare(
        &source.profile,
        Some(&source.version),
        &[roots::Change {
            expected_record: Some(source.row.metadata_object().unwrap()),
            record: published,
        }],
        &mut source.namespace,
        &mut source.nodes,
    )
    .unwrap()
    .unwrap();
    let mut saved = fs::File::create_new(m.root.join("backup-registration-plan")).unwrap();
    std::io::Write::write_all(&mut saved, &plan.to_json().unwrap()).unwrap();
    saved.sync_all().unwrap();
    let registered = roots::apply(
        &source.profile,
        &plan,
        &mut source.namespace,
        &mut source.nodes,
        || Ok(()),
    )
    .unwrap();
    m.metadata =
        MetadataReader::open(&m.root.join("metadata"), &source.nodes.checkpoint()).unwrap();
    retirement::settle(m);
    assert!(
        m.publisher()
            .reopen(backup.destination(), &mut m.resources())
            .is_err()
    );
    let handoff = handoff::Plan::new(
        &f.plan,
        backup.destination(),
        snapshot,
        &mut f.namespaces[0],
    )
    .unwrap();
    batch::configured(worker, &policy, &trust, &request)
        .handoff(
            &handoff,
            &f.plan.clone(),
            backup.destination(),
            &mut f.resources(),
            &mut m.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert!(
        f.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .current
            .is_none()
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
    // Historical bytes need the recorded backup recipe, without F's native repo.
    fs::rename(
        f.profile
            .pool
            .pool
            .repository
            .join(if f.profile.pool.pool.bare {
                "config"
            } else {
                ".git/config"
            }),
        f.root.join("offline-working-config"),
    )
    .unwrap();
    let row = Index::from_root(registered.root)
        .get(&object, &mut source.nodes, TreeLimits::default())
        .unwrap()
        .0
        .unwrap();
    assert_eq!(row.producer, source.row.producer);
    let mut actual = Vec::new();
    m.publisher()
        .open_retained(
            row.publication.as_ref().unwrap(),
            row.retained.as_ref().unwrap(),
            &mut m.resources(),
        )
        .unwrap()
        .read_to_end(&mut actual)
        .unwrap();
    assert_eq!(Object::from_bytes(&actual), object);
}
