use super::*;
use git_asset_core::tree::{MemoryNodes, TreeResult};
use git_asset_store::{
    protection::Limits,
    source_claims::{
        Command, Places, Selection,
        admission::{Policy, Select},
    },
};
use std::{cell::Cell, collections::BTreeMap, path::PathBuf};
fn key(s: &str) -> Key {
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
struct Fixture {
    namespace: Namespace,
    nodes: MemoryNodes,
    snapshot: Snapshot,
    temp: tempfile::TempDir,
}
impl Fixture {
    fn new(kind: Kind) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let place = Places {
            current: vec![key("pool")],
            pending: vec![key("pool")],
            predecessor: vec![key("pool")],
        };
        let policy = Policy {
            prefer_active: true,
            classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                .into_iter()
                .map(|k| (k, place.clone()))
                .collect(),
        };
        let mut namespace = Namespace::create(
            &temp.path().join("namespace"),
            key("project"),
            key("epoch"),
            policy,
        )
        .unwrap();
        let mut nodes = MemoryNodes::default();
        namespace
            .apply(
                key("enroll"),
                Command::Enroll {
                    source: key("archive"),
                    owner: key("owner"),
                    class: kind,
                },
                &nodes,
                &limits(),
            )
            .unwrap();
        let object = Object::from_bytes(b"pinned original");
        let mut entries: BTreeMap<_, _> = (0..5000)
            .map(|i| {
                (
                    LogicalPath::parse(&format!("items/unrelated-{i:04}")).unwrap(),
                    Object::from_bytes(b"other"),
                )
            })
            .collect();
        let path = LogicalPath::parse("items/pinned").unwrap();
        entries.insert(path.clone(), object.clone());
        let (tree, _) = Tree::build(&entries, &mut nodes, TreeLimits::default()).unwrap();
        let selection = Selection {
            root: tree.root().clone(),
            context: Digest::blob(b"exact retained root"),
        };
        let revision = namespace.source(&key("archive")).unwrap().unwrap().revision;
        namespace
            .select(
                &Select {
                    prepare: key("prepare"),
                    adopt: key("adopt"),
                    source: key("archive"),
                    owner: key("owner"),
                    expected_revision: revision,
                    selection,
                },
                &nodes,
                &limits(),
            )
            .unwrap();
        let version = namespace
            .source(&key("archive"))
            .unwrap()
            .unwrap()
            .current
            .unwrap();
        let snapshot = Snapshot {
            namespace: namespace.identity().clone(),
            source: key("archive"),
            owner: key("owner"),
            version,
            place: key("pool"),
            path,
            object,
        };
        Self {
            temp,
            namespace,
            nodes,
            snapshot,
        }
    }
}
struct Checked<'a> {
    nodes: &'a MemoryNodes,
    calls: Cell<usize>,
    payload: Object,
    race: Option<(
        PathBuf,
        git_asset_store::source_claims::Checkpoint,
        Snapshot,
    )>,
    fired: Cell<bool>,
}
impl NodeStore for Checked<'_> {
    fn get(&self, object: &Object) -> TreeResult<Option<Vec<u8>>> {
        assert_ne!(object, &self.payload, "payload must never be opened");
        self.calls.set(self.calls.get() + 1);
        if let Some((root, minimum, proof)) = &self.race
            && !self.fired.replace(true)
        {
            let mut peer = Namespace::open(root, minimum).unwrap();
            let revision = peer.source(&proof.source).unwrap().unwrap().revision;
            peer.apply(
                key("withdraw-race"),
                Command::Withdraw {
                    source: proof.source.clone(),
                    owner: proof.owner.clone(),
                    expected_revision: revision,
                },
                self.nodes,
                &limits(),
            )
            .unwrap();
            let revision = peer.source(&proof.source).unwrap().unwrap().revision;
            peer.apply(
                key("retire-race"),
                Command::Retire {
                    source: proof.source.clone(),
                    owner: proof.owner.clone(),
                    expected_revision: revision,
                    predecessor_revision: proof.version.revision,
                },
                self.nodes,
                &limits(),
            )
            .unwrap();
        }
        self.nodes.get(object)
    }
    fn put(&mut self, _: &[u8]) -> TreeResult<Object> {
        panic!("point proof cannot write")
    }
}
#[test]
fn shared_root_point_proof_preserves_history_and_never_reads_payloads() {
    let mut f = Fixture::new(Kind::Retention);
    let checkpoint = f.namespace.checkpoint();
    let mut nodes = Checked {
        nodes: &f.nodes,
        calls: Cell::new(0),
        payload: f.snapshot.object.clone(),
        race: None,
        fired: Cell::new(false),
    };
    let held = f
        .snapshot
        .verify(&mut f.namespace, &mut nodes, TreeLimits::default())
        .unwrap();
    assert!(nodes.calls.get() < 64);
    assert_eq!(held.snapshot(), &f.snapshot);
    assert!(matches!(
        f.namespace.fence(&f.snapshot.object),
        Err(git_asset_store::source_claims::admission::Error::Busy)
    ));
    assert_eq!(f.namespace.checkpoint(), checkpoint);
    drop(held);
    let revision = f
        .namespace
        .source(&f.snapshot.source)
        .unwrap()
        .unwrap()
        .revision;
    f.namespace
        .apply(
            key("withdraw"),
            Command::Withdraw {
                source: f.snapshot.source.clone(),
                owner: f.snapshot.owner.clone(),
                expected_revision: revision,
            },
            &f.nodes,
            &limits(),
        )
        .unwrap();
    let held = f
        .snapshot
        .verify(&mut f.namespace, &mut nodes, TreeLimits::default())
        .unwrap();
    assert_eq!(held.snapshot().version, f.snapshot.version);
}
#[test]
fn wrong_namespace_owner_kind_place_path_and_object_refuse() {
    let mut f = Fixture::new(Kind::Retention);
    let mut wrong = f.snapshot.clone();
    wrong.namespace = Digest::blob(b"wrong authority");
    assert!(matches!(
        wrong.verify(&mut f.namespace, &mut f.nodes, TreeLimits::default()),
        Err(Error::Stale)
    ));
    let mut wrong = f.snapshot.clone();
    wrong.owner = key("wrong-owner");
    assert!(matches!(
        wrong.verify(&mut f.namespace, &mut f.nodes, TreeLimits::default()),
        Err(Error::Denied)
    ));
    let mut wrong = f.snapshot.clone();
    wrong.place = key("foreign-place");
    assert!(matches!(
        wrong.verify(&mut f.namespace, &mut f.nodes, TreeLimits::default()),
        Err(Error::Denied)
    ));
    let mut wrong = f.snapshot.clone();
    wrong.path = LogicalPath::parse("missing").unwrap();
    assert!(matches!(
        wrong.verify(&mut f.namespace, &mut f.nodes, TreeLimits::default()),
        Err(Error::Missing)
    ));
    let mut wrong = f.snapshot.clone();
    wrong.object = Object::from_bytes(b"different expected bytes");
    assert!(matches!(
        wrong.verify(&mut f.namespace, &mut f.nodes, TreeLimits::default()),
        Err(Error::Conflict)
    ));
    let mut wrong = f.snapshot.clone();
    wrong.version.selection.context = Digest::blob(b"wrong snapshot");
    assert!(matches!(
        wrong.verify(&mut f.namespace, &mut f.nodes, TreeLimits::default()),
        Err(Error::ProtectionLost)
    ));
    let mut other = Fixture::new(Kind::Branch);
    assert!(matches!(
        other.snapshot.verify(
            &mut other.namespace,
            &mut other.nodes,
            TreeLimits::default()
        ),
        Err(Error::Unsupported)
    ));
    let short = TreeLimits {
        max_reads: 0,
        ..TreeLimits::default()
    };
    assert!(matches!(
        f.snapshot.verify(&mut f.namespace, &mut f.nodes, short),
        Err(Error::Budget)
    ));
}
#[test]
fn retirement_during_metadata_read_cannot_return_a_retained_proof() {
    let mut f = Fixture::new(Kind::Retention);
    let mut nodes = Checked {
        nodes: &f.nodes,
        calls: Cell::new(0),
        payload: f.snapshot.object.clone(),
        race: Some((
            f.temp.path().join("namespace"),
            f.namespace.checkpoint(),
            f.snapshot.clone(),
        )),
        fired: Cell::new(false),
    };
    assert!(matches!(
        f.snapshot
            .verify(&mut f.namespace, &mut nodes, TreeLimits::default()),
        Err(Error::ProtectionLost)
    ));
    assert!(nodes.fired.get());
}

#[test]
fn shared_namespace_keeper_verifies_under_existing_exclusive_fence_without_self_deadlock() {
    let mut f = Fixture::new(Kind::Retention);
    let fence = f.namespace.fence(&f.snapshot.object).unwrap();
    let proof = f
        .snapshot
        .verify_fenced(
            &mut f.namespace,
            &fence,
            &mut f.nodes,
            TreeLimits::default(),
        )
        .unwrap();
    assert_eq!(proof.snapshot(), &f.snapshot);
    assert!(proof.metrics().reads < 64);
    drop(proof);
    drop(fence);
    let other = f
        .namespace
        .fence(&Object::from_bytes(b"wrong subject"))
        .unwrap();
    assert!(matches!(
        f.snapshot.verify_fenced(
            &mut f.namespace,
            &other,
            &mut f.nodes,
            TreeLimits::default()
        ),
        Err(Error::Denied)
    ));
}
