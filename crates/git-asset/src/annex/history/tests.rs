use super::*;
use git_asset_core::tree::MemoryNodes;
use git_asset_store::{
    metadata,
    source_claims::{Checkpoint, Places, admission::Policy},
};
use std::{fs, path::PathBuf, process::Command as Process};
fn k(s: &str) -> Key {
    Key::parse(s).unwrap()
}
fn limits() -> Limits {
    Limits {
        millis: 60000,
        roots: 256,
        bindings: 20000,
        associations: 100000,
        reads: 100000,
        read_bytes: 32 * 1024 * 1024,
    }
}
fn namespace(root: &std::path::Path, name: &str, place: &str) -> Namespace {
    let places = Places {
        current: vec![k(place)],
        pending: vec![k(place)],
        predecessor: vec![k(place)],
    };
    Namespace::create_fenced(
        &root.join(name),
        k(name),
        k("epoch"),
        Policy {
            prefer_active: true,
            classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                .into_iter()
                .map(|c| (c, places.clone()))
                .collect(),
        },
    )
    .unwrap()
}
fn select(ns: &mut Namespace, nodes: &MemoryNodes, revision: u64, root: Object, name: &str) {
    ns.select(
        &Select {
            prepare: k(&format!("{name}-prepare")),
            adopt: k(&format!("{name}-adopt")),
            source: k("main"),
            owner: k("owner"),
            expected_revision: revision,
            selection: Selection {
                root,
                context: Digest::blob(name.as_bytes()),
            },
        },
        nodes,
        &limits(),
    )
    .unwrap();
}
struct Fixture {
    temp: tempfile::TempDir,
    working: Namespace,
    media: Namespace,
    records: Namespace,
    input: MemoryNodes,
    nodes: MetadataStore,
    profile: Profile,
    origin: Version,
    object: Object,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let mut working = namespace(temp.path(), "working", "fast");
        let media = namespace(temp.path(), "retained", "cold");
        let records = namespace(temp.path(), "records", "metadata");
        let mut input = MemoryNodes::default();
        let object = Object::from_bytes(b"old bytes, deliberately not a metadata node");
        let old = BTreeMap::from([
            (LogicalPath::parse("assets/old").unwrap(), object.clone()),
            (
                LogicalPath::parse("assets/unavailable").unwrap(),
                Object::from_bytes(b"unavailable media"),
            ),
        ]);
        let (tree, _) = Tree::build(&old, &mut input, TreeLimits::default()).unwrap();
        working
            .apply(
                k("enroll-main"),
                Command::Enroll {
                    source: k("main"),
                    owner: k("owner"),
                    class: Kind::Branch,
                },
                &input,
                &limits(),
            )
            .unwrap();
        select(&mut working, &input, 1, tree.root().clone(), "old");
        let origin = working
            .source(&k("main"))
            .unwrap()
            .unwrap()
            .current
            .unwrap();
        let (next, _) = Tree::build(
            &BTreeMap::from([(
                LogicalPath::parse("assets/new").unwrap(),
                Object::from_bytes(b"new bytes"),
            )]),
            &mut input,
            TreeLimits::default(),
        )
        .unwrap();
        select(&mut working, &input, 3, next.root().clone(), "new");
        let nodes = MetadataStore::create(
            &temp.path().join("metadata"),
            metadata::Limits {
                nodes: 20000,
                bytes: 16 * 1024 * 1024,
                database_bytes: 32 * 1024 * 1024,
                min_free_bytes: 65536,
            },
        )
        .unwrap();
        let profile = Profile {
            working: Working {
                namespace: working.identity().clone(),
                source: k("main"),
                owner: k("owner"),
                class: Kind::Branch,
            },
            target: Target {
                namespace: media.identity().clone(),
                owner: k("owner"),
                place: k("cold"),
                prefix: k("cohort"),
            },
            records: pending::Scope {
                namespace: records.identity().clone(),
                metadata: nodes.checkpoint().identity().clone(),
                owner: k("owner"),
                prefix: k("history"),
                place: k("metadata"),
                allow_store: true,
            },
            limits: limits(),
            allow_publish: true,
            allow_release: true,
        };
        Self {
            temp,
            working,
            media,
            records,
            input,
            nodes,
            profile,
            origin,
            object,
        }
    }
    fn plan(&mut self) -> Plan {
        Plan::capture(&self.profile, &mut self.working, &self.origin).unwrap()
    }
    fn publish(&mut self, plan: &Plan) -> Loaded {
        publish(
            &self.profile,
            plan,
            &mut self.working,
            Destinations {
                media: &mut self.media,
                records: &mut self.records,
            },
            &self.input,
            &mut self.nodes,
            |_| Ok(()),
        )
        .unwrap()
    }
}
#[test]
fn exact_metadata_archive_and_release_preserve_current_and_report_unknown_media() {
    let mut f = Fixture::new();
    let plan = f.plan();
    let current = f.working.source(&k("main")).unwrap().unwrap().current;
    let loaded = f.publish(&plan);
    assert_eq!(loaded.plan.origin(), &f.origin);
    assert_eq!(loaded.media.selection.root, f.origin.selection.root);
    assert!(
        f.nodes.get(&f.object).unwrap().is_none(),
        "publication must not copy payloads"
    );
    let mut reader = f.nodes.finished_reader().unwrap();
    let member = loaded
        .member(
            &f.profile,
            LogicalPath::parse("assets/old").unwrap(),
            &mut reader,
        )
        .unwrap();
    assert_eq!(member.object, f.object);
    member
        .verify(&mut f.media, &mut reader, TreeLimits::default())
        .unwrap();
    assert_eq!(
        loaded
            .member(
                &f.profile,
                LogicalPath::parse("assets/new").unwrap(),
                &mut reader
            )
            .unwrap_err(),
        Error::Missing
    );
    let mut denied = f.profile.clone();
    denied.allow_release = false;
    assert!(matches!(
        loaded.release(
            &denied,
            &mut f.working,
            &mut f.media,
            (&f.input, &mut reader),
            || Ok(())
        ),
        Err(Error::Denied)
    ));
    let mut alias =
        Namespace::open(&f.temp.path().join("retained"), &f.media.checkpoint()).unwrap();
    let callback_reader = f.nodes.finished_reader().unwrap();
    let media_source = alias.source(&member.source).unwrap().unwrap();
    let report = loaded
        .release(
            &f.profile,
            &mut f.working,
            &mut f.media,
            (&f.input, &mut reader),
            || {
                assert!(matches!(
                    alias.apply(
                        k("blocked-withdraw"),
                        Command::Withdraw {
                            source: member.source.clone(),
                            owner: k("owner"),
                            expected_revision: media_source.revision
                        },
                        &callback_reader,
                        &limits()
                    ),
                    Err(git_asset_store::source_claims::admission::Error::Journal(
                        git_asset_store::source_claims::Error::Busy
                    ))
                ));
                alias
                    .apply(
                        k("unrelated-enroll"),
                        Command::Enroll {
                            source: k("unrelated"),
                            owner: k("owner"),
                            class: Kind::Branch,
                        },
                        &callback_reader,
                        &limits(),
                    )
                    .unwrap();
                Ok(())
            },
        )
        .unwrap();
    assert!(!report.historical && !report.authorizes_eviction);
    assert_eq!(report.media_custody_verified, None);
    let source = f.working.source(&k("main")).unwrap().unwrap();
    assert_eq!(source.current, current);
    assert!(!source.predecessors.contains(&f.origin));
    let replay = loaded
        .release(
            &denied,
            &mut f.working,
            &mut f.media,
            (&f.input, &mut reader),
            || panic!("historical inspection is not a new command"),
        )
        .unwrap();
    assert!(replay.historical);
    assert_eq!(replay.receipt.reference, report.receipt.reference);
    assert_eq!(replay.receipt.checkpoint, report.receipt.checkpoint);
    assert_eq!(
        replay.receipt.source_revision,
        report.receipt.source_revision
    );
    assert!(replay.receipt.replayed);
}
#[test]
fn interrupted_media_adoption_retries_original_and_readonly_recovery_wins() {
    let mut f = Fixture::new();
    let plan = f.plan();
    let before = f.working.checkpoint();
    let err = publish(
        &f.profile,
        &plan,
        &mut f.working,
        Destinations {
            media: &mut f.media,
            records: &mut f.records,
        },
        &f.input,
        &mut f.nodes,
        |stage| {
            if matches!(stage, Stage::MediaAdopted) {
                Err(Error::NeedsRecovery)
            } else {
                Ok(())
            }
        },
    )
    .err()
    .unwrap();
    assert_eq!(err, Error::NeedsRecovery);
    assert_eq!(f.working.checkpoint(), before);
    let media_before = f.media.checkpoint();
    let loaded = f.publish(&plan);
    assert_eq!(f.media.checkpoint(), media_before);
    let saved = loaded.saved.clone();
    let mut readonly = f.profile.clone();
    readonly.allow_publish = false;
    readonly.records.allow_store = false;
    let checkpoint = f.nodes.checkpoint();
    let replay = publish(
        &readonly,
        &plan,
        &mut f.working,
        Destinations {
            media: &mut f.media,
            records: &mut f.records,
        },
        &f.input,
        &mut f.nodes,
        |_| panic!("recovery must not publish"),
    )
    .unwrap();
    assert_eq!(saved, replay.saved);
    assert_eq!(checkpoint, f.nodes.checkpoint());
}
#[test]
fn permissions_corruption_and_budget_never_release_or_adopt_incomplete_roots() {
    let mut f = Fixture::new();
    let plan = f.plan();
    let before = (
        f.working.checkpoint(),
        f.media.checkpoint(),
        f.records.checkpoint(),
        f.nodes.checkpoint(),
    );
    let mut denied = f.profile.clone();
    denied.allow_publish = false;
    assert!(matches!(
        publish(
            &denied,
            &plan,
            &mut f.working,
            Destinations {
                media: &mut f.media,
                records: &mut f.records
            },
            &f.input,
            &mut f.nodes,
            |_| Ok(())
        ),
        Err(Error::Denied)
    ));
    struct Corrupt;
    impl NodeStore for Corrupt {
        fn get(&self, _: &Object) -> TreeResult<Option<Vec<u8>>> {
            Ok(Some(b"wrong metadata".to_vec()))
        }
        fn put(&mut self, _: &[u8]) -> TreeResult<Object> {
            unreachable!()
        }
    }
    assert!(matches!(
        publish(
            &f.profile,
            &plan,
            &mut f.working,
            Destinations {
                media: &mut f.media,
                records: &mut f.records
            },
            &Corrupt,
            &mut f.nodes,
            |_| Ok(())
        ),
        Err(Error::Integrity)
    ));
    assert_eq!(f.nodes.checkpoint(), before.3);
    let mut tiny = f.profile.clone();
    tiny.limits.reads = 1;
    assert!(matches!(
        publish(
            &tiny,
            &plan,
            &mut f.working,
            Destinations {
                media: &mut f.media,
                records: &mut f.records
            },
            &f.input,
            &mut f.nodes,
            |_| Ok(())
        ),
        Err(Error::Budget)
    ));
    assert_eq!(
        (
            f.working.checkpoint(),
            f.media.checkpoint(),
            f.records.checkpoint()
        ),
        (before.0, before.1, before.2)
    );
    assert!(
        f.working
            .source(&k("main"))
            .unwrap()
            .unwrap()
            .predecessors
            .contains(&f.origin)
    );
}
#[test]
fn advance_after_last_generation_guard_refuses_original_release() {
    let mut f = Fixture::new();
    let plan = f.plan();
    let loaded = f.publish(&plan);
    let mut reader = f.nodes.finished_reader().unwrap();
    let mut concurrent =
        Namespace::open(&f.temp.path().join("working"), &f.working.checkpoint()).unwrap();
    let mut calls = 0;
    let result = loaded.release(
        &f.profile,
        &mut f.working,
        &mut f.media,
        (&f.input, &mut reader),
        || {
            calls += 1;
            if calls == 2 {
                select(
                    &mut concurrent,
                    &f.input,
                    plan.revision,
                    plan.current.selection.root.clone(),
                    "concurrent",
                );
            }
            Ok(())
        },
    );
    assert!(matches!(result, Err(Error::Stale)));
    assert!(
        f.working
            .source(&k("main"))
            .unwrap()
            .unwrap()
            .predecessors
            .contains(&f.origin)
    );
    let saved = loaded.saved.clone();
    let media_before = f.media.checkpoint();
    let next = refresh(
        &f.profile,
        loaded,
        &mut f.working,
        Destinations {
            media: &mut f.media,
            records: &mut f.records,
        },
        &mut f.nodes,
        |_| Ok(()),
    )
    .unwrap();
    assert!(next.plan.revision > plan.revision);
    assert_eq!(next.saved.source, saved.source);
    assert!(next.saved.version.revision > saved.version.revision);
    assert_eq!(
        f.media.checkpoint(),
        media_before,
        "refresh must not retarget the archived tree"
    );
    let previous = load(
        &f.profile,
        &saved,
        Destinations {
            media: &mut f.media,
            records: &mut f.records,
        },
        &f.nodes,
    )
    .unwrap();
    assert_eq!(
        previous.plan.id().unwrap(),
        plan.id().unwrap(),
        "exact old metadata remains readable"
    );
    let mut reader = f.nodes.finished_reader().unwrap();
    next.release(
        &f.profile,
        &mut f.working,
        &mut f.media,
        (&f.input, &mut reader),
        || Ok(()),
    )
    .unwrap();
    assert!(matches!(
        previous.release(
            &f.profile,
            &mut f.working,
            &mut f.media,
            (&f.input, &mut reader),
            || Ok(())
        ),
        Err(Error::Stale)
    ));
}
#[test]
fn incomplete_record_enrollment_can_finish_after_main_advances() {
    let mut f = Fixture::new();
    let plan = f.plan();
    let result = publish(
        &f.profile,
        &plan,
        &mut f.working,
        Destinations {
            media: &mut f.media,
            records: &mut f.records,
        },
        &f.input,
        &mut f.nodes,
        |stage| {
            if matches!(stage, Stage::RecordEnrolled) {
                Err(Error::NeedsRecovery)
            } else {
                Ok(())
            }
        },
    );
    assert!(matches!(result, Err(Error::NeedsRecovery)));
    select(
        &mut f.working,
        &f.input,
        plan.revision,
        plan.current.selection.root.clone(),
        "later",
    );
    let next = f.plan();
    let loaded = f.publish(&next);
    assert_eq!(loaded.plan.id().unwrap(), next.id().unwrap());
    let mut reader = f.nodes.finished_reader().unwrap();
    loaded
        .release(
            &f.profile,
            &mut f.working,
            &mut f.media,
            (&f.input, &mut reader),
            || Ok(()),
        )
        .unwrap();
}
#[test]
fn cumulative_read_budget_refuses_before_any_source_adoption() {
    let mut f = Fixture::new();
    let plan = f.plan();
    let (_, metrics) = Tree::from_root(f.origin.selection.root.clone())
        .entries(&mut f.input, TreeLimits::default())
        .unwrap();
    let mut bounded = f.profile.clone();
    bounded.limits.reads = metrics.reads * 2 + 2;
    let before = (
        f.working.checkpoint(),
        f.media.checkpoint(),
        f.records.checkpoint(),
    );
    let result = publish(
        &bounded,
        &plan,
        &mut f.working,
        Destinations {
            media: &mut f.media,
            records: &mut f.records,
        },
        &f.input,
        &mut f.nodes,
        |_| Ok(()),
    );
    assert!(matches!(result, Err(Error::Budget)));
    assert_eq!(
        (
            f.working.checkpoint(),
            f.media.checkpoint(),
            f.records.checkpoint()
        ),
        before
    );
}
#[derive(Serialize, Deserialize)]
struct Reopen {
    profile: Profile,
    saved: pending::Saved,
    media: PathBuf,
    media_minimum: Checkpoint,
    records: PathBuf,
    records_minimum: Checkpoint,
    nodes: PathBuf,
    minimum: metadata::Checkpoint,
    object: Object,
}
#[test]
fn reopen_worker() {
    let Some(path) = std::env::var_os("GIT_ASSET_HISTORY_REOPEN") else {
        return;
    };
    let data: Reopen = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let mut media = Namespace::open(&data.media, &data.media_minimum).unwrap();
    let mut records = Namespace::open(&data.records, &data.records_minimum).unwrap();
    let nodes = MetadataStore::open(&data.nodes, &data.minimum).unwrap();
    let loaded = load(
        &data.profile,
        &data.saved,
        Destinations {
            media: &mut media,
            records: &mut records,
        },
        &nodes,
    )
    .unwrap();
    let mut reader = nodes.finished_reader().unwrap();
    let member = loaded
        .member(
            &data.profile,
            LogicalPath::parse("assets/old").unwrap(),
            &mut reader,
        )
        .unwrap();
    assert_eq!(member.object, data.object);
    member
        .verify(&mut media, &mut reader, TreeLimits::default())
        .unwrap();
}
#[test]
fn separate_process_reopens_historical_member_without_working_checkout() {
    let mut f = Fixture::new();
    let plan = f.plan();
    let loaded = f.publish(&plan);
    let data = Reopen {
        profile: f.profile.clone(),
        saved: loaded.saved.clone(),
        media: f.temp.path().join("retained"),
        media_minimum: f.media.checkpoint(),
        records: f.temp.path().join("records"),
        records_minimum: f.records.checkpoint(),
        nodes: f.temp.path().join("metadata"),
        minimum: f.nodes.checkpoint(),
        object: f.object.clone(),
    };
    let path = f.temp.path().join("reopen.json");
    fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    drop(loaded);
    drop(f.working);
    fs::rename(
        f.temp.path().join("working"),
        f.temp.path().join("working-offline"),
    )
    .unwrap();
    let child = Process::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "annex::history::tests::reopen_worker",
            "--nocapture",
        ])
        .env("GIT_ASSET_HISTORY_REOPEN", &path)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
}
