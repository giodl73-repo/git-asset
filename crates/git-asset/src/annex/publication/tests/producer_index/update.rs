use super::*;
use crate::annex::{pending::Scope, producer_index::update as roots};
use git_asset_store::{
    metadata::{self, MetadataStore},
    protection::Limits,
    source_claims::{
        Command, Kind, Places,
        admission::{Namespace, Policy},
    },
};

pub(in super::super) struct Fixture {
    temp: tempfile::TempDir,
    namespace: Namespace,
    nodes: MetadataStore,
    profile: roots::Profile,
}
impl Fixture {
    pub(in super::super) fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let places = Places {
            current: vec![k("metadata")],
            pending: vec![k("metadata")],
            predecessor: vec![k("metadata")],
        };
        let mut namespace = Namespace::create_fenced(
            &temp.path().join("roots"),
            k("producer-roots"),
            k("epoch"),
            Policy {
                prefer_active: true,
                classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                    .into_iter()
                    .map(|kind| (kind, places.clone()))
                    .collect(),
            },
        )
        .unwrap();
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
        let profile = roots::Profile {
            scope: Scope {
                namespace: namespace.identity().clone(),
                metadata: nodes.checkpoint().identity().clone(),
                owner: k("owner"),
                prefix: k("producers"),
                place: k("metadata"),
                allow_store: true,
            },
            limits: Limits {
                millis: 60000,
                roots: 256,
                bindings: 20000,
                associations: 100000,
                reads: 100000,
                read_bytes: 32 * 1024 * 1024,
            },
        };
        namespace
            .apply(
                k("enroll-producers"),
                Command::Enroll {
                    source: k("producers"),
                    owner: k("owner"),
                    class: Kind::Job,
                },
                &nodes,
                &profile.limits,
            )
            .unwrap();
        Self {
            temp,
            namespace,
            nodes,
            profile,
        }
    }
    #[cfg(windows)]
    pub(in super::super) fn target(&mut self) -> roots::initial::Target<'_> {
        roots::initial::Target {
            profile: &self.profile,
            namespace: &mut self.namespace,
            nodes: &mut self.nodes,
        }
    }
    fn row(&mut self, name: &str) -> Record {
        let p = profile(self.temp.path());
        let mut request = request(&mut self.nodes);
        request.operation = k(name);
        request.candidate.object = Object::from_bytes(name.as_bytes());
        Record::new(plan(&mut self.nodes, &p, request), None).unwrap()
    }
    fn prepare(&mut self, rows: &[roots::Change]) -> roots::Plan {
        let current = self
            .namespace
            .source(&k("producers"))
            .unwrap()
            .unwrap()
            .current;
        roots::prepare(
            &self.profile,
            current.as_ref(),
            rows,
            &mut self.namespace,
            &mut self.nodes,
        )
        .unwrap()
        .unwrap()
    }
}
#[test]
fn roots_keep_old_records_fence_writers_and_recover_original_after_restart() {
    let mut f = Fixture::new();
    let row = f.row("first");
    let changes = [roots::Change {
        expected_record: None,
        record: row.clone(),
    }];
    let first = f.prepare(&changes);
    let competing_row = f.row("competing");
    let competing = f.prepare(&[roots::Change {
        expected_record: None,
        record: competing_row,
    }]);
    // Persist before apply; returned success can be lost without repeating selection.
    let raw = first.to_json().unwrap();
    std::fs::write(f.temp.path().join("original-plan"), &raw).unwrap();
    let mut calls = 0;
    let result = roots::apply(&f.profile, &first, &mut f.namespace, &mut f.nodes, || {
        calls += 1;
        if calls == 2 {
            Err(Error::Denied)
        } else {
            Ok(())
        }
    })
    .unwrap();
    assert_eq!(result.post_effect_hold, Some("denied"));
    assert_eq!(result.current, Some(true));
    assert_eq!(result.media_custody_verified, None);
    assert!(!result.authorizes_eviction);
    assert_eq!(
        roots::apply(
            &f.profile,
            &competing,
            &mut f.namespace,
            &mut f.nodes,
            || Ok(())
        )
        .err(),
        Some(Error::Stale)
    );
    let checkpoint = f.namespace.checkpoint();
    let metadata = f.nodes.checkpoint();
    let mut namespace = Namespace::open(&f.temp.path().join("roots"), &checkpoint).unwrap();
    let mut nodes = MetadataStore::open(&f.temp.path().join("metadata"), &metadata).unwrap();
    let recovered =
        roots::Plan::from_json(&std::fs::read(f.temp.path().join("original-plan")).unwrap())
            .unwrap();
    let mut readonly = f.profile.clone();
    readonly.scope.allow_store = false;
    let reopened = roots::apply(&readonly, &recovered, &mut namespace, &mut nodes, || {
        panic!("historical status must not admit another effect")
    })
    .unwrap();
    assert_eq!(reopened.version, result.version);
    assert_eq!(checkpoint, namespace.checkpoint());
    // New retained membership changes a record, preserving its original producer.
    let mut updated = row.clone();
    updated.retained = Some(crate::annex::snapshot::Snapshot {
        namespace: Digest::blob(b"declared retained namespace"),
        source: k("history"),
        owner: k("owner"),
        version: result.version.clone(),
        place: k("cold"),
        path: LogicalPath::parse("assets/old").unwrap(),
        object: row.object.clone(),
    });
    let next = roots::prepare(
        &f.profile,
        Some(&result.version),
        &[roots::Change {
            expected_record: Some(row.metadata_object().unwrap()),
            record: updated.clone(),
        }],
        &mut namespace,
        &mut nodes,
    )
    .unwrap()
    .unwrap();
    let advanced = roots::apply(&f.profile, &next, &mut namespace, &mut nodes, || Ok(())).unwrap();
    let state = namespace.source(&k("producers")).unwrap().unwrap();
    assert!(state.predecessors.contains(&result.version));
    assert_eq!(
        Index::from_root(first.root().clone())
            .get(&row.object, &mut nodes, TreeLimits::default())
            .unwrap()
            .0,
        Some(row.clone())
    );
    assert_eq!(
        Index::from_root(next.root().clone())
            .get(&row.object, &mut nodes, TreeLimits::default())
            .unwrap()
            .0,
        Some(updated.clone())
    );
    let checkpoint = namespace.checkpoint();
    let historical = roots::apply(&readonly, &recovered, &mut namespace, &mut nodes, || {
        panic!("must not reselect old root")
    })
    .unwrap();
    assert_eq!(historical.current, Some(false));
    assert_eq!(historical.version, result.version);
    assert_eq!(checkpoint, namespace.checkpoint());
    assert!(
        roots::prepare(
            &f.profile,
            Some(&advanced.version),
            &[roots::Change {
                expected_record: Some(updated.metadata_object().unwrap()),
                record: updated
            }],
            &mut namespace,
            &mut nodes
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(checkpoint, namespace.checkpoint());
    // A fresh process uses only durable plan/profile/checkpoint files.
    let handoff = serde_json::json!({"profile":readonly,"namespace":namespace.checkpoint(),
        "metadata":nodes.checkpoint(),"version":result.version,"object":row.object,
        "record":row.metadata_object().unwrap()});
    std::fs::write(
        f.temp.path().join("reopen.json"),
        serde_json::to_vec(&handoff).unwrap(),
    )
    .unwrap();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "annex::publication::tests::producer_index::update::reopen_worker",
            "--nocapture",
        ])
        .env("GIT_ASSET_PRODUCER_ROOT_REOPEN", f.temp.path())
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(!profile(f.temp.path()).pool.pool.repository.exists());
}
#[test]
fn refused_permissions_limits_records_and_undeclared_edits_do_not_select() {
    let mut f = Fixture::new();
    let row = f.row("initial");
    let checkpoint = f.namespace.checkpoint();
    let mut denied = f.profile.clone();
    denied.scope.allow_store = false;
    assert_eq!(
        roots::prepare(
            &denied,
            None,
            &[roots::Change {
                expected_record: None,
                record: row.clone()
            }],
            &mut f.namespace,
            &mut f.nodes
        )
        .err(),
        Some(Error::Denied)
    );
    let mut wrong_owner = f.profile.clone();
    wrong_owner.scope.owner = k("another-owner");
    assert_eq!(
        roots::prepare(
            &wrong_owner,
            None,
            &[roots::Change {
                expected_record: None,
                record: row.clone()
            }],
            &mut f.namespace,
            &mut f.nodes
        )
        .err(),
        Some(Error::Denied)
    );
    let mut wrong_provider = f.profile.clone();
    wrong_provider.scope.metadata = Digest::blob(b"wrong metadata provider");
    assert_eq!(
        roots::prepare(
            &wrong_provider,
            None,
            &[roots::Change {
                expected_record: None,
                record: row.clone()
            }],
            &mut f.namespace,
            &mut f.nodes
        )
        .err(),
        Some(Error::Stale)
    );
    let too_many: Vec<_> = (0..33)
        .map(|_| roots::Change {
            expected_record: None,
            record: row.clone(),
        })
        .collect();
    assert_eq!(
        roots::prepare(&f.profile, None, &too_many, &mut f.namespace, &mut f.nodes).err(),
        Some(Error::Budget)
    );
    assert_eq!(
        roots::Plan::from_json(&vec![b' '; 65537]).err(),
        Some(Error::Budget)
    );
    let mut tiny = f.profile.clone();
    tiny.limits.read_bytes = 1;
    assert_eq!(
        roots::prepare(
            &tiny,
            None,
            &[roots::Change {
                expected_record: None,
                record: row.clone()
            }],
            &mut f.namespace,
            &mut f.nodes
        )
        .err(),
        Some(Error::Budget)
    );
    let first = f.prepare(&[roots::Change {
        expected_record: None,
        record: row.clone(),
    }]);
    assert_eq!(
        roots::apply(&denied, &first, &mut f.namespace, &mut f.nodes, || Ok(())).err(),
        Some(Error::Denied)
    );
    assert_eq!(
        roots::apply(&f.profile, &first, &mut f.namespace, &mut f.nodes, || Err(
            Error::Busy
        ))
        .err(),
        Some(Error::Busy)
    );
    assert_eq!(checkpoint, f.namespace.checkpoint());
    // Tampering in an undeclared second row must not become an authorized root.
    let other = f.row("hidden");
    let (path, object) = other.store(&mut f.nodes).unwrap();
    let (tree, _) = Tree::from_root(first.root().clone())
        .edit(
            first.root(),
            &BTreeMap::from([(path, Some(object))]),
            &mut f.nodes,
            TreeLimits::default(),
        )
        .unwrap();
    f.nodes.finish().unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&first.to_json().unwrap()).unwrap();
    wire["root"] = serde_json::to_value(tree.root()).unwrap();
    let forged = roots::Plan::from_json(&identity::canonical_json(&wire).unwrap()).unwrap();
    assert_eq!(
        roots::apply(&f.profile, &forged, &mut f.namespace, &mut f.nodes, || Ok(
            ()
        ))
        .err(),
        Some(Error::Integrity)
    );
    assert_eq!(checkpoint, f.namespace.checkpoint());
    assert_eq!(
        roots::Plan::from_json(&serde_json::to_vec_pretty(&wire).unwrap()).err(),
        Some(Error::Integrity)
    );
    let result = roots::apply(
        &f.profile,
        &first,
        &mut f.namespace,
        &mut f.nodes,
        || Ok(()),
    )
    .unwrap();
    assert_eq!(
        roots::prepare(
            &f.profile,
            None,
            &[roots::Change {
                expected_record: None,
                record: other
            }],
            &mut f.namespace,
            &mut f.nodes
        )
        .err(),
        Some(Error::Stale)
    );
    let mut replacement = row.clone();
    replacement.producer = f.row("different-recipe").producer;
    replacement.object = replacement.producer.candidate().object.clone();
    let native_profile = profile(f.temp.path());
    let mut request = request(&mut f.nodes);
    request.operation = k("changed-original");
    request.candidate.object = row.object.clone();
    let substituted = Record::new(plan(&mut f.nodes, &native_profile, request), None).unwrap();
    assert_eq!(
        roots::prepare(
            &f.profile,
            Some(&result.version),
            &[roots::Change {
                expected_record: Some(row.metadata_object().unwrap()),
                record: substituted
            }],
            &mut f.namespace,
            &mut f.nodes
        )
        .err(),
        Some(Error::Conflict)
    );
    // Wrong expected record is a conflict, even if the requested metadata is valid.
    assert_eq!(
        roots::prepare(
            &f.profile,
            Some(&result.version),
            &[roots::Change {
                expected_record: None,
                record: row.clone()
            }],
            &mut f.namespace,
            &mut f.nodes
        )
        .err(),
        Some(Error::Conflict)
    );
    assert_eq!(
        roots::prepare(
            &f.profile,
            Some(&result.version),
            &[
                roots::Change {
                    expected_record: None,
                    record: replacement.clone()
                },
                roots::Change {
                    expected_record: None,
                    record: replacement
                }
            ],
            &mut f.namespace,
            &mut f.nodes
        )
        .err(),
        Some(Error::Conflict)
    );
}

#[test]
fn a_competing_handle_advancing_during_admission_fences_the_original_update() {
    let mut f = Fixture::new();
    let a = f.row("writer-a");
    let b = f.row("writer-b");
    let first = f.prepare(&[roots::Change {
        expected_record: None,
        record: a,
    }]);
    let second = f.prepare(&[roots::Change {
        expected_record: None,
        record: b,
    }]);
    let mut other =
        Namespace::open(&f.temp.path().join("roots"), &f.namespace.checkpoint()).unwrap();
    let mut other_nodes =
        MetadataStore::open(&f.temp.path().join("metadata"), &f.nodes.checkpoint()).unwrap();
    let mut raced = false;
    let result = roots::apply(&f.profile, &first, &mut f.namespace, &mut f.nodes, || {
        if !raced {
            raced = true;
            roots::apply(&f.profile, &second, &mut other, &mut other_nodes, || Ok(())).unwrap();
        }
        Ok(())
    });
    assert!(result.is_err());
    assert!(raced);
    assert!(
        roots::status(&f.profile, &first, &mut f.namespace, &f.nodes)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        roots::status(&f.profile, &second, &mut f.namespace, &f.nodes)
            .unwrap()
            .unwrap()
            .current,
        Some(true)
    );
}

#[test]
fn reopen_worker() {
    let Some(root) = std::env::var_os("GIT_ASSET_PRODUCER_ROOT_REOPEN") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("reopen.json")).unwrap()).unwrap();
    let profile: roots::Profile = serde_json::from_value(data["profile"].clone()).unwrap();
    let mut namespace = Namespace::open(
        &root.join("roots"),
        &serde_json::from_value(data["namespace"].clone()).unwrap(),
    )
    .unwrap();
    let mut nodes = MetadataStore::open(
        &root.join("metadata"),
        &serde_json::from_value(data["metadata"].clone()).unwrap(),
    )
    .unwrap();
    let checkpoint = namespace.checkpoint();
    let plan = roots::Plan::from_json(&std::fs::read(root.join("original-plan")).unwrap()).unwrap();
    let report = roots::apply(&profile, &plan, &mut namespace, &mut nodes, || {
        panic!("read-only recovery cannot reselect")
    })
    .unwrap();
    assert_eq!(
        serde_json::to_value(&report.version).unwrap(),
        data["version"]
    );
    assert_eq!(report.current, Some(false));
    let object: Object = serde_json::from_value(data["object"].clone()).unwrap();
    let old = Index::from_root(report.root)
        .get(&object, &mut nodes, TreeLimits::default())
        .unwrap()
        .0
        .unwrap();
    assert_eq!(
        serde_json::to_value(old.metadata_object().unwrap()).unwrap(),
        data["record"]
    );
    assert_eq!(checkpoint, namespace.checkpoint());
}

#[test]
fn an_empty_initial_index_is_explicit_and_repeating_it_is_a_noop() {
    let mut f = Fixture::new();
    let first = f.prepare(&[]);
    let report = roots::apply(
        &f.profile,
        &first,
        &mut f.namespace,
        &mut f.nodes,
        || Ok(()),
    )
    .unwrap();
    let page = Index::from_root(report.root)
        .page(None, 1, &mut f.nodes, TreeLimits::default())
        .unwrap();
    assert!(page.records.is_empty());
    assert!(page.next.is_none());
    let checkpoint = f.namespace.checkpoint();
    assert!(
        roots::prepare(
            &f.profile,
            Some(&report.version),
            &[],
            &mut f.namespace,
            &mut f.nodes
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(checkpoint, f.namespace.checkpoint());
}

#[test]
fn stale_single_row_plan_can_be_rebased_after_an_unrelated_insert() {
    let mut f = Fixture::new();
    let row = f.row("original");
    let original = f.prepare(&[roots::Change {
        expected_record: None,
        record: row.clone(),
    }]);
    let other = f.row("unrelated");
    let competing = f.prepare(&[roots::Change {
        expected_record: None,
        record: other.clone(),
    }]);
    let advanced = roots::apply(
        &f.profile,
        &competing,
        &mut f.namespace,
        &mut f.nodes,
        || Ok(()),
    )
    .unwrap();
    let successor = roots::prepare_successor(&f.profile, &original, &mut f.namespace, &mut f.nodes)
        .unwrap()
        .unwrap();
    assert_eq!(successor.expected(), Some(&advanced.version));
    let applied = roots::apply(
        &f.profile,
        &successor,
        &mut f.namespace,
        &mut f.nodes,
        || Ok(()),
    )
    .unwrap();
    let index = Index::from_root(applied.root);
    for expected in [row, other] {
        assert_eq!(
            index
                .get(&expected.object, &mut f.nodes, TreeLimits::default())
                .unwrap()
                .0,
            Some(expected)
        );
    }
    assert!(matches!(
        roots::prepare_successor(&f.profile, &competing, &mut f.namespace, &mut f.nodes),
        Err(Error::Conflict)
    ));
}
