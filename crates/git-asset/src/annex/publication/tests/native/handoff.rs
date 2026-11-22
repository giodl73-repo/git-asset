//! Original journal recovery; no native removal or production enrollment.
use super::*;
use crate::annex::{publication::handoff as transfer, replication};
fn pair(native: bool, fenced: bool) -> (Fixture, Fixture) {
    let tools = native.then(|| {
        (
            std::env::var_os("GIT_ASSET_TEST_GIT_EXE").unwrap().into(),
            std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE").unwrap().into(),
            std::env::split_paths(&std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").unwrap())
                .collect(),
        )
    });
    let bytes = vec![83; 256 * 1024];
    let mut source = Fixture::build_setup(&bytes, tools.clone(), false, None, false);
    let mut keeper = Fixture::build_setup(&bytes, tools, true, None, fenced);
    retirement::publish(&mut source);
    let profile = replication::Profile {
        source: source.profile.clone(),
        destination: keeper.profile.clone(),
        source_domain: k("source-domain"),
        destination_domain: k("keeper-domain"),
    };
    let plan = replication::Plan::new(&profile, source.plan.clone(), keeper.plan.clone()).unwrap();
    let mut copier = replication::Copier::new(profile).unwrap();
    let ready = copier
        .apply(&plan, &mut source.resources(), &mut keeper.resources())
        .unwrap();
    drop(ready);
    drop(copier);
    keeper.plan = plan.destination().clone();
    (source, keeper)
}
fn keeper_snapshot(keeper: &Fixture) -> crate::annex::snapshot::Snapshot {
    let mut namespace = Namespace::open(
        &keeper.root.join("namespace-0"),
        &keeper.namespaces[0].checkpoint(),
    )
    .unwrap();
    crate::annex::snapshot::Snapshot {
        namespace: namespace.identity().clone(),
        source: k("asset"),
        owner: k("owner"),
        version: namespace
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .current
            .unwrap(),
        place: k("pool"),
        path: LogicalPath::parse("candidate/output").unwrap(),
        object: keeper.plan.candidate().object.clone(),
    }
}
fn frozen(source: &mut Fixture, keeper: &Fixture) -> transfer::Plan {
    let plan = transfer::Plan::new(
        &source.plan,
        &keeper.plan,
        keeper_snapshot(keeper),
        &mut source.namespaces[0],
    )
    .unwrap();
    transfer::Plan::from_json(&plan.to_json().unwrap()).unwrap()
}
fn apply(
    source: &mut Fixture,
    keeper: &mut Fixture,
    plan: &transfer::Plan,
) -> Result<transfer::Report> {
    transfer::Execution {
        source: &mut source.publisher(),
        keeper: &mut keeper.publisher(),
        allow_release: true,
    }
    .apply(
        plan,
        &source.plan.clone(),
        &keeper.plan.clone(),
        &mut source.resources(),
        &mut keeper.resources(),
    )
}
#[test]
fn handoff_crash_worker() {
    let Some(root) = std::env::var_os("GIT_ASSET_HANDOFF_SOURCE") else {
        return;
    };
    let mut source = Fixture::load(root.into());
    let mut keeper = Fixture::load(std::env::var_os("GIT_ASSET_HANDOFF_KEEPER").unwrap().into());
    let plan =
        transfer::Plan::from_json(&fs::read(source.root.join("handoff-plan")).unwrap()).unwrap();
    let mut peer = Namespace::open(
        &keeper.root.join("namespace-0"),
        &keeper.namespaces[0].checkpoint(),
    )
    .unwrap();
    let revision = peer.source(&k("asset")).unwrap().unwrap().revision;
    let peer_metadata =
        MetadataReader::open(&keeper.root.join("metadata"), keeper.metadata.checkpoint()).unwrap();
    transfer::Execution {
        source: &mut source.publisher(),
        keeper: &mut keeper.publisher(),
        allow_release: true,
    }
    .apply_hook(
        &plan,
        &source.plan.clone(),
        &keeper.plan.clone(),
        &mut source.resources(),
        &mut keeper.resources(),
        || {
            assert!(matches!(
                peer.apply(
                    k("blocked-keeper-release"),
                    Command::Withdraw {
                        source: k("asset"),
                        owner: k("owner"),
                        expected_revision: revision
                    },
                    &peer_metadata,
                    &Fixture::limits()
                ),
                Err(admission::Error::Journal(
                    git_asset_store::source_claims::Error::Busy
                ))
            ));
            std::process::exit(86)
        },
    )
    .unwrap();
    panic!("handoff child did not exit");
}
fn matrix(native: bool) {
    let (mut source, mut keeper) = pair(native, true);
    let original_keeper = keeper.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .current
        .unwrap();
    keeper.namespaces[0]
        .select(
            &Select {
                prepare: k("keeper-next-prepare"),
                adopt: k("keeper-next-adopt"),
                source: k("asset"),
                owner: k("owner"),
                expected_revision: original_keeper.revision,
                selection: git_asset_store::source_claims::Selection {
                    root: original_keeper.selection.root.clone(),
                    context: Digest::blob(b"newer retained cohort"),
                },
            },
            &keeper.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    let plan = frozen(&mut source, &keeper);
    let before = source.namespaces[0].checkpoint();
    assert!(
        transfer::Plan::new(
            &source.plan,
            &source.plan,
            keeper_snapshot(&source),
            &mut source.namespaces[0]
        )
        .is_err()
    );
    assert!(matches!(
        transfer::Execution {
            source: &mut source.publisher(),
            keeper: &mut keeper.publisher(),
            allow_release: false
        }
        .apply(
            &plan,
            &source.plan.clone(),
            &keeper.plan.clone(),
            &mut source.resources(),
            &mut keeper.resources()
        ),
        Err(Error::Denied)
    ));
    assert_eq!(source.namespaces[0].checkpoint(), before);
    source.save();
    keeper.save();
    fs::write(source.root.join("handoff-plan"), plan.to_json().unwrap()).unwrap();
    let child = Process::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "annex::publication::tests::native::handoff::handoff_crash_worker",
            "--nocapture",
        ])
        .env("GIT_ASSET_HANDOFF_SOURCE", &source.root)
        .env("GIT_ASSET_HANDOFF_KEEPER", &keeper.root)
        .output()
        .unwrap();
    assert_eq!(
        child.status.code(),
        Some(86),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
    let partial = plan.status(&mut source.namespaces[0]).unwrap();
    assert!(partial.withdraw.is_some());
    assert!(partial.retire.is_none());
    let state = source.namespaces[0].source(&k("asset")).unwrap().unwrap();
    assert!(state.current.is_none());
    assert_eq!(state.predecessors.len(), 1);
    // Source bytes and hard history survive abrupt exit; normal publication
    // reopening also revalidates the exact original completion evidence.
    source
        .publisher()
        .reopen(&source.plan.clone(), &mut source.resources())
        .unwrap();
    source.namespaces[0] = Namespace::open(&source.root.join("namespace-0"), &before).unwrap();
    let completed = apply(&mut source, &mut keeper, &plan).unwrap();
    assert!(!completed.historical);
    assert!(completed.status.retire.is_some());
    assert!(completed.keeper_recovery.is_some());
    let state = source.namespaces[0].source(&k("asset")).unwrap().unwrap();
    assert!(state.current.is_none() && state.predecessors.is_empty());
    let mut kept = keeper
        .publisher()
        .reopen(&keeper.plan.clone(), &mut keeper.resources())
        .unwrap();
    let mut bytes = vec![];
    kept.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, vec![83; 256 * 1024]);
    drop(kept);
    let mut raw = crate::annex::Reader::new(source.profile.pool.pool.clone()).unwrap();
    let still_present = raw.open(&source.plan.candidate().object).unwrap();
    assert_eq!(still_present.object(), &source.plan.candidate().object);
    drop(still_present);
    drop(raw);
    // Completed historical retry does not invent fresh custody or need pools.
    let cp = source.namespaces[0].checkpoint();
    let source_config = source.profile.pool.pool.repository.join(".git/config");
    let keeper_config = keeper.profile.pool.pool.repository.join("config");
    fs::rename(&source_config, source_config.with_extension("offline")).unwrap();
    fs::rename(&keeper_config, keeper_config.with_extension("offline")).unwrap();
    let historical = apply(&mut source, &mut keeper, &plan).unwrap();
    assert!(historical.historical);
    assert!(historical.keeper_recovery.is_none());
    assert_eq!(source.namespaces[0].checkpoint(), cp);
    // Handoff never removes media, even though the old publication is no longer
    // protected by its ingest source. All actual eviction checks remain separate.
    assert!(
        plan.status(&mut source.namespaces[0])
            .unwrap()
            .retire
            .is_some()
    );
}
#[test]
fn original_ingest_handoff_survives_child_exit_and_offline_historical_retry() {
    matrix(false);
}
#[test]
#[ignore = "explicit native tools; fresh synthetic pools only"]
fn actual_native_ingest_handoff_retains_keeper_across_child_exit() {
    matrix(true);
}
#[test]
fn unavailable_keeper_and_legacy_fencing_never_release_ingest() {
    for fenced in [false, true] {
        let (mut source, mut keeper) = pair(false, fenced);
        let plan = frozen(&mut source, &keeper);
        let before = source.namespaces[0].checkpoint();
        if fenced {
            let path = keeper.output_path();
            fs::rename(&path, path.with_extension("offline")).unwrap();
            assert!(matches!(
                apply(&mut source, &mut keeper, &plan),
                Err(Error::MissingAcknowledged)
            ));
            fs::rename(path.with_extension("offline"), &path).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            #[allow(
                clippy::permissions_set_readonly_false,
                reason = "owned synthetic fixture corruption test"
            )]
            permissions.set_readonly(false);
            fs::set_permissions(&path, permissions).unwrap();
            fs::write(&path, b"corrupt synthetic keeper").unwrap();
            assert!(matches!(
                apply(&mut source, &mut keeper, &plan),
                Err(Error::Integrity)
            ));
        } else {
            assert!(matches!(
                apply(&mut source, &mut keeper, &plan),
                Err(Error::Unsupported)
            ));
        }
        assert_eq!(source.namespaces[0].checkpoint(), before);
        assert!(
            source.namespaces[0]
                .source(&k("asset"))
                .unwrap()
                .unwrap()
                .current
                .is_some()
        );
    }
}
#[test]
fn handoff_retires_only_original_predecessor_and_refuses_stale_baseline() {
    let (mut source, mut keeper) = pair(false, true);
    let stale = frozen(&mut source, &keeper);
    let original = source.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .current
        .unwrap();
    // A later retained selection of the same bytes is a distinct Version.
    source.namespaces[0]
        .select(
            &Select {
                prepare: k("next-prepare"),
                adopt: k("next-adopt"),
                source: k("asset"),
                owner: k("owner"),
                expected_revision: original.revision,
                selection: original.selection.clone(),
            },
            &source.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    let current = source.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .current
        .unwrap();
    assert_ne!(original.revision, current.revision);
    let cp = source.namespaces[0].checkpoint();
    assert!(matches!(
        apply(&mut source, &mut keeper, &stale),
        Err(Error::Stale)
    ));
    assert_eq!(source.namespaces[0].checkpoint(), cp);
    let mut forged = serde_json::to_value(&stale).unwrap();
    forged["revision"] = serde_json::json!(
        source.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .revision
    );
    let forged = transfer::Plan::from_json(&identity::canonical_json(&forged).unwrap()).unwrap();
    assert!(matches!(
        apply(&mut source, &mut keeper, &forged),
        Err(Error::Conflict)
    ));
    assert_eq!(source.namespaces[0].checkpoint(), cp);
    let plan = frozen(&mut source, &keeper);
    let report = apply(&mut source, &mut keeper, &plan).unwrap();
    assert!(report.status.withdraw.is_none());
    let state = source.namespaces[0].source(&k("asset")).unwrap().unwrap();
    assert_eq!(state.current, Some(current));
    assert!(state.predecessors.is_empty());
}

#[test]
fn active_job_survives_ingest_handoff_and_blocks_native_eviction() {
    let (mut source, mut keeper) = pair(false, true);
    let selection = source.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .current
        .unwrap()
        .selection;
    source.namespaces[0]
        .apply(
            k("enroll-live-job"),
            Command::Enroll {
                source: k("live-job"),
                owner: k("owner"),
                class: Kind::Job,
            },
            &source.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    source.namespaces[0]
        .select(
            &Select {
                prepare: k("job-prepare"),
                adopt: k("job-adopt"),
                source: k("live-job"),
                owner: k("owner"),
                expected_revision: 1,
                selection: selection.clone(),
            },
            &source.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    let plan = frozen(&mut source, &keeper);
    apply(&mut source, &mut keeper, &plan).unwrap();
    assert_eq!(
        source.namespaces[0]
            .source(&k("live-job"))
            .unwrap()
            .unwrap()
            .current
            .unwrap()
            .selection,
        selection
    );
    assert_eq!(
        crate::annex::retirement::Assessor::new(source.profile.pool.clone())
            .unwrap()
            .assess(
                &source.plan.candidate().object,
                &source.metadata,
                &Fixture::limits()
            )
            .err()
            .unwrap()
            .code(),
        "protected"
    );
}

fn cohort(native: bool, same_pool: bool) {
    let (mut source, mut keeper) = pair(native, true);
    let mut metadata =
        MetadataStore::open(&keeper.root.join("metadata"), keeper.metadata.checkpoint()).unwrap();
    let path = LogicalPath::parse("cohort/kept").unwrap();
    let unrelated = Object::from_bytes(b"unrelated retained member, deliberately unavailable");
    let (tree, _) = Tree::build(
        &BTreeMap::from([
            (path.clone(), keeper.plan.candidate().object.clone()),
            (LogicalPath::parse("cohort/unrelated").unwrap(), unrelated),
        ]),
        &mut metadata,
        TreeLimits::default(),
    )
    .unwrap();
    let checkpoint = metadata.finish().unwrap();
    drop(metadata);
    keeper.metadata = MetadataReader::open(&keeper.root.join("metadata"), &checkpoint).unwrap();
    keeper.namespaces[0]
        .apply(
            k("enroll-cohort"),
            Command::Enroll {
                source: k("cohort"),
                owner: k("owner"),
                class: Kind::Retention,
            },
            &keeper.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    keeper.namespaces[0]
        .select(
            &Select {
                prepare: k("cohort-prepare"),
                adopt: k("cohort-adopt"),
                source: k("cohort"),
                owner: k("owner"),
                expected_revision: 1,
                selection: git_asset_store::source_claims::Selection {
                    root: tree.root().clone(),
                    context: Digest::blob(b"retained cohort"),
                },
            },
            &keeper.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    let version = keeper.namespaces[0]
        .source(&k("cohort"))
        .unwrap()
        .unwrap()
        .current
        .unwrap();
    let snapshot = crate::annex::snapshot::Snapshot {
        namespace: keeper.namespaces[0].identity().clone(),
        source: k("cohort"),
        owner: k("owner"),
        version: version.clone(),
        place: k("pool"),
        path,
        object: source.plan.candidate().object.clone(),
    };
    if same_pool {
        keeper.save();
        let mut alias = Fixture::load(keeper.root.clone());
        let plan = transfer::Plan::new(
            &alias.plan,
            &keeper.plan,
            snapshot,
            &mut alias.namespaces[0],
        )
        .unwrap();
        apply(&mut alias, &mut keeper, &plan).unwrap();
        assert!(
            alias.namespaces[0]
                .source(&k("asset"))
                .unwrap()
                .unwrap()
                .predecessors
                .is_empty()
        );
        assert_eq!(
            keeper.namespaces[0]
                .source(&k("cohort"))
                .unwrap()
                .unwrap()
                .current,
            Some(version)
        );
        return;
    }
    retirement::settle(&mut keeper);
    assert!(matches!(
        keeper
            .publisher()
            .reopen(&keeper.plan.clone(), &mut keeper.resources()),
        Err(Error::ProtectionLost)
    ));
    let plan = transfer::Plan::new(
        &source.plan,
        &keeper.plan,
        snapshot,
        &mut source.namespaces[0],
    )
    .unwrap();
    let report = apply(&mut source, &mut keeper, &plan).unwrap();
    assert!(report.status.retire.is_some());
    assert!(report.keeper_snapshot_minimum.is_some());
    assert_eq!(
        keeper.namespaces[0]
            .source(&k("cohort"))
            .unwrap()
            .unwrap()
            .current,
        Some(version)
    );
    assert!(
        source.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .predecessors
            .is_empty()
    );
}

#[test]
fn flat_keeper_cohort_replaces_per_asset_ingest_without_retiring_other_members() {
    cohort(false, false);
}
#[test]
#[ignore = "explicit native tools; fresh synthetic pools only"]
fn actual_native_flat_cohort_replaces_keeper_ingest_claim() {
    cohort(true, false);
}

#[test]
fn same_pool_cohort_replaces_only_temporary_ingest_claim() {
    cohort(false, true);
}
#[test]
#[ignore = "explicit native tools; fresh synthetic pool only"]
fn actual_native_same_pool_cohort_handoff() {
    cohort(true, true);
}
