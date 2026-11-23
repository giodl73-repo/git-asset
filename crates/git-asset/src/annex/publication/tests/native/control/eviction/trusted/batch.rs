//! Receiver's real native fresh plan/apply and original-slot replay, owned fixtures only.
use super::*;
use crate::annex::producer_index::{Index, Record};
use crate::github::native::eviction::Pass;

fn root(record: &Record, metadata: &mut MetadataStore) -> Object {
    let (path, object) = record.store(metadata).unwrap();
    let (tree, _) = Tree::build(
        &BTreeMap::from([(path, object)]),
        metadata,
        TreeLimits::default(),
    )
    .unwrap();
    metadata.finish().unwrap();
    tree.root().clone()
}
impl State {
    pub(in super::super) fn execute_batch(
        &mut self,
        f: &mut Fixture,
        keeper: &mut Fixture,
        plan: &NativePlan,
        uncertain: bool,
        named_heads: bool,
    ) {
        let mut working = Record::new(f.plan.clone(), None).unwrap();
        if named_heads {
            // Keep an earlier prospective context while naming the actual completion.
            working.producer = f
                .plan
                .clone()
                .with_context(Some(Digest::blob(b"earlier prospective storage recipe")));
            working = working
                .with_completed_publication(f.plan.clone(), &f.witness, &mut f.ledger)
                .unwrap();
        }
        let destination = root(&working, &mut self.metadata);
        f.metadata =
            MetadataReader::open(&f.root.join("metadata"), &self.metadata.checkpoint()).unwrap();
        let mut kept = Record::new(keeper.plan.clone(), None).unwrap();
        let mut metadata =
            MetadataStore::open(&keeper.root.join("metadata"), keeper.metadata.checkpoint())
                .unwrap();
        let missing = root(&kept, &mut metadata);
        kept.control_retention = Some(plan.metadata_plan().request().keepers[0].grant.clone());
        let source = root(&kept, &mut metadata);
        keeper.metadata =
            MetadataReader::open(&keeper.root.join("metadata"), &metadata.checkpoint()).unwrap();
        let (read, _) = Index::from_root(source.clone())
            .get(&kept.object, &mut keeper.metadata, TreeLimits::default())
            .unwrap();
        assert_eq!(read, Some(kept.clone()));
        let mut foreign = kept.clone();
        foreign.object = Object::from_bytes(b"another media object");
        assert_eq!(foreign.store(&mut metadata).unwrap_err(), Error::Integrity);
        self.policy.producers = Some(github::native::ProducerRoots {
            source,
            destination,
        });
        self.signer.trust.placement_enrollment = self.policy.id().unwrap();
        let named = named_heads.then(|| {
            let retained = super::super::super::super::github_worker::heads::register(
                keeper,
                self.policy.producers.as_ref().unwrap().source.clone(),
                "retained-producers",
            );
            let working = super::super::super::super::github_worker::heads::register(
                f,
                self.policy.producers.as_ref().unwrap().destination.clone(),
                "working-producers",
            );
            self.metadata =
                MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
            let mut policy = self.policy.clone();
            policy.producers = None;
            policy.producer_heads = Some(github::native::producers::Heads {
                source: retained.head,
                destination: working.head,
            });
            let mut trust = self.signer.trust.clone();
            trust.placement_enrollment = policy.id().unwrap();
            (policy, trust)
        });
        let named_request = named.as_ref().map(|(_, trust)| {
            github::verify(
                trust,
                &self.signer.keys,
                &self.signer.token(&self.signer.claims),
                &self.signer.commit,
                1000,
            )
            .unwrap()
        });
        let native = source::annex::Source {
            repository: &self.repository,
        };
        let request = self.signer.verify(&self.signer.claims, 1000).unwrap();
        let limits = Fixture::limits();
        let budget = github::placement::Budget {
            pages: 1,
            objects: 1,
            stream_bytes: 1024 * 1024,
            millis: 60000,
        };
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
        let object = f.plan.candidate().object.clone();
        let payload = crate::annex::Reader::new(f.profile.pool.pool.clone())
            .unwrap()
            .open(&object)
            .unwrap()
            .with_path(|p| p.to_path_buf())
            .unwrap();
        let unchanged = self.pending.checkpoint();
        if uncertain {
            uncertain_original(
                &mut worker,
                f,
                keeper,
                plan,
                &mut self.pending,
                &budget,
                &payload,
            );
            return;
        }
        let preview = worker
            .reconcile_evictions(
                Pass {
                    cursor: None,
                    budget: &budget,
                    pending: &mut self.pending,
                    apply: false,
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert_eq!(
            preview.outcomes[0].state,
            "planned",
            "{}",
            serde_json::to_string(&preview).unwrap()
        );
        assert!(preview.outcomes[0].eviction.is_none());
        assert_eq!(self.pending.checkpoint(), unchanged);
        assert!(payload.is_file());
        let tiny = github::placement::Budget {
            stream_bytes: 1,
            ..budget.clone()
        };
        let held = worker
            .reconcile_evictions(
                Pass {
                    cursor: None,
                    budget: &tiny,
                    pending: &mut self.pending,
                    apply: true,
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert_eq!(
            held.outcomes[0].hold.as_ref().unwrap().code,
            "admission_denied"
        );
        assert_eq!(self.pending.checkpoint(), unchanged);
        let mut missing_policy = self.policy.clone();
        missing_policy.producers.as_mut().unwrap().source = missing;
        let mut missing_trust = self.signer.trust.clone();
        missing_trust.placement_enrollment = missing_policy.id().unwrap();
        let missing_request = github::verify(
            &missing_trust,
            &self.signer.keys,
            &self.signer.token(&self.signer.claims),
            &self.signer.commit,
            1000,
        )
        .unwrap();
        worker.policy = &missing_policy;
        worker.trust = &missing_trust;
        worker.request = &missing_request;
        let held = worker
            .reconcile_evictions(
                Pass {
                    cursor: None,
                    budget: &budget,
                    pending: &mut self.pending,
                    apply: true,
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert_eq!(held.outcomes[0].hold.as_ref().unwrap().code, "missing");
        assert!(payload.is_file());
        assert_eq!(self.pending.checkpoint(), unchanged);
        worker.policy = &self.policy;
        worker.trust = &self.signer.trust;
        worker.request = &request;
        if let Some((policy, trust)) = &named {
            worker.policy = policy;
            worker.trust = trust;
            worker.request = named_request.as_ref().unwrap();
        }
        // A job added after preview still prevents new planning/removal.
        let working_plan = f.plan.clone();
        demand(f, &working_plan);
        let protected = worker
            .reconcile_evictions(
                Pass {
                    cursor: None,
                    budget: &budget,
                    pending: &mut self.pending,
                    apply: true,
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert_eq!(
            protected.outcomes[0].state,
            "protected",
            "{}",
            serde_json::to_string(&protected).unwrap()
        );
        assert!(protected.outcomes[0].eviction.is_none());
        assert!(payload.is_file());
        let job = f.namespaces[0].source(&k("recovery-job")).unwrap().unwrap();
        let version = job.current.unwrap();
        f.namespaces[0]
            .apply(
                k("batch-withdraw-job"),
                Command::Withdraw {
                    source: k("recovery-job"),
                    owner: k("owner"),
                    expected_revision: job.revision,
                },
                &f.metadata,
                &limits,
            )
            .unwrap();
        f.namespaces[0]
            .apply(
                k("batch-retire-job"),
                Command::Retire {
                    source: k("recovery-job"),
                    owner: k("owner"),
                    expected_revision: job.revision + 1,
                    predecessor_revision: version.revision,
                },
                &f.metadata,
                &limits,
            )
            .unwrap();
        let applied = worker
            .reconcile_evictions(
                Pass {
                    cursor: None,
                    budget: &budget,
                    pending: &mut self.pending,
                    apply: true,
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(if payload.is_file() { 1000 } else { 1300 }),
            )
            .unwrap();
        assert_eq!(
            applied.outcomes[0].state,
            "evicted",
            "{}",
            serde_json::to_string(&applied).unwrap()
        );
        assert_eq!(applied.post_effect_hold, Some(github::Error::Expired));
        assert_eq!(applied.producer_heads.is_some(), named_heads);
        assert!(!payload.exists());
        let saved = &applied.outcomes[0].eviction.as_ref().unwrap().saved;
        assert!(
            !applied.outcomes[0]
                .eviction
                .as_ref()
                .unwrap()
                .mutation
                .as_ref()
                .unwrap()
                .result
                .historical
        );
        // New authentication can reconcile the original without source bytes or keeper payload config.
        let fresh = github::verify(
            worker.trust,
            &self.signer.keys,
            &self.signer.token(&self.signer.claims),
            &self.signer.commit,
            1000,
        )
        .unwrap();
        worker.request = &fresh;
        let config = keeper
            .profile
            .pool
            .pool
            .repository
            .join(if keeper.profile.pool.pool.bare {
                "config"
            } else {
                ".git/config"
            });
        let offline = config.with_extension("offline");
        fs::rename(&config, &offline).unwrap();
        let repeated = worker
            .reconcile_evictions(
                Pass {
                    cursor: None,
                    budget: &budget,
                    pending: &mut self.pending,
                    apply: true,
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        fs::rename(offline, config).unwrap();
        assert_eq!(
            repeated.outcomes[0].state,
            "reconciled",
            "{}",
            serde_json::to_string(&repeated).unwrap()
        );
        let durable = repeated.outcomes[0].eviction.as_ref().unwrap();
        assert_eq!(&durable.saved, saved);
        assert!(durable.mutation.as_ref().unwrap().result.historical);
        let cursor = git_asset_store::placement::Cursor {
            basis: repeated.basis.clone(),
            after: object.sha256.clone(),
        };
        let suffix = worker
            .reconcile_evictions(
                Pass {
                    cursor: Some(&cursor),
                    budget: &budget,
                    pending: &mut self.pending,
                    apply: true,
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert!(suffix.outcomes.is_empty());
        assert_eq!(suffix.storage_converged, None);
        let stale = git_asset_store::placement::Cursor {
            basis: Digest::blob(b"other event"),
            ..cursor
        };
        assert!(matches!(
            worker.reconcile_evictions(
                Pass {
                    cursor: Some(&stale),
                    budget: &budget,
                    pending: &mut self.pending,
                    apply: true
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(1000)
            ),
            Err(github::native::Error::Admission(github::Error::Stale))
        ));
        let mut actual = Vec::new();
        crate::annex::Reader::new(keeper.profile.pool.pool.clone())
            .unwrap()
            .open(&object)
            .unwrap()
            .read_to_end(&mut actual)
            .unwrap();
        assert_eq!(Object::from_bytes(&actual), object);
    }
}

fn uncertain_original(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    f: &mut Fixture,
    keeper: &mut Fixture,
    plan: &NativePlan,
    namespace: &mut Namespace,
    budget: &github::placement::Budget,
    payload: &Path,
) {
    use crate::annex::pending::{self, Action, Operation};
    let limits = Fixture::limits();
    let scope = worker
        .policy
        .actions
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap();
    let saved = pending::save(
        scope,
        worker.policy.id().unwrap(),
        worker.request.commit().as_str(),
        &Action {
            operation: Operation::Eviction(Box::new(plan.clone())),
            working: f.plan.clone(),
            retained: keeper.plan.clone(),
        },
        namespace,
        worker.metadata,
        &limits,
    )
    .unwrap();
    let policy = worker.policy;
    let profile = policy.actions.as_ref().unwrap().eviction.as_ref().unwrap();
    let controller = Controller::new(profile);
    let failed = controller
        .apply_hook(
            plan,
            Sources {
                source: Producer {
                    profile: &f.profile,
                    plan: &f.plan,
                    witness: &f.witness,
                    ledger: &mut f.ledger,
                },
                keeper: Producer {
                    profile: &keeper.profile,
                    plan: &keeper.plan,
                    witness: &keeper.witness,
                    ledger: &mut keeper.ledger,
                },
            },
            Nodes {
                source: &f.metadata,
                keeper: &mut keeper.metadata,
                limits: &limits,
                tree_limits: TreeLimits::default(),
            },
            Cut::AfterRemoval,
        )
        .unwrap_err();
    assert_eq!(failed.code(), "uncertain");
    assert!(!payload.exists());
    let effect = &profile.runtime.as_ref().unwrap().effect;
    let before = CapacityLedger::open(&effect.ledger, &effect.minimum)
        .unwrap()
        .checkpoint();
    for _ in 0..2 {
        let report = worker
            .reconcile_evictions(
                Pass {
                    cursor: None,
                    budget,
                    pending: namespace,
                    apply: true,
                },
                &mut keeper.resources(),
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        let outcome = &report.outcomes[0];
        assert_eq!(outcome.hold.as_ref().unwrap().code, "uncertain");
        let original = outcome.eviction.as_ref().unwrap();
        assert_eq!(original.saved, saved);
        assert!(original.mutation.is_none());
    }
    assert_eq!(
        CapacityLedger::open(&effect.ledger, &effect.minimum)
            .unwrap()
            .checkpoint(),
        before
    );
    assert_eq!(
        controller.status(plan, &f.profile, &f.plan).unwrap().effect,
        "uncertain"
    );
    let mut bytes = Vec::new();
    crate::annex::Reader::new(keeper.profile.pool.pool.clone())
        .unwrap()
        .open(plan.object())
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(Object::from_bytes(&bytes), *plan.object());
}
