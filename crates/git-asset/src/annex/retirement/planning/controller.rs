//! Explicit experimental SDK coordinator; construction performs no IO.
use super::*;
use git_asset_core::tree::NodeStore;
use git_asset_store::local::LocalStore;

/// Original physical producer resources, not an acknowledged alias.
pub struct Producer<'a> {
    pub profile: &'a publication::Profile,
    pub plan: &'a publication::Plan,
    pub witness: &'a LocalStore,
    pub ledger: &'a mut CapacityLedger,
}
pub struct Sources<'a> {
    pub source: Producer<'a>,
    pub keeper: Producer<'a>,
}
/// Caller-selected existing metadata readers and bounded protection/tree limits.
pub struct Nodes<'a, N: NodeStore, K: NodeStore> {
    pub source: &'a N,
    pub keeper: &'a mut K,
    pub limits: &'a Limits,
    pub tree_limits: TreeLimits,
}
pub struct Request {
    pub operation: Key,
    pub snapshot: snapshot::Snapshot,
    pub grant: RetentionGrant,
}
/// Namespace-owned actors only. Trusted owner policy is not legacy handoff proof.
/// No initialization, census, transfer or removal occurs on construction.
pub struct Controller<'a> {
    profile: &'a Profile,
}
impl<'a> Controller<'a> {
    pub fn new(profile: &'a Profile) -> Self {
        Self { profile }
    }
    fn with_live<N: NodeStore, K: NodeStore, T>(
        &self,
        snapshot: &snapshot::Snapshot,
        sources: Sources<'_>,
        nodes: Nodes<'_, N, K>,
        use_live: impl FnOnce(execution::Live<'_, '_, '_>) -> Result<T>,
    ) -> Result<T> {
        let source = sources.source;
        let keeper = sources.keeper;
        // Policy admission precedes explicit native/config/metadata opens.
        let source_id = super::super::super::Reader::new(source.profile.pool.pool.clone())?.store();
        let keeper_id = super::super::super::Reader::new(keeper.profile.pool.pool.clone())?.store();
        self.profile.admit(&source_id, &keeper_id)?;
        let mut assessed = Assessor::new(source.profile.pool.clone())?.assess(
            &snapshot.object,
            nodes.source,
            nodes.limits,
        )?;
        let source_read = assessed.read_for_plan()?;
        let kept = assessed.open_keeper(
            keeper.profile.pool.clone(),
            snapshot,
            nodes.keeper,
            nodes.tree_limits,
        )?;
        let separation = kept.compare_source_storage()?;
        let source_producer = source
            .plan
            .producer_completion(
                source.profile,
                &self.profile.source,
                source.witness,
                source.ledger,
            )
            .map_err(Error::from)?;
        let keeper_producer = keeper
            .plan
            .producer_completion(
                keeper.profile,
                &self.profile.keeper,
                keeper.witness,
                keeper.ledger,
            )
            .map_err(Error::from)?;
        use_live(execution::Live {
            separation: &separation,
            source_read,
            source_producer,
            keeper_producer,
        })
    }
    /// Bounded readonly preview with exactly one explicit keeper/domain.
    pub fn plan<N: NodeStore, K: NodeStore>(
        &self,
        request: Request,
        sources: Sources<'_>,
        nodes: Nodes<'_, N, K>,
    ) -> Result<Plan> {
        self.with_live(&request.snapshot, sources, nodes, |live| {
            live.separation.plan(
                self.profile,
                request.operation,
                &live.source_read,
                &live.source_producer,
                &live.keeper_producer,
                request.grant,
            )
        })
    }
    /// Fresh apply only. Re-admits all proofs and compares the complete frozen plan.
    /// If an effect may have started, use status/resume; never blindly repeat apply.
    pub fn apply<N: NodeStore, K: NodeStore>(
        &self,
        plan: &Plan,
        sources: Sources<'_>,
        nodes: Nodes<'_, N, K>,
    ) -> Result<Report> {
        self.execute(plan, sources, nodes, execution::Cut::None)
    }
    fn execute<N: NodeStore, K: NodeStore>(
        &self,
        plan: &Plan,
        sources: Sources<'_>,
        nodes: Nodes<'_, N, K>,
        cut: execution::Cut,
    ) -> Result<Report> {
        plan.id()?;
        let source_nodes = nodes.source;
        let limits = nodes.limits;
        self.with_live(&plan.intent.snapshot, sources, nodes, |live| {
            execution::apply_hook(self.profile, plan, live, source_nodes, limits, cut)
        })
    }
    #[cfg(all(test, windows))]
    pub(crate) fn apply_hook<N: NodeStore, K: NodeStore>(
        &self,
        plan: &Plan,
        sources: Sources<'_>,
        nodes: Nodes<'_, N, K>,
        cut: execution::Cut,
    ) -> Result<Report> {
        self.execute(plan, sources, nodes, cut)
    }
    /// Readonly recovery inspection, not removal permission.
    pub fn status(
        &self,
        plan: &Plan,
        source: &publication::Profile,
        producer: &publication::Plan,
    ) -> Result<Status> {
        execution::status(self.profile, plan, source, producer)
    }
    /// Reconcile original completion/cancellation without repeating native removal.
    pub fn resume<N: NodeStore>(
        &self,
        plan: &Plan,
        source: &publication::Profile,
        producer: &publication::Plan,
        nodes: &N,
        limits: &Limits,
    ) -> Result<Report> {
        execution::resume(self.profile, plan, source, producer, nodes, limits)
    }
    /// Preserve surviving bytes; any native intent/completion refuses cancellation.
    pub fn cancel<N: NodeStore>(
        &self,
        plan: &Plan,
        source: &publication::Profile,
        producer: &publication::Plan,
        nodes: &N,
        limits: &Limits,
    ) -> Result<Report> {
        execution::cancel(self.profile, plan, source, producer, nodes, limits)
    }
}
