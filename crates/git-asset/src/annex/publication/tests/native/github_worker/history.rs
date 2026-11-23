use super::*;
use crate::annex::history as archive;

pub(super) fn exercise(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    m: &mut Fixture,
    f: &mut Fixture,
    project: &Path,
    signer: &github::fixture::Fixture,
    indexed: bool,
) {
    if indexed {
        retirement::publish(m);
    }
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
    let mut indexed_target = None;
    let mut actions = indexed.then(|| batch::pending_namespace_at(&f.root.join("archive-actions")));
    if indexed {
        let old = policy.producers.take().unwrap();
        let mut retained = heads::register(m, old.source, "archive-retained").head;
        let mut working = heads::register(f, old.destination, "archive-working").head;
        working.profile.scope.allow_store = true;
        let golden = publish_golden(f);
        let mut ns = Namespace::open(&working.root, &working.minimum).unwrap();
        let mut nodes =
            MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
        let baseline = ns
            .source(&working.profile.scope.prefix)
            .unwrap()
            .unwrap()
            .current;
        use crate::annex::producer_index::update::initial;
        let mut target = initial::Target {
            profile: &working.profile,
            namespace: &mut ns,
            nodes: &mut nodes,
        };
        let mut publisher = f.publisher();
        let plan = initial::prepare(
            &mut target,
            baseline.as_ref(),
            &golden,
            &mut publisher,
            &mut f.resources(),
        )
        .unwrap()
        .unwrap();
        let plan = crate::annex::producer_index::update::Plan::from_json(&plan.to_json().unwrap())
            .unwrap();
        let receipt = initial::apply(
            &mut target,
            &plan,
            &golden,
            &mut publisher,
            &mut f.resources(),
            || Ok(()),
        )
        .unwrap();
        assert_eq!(receipt.current, Some(true));
        let mut read = publisher.reopen(&golden, &mut f.resources()).unwrap();
        let mut bytes = Vec::new();
        read.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"synthetic version 2");
        retained.profile.scope.allow_store = true;
        indexed_target = Some(Namespace::open(&retained.root, &retained.minimum).unwrap());
        policy.producer_heads = Some(github::native::producers::Heads {
            source: retained,
            destination: working,
        });
        let admitted = policy.actions.as_mut().unwrap();
        admitted.archive_index = true;
        admitted.pending = Some(batch::pending_scope(f, actions.as_ref().unwrap()));
        metadata = MetadataStore::open(&m.root.join("metadata"), m.metadata.checkpoint()).unwrap();
    }
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
    if indexed {
        let mut execution = batch::configured(worker, &policy, &trust, &request);
        let mut indexed = 0;
        let mut held_missing = 0;
        // One registered predecessor and one absent producer, independent of ordering.
        for _ in 0..2 {
            let r = execution
                .reconcile_history(
                    github::native::history::Pass {
                        index: Some(github::native::history::indexing::Target {
                            namespace: indexed_target.as_mut().unwrap(),
                            actions: actions.as_mut().unwrap(),
                        }),
                        cursor: None,
                        budget: &budget,
                        records: &mut records,
                        metadata: &mut metadata,
                        apply: true,
                    },
                    || Ok(1000),
                )
                .unwrap();
            let outcome = &r.outcomes[0];
            let row = outcome.indexing.as_ref().unwrap();
            if row.indexed == 1 {
                indexed += 1;
                assert!(row.saved.is_some());
                assert!(outcome.release.is_some());
                let saved = row.saved.as_ref().unwrap();
                let scope = policy.actions.as_ref().unwrap().pending.as_ref().unwrap();
                let loaded = crate::annex::pending::reopen(
                    scope,
                    saved,
                    actions.as_mut().unwrap(),
                    execution.metadata,
                    &Fixture::limits(),
                )
                .unwrap();
                let crate::annex::pending::Operation::ArchiveIndex(body) = &loaded.action.operation
                else {
                    panic!("wrong archive index body")
                };
                let config = f.root.join("archive-index-reopen.json");
                fs::write(&config, serde_json::to_vec(&serde_json::json!({
                    "scope":scope,"saved":saved,"namespace":f.root.join("archive-actions"),
                    "minimum":actions.as_ref().unwrap().checkpoint(),
                    "metadata":f.root.join("metadata"),"metadata_minimum":execution.metadata.checkpoint(),
                    "plan":Digest::blob(&body.index.to_json().unwrap())
                })).unwrap()).unwrap();
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
            } else {
                held_missing += 1;
                assert_eq!(row.hold.as_ref().unwrap().code, "missing");
                assert!(outcome.release.is_none());
                assert!(outcome.saved.is_some());
            }
        }
        assert_eq!(indexed, 1);
        assert_eq!(held_missing, 1);
        let state = execution
            .namespace
            .source(&execution.enrollment.source)
            .unwrap()
            .unwrap();
        assert_eq!(state.current, before.current);
        assert_eq!(state.predecessors.len(), 1);
        let [
            (retained_config, retained_offline),
            (working_config, working_offline),
        ] = configs.try_into().unwrap();
        fs::rename(retained_offline, retained_config).unwrap();
        let head = &policy.producer_heads.as_ref().unwrap().source;
        let version = indexed_target
            .as_mut()
            .unwrap()
            .source(&head.profile.scope.prefix)
            .unwrap()
            .unwrap()
            .current
            .unwrap();
        let row = crate::annex::producer_index::Index::from_root(version.selection.root)
            .get(
                &m.plan.candidate().object,
                &mut metadata,
                TreeLimits::default(),
            )
            .unwrap()
            .0
            .unwrap();
        let snapshot = row.retained.as_ref().unwrap();
        m.metadata = metadata.finished_reader().unwrap();
        assert!(!working_config.exists());
        let mut retained = m
            .publisher()
            .open_retained(&row.producer, snapshot, &mut m.resources())
            .unwrap();
        let mut bytes = Vec::new();
        retained.read_to_end(&mut bytes).unwrap();
        assert_eq!(Object::from_bytes(&bytes), snapshot.object);
        drop(retained);
        fs::rename(working_offline, working_config).unwrap();
        return;
    }
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
                    index: None,
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
                    index: None,
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
                index: None,
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
                    index: None,
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
                index: None,
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
                index: None,
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
                index: None,
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
                    index: None,
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

fn publish_golden(f: &mut Fixture) -> Plan {
    use crate::annex::control;
    let bytes = b"synthetic version 2";
    let mut request = f.plan.request.clone();
    request.operation = k("publish-golden");
    request.expected_revision = f.namespaces[0]
        .source(&request.source)
        .unwrap()
        .unwrap()
        .revision;
    request.candidate.object = Object::from_bytes(bytes);
    fs::write(
        f.root
            .join("input/objects")
            .join(request.candidate.object.sha256.as_str()),
        bytes,
    )
    .unwrap();
    let mut nodes = MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let selection = freeze(&request, &mut nodes, TreeLimits::default()).unwrap();
    nodes.finish().unwrap();
    f.metadata = nodes.finished_reader().unwrap();
    let profile = f.control_profile.as_ref().unwrap();
    let store = LocalStore::open(&profile.root).unwrap();
    let mut ledger = CapacityLedger::open(&profile.ledger, &profile.minimum).unwrap();
    let control_request =
        control::initial_request(&request.candidate.object, profile.owner.clone()).unwrap();
    let publication = git_asset_store::accounted::plan_bound(
        &store,
        &control_request,
        control::writer_context(&f.profile, &request).unwrap(),
    )
    .unwrap();
    let job = f.plan.control.as_ref().unwrap().plan.job.clone();
    let document = control::document(&request.candidate.object).unwrap();
    git_asset_store::accounted::publish_accounted(
        &mut ledger,
        &store,
        &job,
        &publication,
        &mut Cursor::new(document),
    )
    .unwrap();
    let golden = Plan::controlled(
        &f.profile,
        request,
        selection,
        f.plan.admission.clone(),
        profile,
        control::Plan { publication, job },
    )
    .unwrap();
    let old = std::mem::replace(&mut f.plan, golden.clone());
    retirement::publish(f);
    f.plan = old;
    golden
}
