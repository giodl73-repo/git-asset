//! Authenticated bounded history metadata passes. Never native media eviction.
use super::*;
use crate::annex::{history as archive, pending::Saved};
use git_asset_core::identity;
use git_asset_store::{placement::Cursor, source_claims::Version};
use std::time::{Duration, Instant};

pub struct Pass<'a> {
    pub cursor: Option<&'a Cursor>,
    pub budget: &'a super::super::placement::Budget,
    pub records: &'a mut Namespace,
    pub metadata: &'a mut MetadataStore,
    pub apply: bool,
}
#[derive(Serialize)]
pub struct Outcome {
    pub origin: Version,
    pub archive: Digest,
    pub plan: Option<Digest>,
    pub saved: Option<Saved>,
    pub release: Option<archive::Report>,
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
    pub media_stream_bytes: u64,
    pub metadata_limits_per_root: Limits,
    pub post_effect_hold: Option<batch::Failure>,
    pub storage_converged: Option<bool>,
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    fn history_gate(
        &self,
        observer: &mut Namespace,
        version: &Version,
        now: &mut impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<(), Error> {
        self.admission(true, now)?;
        if self
            .repository
            .named_commit(&self.enrollment.ref_name)
            .map_err(Error::Selection)?
            != *self.request.commit()
        {
            return Err(Error::Selection(lifecycle::Error::Stale));
        }
        let state = observer
            .source(&self.enrollment.source)
            .map_err(publication::Error::from)?
            .ok_or(publication::Error::ProtectionLost)?;
        if state.owner != self.enrollment.owner
            || state.kind.as_ref() != Some(&self.enrollment.class)
            || state.pending.is_some()
            || state.current.as_ref() != Some(version)
        {
            return Err(Error::Admission(AdmissionError::Stale));
        }
        self.admission(true, now)?;
        Ok(())
    }
    pub fn reconcile_history(
        &mut self,
        pass: Pass<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Report, Error> {
        pass.budget.validate().map_err(Error::Admission)?;
        let start = Instant::now();
        let p = self
            .policy
            .actions
            .as_ref()
            .and_then(|a| a.history.clone())
            .ok_or(publication::Error::Unsupported)?;
        let ids = self.admission(true, &mut now)?;
        if p.working.namespace != self.enrollment.namespace
            || p.working.source != self.enrollment.source
            || p.working.owner != self.enrollment.owner
            || p.working.class != self.enrollment.class
            || pass.records.identity() != &p.records.namespace
            || pass.metadata.checkpoint().identity() != &p.records.metadata
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        let member = self
            .policy
            .copy
            .source
            .pool
            .members
            .iter()
            .find(|m| m.minimum.identity() == &p.target.namespace)
            .ok_or(Error::Admission(AdmissionError::Trust))?;
        let mut media =
            Namespace::open(&member.root, &member.minimum).map_err(publication::Error::from)?;
        let working_member = self
            .policy
            .copy
            .destination
            .pool
            .members
            .iter()
            .find(|m| m.minimum.identity() == &self.enrollment.namespace)
            .ok_or(Error::Admission(AdmissionError::Trust))?;
        let mut observer = Namespace::open(&working_member.root, &working_member.minimum)
            .map_err(publication::Error::from)?;
        let selected = self.select(ids)?;
        let Selected {
            policy,
            mut status,
            generation,
            namespace: mut working,
            ..
        } = selected;
        let state = working
            .source(&self.enrollment.source)
            .map_err(publication::Error::from)?
            .ok_or(publication::Error::Missing)?;
        let current = state.current.ok_or(publication::Error::ProtectionLost)?;
        let mut versions = state.predecessors;
        versions.sort_by_key(|v| v.revision);
        let basis = Digest::blob(
            &identity::canonical_json(&serde_json::json!([
                "git-asset.native-history-pass.experimental-1",
                policy,
                self.request.commit().as_str(),
                current
            ]))
            .map_err(|_| publication::Error::Invalid)?,
        );
        if pass.cursor.is_some_and(|c| c.basis != basis) {
            return Err(Error::Admission(AdmissionError::Stale));
        }
        let mut after_revision = 0;
        if let Some(cursor) = pass.cursor {
            if let Some(v) = versions
                .iter()
                .find(|v| p.archive_id(v).is_ok_and(|id| id == cursor.after))
            {
                after_revision = v.revision;
            } else {
                let old = archive::find_archive(
                    &p,
                    &cursor.after,
                    archive::Destinations {
                        media: &mut media,
                        records: pass.records,
                    },
                    pass.metadata,
                )?
                .ok_or(Error::Admission(AdmissionError::Stale))?;
                if old.plan().origin().revision >= current.revision {
                    return Err(Error::Admission(AdmissionError::Stale));
                }
                if old.release_status(&p, &mut working)?.is_none() {
                    return Err(Error::Admission(AdmissionError::Stale));
                }
                // The cursor names positive saved work, never inferred missing history.
                after_revision = old.plan().origin().revision;
            }
        }
        drop(generation);
        let input = self
            .metadata
            .finished_reader()
            .map_err(|_| publication::Error::NeedsRecovery)?;
        let mut report = Report {
            schema: "git-asset.native-history-pass-report.experimental-1",
            policy,
            provenance: self.request.provenance().clone(),
            selection: status.clone(),
            basis: basis.clone(),
            next: None,
            outcomes: Vec::new(),
            media_stream_bytes: 0,
            metadata_limits_per_root: p.limits.clone(),
            post_effect_hold: None,
            storage_converged: None,
        };
        let mut after = pass.cursor.map(|c| c.after.clone());
        let remaining: Vec<_> = versions
            .into_iter()
            .filter(|v| v.revision > after_revision)
            .collect();
        for origin in remaining.iter() {
            if report.outcomes.len() >= pass.budget.objects.min(32) {
                report.next = after.map(|after| Cursor {
                    basis: basis.clone(),
                    after,
                });
                break;
            }
            let mut outcome = Outcome {
                origin: origin.clone(),
                archive: p.archive_id(origin)?,
                plan: None,
                saved: None,
                release: None,
                state: "held",
                hold: None,
            };
            let result = (|| -> Result<(), Error> {
                self.history_gate(&mut observer, &current, &mut now)?;
                if start.elapsed() >= Duration::from_millis(pass.budget.millis) {
                    return Err(Error::Admission(AdmissionError::Budget));
                }
                let plan = archive::Plan::capture(&p, &mut working, origin)?;
                outcome.plan = Some(plan.id()?);
                if !pass.apply {
                    outcome.state = "planned";
                    return Ok(());
                }
                let mut refused = None;
                let loaded = archive::publish(
                    &p,
                    &plan,
                    &mut working,
                    archive::Destinations {
                        media: &mut media,
                        records: pass.records,
                    },
                    &input,
                    pass.metadata,
                    |_| {
                        self.history_gate(&mut observer, &current, &mut now)
                            .map_err(|e| {
                                refused = Some(e);
                                publication::Error::Denied
                            })?;
                        if start.elapsed() >= Duration::from_millis(pass.budget.millis) {
                            return Err(publication::Error::Budget);
                        }
                        Ok(())
                    },
                );
                if let Some(e) = refused {
                    return Err(e);
                }
                let mut loaded = loaded?;
                outcome.saved = Some(loaded.saved().clone());
                outcome.plan = Some(loaded.plan().id()?);
                let revision = working
                    .source(&self.enrollment.source)
                    .map_err(publication::Error::from)?
                    .ok_or(publication::Error::Missing)?
                    .revision;
                if p.allow_release && revision > loaded.plan().expected_revision() {
                    let refreshed = archive::refresh(
                        &p,
                        loaded,
                        &mut working,
                        archive::Destinations {
                            media: &mut media,
                            records: pass.records,
                        },
                        pass.metadata,
                        |_| {
                            self.history_gate(&mut observer, &current, &mut now)
                                .map_err(|e| {
                                    refused = Some(e);
                                    publication::Error::Denied
                                })?;
                            if start.elapsed() >= Duration::from_millis(pass.budget.millis) {
                                return Err(publication::Error::Budget);
                            }
                            Ok(())
                        },
                    );
                    if let Some(e) = refused {
                        return Err(e);
                    }
                    loaded = refreshed?;
                    outcome.saved = Some(loaded.saved().clone());
                    outcome.plan = Some(loaded.plan().id()?);
                }
                if let Some(e) = refused {
                    return Err(e);
                }
                outcome.state = "archived";
                if p.allow_release {
                    let mut read = pass
                        .metadata
                        .finished_reader()
                        .map_err(|_| publication::Error::NeedsRecovery)?;
                    let released =
                        loaded.release(&p, &mut working, &mut media, (&input, &mut read), || {
                            self.history_gate(&mut observer, &current, &mut now)
                                .map_err(|e| {
                                    refused = Some(e);
                                    publication::Error::Denied
                                })?;
                            if start.elapsed() >= Duration::from_millis(pass.budget.millis) {
                                return Err(publication::Error::Budget);
                            }
                            Ok(())
                        });
                    if let Some(e) = refused {
                        return Err(e);
                    }
                    outcome.release = Some(released?);
                    outcome.state = "released";
                }
                self.history_gate(&mut observer, &current, &mut now)?;
                Ok(())
            })();
            if let Err(e) = result {
                outcome.hold = Some(batch::Failure::from(&e));
            }
            let held = outcome.hold.is_some();
            let released = outcome.release.is_some();
            let id = outcome.archive.clone();
            report.outcomes.push(outcome);
            if held {
                if released {
                    after = Some(id);
                }
                report.next = after.map(|after| Cursor {
                    basis: basis.clone(),
                    after,
                });
                break;
            }
            after = Some(id);
        }
        let final_check = (|| -> Result<(), Error> {
            let state = working
                .source(&self.enrollment.source)
                .map_err(publication::Error::from)?
                .ok_or(publication::Error::Missing)?;
            status.source_revision = state.revision;
            status.desired_current = state.current.as_ref() == Some(&current);
            status.observed_ref_current = Some(
                self.repository
                    .named_commit(&self.enrollment.ref_name)
                    .map_err(Error::Selection)?
                    == *self.request.commit(),
            );
            self.history_gate(&mut observer, &current, &mut now)
        })();
        if let Err(e) = final_check {
            report.post_effect_hold = Some(batch::Failure::from(&e));
        }
        report.selection = status;
        Ok(report)
    }
}
