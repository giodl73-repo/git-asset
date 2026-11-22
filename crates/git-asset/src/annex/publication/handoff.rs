//! Release only a temporary single-output ingest claim; never remove bytes.
use super::*;
use git_asset_store::source_claims::{Command, Receipt as ClaimReceipt, Version};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    schema: String,
    source: Digest,
    keeper: Digest,
    retained: super::super::snapshot::Snapshot,
    namespace: Digest,
    source_key: Key,
    owner: Key,
    version: Version,
    revision: u64,
    withdraw: bool,
}
fn selected(plan: &super::Plan, namespace: &mut Namespace) -> Result<Version> {
    if namespace.identity() != &plan.admission.namespace {
        return Err(Error::Denied);
    }
    let receipt = namespace
        .selection_status(&plan.selection)?
        .ok_or(Error::NeedsRecovery)?;
    Ok(Version {
        revision: receipt.source_revision,
        selection: plan.selection.selection.clone(),
    })
}
fn retained(
    plan: &super::Plan,
    namespace: &mut Namespace,
) -> Result<(git_asset_store::source_claims::Source, Version)> {
    let version = selected(plan, namespace)?;
    let source = namespace
        .source(&plan.request.source)?
        .ok_or(Error::ProtectionLost)?;
    if source.kind != Some(Kind::Retention) || source.owner != plan.request.owner {
        return Err(Error::Denied);
    }
    if source.pending.is_some() {
        return Err(Error::Busy);
    }
    if !source
        .current
        .iter()
        .chain(&source.predecessors)
        .any(|v| v == &version)
    {
        return Err(Error::ProtectionLost);
    }
    Ok((source, version))
}
impl Plan {
    /// Readonly original journal/selection lookup. No native IO or release grant.
    pub fn new(
        source: &super::Plan,
        keeper: &super::Plan,
        snapshot: super::super::snapshot::Snapshot,
        namespace: &mut Namespace,
    ) -> Result<Self> {
        if source.candidate() != keeper.candidate()
            || snapshot.object != source.candidate().object
            || source.admission.namespace == snapshot.namespace
                && source.request.source == snapshot.source
        {
            return Err(Error::Denied);
        }
        let (state, version) = retained(source, namespace)?;
        let withdraw = state.current.as_ref() == Some(&version);
        let plan = Self {
            schema: "git-asset.native-ingest-handoff.experimental-1".into(),
            source: source.id()?,
            keeper: keeper.id()?,
            retained: snapshot,
            namespace: namespace.identity().clone(),
            source_key: source.request.source.clone(),
            owner: source.request.owner.clone(),
            version,
            revision: state.revision,
            withdraw,
        };
        plan.id()?;
        Ok(plan)
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        if self.schema != "git-asset.native-ingest-handoff.experimental-1"
            || self.revision == 0
            || self.version.revision == 0
            || self.version.revision > self.revision
            || self.retained.version.revision == 0
            || self
                .revision
                .checked_add(u64::from(self.withdraw))
                .is_none()
        {
            return Err(Error::Invalid);
        }
        encode(self)
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > 16384 {
            return Err(Error::Budget);
        }
        let plan: Self =
            serde_json::from_value(identity::parse_json(raw).map_err(|_| Error::Invalid)?)
                .map_err(|_| Error::Invalid)?;
        if plan.to_json()? != raw {
            return Err(Error::Invalid);
        }
        Ok(plan)
    }
    pub fn id(&self) -> Result<Digest> {
        Ok(Digest::blob(&self.to_json()?))
    }
    fn withdraw_command(&self) -> Command {
        Command::Withdraw {
            source: self.source_key.clone(),
            owner: self.owner.clone(),
            expected_revision: self.revision,
        }
    }
    fn retire_command(&self) -> Command {
        Command::Retire {
            source: self.source_key.clone(),
            owner: self.owner.clone(),
            expected_revision: self.revision + u64::from(self.withdraw),
            predecessor_revision: self.version.revision,
        }
    }
    fn check(
        &self,
        source: &super::Plan,
        keeper: &super::Plan,
        namespace: &mut Namespace,
    ) -> Result<()> {
        self.id()?;
        if self.source != source.id()?
            || self.keeper != keeper.id()?
            || &self.namespace != namespace.identity()
            || self.source_key != source.request.source
            || self.owner != source.request.owner
            || self.version != selected(source, namespace)?
            || source.candidate() != keeper.candidate()
            || self.retained.object != source.candidate().object
            || source.admission.namespace == self.retained.namespace
                && source.request.source == self.retained.source
        {
            return Err(Error::Conflict);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Status {
    pub plan: Digest,
    pub withdraw: Option<ClaimReceipt>,
    pub retire: Option<ClaimReceipt>,
}
impl Plan {
    /// Exact historical acknowledgements only, even when both pools are offline.
    pub fn status(&self, namespace: &mut Namespace) -> Result<Status> {
        let id = self.id()?;
        if &self.namespace != namespace.identity() {
            return Err(Error::Denied);
        }
        let withdraw = if self.withdraw {
            namespace.command_status(key(&id, "handoff-withdraw")?, self.withdraw_command())?
        } else {
            None
        };
        let retire =
            namespace.command_status(key(&id, "handoff-retire")?, self.retire_command())?;
        if self.withdraw && retire.is_some() && withdraw.is_none() {
            return Err(Error::NeedsRecovery);
        }
        Ok(Status {
            plan: id,
            withdraw,
            retire,
        })
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub status: Status,
    pub historical: bool,
    pub keeper_recovery: Option<Recovery>,
    pub keeper_snapshot_minimum: Option<git_asset_store::source_claims::Checkpoint>,
}
/// Existing publishers enforce the exact admitted native/config/control profiles.
/// Permission is explicit; this never grants native removal or creative approval.
pub struct Execution<'a> {
    pub source: &'a mut Publisher,
    pub keeper: &'a mut Publisher,
    pub allow_release: bool,
}
impl Execution<'_> {
    pub fn apply(
        &mut self,
        plan: &Plan,
        source_plan: &super::Plan,
        keeper_plan: &super::Plan,
        source: &mut Resources<'_>,
        keeper: &mut Resources<'_>,
    ) -> Result<Report> {
        self.apply_hook(plan, source_plan, keeper_plan, source, keeper, || Ok(()))
    }
    pub(super) fn apply_hook(
        &mut self,
        plan: &Plan,
        source_plan: &super::Plan,
        keeper_plan: &super::Plan,
        source: &mut Resources<'_>,
        keeper: &mut Resources<'_>,
        mut after_withdraw: impl FnMut() -> Result<()>,
    ) -> Result<Report> {
        if !self.allow_release {
            return Err(Error::Denied);
        }
        plan.check(source_plan, keeper_plan, source.namespace)?;
        let original = plan.status(source.namespace)?;
        if original.retire.is_some() {
            return Ok(Report {
                status: original,
                historical: true,
                keeper_recovery: None,
                keeper_snapshot_minimum: None,
            });
        }
        let kept = self
            .keeper
            .open_retained(keeper_plan, &plan.retained, keeper)?;
        let _source = self.source.reopen(source_plan, source)?;
        let (state, _) = retained(source_plan, source.namespace)?;
        let withdrew = original.withdraw.is_some();
        if state.revision != plan.revision + u64::from(withdrew)
            || withdrew && state.current.is_some()
        {
            return Err(Error::Stale);
        }
        // A canonical document is not authority to withdraw some newer version.
        // The journal revision comparison fences the state checked here.
        if !withdrew && plan.withdraw != (state.current.as_ref() == Some(&plan.version)) {
            return Err(Error::Conflict);
        }
        let id = plan.id()?;
        if plan.withdraw {
            source.namespace.apply(
                key(&id, "handoff-withdraw")?,
                plan.withdraw_command(),
                source.metadata,
                &source.limits,
            )?;
            after_withdraw()?;
        }
        source.namespace.apply(
            key(&id, "handoff-retire")?,
            plan.retire_command(),
            source.metadata,
            &source.limits,
        )?;
        Ok(Report {
            status: plan.status(source.namespace)?,
            historical: false,
            keeper_recovery: Some(kept.recovery().clone()),
            keeper_snapshot_minimum: Some(kept.snapshot_minimum().clone()),
        })
    }
}
