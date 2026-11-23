//! Exact archive members into an enrolled retained index before working release.
use super::*;
use crate::annex::{
    pending::{self, Action, Operation},
    producer_index::{Index, update as roots},
};
use std::collections::BTreeMap;

pub struct Target<'a> {
    pub namespace: &'a mut Namespace,
    pub actions: &'a mut Namespace,
}
pub(super) struct Pass<'a> {
    pub profile: &'a archive::Profile,
    pub archive: &'a archive::Loaded,
    pub namespace: &'a mut Namespace,
    pub actions: &'a mut Namespace,
    pub nodes: &'a mut MetadataStore,
    pub observer: &'a mut Namespace,
    pub current: &'a Version,
    pub started: Instant,
    pub budget: &'a crate::github::placement::Budget,
}
#[derive(Serialize)]
pub struct Report {
    pub examined: usize,
    pub indexed: usize,
    pub saved: Option<Saved>,
    pub registration: Option<roots::Report>,
    pub lookup: batch::Metrics,
    pub index_limits_per_row: Option<Limits>,
    pub hold: Option<batch::Failure>,
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    pub(super) fn index_archive(
        &mut self,
        pass: Pass<'_>,
        now: &mut impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Report {
        let mut out = Report {
            examined: 0,
            indexed: 0,
            saved: None,
            registration: None,
            lookup: batch::Metrics::default(),
            index_limits_per_row: None,
            hold: None,
        };
        let result = (|| -> Result<(), Error> {
            let heads = self
                .policy
                .producer_heads
                .as_ref()
                .ok_or(publication::Error::Unsupported)?;
            let head = &heads.source;
            out.index_limits_per_row = Some(head.profile.limits.clone());
            if pass.namespace.identity() != head.minimum.identity()
                || pass.nodes.checkpoint().identity() != &head.profile.scope.metadata
            {
                return Err(Error::Admission(AdmissionError::Trust));
            }
            let mut source_nodes = self
                .metadata
                .finished_reader()
                .map_err(|_| publication::Error::NeedsRecovery)?;
            let (working, _working_guard) = heads.destination.hold(&source_nodes)?;
            let scope = self.pending_scope()?;
            let (members, metrics) = pass.archive.members(pass.profile, pass.nodes)?;
            out.lookup.add(metrics);
            let mut distinct = BTreeMap::new();
            for member in members.into_values() {
                if distinct.get(&member.object.sha256).is_some_and(
                    |old: &crate::annex::snapshot::Snapshot| old.object != member.object,
                ) {
                    return Err(publication::Error::Integrity.into());
                }
                distinct
                    .entry(member.object.sha256.clone())
                    .or_insert(member);
            }
            for member in distinct.into_values() {
                let gate =
                    |worker: &Self, observer: &mut Namespace, clock: &mut _| -> Result<(), Error> {
                        worker.history_gate(observer, pass.current, clock)?;
                        if pass.started.elapsed() >= Duration::from_millis(pass.budget.millis) {
                            return Err(Error::Admission(AdmissionError::Budget));
                        }
                        Ok(())
                    };
                gate(self, pass.observer, now)?;
                let reader = pass
                    .nodes
                    .finished_reader()
                    .map_err(|_| publication::Error::NeedsRecovery)?;
                let held = head.profile.hold_current(pass.namespace, &reader)?;
                let version = held.version().clone();
                let (retained, m) = Index::from_root(version.selection.root.clone()).get(
                    &member.object,
                    pass.nodes,
                    out.lookup.remaining(&pass.profile.limits)?,
                )?;
                out.lookup.add(m);
                let retained = retained.ok_or(publication::Error::Missing)?;
                out.examined += 1;
                if retained.retained.as_ref() == Some(&member) {
                    continue;
                }
                let (original, m) = Index::from_root(working.version.selection.root.clone()).get(
                    &member.object,
                    &mut source_nodes,
                    out.lookup.remaining(&pass.profile.limits)?,
                )?;
                out.lookup.add(m);
                let original = original.ok_or(publication::Error::Missing)?;
                let slot = pending::archive_slot(&original.producer.id()?, pass.archive.saved())?;
                let existing = pending::find(
                    &scope,
                    pending::Phase::ArchiveIndex,
                    &member.object,
                    &slot,
                    pass.actions,
                    self.metadata,
                    self.limits,
                )?;
                let saved = if let Some(old) = existing {
                    old.saved.clone()
                } else {
                    let mut updated = retained.clone();
                    updated.retained = Some(member.clone());
                    let index = roots::prepare(
                        &head.profile,
                        Some(&version),
                        &[roots::Change {
                            expected_record: Some(retained.metadata_object()?),
                            record: updated.clone(),
                        }],
                        pass.namespace,
                        pass.nodes,
                    )?
                    .ok_or(publication::Error::Conflict)?;
                    gate(self, pass.observer, now)?;
                    let action = Action {
                        operation: Operation::ArchiveIndex(Box::new(pending::ArchiveIndex {
                            archive: pass.archive.saved().clone(),
                            path: member.path.clone(),
                            index,
                            record: updated,
                        })),
                        working: original.producer,
                        retained: retained.producer.clone(),
                    };
                    pending::save(
                        &scope,
                        self.policy.id()?,
                        self.request.commit().as_str(),
                        &action,
                        pass.actions,
                        self.metadata,
                        self.limits,
                    )?
                };
                out.saved = Some(saved.clone());
                let loaded =
                    pending::reopen(&scope, &saved, pass.actions, self.metadata, self.limits)?;
                let Operation::ArchiveIndex(body) = &loaded.action.operation else {
                    return Err(publication::Error::Integrity.into());
                };
                if body.archive.namespace != pass.archive.saved().namespace
                    || body.archive.source != pass.archive.saved().source
                    || body.record.retained.as_ref() != Some(&member)
                {
                    return Err(publication::Error::Integrity.into());
                }
                if let Some(receipt) =
                    roots::status(&head.profile, &body.index, pass.namespace, pass.nodes)?
                {
                    out.registration = Some(receipt);
                    return Err(publication::Error::Stale.into());
                }
                let mut expected = retained.clone();
                expected.retained = Some(member);
                if body.record != expected {
                    return Err(publication::Error::Integrity.into());
                }
                gate(self, pass.observer, now)?;
                drop(held);
                let mut refused = None;
                let receipt = roots::apply(
                    &head.profile,
                    &body.index,
                    pass.namespace,
                    pass.nodes,
                    || {
                        gate(self, pass.observer, now).map_err(|e| {
                            refused = Some(e);
                            publication::Error::Denied
                        })
                    },
                )
                .map_err(|e| refused.take().unwrap_or(Error::Native(e)))?;
                let current = receipt.current;
                out.registration = Some(receipt);
                out.indexed += 1;
                if let Some(e) = refused {
                    return Err(e);
                }
                if current != Some(true) {
                    return Err(publication::Error::NeedsRecovery.into());
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            out.hold = Some(batch::Failure::from(&error));
        }
        out
    }
}
