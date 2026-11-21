use super::*;
use crate::annex::{retirement::Assessor, snapshot::Snapshot};
fn publish(f: &mut Fixture) {
    let plan = f.plan.clone();
    let mut input = f.input();
    f.refresh();
    f.publisher()
        .apply(&plan, &mut f.resources(), &mut input)
        .unwrap();
}
fn settle(f: &mut Fixture) {
    let source = f.namespaces[0].source(&k("asset")).unwrap().unwrap();
    let version = source
        .current
        .clone()
        .or_else(|| source.predecessors.first().cloned())
        .unwrap();
    if source.current.is_some() {
        f.namespaces[0]
            .apply(
                k(&format!("withdraw-{}", source.revision)),
                Command::Withdraw {
                    source: k("asset"),
                    owner: k("owner"),
                    expected_revision: source.revision,
                },
                &f.metadata,
                &Fixture::limits(),
            )
            .unwrap();
    }
    let revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    f.namespaces[0]
        .apply(
            k(&format!("retire-{revision}")),
            Command::Retire {
                source: k("asset"),
                owner: k("owner"),
                expected_revision: revision,
                predecessor_revision: version.revision,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
}

fn snapshot(f: &mut Fixture) -> Snapshot {
    Snapshot {
        namespace: f.namespaces[0].identity().clone(),
        source: k("asset"),
        owner: k("owner"),
        version: f.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .current
            .unwrap(),
        place: k("pool"),
        path: LogicalPath::parse("candidate/output").unwrap(),
        object: f.plan.candidate().object.clone(),
    }
}
#[test]
fn keeper_requires_exact_pool_place_retention_and_bytes() {
    let source = Fixture::new(b"retained keeper");
    let keeper = Fixture::new(b"retained keeper");
    assert_ne!(source.profile.pool.pool.uuid, keeper.profile.pool.pool.uuid);
    keeper_checks(source, keeper, false);
}
fn keeper_checks(mut source: Fixture, mut keeper: Fixture, separated: bool) {
    publish(&mut source);
    publish(&mut keeper);
    let keeper_path = crate::annex::Reader::new(keeper.profile.pool.pool.clone())
        .unwrap()
        .open(&keeper.plan.candidate().object)
        .unwrap()
        .with_path(|p| p.to_path_buf())
        .unwrap();
    // Owned native fixture: clear its readonly attribute so write refusal proves guards.
    let mut permissions = fs::metadata(&keeper_path).unwrap().permissions();
    #[allow(
        clippy::permissions_set_readonly_false,
        reason = "Windows-only fixture: clear the readonly attribute to prove handle exclusion"
    )]
    permissions.set_readonly(false);
    fs::set_permissions(&keeper_path, permissions).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&keeper_path)
        .unwrap();
    let mut pin = snapshot(&mut keeper);
    settle(&mut source);
    let held = Assessor::new(source.profile.pool.clone())
        .unwrap()
        .assess(&pin.object, &source.metadata, &Fixture::limits())
        .unwrap();
    let limits = TreeLimits::default();
    let mut profile = keeper.profile.pool.clone();
    profile.pool.uuid = source.profile.pool.pool.uuid.clone();
    assert_eq!(
        held.open_keeper(profile, &pin, &mut keeper.metadata, limits)
            .err()
            .unwrap()
            .code(),
        "denied"
    );
    pin.place = k("other");
    assert_eq!(
        held.open_keeper(
            keeper.profile.pool.clone(),
            &pin,
            &mut keeper.metadata,
            limits
        )
        .err()
        .unwrap()
        .code(),
        "denied"
    );
    pin.place = k("pool");
    let mut read = held
        .open_keeper(
            keeper.profile.pool.clone(),
            &pin,
            &mut keeper.metadata,
            limits,
        )
        .unwrap();
    assert_eq!(read.snapshot(), &pin);
    assert_ne!(read.store(), &held.store());
    if separated {
        let device_view = read.compare_source_storage().unwrap();
        let id = device_view.id().unwrap();
        device_view.revalidate().unwrap();
        assert_eq!(device_view.id().unwrap(), id);
    } else {
        // These owned pools intentionally share one volume. UUIDs are not independent custody.
        assert_eq!(
            read.compare_source_storage().err().unwrap().code(),
            "denied"
        );
    }
    assert!(matches!(
        keeper.namespaces[0].fence(&pin.object),
        Err(admission::Error::Busy)
    ));
    assert!(
        fs::OpenOptions::new()
            .write(true)
            .open(&keeper_path)
            .is_err()
    );
    assert!(
        fs::OpenOptions::new()
            .write(true)
            .open(keeper.profile.pool.pool.repository.join(".git/config"))
            .is_err()
    );
    let current = keeper.namespaces[0].source(&k("asset")).unwrap().unwrap();
    // Withdraw preserves the same hard-retained predecessor; retiring it changes demand.
    keeper.namespaces[0]
        .apply(
            k("blocked-withdraw"),
            Command::Withdraw {
                source: k("asset"),
                owner: k("owner"),
                expected_revision: current.revision,
            },
            &keeper.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    assert!(matches!(
        keeper.namespaces[0].apply(
            k("blocked-retire"),
            Command::Retire {
                source: k("asset"),
                owner: k("owner"),
                expected_revision: current.revision + 1,
                predecessor_revision: current.current.unwrap().revision,
            },
            &keeper.metadata,
            &Fixture::limits()
        ),
        Err(admission::Error::Busy)
    ));
    let mut bytes = vec![];
    read.read_to_end(&mut bytes).unwrap();
    assert_eq!(Object::from_bytes(&bytes), pin.object);
    drop(read);
    fs::write(&keeper_path, b"corrupt").unwrap();
    assert_eq!(
        held.open_keeper(
            keeper.profile.pool.clone(),
            &pin,
            &mut keeper.metadata,
            limits
        )
        .err()
        .unwrap()
        .code(),
        "corrupt"
    );
    fs::remove_file(&keeper_path).unwrap();
    assert_eq!(
        held.open_keeper(
            keeper.profile.pool.clone(),
            &pin,
            &mut keeper.metadata,
            limits
        )
        .err()
        .unwrap()
        .code(),
        "missing"
    );
    settle(&mut keeper);
    assert_eq!(
        held.open_keeper(
            keeper.profile.pool.clone(),
            &pin,
            &mut keeper.metadata,
            limits
        )
        .err()
        .unwrap()
        .code(),
        "protection_lost"
    );
}
#[test]
fn shared_authority_keeper_borrows_source_fence_and_rejects_cloned_location() {
    let source = Fixture::new(b"shared authority keeper");
    let mut keeper = Fixture::new(b"shared authority keeper");
    publish(&mut keeper);
    let policy = Policy {
        prefer_active: true,
        classes: [Kind::Retention, Kind::Branch, Kind::Draft, Kind::Job]
            .into_iter()
            .map(|kind| {
                let place = k(if kind == Kind::Retention {
                    "backup"
                } else {
                    "pool"
                });
                (
                    kind,
                    Places {
                        current: vec![place.clone()],
                        pending: vec![place.clone()],
                        predecessor: vec![place],
                    },
                )
            })
            .collect(),
    };
    let root = source.root.join("shared-authority");
    let mut ns = Namespace::create(&root, k("project"), k("epoch"), policy).unwrap();
    ns.apply(
        k("enroll"),
        Command::Enroll {
            source: k("asset"),
            owner: k("owner"),
            class: Kind::Retention,
        },
        &keeper.metadata,
        &Fixture::limits(),
    )
    .unwrap();
    ns.select(
        &Select {
            prepare: k("prepare"),
            adopt: k("adopt"),
            source: k("asset"),
            owner: k("owner"),
            expected_revision: 1,
            selection: keeper.plan.selection.selection.clone(),
        },
        &keeper.metadata,
        &Fixture::limits(),
    )
    .unwrap();
    let pin = Snapshot {
        namespace: ns.identity().clone(),
        source: k("asset"),
        owner: k("owner"),
        version: ns.source(&k("asset")).unwrap().unwrap().current.unwrap(),
        place: k("backup"),
        path: LogicalPath::parse("candidate/output").unwrap(),
        object: keeper.plan.candidate().object.clone(),
    };
    let profile = |mut p: managed::Profile, place: &str| {
        p.binding = Binding::new(
            1,
            Digest::blob(b"shared admitted boundary"),
            vec![Member {
                namespace: ns.identity().clone(),
                places: vec![k(place)],
            }],
        )
        .unwrap();
        p.members = vec![managed::Member {
            root: root.clone(),
            minimum: ns.checkpoint(),
        }];
        fs::write(
            p.pool.repository.join("binding"),
            p.binding.id().unwrap().as_str(),
        )
        .unwrap();
        p
    };
    let source_profile = profile(source.profile.pool.clone(), "pool");
    let keeper_profile = profile(keeper.profile.pool.clone(), "backup");
    let held = Assessor::new(source_profile)
        .unwrap()
        .assess(&pin.object, &keeper.metadata, &Fixture::limits())
        .unwrap();
    let mut alias = keeper_profile.clone();
    alias.members[0].root = keeper.root.join("cloned-authority");
    assert_eq!(
        held.open_keeper(alias, &pin, &mut keeper.metadata, TreeLimits::default())
            .err()
            .unwrap()
            .code(),
        "denied"
    );
    let mut read = held
        .open_keeper(
            keeper_profile,
            &pin,
            &mut keeper.metadata,
            TreeLimits::default(),
        )
        .unwrap();
    let mut bytes = vec![];
    read.read_to_end(&mut bytes).unwrap();
    assert_eq!(Object::from_bytes(&bytes), pin.object);
    assert!(matches!(ns.hold(&pin.object), Err(admission::Error::Busy)));
    drop(read);
    drop(held);
    ns.hold(&pin.object).unwrap();
}

#[test]
fn same_hash_reingest_changes_replay_basis_even_after_new_claim_is_retired() {
    let mut f = Fixture::new(b"same bytes later incarnation");
    publish(&mut f);
    let original = f.plan.clone();
    settle(&mut f);
    let held = Assessor::new(f.profile.pool.clone())
        .unwrap()
        .assess(
            &original.candidate().object,
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    let before = held.replay_basis().unwrap();
    held.check_replay(&before).unwrap();
    before.id().unwrap();
    let mut input = f.input();
    assert!(matches!(
        f.publisher()
            .apply(&original, &mut f.resources(), &mut input),
        Err(Error::Busy)
    ));
    f.namespaces[1]
        .apply(
            k("unrelated"),
            Command::Enroll {
                source: k("another"),
                owner: k("owner"),
                class: Kind::Branch,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    held.check_replay(&before).unwrap(); // live subject predicates remain stable
    drop(held);
    let held = Assessor::new(f.profile.pool.clone())
        .unwrap()
        .assess(
            &original.candidate().object,
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    assert_eq!(held.check_replay(&before).unwrap_err().code(), "stale");
    let before = held.replay_basis().unwrap();
    drop(held); // models lost live fences, not a persisted removal authorization
    // Owned fake-protocol fixture effect only; production exposes no unlink method.
    fs::remove_file(f.output_path()).unwrap();
    let imports = |f: &Fixture| {
        fs::read_to_string(f.profile.pool.pool.repository.join("calls"))
            .unwrap()
            .lines()
            .filter(|s| s.starts_with("reinject "))
            .count()
    };
    let calls = imports(&f);
    let capacity = f.ledger.checkpoint();
    let mut input = f.input();
    assert!(matches!(
        f.publisher()
            .apply(&original, &mut f.resources(), &mut input),
        Err(Error::MissingAcknowledged)
    ));
    assert_eq!(imports(&f), calls);
    assert_eq!(f.ledger.checkpoint(), capacity);
    assert!(!f.output_path().exists());
    // Fresh admitted publication must first advance hard retention in the same roster.
    let mut metadata =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let mut request = original.request.clone();
    request.operation = k("reingest-same-hash");
    request.expected_revision = f.namespaces[0]
        .source(&k("asset"))
        .unwrap()
        .unwrap()
        .revision;
    let frozen = freeze(&request, &mut metadata, TreeLimits::default()).unwrap();
    let cp = metadata.finish().unwrap();
    drop(metadata);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &cp).unwrap();
    f.plan = Plan::new(&f.profile, request, frozen, original.admission.clone()).unwrap();
    publish(&mut f);
    assert_eq!(
        Object::from_bytes(&fs::read(f.output_path()).unwrap()),
        original.candidate().object
    );
    settle(&mut f);
    let current = Assessor::new(f.profile.pool.clone())
        .unwrap()
        .assess(
            &original.candidate().object,
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    assert_eq!(current.check_replay(&before).unwrap_err().code(), "stale");
    assert_ne!(
        before.id().unwrap(),
        current.replay_basis().unwrap().id().unwrap()
    );
    assert_eq!(
        Object::from_bytes(&fs::read(f.output_path()).unwrap()),
        original.candidate().object
    );
}

#[test]
#[ignore = "explicit native Git/annex tools; owned ordinary and bare keeper pools"]
fn actual_native_keeper_fences_verified_bytes_and_retention_in_ordinary_and_bare_pools() {
    let git: PathBuf = std::env::var_os("GIT_ASSET_TEST_GIT_EXE").unwrap().into();
    let annex: PathBuf = std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE").unwrap().into();
    let path = std::env::split_paths(&std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").unwrap())
        .collect::<Vec<_>>();
    for bare in [false, true] {
        let source = Fixture::build(
            &vec![42; 256 * 1024],
            Some((git.clone(), annex.clone(), path.clone())),
            !bare,
        );
        let keeper = Fixture::build(
            &vec![42; 256 * 1024],
            Some((git.clone(), annex.clone(), path.clone())),
            bare,
        );
        assert_ne!(source.profile.pool.pool.uuid, keeper.profile.pool.pool.uuid);
        keeper_checks(source, keeper, false);
    }
}

#[test]
#[ignore = "explicit native tools and source/keeper volume roots; fresh synthetic pools only"]
fn actual_native_keeper_device_views_on_declared_distinct_volumes() {
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
    for bare in [false, true] {
        let source = Fixture::build_in(
            &vec![42; 256 * 1024],
            Some((git.clone(), annex.clone(), path.clone())),
            !bare,
            Some(&source_root),
        );
        let keeper = Fixture::build_in(
            &vec![42; 256 * 1024],
            Some((git.clone(), annex.clone(), path.clone())),
            bare,
            Some(&keeper_root),
        );
        keeper_checks(source, keeper, true);
    }
}
