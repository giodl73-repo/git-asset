//! Explicit reacquisition after completed authorized native eviction.
//! Reuses the original metadata lifecycle and Publisher; never repairs unexplained loss.
use super::*;
use crate::annex::retirement::planning;
use git_asset_store::{accounted, metadata::MetadataStore, protection::Limits, retirement};

pub struct Context<'a> {
    pub profile: &'a Profile,
    pub control: &'a control::Profile,
    pub retirement: &'a planning::Profile,
    pub namespace: &'a mut Namespace,
    pub metadata: &'a mut MetadataStore,
    pub limits: &'a Limits,
}
pub struct Original<'a> {
    pub eviction: &'a planning::Plan,
    pub publication: &'a super::Plan,
}
pub struct Operations {
    pub native: Key,
    pub control: Key,
    /// Existing owned Retention source; must be empty when preparing.
    pub source: Key,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    schema: String,
    original: Digest,
    eviction: Digest,
    metadata: Digest,
    control: retirement::ReacquisitionPlan,
    native: super::Plan,
}
#[derive(Clone, Copy, Debug)]
pub enum Stage {
    BeforeControl,
    ControlReady,
}
fn meta_error(error: retirement::Error) -> Error {
    match error {
        retirement::Error::Store(e) => Error::from(accounted::AccountedError::Store(e)),
        retirement::Error::Ledger(e) => Error::from(accounted::AccountedError::Ledger(e)),
        retirement::Error::InvalidPlan | retirement::Error::WrongAuthorization => Error::Stale,
        _ => Error::NeedsRecovery,
    }
}
fn native_error(error: crate::annex::retirement::Error) -> Error {
    match error {
        crate::annex::retirement::Error::Operation(e) => e,
        _ => Error::NeedsRecovery,
    }
}
fn limits(ctx: &Context<'_>) -> TreeLimits {
    TreeLimits {
        max_entries: ctx.limits.bindings,
        max_reads: ctx.limits.reads,
        max_writes: ctx.limits.reads,
        max_read_bytes: ctx.limits.read_bytes,
        max_write_bytes: ctx.limits.read_bytes,
        ..TreeLimits::default()
    }
}
fn original(ctx: &Context<'_>, input: &Original<'_>) -> Result<planning::execution::Status> {
    ctx.limits.validate().map_err(|_| Error::Budget)?;
    input.publication.check_profile(ctx.profile)?;
    if input.eviction.source_producer() != &input.publication.id()?
        || input.eviction.object() != &input.publication.candidate().object
        || ctx.namespace.identity() != &input.publication.admission.namespace
    {
        return Err(Error::Stale);
    }
    let native = super::super::Reader::new(ctx.profile.pool.pool.clone())?.store();
    let binding = ctx.control.binding(&native)?;
    if input
        .publication
        .control
        .as_ref()
        .ok_or(Error::Unsupported)?
        .binding
        != binding
        || ctx.retirement.source.binding(&native)? != binding
    {
        return Err(Error::Stale);
    }
    let status = planning::Controller::new(ctx.retirement)
        .status(input.eviction, ctx.profile, input.publication)
        .map_err(native_error)?;
    if status.effect == "uncertain" {
        return Err(Error::Uncertain);
    }
    if status.effect != "completed"
        || !matches!(status.metadata_phase, retirement::status::Phase::Retired)
    {
        return Err(Error::NeedsRecovery);
    }
    Ok(status)
}
impl Plan {
    pub fn publication(&self) -> &super::Plan {
        &self.native
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        super::encode(&serde_json::to_value(self).map_err(|_| Error::Invalid)?)
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > 16384 {
            return Err(Error::Budget);
        }
        let plan: Self = serde_json::from_slice(raw).map_err(|_| Error::Invalid)?;
        if plan.schema != "git-asset.native-restore-plan.experimental-1" || plan.to_json()? != raw {
            return Err(Error::Integrity);
        }
        retirement::ReacquisitionPlan::from_json(&plan.control.to_json().map_err(meta_error)?)
            .map_err(meta_error)?;
        Ok(plan)
    }
    fn bound(&self, ctx: &Context<'_>, input: &Original<'_>) -> Result<()> {
        if self.schema != "git-asset.native-restore-plan.experimental-1"
            || self.original != input.publication.id()?
            || self.eviction != input.eviction.id().map_err(native_error)?
            || &self.metadata != ctx.metadata.checkpoint().identity()
            || self.native.candidate() != input.publication.candidate()
            || self.native.admission != input.publication.admission
            || self.native.request.owner != input.publication.request.owner
        {
            return Err(Error::Stale);
        }
        self.native.check_profile(ctx.profile)?;
        let admitted = self.native.control.as_ref().ok_or(Error::Unsupported)?;
        let previous = input
            .publication
            .control
            .as_ref()
            .ok_or(Error::Unsupported)?;
        if admitted.plan.job != previous.plan.job {
            return Err(Error::Stale);
        }
        if super::digest(&admitted.plan.publication)? != super::digest(self.control.payload_plan())?
            || self.control.context()
                != &control::writer_context(ctx.profile, &self.native.request)?
        {
            return Err(Error::Stale);
        }
        let native = super::super::Reader::new(ctx.profile.pool.pool.clone())?.store();
        if admitted.binding != ctx.control.binding(&native)? {
            return Err(Error::Stale);
        }
        let store = ctx.control.open_store(&native)?;
        self.control
            .validate_retirement(&store, input.eviction.metadata_plan())
            .map_err(meta_error)?;
        Ok(())
    }
}
/// Prepare immutable metadata only; caller persists this exact Plan before applying.
pub fn prepare(ctx: &mut Context<'_>, input: Original<'_>, operations: Operations) -> Result<Plan> {
    if !ctx.profile.allow_publish || !ctx.control.allow_reacquire {
        return Err(Error::Denied);
    }
    let status = original(ctx, &input)?;
    if !matches!(
        status.observed,
        planning::execution::Observation::ObservedAbsent
    ) {
        return Err(Error::Conflict);
    }
    let state = ctx
        .namespace
        .source(&operations.source)?
        .ok_or(Error::Missing)?;
    if state.owner != input.publication.request.owner || state.kind != Some(Kind::Retention) {
        return Err(Error::Denied);
    }
    if state.pending.is_some() || state.current.is_some() {
        return Err(Error::Stale);
    }
    let old = input
        .publication
        .control
        .as_ref()
        .ok_or(Error::Unsupported)?;
    if operations.native == input.publication.request.operation
        || operations.control == old.plan.publication.request().operation
    {
        return Err(Error::Conflict);
    }
    let mut request = input.publication.request.clone();
    request.operation = operations.native;
    request.source = operations.source;
    request.expected_revision = state.revision;
    let tree_limits = limits(ctx);
    let frozen = freeze(&request, ctx.metadata, tree_limits)?;
    ctx.metadata.finish().map_err(|_| Error::NeedsRecovery)?;
    let native = super::super::Reader::new(ctx.profile.pool.pool.clone())?.store();
    let store = ctx.control.open_store(&native)?;
    let mut publication = old.plan.publication.request().clone();
    publication.operation = operations.control;
    let control = retirement::plan_reacquisition(
        &store,
        input.eviction.metadata_plan(),
        publication,
        control::writer_context(ctx.profile, &request)?,
    )
    .map_err(meta_error)?;
    let target = super::Plan::controlled(
        ctx.profile,
        request,
        frozen,
        input.publication.admission.clone(),
        ctx.control,
        control::Plan {
            publication: control.payload_plan().clone(),
            job: old.plan.job.clone(),
        },
    )?;
    let plan = Plan {
        schema: "git-asset.native-restore-plan.experimental-1".into(),
        original: input.publication.id()?,
        eviction: input.eviction.id().map_err(native_error)?,
        metadata: ctx.metadata.checkpoint().identity().clone(),
        control,
        native: target,
    };
    let mut reader = ctx
        .metadata
        .finished_reader()
        .map_err(|_| Error::NeedsRecovery)?;
    verify(&plan.native, &mut reader, limits(ctx))?;
    plan.bound(ctx, &input)?;
    plan.to_json()?;
    Ok(plan)
}
fn admit_control(
    ctx: &mut Context<'_>,
    input: &Original<'_>,
    plan: &Plan,
    admit: &mut impl FnMut(Stage) -> Result<()>,
) -> Result<()> {
    if !ctx.profile.allow_publish || !ctx.control.allow_reacquire {
        return Err(Error::Denied);
    }
    original(ctx, input)?;
    plan.bound(ctx, input)?;
    let mut nodes = ctx
        .metadata
        .finished_reader()
        .map_err(|_| Error::NeedsRecovery)?;
    verify(&plan.native, &mut nodes, limits(ctx))?;
    let native = super::super::Reader::new(ctx.profile.pool.pool.clone())?.store();
    let store = ctx.control.open_store(&native)?;
    let mut ledger = CapacityLedger::open(&ctx.control.ledger, &ctx.control.minimum)
        .map_err(|e| Error::from(accounted::AccountedError::Ledger(e)))?;
    admit(Stage::BeforeControl)?;
    retirement::Reacquisition {
        store: &store,
        ledger: &mut ledger,
        job: &plan
            .native
            .control
            .as_ref()
            .ok_or(Error::Unsupported)?
            .plan
            .job,
    }
    .apply(
        &plan.control,
        &plan.control.authorize().map_err(meta_error)?,
        &mut std::io::Cursor::new(control::document(&plan.native.candidate().object)?),
    )
    .map_err(meta_error)?;
    admit(Stage::ControlReady)
}
/// Retry uses this same saved Plan; original native uncertainty is not a new import.
pub fn apply<S: ByteSource>(
    ctx: &mut Context<'_>,
    input: Original<'_>,
    plan: &Plan,
    resources: &mut Resources<'_>,
    source: &mut S,
    mut admit: impl FnMut(Stage) -> Result<()>,
) -> Result<Ready> {
    admit_control(ctx, &input, plan, &mut admit)?;
    Publisher::controlled(ctx.profile.clone(), ctx.control.clone())?.apply(
        &plan.native,
        resources,
        source,
    )
}
pub fn resume(
    ctx: &mut Context<'_>,
    input: Original<'_>,
    plan: &Plan,
    resources: &mut Resources<'_>,
    mut admit: impl FnMut(Stage) -> Result<()>,
) -> Result<Ready> {
    admit_control(ctx, &input, plan, &mut admit)?;
    Publisher::controlled(ctx.profile.clone(), ctx.control.clone())?.resume(&plan.native, resources)
}
