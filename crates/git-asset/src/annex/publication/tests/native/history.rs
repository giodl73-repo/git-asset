//! Root metadata archive composes with existing per-member native retained readback.
use super::*;
use crate::annex::history as archive;
fn select(ns: &mut Namespace, nodes: &MetadataReader, root: Object, revision: u64, name: &str) {
    ns.select(
        &Select {
            prepare: k(&format!("{name}-prepare")),
            adopt: k(&format!("{name}-adopt")),
            source: k("main"),
            owner: k("owner"),
            expected_revision: revision,
            selection: git_asset_store::source_claims::Selection {
                root,
                context: Digest::blob(name.as_bytes()),
            },
        },
        nodes,
        &Fixture::limits(),
    )
    .unwrap();
}
fn matrix(native: bool) {
    let tools = native.then(|| {
        (
            std::env::var_os("GIT_ASSET_TEST_GIT_EXE").unwrap().into(),
            std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE").unwrap().into(),
            std::env::split_paths(&std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").unwrap())
                .collect(),
        )
    });
    let bytes = vec![83; 256 * 1024];
    let mut f = Fixture::build_setup(&bytes, tools.clone(), false, None, true);
    let mut m = Fixture::build_setup(&bytes, tools, true, None, true);
    super::retirement::publish(&mut f);
    super::retirement::publish(&mut m);
    let object = f.plan.candidate().object.clone();
    let missing = Object::from_bytes(b"declared old member, no native bytes");
    let mut writer =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let old_path = LogicalPath::parse("assets/old").unwrap();
    let (old, _) = Tree::build(
        &BTreeMap::from([
            (old_path.clone(), object.clone()),
            (
                LogicalPath::parse("assets/unavailable").unwrap(),
                missing.clone(),
            ),
        ]),
        &mut writer,
        TreeLimits::default(),
    )
    .unwrap();
    let (new, _) = Tree::build(
        &BTreeMap::from([(
            LogicalPath::parse("assets/new").unwrap(),
            Object::from_bytes(b"new selected version"),
        )]),
        &mut writer,
        TreeLimits::default(),
    )
    .unwrap();
    let checkpoint = writer.finish().unwrap();
    drop(writer);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &checkpoint).unwrap();
    f.namespaces[0]
        .apply(
            k("history-enroll-main"),
            Command::Enroll {
                source: k("main"),
                owner: k("owner"),
                class: Kind::Branch,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    select(
        &mut f.namespaces[0],
        &f.metadata,
        old.root().clone(),
        1,
        "main-old",
    );
    let origin = f.namespaces[0]
        .source(&k("main"))
        .unwrap()
        .unwrap()
        .current
        .unwrap();
    select(
        &mut f.namespaces[0],
        &f.metadata,
        new.root().clone(),
        3,
        "main-new",
    );
    let places = Places {
        current: vec![k("metadata")],
        pending: vec![k("metadata")],
        predecessor: vec![k("metadata")],
    };
    let mut records = Namespace::create_fenced(
        &m.root.join("history-records"),
        k("history"),
        k("epoch"),
        Policy {
            prefer_active: true,
            classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                .into_iter()
                .map(|c| (c, places.clone()))
                .collect(),
        },
    )
    .unwrap();
    let mut nodes = MetadataStore::open(&m.root.join("metadata"), m.metadata.checkpoint()).unwrap();
    let profile = archive::Profile {
        working: archive::Working {
            namespace: f.namespaces[0].identity().clone(),
            source: k("main"),
            owner: k("owner"),
            class: Kind::Branch,
        },
        target: archive::Target {
            namespace: m.namespaces[0].identity().clone(),
            owner: k("owner"),
            place: k("pool"),
            prefix: k("history-media"),
        },
        records: crate::annex::pending::Scope {
            namespace: records.identity().clone(),
            metadata: nodes.checkpoint().identity().clone(),
            owner: k("owner"),
            prefix: k("history-record"),
            place: k("metadata"),
            allow_store: true,
        },
        limits: Fixture::limits(),
        allow_publish: true,
        allow_release: true,
    };
    let plan = archive::Plan::capture(&profile, &mut f.namespaces[0], &origin).unwrap();
    let loaded = archive::publish(
        &profile,
        &plan,
        &mut f.namespaces[0],
        archive::Destinations {
            media: &mut m.namespaces[0],
            records: &mut records,
        },
        &f.metadata,
        &mut nodes,
        |_| Ok(()),
    )
    .unwrap();
    let mut reader = nodes.finished_reader().unwrap();
    let member = loaded.member(&profile, old_path, &mut reader).unwrap();
    let released = loaded
        .release(
            &profile,
            &mut f.namespaces[0],
            &mut m.namespaces[0],
            (&f.metadata, &mut reader),
            || Ok(()),
        )
        .unwrap();
    assert_eq!(released.media_custody_verified, None);
    assert!(!released.authorizes_eviction);
    assert!(
        !f.namespaces[0]
            .source(&k("main"))
            .unwrap()
            .unwrap()
            .predecessors
            .contains(&origin)
    );
    m.metadata = MetadataReader::open(&m.root.join("metadata"), &nodes.checkpoint()).unwrap();
    // Retire original keeper ingress: the exact archived root is now its retained basis.
    super::retirement::settle(&mut m);
    let m_plan = m.plan.clone();
    assert!(matches!(
        m.publisher().reopen(&m_plan, &mut m.resources()),
        Err(Error::ProtectionLost)
    ));
    let f_plan = f.plan.clone();
    let handoff = crate::annex::publication::handoff::Plan::new(
        &f_plan,
        &m_plan,
        member.clone(),
        &mut f.namespaces[0],
    )
    .unwrap();
    crate::annex::publication::handoff::Execution {
        source: &mut f.publisher(),
        keeper: &mut m.publisher(),
        allow_release: true,
    }
    .apply(
        &handoff,
        &f_plan,
        &m_plan,
        &mut f.resources(),
        &mut m.resources(),
    )
    .unwrap();
    let mut still_present = Vec::new();
    crate::annex::Reader::new(f.profile.pool.pool.clone())
        .unwrap()
        .open(&object)
        .unwrap()
        .read_to_end(&mut still_present)
        .unwrap();
    assert_eq!(
        still_present, bytes,
        "claim handoff must not remove working bytes"
    );
    fs::rename(
        f.profile.pool.pool.repository.join(".git/config"),
        f.profile.pool.pool.repository.join(".git/config.offline"),
    )
    .unwrap();
    let mut actual = Vec::new();
    m.publisher()
        .open_retained(&m_plan, &member, &mut m.resources())
        .unwrap()
        .read_to_end(&mut actual)
        .unwrap();
    assert_eq!(actual, bytes);
    let missing_error = crate::annex::Reader::new(m.profile.pool.pool.clone())
        .unwrap()
        .open(&missing)
        .err()
        .unwrap();
    assert_eq!(Error::from(missing_error), Error::Missing);
    let frozen = loaded
        .member(
            &profile,
            LogicalPath::parse("assets/unavailable").unwrap(),
            &mut reader,
        )
        .unwrap();
    assert_eq!(
        frozen.object, missing,
        "missing bytes must not retarget a logical member"
    );
}
#[test]
fn archived_main_root_reopens_exact_member_after_ingest_release_without_old_repo() {
    matrix(false);
}
#[test]
#[ignore = "explicit native tools; owned synthetic pools only"]
fn actual_archived_main_root_reopens_exact_member_after_ingest_release_without_old_repo() {
    matrix(true);
}
