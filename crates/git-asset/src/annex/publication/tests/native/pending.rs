use super::*;
use crate::annex::pending::{self, Action, Operation, Saved, Scope};
#[derive(Serialize, Deserialize)]
struct SavedFixture {
    scope: Scope,
    saved: Saved,
    namespace: PathBuf,
    minimum: Checkpoint,
    metadata: PathBuf,
    metadata_minimum: git_asset_store::metadata::Checkpoint,
    plan: Digest,
}
#[test]
fn reopen_worker() {
    let Some(path) = std::env::var_os("GIT_ASSET_PENDING_REOPEN_FIXTURE") else {
        return;
    };
    let data: SavedFixture = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let mut ns = Namespace::open(&data.namespace, &data.minimum).unwrap();
    let nodes = MetadataStore::open(&data.metadata, &data.metadata_minimum).unwrap();
    let loaded = pending::reopen(
        &data.scope,
        &data.saved,
        &mut ns,
        &nodes,
        &Fixture::limits(),
    )
    .unwrap();
    match loaded.action.operation {
        Operation::Handoff(ref p) => assert_eq!(p.id().unwrap(), data.plan),
        Operation::Restore(ref p) => {
            assert_eq!(Digest::blob(&p.restore.to_json().unwrap()), data.plan)
        }
        Operation::RestoreIndex(ref p) => {
            assert_eq!(Digest::blob(&p.index.to_json().unwrap()), data.plan)
        }
        Operation::Registration(ref p) => {
            assert_eq!(Digest::blob(&p.index.to_json().unwrap()), data.plan)
        }
        _ => panic!("wrong operation"),
    };
}
#[test]
fn saved_original_survives_process_restart_denied_store_and_competing_preview() {
    let mut f = Fixture::build_setup(b"queue fixture", None, false, None, true);
    let mut m = Fixture::build_setup(b"queue fixture", None, false, None, true);
    retirement::publish(&mut f);
    retirement::publish(&mut m);
    let snapshot = retirement::snapshot(&mut m);
    let plan = crate::annex::publication::handoff::Plan::new(
        &f.plan,
        &m.plan,
        snapshot.clone(),
        &mut f.namespaces[0],
    )
    .unwrap();
    let action = Action {
        operation: Operation::Handoff(Box::new(plan.clone())),
        working: f.plan.clone(),
        retained: m.plan.clone(),
    };
    let mut ns = super::github_worker::batch::pending_namespace(&f);
    let metadata = f.root.join("pending-metadata");
    let mut nodes = MetadataStore::create(
        &metadata,
        MetadataLimits {
            nodes: 1000,
            bytes: 1024 * 1024,
            database_bytes: 4 * 1024 * 1024,
            min_free_bytes: 65536,
        },
    )
    .unwrap();
    let mut scope = Scope {
        namespace: ns.identity().clone(),
        metadata: nodes.checkpoint().identity().clone(),
        owner: k("owner"),
        prefix: k("pending"),
        place: k("metadata"),
        allow_store: false,
    };
    let policy = Digest::blob(b"policy");
    let commit = "1111111111111111111111111111111111111111";
    let before = nodes.checkpoint();
    assert_eq!(
        pending::save(
            &scope,
            policy.clone(),
            commit,
            &action,
            &mut ns,
            &mut nodes,
            &Fixture::limits()
        )
        .unwrap_err(),
        Error::Denied
    );
    assert_eq!(nodes.checkpoint(), before);
    scope.allow_store = true;
    let invalid = git_asset_store::protection::Limits {
        reads: 0,
        ..Fixture::limits()
    };
    assert_eq!(
        pending::save(
            &scope,
            policy.clone(),
            commit,
            &action,
            &mut ns,
            &mut nodes,
            &invalid
        )
        .unwrap_err(),
        Error::Budget
    );
    assert_eq!(nodes.checkpoint(), before);
    nodes.finished_reader().unwrap();
    let saved = pending::save(
        &scope,
        policy.clone(),
        commit,
        &action,
        &mut ns,
        &mut nodes,
        &Fixture::limits(),
    )
    .unwrap();
    let loaded = pending::reopen(&scope, &saved, &mut ns, &nodes, &Fixture::limits()).unwrap();
    assert!(matches!(
        ns.apply(
            k("blocked-withdraw"),
            Command::Withdraw {
                source: saved.source.clone(),
                owner: k("owner"),
                expected_revision: saved.version.revision
            },
            &nodes,
            &Fixture::limits()
        ),
        Err(admission::Error::Journal(
            git_asset_store::source_claims::Error::Busy
        ))
    ));
    // Metadata operations outside this action's source gate still progress.
    ns.apply(
        k("other-enrol"),
        Command::Enroll {
            source: k("other"),
            owner: k("owner"),
            class: Kind::Job,
        },
        &nodes,
        &Fixture::limits(),
    )
    .unwrap();
    drop(loaded);
    let mut changed = snapshot;
    changed.path = LogicalPath::parse("different-member").unwrap();
    let changed = crate::annex::publication::handoff::Plan::new(
        &f.plan,
        &m.plan,
        changed,
        &mut f.namespaces[0],
    )
    .unwrap();
    assert_ne!(changed.id().unwrap(), plan.id().unwrap());
    let alternative = Action {
        operation: Operation::Handoff(Box::new(changed)),
        ..action
    };
    let before = nodes.checkpoint();
    scope.allow_store = false;
    let again = pending::save(
        &scope,
        Digest::blob(b"new policy"),
        commit,
        &alternative,
        &mut ns,
        &mut nodes,
        &Fixture::limits(),
    )
    .unwrap();
    assert_eq!(again, saved);
    assert_eq!(nodes.checkpoint(), before);
    let limits = git_asset_store::protection::Limits {
        reads: 1,
        ..Fixture::limits()
    };
    assert!(matches!(
        pending::reopen(&scope, &saved, &mut ns, &nodes, &limits),
        Err(Error::Budget)
    ));
    let mut corrupt = saved.clone();
    corrupt.record.bytes += 1;
    assert!(matches!(
        pending::reopen(&scope, &corrupt, &mut ns, &nodes, &Fixture::limits()),
        Err(Error::Integrity)
    ));
    let mut wrong = scope.clone();
    wrong.metadata = Digest::blob(b"wrong provider");
    assert!(matches!(
        pending::reopen(&wrong, &saved, &mut ns, &nodes, &Fixture::limits()),
        Err(Error::Stale)
    ));
    let data = SavedFixture {
        scope: scope.clone(),
        saved: saved.clone(),
        namespace: f.root.join("pending-actions"),
        minimum: ns.checkpoint(),
        metadata: metadata.clone(),
        metadata_minimum: nodes.checkpoint(),
        plan: plan.id().unwrap(),
    };
    let config = f.root.join("pending-reopen.json");
    fs::write(&config, serde_json::to_vec(&data).unwrap()).unwrap();
    drop(nodes);
    let child = Process::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "annex::publication::tests::native::pending::reopen_worker",
            "--nocapture",
        ])
        .env("GIT_ASSET_PENDING_REOPEN_FIXTURE", &config)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}",
        String::from_utf8_lossy(&child.stdout)
    );
    let nodes = MetadataStore::open(&metadata, &data.metadata_minimum).unwrap();
    let loaded = pending::reopen(&scope, &saved, &mut ns, &nodes, &Fixture::limits()).unwrap();
    assert_eq!(loaded.policy, policy);
    assert_eq!(loaded.commit, commit);
}
