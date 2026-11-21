//! Unselected candidate registration, native ingress and retained readback.
//! Uses existing namespace/metadata/ledger/witness engines; never selects Git.
use super::{control, managed, process};
use git_asset_core::{
    Digest, LogicalPath, Object, identity,
    tree::{NodeStore, Tree, TreeLimits},
};
use git_asset_store::{
    Key,
    existing::{self, ByteSource, ProviderPath, ReadCache, RequiredProtection},
    ledger::{CapacityLedger, JournalRef, LedgerError},
    local::LocalStore,
    source_claims::{
        Kind, Reference, Selection,
        admission::{self, Namespace, Select},
    },
    upload_witness::{self, Document, Packet},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Seek},
    path::PathBuf,
    time::{Duration, Instant},
};
mod staging;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Unsupported,
    Denied,
    Budget,
    Busy,
    Conflict,
    Stale,
    Integrity,
    Missing,
    MissingAcknowledged,
    ProtectionLost,
    NeedsRecovery,
    Uncertain,
}
impl Error {
    pub fn code(self) -> &'static str {
        match self {
            Self::Invalid => "invalid",
            Self::Unsupported => "unsupported",
            Self::Denied => "denied",
            Self::Budget => "admission_denied",
            Self::Busy => "busy",
            Self::Conflict => "conflict",
            Self::Stale => "stale",
            Self::Integrity => "corrupt",
            Self::Missing => "missing",
            Self::MissingAcknowledged => "missing_acknowledged",
            Self::ProtectionLost => "protection_lost",
            Self::NeedsRecovery => "needs_recovery",
            Self::Uncertain => "uncertain",
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
impl From<existing::Error> for Error {
    fn from(e: existing::Error) -> Self {
        match e {
            existing::Error::Budget => Self::Budget,
            existing::Error::Busy => Self::Busy,
            existing::Error::Missing => Self::Missing,
            existing::Error::Integrity => Self::Integrity,
            existing::Error::Denied => Self::Denied,
            existing::Error::Unsupported => Self::Unsupported,
            _ => Self::NeedsRecovery,
        }
    }
}
impl From<super::Error> for Error {
    fn from(e: super::Error) -> Self {
        match e {
            super::Error::Unsupported => Self::Unsupported,
            super::Error::Missing => Self::Missing,
            super::Error::Budget => Self::Budget,
            super::Error::Invalid => Self::Invalid,
            super::Error::WrongIdentity => Self::Stale,
            super::Error::Configuration(e) | super::Error::Bytes(e) => e.into(),
            _ => Self::Uncertain,
        }
    }
}
impl From<admission::Error> for Error {
    fn from(e: admission::Error) -> Self {
        use git_asset_store::source_claims::Error as J;
        match e {
            admission::Error::Busy | admission::Error::Journal(J::Busy) => Self::Busy,
            admission::Error::Budget | admission::Error::Journal(J::Budget) => Self::Budget,
            admission::Error::Journal(J::Conflict) => Self::Conflict,
            admission::Error::Journal(J::Stale) => Self::Stale,
            _ => Self::NeedsRecovery,
        }
    }
}
impl From<managed::Error> for Error {
    fn from(e: managed::Error) -> Self {
        match e {
            managed::Error::Annex(e) => e.into(),
            managed::Error::Admission(e) => e.into(),
            _ => Self::Denied,
        }
    }
}
impl From<LedgerError> for Error {
    fn from(e: LedgerError) -> Self {
        match e {
            LedgerError::Busy => Self::Busy,
            LedgerError::Conflict => Self::Conflict,
            LedgerError::Budget | LedgerError::Capacity(_) | LedgerError::NeedsProbe => {
                Self::Budget
            }
            _ => Self::NeedsRecovery,
        }
    }
}
impl From<upload_witness::Error> for Error {
    fn from(e: upload_witness::Error) -> Self {
        match e {
            upload_witness::Error::Busy => Self::Busy,
            upload_witness::Error::Budget => Self::Budget,
            upload_witness::Error::Conflict => Self::Conflict,
            upload_witness::Error::InvalidPlan => Self::Invalid,
            _ => Self::NeedsRecovery,
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::NeedsRecovery
    }
}
impl From<git_asset_core::tree::TreeError> for Error {
    fn from(e: git_asset_core::tree::TreeError) -> Self {
        use git_asset_core::tree::TreeError as T;
        match e {
            T::Budget => Self::Budget,
            T::MissingNode(_) => Self::Missing,
            T::Integrity | T::InvalidNode => Self::Integrity,
            T::PathConflict => Self::Conflict,
            T::StaleRoot => Self::Stale,
            T::Store(_) => Self::NeedsRecovery,
        }
    }
}
fn encode(v: &impl Serialize) -> Result<Vec<u8>> {
    let b = identity::canonical_json(&serde_json::to_value(v).map_err(|_| Error::Invalid)?)
        .map_err(|_| Error::Invalid)?;
    if b.len() > 16384 {
        return Err(Error::Budget);
    }
    Ok(b)
}
fn digest(v: &impl Serialize) -> Result<Digest> {
    Ok(Digest::blob(&encode(v)?))
}
fn key(scope: &Digest, role: &str) -> Result<Key> {
    Key::parse(&format!("annex-{role}-{}", scope.as_str())).map_err(|_| Error::Invalid)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub logical_id: String,
    pub object: Object,
    pub provenance: Object,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub operation: Key,
    pub owner: Key,
    pub source: Key,
    pub expected_revision: u64,
    pub candidate: Candidate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub object_bytes: u64,
    pub transfer_bytes: u64,
    pub millis: u64,
}
impl Budget {
    fn validate(&self) -> Result<()> {
        if self.object_bytes == 0
            || self.object_bytes > 64 * 1024 * 1024 * 1024
            || self.transfer_bytes == 0
            || self.transfer_bytes > 1024 * 1024 * 1024 * 1024
            || self.millis == 0
            || self.millis > 300000
        {
            return Err(Error::Budget);
        }
        Ok(())
    }
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub pool: managed::Profile,
    pub retained_place: Key,
    pub staging: PathBuf,
    pub staging_marker: Object,
    pub budget: Budget,
    #[serde(default)]
    pub allow_publish: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub namespace: Digest,
    pub witness: Digest,
    pub capacity: Digest,
    pub job: JournalRef,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    request: Request,
    selection: Select,
    admission: Admission,
    pool: Digest,
    configuration: Object,
    binding: Digest,
    retained_place: Key,
    staging_marker: Object,
    staging_location: Digest,
    budget: Budget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    context: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    control: Option<ControlAdmission>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlAdmission {
    binding: Digest,
    plan: control::Plan,
}
fn descriptor(request: &Request) -> Result<Vec<u8>> {
    if request.candidate.logical_id.is_empty()
        || request.candidate.logical_id.len() > 1024
        || request.candidate.provenance.bytes > 65536
    {
        return Err(Error::Invalid);
    }
    encode(
        &serde_json::json!({"schema":"git-asset.annex-candidate.experimental-1","request":request}),
    )
}
fn bindings(request: &Request) -> Result<BTreeMap<LogicalPath, Object>> {
    Ok(BTreeMap::from([(
        LogicalPath::parse("candidate/output").map_err(|_| Error::Invalid)?,
        request.candidate.object.clone(),
    )]))
}
/// Writes candidate metadata only; caller must finish/reopen before registration.
pub fn freeze(
    request: &Request,
    nodes: &mut impl NodeStore,
    mut limits: TreeLimits,
) -> Result<Selection> {
    let raw = descriptor(request)?;
    limits.max_reads = limits.max_reads.checked_sub(1).ok_or(Error::Budget)?;
    limits.max_read_bytes = limits
        .max_read_bytes
        .checked_sub(request.candidate.provenance.bytes as usize)
        .ok_or(Error::Budget)?;
    let provenance = nodes
        .get(&request.candidate.provenance)
        .map_err(Error::from)?
        .ok_or(Error::Missing)?;
    if Object::from_bytes(&provenance) != request.candidate.provenance {
        return Err(Error::Integrity);
    }
    limits.max_writes = limits.max_writes.checked_sub(1).ok_or(Error::Budget)?;
    limits.max_write_bytes = limits
        .max_write_bytes
        .checked_sub(raw.len())
        .ok_or(Error::Budget)?;
    let header = nodes.put(&raw)?;
    if header != Object::from_bytes(&raw) {
        return Err(Error::Integrity);
    }
    let (tree, _) = Tree::build(&bindings(request)?, nodes, limits)?;
    Ok(Selection {
        root: tree.root().clone(),
        context: header.sha256,
    })
}
fn verify(plan: &Plan, nodes: &mut impl NodeStore, mut limits: TreeLimits) -> Result<()> {
    let expected = bindings(&plan.request)?;
    let raw = descriptor(&plan.request)?;
    limits.max_reads = limits.max_reads.checked_sub(2).ok_or(Error::Budget)?;
    let metadata_bytes = raw
        .len()
        .checked_add(plan.candidate().provenance.bytes as usize)
        .ok_or(Error::Budget)?;
    limits.max_read_bytes = limits
        .max_read_bytes
        .checked_sub(metadata_bytes)
        .ok_or(Error::Budget)?;
    if plan.selection.selection.context != Digest::blob(&raw) {
        return Err(Error::Integrity);
    }
    let header = Object::from_bytes(&raw);
    for object in [&header, &plan.candidate().provenance] {
        let bytes = nodes
            .get(object)
            .map_err(Error::from)?
            .ok_or(Error::Missing)?;
        if Object::from_bytes(&bytes) != *object || object == &header && bytes != raw {
            return Err(Error::Integrity);
        }
    }
    let (actual, _) =
        Tree::from_root(plan.selection.selection.root.clone()).entries(nodes, limits)?;
    if actual != expected {
        return Err(Error::Integrity);
    }
    Ok(())
}
impl Plan {
    pub fn new(
        profile: &Profile,
        request: Request,
        frozen: Selection,
        admission: Admission,
    ) -> Result<Self> {
        Self::build(profile, request, frozen, admission, None)
    }
    pub fn controlled(
        profile: &Profile,
        request: Request,
        frozen: Selection,
        admission: Admission,
        control_profile: &control::Profile,
        control: control::Plan,
    ) -> Result<Self> {
        let native = super::Reader::new(profile.pool.pool.clone())?.store();
        control_profile.check(&native, &request.candidate.object, &control)?;
        let control = ControlAdmission {
            binding: control_profile.binding(&native)?,
            plan: control,
        };
        Self::build(profile, request, frozen, admission, Some(control))
    }
    fn build(
        profile: &Profile,
        request: Request,
        frozen: Selection,
        admission: Admission,
        control: Option<ControlAdmission>,
    ) -> Result<Self> {
        profile.budget.validate()?;
        descriptor(&request)?;
        if request.candidate.object.bytes > profile.budget.object_bytes
            || request.candidate.object.bytes > profile.budget.transfer_bytes
        {
            return Err(Error::Budget);
        }
        let reader = managed::Reader::new(profile.pool.clone())?;
        let scope = digest(&(reader.store(), &request.operation))?;
        let selection = Select {
            prepare: key(&scope, "prepare")?,
            adopt: key(&scope, "adopt")?,
            source: request.source.clone(),
            owner: request.owner.clone(),
            expected_revision: request.expected_revision,
            selection: frozen,
        };
        Ok(Self {
            request,
            selection,
            admission,
            pool: reader.store(),
            configuration: profile.pool.pool.configuration.clone(),
            binding: reader.binding().clone(),
            retained_place: profile.retained_place.clone(),
            staging_marker: profile.staging_marker.clone(),
            staging_location: digest(&profile.staging)?,
            budget: profile.budget,
            context: None,
            control,
        })
    }
    pub(super) fn with_context(mut self, context: Option<Digest>) -> Self {
        self.context = context;
        self
    }
    pub(super) fn check_profile(&self, profile: &Profile) -> Result<()> {
        let rebuilt = Self::build(
            profile,
            self.request.clone(),
            self.selection.selection.clone(),
            self.admission.clone(),
            self.control.clone(),
        )?
        .with_context(self.context.clone());
        if &rebuilt != self {
            return Err(Error::Stale);
        }
        Ok(())
    }
    pub fn id(&self) -> Result<Digest> {
        digest(self)
    }
    pub fn candidate(&self) -> &Candidate {
        &self.request.candidate
    }
    fn scope(&self) -> Result<Digest> {
        digest(&(&self.pool, &self.request.operation))
    }
    pub fn payload_peak(&self) -> Result<u64> {
        self.candidate()
            .object
            .bytes
            .checked_add(4095)
            .map(|n| n / 4096 * 4096)
            .and_then(|n| n.checked_mul(3))
            .and_then(|n| n.checked_add(65536))
            .ok_or(Error::Budget)
    }
    fn packet(&self, witness: &LocalStore) -> Result<Packet> {
        let scope = self.scope()?;
        Ok(Packet::new(
            witness,
            &self.id()?,
            &self.request.owner,
            [
                Document {
                    operation: key(&scope, "intent")?,
                    bytes: encode(
                        &serde_json::json!({"schema":"git-asset.annex-candidate-intent.experimental-1","plan":self}),
                    )?,
                },
                Document {
                    operation: key(&scope, "done")?,
                    bytes: encode(
                        &serde_json::json!({"schema":"git-asset.annex-candidate-done.experimental-1","plan":self.id()?,"object":self.candidate().object}),
                    )?,
                },
            ],
        )?)
    }
    pub fn required_peak(&self, witness: &LocalStore) -> Result<u64> {
        self.payload_peak()?
            .checked_add(self.packet(witness)?.required_peak()?)
            .ok_or(Error::Budget)
    }
}
pub struct Resources<'a> {
    pub namespace: &'a mut Namespace,
    pub metadata: &'a mut git_asset_store::metadata::MetadataReader,
    pub limits: git_asset_store::protection::Limits,
    pub witness: &'a LocalStore,
    pub ledger: &'a mut CapacityLedger,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub descriptor: Object,
    pub plan: Digest,
    pub candidate: Candidate,
    pub registration: Reference,
    pub charged_file_data: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recovery {
    pub metadata_minimum: git_asset_store::metadata::Checkpoint,
    pub namespace_minimum: git_asset_store::source_claims::Checkpoint,
    pub capacity_minimum: git_asset_store::ledger::Checkpoint,
}
pub struct Ready {
    receipt: Receipt,
    read: managed::Read,
    recovery: Recovery,
    _control: Option<control::Use>,
}
impl Ready {
    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }
    pub fn recovery(&self) -> &Recovery {
        &self.recovery
    }
    pub fn with_path<T>(&self, f: impl FnOnce(&std::path::Path) -> T) -> existing::Result<T> {
        self.read.with_path(f)
    }
    pub fn held_namespaces(&self) -> usize {
        self.read.held_namespaces()
    }
}
impl Read for Ready {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        self.read.read(b)
    }
}
impl Seek for Ready {
    fn seek(&mut self, p: std::io::SeekFrom) -> std::io::Result<u64> {
        self.read.seek(p)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Registered,
    Intent,
    StagingChunk,
    Staged,
    Imported,
    Completed,
    Settled,
}
struct Work {
    start: Instant,
    remaining: u64,
    millis: u64,
}
impl Work {
    fn new(b: Budget) -> Self {
        Self {
            start: Instant::now(),
            remaining: b.transfer_bytes,
            millis: b.millis,
        }
    }
    fn check(&self) -> Result<()> {
        if self.start.elapsed() > Duration::from_millis(self.millis) {
            Err(Error::Budget)
        } else {
            Ok(())
        }
    }
    fn consume(&mut self, n: u64) -> Result<()> {
        self.check()?;
        self.remaining = self.remaining.checked_sub(n).ok_or(Error::Budget)?;
        Ok(())
    }
}
pub struct Publisher {
    profile: Profile,
    control: Option<control::Profile>,
    reader: managed::Reader,
    staging: ReadCache,
}
#[derive(Debug, Serialize)]
pub struct Status {
    pub plan: Digest,
    pub registered: bool,
    pub intent_recorded: bool,
    pub historical_completion: bool,
    pub accounting_settled: bool,
    pub protected: bool,
    pub bytes_verified: bool,
}
struct Observed {
    status: Status,
    registration: Option<Reference>,
    hold: managed::HeldUse,
    read: Option<existing::VerifiedRead>,
    charged: Option<u64>,
    control: Option<control::Use>,
}
impl Publisher {
    pub fn new(profile: Profile) -> Result<Self> {
        Self::build(profile, None)
    }
    pub fn controlled(profile: Profile, control: control::Profile) -> Result<Self> {
        Self::build(profile, Some(control))
    }
    fn build(profile: Profile, control: Option<control::Profile>) -> Result<Self> {
        profile.budget.validate()?;
        if !super::absolute(&profile.staging)
            || profile.staging_marker.bytes == 0
            || profile.staging_marker.bytes > 65536
        {
            return Err(Error::Invalid);
        }
        if let Some(control) = &control {
            control.binding(&super::Reader::new(profile.pool.pool.clone())?.store())?;
        }
        let staging = ReadCache::provider(
            RequiredProtection::PreventMutation,
            existing::Limits {
                object_bytes: 65536,
                total_bytes: 65536 * 65536,
                requests: 65536,
                live_readers: 1,
                verify_millis: profile.pool.pool.limits.verify_millis,
            },
        )?;
        let reader = managed::Reader::new(profile.pool.clone())?;
        Ok(Self {
            profile,
            control,
            reader,
            staging,
        })
    }
    fn check(&self, p: &Plan, r: &mut Resources<'_>) -> Result<()> {
        p.check_profile(&self.profile)?;
        match (&self.control, &p.control) {
            (Some(profile), Some(admission)) => {
                if profile.binding(&self.reader.store())? != admission.binding {
                    return Err(Error::Stale);
                }
                profile.check(&self.reader.store(), &p.candidate().object, &admission.plan)?;
            }
            (None, None) => (),
            _ => return Err(Error::Stale),
        }

        if self.reader.store() != p.pool
            || self.reader.binding() != &p.binding
            || r.namespace.identity() != &p.admission.namespace
            || r.ledger.identity() != &p.admission.capacity
            || r.witness.identity() != &p.admission.witness
        {
            return Err(Error::Stale);
        }
        if !self
            .profile
            .pool
            .binding
            .members()
            .iter()
            .any(|m| m.namespace == p.admission.namespace && m.places.contains(&p.retained_place))
        {
            return Err(Error::Denied);
        }
        let policy = &r.namespace.policy().classes[&Kind::Retention];
        if ![&policy.current, &policy.pending, &policy.predecessor]
            .iter()
            .all(|places| places.contains(&p.retained_place))
        {
            return Err(Error::Denied);
        }
        r.limits.validate().map_err(|_| Error::Budget)?;
        verify(
            p,
            r.metadata,
            TreeLimits {
                max_entries: r.limits.bindings,
                max_reads: r.limits.reads,
                max_read_bytes: r.limits.read_bytes,
                ..TreeLimits::default()
            },
        )?;
        Ok(())
    }
    fn protected(p: &Plan, namespace: &mut Namespace) -> Result<()> {
        let s = namespace
            .source(&p.request.source)?
            .ok_or(Error::ProtectionLost)?;
        if s.kind != Some(Kind::Retention)
            || s.owner != p.request.owner
            || !s
                .current
                .iter()
                .chain(&s.predecessors)
                .any(|v| v.selection == p.selection.selection)
        {
            return Err(Error::ProtectionLost);
        }
        Ok(())
    }
    fn observe(&mut self, p: &Plan, r: &mut Resources<'_>) -> Result<Observed> {
        if !process::supported() {
            return Err(Error::Unsupported);
        }
        self.check(p, r)?;
        let hold = self.reader.hold_for_publication(&p.candidate().object)?;
        let expected = self
            .control
            .as_ref()
            .map(|c| c.binding(&self.reader.store()))
            .transpose()?;
        control::configuration(
            &self.profile.pool.pool,
            &self.profile.pool.binding.id().map_err(|_| Error::Invalid)?,
            expected.as_ref(),
        )?;
        let control = match (&self.control, &p.control) {
            (Some(profile), Some(plan)) => {
                Some(profile.open(&self.reader.store(), &p.candidate().object, &plan.plan)?)
            }
            (None, None) => None,
            _ => return Err(Error::Invalid),
        };
        let registration = r
            .namespace
            .selection_status(&p.selection)?
            .map(|v| v.reference);
        let protected = match Self::protected(p, r.namespace) {
            Ok(()) => true,
            Err(Error::ProtectionLost) => false,
            Err(e) => return Err(e),
        };
        let state = p
            .packet(r.witness)?
            .inspect(r.witness, r.ledger, &p.admission.job)?;
        let allocation = r
            .ledger
            .allocation_observation(&key(&p.scope()?, "payload")?)?;
        let charged = allocation.as_ref().and_then(|v| v.charged_bytes);
        if state.intent && (registration.is_none() || allocation.is_none())
            || charged.is_some() && !state.completion
        {
            return Err(Error::NeedsRecovery);
        }
        let read = match self.reader.lookup(&p.candidate().object, &hold) {
            Ok(read) => Some(read),
            Err(managed::Error::Annex(super::Error::Missing)) => None,
            Err(e) => return Err(e.into()),
        };
        if state.completion && read.is_none() {
            return Err(Error::MissingAcknowledged);
        }
        if read.is_none()
            && let (Some(held), Some(plan)) = (&control, &p.control)
            && !held.allows_missing(
                &self.profile,
                self.control.as_ref().ok_or(Error::Invalid)?,
                &p.request,
                &plan.plan,
            )?
        {
            // Another producer may still be unfinished; absence is not evidence of its acknowledgement.
            return Err(Error::NeedsRecovery);
        }
        Ok(Observed {
            status: Status {
                plan: p.id()?,
                registered: registration.is_some(),
                intent_recorded: state.intent,
                historical_completion: state.completion,
                accounting_settled: state.settled && charged.is_some(),
                protected,
                bytes_verified: read.is_some(),
            },
            registration,
            hold,
            read,
            charged,
            control,
        })
    }
    /// Exact read-only state and fresh bytes; never reconstructs missing evidence.
    pub fn status(&mut self, p: &Plan, r: &mut Resources<'_>) -> Result<Status> {
        Ok(self.observe(p, r)?.status)
    }
    fn ready(p: &Plan, r: &mut Resources<'_>, observed: Observed) -> Result<Ready> {
        Self::protected(p, r.namespace)?;
        if !observed.status.historical_completion
            || !observed.status.accounting_settled
            || !observed.status.protected
        {
            return Err(Error::NeedsRecovery);
        }
        let receipt = Receipt {
            descriptor: Object::from_bytes(&descriptor(&p.request)?),
            plan: p.id()?,
            candidate: p.candidate().clone(),
            registration: observed.registration.ok_or(Error::NeedsRecovery)?,
            charged_file_data: observed.charged.ok_or(Error::NeedsRecovery)?,
        };
        let recovery = Recovery {
            metadata_minimum: r.metadata.checkpoint().clone(),
            namespace_minimum: r.namespace.checkpoint(),
            capacity_minimum: r.ledger.checkpoint(),
        };
        Ok(Ready {
            receipt,
            recovery,
            _control: observed.control,
            read: managed::Reader::finish_read(
                observed.hold,
                observed.read.ok_or(Error::MissingAcknowledged)?,
            ),
        })
    }
    /// Works after producer input and owned staging have gone offline. No writes.
    pub fn reopen(&mut self, p: &Plan, r: &mut Resources<'_>) -> Result<Ready> {
        let observed = self.observe(p, r)?;
        if !observed.status.protected {
            return Err(Error::ProtectionLost);
        }
        Self::ready(p, r, observed)
    }
    pub fn apply<S: ByteSource>(
        &mut self,
        p: &Plan,
        r: &mut Resources<'_>,
        source: &mut S,
    ) -> Result<Ready> {
        self.apply_hook(p, r, source, |_| Ok(()))
    }
    pub fn apply_hook<S: ByteSource>(
        &mut self,
        p: &Plan,
        r: &mut Resources<'_>,
        source: &mut S,
        mut hook: impl FnMut(Stage) -> Result<()>,
    ) -> Result<Ready> {
        self.run(
            p,
            r,
            &mut || {
                source
                    .open_verified(&p.candidate().object)
                    .map_err(Error::from)
            },
            &mut hook,
        )
    }
    /// Explicit original-operation recovery without opening producer input.
    /// An incomplete stage requires apply with the exact guarded input again.
    pub fn resume(&mut self, p: &Plan, r: &mut Resources<'_>) -> Result<Ready> {
        self.run::<std::io::Cursor<Vec<u8>>>(p, r, &mut || Err(Error::Missing), &mut |_| Ok(()))
    }
    fn run<R: Read + Seek>(
        &mut self,
        p: &Plan,
        r: &mut Resources<'_>,
        source: &mut impl FnMut() -> Result<R>,
        hook: &mut impl FnMut(Stage) -> Result<()>,
    ) -> Result<Ready> {
        if !self.profile.allow_publish {
            return Err(Error::Denied);
        }
        let mut work = Work::new(p.budget);
        let initial_metrics = r.metadata.metrics();
        let mut observed = self.observe(p, r)?;
        work.check()?;
        if observed.read.is_some() {
            work.consume(p.candidate().object.bytes)?;
        }
        if observed.status.accounting_settled {
            return Self::ready(p, r, observed);
        }
        if observed.registration.is_none() {
            let state = r
                .namespace
                .source(&p.request.source)?
                .ok_or(Error::ProtectionLost)?;
            if state.kind != Some(Kind::Retention) || state.owner != p.request.owner {
                return Err(Error::Denied);
            }
            let mut limits = r.limits.clone();
            let metrics = r.metadata.metrics();
            limits.reads = limits
                .reads
                .checked_sub(
                    (metrics.sql_node_reads - initial_metrics.sql_node_reads)
                        .try_into()
                        .map_err(|_| Error::Budget)?,
                )
                .ok_or(Error::Budget)?;
            limits.read_bytes = limits
                .read_bytes
                .checked_sub(
                    (metrics.node_bytes_read - initial_metrics.node_bytes_read)
                        .try_into()
                        .map_err(|_| Error::Budget)?,
                )
                .ok_or(Error::Budget)?;
            let receipt = r.namespace.select(&p.selection, r.metadata, &limits)?;
            observed.registration = Some(receipt.reference);
        }
        Self::protected(p, r.namespace)?;
        hook(Stage::Registered)?;
        work.check()?;
        let _marker = self.staging.open_provider_file(
            &self.profile.staging,
            &ProviderPath::parse(".git-asset-staging")?,
            &p.staging_marker,
        )?;
        if fs2::available_space(&self.profile.staging)? < p.payload_peak()?
            || fs2::available_space(&self.profile.pool.pool.repository)? < p.payload_peak()?
        {
            return Err(Error::Budget);
        }
        let payload = r.ledger.begin_allocation(
            key(&p.scope()?, "payload")?,
            &p.admission.job,
            key(&p.id()?, "binding")?,
            p.payload_peak()?,
        )?;
        let packet = p.packet(r.witness)?;
        let pending = packet.begin(r.witness, r.ledger, &p.admission.job)?;
        hook(Stage::Intent)?;
        work.check()?;
        let read = match observed.read.take() {
            Some(read) => read,
            None => {
                let path = staging::prepare(&self.profile, p, &mut work, source, hook)?;
                hook(Stage::Staged)?;
                work.check()?;
                let output = process::run_for_object(
                    &self.profile.pool.pool,
                    &p.candidate().object,
                    true,
                    &[
                        "reinject",
                        "--guesskeys",
                        "--json",
                        "--json-error-messages",
                        "--",
                        path.to_str().ok_or(Error::Invalid)?,
                    ],
                )
                .map_err(|_| Error::Uncertain)?;
                let line = super::line(&output.stdout).map_err(|_| Error::Uncertain)?;
                let response =
                    identity::parse_json(line.as_bytes()).map_err(|_| Error::Uncertain)?;
                let native_key = format!(
                    "SHA256-s{}--{}",
                    p.candidate().object.bytes,
                    p.candidate().object.sha256.as_str()
                );
                if response.get("success") != Some(&serde_json::Value::Bool(true))
                    || response.get("key").and_then(|v| v.as_str()) != Some(native_key.as_str())
                {
                    return Err(Error::Uncertain);
                }
                hook(Stage::Imported)?;
                work.consume(p.candidate().object.bytes)?;
                self.reader.lookup(&p.candidate().object, &observed.hold)?
            }
        };
        work.check()?;
        Self::protected(p, r.namespace)?;
        let final_bytes = read
            .with_path(|path| File::open(path).and_then(|f| fs2::FileExt::allocated_size(&f)))??;
        let footprint = final_bytes
            .checked_add(staging::footprint(&self.profile, p)?)
            .ok_or(Error::Budget)?;
        if footprint > p.payload_peak()? {
            return Err(Error::Budget);
        }
        pending.complete().map_err(|_| Error::Uncertain)?;
        hook(Stage::Completed)?;
        r.ledger
            .complete_allocation(key(&p.scope()?, "settled")?, payload.reference(), footprint)
            .map_err(|_| Error::Uncertain)?;
        hook(Stage::Settled)?;
        observed.read = Some(read);
        observed.charged = Some(footprint);
        observed.status.historical_completion = true;
        observed.status.accounting_settled = true;
        observed.status.protected = true;
        Self::ready(p, r, observed)
    }
}
