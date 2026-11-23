//! Record completed recipes before archival changes their derivation context.
use super::*;
use crate::annex::{
    pending::{self, Action, Operation, Registration, RegistrationSide, Saved},
    producer_index::{Record, update as roots},
};
use git_asset_core::identity;

pub struct Target<'a> {
    pub side: RegistrationSide,
    pub namespace: &'a mut Namespace,
    pub nodes: &'a mut MetadataStore,
}
pub struct Pass<'a> {
    pub cursor: Option<&'a Cursor>,
    pub budget: &'a crate::github::placement::Budget,
    pub pending: &'a mut Namespace,
    pub target: Target<'a>,
    pub apply: bool,
    pub replan: bool,
}
#[derive(Serialize)]
pub struct Registered {
    pub object: Object,
    pub state: &'static str,
    pub saved: Option<Saved>,
    pub registration: Option<roots::Report>,
    pub hold: Option<Failure>,
}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub policy: Digest,
    pub selection: lifecycle::Status,
    pub provenance: crate::github::Provenance,
    pub side: RegistrationSide,
    pub basis: Digest,
    pub producer_heads: producers::Pins,
    pub outcomes: Vec<Registered>,
    pub next: Option<Cursor>,
    pub fresh_pass_required: bool,
    pub lookup: Metrics,
    pub media_stream_bytes: u64,
    pub post_effect_hold: Option<AdmissionError>,
    pub storage_converged: Option<bool>,
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    pub fn reconcile_registrations(
        &mut self,
        pass: Pass<'_>,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Report, Error> {
        pass.budget.validate().map_err(Error::Admission)?;
        let started = Instant::now();
        let ids = self.admission(
            self.policy.actions.as_ref().is_some_and(|a| a.registration),
            &mut now,
        )?;
        let heads = self
            .policy
            .producer_heads
            .as_ref()
            .ok_or(publication::Error::Unsupported)?;
        let head = match pass.target.side {
            RegistrationSide::Working => &heads.destination,
            RegistrationSide::Retained => &heads.source,
        };
        if pass.target.namespace.identity() != head.minimum.identity()
            || pass.target.nodes.checkpoint().identity() != &head.profile.scope.metadata
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        let mut resolved = Some(self.resolve_producers(source.metadata, destination.metadata)?);
        let pins = resolved
            .as_ref()
            .unwrap()
            .pins
            .clone()
            .ok_or(publication::Error::Unsupported)?;
        let root_pair = resolved.as_ref().unwrap().roots.clone();
        let pin = match pass.target.side {
            RegistrationSide::Working => &pins.destination,
            RegistrationSide::Retained => &pins.source,
        };
        let scope = self.pending_scope()?;
        let mut selected = self.select(ids)?;
        let reader = self
            .metadata
            .finished_reader()
            .map_err(|_| publication::Error::NeedsRecovery)?;
        let required = selected
            .namespace
            .requirements(&reader, self.limits)
            .map_err(Error::Requirements)?;
        let mut objects = std::collections::BTreeMap::new();
        for a in required
            .entries
            .into_iter()
            .filter(|a| a.places.contains(&self.policy.required_place))
        {
            if objects
                .insert(a.object.sha256.clone(), a.object.clone())
                .is_some_and(|old| old != a.object)
            {
                return Err(publication::Error::Integrity.into());
            }
        }
        let basis = Digest::blob(
            &identity::canonical_json(&serde_json::json!([
                "git-asset.native-registration-pass.experimental-1",
                selected.policy,
                self.request.commit().as_str(),
                selected.status.source_revision,
                pins,
                pass.target.side
            ]))
            .map_err(|_| publication::Error::Invalid)?,
        );
        if pass
            .cursor
            .is_some_and(|c| c.basis != basis || !objects.contains_key(&c.after))
        {
            return Err(Error::Admission(AdmissionError::Stale));
        }
        let mut report = Report {
            schema: "git-asset.native-registration-report.experimental-1",
            policy: selected.policy.clone(),
            selection: selected.status.clone(),
            provenance: self.request.provenance().clone(),
            side: pass.target.side,
            basis: basis.clone(),
            producer_heads: pins.clone(),
            outcomes: Vec::new(),
            next: None,
            fresh_pass_required: false,
            lookup: Metrics::default(),
            media_stream_bytes: 0,
            post_effect_hold: None,
            storage_converged: None,
        };
        let start_after = pass.cursor.map(|c| c.after.clone());
        let mut after = start_after.clone();
        for (hash, object) in objects
            .iter()
            .filter(|(h, _)| start_after.as_ref().is_none_or(|a| *h > a))
            .take(pass.budget.objects.min(128))
        {
            let mut out = Registered {
                object: object.clone(),
                state: "held",
                saved: None,
                registration: None,
                hold: None,
            };
            let result = (|| -> Result<(), Error> {
                if started.elapsed() >= Duration::from_millis(pass.budget.millis) {
                    return Err(publication::Error::Budget.into());
                }
                self.revalidate(&selected, &mut now)?;
                let (working, m) = Index::from_root(root_pair.destination.clone()).get(
                    object,
                    destination.metadata,
                    report.lookup.remaining(self.limits)?,
                )?;
                report.lookup.add(m);
                let working = working.ok_or(publication::Error::Missing)?;
                let (retained, m) = Index::from_root(root_pair.source.clone()).get(
                    object,
                    source.metadata,
                    report.lookup.remaining(self.limits)?,
                )?;
                report.lookup.add(m);
                let retained = retained.ok_or(publication::Error::Missing)?;
                let row = match pass.target.side {
                    RegistrationSide::Working => &working,
                    RegistrationSide::Retained => &retained,
                };
                let phase = match pass.target.side {
                    RegistrationSide::Working => pending::Phase::RegisterWorking,
                    RegistrationSide::Retained => pending::Phase::RegisterRetained,
                };
                let original = pending::find(
                    &scope,
                    phase,
                    object,
                    &working.producer.id()?,
                    pass.pending,
                    self.metadata,
                    self.limits,
                )?;
                let saved = if let Some(original) = original {
                    out.saved = Some(original.saved.clone());
                    if !pass.apply {
                        out.state = "saved";
                        return Ok(());
                    }
                    original.saved.clone()
                } else {
                    if pass.replan {
                        out.state = "not_applicable";
                        return Ok(());
                    }
                    let mut diagnostic = super::Outcome {
                        object: object.clone(),
                        plan: None,
                        publication: None,
                        backup: None,
                        retained_publication: None,
                        handoff: None,
                        state: "held",
                        hold: None,
                    };
                    let declared = self.declared_plan(
                        &root_pair,
                        object,
                        source,
                        destination,
                        &mut report.lookup,
                        &mut diagnostic,
                    )?;
                    let mut candidates = row.publications().collect::<Vec<_>>();
                    candidates.push(match pass.target.side {
                        RegistrationSide::Working => declared.forward.destination(),
                        RegistrationSide::Retained => declared.forward.source(),
                    });
                    let recipe = match pass.target.side {
                        RegistrationSide::Working => completed(&candidates, destination)?,
                        RegistrationSide::Retained => completed(&candidates, source)?,
                    }
                    .ok_or(publication::Error::NeedsRecovery)?;
                    if recipe == row.producer || row.publication.as_ref() == Some(&recipe) {
                        out.state = "recorded";
                        return Ok(());
                    }
                    let updated = recorded(row, recipe, pass.target.side, source, destination)?;
                    if !pass.apply {
                        out.state = "eligible";
                        return Ok(());
                    }
                    let plan = roots::prepare(
                        &head.profile,
                        Some(&pin.version),
                        &[roots::Change {
                            expected_record: Some(row.metadata_object()?),
                            record: updated.clone(),
                        }],
                        pass.target.namespace,
                        pass.target.nodes,
                    )?
                    .ok_or(publication::Error::Conflict)?;
                    self.revalidate(&selected, &mut now)?;
                    let action = Action {
                        operation: Operation::Registration(Box::new(Registration {
                            side: pass.target.side,
                            index: plan,
                            record: updated,
                        })),
                        working: working.producer.clone(),
                        retained: retained.producer.clone(),
                    };
                    let saved = pending::save(
                        &scope,
                        selected.policy.clone(),
                        self.request.commit().as_str(),
                        &action,
                        pass.pending,
                        self.metadata,
                        self.limits,
                    )?;
                    out.saved = Some(saved.clone());
                    saved
                };
                let loaded =
                    pending::reopen(&scope, &saved, pass.pending, self.metadata, self.limits)?;
                let Operation::Registration(body) = &loaded.action.operation else {
                    return Err(publication::Error::Integrity.into());
                };
                if body.side != pass.target.side {
                    return Err(publication::Error::Integrity.into());
                }
                if let Some(receipt) = roots::status(
                    &head.profile,
                    &body.index,
                    pass.target.namespace,
                    pass.target.nodes,
                )? {
                    out.registration = Some(receipt);
                    if row.producer != body.record.producer
                        || row.publication != body.record.publication
                    {
                        return Err(publication::Error::Stale.into());
                    }
                    out.state = "recorded";
                    return Ok(());
                }
                let expected = body.index.expected().ok_or(publication::Error::Integrity)?;
                let (old, m) = Index::from_root(expected.selection.root.clone()).get(
                    object,
                    pass.target.nodes,
                    report.lookup.remaining(self.limits)?,
                )?;
                report.lookup.add(m);
                let old = old.ok_or(publication::Error::Missing)?;
                let checked = recorded(
                    &old,
                    body.record
                        .publication
                        .clone()
                        .ok_or(publication::Error::Integrity)?,
                    pass.target.side,
                    source,
                    destination,
                )?;
                if checked != body.record {
                    return Err(publication::Error::Integrity.into());
                }
                self.revalidate(&selected, &mut now)?;
                drop(resolved.take());
                let mut index = body.index.clone();
                drop(loaded);
                if pass.replan && index.expected() != Some(&pin.version) {
                    let successor = roots::prepare_successor(
                        &head.profile,
                        &index,
                        pass.target.namespace,
                        pass.target.nodes,
                    )?
                    .ok_or(publication::Error::Stale)?;
                    index = successor.plan().clone();
                    self.revalidate(&selected, &mut now)?;
                    out.saved = Some(pending::save_ordinary_index_successor(
                        &scope,
                        selected.policy.clone(),
                        self.request.commit().as_str(),
                        pending::OrdinaryIndexSuccessor {
                            expected: &saved,
                            proposal: successor,
                        },
                        pass.pending,
                        self.metadata,
                        self.limits,
                    )?);
                }
                let mut refused = None;
                let receipt = roots::apply(
                    &head.profile,
                    &index,
                    pass.target.namespace,
                    pass.target.nodes,
                    || {
                        self.revalidate(&selected, &mut now).map_err(|error| {
                            refused = Some(error);
                            publication::Error::Denied
                        })
                    },
                )
                .map_err(|error| refused.take().unwrap_or(Error::Native(error)))?;
                report.fresh_pass_required = true;
                let current = receipt.current;
                out.registration = Some(receipt);
                if let Some(error) = refused {
                    out.hold = Some(Failure::from(&error));
                    out.state = "recorded_held";
                    return Ok(());
                }
                if current != Some(true) {
                    return Err(publication::Error::NeedsRecovery.into());
                }
                out.state = "recorded";
                Ok(())
            })();
            if let Err(e) = result {
                out.hold = Some(Failure::from(&e));
                report.fresh_pass_required = true;
            }
            if out
                .registration
                .as_ref()
                .is_some_and(|m| m.post_effect_hold.is_some())
            {
                report.fresh_pass_required = true;
            }
            after = Some(hash.clone());
            report.outcomes.push(out);
            if report.fresh_pass_required {
                break;
            }
        }
        if let Some(error) = self.post(&mut selected, &mut now) {
            report.fresh_pass_required = true;
            report.post_effect_hold = Some(error);
        }
        if !report.fresh_pass_required
            && after
                .as_ref()
                .is_some_and(|a| objects.keys().any(|h| h > a))
        {
            report.next = after.map(|after| Cursor { basis, after });
        }
        report.selection = selected.status;
        Ok(report)
    }
}

fn recorded(
    row: &Record,
    recipe: publication::Plan,
    side: RegistrationSide,
    source: &mut publication::Resources<'_>,
    destination: &mut publication::Resources<'_>,
) -> Result<Record, Error> {
    Ok(match side {
        RegistrationSide::Working => {
            row.with_completed_publication(recipe, destination.witness, destination.ledger)?
        }
        RegistrationSide::Retained => {
            row.with_completed_publication(recipe, source.witness, source.ledger)?
        }
    })
}
