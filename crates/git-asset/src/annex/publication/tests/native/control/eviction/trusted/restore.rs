//! Signed saved execution over the ordinary/bare owned lifecycle fixture.
use super::*;
use crate::annex::producer_index::{
    Record,
    update::{self as roots, restore as index},
};
use crate::annex::{pending, publication::restore as native};
impl State {
    pub(in super::super) fn saved_native(
        &mut self,
        f: &mut Fixture,
        scope: &pending::Scope,
        saved: &pending::Saved,
        records: &mut Namespace,
        context: &mut native::Context<'_>,
    ) -> crate::annex::publication::Ready {
        self.policy.actions.as_mut().unwrap().pending = Some(scope.clone());
        self.policy.allow_copy = true;
        self.signer.trust.placement_enrollment = self.policy.id().unwrap();
        f.metadata = self.metadata.finished_reader().unwrap();
        let native = source::annex::Source {
            repository: &self.repository,
        };
        let mut observer =
            Namespace::open(&f.root.join("namespace-1"), &f.namespaces[1].checkpoint()).unwrap();
        let limits = Fixture::limits();
        macro_rules! execution {
            ($request:expr) => {
                Execution {
                    trust: &self.signer.trust,
                    request: $request,
                    policy: &self.policy,
                    enrollment: &self.enrollment,
                    repository: &native,
                    namespace: &mut observer,
                    metadata: &mut self.metadata,
                    limits: &limits,
                }
            };
        }
        self.policy.allow_copy = false;
        self.signer.trust.placement_enrollment = self.policy.id().unwrap();
        let denied = self.signer.verify(&self.signer.claims, 1000).unwrap();
        let checkpoint = f.namespaces[0].checkpoint();
        assert!(matches!(
            execution!(&denied)
                .restore_saved(saved, records, context, &mut f.resources(), || Ok(1000)),
            Err(crate::github::native::Error::Admission(
                github::Error::Disabled
            ))
        ));
        assert_eq!(checkpoint, f.namespaces[0].checkpoint());
        self.policy.allow_copy = true;
        self.signer.trust.placement_enrollment = self.policy.id().unwrap();
        let request = self.signer.verify(&self.signer.claims, 1000).unwrap();
        let mut foreign = context.profile.clone();
        foreign.budget.transfer_bytes += 1;
        let mut wrong = native::Context {
            profile: &foreign,
            control: context.control,
            retirement: context.retirement,
            namespace: &mut *context.namespace,
            metadata: &mut *context.metadata,
            limits: context.limits,
        };
        assert!(matches!(
            execution!(&request).restore_saved(
                saved,
                records,
                &mut wrong,
                &mut f.resources(),
                || Ok(1000)
            ),
            Err(crate::github::native::Error::Admission(
                github::Error::Trust
            ))
        ));
        assert_eq!(checkpoint, f.namespaces[0].checkpoint());
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
        let mut calls = 0;
        let failure = worker
            .restore_saved(saved, records, context, &mut f.resources(), || {
                calls += 1;
                Ok(if calls >= 4 { 2000 } else { 1000 })
            })
            .err();
        assert!(
            matches!(
                failure.as_ref(),
                Some(crate::github::native::Error::Admission(
                    github::Error::Expired
                ))
            ),
            "{failure:?}; clock calls={calls}"
        );
        let mut renewed = self.signer.claims.clone();
        renewed["iat"] = serde_json::json!(2000);
        renewed["nbf"] = serde_json::json!(2000);
        renewed["exp"] = serde_json::json!(2300);
        renewed["jti"] = serde_json::json!("renewed-native-restore");
        self.signer.keys.valid_from = 1900;
        self.signer.keys.valid_until = 2500;
        let request = self.signer.verify(&renewed, 2000).unwrap();
        let mut worker = execution!(&request);
        let result = worker
            .restore_saved(saved, records, context, &mut f.resources(), || Ok(2000))
            .unwrap();
        assert!(result.report.post_effect_hold.is_none());
        assert_eq!(result.report.result.plan, result.ready.receipt().plan);
        result.ready
    }
    pub(in super::super) fn saved_index(
        &mut self,
        f: &mut Fixture,
        keeper: &mut Fixture,
        slot: (&pending::Scope, &pending::Saved, &mut Namespace),
        context: &native::Context<'_>,
        target: &mut index::Target<'_>,
    ) -> roots::Report {
        use crate::github::native::producers::{Head, Heads};
        let (scope, saved, records) = slot;
        let source = if let Some(heads) = &self.policy.producer_heads {
            heads.source.clone()
        } else {
            let mut nodes =
                MetadataStore::open(&keeper.root.join("metadata"), keeper.metadata.checkpoint())
                    .unwrap();
            let row = Record::new(keeper.plan.clone(), None).unwrap();
            let (path, object) = row.store(&mut nodes).unwrap();
            let (tree, _) = git_asset_core::tree::Tree::build(
                &BTreeMap::from([(path, object)]),
                &mut nodes,
                TreeLimits::default(),
            )
            .unwrap();
            nodes.finish().unwrap();
            keeper.metadata = nodes.finished_reader().unwrap();
            super::super::super::super::github_worker::heads::register(
                keeper,
                tree.root().clone(),
                "worker-retained-index",
            )
            .head
        };
        self.policy.producer_heads = Some(Heads {
            source,
            destination: Head {
                root: f.root.join("restore-index"),
                minimum: target.namespace.checkpoint(),
                profile: target.profile.clone(),
            },
        });
        self.policy.actions.as_mut().unwrap().pending = Some(scope.clone());
        self.policy.allow_copy = true;
        self.signer.trust.placement_enrollment = self.policy.id().unwrap();
        f.metadata = self.metadata.finished_reader().unwrap();
        let native = source::annex::Source {
            repository: &self.repository,
        };
        let base = if target.profile.scope.allow_store {
            1000
        } else {
            2000
        };
        if base == 2000 {
            self.signer.keys.valid_from = 1900;
            self.signer.keys.valid_until = 2500;
            self.signer.claims["iat"] = serde_json::json!(2000);
            self.signer.claims["nbf"] = serde_json::json!(2000);
            self.signer.claims["exp"] = serde_json::json!(2300);
            self.signer.claims["jti"] = serde_json::json!("renewed-index-recovery");
        }
        let request = self.signer.verify(&self.signer.claims, base).unwrap();
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
        let mut calls = 0;
        let result = worker
            .register_restore_saved(saved, records, context, target, &mut f.resources(), || {
                calls += 1;
                Ok(if calls >= 4 { base + 1000 } else { base })
            })
            .unwrap();
        if target.profile.scope.allow_store {
            assert!(matches!(
                result.post_effect_hold,
                Some(github::Error::Expired)
            ));
        } else {
            assert!(result.post_effect_hold.is_none());
        }
        result.result
    }
}
