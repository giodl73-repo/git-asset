use super::*;
use crate::annex::producer_index::{Index, Record};

#[test]
fn exact_index_lookup_pages_budget_and_stale_roots_without_native_io() {
    let root = tempfile::tempdir().unwrap();
    let p = profile(root.path());
    let mut nodes = MemoryNodes::default();
    let mut bindings = BTreeMap::new();
    let mut records = Vec::new();
    for i in 0..3 {
        let mut r = request(&mut nodes);
        r.operation = k(&format!("attempt-{i}"));
        r.candidate.object = Object::from_bytes(format!("result-{i}").as_bytes());
        let producer = plan(&mut nodes, &p, r);
        let retained = (i == 1).then(|| crate::annex::snapshot::Snapshot {
            namespace: Digest::blob(b"retained namespace"),
            source: k("history"),
            owner: k("owner"),
            version: git_asset_store::source_claims::Version {
                revision: 2,
                selection: producer.selection.selection.clone(),
            },
            place: k("retained"),
            path: LogicalPath::parse("member").unwrap(),
            object: producer.candidate().object.clone(),
        });
        let row = Record::new(producer, retained).unwrap();
        let (path, object) = row.store(&mut nodes).unwrap();
        bindings.insert(path, object);
        records.push(row);
    }
    let (tree, _) = Tree::build(&bindings, &mut nodes, TreeLimits::default()).unwrap();
    let index = Index::from_root(tree.root().clone());
    for row in &records {
        let (found, metrics) = index
            .get(&row.object, &mut nodes, TreeLimits::default())
            .unwrap();
        assert_eq!(found.as_ref(), Some(row));
        assert!(metrics.reads > 1);
        assert_eq!(
            index
                .get(
                    &row.object,
                    &mut nodes,
                    TreeLimits {
                        max_reads: metrics.reads - 1,
                        ..TreeLimits::default()
                    }
                )
                .unwrap_err(),
            Error::Budget
        );
        assert_eq!(
            index
                .get(
                    &row.object,
                    &mut nodes,
                    TreeLimits {
                        max_read_bytes: metrics.read_bytes - 1,
                        ..TreeLimits::default()
                    }
                )
                .unwrap_err(),
            Error::Budget
        );
        let mut wrong_size = row.object.clone();
        wrong_size.bytes += 1;
        assert_eq!(
            index
                .get(&wrong_size, &mut nodes, TreeLimits::default())
                .unwrap_err(),
            Error::Integrity
        );
    }
    assert!(
        index
            .get(
                &Object::from_bytes(b"unknown"),
                &mut nodes,
                TreeLimits::default()
            )
            .unwrap()
            .0
            .is_none()
    );
    let first = index
        .page(None, 2, &mut nodes, TreeLimits::default())
        .unwrap();
    assert_eq!(first.records.len(), 2);
    let cursor = first.next.unwrap();
    let last = index
        .page(Some(&cursor), 2, &mut nodes, TreeLimits::default())
        .unwrap();
    assert_eq!(last.records.len(), 1);
    assert!(last.next.is_none());
    let empty = Tree::empty(&mut nodes).unwrap();
    assert_eq!(
        Index::from_root(empty.root().clone())
            .page(Some(&cursor), 2, &mut nodes, TreeLimits::default())
            .unwrap_err(),
        Error::Stale
    );
    let page = index
        .page(None, 3, &mut nodes, TreeLimits::default())
        .unwrap();
    assert_eq!(
        index
            .page(
                None,
                3,
                &mut nodes,
                TreeLimits {
                    max_reads: page.metrics.reads - 1,
                    ..TreeLimits::default()
                }
            )
            .unwrap_err(),
        Error::Budget
    );
    assert!(!p.pool.pool.repository.exists());
    let retained = records.iter().find(|r| r.retained.is_some()).unwrap();
    let mut invalid = retained.clone();
    invalid.retained.as_mut().unwrap().object.bytes += 1;
    assert_eq!(invalid.store(&mut nodes).unwrap_err(), Error::Integrity);
    let path = LogicalPath::parse(records[0].object.sha256.as_str()).unwrap();
    let (changed, _) = tree
        .edit(
            tree.root(),
            &BTreeMap::from([(path, None)]),
            &mut nodes,
            TreeLimits::default(),
        )
        .unwrap();
    assert_ne!(tree.root(), changed.root());
    assert!(
        Index::from_root(changed.root().clone())
            .get(&records[0].object, &mut nodes, TreeLimits::default())
            .unwrap()
            .0
            .is_none()
    );
    assert_eq!(
        index
            .get(&records[0].object, &mut nodes, TreeLimits::default())
            .unwrap()
            .0,
        Some(records[0].clone())
    );
}

#[test]
fn index_rejects_wrong_key_noncanonical_missing_and_corrupt_records() {
    let root = tempfile::tempdir().unwrap();
    let mut nodes = MemoryNodes::default();
    let r = request(&mut nodes);
    let row = Record::new(plan(&mut nodes, &profile(root.path()), r), None).unwrap();
    let (key, stored) = row.store(&mut nodes).unwrap();
    let wrong = LogicalPath::parse(Digest::blob(b"other").as_str()).unwrap();
    let (tree, _) = Tree::build(
        &BTreeMap::from([(wrong, stored.clone())]),
        &mut nodes,
        TreeLimits::default(),
    )
    .unwrap();
    assert_eq!(
        Index::from_root(tree.root().clone())
            .page(None, 1, &mut nodes, TreeLimits::default())
            .unwrap_err(),
        Error::Integrity
    );
    let bad = nodes
        .put(&serde_json::to_vec_pretty(&row).unwrap())
        .unwrap();
    let (tree, _) = Tree::build(
        &BTreeMap::from([(key.clone(), bad)]),
        &mut nodes,
        TreeLimits::default(),
    )
    .unwrap();
    assert_eq!(
        Index::from_root(tree.root().clone())
            .get(&row.object, &mut nodes, TreeLimits::default())
            .unwrap_err(),
        Error::Integrity
    );
    let (tree, _) = Tree::build(
        &BTreeMap::from([(key, stored.clone())]),
        &mut nodes,
        TreeLimits::default(),
    )
    .unwrap();
    struct Broken<'a> {
        nodes: &'a MemoryNodes,
        row: Object,
        corrupt: bool,
    }
    impl NodeStore for Broken<'_> {
        fn get(&self, o: &Object) -> git_asset_core::tree::TreeResult<Option<Vec<u8>>> {
            if *o == self.row {
                Ok(self.corrupt.then(|| b"corrupt".to_vec()))
            } else {
                self.nodes.get(o)
            }
        }
        fn put(&mut self, _: &[u8]) -> git_asset_core::tree::TreeResult<Object> {
            unreachable!()
        }
    }
    for (corrupt, error) in [(false, Error::Missing), (true, Error::Integrity)] {
        assert_eq!(
            Index::from_root(tree.root().clone())
                .get(
                    &row.object,
                    &mut Broken {
                        nodes: &nodes,
                        row: stored.clone(),
                        corrupt
                    },
                    TreeLimits::default()
                )
                .unwrap_err(),
            error
        );
    }
    let mut invalid = row;
    invalid.object.bytes += 1;
    assert_eq!(invalid.store(&mut nodes).unwrap_err(), Error::Integrity);
}
