//! Restored producer registration preserves original roots and original receipts.
use super::*;
use crate::annex::{
    pending::Scope,
    producer_index::{Index, Record, update as roots},
    publication::restore as native,
};
use roots::restore as register;

pub(super) fn exercise(
    f: &mut Fixture,
    keeper: &mut Fixture,
    owner: &OwnerProfile,
    evicted: &NativePlan,
    old: &Plan,
    restored: &native::Plan,
) {
    let places = Places {
        current: vec![k("metadata")],
        pending: vec![k("metadata")],
        predecessor: vec![k("metadata")],
    };
    let mut ns = Namespace::create_fenced(
        &f.root.join("restore-index"),
        k("restore-index"),
        k("epoch"),
        admission::Policy {
            prefer_active: true,
            classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                .into_iter()
                .map(|kind| (kind, places.clone()))
                .collect(),
        },
    )
    .unwrap();
    let mut nodes = MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let profile = roots::Profile {
        scope: Scope {
            namespace: ns.identity().clone(),
            metadata: nodes.checkpoint().identity().clone(),
            owner: k("owner"),
            prefix: k("producers"),
            place: k("metadata"),
            allow_store: true,
        },
        limits: Fixture::limits(),
    };
    ns.apply(
        k("enroll-index"),
        Command::Enroll {
            source: k("producers"),
            owner: k("owner"),
            class: Kind::Job,
        },
        &nodes,
        &profile.limits,
    )
    .unwrap();
    let row = Record::new(old.clone(), None).unwrap();
    let initial = roots::prepare(
        &profile,
        None,
        &[roots::Change {
            expected_record: None,
            record: row.clone(),
        }],
        &mut ns,
        &mut nodes,
    )
    .unwrap()
    .unwrap();
    let first = roots::apply(&profile, &initial, &mut ns, &mut nodes, || Ok(())).unwrap();
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &nodes.checkpoint()).unwrap();
    let native_profile = f.profile.clone();
    let control = f.control_profile.as_ref().unwrap().clone();
    let mut native_ns =
        Namespace::open(&f.root.join("namespace-0"), &f.namespaces[0].checkpoint()).unwrap();
    let mut native_nodes =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let limits = Fixture::limits();
    let context = native::Context {
        profile: &native_profile,
        control: &control,
        retirement: owner,
        namespace: &mut native_ns,
        metadata: &mut native_nodes,
        limits: &limits,
    };
    let original = native::Original {
        eviction: evicted,
        publication: old,
    };
    let baseline = register::Baseline {
        version: first.version.clone(),
        record: row.metadata_object().unwrap(),
    };
    let plan = register::prepare(
        &mut register::Target {
            profile: &profile,
            namespace: &mut ns,
            nodes: &mut nodes,
        },
        &baseline,
        &context,
        original,
        restored,
        &mut f.resources(),
    )
    .unwrap();
    let mut actions = super::pending::Harness::new(f, "restore-index-actions");
    actions.save(crate::annex::pending::Action {
        operation: crate::annex::pending::Operation::RestoreIndex(Box::new(
            crate::annex::pending::RestoreIndexAction {
                original: crate::annex::pending::RestoreAction {
                    eviction: evicted.clone(),
                    restore: restored.clone(),
                },
                index: plan.clone(),
            },
        )),
        working: old.clone(),
        retained: keeper.plan.clone(),
    });
    let replacement = Record::new(restored.publication().clone(), None).unwrap();
    assert_eq!(
        roots::prepare(
            &profile,
            Some(&first.version),
            &[roots::Change {
                expected_record: Some(row.metadata_object().unwrap()),
                record: replacement.clone(),
            }],
            &mut ns,
            &mut nodes
        )
        .err(),
        Some(Error::Conflict)
    );
    // Even a parsed inner Plan cannot bypass ordinary immutable-producer updates.
    let wire: serde_json::Value = serde_json::from_slice(&plan.to_json().unwrap()).unwrap();
    let inner =
        roots::Plan::from_json(&git_asset_core::identity::canonical_json(&wire["update"]).unwrap())
            .unwrap();
    assert_eq!(
        roots::apply(&profile, &inner, &mut ns, &mut nodes, || Ok(())).err(),
        Some(Error::Unsupported)
    );
    assert!(register::Plan::from_json(&serde_json::to_vec_pretty(&wire).unwrap()).is_err());
    // Another admitted metadata writer advances the same index first.
    let mut changed = row.clone();
    changed.retained = Some(crate::annex::snapshot::Snapshot {
        namespace: ns.identity().clone(),
        source: k("history"),
        owner: k("owner"),
        version: first.version.clone(),
        place: k("metadata"),
        path: LogicalPath::parse("declared/member").unwrap(),
        object: row.object.clone(),
    });
    let competing = roots::prepare(
        &profile,
        Some(&first.version),
        &[roots::Change {
            expected_record: Some(row.metadata_object().unwrap()),
            record: changed.clone(),
        }],
        &mut ns,
        &mut nodes,
    )
    .unwrap()
    .unwrap();
    let advanced = roots::apply(&profile, &competing, &mut ns, &mut nodes, || Ok(())).unwrap();
    assert_eq!(
        register::apply(
            &mut register::Target {
                profile: &profile,
                namespace: &mut ns,
                nodes: &mut nodes
            },
            &context,
            original,
            &plan,
            &mut f.resources(),
            || panic!("stale update must not select")
        )
        .err(),
        Some(Error::Stale)
    );
    let baseline = register::Baseline {
        version: advanced.version,
        record: changed.metadata_object().unwrap(),
    };
    let plan = register::prepare(
        &mut register::Target {
            profile: &profile,
            namespace: &mut ns,
            nodes: &mut nodes,
        },
        &baseline,
        &context,
        original,
        restored,
        &mut f.resources(),
    )
    .unwrap();
    let saved = f.root.join("saved-restored-index-plan");
    let mut file = fs::File::create_new(&saved).unwrap();
    std::io::Write::write_all(&mut file, &plan.to_json().unwrap()).unwrap();
    file.sync_all().unwrap();
    let checkpoint = ns.checkpoint();
    assert_eq!(
        register::apply(
            &mut register::Target {
                profile: &profile,
                namespace: &mut ns,
                nodes: &mut nodes
            },
            &context,
            original,
            &plan,
            &mut f.resources(),
            || Err(Error::Busy)
        )
        .err(),
        Some(Error::Busy)
    );
    assert_eq!(checkpoint, ns.checkpoint());
    let plan = register::Plan::from_json(&fs::read(saved).unwrap()).unwrap();
    #[cfg(feature = "github")]
    let mut worker_state = super::super::trusted::State::named(
        f,
        keeper,
        owner,
        "worker-index-project",
        "worker-index-main",
        old.candidate().object.clone(),
    );
    #[cfg(feature = "github")]
    let mut worker_actions = super::pending::Harness::new(f, "worker-index-actions");
    #[cfg(feature = "github")]
    let worker_saved = worker_actions.save(crate::annex::pending::Action {
        operation: crate::annex::pending::Operation::RestoreIndex(Box::new(
            crate::annex::pending::RestoreIndexAction {
                original: crate::annex::pending::RestoreAction {
                    eviction: evicted.clone(),
                    restore: restored.clone(),
                },
                index: plan.clone(),
            },
        )),
        working: old.clone(),
        retained: keeper.plan.clone(),
    });
    #[cfg(feature = "github")]
    let receipt = worker_state.saved_index(
        f,
        keeper,
        (
            &worker_actions.scope,
            &worker_saved,
            &mut worker_actions.namespace,
        ),
        &context,
        &mut register::Target {
            profile: &profile,
            namespace: &mut ns,
            nodes: &mut nodes,
        },
    );
    #[cfg(not(feature = "github"))]
    let mut calls = 0;
    #[cfg(not(feature = "github"))]
    let receipt = register::apply(
        &mut register::Target {
            profile: &profile,
            namespace: &mut ns,
            nodes: &mut nodes,
        },
        &context,
        original,
        &plan,
        &mut f.resources(),
        || {
            calls += 1;
            if calls == 2 {
                Err(Error::Denied)
            } else {
                Ok(())
            }
        },
    )
    .unwrap();
    assert_eq!(receipt.post_effect_hold, Some("denied"));
    assert_eq!(receipt.current, Some(true));
    assert_eq!(receipt.media_custody_verified, None);
    assert!(!receipt.authorizes_eviction);
    assert_eq!(
        Index::from_root(initial.root().clone())
            .get(&row.object, &mut nodes, TreeLimits::default())
            .unwrap()
            .0,
        Some(row)
    );
    assert_eq!(
        Index::from_root(plan.root().clone())
            .get(&replacement.object, &mut nodes, TreeLimits::default())
            .unwrap()
            .0,
        Some(replacement)
    );
    assert!(
        ns.source(&k("producers"))
            .unwrap()
            .unwrap()
            .predecessors
            .contains(&first.version)
    );
    let checkpoint = ns.checkpoint();
    let pool = &f.profile.pool.pool;
    let config = pool
        .repository
        .join(if pool.bare { "config" } else { ".git/config" });
    let offline = f.root.join("offline-native-config");
    fs::rename(&config, &offline).unwrap();
    let mut readonly = profile.clone();
    readonly.scope.allow_store = false;
    #[cfg(feature = "github")]
    let recovered = worker_state.saved_index(
        f,
        keeper,
        (
            &worker_actions.scope,
            &worker_saved,
            &mut worker_actions.namespace,
        ),
        &context,
        &mut register::Target {
            profile: &readonly,
            namespace: &mut ns,
            nodes: &mut nodes,
        },
    );
    #[cfg(not(feature = "github"))]
    let recovered = register::apply(
        &mut register::Target {
            profile: &readonly,
            namespace: &mut ns,
            nodes: &mut nodes,
        },
        &context,
        original,
        &plan,
        &mut f.resources(),
        || panic!("completed receipt must not read native bytes"),
    )
    .unwrap();
    fs::rename(offline, config).unwrap();
    assert_eq!(recovered.version, receipt.version);
    assert_eq!(checkpoint, ns.checkpoint());
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &nodes.checkpoint()).unwrap();
}
