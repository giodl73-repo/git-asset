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
                k("withdraw"),
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
            k("retire"),
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
fn distinct(f: &mut Fixture) {
    f.profile.pool.pool.uuid = "00000000-0000-0000-0000-000000000002".into();
    fs::write(
        f.profile.pool.pool.repository.join("uuid"),
        &f.profile.pool.pool.uuid,
    )
    .unwrap();
    f.plan = Plan::new(
        &f.profile,
        f.plan.request.clone(),
        f.plan.selection.selection.clone(),
        f.plan.admission.clone(),
    )
    .unwrap();
}
#[test]
fn keeper_requires_exact_pool_place_retention_and_bytes() {
    let mut source = Fixture::new(b"retained keeper");
    let mut keeper = Fixture::new(b"retained keeper");
    distinct(&mut keeper);
    publish(&mut source);
    publish(&mut keeper);
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
    assert!(matches!(
        keeper.namespaces[0].fence(&pin.object),
        Err(admission::Error::Busy)
    ));
    assert!(
        fs::OpenOptions::new()
            .write(true)
            .open(keeper.output_path())
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
    fs::write(keeper.output_path(), b"corrupt").unwrap();
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
    fs::remove_file(keeper.output_path()).unwrap();
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
    distinct(&mut keeper);
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
