//! Save/read back original action before invoking existing authenticated execution.
use super::*;
use crate::annex::{
    pending::{self, Action, Operation, Phase, Saved},
    snapshot::Snapshot,
};
use git_asset_core::tree::NodeStore;
use retirement::planning::{self, Nodes, Producer, Sources};

#[derive(Serialize)]
pub struct Durable<T> {
    pub saved: Saved,
    pub mutation: Option<Mutation<T>>,
    pub failure: Option<batch::Failure>,
}
pub struct HandoffRequest<'a> {
    pub working: &'a publication::Plan,
    pub retained: &'a publication::Plan,
    pub snapshot: Snapshot,
}
fn report<T>(saved: Saved, result: Result<Mutation<T>, Error>) -> Durable<T> {
    match result {
        Ok(mutation) => Durable {
            saved,
            mutation: Some(mutation),
            failure: None,
        },
        Err(error) => Durable {
            saved,
            mutation: None,
            failure: Some(batch::Failure::from(&error)),
        },
    }
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    pub(super) fn pending_scope(&self) -> Result<pending::Scope, Error> {
        Ok(self
            .policy
            .actions
            .as_ref()
            .and_then(|a| a.pending.clone())
            .ok_or(publication::Error::Unsupported)?)
    }
    pub fn handoff_durable(
        &mut self,
        request: HandoffRequest<'_>,
        namespace: &mut Namespace,
        working: &mut publication::Resources<'_>,
        retained: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Durable<publication::handoff::Report>, Error> {
        let HandoffRequest {
            working: working_plan,
            retained: retained_plan,
            snapshot,
        } = request;
        let allowed = self.policy.actions.as_ref().is_some_and(|a| a.handoff);
        let ids = self.admission(allowed, &mut now)?;
        let scope = self.pending_scope()?;
        let old = pending::find(
            &scope,
            Phase::Handoff,
            &working_plan.candidate().object,
            &working_plan.id()?,
            namespace,
            self.metadata,
            self.limits,
        )?;
        let saved = match old {
            Some(old) => old.saved,
            None => {
                let plan = publication::handoff::Plan::new(
                    working_plan,
                    retained_plan,
                    snapshot,
                    working.namespace,
                )?;
                let action = Action {
                    operation: Operation::Handoff(Box::new(plan)),
                    working: working_plan.clone(),
                    retained: retained_plan.clone(),
                };
                pending::save(
                    &scope,
                    ids.0,
                    self.request.commit().as_str(),
                    &action,
                    namespace,
                    self.metadata,
                    self.limits,
                )?
            }
        };
        let loaded = pending::reopen(&scope, &saved, namespace, self.metadata, self.limits)?;
        let result = match &loaded.action.operation {
            Operation::Handoff(plan) => self.handoff(
                plan,
                &loaded.action.working,
                &loaded.action.retained,
                working,
                retained,
                &mut now,
            ),
            _ => Err(publication::Error::Integrity.into()),
        };
        Ok(report(saved, result))
    }
    pub fn evict_durable<N: NodeStore, K: NodeStore>(
        &mut self,
        plan: &planning::Plan,
        namespace: &mut Namespace,
        sources: Sources<'_>,
        nodes: Nodes<'_, N, K>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Durable<planning::Report>, Error> {
        let allowed = self
            .policy
            .actions
            .as_ref()
            .and_then(|a| a.eviction.as_ref())
            .is_some_and(|p| p.allow_evict);
        let ids = self.admission(allowed, &mut now)?;
        let scope = self.pending_scope()?;
        let action = Action {
            operation: Operation::Eviction(Box::new(plan.clone())),
            working: sources.source.plan.clone(),
            retained: sources.keeper.plan.clone(),
        };
        let saved = pending::save(
            &scope,
            ids.0,
            self.request.commit().as_str(),
            &action,
            namespace,
            self.metadata,
            self.limits,
        )?;
        let loaded = pending::reopen(&scope, &saved, namespace, self.metadata, self.limits)?;
        let result = match &loaded.action.operation {
            Operation::Eviction(plan) => self.evict_original(
                plan,
                Sources {
                    source: Producer {
                        plan: &loaded.action.working,
                        ..sources.source
                    },
                    keeper: Producer {
                        plan: &loaded.action.retained,
                        ..sources.keeper
                    },
                },
                nodes,
                &mut now,
            ),
            _ => Err(publication::Error::Integrity.into()),
        };
        Ok(report(saved, result))
    }
}
