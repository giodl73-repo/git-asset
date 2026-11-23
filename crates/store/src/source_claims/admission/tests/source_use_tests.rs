use super::*;
use crate::source_claims::admission::source_use::UseError;
fn fixture() -> Fixture {
    let mut f = Fixture::new(true);
    f.namespace = Namespace::create_fenced(
        &f.temp.path().join("fenced"),
        key("project"),
        key("epoch"),
        policy(true),
    )
    .unwrap();
    f
}
fn root(f: &Fixture) -> PathBuf {
    f.temp.path().join("fenced")
}
fn version(f: &mut Fixture, name: &str) -> crate::source_claims::Version {
    f.namespace
        .source(&key(name))
        .unwrap()
        .unwrap()
        .current
        .unwrap()
}
fn select(f: &mut Fixture, name: &str, operation: &str) -> Select {
    Select {
        prepare: key(&format!("{operation}-prepare")),
        adopt: key(&format!("{operation}-adopt")),
        source: key(name),
        owner: key("owner"),
        expected_revision: f.namespace.source(&key(name)).unwrap().unwrap().revision,
        selection: f.new.clone(),
    }
}
#[test]
fn source_use_blocks_effects_but_not_history_or_unrelated_sources() {
    let mut f = fixture();
    f.enroll("z-job", Kind::Job);
    f.selected("z-job", f.old.clone());
    f.selected("z-job", f.new.clone());
    f.enroll("other", Kind::Job);
    f.selected("other", f.old.clone());
    let v = version(&mut f, "z-job");
    let held = f
        .namespace
        .source_use(&key("z-job"), &key("owner"), v.revision, &v)
        .unwrap();
    let second = f
        .namespace
        .source_use(&key("z-job"), &key("owner"), v.revision, &v)
        .unwrap();
    assert_eq!(held.namespace(), f.namespace.identity());
    assert_eq!(held.source(), &key("z-job"));
    assert_eq!(held.owner(), &key("owner"));
    assert_eq!(held.version(), &v);
    assert_eq!(held.selection(), &v.selection);
    let cp = f.namespace.checkpoint();
    let mut peer = Namespace::open(&root(&f), &cp).unwrap();
    let commands = vec![
        Command::Prepare {
            source: key("z-job"),
            owner: key("owner"),
            expected_revision: v.revision,
            selection: f.new.clone(),
        },
        Command::Withdraw {
            source: key("z-job"),
            owner: key("owner"),
            expected_revision: v.revision,
        },
        Command::Retire {
            source: key("z-job"),
            owner: key("owner"),
            expected_revision: v.revision,
            predecessor_revision: 3,
        },
    ];
    for (i, c) in commands.into_iter().enumerate() {
        assert_eq!(
            peer.apply(key(&format!("blocked-{i}")), c, &f.nodes, &limits()),
            Err(Error::Journal(crate::source_claims::Error::Busy))
        );
        assert_eq!(peer.observe(&cp).unwrap(), cp);
    }
    let historical = Select {
        prepare: key("prepare-z-job-1"),
        adopt: key("adopt-z-job-1"),
        source: key("z-job"),
        owner: key("owner"),
        expected_revision: 1,
        selection: f.old.clone(),
    };
    assert!(
        peer.select(&historical, &MemoryNodes::default(), &limits())
            .unwrap()
            .replayed
    );
    assert_eq!(
        peer.source(&key("z-job")).unwrap().unwrap().current,
        Some(v.clone())
    );
    assert_eq!(peer.checkpoint(), cp);
    let selection = select(&mut f, "z-job", "blocked-selection");
    assert_eq!(
        peer.select(&selection, &f.nodes, &limits()),
        Err(Error::Journal(crate::source_claims::Error::Busy))
    );
    assert_eq!(
        peer.select_predecessor(&selection, key("blocked-withdraw"), &f.nodes, &limits()),
        Err(Error::Journal(crate::source_claims::Error::Busy))
    );
    let replay = peer
        .apply(
            key("enroll-z-job"),
            Command::Enroll {
                source: key("z-job"),
                owner: key("owner"),
                class: Kind::Job,
            },
            &MemoryNodes::default(),
            &limits(),
        )
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(peer.checkpoint(), cp);
    let other = select(&mut f, "other", "unrelated");
    peer.select(&other, &f.nodes, &limits()).unwrap();
    peer.apply(
        key("lexically-earlier"),
        Command::Enroll {
            source: key("a-first"),
            owner: key("owner"),
            class: Kind::Job,
        },
        &f.nodes,
        &limits(),
    )
    .unwrap();
    let next = peer.checkpoint();
    let original_root = root(&f);
    drop(f.namespace);
    let mut reopened = Namespace::open(&original_root, &next).unwrap();
    let withdraw = Command::Withdraw {
        source: key("z-job"),
        owner: key("owner"),
        expected_revision: v.revision,
    };
    drop(held);
    assert_eq!(
        reopened.apply(key("cancel"), withdraw.clone(), &f.nodes, &limits()),
        Err(Error::Journal(crate::source_claims::Error::Busy))
    );
    drop(second);
    assert_eq!(
        reopened.source(&key("z-job")).unwrap().unwrap().current,
        Some(v)
    );
    reopened
        .apply(key("cancel"), withdraw, &f.nodes, &limits())
        .unwrap();
}
#[test]
fn source_use_rejects_wrong_owner_version_pending_and_legacy() {
    let mut f = fixture();
    f.enroll("job", Kind::Job);
    f.selected("job", f.old.clone());
    let v = version(&mut f, "job");
    assert!(matches!(
        f.namespace
            .source_use(&key("job"), &key("stranger"), v.revision, &v),
        Err(UseError::Owner)
    ));
    let mut wrong = v.clone();
    wrong.selection.context = Digest::blob(b"wrong");
    assert!(matches!(
        f.namespace
            .source_use(&key("job"), &key("owner"), wrong.revision, &wrong),
        Err(UseError::Stale)
    ));
    wrong = v.clone();
    wrong.revision += 1;
    assert!(matches!(
        f.namespace
            .source_use(&key("job"), &key("owner"), wrong.revision, &wrong),
        Err(UseError::Stale)
    ));
    assert!(matches!(
        f.namespace
            .source_use(&key("missing"), &key("owner"), v.revision, &v),
        Err(UseError::Inactive)
    ));
    f.namespace
        .apply(
            key("pending"),
            Command::Prepare {
                source: key("job"),
                owner: key("owner"),
                expected_revision: v.revision,
                selection: f.new.clone(),
            },
            &f.nodes,
            &limits(),
        )
        .unwrap();
    assert!(matches!(
        f.namespace
            .source_use(&key("job"), &key("owner"), v.revision, &v),
        Err(UseError::Inactive)
    ));
    let mut legacy = Fixture::new(true);
    let marker = fs::read(legacy.root().join("claims.json")).unwrap();
    assert!(matches!(
        legacy
            .namespace
            .source_use(&key("job"), &key("owner"), v.revision, &v),
        Err(UseError::Unsupported)
    ));
    assert_eq!(fs::read(legacy.root().join("claims.json")).unwrap(), marker);
    assert!(!legacy.root().join("source-gates").exists());
}
#[test]
fn source_use_missing_or_malformed_anchor_refuses_without_recreation() {
    for malformed in [false, true] {
        let mut f = fixture();
        f.enroll("job", Kind::Job);
        f.selected("job", f.old.clone());
        let v = version(&mut f, "job");
        let cp = f.namespace.checkpoint();
        let anchor = root(&f).join("source-gates/0000");
        if malformed {
            fs::write(&anchor, b"changed").unwrap();
        } else {
            fs::remove_file(&anchor).unwrap();
        }
        assert!(
            f.namespace
                .source_use(&key("job"), &key("owner"), v.revision, &v)
                .is_err()
        );
        assert_eq!(f.namespace.checkpoint(), cp);
        if malformed {
            assert_eq!(fs::read(&anchor).unwrap(), b"changed");
        } else {
            assert!(!anchor.exists());
        }
        assert!(
            f.namespace
                .apply(
                    key("refused"),
                    Command::Withdraw {
                        source: key("job"),
                        owner: key("owner"),
                        expected_revision: v.revision
                    },
                    &f.nodes,
                    &limits()
                )
                .is_err()
        );
        assert_eq!(f.namespace.checkpoint(), cp);
    }
}

#[derive(Serialize, Deserialize)]
struct UseInput {
    minimum: Checkpoint,
    version: crate::source_claims::Version,
}
#[test]
#[ignore = "invoked by source-use parent fixture"]
fn source_use_child() {
    let root = PathBuf::from(std::env::var_os("GIT_ASSET_SOURCE_USE_CHILD").unwrap());
    let input: UseInput =
        serde_json::from_slice(&fs::read(root.join("child-input.json")).unwrap()).unwrap();
    let mut ns = Namespace::open(&root, &input.minimum).unwrap();
    let _held = ns
        .source_use(
            &key("job"),
            &key("owner"),
            input.version.revision,
            &input.version,
        )
        .unwrap();
    drop(ns);
    fs::write(root.join("child-ready"), b"ready").unwrap();
    let deadline = Instant::now() + std::time::Duration::from_secs(30);
    while !root.join("child-exit").exists() {
        assert!(Instant::now() < deadline, "parent did not release child");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    std::process::exit(79);
}
#[test]
fn source_use_child_exit_releases_live_guard_without_withdrawing_durable_claim() {
    let mut f = fixture();
    f.enroll("job", Kind::Job);
    f.selected("job", f.old.clone());
    let v = version(&mut f, "job");
    let cp = f.namespace.checkpoint();
    let path = root(&f);
    fs::write(
        path.join("child-input.json"),
        serde_json::to_vec(&UseInput {
            minimum: cp.clone(),
            version: v.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    let mut child = Child(
        Process::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "source_claims::admission::tests::source_use_tests::source_use_child",
                "--ignored",
                "--nocapture",
            ])
            .env("GIT_ASSET_SOURCE_USE_CHILD", &path)
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + std::time::Duration::from_secs(30);
    while !path.join("child-ready").exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "child stopped before guard"
        );
        assert!(Instant::now() < deadline, "child readiness timeout");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let command = Command::Withdraw {
        source: key("job"),
        owner: key("owner"),
        expected_revision: v.revision,
    };
    assert_eq!(
        f.namespace
            .apply(key("cancel"), command.clone(), &f.nodes, &limits()),
        Err(Error::Journal(crate::source_claims::Error::Busy))
    );
    assert_eq!(f.namespace.checkpoint(), cp);
    f.enroll("unrelated", Kind::Job);
    fs::write(path.join("child-exit"), b"exit").unwrap();
    loop {
        if let Some(exit) = child.0.try_wait().unwrap() {
            assert_eq!(exit.code(), Some(79));
            break;
        }
        assert!(Instant::now() < deadline, "child exit timeout");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let mut reopened = Namespace::open(&path, &cp).unwrap();
    assert_eq!(
        reopened.source(&key("job")).unwrap().unwrap().current,
        Some(v.clone())
    );
    let held = reopened
        .source_use(&key("job"), &key("owner"), v.revision, &v)
        .unwrap();
    assert_eq!(
        reopened.apply(key("cancel"), command.clone(), &f.nodes, &limits()),
        Err(Error::Journal(crate::source_claims::Error::Busy))
    );
    drop(held);
    reopened
        .apply(key("cancel"), command, &f.nodes, &limits())
        .unwrap();
    let state = reopened.source(&key("job")).unwrap().unwrap();
    assert!(state.current.is_none());
    assert_eq!(state.predecessors, vec![v]);
}

#[test]
fn source_use_distinguishes_source_revision_from_selected_version() {
    let mut f = fixture();
    f.enroll("job", Kind::Job);
    f.selected("job", f.old.clone());
    let old = version(&mut f, "job");
    f.selected("job", f.new.clone());
    let current = version(&mut f, "job");
    let receipt = f
        .namespace
        .apply(
            key("retire-history"),
            Command::Retire {
                source: key("job"),
                owner: key("owner"),
                expected_revision: current.revision,
                predecessor_revision: old.revision,
            },
            &f.nodes,
            &limits(),
        )
        .unwrap();
    assert!(receipt.source_revision > current.revision);
    assert!(matches!(
        f.namespace
            .source_use(&key("job"), &key("owner"), current.revision, &current),
        Err(UseError::Stale)
    ));
    let held = f
        .namespace
        .source_use(
            &key("job"),
            &key("owner"),
            receipt.source_revision,
            &current,
        )
        .unwrap();
    assert_eq!(held.revision(), receipt.source_revision);
    assert_eq!(held.version(), &current);
    let cp = f.namespace.checkpoint();
    let mut direct = SourceJournal::open(&root(&f), &cp).unwrap();
    assert_eq!(
        direct.apply(
            key("raw-bypass"),
            Command::Withdraw {
                source: key("job"),
                owner: key("owner"),
                expected_revision: receipt.source_revision
            }
        ),
        Err(crate::source_claims::Error::AdmissionRequired)
    );
}
#[cfg(windows)]
#[test]
fn source_use_keeps_physical_control_path_after_namespace_is_dropped() {
    let mut f = fixture();
    f.enroll("job", Kind::Job);
    f.selected("job", f.old.clone());
    let v = version(&mut f, "job");
    let path = root(&f);
    let held = f
        .namespace
        .source_use(&key("job"), &key("owner"), v.revision, &v)
        .unwrap();
    drop(f.namespace);
    assert!(fs::rename(path.join("source-gates/0000"), path.join("moved-slot")).is_err());
    assert!(fs::rename(path.join("source-gates"), path.join("moved-gates")).is_err());
    assert!(fs::rename(&path, f.temp.path().join("moved-root")).is_err());
    drop(held);
    fs::rename(&path, f.temp.path().join("moved-root")).unwrap();
}
