//! Restore plans use the original retained four-blob action mechanism.
use super::*;
use crate::annex::pending::{self, Action, Operation, Scope};
pub(super) struct Harness {
    root: PathBuf,
    metadata_root: PathBuf,
    pub(super) scope: Scope,
    pub(super) namespace: Namespace,
    nodes: MetadataStore,
}
impl Harness {
    pub(super) fn new(f: &Fixture, name: &str) -> Self {
        let root = f.root.join(name);
        let places = Places {
            current: vec![k("metadata")],
            pending: vec![k("metadata")],
            predecessor: vec![k("metadata")],
        };
        let namespace = Namespace::create_fenced(
            &root,
            k(name),
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
        let nodes = MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
        let scope = Scope {
            namespace: namespace.identity().clone(),
            metadata: nodes.checkpoint().identity().clone(),
            owner: k("owner"),
            prefix: k("restores"),
            place: k("metadata"),
            allow_store: true,
        };
        Self {
            root,
            metadata_root: f.root.join("metadata"),
            scope,
            namespace,
            nodes,
        }
    }
    pub(super) fn save(&mut self, action: Action) -> pending::Saved {
        let limits = Fixture::limits();
        let policy = Digest::blob(b"synthetic fixed policy");
        let commit = "a".repeat(40);
        let checkpoint = self.namespace.checkpoint();
        let mut wrong = action.clone();
        wrong.working = wrong
            .working
            .with_context(Some(Digest::blob(b"wrong retired recipe")));
        assert_eq!(
            pending::save(
                &self.scope,
                policy.clone(),
                &commit,
                &wrong,
                &mut self.namespace,
                &mut self.nodes,
                &limits
            )
            .err(),
            Some(Error::Integrity)
        );
        assert_eq!(checkpoint, self.namespace.checkpoint());
        if let Operation::RestoreIndex(index) = &action.operation {
            let mut bad = action.clone();
            let mut raw: serde_json::Value =
                serde_json::from_slice(&index.original.restore.to_json().unwrap()).unwrap();
            raw["native"]["request"]["expected_revision"] = serde_json::json!(999);
            let changed = crate::annex::publication::restore::Plan::from_json(
                &git_asset_core::identity::canonical_json(&raw).unwrap(),
            )
            .unwrap();
            if let Operation::RestoreIndex(body) = &mut bad.operation {
                body.original.restore = changed;
            }
            assert_eq!(
                pending::save(
                    &self.scope,
                    policy.clone(),
                    &commit,
                    &bad,
                    &mut self.namespace,
                    &mut self.nodes,
                    &limits
                )
                .err(),
                Some(Error::Integrity)
            );
            assert_eq!(checkpoint, self.namespace.checkpoint());
        }
        let saved = pending::save(
            &self.scope,
            policy.clone(),
            &commit,
            &action,
            &mut self.namespace,
            &mut self.nodes,
            &limits,
        )
        .unwrap();
        let mut reader = Namespace::open(&self.root, &self.namespace.checkpoint()).unwrap();
        let reopened =
            pending::reopen(&self.scope, &saved, &mut reader, &self.nodes, &limits).unwrap();
        assert_eq!(reopened.action.phase(), action.phase());
        assert_eq!(reopened.action.working, action.working);
        assert_eq!(reopened.action.retained, action.retained);
        match (&reopened.action.operation, &action.operation) {
            (Operation::Restore(a), Operation::Restore(b)) => {
                assert_eq!(a.restore.to_json().unwrap(), b.restore.to_json().unwrap())
            }
            (Operation::RestoreIndex(a), Operation::RestoreIndex(b)) => {
                assert_eq!(a.index.to_json().unwrap(), b.index.to_json().unwrap())
            }
            _ => panic!("wrong recovered action"),
        }
        drop(reopened);
        let plan = match &action.operation {
            Operation::Restore(body) => Digest::blob(&body.restore.to_json().unwrap()),
            Operation::RestoreIndex(body) => Digest::blob(&body.index.to_json().unwrap()),
            _ => unreachable!(),
        };
        let fixture = self.root.join("reopen.json");
        fs::write(
            &fixture,
            serde_json::to_vec(&serde_json::json!({
                "scope": &self.scope, "saved": &saved, "namespace": &self.root,
                "minimum": self.namespace.checkpoint(), "metadata": &self.metadata_root,
                "metadata_minimum": self.nodes.checkpoint(), "plan": plan,
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "annex::publication::tests::native::pending::reopen_worker",
                    "--nocapture"
                ])
                .env("GIT_ASSET_PENDING_REOPEN_FIXTURE", fixture)
                .status()
                .unwrap()
                .success()
        );
        let mut readonly = self.scope.clone();
        readonly.allow_store = false;
        // The original wins even against an invalid later preview in the same slot.
        let mut competing = action.clone();
        let changed = |plan: &crate::annex::publication::restore::Plan| {
            let mut wire: serde_json::Value =
                serde_json::from_slice(&plan.to_json().unwrap()).unwrap();
            wire["original"] = serde_json::to_value(Digest::blob(b"competing original")).unwrap();
            crate::annex::publication::restore::Plan::from_json(
                &git_asset_core::identity::canonical_json(&wire).unwrap(),
            )
            .unwrap()
        };
        match &mut competing.operation {
            Operation::Restore(body) => body.restore = changed(&body.restore),
            Operation::RestoreIndex(body) => {
                body.original.restore = changed(&body.original.restore)
            }
            _ => unreachable!(),
        }
        assert_eq!(
            pending::save(
                &readonly,
                policy,
                &commit,
                &competing,
                &mut reader,
                &mut self.nodes,
                &limits
            )
            .unwrap(),
            saved
        );
        let found = pending::find(
            &readonly,
            action.phase(),
            action.object(),
            &action.working.id().unwrap(),
            &mut reader,
            &self.nodes,
            &limits,
        )
        .unwrap()
        .unwrap();
        assert_eq!(found.saved, saved);
        drop(found);
        saved
    }
}
