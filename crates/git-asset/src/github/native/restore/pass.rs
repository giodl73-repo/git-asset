//! Bounded current-demand scheduler; originals precede effects and index CAS.
use super::*;
use crate::annex::{
    pending::{Action, RestoreAction, RestoreIndexAction},
    producer_index::Index,
};
use git_asset_core::tree::Tree;
use git_asset_store::placement::Cursor;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};

pub struct Pass<'a, 'b> {
    pub cursor: Option<&'a Cursor>,
    pub budget: &'a crate::github::placement::Budget,
    pub pending: &'a mut Namespace,
    pub context: &'a mut native::Context<'b>,
    pub target: &'a mut index::Target<'b>,
    pub apply: bool,
    /// Explicitly admitted successor preparation; ordinary apply never replaces a plan.
    pub replan: bool,
}
#[derive(Serialize)]
pub struct Outcome {
    pub object: Object,
    pub state: &'static str,
    pub native_saved: Option<Saved>,
    pub index_saved: Option<Saved>,
    pub native: Option<Mutation<publication::Receipt>>,
    pub registration: Option<Mutation<crate::annex::producer_index::update::Report>>,
    pub hold: Option<batch::Failure>,
}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub policy: Digest,
    pub provenance: crate::github::Provenance,
    pub selection: lifecycle::Status,
    pub basis: Digest,
    pub producer: producers::Pin,
    pub next: Option<Cursor>,
    pub fresh_pass_required: bool,
    pub outcomes: Vec<Outcome>,
    pub prospective_bytes: u64,
    pub lookup: batch::Metrics,
    pub post_effect_hold: Option<AdmissionError>,
    pub storage_converged: Option<bool>,
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    pub fn reconcile_restores(
        &mut self,
        pass: Pass<'_, '_>,
        destination: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Report, Error> {
        pass.budget.validate().map_err(Error::Admission)?;
        if pass.replan && !pass.apply {
            return Err(Error::Admission(AdmissionError::Invalid));
        }
        let started = Instant::now();
        let ids = self.admission(self.restore_allowed(), &mut now)?;
        self.restore_binding(pass.context, destination)?;
        let head = &self
            .policy
            .producer_heads
            .as_ref()
            .ok_or(publication::Error::Unsupported)?
            .destination;
        same(pass.target.profile, &head.profile)?;
        if pass.target.namespace.identity() != head.minimum.identity()
            || pass.target.nodes.checkpoint().identity() != &head.profile.scope.metadata
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        if pass.apply && !head.profile.scope.allow_store {
            return Err(Error::Admission(AdmissionError::Disabled));
        }
        let (pin, guard) = head.hold(destination.metadata)?;
        let mut guard = Some(guard);
        let scope = self.pending_scope()?;
        let mut selected = self.select(ids)?;
        let mut nodes = self
            .metadata
            .finished_reader()
            .map_err(|_| publication::Error::NeedsRecovery)?;
        let current = selected
            .namespace
            .source(&self.enrollment.source)
            .map_err(publication::Error::from)?
            .and_then(|s| s.current)
            .ok_or(publication::Error::ProtectionLost)?;
        let mut lookup = batch::Metrics::default();
        let (entries, metrics) = Tree::from_root(current.selection.root)
            .entries(&mut nodes, lookup.remaining(self.limits)?)
            .map_err(publication::Error::from)?;
        lookup.add(metrics);
        let mut required = BTreeMap::new();
        for object in entries.into_values() {
            if required
                .insert(object.sha256.clone(), object.clone())
                .is_some_and(|old| old != object)
            {
                return Err(publication::Error::Integrity.into());
            }
        }
        let demand = selected
            .namespace
            .requirements(&nodes, self.limits)
            .map_err(Error::Requirements)?;
        let wanted: BTreeSet<_> = demand
            .entries
            .iter()
            .filter(|a| a.places.contains(&self.policy.required_place))
            .map(|a| a.object.clone().sha256)
            .collect();
        let basis = Digest::blob(
            &identity::canonical_json(&serde_json::json!([
                "git-asset.native-restore-pass.experimental-1",
                selected.policy,
                self.request.commit().as_str(),
                selected.status.source_revision,
                pin,
            ]))
            .map_err(|_| publication::Error::Invalid)?,
        );
        if pass.cursor.is_some_and(|c| c.basis != basis) {
            return Err(Error::Admission(AdmissionError::Stale));
        }
        let mut report = Report {
            schema: "git-asset.native-restore-pass-report.experimental-1",
            policy: selected.policy.clone(),
            provenance: self.request.provenance().clone(),
            selection: selected.status.clone(),
            basis: basis.clone(),
            producer: pin,
            next: None,
            fresh_pass_required: false,
            outcomes: Vec::new(),
            prospective_bytes: 0,
            lookup,
            post_effect_hold: None,
            storage_converged: None,
        };
        let mut after = pass.cursor.map(|c| c.after.clone());
        let start_after = after.clone();
        let mut attempted = false;
        for (hash, object) in required
            .iter()
            .filter(|(hash, _)| start_after.as_ref().is_none_or(|a| *hash > a))
            .take(pass.budget.objects.min(128))
        {
            let mut out = Outcome {
                object: object.clone(),
                state: "held",
                native_saved: None,
                index_saved: None,
                native: None,
                registration: None,
                hold: None,
            };
            let result = (|| -> Result<(), Error> {
                if started.elapsed() >= Duration::from_millis(pass.budget.millis) {
                    return Err(publication::Error::Budget.into());
                }
                self.revalidate(&selected, &mut now)?;
                if !wanted.contains(hash) {
                    out.state = "not_required";
                    return Ok(());
                }
                let next = report
                    .prospective_bytes
                    .checked_add(object.bytes)
                    .ok_or(publication::Error::Budget)?;
                if next > pass.budget.stream_bytes {
                    return Err(publication::Error::Budget.into());
                }
                report.prospective_bytes = next;
                let (row, metrics) =
                    Index::from_root(report.producer.version.selection.root.clone()).get(
                        object,
                        pass.target.nodes,
                        report.lookup.remaining(self.limits)?,
                    )?;
                report.lookup.add(metrics);
                let row = row.ok_or(publication::Error::Missing)?;
                let mut eviction = None;
                let mut checked = BTreeSet::new();
                for publication in row.publications() {
                    let id = publication.id()?;
                    if !checked.insert(id.clone()) {
                        continue;
                    }
                    if let Some(found) = pending::find(
                        &scope,
                        pending::Phase::Eviction,
                        object,
                        &id,
                        pass.pending,
                        self.metadata,
                        self.limits,
                    )? {
                        if eviction.is_some() {
                            return Err(publication::Error::Conflict.into());
                        }
                        eviction = Some(found);
                    }
                }
                let Some(eviction) = eviction else {
                    out.state = "not_applicable";
                    return Ok(());
                };
                let Operation::Eviction(retired) = &eviction.action.operation else {
                    return Err(publication::Error::Integrity.into());
                };
                let original = native::Original {
                    eviction: retired,
                    publication: &eviction.action.working,
                };
                let producer = eviction.action.working.id()?;
                let mut saved = pending::find(
                    &scope,
                    pending::Phase::Restore,
                    object,
                    &producer,
                    pass.pending,
                    self.metadata,
                    self.limits,
                )?;
                let indexed = pending::find(
                    &scope,
                    pending::Phase::RestoreIndex,
                    object,
                    &producer,
                    pass.pending,
                    self.metadata,
                    self.limits,
                )?;
                if !pass.apply {
                    if let Some(indexed) = indexed {
                        out.index_saved = Some(indexed.saved);
                        out.state = "saved_index";
                    } else if let Some(saved) = saved {
                        out.native_saved = Some(saved.saved);
                        out.state = "saved_native";
                    } else {
                        let status = native::inspect(pass.context, original)?;
                        if !matches!(
                            status.observed,
                            retirement::planning::execution::Observation::ObservedAbsent
                        ) {
                            return Err(publication::Error::Conflict.into());
                        }
                        out.state = "eligible";
                    }
                    return Ok(());
                }
                attempted = true;
                if saved.is_none() && indexed.is_none() {
                    let id = Digest::blob(
                        &identity::canonical_json(&serde_json::json!([
                            producer,
                            retired.id().map_err(Error::Eviction)?
                        ]))
                        .map_err(|_| publication::Error::Invalid)?,
                    );
                    let key = |prefix: &str| {
                        Key::parse(&format!("{prefix}-{}", id.as_str()))
                            .map_err(|_| publication::Error::Invalid)
                    };
                    let plan = native::prepare(
                        pass.context,
                        original,
                        native::Operations {
                            native: key("restore-native")?,
                            control: key("restore-control")?,
                            source: original.publication.retention_source().clone(),
                        },
                    )?;
                    self.revalidate(&selected, &mut now)?;
                    let action = Action {
                        operation: Operation::Restore(Box::new(RestoreAction {
                            eviction: (**retired).clone(),
                            restore: plan,
                        })),
                        working: eviction.action.working.clone(),
                        retained: eviction.action.retained.clone(),
                    };
                    let handle = pending::save(
                        &scope,
                        selected.policy.clone(),
                        self.request.commit().as_str(),
                        &action,
                        pass.pending,
                        self.metadata,
                        self.limits,
                    )?;
                    saved = Some(pending::reopen(
                        &scope,
                        &handle,
                        pass.pending,
                        self.metadata,
                        self.limits,
                    )?);
                }
                out.native_saved = saved.as_ref().map(|s| s.saved.clone());
                if let Some(indexed) = indexed {
                    out.index_saved = Some(indexed.saved.clone());
                    let original_handle = indexed.saved.clone();
                    drop(indexed);
                    let handle = if pass.replan {
                        let replanned = self.replan_restore_index_saved(
                            IndexReplan {
                                saved: &original_handle,
                                baseline: &index::Baseline {
                                    version: report.producer.version.clone(),
                                    record: row.metadata_object()?,
                                },
                            },
                            pass.pending,
                            pass.context,
                            pass.target,
                            destination,
                            &mut now,
                        )?;
                        report.post_effect_hold = replanned.post_effect_hold;
                        out.index_saved = Some(replanned.result.clone());
                        if report.post_effect_hold.is_some() {
                            out.state = "replanned_held";
                            return Ok(());
                        }
                        replanned.result
                    } else {
                        original_handle
                    };
                    drop(guard.take());
                    let receipt = self.register_restore_saved(
                        &handle,
                        pass.pending,
                        pass.context,
                        pass.target,
                        destination,
                        &mut now,
                    )?;
                    report.post_effect_hold = receipt.post_effect_hold;
                    out.registration = Some(receipt);
                } else {
                    let saved = saved.ok_or(publication::Error::NeedsRecovery)?;
                    let Operation::Restore(body) = &saved.action.operation else {
                        return Err(publication::Error::Integrity.into());
                    };
                    let ready = self.restore_saved(
                        &saved.saved,
                        pass.pending,
                        pass.context,
                        destination,
                        &mut now,
                    )?;
                    report.post_effect_hold = ready.report.post_effect_hold;
                    out.native = Some(ready.report);
                    if report.post_effect_hold.is_some() {
                        out.state = "restored_held";
                        return Ok(());
                    }
                    // Durable ingest protection remains. The index SDK opens its
                    // own fresh guard and holds it through atomic root adoption.
                    drop(ready.ready);
                    self.revalidate(&selected, &mut now)?;
                    let plan = index::prepare(
                        pass.target,
                        &index::Baseline {
                            version: report.producer.version.clone(),
                            record: row.metadata_object()?,
                        },
                        pass.context,
                        original,
                        &body.restore,
                        destination,
                    )?;
                    let action = Action {
                        operation: Operation::RestoreIndex(Box::new(RestoreIndexAction {
                            original: body.as_ref().clone(),
                            index: plan,
                        })),
                        working: saved.action.working.clone(),
                        retained: saved.action.retained.clone(),
                    };
                    let handle = pending::save(
                        &scope,
                        selected.policy.clone(),
                        self.request.commit().as_str(),
                        &action,
                        pass.pending,
                        self.metadata,
                        self.limits,
                    )?;
                    out.index_saved = Some(handle.clone());
                    drop(guard.take());
                    let receipt = self.register_restore_saved(
                        &handle,
                        pass.pending,
                        pass.context,
                        pass.target,
                        destination,
                        &mut now,
                    )?;
                    report.post_effect_hold = receipt.post_effect_hold;
                    out.registration = Some(receipt);
                }
                report.fresh_pass_required = true;
                match out.registration.as_ref().and_then(|r| r.result.current) {
                    Some(true) => {}
                    Some(false) => return Err(publication::Error::Stale.into()),
                    None => return Err(publication::Error::NeedsRecovery.into()),
                }
                out.state = "indexed";
                Ok(())
            })();
            if let Err(error) = result {
                out.hold = Some(batch::Failure::from(&error));
                report.fresh_pass_required = true;
            }
            after = Some(hash.clone());
            report.outcomes.push(out);
            if attempted || report.post_effect_hold.is_some() {
                break;
            }
        }
        report.post_effect_hold = report
            .post_effect_hold
            .or_else(|| self.post(&mut selected, &mut now));
        report.fresh_pass_required |= report.post_effect_hold.is_some();
        if !report.fresh_pass_required
            && after
                .as_ref()
                .is_some_and(|a| required.keys().any(|k| k > a))
        {
            report.next = after.map(|after| Cursor { basis, after });
        }
        report.selection = selected.status;
        Ok(report)
    }
}
