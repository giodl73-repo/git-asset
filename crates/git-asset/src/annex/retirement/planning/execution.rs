//! Shared native executor behind the explicit experimental controller.
use super::*;
use git_asset_core::tree::NodeStore;
use git_asset_store::{
    ledger::{Checkpoint, JournalRef},
    local::LocalStore,
    upload_witness::{self, Document, Packet},
};

pub(super) fn authority(pool: &crate::annex::Profile, expected: &Digest) -> Result<()> {
    let output = process::run(
        pool,
        false,
        &[
            "config",
            "--local",
            "--no-includes",
            "--get-all",
            "gitasset.evictionpolicy",
        ],
    )
    .map_err(|_| publication::Error::Stale)?;
    if crate::annex::line(&output.stdout)? != expected.as_str() {
        return Err(publication::Error::Stale.into());
    }
    Ok(())
}
fn store_error(error: git_asset_store::Error) -> Error {
    publication::Error::from(accounted::AccountedError::Store(error)).into()
}
fn ledger_error(error: git_asset_store::ledger::LedgerError) -> Error {
    publication::Error::from(accounted::AccountedError::Ledger(error)).into()
}
fn open_ledger(path: &std::path::Path, minimum: &Checkpoint) -> Result<CapacityLedger> {
    CapacityLedger::open(path, minimum).map_err(ledger_error)
}
fn metadata_error(error: metadata::Error) -> Error {
    match error {
        metadata::Error::Store(e) => store_error(e),
        metadata::Error::Ledger(e) => ledger_error(e),
        metadata::Error::Protection(e) => protection_error(e),
        metadata::Error::Retention(e) => retention_error(e),
        metadata::Error::InvalidPlan | metadata::Error::WrongAuthorization => {
            publication::Error::Stale.into()
        }
        _ => publication::Error::NeedsRecovery.into(),
    }
}
struct Effect {
    store: LocalStore,
    ledger: CapacityLedger,
    job: JournalRef,
    packet: Packet,
}
impl Effect {
    fn open(runtime: &Runtime, plan: &Plan) -> Result<Self> {
        runtime.id()?;
        let profile = &runtime.effect;
        let store = LocalStore::open(&profile.root).map_err(store_error)?;
        if store.identity() != &profile.store || store.supports_retirement() {
            return Err(publication::Error::Stale.into());
        }
        let ledger = open_ledger(&profile.ledger, &profile.minimum)?;
        let id = plan.id()?;
        let documents = ["intent", "completion"].map(|phase| -> Result<Document> {
            Ok(Document {
                operation: Key::parse(&format!("native-{phase}-{}", id.as_str()))
                    .map_err(|_| publication::Error::Invalid)?,
                bytes: encode(&serde_json::json!({"schema":"git-asset.native-effect.experimental-1",
                    "plan":id,"metadata":plan.metadata.id().map_err(metadata_error)?,"phase":phase,
                    "command":["dropkey","--force","--json","--json-error-messages"],"object":plan.object()}))?,
            })
        });
        let [intent, complete] = documents;
        let packet = Packet::new(&store, &id, &profile.owner, [intent?, complete?])
            .map_err(publication::Error::from)?;
        Ok(Self {
            store,
            ledger,
            job: profile.job.clone(),
            packet,
        })
    }
    fn state(&mut self) -> Result<upload_witness::State> {
        self.packet
            .inspect(&self.store, &mut self.ledger, &self.job)
            .map_err(publication::Error::from)
            .map_err(Error::from)
    }
}
fn namespaces(runtime: &Runtime) -> Result<Vec<Namespace>> {
    runtime
        .namespaces
        .iter()
        .map(|n| Namespace::open(&n.root, &n.minimum).map_err(Error::from))
        .collect()
}
pub(crate) struct Live<'a, 'read, 'source> {
    pub separation: &'a StorageSeparation<'read, 'source>,
    pub source_read: SourceRead,
    pub source_producer: publication::ProducerCompletion,
    pub keeper_producer: publication::ProducerCompletion,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Observation {
    VerifiedPresent,
    ObservedAbsent,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub plan: Digest,
    pub metadata: metadata::Receipt,
    pub logical_subject_bytes: u64,
    pub source_allocation_before: Option<u64>,
    pub observed: Observation,
    pub historical: bool,
}
fn finish<N: NodeStore>(
    profile: &Profile,
    plan: &Plan,
    runtime: &Runtime,
    nodes: &N,
    limits: &Limits,
    effect: impl FnMut(&metadata::Plan) -> metadata::Result<()>,
) -> Result<metadata::Receipt> {
    let store = profile.source.open_store(&plan.intent.source)?;
    let backup = profile.keeper.open_store(&plan.intent.keeper)?;
    let mut ledger = open_ledger(&profile.source.ledger, &profile.source.minimum)?;
    let mut backup_ledger = open_ledger(&profile.keeper.ledger, &profile.keeper.minimum)?;
    let mut namespaces = namespaces(runtime)?;
    let mut backups = [metadata::Backup {
        store: &backup,
        ledger: &mut backup_ledger,
        enrollment: &plan.metadata.request().keepers[0].enrollment,
    }];
    let mut execution = metadata::Execution {
        store: &store,
        ledger: &mut ledger,
        job: &runtime.job,
        namespaces: &mut namespaces,
        nodes,
        limits,
        backups: &mut backups,
        admitted_context: &plan.metadata.request().context,
        source_enrollment: &plan.metadata.request().source,
    };
    execution
        .apply_with_effect(
            &plan.metadata,
            &plan.metadata.authorize().map_err(metadata_error)?,
            effect,
        )
        .map_err(metadata_error)
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cut {
    None,
    BeforeEffect,
    AfterRemoval,
    AfterCompletion,
    #[cfg(all(test, windows))]
    ExitBeforeEffect,
    #[cfg(all(test, windows))]
    ExitAfterRemoval,
    #[cfg(all(test, windows))]
    ExitAfterCompletion,
}
#[cfg(all(test, windows))]
fn exit_at(cut: Cut, stage: Cut) {
    if cut == Cut::ExitBeforeEffect
        && stage == Cut::BeforeEffect
        && let Some(root) = std::env::var_os("GIT_ASSET_TEST_EVICTION_PAUSE")
    {
        let root = std::path::PathBuf::from(root);
        std::fs::write(root.join("ready"), b"metadata_intent").unwrap();
        let started = std::time::Instant::now();
        while !root.join("release").exists() {
            assert!(
                started.elapsed() < std::time::Duration::from_secs(60),
                "parent release timeout"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    if matches!(
        (cut, stage),
        (Cut::ExitBeforeEffect, Cut::BeforeEffect)
            | (Cut::ExitAfterRemoval, Cut::AfterRemoval)
            | (Cut::ExitAfterCompletion, Cut::AfterCompletion)
    ) {
        std::process::exit(73);
    }
}
/// Internal execution with fixture cuts at metadata intent, removal and completion.
pub(crate) fn apply_hook<N: NodeStore>(
    profile: &Profile,
    plan: &Plan,
    live: Live<'_, '_, '_>,
    nodes: &N,
    limits: &Limits,
    cut: Cut,
) -> Result<Report> {
    plan.id()?;
    if profile.admit(&plan.intent.source, &plan.intent.keeper)? != plan.intent.policy {
        return Err(publication::Error::Stale.into());
    }
    let runtime = profile
        .runtime
        .as_ref()
        .ok_or(publication::Error::Unsupported)?;
    let mut effect = Effect::open(runtime, plan)?;
    let state = effect.state()?;
    if state.intent {
        return Err(publication::Error::Uncertain.into());
    }
    let current = live.separation.plan(
        profile,
        plan.metadata.request().operation.clone(),
        &live.source_read,
        &live.source_producer,
        &live.keeper_producer,
        plan.metadata.request().keepers[0].grant.clone(),
    )?;
    if current.id()? != plan.id()? {
        return Err(publication::Error::Stale.into());
    }
    let allocation = live
        .source_read
        .read
        .with_path(|p| std::fs::File::open(p).and_then(|f| fs2::FileExt::allocated_size(&f)))
        .map_err(publication::Error::from)?
        .map_err(|_| publication::Error::NeedsRecovery)?;
    let _producer = live.source_producer.release_control();
    let _keeper_producer = live.keeper_producer.release_control();
    let mut source_read = Some(live.source_read);
    let separation = live.separation;
    let pool = &separation.keeper._source.profile.pool;
    let key = format!(
        "SHA256-s{}--{}",
        plan.object().bytes,
        plan.object().sha256.as_str()
    );
    let mut effect_error = None;
    let result = finish(profile, plan, runtime, nodes, limits, |_| {
        #[cfg(all(test, windows))]
        exit_at(cut, Cut::BeforeEffect);
        if cut == Cut::BeforeEffect {
            return Err(git_asset_store::Error::NeedsRecovery.into());
        }
        effect
            .packet
            .apply_once(&effect.store, &mut effect.ledger, &effect.job, || {
                let mut execute = || -> Result<_> {
                    separation.revalidate()?;
                    authority(pool, &plan.intent.policy)?;
                    drop(source_read.take());
                    let group = process::run_for_object(
                        pool,
                        plan.object(),
                        true,
                        &[
                            "dropkey",
                            "--force",
                            "--json",
                            "--json-error-messages",
                            &key,
                        ],
                    )?;
                    let response = git_asset_core::identity::parse_json(
                        crate::annex::line(&group.stdout)?.as_bytes(),
                    )
                    .map_err(|_| publication::Error::Uncertain)?;
                    if response["command"] != "dropkey"
                        || response["key"] != key
                        || response["success"] != true
                    {
                        return Err(publication::Error::Uncertain.into());
                    }
                    if !matches!(
                        process::run(pool, true, &["contentlocation", &key]),
                        Err(crate::annex::Error::Missing)
                    ) {
                        return Err(publication::Error::Uncertain.into());
                    }
                    separation.revalidate()?;
                    #[cfg(all(test, windows))]
                    exit_at(cut, Cut::AfterRemoval);
                    if cut == Cut::AfterRemoval {
                        return Err(publication::Error::Uncertain.into());
                    }
                    Ok(group)
                };
                execute().map_err(|_| upload_witness::Error::NeedsRecovery)
            })
            .map_err(|e| {
                effect_error = Some(publication::Error::from(e));
                git_asset_store::Error::NeedsRecovery
            })?;
        #[cfg(all(test, windows))]
        exit_at(cut, Cut::AfterCompletion);
        if cut == Cut::AfterCompletion {
            return Err(git_asset_store::Error::NeedsRecovery.into());
        }
        Ok(())
    });
    let receipt = result.map_err(|e| effect_error.map(Error::from).unwrap_or(e))?;
    Ok(Report {
        plan: plan.id()?,
        metadata: receipt,
        logical_subject_bytes: plan.object().bytes,
        source_allocation_before: Some(allocation),
        observed: Observation::ObservedAbsent,
        historical: false,
    })
}

pub(crate) fn cancel<N: NodeStore>(
    profile: &Profile,
    plan: &Plan,
    source_profile: &publication::Profile,
    source_producer: &publication::Plan,
    nodes: &N,
    limits: &Limits,
) -> Result<Report> {
    plan.id()?;
    let runtime = profile
        .runtime
        .as_ref()
        .ok_or(publication::Error::Unsupported)?;
    if profile.admit(&plan.intent.source, &plan.intent.keeper)? != plan.intent.policy
        || source_producer.id()? != plan.intent.source_producer
    {
        return Err(publication::Error::Stale.into());
    }
    source_producer.check_profile(source_profile)?;
    let mut scope = Scope::pin(source_profile.pool.clone(), plan.object())?;
    authority(&source_profile.pool.pool, &plan.intent.policy)?;
    control::configuration(
        &source_profile.pool.pool,
        &source_profile.pool.binding.id().map_err(protection_error)?,
        Some(&profile.source.binding(&plan.intent.source)?),
    )?;
    let mut effect = Effect::open(runtime, plan)?;
    let early = effect.state()?;
    if early.intent || early.completion {
        return Err(publication::Error::Uncertain.into());
    }
    // Keep surviving bytes pinned through cancellation; the effect state is
    // checked again inside the exact core operation fence below.
    let _bytes = scope.assessor.reader.open_local(plan.object())?;
    let store = profile.source.open_store(&plan.intent.source)?;
    let mut ledger = open_ledger(&profile.source.ledger, &profile.source.minimum)?;
    let mut namespaces = namespaces(runtime)?;
    let mut execution = metadata::Execution {
        store: &store,
        ledger: &mut ledger,
        job: &runtime.job,
        namespaces: &mut namespaces,
        nodes,
        limits,
        backups: &mut [],
        admitted_context: &plan.metadata.request().context,
        source_enrollment: &plan.metadata.request().source,
    };
    let mut denied = false;
    let result = execution.cancel_with_guard(
        &plan.metadata,
        &plan.metadata.authorize().map_err(metadata_error)?,
        |_| {
            let state = effect
                .state()
                .map_err(|_| git_asset_store::Error::NeedsRecovery)?;
            if state.intent || state.completion {
                denied = true;
                return Err(git_asset_store::Error::NeedsRecovery.into());
            }
            Ok(())
        },
    );
    let receipt = result.map_err(|e| {
        if denied {
            publication::Error::Uncertain.into()
        } else {
            metadata_error(e)
        }
    })?;
    Ok(Report {
        plan: plan.id()?,
        metadata: receipt,
        logical_subject_bytes: plan.object().bytes,
        source_allocation_before: None,
        observed: Observation::VerifiedPresent,
        historical: true,
    })
}

/// Full current roster fences without asserting today's absence of demand.
struct Scope {
    assessor: Assessor,
    _configuration: VerifiedRead,
    _directory: std::fs::File,
    _namespaces: Vec<Namespace>,
    _fences: Vec<SubjectFence>,
}
impl Scope {
    fn pin(profile: managed::Profile, object: &Object) -> Result<Self> {
        let mut assessor = Assessor::new(profile)?;
        let (configuration, directory, namespaces) = assessor.pin(object)?;
        let fences = namespaces
            .iter()
            .map(|n| n.fence(object).map_err(Error::from))
            .collect::<Result<_>>()?;
        Ok(Self {
            assessor,
            _configuration: configuration,
            _directory: directory,
            _namespaces: namespaces,
            _fences: fences,
        })
    }
    fn observe(&mut self, object: &Object) -> Result<Observation> {
        if !self.assessor.reader.objects_present()? {
            return Ok(Observation::ObservedAbsent);
        }
        match self.assessor.reader.open_local(object) {
            Ok(_read) => Ok(Observation::VerifiedPresent),
            Err(crate::annex::Error::Missing) => Ok(Observation::ObservedAbsent),
            Err(e) => Err(e.into()),
        }
    }
}
pub(crate) fn resume<N: NodeStore>(
    profile: &Profile,
    plan: &Plan,
    source_profile: &publication::Profile,
    source_producer: &publication::Plan,
    nodes: &N,
    limits: &Limits,
) -> Result<Report> {
    plan.id()?;
    let runtime = profile
        .runtime
        .as_ref()
        .ok_or(publication::Error::Unsupported)?;
    if profile.admit(&plan.intent.source, &plan.intent.keeper)? != plan.intent.policy
        || source_producer.id()? != plan.intent.source_producer
    {
        return Err(publication::Error::Stale.into());
    }
    source_producer.check_profile(source_profile)?;
    let mut scope = Scope::pin(source_profile.pool.clone(), plan.object())?;
    authority(&source_profile.pool.pool, &plan.intent.policy)?;
    control::configuration(
        &source_profile.pool.pool,
        &source_profile.pool.binding.id().map_err(protection_error)?,
        Some(&profile.source.binding(&plan.intent.source)?),
    )?;
    let mut effect = Effect::open(runtime, plan)?;
    let state = effect.state()?;
    let store = profile.source.open_store(&plan.intent.source)?;
    let mut ledger = open_ledger(&profile.source.ledger, &profile.source.minimum)?;
    let status = metadata::status::inspect(&store, &mut ledger, &runtime.job, &plan.metadata)
        .map_err(metadata_error)?;
    let cancelled = matches!(status.phase, metadata::status::Phase::Cancelled);
    if cancelled && !state.intent && !state.completion {
        let observed = scope.observe(plan.object())?;
        let receipt = finish(profile, plan, runtime, nodes, limits, |_| {
            Err(git_asset_store::Error::NeedsRecovery.into())
        })?;
        return Ok(Report {
            plan: plan.id()?,
            metadata: receipt,
            logical_subject_bytes: plan.object().bytes,
            source_allocation_before: None,
            observed,
            historical: true,
        });
    }
    if !state.completion {
        return Err(publication::Error::Uncertain.into());
    }
    let observed = scope.observe(plan.object())?;
    if !matches!(status.phase, metadata::status::Phase::Retired)
        && !matches!(observed, Observation::ObservedAbsent)
    {
        return Err(publication::Error::Uncertain.into());
    }
    let _completion = effect
        .packet
        .apply_once(
            &effect.store,
            &mut effect.ledger,
            &effect.job,
            || -> upload_witness::Result<()> { Err(upload_witness::Error::NeedsRecovery) },
        )
        .map_err(publication::Error::from)?;
    let receipt = finish(profile, plan, runtime, nodes, limits, |_| {
        effect
            .packet
            .reopen(&effect.store, &mut effect.ledger, &effect.job)
            .map_err(|_| git_asset_store::Error::NeedsRecovery)?;
        Ok(())
    })?;
    Ok(Report {
        plan: plan.id()?,
        metadata: receipt,
        logical_subject_bytes: plan.object().bytes,
        source_allocation_before: None,
        observed,
        historical: true,
    })
}

#[derive(Debug, Serialize)]
pub struct Status {
    pub plan: Digest,
    pub effect: &'static str,
    pub metadata_phase: metadata::status::Phase,
    pub observed: Observation,
    pub removal_authority: bool,
}
pub(crate) fn status(
    profile: &Profile,
    plan: &Plan,
    source_profile: &publication::Profile,
    source_producer: &publication::Plan,
) -> Result<Status> {
    plan.id()?;
    let runtime = profile
        .runtime
        .as_ref()
        .ok_or(publication::Error::Unsupported)?;
    if profile.admit(&plan.intent.source, &plan.intent.keeper)? != plan.intent.policy
        || source_producer.id()? != plan.intent.source_producer
    {
        return Err(publication::Error::Stale.into());
    }
    source_producer.check_profile(source_profile)?;
    let mut scope = Scope::pin(source_profile.pool.clone(), plan.object())?;
    authority(&source_profile.pool.pool, &plan.intent.policy)?;
    control::configuration(
        &source_profile.pool.pool,
        &source_profile.pool.binding.id().map_err(protection_error)?,
        Some(&profile.source.binding(&plan.intent.source)?),
    )?;
    let mut effect = Effect::open(runtime, plan)?;
    let state = effect.state()?;
    let store = profile.source.open_store(&plan.intent.source)?;
    let mut ledger = open_ledger(&profile.source.ledger, &profile.source.minimum)?;
    let metadata = metadata::status::inspect(&store, &mut ledger, &runtime.job, &plan.metadata)
        .map_err(metadata_error)?;
    Ok(Status {
        plan: plan.id()?,
        effect: if state.completion {
            if state.settled {
                "completed"
            } else {
                "completion_unsettled"
            }
        } else if state.intent {
            "uncertain"
        } else {
            "not_started"
        },
        metadata_phase: metadata.phase,
        observed: scope.observe(plan.object())?,
        removal_authority: false,
    })
}
