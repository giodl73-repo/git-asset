//! Fixed-policy mutation actions, not event-supplied paths or authority.
use super::*;
use git_asset_core::tree::NodeStore;
use retirement::planning::{self, Controller, Nodes, Producer, Sources};
#[derive(Serialize)]
pub struct Mutation<T> {
    pub schema: &'static str,
    pub policy: Digest,
    pub provenance: super::super::Provenance,
    pub selection: lifecycle::Status,
    pub result: T,
    pub post_effect_hold: Option<AdmissionError>,
    pub storage_converged: Option<bool>,
}
pub(super) fn publisher(
    profile: &publication::Profile,
    control: &Option<control::Profile>,
) -> Result<publication::Publisher, Error> {
    Ok(match control {
        Some(c) => publication::Publisher::controlled(profile.clone(), c.clone())?,
        None => publication::Publisher::new(profile.clone())?,
    })
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    fn mutation<T>(
        &self,
        mut state: Selected,
        result: T,
        now: &mut impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Mutation<T> {
        let hold = self.post(&mut state, now);
        Mutation {
            schema: "git-asset.trusted-native-mutation-report.experimental-1",
            policy: state.policy,
            provenance: self.request.provenance().clone(),
            selection: state.status,
            result,
            post_effect_hold: hold,
            storage_converged: None,
        }
    }
    pub fn handoff(
        &mut self,
        plan: &publication::handoff::Plan,
        working_plan: &publication::Plan,
        retained_plan: &publication::Plan,
        working: &mut publication::Resources<'_>,
        retained: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Mutation<publication::handoff::Report>, Error> {
        let policy = self.policy;
        let allowed = policy.actions.as_ref().is_some_and(|a| a.handoff);
        let ids = self.admission(allowed, &mut now)?;
        // Reverse of the admitted retained->working copy direction.
        replication::Plan::new(&policy.copy, retained_plan.clone(), working_plan.clone())?;
        let state = self.select(ids)?;
        self.revalidate(&state, &mut now)?;
        let mut source = publisher(&policy.copy.destination, &policy.destination_control)?;
        let mut keeper = publisher(&policy.copy.source, &policy.source_control)?;
        let mut refused = None;
        let result = publication::handoff::Execution {
            source: &mut source,
            keeper: &mut keeper,
            allow_release: true,
        }
        .apply_with_admission(plan, working_plan, retained_plan, working, retained, || {
            self.revalidate(&state, &mut now).map_err(|e| {
                refused = Some(e);
                publication::Error::Denied
            })
        });
        if let Some(error) = refused {
            return Err(error);
        }
        Ok(self.mutation(state, result?, &mut now))
    }
    pub fn evict<N: NodeStore, K: NodeStore>(
        &mut self,
        plan: &planning::Plan,
        sources: Sources<'_>,
        nodes: Nodes<'_, N, K>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Mutation<planning::Report>, Error> {
        let policy = self.policy;
        let profile = policy.actions.as_ref().and_then(|a| a.eviction.as_ref());
        let ids = self.admission(profile.is_some_and(|p| p.allow_evict), &mut now)?;
        let profile = profile.ok_or(Error::Admission(AdmissionError::Disabled))?;
        replication::Plan::new(
            &policy.copy,
            sources.keeper.plan.clone(),
            sources.source.plan.clone(),
        )?;
        let state = self.select(ids)?;
        self.revalidate(&state, &mut now)?;
        // Always use configured native locations, ignoring caller profile paths.
        let sources = Sources {
            source: Producer {
                profile: &policy.copy.destination,
                ..sources.source
            },
            keeper: Producer {
                profile: &policy.copy.source,
                ..sources.keeper
            },
        };
        let mut refused = None;
        let result = Controller::new(profile).apply_with_admission(plan, sources, nodes, || {
            self.revalidate(&state, &mut now).map_err(|e| {
                refused = Some(e);
                retirement::Error::Operation(publication::Error::Denied)
            })
        });
        if let Some(error) = refused {
            return Err(error);
        }
        Ok(self.mutation(state, result.map_err(Error::Eviction)?, &mut now))
    }
}
