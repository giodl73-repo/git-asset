//! Fixed private native profile. Invoked only after strict authenticated intake.
use super::*;
use git_asset::{
    annex::publication,
    github::native::{self, batch::Failure},
};
use git_asset_store::{ledger, metadata::MetadataReader, protection::Limits};

pub(super) const SCHEMA: &str = "git-asset.native-receiver-placement.experimental-1";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Location {
    namespace: profiles::NamespaceRoot,
    metadata_root: PathBuf,
    metadata_minimum: metadata::Checkpoint,
    witness_root: PathBuf,
    ledger_root: PathBuf,
    capacity_minimum: ledger::Checkpoint,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Storage {
    schema: String,
    policy: native::Policy,
    source: Location,
    destination: Location,
    #[serde(default)]
    pending: Option<profiles::NamespaceRoot>,
    limits: Limits,
}
struct Opened {
    namespace: Namespace,
    metadata: MetadataReader,
    witness: LocalStore,
    ledger: CapacityLedger,
}
impl Location {
    fn open(&self) -> github::Result<Opened> {
        Ok(Opened {
            namespace: Namespace::open(&self.namespace.root, &self.namespace.minimum)
                .map_err(|_| github::Error::Unavailable)?,
            metadata: MetadataReader::open(&self.metadata_root, &self.metadata_minimum)
                .map_err(|_| github::Error::Unavailable)?,
            witness: LocalStore::open(&self.witness_root)
                .map_err(|_| github::Error::Unavailable)?,
            ledger: CapacityLedger::open(&self.ledger_root, &self.capacity_minimum)
                .map_err(|_| github::Error::Unavailable)?,
        })
    }
}
impl Opened {
    fn resources<'a>(&'a mut self, limits: &'a Limits) -> publication::Resources<'a> {
        publication::Resources {
            namespace: &mut self.namespace,
            metadata: &mut self.metadata,
            limits: limits.clone(),
            witness: &self.witness,
            ledger: &mut self.ledger,
        }
    }
}
pub(super) fn dispatch(
    profile: &Profile,
    client: &Client,
    proof: github::Verified,
    cursor: Option<Cursor>,
    document: Value,
) -> github::Result<Value> {
    let storage: Storage = serde_json::from_value(document).map_err(|_| github::Error::Invalid)?;
    if storage.schema != SCHEMA || storage.policy.producers.is_none() {
        return Err(github::Error::Invalid);
    }
    storage
        .limits
        .validate()
        .map_err(|_| github::Error::Budget)?;
    profile.budget.validate()?;
    let policy = storage.policy.id().map_err(|_| github::Error::Trust)?;
    proof.admit(
        now()?,
        &profile.enrollment.id().map_err(|_| github::Error::Trust)?,
        &policy,
    )?;
    let scope = storage
        .policy
        .actions
        .as_ref()
        .and_then(|a| a.pending.as_ref());
    if storage.pending.as_ref().map(|p| p.minimum.identity()) != scope.map(|p| &p.namespace)
        || storage.policy.actions.as_ref().is_some_and(|a| a.handoff) && scope.is_none()
    {
        return Err(github::Error::Trust);
    }
    if !profile.allow_selection {
        return Err(github::Error::Disabled);
    }
    if storage.policy.allow_copy && !cfg!(windows) {
        return Ok(
            json!({"schema":"git-asset.native-copy-pass-refusal.experimental-1",
            "ok":false,"failure":{"layer":"native","code":"unsupported"},"storage_converged":null}),
        );
    }
    // No resource locations, plans, tools or credentials come from the request.
    let mut source = storage.source.open()?;
    let mut destination = storage.destination.open()?;
    let mut pending = storage
        .pending
        .as_ref()
        .map(|p| Namespace::open(&p.root, &p.minimum).map_err(|_| github::Error::Unavailable))
        .transpose()?;
    let mut namespace = Namespace::open(&profile.namespace.root, &profile.namespace.minimum)
        .map_err(|_| github::Error::Unavailable)?;
    let mut metadata = MetadataStore::open(&profile.metadata_root, &profile.metadata_minimum)
        .map_err(|_| github::Error::Unavailable)?;
    let repository = github::source::AnnexSource {
        client,
        trust: &profile.trust,
    };
    let result = native::Execution {
        trust: &profile.trust,
        request: &proof,
        policy: &storage.policy,
        enrollment: &profile.enrollment,
        repository: &repository,
        namespace: &mut namespace,
        metadata: &mut metadata,
        limits: &storage.limits,
    }
    .reconcile_with_actions(
        cursor.as_ref(),
        &profile.budget,
        pending.as_mut(),
        &mut source.resources(&storage.limits),
        &mut destination.resources(&storage.limits),
        now,
    );
    match result {
        Ok(report) => serde_json::to_value(report).map_err(|_| github::Error::Invalid),
        Err(e) => Ok(
            json!({"schema":"git-asset.native-copy-pass-refusal.experimental-1",
            "ok":false,"failure":Failure::from(&e),"storage_converged":null}),
        ),
    }
}
