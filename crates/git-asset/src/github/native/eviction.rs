//! Bounded registered-candidate eviction, never a payload census or final deletion.
use super::*;
use crate::annex::{pending, producer_index::Index};
use git_asset_core::{
    Object, identity,
    tree::{Tree, TreeLimits},
};
use git_asset_store::placement::Cursor;
use retirement::planning::{self, Controller, Nodes, Producer, Sources};
use std::time::{Duration, Instant};

pub struct Pass<'a> {
    pub cursor: Option<&'a Cursor>,
    pub budget: &'a super::super::placement::Budget,
    pub pending: &'a mut Namespace,
    pub apply: bool,
}
#[derive(Serialize)]
pub struct Outcome {
    pub object: Object,
    pub publication: Digest,
    pub retained_publication: Option<Digest>,
    pub backup: Option<Digest>,
    pub plan: Option<Digest>,
    pub handoff: Option<Durable<publication::handoff::Report>>,
    pub eviction: Option<Durable<planning::Report>>,
    pub state: &'static str,
    pub hold: Option<batch::Failure>,
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
    pub lookup: batch::Metrics,
    pub post_effect_hold: Option<AdmissionError>,
    pub storage_converged: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer_heads: Option<super::producers::Pins>,
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    fn apply_registered_eviction(
        &mut self,
        plan: &planning::Plan,
        working: &publication::Plan,
        retained: &publication::Plan,
        namespace: &mut Namespace,
        resources: (
            &mut publication::Resources<'_>,
            &mut publication::Resources<'_>,
        ),
        now: &mut impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Durable<planning::Report>, Error> {
        let (source, destination) = resources;
        let policy = self.policy;
        self.evict_durable(
            plan,
            namespace,
            Sources {
                source: Producer {
                    profile: &policy.copy.destination,
                    plan: working,
                    witness: destination.witness,
                    ledger: destination.ledger,
                },
                keeper: Producer {
                    profile: &policy.copy.source,
                    plan: retained,
                    witness: source.witness,
                    ledger: source.ledger,
                },
            },
            Nodes {
                source: &*destination.metadata,
                keeper: &mut *source.metadata,
                limits: self.limits,
                tree_limits: TreeLimits {
                    max_entries: self.limits.bindings,
                    max_reads: self.limits.reads,
                    max_read_bytes: self.limits.read_bytes,
                    ..TreeLimits::default()
                },
            },
            now,
        )
    }
    /// Fixed enrolled index only. Preview never releases claims or saves/applies plans.
    pub fn reconcile_evictions(
        &mut self,
        pass: Pass<'_>,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Report, Error> {
        pass.budget.validate().map_err(Error::Admission)?;
        let start = Instant::now();
        let profile = self
            .policy
            .actions
            .as_ref()
            .and_then(|a| a.eviction.as_ref());
        let ids = self.admission(profile.is_some_and(|p| p.allow_evict), &mut now)?;
        let profile = profile.ok_or(publication::Error::Unsupported)?;
        let scope = self.pending_scope()?;
        if pass.pending.identity() != &scope.namespace
            || self.metadata.checkpoint().identity() != &scope.metadata
        {
            return Err(publication::Error::Denied.into());
        }
        if !cfg!(windows) || profile.runtime.is_none() {
            return Err(publication::Error::Unsupported.into());
        }
        let producers = self.resolve_producers(source.metadata, destination.metadata)?;
        let roots = &producers.roots;
        let mut selected = self.select(ids)?;
        let mut nodes = self
            .metadata
            .finished_reader()
            .map_err(|_| Error::Selection(lifecycle::Error::Metadata))?;
        let current = selected
            .namespace
            .source(&self.enrollment.source)
            .map_err(publication::Error::from)?
            .and_then(|s| s.current)
            .ok_or(publication::Error::ProtectionLost)?;
        let mut lookup = batch::Metrics::default();
        let (required, metrics) = Tree::from_root(current.selection.root)
            .entries(&mut nodes, lookup.remaining(self.limits)?)
            .map_err(publication::Error::from)?;
        lookup.add(metrics);
        let requires_working = selected
            .namespace
            .policy()
            .classes
            .get(&self.enrollment.class)
            .ok_or(publication::Error::Unsupported)?
            .current
            .contains(&self.policy.required_place);
        let basis = producers.basis(Digest::blob(
            &identity::canonical_json(&serde_json::json!([
                "git-asset.native-eviction-pass.experimental-1",
                selected.policy,
                self.request.commit().as_str(),
                selected.status.source_revision,
                roots.destination
            ]))
            .map_err(|_| publication::Error::Invalid)?,
        ))?;
        if pass.cursor.is_some_and(|c| c.basis != basis) {
            return Err(Error::Admission(AdmissionError::Stale));
        }
        let cursor = pass.cursor.map(|c| crate::annex::producer_index::Cursor {
            root: roots.destination.clone(),
            after: c.after.clone(),
        });
        let page = Index::from_root(roots.destination.clone()).page(
            cursor.as_ref(),
            pass.budget.objects.min(128),
            destination.metadata,
            lookup.remaining(self.limits)?,
        )?;
        lookup.add(page.metrics);
        let mut report = Report {
            schema: "git-asset.native-eviction-pass-report.experimental-1",
            policy: selected.policy.clone(),
            provenance: self.request.provenance().clone(),
            selection: selected.status.clone(),
            basis: basis.clone(),
            next: None,
            outcomes: Vec::new(),
            prospective_bytes: 0,
            lookup,
            post_effect_hold: None,
            storage_converged: None,
            producer_heads: producers.pins.clone(),
        };
        let mut after = pass.cursor.map(|c| c.after.clone());
        for row in page.records {
            let mut outcome = Outcome {
                object: row.object.clone(),
                publication: row.publication.as_ref().unwrap_or(&row.producer).id()?,
                retained_publication: None,
                backup: None,
                plan: None,
                handoff: None,
                eviction: None,
                state: "held",
                hold: None,
            };
            let result = (|| -> Result<(), Error> {
                if start.elapsed() >= Duration::from_millis(pass.budget.millis) {
                    return Err(Error::Admission(AdmissionError::Budget));
                }
                self.revalidate(&selected, &mut now)?;
                if requires_working && required.values().any(|r| r == &row.object) {
                    outcome.state = "required";
                    return Ok(());
                }
                let next_bytes = report
                    .prospective_bytes
                    .checked_add(row.object.bytes)
                    .ok_or(publication::Error::Budget)?;
                if next_bytes > pass.budget.stream_bytes {
                    return Err(publication::Error::Budget.into());
                }
                report.prospective_bytes = next_bytes;
                // Direct-ingress originals remain discoverable even with keeper payload offline.
                let mut saved = pending::find(
                    &scope,
                    pending::Phase::Eviction,
                    &row.object,
                    &outcome.publication,
                    pass.pending,
                    self.metadata,
                    self.limits,
                )?;
                let mut working = row.publication.as_ref().unwrap_or(&row.producer).clone();
                let mut retained = None;
                let mut snapshot = None;
                let mut grant = None;
                if saved.is_none() {
                    let (a, metrics) = Index::from_root(roots.source.clone()).get(
                        &row.object,
                        source.metadata,
                        report.lookup.remaining(self.limits)?,
                    )?;
                    report.lookup.add(metrics);
                    let a = a.ok_or(publication::Error::Missing)?;
                    let keeper =
                        match batch::completed(&a.publications().collect::<Vec<_>>(), source) {
                            Ok(Some(p)) => p,
                            Ok(None) | Err(Error::Native(publication::Error::Conflict)) => {
                                let reverse = match row.retained.clone() {
                                    Some(s) => replication::Plan::from_snapshot(
                                        &self.policy.backup_profile(),
                                        row.producer.clone(),
                                        a.producer.clone(),
                                        s,
                                    )?,
                                    None => replication::Plan::new(
                                        &self.policy.backup_profile(),
                                        row.producer.clone(),
                                        a.producer.clone(),
                                    )?,
                                };
                                outcome.backup = Some(reverse.id()?);
                                let mut candidates = a.publications().collect::<Vec<_>>();
                                candidates.push(reverse.destination());
                                batch::completed(&candidates, source)?
                                    .ok_or(publication::Error::Missing)?
                            }
                            Err(e) => return Err(e),
                        };
                    let forward = match &a.retained {
                        Some(s) => replication::Plan::from_snapshot(
                            &self.policy.copy,
                            keeper.clone(),
                            row.producer.clone(),
                            s.clone(),
                        )?,
                        None => replication::Plan::new(
                            &self.policy.copy,
                            keeper.clone(),
                            row.producer.clone(),
                        )?,
                    };
                    let mut candidates = row.publications().collect::<Vec<_>>();
                    candidates.push(forward.destination());
                    working = batch::completed(&candidates, destination)?
                        .ok_or(publication::Error::Missing)?;
                    outcome.publication = working.id()?;
                    saved = pending::find(
                        &scope,
                        pending::Phase::Eviction,
                        &row.object,
                        &outcome.publication,
                        pass.pending,
                        self.metadata,
                        self.limits,
                    )?;
                    if saved.is_none() {
                        snapshot = Some(match a.retained {
                            Some(s) => s,
                            None => keeper.retained_snapshot(source.namespace)?,
                        });
                        grant = a.control_retention;
                    }
                    retained = Some(keeper);
                }
                if let Some(loaded) = saved {
                    let pending::Operation::Eviction(plan) = &loaded.action.operation else {
                        return Err(publication::Error::Integrity.into());
                    };
                    outcome.plan = Some(plan.id().map_err(Error::Eviction)?);
                    outcome.retained_publication = Some(loaded.action.retained.id()?);
                    outcome.eviction = Some(if pass.apply {
                        self.apply_registered_eviction(
                            plan,
                            &loaded.action.working,
                            &loaded.action.retained,
                            pass.pending,
                            (source, destination),
                            &mut now,
                        )?
                    } else {
                        Durable {
                            saved: loaded.saved.clone(),
                            mutation: None,
                            failure: None,
                        }
                    });
                    outcome.state = "saved";
                } else {
                    let keeper = retained.ok_or(publication::Error::Missing)?;
                    let snapshot = snapshot.ok_or(publication::Error::Missing)?;
                    let grant = grant.ok_or(publication::Error::Missing)?;
                    outcome.retained_publication = Some(keeper.id()?);
                    if pass.apply && self.policy.actions.as_ref().is_some_and(|a| a.handoff) {
                        let handed = self.handoff_durable(
                            HandoffRequest {
                                working: &working,
                                retained: &keeper,
                                snapshot: snapshot.clone(),
                            },
                            pass.pending,
                            destination,
                            source,
                            &mut now,
                        )?;
                        outcome.hold = handed.failure.clone();
                        if let Some(m) = &handed.mutation {
                            report.post_effect_hold = m.post_effect_hold;
                        }
                        outcome.handoff = Some(handed);
                        if outcome.hold.is_some() || report.post_effect_hold.is_some() {
                            return Ok(());
                        }
                    }
                    if start.elapsed() >= Duration::from_millis(pass.budget.millis) {
                        return Err(Error::Admission(AdmissionError::Budget));
                    }
                    self.revalidate(&selected, &mut now)?;
                    let operation = Key::parse(&format!(
                        "evict-{}",
                        Digest::blob(
                            &identity::canonical_json(&serde_json::json!([
                                selected.policy,
                                working.id()?,
                                keeper.id()?,
                                snapshot.id()?
                            ]))
                            .map_err(|_| publication::Error::Invalid)?
                        )
                        .as_str()
                    ))
                    .map_err(|_| publication::Error::Invalid)?;
                    let policy = self.policy;
                    let plan = Controller::new(profile).plan(
                        planning::Request {
                            operation,
                            snapshot,
                            grant,
                        },
                        Sources {
                            source: Producer {
                                profile: &policy.copy.destination,
                                plan: &working,
                                witness: destination.witness,
                                ledger: destination.ledger,
                            },
                            keeper: Producer {
                                profile: &policy.copy.source,
                                plan: &keeper,
                                witness: source.witness,
                                ledger: source.ledger,
                            },
                        },
                        Nodes {
                            source: &*destination.metadata,
                            keeper: &mut *source.metadata,
                            limits: self.limits,
                            tree_limits: TreeLimits {
                                max_entries: self.limits.bindings,
                                max_reads: self.limits.reads,
                                max_read_bytes: self.limits.read_bytes,
                                ..TreeLimits::default()
                            },
                        },
                    );
                    let plan = match plan {
                        Ok(p) => p,
                        Err(retirement::Error::Protected) => {
                            outcome.state = "protected";
                            return Ok(());
                        }
                        Err(e) => return Err(Error::Eviction(e)),
                    };
                    outcome.plan = Some(plan.id().map_err(Error::Eviction)?);
                    outcome.state = "planned";
                    if pass.apply {
                        outcome.eviction = Some(self.apply_registered_eviction(
                            &plan,
                            &working,
                            &keeper,
                            pass.pending,
                            (source, destination),
                            &mut now,
                        )?);
                    }
                }
                if let Some(durable) = &outcome.eviction {
                    outcome.hold = durable.failure.clone();
                    if let Some(m) = &durable.mutation {
                        report.post_effect_hold = m.post_effect_hold;
                        selected.status.observed_ref_current = m.selection.observed_ref_current;
                        outcome.state = if m.result.historical {
                            "reconciled"
                        } else {
                            "evicted"
                        };
                    }
                }
                Ok(())
            })();
            if let Err(e) = result {
                outcome.hold = Some(batch::Failure::from(&e));
            }
            let held = outcome.hold.is_some();
            let digest = outcome.object.sha256.clone();
            report.outcomes.push(outcome);
            if report.post_effect_hold.is_none() {
                report.post_effect_hold = self.post(&mut selected, &mut now);
            }
            if held || report.post_effect_hold.is_some() {
                report.next = after.map(|after| Cursor {
                    basis: basis.clone(),
                    after,
                });
                break;
            }
            after = Some(digest);
            report.next = page.next.as_ref().map(|_| Cursor {
                basis: basis.clone(),
                after: after.clone().unwrap(),
            });
        }
        if report.post_effect_hold.is_none() {
            report.post_effect_hold = self.post(&mut selected, &mut now);
        }
        report.selection = selected.status;
        Ok(report)
    }
}
