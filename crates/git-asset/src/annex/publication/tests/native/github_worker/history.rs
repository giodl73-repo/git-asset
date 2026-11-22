use super::*;
use crate::annex::history as archive;

pub(super) fn exercise(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    m: &mut Fixture,
    f: &mut Fixture,
    project: &Path,
    signer: &github::fixture::Fixture,
) {
    let mut records = batch::pending_namespace(m);
    let mut metadata =
        MetadataStore::open(&m.root.join("metadata"), m.metadata.checkpoint()).unwrap();
    let profile = archive::Profile {
        working: archive::Working {
            namespace: worker.enrollment.namespace.clone(),
            source: worker.enrollment.source.clone(),
            owner: worker.enrollment.owner.clone(),
            class: worker.enrollment.class.clone(),
        },
        target: archive::Target {
            namespace: m.namespaces[0].identity().clone(),
            owner: k("owner"),
            place: k("pool"),
            prefix: k("history-media"),
        },
        records: batch::pending_scope(m, &records),
        limits: Fixture::limits(),
        allow_publish: true,
        allow_release: true,
    };
    // Three immutable Git snapshots establish two real predecessor obligations.
    for index in 0..3 {
        if index != 0 {
            let object = Object::from_bytes(format!("synthetic version {index}").as_bytes());
            fs::write(
                project.join("assets/clip"),
                format!(
                    "/annex/objects/SHA256-s{}--{}\n",
                    object.bytes,
                    object.sha256.as_str()
                ),
            )
            .unwrap();
            git(project, &["add", "assets"]);
            git(project, &["commit", "-m", "advance synthetic main"]);
        }
        let mut selection = lifecycle::Execution {
            enrollment: worker.enrollment,
            repository: worker.repository,
            namespace: worker.namespace,
            metadata: worker.metadata,
            allow_selection: true,
        };
        let plan = selection.plan(None).unwrap();
        selection.apply(&plan).unwrap();
    }
    let before = worker
        .namespace
        .source(&worker.enrollment.source)
        .unwrap()
        .unwrap();
    assert_eq!(before.predecessors.len(), 2);
    let commit = source::GitId::parse(
        String::from_utf8(git(project, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim(),
    )
    .unwrap();
    let mut policy = worker.policy.clone();
    policy.actions.as_mut().unwrap().history = Some(profile.clone());
    let mut trust = worker.trust.clone();
    trust.placement_enrollment = policy.id().unwrap();
    let mut claims = signer.claims.clone();
    claims["sha"] = serde_json::json!(commit.as_str());
    let request =
        github::verify(&trust, &signer.keys, &signer.token(&claims), &commit, 1000).unwrap();
    let budget = github::placement::Budget {
        pages: 1,
        objects: 1,
        stream_bytes: 1,
        millis: 60000,
    };
    // Native repositories are offline throughout the metadata-only pass.
    let configs: Vec<_> = [
        m.profile
            .pool
            .pool
            .repository
            .join(if m.profile.pool.pool.bare {
                "config"
            } else {
                ".git/config"
            }),
        f.profile
            .pool
            .pool
            .repository
            .join(if f.profile.pool.pool.bare {
                "config"
            } else {
                ".git/config"
            }),
    ]
    .into_iter()
    .map(|path| {
        let offline = path.with_extension("offline");
        fs::rename(&path, &offline).unwrap();
        (path, offline)
    })
    .collect();
    let checkpoint = records.checkpoint();
    let media_checkpoint = m.namespaces[0].checkpoint();
    let metadata_checkpoint = metadata.checkpoint();
    // A changed policy cannot borrow the signed admission of another policy.
    let mut unsigned = policy.clone();
    unsigned
        .actions
        .as_mut()
        .unwrap()
        .history
        .as_mut()
        .unwrap()
        .allow_release = false;
    assert!(
        batch::configured(worker, &unsigned, &trust, &request)
            .reconcile_history(
                github::native::history::Pass {
                    cursor: None,
                    budget: &budget,
                    records: &mut records,
                    metadata: &mut metadata,
                    apply: true,
                },
                || Ok(1000)
            )
            .is_err()
    );
    assert_eq!(checkpoint, records.checkpoint());
    assert_eq!(
        before,
        worker
            .namespace
            .source(&worker.enrollment.source)
            .unwrap()
            .unwrap()
    );
    let mut mismatched = policy.clone();
    mismatched
        .actions
        .as_mut()
        .unwrap()
        .history
        .as_mut()
        .unwrap()
        .working
        .source = k("unrelated");
    let mut mismatched_trust = trust.clone();
    mismatched_trust.placement_enrollment = mismatched.id().unwrap();
    let mismatched_request = github::verify(
        &mismatched_trust,
        &signer.keys,
        &signer.token(&claims),
        &commit,
        1000,
    )
    .unwrap();
    assert!(
        batch::configured(worker, &mismatched, &mismatched_trust, &mismatched_request)
            .reconcile_history(
                github::native::history::Pass {
                    cursor: None,
                    budget: &budget,
                    records: &mut records,
                    metadata: &mut metadata,
                    apply: true,
                },
                || Ok(1000)
            )
            .is_err()
    );
    assert_eq!(
        before,
        worker
            .namespace
            .source(&worker.enrollment.source)
            .unwrap()
            .unwrap()
    );
    let mut execution = batch::configured(worker, &policy, &trust, &request);
    let preview = execution
        .reconcile_history(
            github::native::history::Pass {
                cursor: None,
                budget: &budget,
                records: &mut records,
                metadata: &mut metadata,
                apply: false,
            },
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(preview.outcomes.len(), 1);
    assert_eq!(preview.outcomes[0].state, "planned");
    assert!(preview.outcomes[0].saved.is_none());
    assert_eq!(checkpoint, records.checkpoint());
    assert_eq!(media_checkpoint, m.namespaces[0].checkpoint());
    assert_eq!(metadata_checkpoint, metadata.checkpoint());
    assert_eq!(
        before,
        execution
            .namespace
            .source(&execution.enrollment.source)
            .unwrap()
            .unwrap()
    );

    let unknown = git_asset_store::placement::Cursor {
        basis: preview.basis.clone(),
        after: Digest::blob(b"unknown archive cursor"),
    };
    assert!(
        execution
            .reconcile_history(
                github::native::history::Pass {
                    cursor: Some(&unknown),
                    budget: &budget,
                    records: &mut records,
                    metadata: &mut metadata,
                    apply: true,
                },
                || Ok(1000)
            )
            .is_err()
    );
    assert_eq!(checkpoint, records.checkpoint());
    let first = execution
        .reconcile_history(
            github::native::history::Pass {
                cursor: None,
                budget: &budget,
                records: &mut records,
                metadata: &mut metadata,
                apply: true,
            },
            || {
                // Expiry after the release must retain its successful receipt.
                let state = f.namespaces[0]
                    .source(&profile.working.source)
                    .unwrap()
                    .unwrap();
                Ok(if state.predecessors.len() == 1 {
                    1300
                } else {
                    1000
                })
            },
        )
        .unwrap();
    assert_eq!(
        first.outcomes[0].state,
        "released",
        "{}",
        serde_json::to_string(&first).unwrap()
    );
    assert!(first.outcomes[0].hold.is_some());
    assert!(first.post_effect_hold.is_some());
    assert!(first.outcomes[0].saved.is_some());
    let release = first.outcomes[0].release.as_ref().unwrap();
    assert!(!release.authorizes_eviction);
    assert_eq!(release.media_custody_verified, None);
    assert_eq!(first.media_stream_bytes, 0);
    assert_eq!(first.storage_converged, None);
    let cursor = first.next.as_ref().unwrap();
    let state = execution
        .namespace
        .source(&execution.enrollment.source)
        .unwrap()
        .unwrap();
    assert_eq!(state.current, before.current);
    assert_eq!(state.predecessors.len(), 1);
    assert!(!state.predecessors.contains(&first.outcomes[0].origin));

    // Resume names the removed predecessor through its durable original receipt.
    // A new authenticated request renews admission without rewinding its clock.
    claims["iat"] = serde_json::json!(1300);
    claims["nbf"] = serde_json::json!(1300);
    claims["exp"] = serde_json::json!(1450);
    let renewed =
        github::verify(&trust, &signer.keys, &signer.token(&claims), &commit, 1300).unwrap();
    let mut execution = batch::configured(&mut execution, &policy, &trust, &renewed);
    let second = execution
        .reconcile_history(
            github::native::history::Pass {
                cursor: Some(cursor),
                budget: &budget,
                records: &mut records,
                metadata: &mut metadata,
                apply: true,
            },
            || Ok(1300),
        )
        .unwrap();
    assert_eq!(second.basis, first.basis);
    assert_eq!(second.outcomes.len(), 1);
    assert_eq!(
        second.outcomes[0].state,
        "released",
        "{}",
        serde_json::to_string(&second).unwrap()
    );
    assert_ne!(second.outcomes[0].archive, first.outcomes[0].archive);
    assert!(second.next.is_none());
    assert!(
        execution
            .namespace
            .source(&execution.enrollment.source)
            .unwrap()
            .unwrap()
            .predecessors
            .is_empty()
    );
    let repeated = execution
        .reconcile_history(
            github::native::history::Pass {
                cursor: Some(cursor),
                budget: &budget,
                records: &mut records,
                metadata: &mut metadata,
                apply: true,
            },
            || Ok(1300),
        )
        .unwrap();
    assert!(repeated.outcomes.is_empty());

    // Historical metadata remains exact; missing media never gets read or hydrated.
    for outcome in [&first.outcomes[0], &second.outcomes[0]] {
        let loaded = archive::load(
            &profile,
            outcome.saved.as_ref().unwrap(),
            archive::Destinations {
                media: &mut m.namespaces[0],
                records: &mut records,
            },
            &metadata,
        )
        .unwrap();
        assert_eq!(loaded.plan().origin(), &outcome.origin);
    }
    assert_eq!(f.plan.candidate().object, m.plan.candidate().object);
    git(
        project,
        &["commit", "--allow-empty", "-m", "advance after history"],
    );
    assert!(
        execution
            .reconcile_history(
                github::native::history::Pass {
                    cursor: Some(cursor),
                    budget: &budget,
                    records: &mut records,
                    metadata: &mut metadata,
                    apply: true,
                },
                || Ok(1300)
            )
            .is_err()
    );
    for (path, offline) in configs {
        assert!(!path.exists());
        fs::rename(offline, path).unwrap();
    }
}
