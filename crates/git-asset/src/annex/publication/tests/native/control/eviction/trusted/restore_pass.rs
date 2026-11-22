//! Real Git/native bounded scheduling, including immutable preview and new head retry.
use super::*;
use crate::annex::{
    pending,
    producer_index::{Index, Record},
    publication::restore as native,
};
use crate::github::native::{
    producers::{Head, Heads},
    restore::pass::Pass,
};
fn head(f: &mut Fixture, plan: Plan, name: &str, write: bool) -> Head {
    let mut nodes = MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let (path, object) = Record::new(plan, None).unwrap().store(&mut nodes).unwrap();
    let (tree, _) = git_asset_core::tree::Tree::build(
        &BTreeMap::from([(path, object)]),
        &mut nodes,
        TreeLimits::default(),
    )
    .unwrap();
    nodes.finish().unwrap();
    f.metadata = nodes.finished_reader().unwrap();
    let mut head =
        super::super::super::super::github_worker::heads::register(f, tree.root().clone(), name)
            .head;
    head.profile.scope.allow_store = write;
    head
}
impl State {
    pub(in super::super) fn restore_pass(
        &mut self,
        f: &mut Fixture,
        keeper: &mut Fixture,
        owner: &OwnerProfile,
        evicted: &NativePlan,
        old: &Plan,
    ) -> Plan {
        let working = head(f, old.clone(), "scheduled-working", true);
        let retained = head(keeper, keeper.plan.clone(), "scheduled-retained", false);
        self.policy.producer_heads = Some(Heads {
            source: retained,
            destination: working.clone(),
        });
        self.policy.allow_copy = true;
        self.signer.trust.placement_enrollment = self.policy.id().unwrap();
        let scope = self
            .policy
            .actions
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .clone();
        let original = pending::Action {
            operation: pending::Operation::Eviction(Box::new(evicted.clone())),
            working: old.clone(),
            retained: keeper.plan.clone(),
        };
        pending::save(
            &scope,
            self.policy.id().unwrap(),
            self.signer.commit.as_str(),
            &original,
            &mut self.pending,
            &mut self.metadata,
            &Fixture::limits(),
        )
        .unwrap();
        f.metadata = self.metadata.finished_reader().unwrap();
        let profile = f.profile.clone();
        let control = f.control_profile.as_ref().unwrap().clone();
        let mut native_ns =
            Namespace::open(&f.root.join("namespace-0"), &f.namespaces[0].checkpoint()).unwrap();
        let mut native_nodes =
            MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
        let mut index_ns = Namespace::open(&working.root, &working.minimum).unwrap();
        let mut index_nodes =
            MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
        let limits = Fixture::limits();
        let mut context = native::Context {
            profile: &profile,
            control: &control,
            retirement: owner,
            namespace: &mut native_ns,
            metadata: &mut native_nodes,
            limits: &limits,
        };
        let mut target = crate::annex::producer_index::update::restore::Target {
            profile: &working.profile,
            namespace: &mut index_ns,
            nodes: &mut index_nodes,
        };
        let native = source::annex::Source {
            repository: &self.repository,
        };
        let request = self.signer.verify(&self.signer.claims, 1000).unwrap();
        let mut observer =
            Namespace::open(&f.root.join("namespace-1"), &f.namespaces[1].checkpoint()).unwrap();
        let mut worker = Execution {
            trust: &self.signer.trust,
            request: &request,
            policy: &self.policy,
            enrollment: &self.enrollment,
            repository: &native,
            namespace: &mut observer,
            metadata: &mut self.metadata,
            limits: &limits,
        };
        let budget = github::placement::Budget {
            pages: 1,
            objects: 1,
            stream_bytes: 1024 * 1024,
            millis: 60000,
        };
        let checkpoint = f.namespaces[0].checkpoint();
        let records = self.pending.checkpoint();
        let index = target.namespace.checkpoint();
        macro_rules! pass {
            ($apply:expr,$cursor:expr) => {
                Pass {
                    cursor: $cursor,
                    budget: &budget,
                    pending: &mut self.pending,
                    context: &mut context,
                    target: &mut target,
                    apply: $apply,
                }
            };
        }
        let preview = worker
            .reconcile_restores(pass!(false, None), &mut f.resources(), || Ok(1000))
            .unwrap();
        assert_eq!(preview.outcomes[0].state, "eligible");
        assert!(preview.outcomes[0].native_saved.is_none());
        assert_eq!(checkpoint, f.namespaces[0].checkpoint());
        assert_eq!(records, self.pending.checkpoint());
        assert_eq!(index, target.namespace.checkpoint());
        let applied = worker
            .reconcile_restores(pass!(true, None), &mut f.resources(), || Ok(1000))
            .unwrap();
        assert!(
            applied.outcomes[0].hold.is_none(),
            "{:?}; native_done={}, index_saved={}",
            applied.outcomes[0]
                .hold
                .as_ref()
                .map(|h| (&h.layer, &h.code)),
            applied.outcomes[0].native.is_some(),
            applied.outcomes[0].index_saved.is_some()
        );
        assert_eq!(applied.outcomes[0].state, "indexed");
        assert!(applied.fresh_pass_required && applied.next.is_none());
        assert!(
            applied.outcomes[0].native_saved.is_some() && applied.outcomes[0].index_saved.is_some()
        );
        let cursor = git_asset_store::placement::Cursor {
            basis: preview.basis,
            after: old.candidate().object.sha256.clone(),
        };
        assert!(matches!(
            worker.reconcile_restores(pass!(false, Some(&cursor)), &mut f.resources(), || Ok(1000)),
            Err(crate::github::native::Error::Admission(
                github::Error::Stale
            ))
        ));
        let checkpoint = f.namespaces[0].checkpoint();
        let index_checkpoint = target.namespace.checkpoint();
        let retry = worker
            .reconcile_restores(pass!(true, None), &mut f.resources(), || Ok(1000))
            .unwrap();
        assert_eq!(retry.outcomes[0].state, "not_applicable");
        assert!(retry.outcomes[0].hold.is_none());
        assert_eq!(checkpoint, f.namespaces[0].checkpoint());
        assert_eq!(index_checkpoint, target.namespace.checkpoint());
        let state = target
            .namespace
            .source(&working.profile.scope.prefix)
            .unwrap()
            .unwrap();
        let row = Index::from_root(state.current.unwrap().selection.root)
            .get(&old.candidate().object, target.nodes, TreeLimits::default())
            .unwrap()
            .0
            .unwrap();
        f.metadata = target.nodes.finished_reader().unwrap();
        let mut reader =
            crate::annex::publication::Publisher::controlled(profile.clone(), control.clone())
                .unwrap();
        let mut restored = reader.reopen(&row.producer, &mut f.resources()).unwrap();
        let mut bytes = Vec::new();
        restored.read_to_end(&mut bytes).unwrap();
        assert_eq!(Object::from_bytes(&bytes), old.candidate().object);
        drop(restored);
        // A metadata-only rollback must not turn an old receipt into current
        // indexing. Recover the index body even with no native-phase slot.
        let state = target
            .namespace
            .source(&working.profile.scope.prefix)
            .unwrap()
            .unwrap();
        target
            .namespace
            .select(
                &git_asset_store::source_claims::admission::Select {
                    prepare: k("rollback-index-prepare"),
                    adopt: k("rollback-index-adopt"),
                    source: working.profile.scope.prefix.clone(),
                    owner: working.profile.scope.owner.clone(),
                    expected_revision: state.revision,
                    selection: preview.producer.version.selection.clone(),
                },
                target.nodes,
                &limits,
            )
            .unwrap();
        let index_handle = applied.outcomes[0].index_saved.as_ref().unwrap();
        let loaded = pending::reopen(
            &scope,
            index_handle,
            &mut self.pending,
            worker.metadata,
            &limits,
        )
        .unwrap();
        let index_action = loaded.action.clone();
        drop(loaded);
        let mut only = super::super::super::super::github_worker::batch::pending_namespace_at(
            &f.root.join("index-only-actions"),
        );
        let mut only_scope = scope.clone();
        only_scope.namespace = only.identity().clone();
        pending::save(
            &only_scope,
            worker.policy.id().unwrap(),
            worker.request.commit().as_str(),
            &original,
            &mut only,
            worker.metadata,
            &limits,
        )
        .unwrap();
        pending::save(
            &only_scope,
            worker.policy.id().unwrap(),
            worker.request.commit().as_str(),
            &index_action,
            &mut only,
            worker.metadata,
            &limits,
        )
        .unwrap();
        assert!(
            pending::find(
                &only_scope,
                pending::Phase::Restore,
                &old.candidate().object,
                &old.id().unwrap(),
                &mut only,
                worker.metadata,
                &limits
            )
            .unwrap()
            .is_none()
        );
        // End the old policy borrow before authorizing the new recovery scope.
        self.policy.actions.as_mut().unwrap().pending = Some(only_scope);
        self.signer.trust.placement_enrollment = self.policy.id().unwrap();
        let proof = self.signer.verify(&self.signer.claims, 1000).unwrap();
        let mut worker = Execution {
            trust: &self.signer.trust,
            request: &proof,
            policy: &self.policy,
            enrollment: &self.enrollment,
            repository: &native,
            namespace: &mut observer,
            metadata: &mut self.metadata,
            limits: &limits,
        };
        let recovery = worker
            .reconcile_restores(
                Pass {
                    cursor: None,
                    budget: &budget,
                    pending: &mut only,
                    context: &mut context,
                    target: &mut target,
                    apply: true,
                },
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert!(recovery.outcomes[0].registration.is_some());
        assert!(recovery.outcomes[0].native_saved.is_none());
        assert_eq!(
            recovery.outcomes[0].hold.as_ref().map(|h| h.code.as_str()),
            Some("stale")
        );
        row.producer
    }
}
