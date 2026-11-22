//! Bounded copy reconciliation over immutable producer declarations; never pruning.
use super::*;
use crate::annex::producer_index::Index;
use git_asset_core::{
    Object,
    tree::{TreeLimits, TreeMetrics},
};
use git_asset_store::placement::Cursor;
use sha2::{Digest as _, Sha256};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize)]
pub struct Failure {
    pub layer: &'static str,
    pub code: String,
}
impl From<&Error> for Failure {
    fn from(error: &Error) -> Self {
        let (layer, code) = match error {
            Error::Admission(e) => (
                "admission",
                serde_json::to_value(e)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned(),
            ),
            Error::Native(e) => ("native", e.code().into()),
            Error::Selection(e) => ("selection", format!("{e:?}").to_ascii_lowercase()),
            Error::Generation(lifecycle::CurrentUseError::Protection(
                source_use::UseError::Unsupported,
            )) => ("generation", "unsupported".into()),
            Error::Generation(lifecycle::CurrentUseError::Selection(e)) => {
                ("generation", format!("{e:?}").to_ascii_lowercase())
            }
            Error::Generation(lifecycle::CurrentUseError::Protection(e)) => (
                "generation",
                match e {
                    source_use::UseError::Admission(e) => {
                        publication::Error::from(e.clone()).code()
                    }
                    source_use::UseError::Owner => "denied",
                    source_use::UseError::Stale => "stale",
                    source_use::UseError::Inactive => "protection_lost",
                    source_use::UseError::Unsupported => "unsupported",
                }
                .into(),
            ),
            Error::Requirements(e) => (
                "requirements",
                publication::Error::from(e.clone()).code().into(),
            ),
            Error::Eviction(e) => ("eviction", e.code().into()),
        };
        Self { layer, code }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_codes_preserve_retry_and_recovery_distinctions() {
        let cases = [
            (Error::Native(publication::Error::Missing), "missing"),
            (Error::Native(publication::Error::Integrity), "corrupt"),
            (Error::Native(publication::Error::Uncertain), "uncertain"),
            (
                Error::Native(publication::Error::Budget),
                "admission_denied",
            ),
            (
                Error::Generation(lifecycle::CurrentUseError::Protection(
                    source_use::UseError::Stale,
                )),
                "stale",
            ),
            (
                Error::Generation(lifecycle::CurrentUseError::Protection(
                    source_use::UseError::Inactive,
                )),
                "protection_lost",
            ),
            (
                Error::Generation(lifecycle::CurrentUseError::Protection(
                    source_use::UseError::Owner,
                )),
                "denied",
            ),
            (
                Error::Generation(lifecycle::CurrentUseError::Protection(
                    source_use::UseError::Admission(admission::Error::Busy),
                )),
                "busy",
            ),
            (
                Error::Eviction(retirement::Error::Operation(publication::Error::Uncertain)),
                "uncertain",
            ),
        ];
        for (error, code) in cases {
            assert_eq!(Failure::from(&error).code, code);
        }
    }
}
#[derive(Serialize)]
pub struct Outcome {
    pub object: Object,
    pub plan: Option<Digest>,
    pub publication: Option<Digest>,
    pub state: &'static str,
    pub hold: Option<Failure>,
}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub policy: Digest,
    pub provenance: super::super::Provenance,
    pub selection: lifecycle::Status,
    pub basis: Digest,
    pub next: Option<Cursor>,
    pub outcomes: Vec<Outcome>,
    pub prospective_bytes: u64,
    pub lookup: Metrics,
    pub post_effect_hold: Option<AdmissionError>,
    pub storage_converged: Option<bool>,
}
#[derive(Default, Serialize)]
pub struct Metrics {
    pub reads: usize,
    pub read_bytes: usize,
}
impl Metrics {
    fn remaining(&self, limits: &Limits) -> Result<TreeLimits, Error> {
        Ok(TreeLimits {
            max_entries: limits.bindings,
            max_reads: limits
                .reads
                .checked_sub(self.reads)
                .ok_or(publication::Error::Budget)?,
            max_read_bytes: limits
                .read_bytes
                .checked_sub(self.read_bytes)
                .ok_or(publication::Error::Budget)?,
            max_writes: 0,
            max_write_bytes: 0,
            ..TreeLimits::default()
        })
    }
    fn add(&mut self, m: TreeMetrics) {
        self.reads += m.reads;
        self.read_bytes += m.read_bytes;
    }
}
fn basis(policy: &Digest, commit: &str, revision: u64, objects: &[Object]) -> Digest {
    let mut h = Sha256::new();
    h.update(b"git-asset.native-copy-pass.experimental-1\0");
    h.update(policy.as_str());
    h.update((commit.len() as u64).to_be_bytes());
    h.update(commit);
    h.update(revision.to_be_bytes());
    h.update((objects.len() as u64).to_be_bytes());
    for o in objects {
        h.update(o.sha256.as_str());
        h.update(o.bytes.to_be_bytes());
    }
    Digest::parse(
        &h.finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
    )
    .unwrap()
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    fn declared_plan(
        &self,
        object: &Object,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
        meter: &mut Metrics,
    ) -> Result<(replication::Plan, publication::Plan), Error> {
        let roots = self
            .policy
            .producers
            .as_ref()
            .ok_or(publication::Error::Unsupported)?;
        let (a, metrics) = Index::from_root(roots.source.clone()).get(
            object,
            source.metadata,
            meter.remaining(self.limits)?,
        )?;
        meter.add(metrics);
        let a = a.ok_or(publication::Error::Missing)?;
        let (b, metrics) = Index::from_root(roots.destination.clone()).get(
            object,
            destination.metadata,
            meter.remaining(self.limits)?,
        )?;
        meter.add(metrics);
        let b = b.ok_or(publication::Error::Missing)?;
        let original = b.producer.clone();
        let plan = match a.retained {
            Some(snapshot) => replication::Plan::from_snapshot(
                &self.policy.copy,
                a.producer,
                b.producer,
                snapshot,
            )?,
            None => replication::Plan::new(&self.policy.copy, a.producer, b.producer)?,
        };
        Ok((plan, original))
    }
    fn present(
        &self,
        plan: &replication::Plan,
        original: &publication::Plan,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
    ) -> Result<Option<Digest>, Error> {
        let mut chosen = None;
        let mut conflict = false;
        for (i, recipe) in [original, plan.destination()].into_iter().enumerate() {
            if i == 1 && original == recipe {
                continue;
            }
            let state = match recipe.progress(destination.witness, destination.ledger) {
                Ok(state) => state,
                Err(publication::Error::Conflict) => {
                    conflict = true;
                    continue;
                }
                Err(e) => return Err(e.into()),
            };
            if state.intent && !state.completion {
                return Err(publication::Error::Uncertain.into());
            }
            if state.completion {
                if !state.settled {
                    return Err(publication::Error::NeedsRecovery.into());
                }
                if chosen.replace(recipe).is_some() {
                    return Err(publication::Error::Conflict.into());
                }
            }
        }
        let Some(recipe) = chosen else {
            return if conflict {
                Err(publication::Error::Conflict.into())
            } else {
                Ok(None)
            };
        };
        let mut working = super::mutations::publisher(
            &self.policy.copy.destination,
            &self.policy.destination_control,
        )?;
        let observed = working.observe_guarded(recipe, destination)?;
        let status = observed.status();
        if status.intent_recorded && !status.historical_completion {
            return Err(publication::Error::Uncertain.into());
        }
        if !status.historical_completion {
            return Err(publication::Error::NeedsRecovery.into());
        }
        if !status.registered || !status.accounting_settled || !status.bytes_verified {
            return Err(publication::Error::NeedsRecovery.into());
        }
        let mut publisher =
            super::mutations::publisher(&self.policy.copy.source, &self.policy.source_control)?;
        // Both byte guards coexist through verification, including after ingest handoff.
        if let Some(snapshot) = plan.source_snapshot() {
            let _retained = publisher.open_retained(plan.source(), snapshot, source)?;
        } else {
            let _retained = publisher.reopen(plan.source(), source)?;
        }
        Ok(Some(recipe.id()?))
    }
    pub fn reconcile(
        &mut self,
        cursor: Option<&Cursor>,
        budget: &super::super::placement::Budget,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Report, Error> {
        budget.validate().map_err(Error::Admission)?;
        let start = Instant::now();
        let ids = self.admission(true, &mut now)?;
        if self.policy.producers.is_none() {
            return Err(publication::Error::Unsupported.into());
        }
        if self.policy.allow_copy && !cfg!(windows) {
            return Err(publication::Error::Unsupported.into());
        }
        let mut selected = self.select(ids)?;
        let nodes = self
            .metadata
            .finished_reader()
            .map_err(|_| Error::Selection(lifecycle::Error::Metadata))?;
        let required = selected
            .namespace
            .requirements(&nodes, self.limits)
            .map_err(Error::Requirements)?;
        let mut objects: Vec<_> = required
            .entries
            .into_iter()
            .filter(|a| a.places.contains(&self.policy.required_place))
            .map(|a| a.object)
            .collect();
        objects.sort_by(|a, b| a.sha256.cmp(&b.sha256));
        let basis = basis(
            &selected.policy,
            self.request.commit().as_str(),
            selected.status.source_revision,
            &objects,
        );
        if cursor.is_some_and(|c| c.basis != basis || !objects.iter().any(|o| o.sha256 == c.after))
        {
            return Err(Error::Admission(AdmissionError::Stale));
        }
        let mut report = Report {
            schema: "git-asset.native-copy-pass-report.experimental-1",
            policy: selected.policy.clone(),
            provenance: self.request.provenance().clone(),
            selection: selected.status.clone(),
            basis: basis.clone(),
            next: None,
            outcomes: Vec::new(),
            prospective_bytes: 0,
            lookup: Metrics::default(),
            post_effect_hold: None,
            storage_converged: None,
        };
        let mut after = cursor.map(|c| c.after.clone());
        for object in objects
            .iter()
            .filter(|o| cursor.is_none_or(|c| o.sha256 > c.after))
        {
            if report.outcomes.len() >= budget.objects.min(128) {
                report.next = after.map(|after| Cursor {
                    basis: basis.clone(),
                    after,
                });
                break;
            }
            let mut outcome = Outcome {
                object: object.clone(),
                plan: None,
                publication: None,
                state: "held",
                hold: None,
            };
            let result = (|| -> Result<(), Error> {
                if start.elapsed() >= Duration::from_millis(budget.millis) {
                    return Err(Error::Admission(AdmissionError::Budget));
                }
                let next = report
                    .prospective_bytes
                    .checked_add(object.bytes)
                    .ok_or(publication::Error::Budget)?;
                if next > budget.stream_bytes {
                    return Err(publication::Error::Budget.into());
                }
                let (plan, original) =
                    self.declared_plan(object, source, destination, &mut report.lookup)?;
                outcome.plan = Some(plan.id()?);
                self.revalidate(&selected, &mut now)?;
                report.prospective_bytes = next;
                if !self.policy.allow_copy {
                    outcome.state = "planned";
                    return Ok(());
                }
                if let Some(publication) = self.present(&plan, &original, source, destination)? {
                    outcome.state = "present";
                    outcome.publication = Some(publication);
                    return Ok(());
                }
                self.revalidate(&selected, &mut now)?;
                let publication = plan.destination().id()?;
                let copied = self.copy(&plan, source, destination, &mut now)?;
                outcome.state = "copied";
                outcome.publication = Some(publication);
                report.post_effect_hold = copied.report().post_copy_hold;
                Ok(())
            })();
            if let Err(e) = result {
                outcome.hold = Some(Failure::from(&e));
            }
            let held = outcome.hold.is_some();
            report.outcomes.push(outcome);
            if report.post_effect_hold.is_none() {
                report.post_effect_hold = self.post(&mut selected, &mut now);
            }
            if held || report.post_effect_hold.is_some() {
                break;
            }
            after = Some(object.sha256.clone());
        }
        if report.post_effect_hold.is_none() {
            report.post_effect_hold = self.post(&mut selected, &mut now);
        }
        report.selection = selected.status;
        Ok(report)
    }
}
