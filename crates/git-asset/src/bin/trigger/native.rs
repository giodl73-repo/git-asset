//! Fixed private native profile. Invoked only after strict authenticated intake.
use super::*;
use git_asset::{
    annex::publication,
    github::native::{self, batch::Failure},
};
use git_asset_store::{ledger, metadata::MetadataReader, protection::Limits};

pub(super) const SCHEMA: &str = "git-asset.native-receiver-placement.experimental-1";
#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    #[default]
    Copy,
    EvictionPreview,
    EvictionApply,
    HistoryPreview,
    HistoryApply,
}
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
    #[serde(default)]
    phase: Phase,
    #[serde(default)]
    history_records: Option<profiles::NamespaceRoot>,
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
    let history_phase = matches!(storage.phase, Phase::HistoryPreview | Phase::HistoryApply);
    if storage.schema != SCHEMA || (!history_phase && storage.policy.producers.is_none()) {
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
        || ((matches!(storage.phase, Phase::Copy)
            && storage.policy.actions.as_ref().is_some_and(|a| a.handoff))
            || matches!(storage.phase, Phase::EvictionPreview | Phase::EvictionApply))
            && scope.is_none()
    {
        return Err(github::Error::Trust);
    }
    if !profile.allow_selection {
        return Err(github::Error::Disabled);
    }
    let history = storage
        .policy
        .actions
        .as_ref()
        .and_then(|a| a.history.as_ref());
    if storage
        .history_records
        .as_ref()
        .is_some_and(|r| history.is_none_or(|h| r.minimum.identity() != &h.records.namespace))
    {
        return Err(github::Error::Trust);
    }
    if history_phase {
        let h = history.ok_or(github::Error::Trust)?;
        let records = storage
            .history_records
            .as_ref()
            .ok_or(github::Error::Trust)?;
        if storage.source.namespace.minimum.identity() != &h.target.namespace
            || storage.source.metadata_minimum.identity() != &h.records.metadata
        {
            return Err(github::Error::Trust);
        }
        let mut records = Namespace::open(&records.root, &records.minimum)
            .map_err(|_| github::Error::Unavailable)?;
        let mut retained = MetadataStore::open(
            &storage.source.metadata_root,
            &storage.source.metadata_minimum,
        )
        .map_err(|_| github::Error::Unavailable)?;
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
        .reconcile_history(
            native::history::Pass {
                cursor: cursor.as_ref(),
                budget: &profile.budget,
                records: &mut records,
                metadata: &mut retained,
                apply: matches!(storage.phase, Phase::HistoryApply),
            },
            now,
        );
        return match result {
            Ok(report) => serde_json::to_value(report).map_err(|_| github::Error::Invalid),
            Err(e) => Ok(
                json!({"schema":"git-asset.native-history-pass-refusal.experimental-1",
                "ok":false,"failure":Failure::from(&e),"storage_converged":null}),
            ),
        };
    }
    if (storage.policy.allow_copy || !matches!(storage.phase, Phase::Copy)) && !cfg!(windows) {
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
    let mut worker = native::Execution {
        trust: &profile.trust,
        request: &proof,
        policy: &storage.policy,
        enrollment: &profile.enrollment,
        repository: &repository,
        namespace: &mut namespace,
        metadata: &mut metadata,
        limits: &storage.limits,
    };
    let mut source = source.resources(&storage.limits);
    let mut destination = destination.resources(&storage.limits);
    let result = match storage.phase {
        Phase::Copy => worker
            .reconcile_with_actions(
                cursor.as_ref(),
                &profile.budget,
                pending.as_mut(),
                &mut source,
                &mut destination,
                now,
            )
            .and_then(|r| serde_json::to_value(r).map_err(|_| publication::Error::Invalid.into())),
        Phase::EvictionPreview | Phase::EvictionApply => worker
            .reconcile_evictions(
                native::eviction::Pass {
                    cursor: cursor.as_ref(),
                    budget: &profile.budget,
                    pending: pending.as_mut().ok_or(github::Error::Trust)?,
                    apply: matches!(storage.phase, Phase::EvictionApply),
                },
                &mut source,
                &mut destination,
                now,
            )
            .and_then(|r| serde_json::to_value(r).map_err(|_| publication::Error::Invalid.into())),
        Phase::HistoryPreview | Phase::HistoryApply => unreachable!("history dispatched above"),
    };
    match result {
        Ok(report) => Ok(report),
        Err(e) => Ok(json!({"schema": if matches!(storage.phase, Phase::Copy) {
                "git-asset.native-copy-pass-refusal.experimental-1"
            } else { "git-asset.native-eviction-pass-refusal.experimental-1" },
            "ok":false,"failure":Failure::from(&e),"storage_converged":null})),
    }
}
