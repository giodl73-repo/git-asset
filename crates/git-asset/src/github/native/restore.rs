//! Authenticated original saved execution; no plan replacement or receiver scheduler.
use super::*;
use crate::annex::{
    pending::{self, Operation, Saved},
    producer_index::update::restore as index,
    publication::restore as native,
};
use git_asset_core::{Object, identity};

pub struct Restored {
    pub report: Mutation<publication::Receipt>,
    pub ready: publication::Ready,
}
fn same(a: &impl Serialize, b: &impl Serialize) -> Result<(), Error> {
    let a = identity::canonical_json(
        &serde_json::to_value(a).map_err(|_| publication::Error::Invalid)?,
    )
    .map_err(|_| publication::Error::Invalid)?;
    let b = identity::canonical_json(
        &serde_json::to_value(b).map_err(|_| publication::Error::Invalid)?,
    )
    .map_err(|_| publication::Error::Invalid)?;
    if a.len() > 65536 || b.len() > 65536 {
        return Err(publication::Error::Budget.into());
    }
    if a != b {
        return Err(Error::Admission(AdmissionError::Trust));
    }
    Ok(())
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    fn restore_allowed(&self) -> bool {
        self.policy.allow_copy
            && self
                .policy
                .destination_control
                .as_ref()
                .is_some_and(|p| p.allow_reacquire)
    }
    fn restore_binding(
        &self,
        c: &native::Context<'_>,
        r: &publication::Resources<'_>,
    ) -> Result<(), Error> {
        same(c.profile, &self.policy.copy.destination)?;
        same(
            c.control,
            self.policy
                .destination_control
                .as_ref()
                .ok_or(publication::Error::Unsupported)?,
        )?;
        same(
            c.retirement,
            self.policy
                .actions
                .as_ref()
                .and_then(|a| a.eviction.as_ref())
                .ok_or(publication::Error::Unsupported)?,
        )?;
        same(c.limits, self.limits)?;
        same(&r.limits, self.limits)?;
        if c.namespace.identity() != r.namespace.identity()
            || c.metadata.checkpoint().identity() != r.metadata.checkpoint().identity()
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        Ok(())
    }
    fn restore_demand(&self, state: &mut Selected, object: &Object) -> Result<(), Error> {
        let nodes = self
            .metadata
            .finished_reader()
            .map_err(|_| publication::Error::NeedsRecovery)?;
        let demand = state
            .namespace
            .requirements(&nodes, self.limits)
            .map_err(Error::Requirements)?;
        if !demand
            .entries
            .iter()
            .any(|a| &a.object == object && a.places.contains(&self.policy.required_place))
        {
            return Err(Error::Admission(AdmissionError::Denied));
        }
        Ok(())
    }
    /// The caller already owns this durable handle; errors never replace its plan.
    pub fn restore_saved(
        &mut self,
        saved: &Saved,
        records: &mut Namespace,
        context: &mut native::Context<'_>,
        destination: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Restored, Error> {
        let ids = self.admission(self.restore_allowed(), &mut now)?;
        self.restore_binding(context, destination)?;
        let scope = self.pending_scope()?;
        let loaded = pending::reopen(&scope, saved, records, self.metadata, self.limits)?;
        let Operation::Restore(body) = &loaded.action.operation else {
            return Err(publication::Error::Integrity.into());
        };
        replication::Plan::new(
            &self.policy.copy,
            loaded.action.retained.clone(),
            loaded.action.working.clone(),
        )?;
        let mut state = self.select(ids)?;
        self.restore_demand(&mut state, loaded.action.object())?;
        self.revalidate(&state, &mut now)?;
        *destination.metadata = context
            .metadata
            .finished_reader()
            .map_err(|_| publication::Error::NeedsRecovery)?;
        let progress = body
            .restore
            .publication()
            .progress(destination.witness, destination.ledger)?;
        if progress.completion && progress.settled {
            let ready = native::reopen(
                context,
                native::Original {
                    eviction: &body.eviction,
                    publication: &loaded.action.working,
                },
                &body.restore,
                destination,
            )?;
            let report = self.mutation(state, ready.receipt().clone(), &mut now);
            return Ok(Restored { report, ready });
        }
        let mut source = managed::Reader::new(self.policy.copy.source.pool.clone())
            .map_err(publication::Error::from)?;
        let mut hold = None;
        let result = native::apply(
            context,
            native::Original {
                eviction: &body.eviction,
                publication: &loaded.action.working,
            },
            &body.restore,
            destination,
            &mut source,
            |_| {
                self.revalidate(&state, &mut now).map_err(|e| {
                    hold = Some(e);
                    publication::Error::Denied
                })
            },
        );
        let ready = result.map_err(|e| hold.unwrap_or(Error::Native(e)))?;
        let report = self.mutation(state, ready.receipt().clone(), &mut now);
        Ok(Restored { report, ready })
    }
    /// Completed original index receipt precedes fresh media access or write permission.
    pub fn register_restore_saved(
        &mut self,
        saved: &Saved,
        records: &mut Namespace,
        context: &native::Context<'_>,
        target: &mut index::Target<'_>,
        destination: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Mutation<crate::annex::producer_index::update::Report>, Error> {
        let ids = self.admission(self.restore_allowed(), &mut now)?;
        self.restore_binding(context, destination)?;
        let head = &self
            .policy
            .producer_heads
            .as_ref()
            .ok_or(publication::Error::Unsupported)?
            .destination;
        same(target.profile, &head.profile)?;
        if target.namespace.identity() != head.minimum.identity()
            || target.nodes.checkpoint().identity() != &head.profile.scope.metadata
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        let loaded = pending::reopen(
            &self.pending_scope()?,
            saved,
            records,
            self.metadata,
            self.limits,
        )?;
        let Operation::RestoreIndex(body) = &loaded.action.operation else {
            return Err(publication::Error::Integrity.into());
        };
        replication::Plan::new(
            &self.policy.copy,
            loaded.action.retained.clone(),
            loaded.action.working.clone(),
        )?;
        let mut state = self.select(ids)?;
        if let Some(receipt) = index::status(target, &body.index)? {
            return Ok(self.mutation(state, receipt, &mut now));
        }
        if !head.profile.scope.allow_store {
            return Err(Error::Admission(AdmissionError::Disabled));
        }
        self.restore_demand(&mut state, loaded.action.object())?;
        self.revalidate(&state, &mut now)?;
        *destination.metadata = context
            .metadata
            .finished_reader()
            .map_err(|_| publication::Error::NeedsRecovery)?;
        let mut hold = None;
        let result = index::apply(
            target,
            context,
            native::Original {
                eviction: &body.original.eviction,
                publication: &loaded.action.working,
            },
            &body.index,
            destination,
            || {
                self.revalidate(&state, &mut now).map_err(|e| {
                    hold = Some(e);
                    publication::Error::Denied
                })
            },
        );
        let receipt = result.map_err(|e| hold.unwrap_or(Error::Native(e)))?;
        Ok(self.mutation(state, receipt, &mut now))
    }
}
