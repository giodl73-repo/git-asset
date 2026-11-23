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
        modes: (bool, bool),
    ) -> Plan {
        let (replan, cycle) = modes;
        let working = head(f, old.clone(), "scheduled-working", true);
        let retained = head(keeper, keeper.plan.clone(), "scheduled-retained", cycle);
        self.policy.producer_heads = Some(Heads {
            source: retained,
            destination: working.clone(),
        });
        self.policy.allow_copy = true;
        let mut history_records = cycle.then(|| {
            let places = Places {
                current: vec![k("metadata")],
                pending: vec![k("metadata")],
                predecessor: vec![k("metadata")],
            };
            Namespace::create_fenced(
                &f.root.join("cycle-history-records"),
                k("cycle-history-records"),
                k("epoch"),
                admission::Policy {
                    prefer_active: true,
                    classes: [Kind::Branch, Kind::Draft, Kind::Job, Kind::Retention]
                        .into_iter()
                        .map(|kind| (kind, places.clone()))
                        .collect(),
                },
            )
            .unwrap()
        });
        let mut history_nodes = cycle.then(|| {
            MetadataStore::open(&keeper.root.join("metadata"), keeper.metadata.checkpoint())
                .unwrap()
        });
        if cycle {
            let actions = self.policy.actions.as_mut().unwrap();
            actions.orchestration = true;
            actions.archive_index = true;
            actions.registration = true;
            actions.backup = true;
            actions.handoff = true;
            actions.history = Some(crate::annex::history::Profile {
                working: crate::annex::history::Working {
                    namespace: self.enrollment.namespace.clone(),
                    source: self.enrollment.source.clone(),
                    owner: self.enrollment.owner.clone(),
                    class: self.enrollment.class.clone(),
                },
                target: crate::annex::history::Target {
                    namespace: keeper.namespaces[1].identity().clone(),
                    owner: k("owner"),
                    place: k("pool"),
                    prefix: k("cycle-history"),
                },
                records: pending::Scope {
                    namespace: history_records.as_ref().unwrap().identity().clone(),
                    metadata: history_nodes
                        .as_ref()
                        .unwrap()
                        .checkpoint()
                        .identity()
                        .clone(),
                    owner: k("owner"),
                    prefix: k("cycle-records"),
                    place: k("metadata"),
                    allow_store: true,
                },
                limits: Fixture::limits(),
                allow_publish: true,
                allow_release: true,
            });
        }
        if cycle {
            // A new committed definition shares bytes, but leaves an exact old root.
            let project = f.root.join("restore-pass-project");
            fs::rename(project.join("assets/next"), project.join("assets/released")).unwrap();
            git(&project, &["add", "-A"]);
            git(
                &project,
                &["commit", "-m", "release renamed synthetic asset"],
            );
            self.signer.commit = source::GitId::parse(
                String::from_utf8(git(&project, &["rev-parse", "HEAD"]))
                    .unwrap()
                    .trim(),
            )
            .unwrap();
            self.signer.claims["sha"] = serde_json::json!(self.signer.commit.as_str());
        }
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
                    replan: false,
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
        if cycle {
            use crate::github::native::{batch::registration, eviction, history, pipeline};
            pipeline::validate(worker.policy).unwrap();
            let source_head = worker
                .policy
                .producer_heads
                .as_ref()
                .unwrap()
                .source
                .clone();
            let mut source_index =
                Namespace::open(&source_head.root, &source_head.minimum).unwrap();
            let mut source_nodes =
                MetadataStore::open(&keeper.root.join("metadata"), keeper.metadata.checkpoint())
                    .unwrap();
            let binding = pipeline::Binding {
                policy: worker.policy.id().unwrap(),
                provenance: worker.request.provenance().clone(),
            };
            let limits = github::placement::Budget {
                pages: 32,
                objects: 128,
                stream_bytes: 16 * 1024 * 1024,
                millis: 300000,
            };
            let predecessors = worker
                .namespace
                .source(&worker.enrollment.source)
                .unwrap()
                .unwrap()
                .predecessors;
            assert!(!predecessors.is_empty());
            // Missing index target must leave the archived working claim intact.
            let held = worker
                .reconcile_history(
                    history::Pass {
                        index: None,
                        cursor: None,
                        budget: &limits,
                        records: history_records.as_mut().unwrap(),
                        metadata: history_nodes.as_mut().unwrap(),
                        replan: false,
                        apply: true,
                    },
                    || Ok(1000),
                )
                .unwrap();
            assert!(held.outcomes.iter().any(|o| o.saved.is_some()));
            assert!(
                held.outcomes
                    .iter()
                    .all(|o| o.hold.is_some() && o.release.is_none())
            );
            assert_eq!(
                worker
                    .namespace
                    .source(&worker.enrollment.source)
                    .unwrap()
                    .unwrap()
                    .predecessors,
                predecessors
            );
            for round in 0..2 {
                let checkpoint = f.ledger.checkpoint();
                let report=pipeline::run(&binding,&limits,|stage,cursor,budget|{
                    let value=match stage {
                        pipeline::Stage::Restore=>worker.reconcile_restores(Pass {cursor,budget,pending:&mut self.pending,context:&mut context,target:&mut target,apply:true,replan:false},&mut f.resources(),||Ok(1000)).map(|r|serde_json::to_value(r).unwrap()),
                        pipeline::Stage::Copy=>worker.reconcile_with_actions(cursor,budget,Some(&mut self.pending),&mut keeper.resources(),&mut f.resources(),||Ok(1000)).map(|r|serde_json::to_value(r).unwrap()),
                        pipeline::Stage::RegisterWorking=>worker.reconcile_registrations(registration::Pass {cursor,budget,pending:&mut self.pending,target:registration::Target {side:pending::RegistrationSide::Working,namespace:target.namespace,nodes:target.nodes},apply:true,replan:false},&mut keeper.resources(),&mut f.resources(),||Ok(1000)).map(|r|serde_json::to_value(r).unwrap()),
                        pipeline::Stage::RegisterRetained=>worker.reconcile_registrations(registration::Pass {cursor,budget,pending:&mut self.pending,target:registration::Target {side:pending::RegistrationSide::Retained,namespace:&mut source_index,nodes:&mut source_nodes},apply:true,replan:false},&mut keeper.resources(),&mut f.resources(),||Ok(1000)).map(|r|serde_json::to_value(r).unwrap()),
                        pipeline::Stage::History=>worker.reconcile_history(history::Pass {index:Some(history::indexing::Target {namespace:&mut source_index,actions:&mut self.pending}),cursor,budget,records:history_records.as_mut().unwrap(),metadata:history_nodes.as_mut().unwrap(),replan: false,apply:true},||Ok(1000)).map(|r|serde_json::to_value(r).unwrap()),
                        pipeline::Stage::Eviction=>worker.reconcile_evictions(eviction::Pass {cursor,budget,pending:&mut self.pending,apply:true},&mut keeper.resources(),&mut f.resources(),||Ok(1000)).map(|r|serde_json::to_value(r).unwrap()),
                    };
                    Ok(value.unwrap_or_else(|error|serde_json::json!({"failure":crate::github::native::batch::Failure::from(&error)})))
                }).unwrap();
                assert!(
                    report.ordered_passes_complete,
                    "{}",
                    serde_json::to_string(&report).unwrap()
                );
                assert!(report.storage_converged.is_none());
                assert_eq!(report.stages.len(), 6);
                if round == 1 {
                    assert_eq!(checkpoint, f.ledger.checkpoint());
                }
            }
            assert!(
                worker
                    .namespace
                    .source(&worker.enrollment.source)
                    .unwrap()
                    .unwrap()
                    .predecessors
                    .is_empty()
            );
            let retained_version = source_index
                .source(&source_head.profile.scope.prefix)
                .unwrap()
                .unwrap()
                .current
                .unwrap();
            let retained_row = Index::from_root(retained_version.selection.root)
                .get(
                    &old.candidate().object,
                    history_nodes.as_mut().unwrap(),
                    TreeLimits::default(),
                )
                .unwrap()
                .0
                .unwrap();
            let archived = retained_row.retained.as_ref().unwrap();
            assert_eq!(archived.path.as_str(), "assets/next");
            let mut retained = keeper
                .publisher()
                .open_retained(
                    retained_row
                        .publication
                        .as_ref()
                        .unwrap_or(&retained_row.producer),
                    archived,
                    &mut keeper.resources(),
                )
                .unwrap();
            let mut bytes = Vec::new();
            retained.read_to_end(&mut bytes).unwrap();
            assert_eq!(Object::from_bytes(&bytes), archived.object);
            drop(retained);
            let current = target
                .namespace
                .source(&working.profile.scope.prefix)
                .unwrap()
                .unwrap()
                .current
                .unwrap();
            let row = Index::from_root(current.selection.root)
                .get(&old.candidate().object, target.nodes, TreeLimits::default())
                .unwrap()
                .0
                .unwrap();
            let mut read = crate::annex::managed::Reader::new(f.profile.pool.clone())
                .unwrap()
                .open(&old.candidate().object)
                .unwrap();
            let mut bytes = Vec::new();
            read.read_to_end(&mut bytes).unwrap();
            assert_eq!(Object::from_bytes(&bytes), old.candidate().object);
            return row.producer;
        }
        use crate::annex::producer_index::update::restore as index_restore;
        let prior = if replan {
            let restore = native::prepare(
                &mut context,
                native::Original {
                    eviction: evicted,
                    publication: old,
                },
                native::Operations {
                    native: k("prepared-native"),
                    control: k("prepared-control"),
                    source: old.retention_source().clone(),
                },
            )
            .unwrap();
            let original_body = pending::RestoreAction {
                eviction: evicted.clone(),
                restore: restore.clone(),
            };
            let action = pending::Action {
                operation: pending::Operation::Restore(Box::new(original_body.clone())),
                working: old.clone(),
                retained: keeper.plan.clone(),
            };
            let saved = pending::save(
                &scope,
                worker.policy.id().unwrap(),
                worker.request.commit().as_str(),
                &action,
                &mut self.pending,
                worker.metadata,
                &limits,
            )
            .unwrap();
            let restored = worker
                .restore_saved(
                    &saved,
                    &mut self.pending,
                    &mut context,
                    &mut f.resources(),
                    || Ok(1000),
                )
                .unwrap();
            drop(restored);
            let baseline = index_restore::Baseline {
                version: preview.producer.version.clone(),
                record: Record::new(old.clone(), None)
                    .unwrap()
                    .metadata_object()
                    .unwrap(),
            };
            let frozen = index_restore::prepare(
                &mut target,
                &baseline,
                &context,
                native::Original {
                    eviction: evicted,
                    publication: old,
                },
                &restore,
                &mut f.resources(),
            )
            .unwrap();
            assert!(matches!(
                index_restore::prepare_replan(
                    &mut target,
                    &baseline,
                    &context,
                    native::Original {
                        eviction: evicted,
                        publication: old
                    },
                    &frozen,
                    &mut f.resources()
                ),
                Err(Error::Stale)
            ));
            let action = pending::Action {
                operation: pending::Operation::RestoreIndex(Box::new(
                    pending::RestoreIndexAction {
                        original: original_body,
                        index: frozen.clone(),
                    },
                )),
                ..action
            };
            let saved = pending::save(
                &scope,
                worker.policy.id().unwrap(),
                worker.request.commit().as_str(),
                &action,
                &mut self.pending,
                worker.metadata,
                &limits,
            )
            .unwrap();
            // Same bytes, newer index incarnation: the old CAS must still refuse.
            let state = target
                .namespace
                .source(&working.profile.scope.prefix)
                .unwrap()
                .unwrap();
            target
                .namespace
                .select(
                    &git_asset_store::source_claims::admission::Select {
                        prepare: k("competing-prepare"),
                        adopt: k("competing-adopt"),
                        source: working.profile.scope.prefix.clone(),
                        owner: working.profile.scope.owner.clone(),
                        expected_revision: state.revision,
                        selection: state.current.unwrap().selection,
                    },
                    target.nodes,
                    &limits,
                )
                .unwrap();
            let held = worker
                .reconcile_restores(pass!(true, None), &mut f.resources(), || Ok(1000))
                .unwrap();
            assert_eq!(
                held.outcomes[0].hold.as_ref().map(|h| h.code.as_str()),
                Some("stale")
            );
            assert_eq!(held.outcomes[0].index_saved.as_ref(), Some(&saved));
            let state = target
                .namespace
                .source(&working.profile.scope.prefix)
                .unwrap()
                .unwrap();
            let baseline = index_restore::Baseline {
                version: state.current.unwrap(),
                record: baseline.record,
            };
            let proposal = index_restore::prepare_replan(
                &mut target,
                &baseline,
                &context,
                native::Original {
                    eviction: evicted,
                    publication: old,
                },
                &frozen,
                &mut f.resources(),
            )
            .unwrap();
            let mut readonly = scope.clone();
            readonly.allow_store = false;
            let checkpoint = self.pending.checkpoint();
            assert_eq!(
                pending::save_restore_index_successor(
                    &readonly,
                    worker.policy.id().unwrap(),
                    worker.request.commit().as_str(),
                    pending::RestoreIndexSuccessor {
                        expected: &saved,
                        proposal: proposal.clone()
                    },
                    &mut self.pending,
                    worker.metadata,
                    &limits
                )
                .err(),
                Some(Error::Denied)
            );
            assert_eq!(checkpoint, self.pending.checkpoint());
            let competing = proposal;
            let state = target
                .namespace
                .source(&working.profile.scope.prefix)
                .unwrap()
                .unwrap();
            target
                .namespace
                .select(
                    &git_asset_store::source_claims::admission::Select {
                        prepare: k("second-writer-prepare"),
                        adopt: k("second-writer-adopt"),
                        source: working.profile.scope.prefix.clone(),
                        owner: working.profile.scope.owner.clone(),
                        expected_revision: state.revision,
                        selection: state.current.unwrap().selection,
                    },
                    target.nodes,
                    &limits,
                )
                .unwrap();
            let state = target
                .namespace
                .source(&working.profile.scope.prefix)
                .unwrap()
                .unwrap();
            let baseline = index_restore::Baseline {
                version: state.current.unwrap(),
                record: baseline.record,
            };
            let proposal = index_restore::prepare_replan(
                &mut target,
                &baseline,
                &context,
                native::Original {
                    eviction: evicted,
                    publication: old,
                },
                &frozen,
                &mut f.resources(),
            )
            .unwrap();
            Some((
                saved,
                frozen,
                proposal,
                competing,
                f.namespaces[0].checkpoint(),
            ))
        } else {
            None
        };
        let mut next = pass!(true, None);
        next.replan = replan;
        let applied = worker
            .reconcile_restores(next, &mut f.resources(), || Ok(1000))
            .unwrap();
        if let Some((previous, frozen, proposal, competing, native_checkpoint)) = prior {
            assert_eq!(native_checkpoint, f.namespaces[0].checkpoint());
            let successor = applied.outcomes[0].index_saved.as_ref().unwrap();
            assert_ne!(&previous, successor);
            assert_eq!(previous.source, successor.source);
            let loaded = pending::reopen(
                &scope,
                &previous,
                &mut self.pending,
                worker.metadata,
                &limits,
            )
            .unwrap();
            let pending::Operation::RestoreIndex(body) = &loaded.action.operation else {
                panic!("wrong body")
            };
            assert_eq!(body.index.to_json().unwrap(), frozen.to_json().unwrap());
            drop(loaded);
            let mut readonly = scope.clone();
            readonly.allow_store = false;
            assert_eq!(
                &pending::save_restore_index_successor(
                    &readonly,
                    worker.policy.id().unwrap(),
                    worker.request.commit().as_str(),
                    pending::RestoreIndexSuccessor {
                        expected: &previous,
                        proposal: proposal.clone()
                    },
                    &mut self.pending,
                    worker.metadata,
                    &limits
                )
                .unwrap(),
                successor
            );
            assert_eq!(
                pending::save_restore_index_successor(
                    &scope,
                    worker.policy.id().unwrap(),
                    worker.request.commit().as_str(),
                    pending::RestoreIndexSuccessor {
                        expected: successor,
                        proposal
                    },
                    &mut self.pending,
                    worker.metadata,
                    &limits
                )
                .err(),
                Some(Error::Integrity)
            );
            let checkpoint = self.pending.checkpoint();
            assert_eq!(
                pending::save_restore_index_successor(
                    &scope,
                    worker.policy.id().unwrap(),
                    worker.request.commit().as_str(),
                    pending::RestoreIndexSuccessor {
                        expected: &previous,
                        proposal: competing
                    },
                    &mut self.pending,
                    worker.metadata,
                    &limits
                )
                .err(),
                Some(Error::Conflict)
            );
            assert_eq!(checkpoint, self.pending.checkpoint());
            for handle in [&previous, successor] {
                let loaded =
                    pending::reopen(&scope, handle, &mut self.pending, worker.metadata, &limits)
                        .unwrap();
                let pending::Operation::RestoreIndex(body) = &loaded.action.operation else {
                    panic!("wrong body")
                };
                let config = f.root.join("successor-reopen.json");
                fs::write(&config, serde_json::to_vec(&serde_json::json!({
                    "scope":scope,"saved":handle,"namespace":f.root.join("restore-pass-main-pending"),"minimum":self.pending.checkpoint(),
                    "metadata":f.root.join("metadata"),"metadata_minimum":worker.metadata.checkpoint(),"plan":Digest::blob(&body.index.to_json().unwrap()),
                })).unwrap()).unwrap();
                drop(loaded);
                assert!(
                    std::process::Command::new(std::env::current_exe().unwrap())
                        .args([
                            "--exact",
                            "annex::publication::tests::native::pending::reopen_worker"
                        ])
                        .env("GIT_ASSET_PENDING_REOPEN_FIXTURE", config)
                        .status()
                        .unwrap()
                        .success()
                );
            }
        }

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
                    replan,
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
