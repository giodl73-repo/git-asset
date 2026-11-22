//! Real Git, signed provenance and the existing synthetic eviction/restore matrix.
use super::super::super::github_worker::git;
use super::*;
use crate::github::native::{Actions, Execution, Policy};
use crate::{github, lifecycle, source};
mod batch;
pub(super) struct State {
    repository: source::Repository,
    enrollment: lifecycle::Enrollment,
    policy: Policy,
    signer: github::fixture::Fixture,
    metadata: MetadataStore,
    pending: Namespace,
}
impl State {
    pub(super) fn new(f: &mut Fixture, keeper: &Fixture, owner: &OwnerProfile) -> Self {
        let project = f.root.join("trusted-project");
        fs::create_dir_all(project.join("assets")).unwrap();
        git(&project, &["init", "-b", "main"]);
        git(&project, &["config", "user.name", "Synthetic"]);
        git(
            &project,
            &["config", "user.email", "fixture@example.invalid"],
        );
        let next =
            Object::from_bytes(b"a newly selected asset, no claim of whole-project convergence");
        fs::write(
            project.join("assets/next"),
            format!(
                "/annex/objects/SHA256-s{}--{}\n",
                next.bytes,
                next.sha256.as_str()
            ),
        )
        .unwrap();
        git(&project, &["add", "assets"]);
        git(&project, &["commit", "-m", "new selected native asset"]);
        let commit = source::GitId::parse(
            String::from_utf8(git(&project, &["rev-parse", "HEAD"]))
                .unwrap()
                .trim(),
        )
        .unwrap();
        let repository = source::Repository::new(
            f.profile.pool.pool.git.clone(),
            project,
            source::Limits::default(),
        )
        .unwrap();
        f.namespaces[1]
            .apply(
                k("enroll-trusted-main"),
                Command::Enroll {
                    source: k("main"),
                    owner: k("owner"),
                    class: Kind::Branch,
                },
                &f.metadata,
                &Fixture::limits(),
            )
            .unwrap();
        let enrollment = lifecycle::Enrollment {
            repository: Digest::blob(b"trusted synthetic project"),
            authority: Digest::blob(b"trusted runner"),
            namespace: f.namespaces[1].identity().clone(),
            policy: Digest::blob(
                &identity::canonical_json(&serde_json::to_value(f.namespaces[1].policy()).unwrap())
                    .unwrap(),
            ),
            source: k("main"),
            owner: k("owner"),
            class: Kind::Branch,
            ref_name: "refs/heads/main".into(),
            scope: vec![LogicalPath::parse("assets").unwrap()],
            budget: lifecycle::Budget {
                entries: 128,
                nodes: 1024,
                node_bytes: 4 * 1024 * 1024,
                millis: 60000,
            },
        };
        let mut metadata =
            MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
        let native = source::annex::Source {
            repository: &repository,
        };
        let mut selected = lifecycle::Execution {
            enrollment: &enrollment,
            repository: &native,
            namespace: &mut f.namespaces[1],
            metadata: &mut metadata,
            allow_selection: true,
        };
        let plan = selected.plan(Some(&commit)).unwrap();
        selected.apply(&plan).unwrap();
        let pending = super::super::super::github_worker::batch::pending_namespace(f);
        let pending_scope = super::super::super::github_worker::batch::pending_scope(f, &pending);
        let policy = Policy {
            copy: crate::annex::replication::Profile {
                source: keeper.profile.clone(),
                destination: f.profile.clone(),
                source_domain: owner.keeper_domain.clone(),
                destination_domain: owner.source_domain.clone(),
            },
            source_control: keeper.control_profile.clone(),
            destination_control: f.control_profile.clone(),
            namespace: enrollment.namespace.clone(),
            required_place: k("pool"),
            allow_selection: true,
            allow_copy: false,
            producer_heads: None,
            producers: None,
            actions: Some(Actions {
                handoff: false,
                eviction: Some(owner.clone()),
                backup: false,
                history: None,
                pending: Some(pending_scope),
            }),
        };
        let mut signer = github::fixture::Fixture::new();
        signer.commit = commit.clone();
        signer.claims["sha"] = serde_json::json!(commit.as_str());
        signer.trust.selection_enrollment = enrollment.id().unwrap();
        signer.trust.placement_enrollment = policy.id().unwrap();
        Self {
            repository,
            enrollment,
            policy,
            signer,
            metadata,
            pending,
        }
    }
    pub(super) fn execute(
        &mut self,
        f: &mut Fixture,
        keeper: &mut Fixture,
        plan: &NativePlan,
    ) -> crate::annex::retirement::planning::Report {
        let native = source::annex::Source {
            repository: &self.repository,
        };
        let request = self.signer.verify(&self.signer.claims, 1000).unwrap();
        let mut observer =
            Namespace::open(&f.root.join("namespace-1"), &f.namespaces[1].checkpoint()).unwrap();
        let limits = Fixture::limits();
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
        macro_rules! producers {
            () => {
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
                }
            };
        }
        macro_rules! readers {
            () => {
                Nodes {
                    source: &f.metadata,
                    keeper: &mut keeper.metadata,
                    limits: &limits,
                    tree_limits: TreeLimits::default(),
                }
            };
        }
        let mut copy_only = self.policy.clone();
        copy_only.actions = None;
        let mut copy_trust = self.signer.trust.clone();
        copy_trust.placement_enrollment = copy_only.id().unwrap();
        let copy_request = github::verify(
            &copy_trust,
            &self.signer.keys,
            &self.signer.token(&self.signer.claims),
            &self.signer.commit,
            1000,
        )
        .unwrap();
        worker.policy = &copy_only;
        worker.trust = &copy_trust;
        worker.request = &copy_request;
        assert!(matches!(
            worker.evict(plan, producers!(), readers!(), || Ok(1000)),
            Err(github::native::Error::Admission(github::Error::Disabled))
        ));
        worker.policy = &self.policy;
        worker.trust = &self.signer.trust;
        worker.request = &request;
        let mut calls = 0;
        assert!(matches!(
            worker.evict(plan, producers!(), readers!(), || {
                calls += 1;
                Ok(if calls == 3 { 1300 } else { 1000 })
            }),
            Err(github::native::Error::Admission(github::Error::Expired))
        ));
        assert_eq!(calls, 3);
        let status = Controller::new(
            self.policy
                .actions
                .as_ref()
                .unwrap()
                .eviction
                .as_ref()
                .unwrap(),
        )
        .status(plan, &f.profile, &f.plan)
        .unwrap();
        assert_eq!(status.effect, "not_started");
        let fresh = self.signer.verify(&self.signer.claims, 1000).unwrap();
        worker.request = &fresh;
        let report = worker
            .evict_durable(plan, &mut self.pending, producers!(), readers!(), || {
                Ok(1000)
            })
            .unwrap();
        assert!(report.failure.is_none(), "{:?}", report.failure);
        let saved = report.saved.clone();
        let report = report.mutation.unwrap();
        assert!(report.post_effect_hold.is_none());
        assert!(report.selection.desired_current);
        assert_eq!(report.storage_converged, None);
        let scope = self
            .policy
            .actions
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap();
        let loaded = crate::annex::pending::reopen(
            scope,
            &saved,
            &mut self.pending,
            worker.metadata,
            &limits,
        )
        .unwrap();
        match &loaded.action.operation {
            crate::annex::pending::Operation::Eviction(p) => {
                assert_eq!(p.id().unwrap(), plan.id().unwrap())
            }
            _ => panic!("wrong saved action"),
        };
        report.result
    }
}
