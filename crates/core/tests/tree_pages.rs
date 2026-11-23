use git_asset_core::{LogicalPath, Object, tree::*};
use std::collections::BTreeMap;
fn path(s: &str) -> LogicalPath {
    LogicalPath::parse(s).unwrap()
}

#[test]
fn bounded_flat_pages_cover_exactly_one_large_immutable_root() {
    let mut nodes = MemoryNodes::default();
    let bindings: BTreeMap<_, _> = (0..10000)
        .map(|i| {
            (
                path(&format!("k{i:05}")),
                Object::from_bytes(format!("v{i}").as_bytes()),
            )
        })
        .collect();
    let (tree, _) = Tree::build(&bindings, &mut nodes, TreeLimits::default()).unwrap();
    let limits = TreeLimits {
        max_reads: 200,
        max_read_bytes: 200000,
        ..TreeLimits::default()
    };
    let mut after = None;
    let mut seen = BTreeMap::new();
    loop {
        let page = tree
            .flat_page(after.as_ref(), 37, &mut nodes, limits)
            .unwrap();
        assert!(page.metrics.reads <= 200);
        assert!(page.entries.len() <= 37);
        for (key, value) in page.entries {
            assert!(seen.insert(key, value).is_none());
        }
        match page.next {
            Some(next) => after = Some(next),
            None => break,
        }
    }
    assert_eq!(seen, bindings);
    assert_eq!(
        tree.flat_page(Some(&path("missing")), 1, &mut nodes, limits)
            .unwrap_err(),
        TreeError::StaleRoot
    );
    assert_eq!(
        tree.flat_page(None, 0, &mut nodes, limits).unwrap_err(),
        TreeError::Budget
    );
    assert_eq!(
        tree.flat_page(None, 4097, &mut nodes, limits).unwrap_err(),
        TreeError::Budget
    );
    let short = TreeLimits {
        max_reads: 1,
        ..limits
    };
    assert_eq!(
        tree.flat_page(None, 1, &mut nodes, short).unwrap_err(),
        TreeError::Budget
    );
    let first = tree.flat_page(None, 1, &mut nodes, limits).unwrap();
    let short = TreeLimits {
        max_read_bytes: first.metrics.read_bytes - 1,
        ..limits
    };
    assert_eq!(
        tree.flat_page(None, 1, &mut nodes, short).unwrap_err(),
        TreeError::Budget
    );
}

#[test]
fn directories_and_missing_or_corrupt_required_metadata_refuse() {
    let mut nodes = MemoryNodes::default();
    let (tree, _) = Tree::build(
        &BTreeMap::from([(path("a/file"), Object::from_bytes(b"v"))]),
        &mut nodes,
        TreeLimits::default(),
    )
    .unwrap();
    assert_eq!(
        tree.flat_page(None, 1, &mut nodes, TreeLimits::default())
            .unwrap_err(),
        TreeError::PathConflict
    );
    struct Broken(bool);
    impl NodeStore for Broken {
        fn get(&self, _: &Object) -> TreeResult<Option<Vec<u8>>> {
            Ok(self.0.then(|| b"wrong".to_vec()))
        }
        fn put(&mut self, _: &[u8]) -> TreeResult<Object> {
            unreachable!()
        }
    }
    for (mode, error) in [
        (false, TreeError::MissingNode(tree.root().clone())),
        (true, TreeError::Integrity),
    ] {
        assert_eq!(
            tree.flat_page(None, 1, &mut Broken(mode), TreeLimits::default())
                .unwrap_err(),
            error
        );
    }
}

#[test]
fn paging_rejects_forged_entry_counts_and_ancestor_order() {
    fn put(nodes: &mut MemoryNodes, v: &serde_json::Value) -> Object {
        nodes
            .put(&git_asset_core::identity::canonical_json(v).unwrap())
            .unwrap()
    }
    let mut nodes = MemoryNodes::default();
    let (tree, _) = Tree::build(
        &BTreeMap::from([
            (path("a"), Object::from_bytes(b"a")),
            (path("b"), Object::from_bytes(b"b")),
            (path("c"), Object::from_bytes(b"c")),
        ]),
        &mut nodes,
        TreeLimits::default(),
    )
    .unwrap();
    let directory: serde_json::Value =
        serde_json::from_slice(&nodes.get(tree.root()).unwrap().unwrap()).unwrap();
    let mut false_count = directory.clone();
    false_count["root"]["entries"] = serde_json::json!(4);
    let root = put(&mut nodes, &false_count);
    assert_eq!(
        Tree::from_root(root)
            .flat_page(None, 3, &mut nodes, TreeLimits::default())
            .unwrap_err(),
        TreeError::InvalidNode
    );
    let object: Object = serde_json::from_value(directory["root"]["object"].clone()).unwrap();
    let mut parent: serde_json::Value =
        serde_json::from_slice(&nodes.get(&object).unwrap().unwrap()).unwrap();
    let side = if parent["left"].is_null() {
        "right"
    } else {
        "left"
    };
    let object: Object = serde_json::from_value(parent[side]["object"].clone()).unwrap();
    let mut child: serde_json::Value =
        serde_json::from_slice(&nodes.get(&object).unwrap().unwrap()).unwrap();
    child["name"] = parent["name"].clone();
    parent[side]["object"] = serde_json::to_value(put(&mut nodes, &child)).unwrap();
    let mut directory = directory;
    directory["root"]["object"] = serde_json::to_value(put(&mut nodes, &parent)).unwrap();
    let root = put(&mut nodes, &directory);
    assert_eq!(
        Tree::from_root(root)
            .flat_page(None, 3, &mut nodes, TreeLimits::default())
            .unwrap_err(),
        TreeError::InvalidNode
    );
}
