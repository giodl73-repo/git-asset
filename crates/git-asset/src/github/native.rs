//! One authenticated demanded native copy; no new publisher, journal or pruner.
use super::{Error as AdmissionError, Trust, Verified};
use crate::{
    annex::{control, managed, publication, replication, retirement},
    lifecycle::{self, CommittedSource},
};
use git_asset_core::Digest;
use git_asset_store::{
    Key,
    metadata::MetadataStore,
    protection::Limits,
    source_claims::admission::{self, Namespace, source_use},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub copy: replication::Profile,
    pub source_control: Option<control::Profile>,
    pub destination_control: Option<control::Profile>,
    pub namespace: Digest,
    pub required_place: Key,
    #[serde(default)]
    pub allow_selection: bool,
    #[serde(default)]
    pub allow_copy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<Actions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producers: Option<ProducerRoots>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerRoots {
    pub source: git_asset_core::Object,
    pub destination: git_asset_core::Object,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Actions {
    pub handoff: bool,
    pub eviction: Option<retirement::planning::Profile>,
    #[serde(skip_serializing_if = "disabled")]
    pub backup: bool,
}
fn disabled(value: &bool) -> bool {
    !*value
}
impl Policy {
    fn backup_profile(&self) -> replication::Profile {
        replication::Profile {
            source: self.copy.destination.clone(),
            destination: self.copy.source.clone(),
            source_domain: self.copy.destination_domain.clone(),
            destination_domain: self.copy.source_domain.clone(),
        }
    }
    /// Pure enrollment identity; never opens a pool or discovers locations.
    pub fn id(&self) -> Result<Digest, Error> {
        if self.producers.as_ref().is_some_and(|r| {
            [&r.source, &r.destination]
                .iter()
                .any(|o| o.bytes == 0 || o.bytes > 65536)
        }) {
            return Err(Error::Native(publication::Error::Budget));
        }
        for profile in [&self.copy.source, &self.copy.destination] {
            managed::Reader::new(profile.pool.clone()).map_err(publication::Error::from)?;
        }
        if self.copy.source_domain == self.copy.destination_domain
            || !self
                .copy
                .destination
                .pool
                .binding
                .members()
                .iter()
                .any(|m| m.namespace == self.namespace && m.places.contains(&self.required_place))
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        replication::Copier::controlled(
            self.copy.clone(),
            self.source_control.clone(),
            self.destination_control.clone(),
        )?;
        if let Some(eviction) = self.actions.as_ref().and_then(|a| a.eviction.as_ref()) {
            let working = crate::annex::Reader::new(self.copy.destination.pool.pool.clone())
                .map_err(publication::Error::from)?
                .store();
            let retained = crate::annex::Reader::new(self.copy.source.pool.pool.clone())
                .map_err(publication::Error::from)?
                .store();
            if eviction.source_domain != self.copy.destination_domain
                || eviction.keeper_domain != self.copy.source_domain
                || eviction.source.binding(&working)?
                    != self
                        .destination_control
                        .as_ref()
                        .ok_or(Error::Admission(AdmissionError::Trust))?
                        .binding(&working)?
                || eviction.keeper.binding(&retained)?
                    != self
                        .source_control
                        .as_ref()
                        .ok_or(Error::Admission(AdmissionError::Trust))?
                        .binding(&retained)?
            {
                return Err(Error::Admission(AdmissionError::Trust));
            }
            eviction
                .admit(&working, &retained)
                .map_err(Error::Eviction)?;
        }
        let raw = git_asset_core::identity::canonical_json(&serde_json::json!({
            "schema":"git-asset.trusted-native-copy-policy.experimental-1", "policy":self
        }))
        .map_err(|_| Error::Admission(AdmissionError::Trust))?;
        if raw.len() > 65536 {
            return Err(Error::Admission(AdmissionError::Budget));
        }
        Ok(Digest::blob(&raw))
    }
}
#[derive(Debug)]
pub enum Error {
    Admission(AdmissionError),
    Selection(lifecycle::Error),
    Generation(lifecycle::CurrentUseError),
    Requirements(admission::Error),
    Native(publication::Error),
    Eviction(retirement::Error),
}
impl From<publication::Error> for Error {
    fn from(e: publication::Error) -> Self {
        Self::Native(e)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub policy: Digest,
    pub provenance: super::Provenance,
    pub selection: lifecycle::Status,
    pub copy: replication::Receipt,
    /// Admission after successful effect; never discard its original receipt.
    pub post_copy_hold: Option<AdmissionError>,
    pub storage_converged: Option<bool>,
}
pub struct Copied {
    report: Report,
    ready: replication::Ready,
    _generation: source_use::Use,
}
impl Copied {
    pub fn report(&self) -> &Report {
        &self.report
    }
    pub fn retained(&mut self) -> &mut replication::Ready {
        &mut self.ready
    }
}
/// Use the explicit native source adapter for annex projects. The existing
/// CommittedSource contract still requires complete exact capture and IO limits.
/// No IO occurs merely by constructing this value.
pub struct Execution<'a, R: CommittedSource + ?Sized> {
    pub trust: &'a Trust,
    pub request: &'a Verified,
    pub policy: &'a Policy,
    pub enrollment: &'a lifecycle::Enrollment,
    pub repository: &'a R,
    pub namespace: &'a mut Namespace,
    pub metadata: &'a mut MetadataStore,
    pub limits: &'a Limits,
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    pub fn copy(
        &mut self,
        plan: &replication::Plan,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
        now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Copied, Error> {
        self.copy_direction(false, plan, source, destination, now)
    }
    pub fn backup(
        &mut self,
        plan: &replication::Plan,
        working: &mut publication::Resources<'_>,
        retained: &mut publication::Resources<'_>,
        now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Copied, Error> {
        self.copy_direction(true, plan, working, retained, now)
    }
    fn copy_direction(
        &mut self,
        backup: bool,
        plan: &replication::Plan,
        source: &mut publication::Resources<'_>,
        destination: &mut publication::Resources<'_>,
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Copied, Error> {
        let allowed = self.policy.allow_copy
            && (!backup || self.policy.actions.as_ref().is_some_and(|a| a.backup));
        let ids = self.admission(allowed, &mut now)?;
        let profile = if backup {
            self.policy.backup_profile()
        } else {
            self.policy.copy.clone()
        };
        let (source_control, destination_control) = if backup {
            (
                self.policy.destination_control.clone(),
                self.policy.source_control.clone(),
            )
        } else {
            (
                self.policy.source_control.clone(),
                self.policy.destination_control.clone(),
            )
        };
        let mut copier =
            replication::Copier::controlled(profile.clone(), source_control, destination_control)?;
        // Validate the frozen per-object plan against the enrolled pair before
        // any selection mutation; do not substitute a new publication attempt.
        let expected = match plan.source_snapshot() {
            Some(snapshot) => replication::Plan::from_snapshot(
                &profile,
                plan.source().clone(),
                plan.destination().clone(),
                snapshot.clone(),
            )?,
            None => {
                replication::Plan::new(&profile, plan.source().clone(), plan.destination().clone())?
            }
        };
        if expected != *plan {
            return Err(Error::Native(publication::Error::Conflict));
        }
        let mut state = self.select(ids)?;
        let nodes = self
            .metadata
            .finished_reader()
            .map_err(|_| Error::Selection(lifecycle::Error::Metadata))?;
        let required = state
            .namespace
            .requirements(&nodes, self.limits)
            .map_err(Error::Requirements)?;
        let object = &plan.destination().candidate().object;
        if !required
            .entries
            .iter()
            .any(|a| &a.object == object && a.places.contains(&self.policy.required_place))
        {
            return Err(Error::Admission(AdmissionError::Denied));
        }
        self.revalidate(&state, &mut now)?;
        let ready = copier.apply(plan, source, destination)?;
        let hold = self.post(&mut state, &mut now);
        Ok(Copied {
            report: Report {
                schema: if backup {
                    "git-asset.trusted-native-backup-report.experimental-1"
                } else {
                    "git-asset.trusted-native-copy-report.experimental-1"
                },
                policy: state.policy,
                provenance: self.request.provenance().clone(),
                selection: state.status,
                copy: ready.receipt().clone(),
                post_copy_hold: hold,
                storage_converged: None,
            },
            ready,
            _generation: state.generation,
        })
    }
}

struct Selected {
    policy: Digest,
    selection: Digest,
    status: lifecycle::Status,
    generation: source_use::Use,
    namespace: Namespace,
}
impl<R: CommittedSource + ?Sized> Execution<'_, R> {
    fn admission(
        &self,
        allowed: bool,
        now: &mut impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<(Digest, Digest), Error> {
        let policy = self.policy.id()?;
        let selection = self.enrollment.id().map_err(Error::Selection)?;
        let admit = |time| {
            self.request
                .admit(time, &selection, &policy)
                .map_err(Error::Admission)
        };
        if self.request.provenance().trust != self.trust.id().map_err(Error::Admission)?
            || self.policy.namespace != self.enrollment.namespace
            || self.namespace.identity() != &self.enrollment.namespace
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        admit(now().map_err(Error::Admission)?)?;
        if !self.policy.allow_selection || !allowed {
            return Err(Error::Admission(AdmissionError::Disabled));
        }
        self.limits
            .validate()
            .map_err(|_| Error::Admission(AdmissionError::Budget))?;
        Ok((policy, selection))
    }
    fn select(&mut self, ids: (Digest, Digest)) -> Result<Selected, Error> {
        let member = self
            .policy
            .copy
            .destination
            .pool
            .members
            .iter()
            .find(|m| m.minimum.identity() == &self.enrollment.namespace)
            .ok_or(Error::Admission(AdmissionError::Trust))?;
        let mut namespace =
            Namespace::open(&member.root, &member.minimum).map_err(Error::Requirements)?;
        if !namespace.supports_source_use() {
            return Err(Error::Generation(lifecycle::CurrentUseError::Protection(
                source_use::UseError::Unsupported,
            )));
        }
        let mut lifecycle = lifecycle::Execution {
            enrollment: self.enrollment,
            repository: self.repository,
            namespace: &mut namespace,
            metadata: self.metadata,
            allow_selection: true,
        };
        let plan = lifecycle
            .plan(Some(self.request.commit()))
            .map_err(Error::Selection)?;
        let status = lifecycle.apply(&plan).map_err(Error::Selection)?;
        let generation = lifecycle.hold_current(&plan).map_err(Error::Generation)?;
        Ok(Selected {
            policy: ids.0,
            selection: ids.1,
            status,
            generation,
            namespace,
        })
    }
    fn revalidate(
        &self,
        state: &Selected,
        now: &mut impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<(), Error> {
        let time = now().map_err(Error::Admission)?;
        self.request
            .admit(time, &state.selection, &state.policy)
            .map_err(Error::Admission)?;
        if self
            .repository
            .named_commit(&self.enrollment.ref_name)
            .map_err(Error::Selection)?
            != *self.request.commit()
        {
            return Err(Error::Admission(AdmissionError::Stale));
        }
        // The monotonic token lifetime is checked again after repository IO.
        self.request
            .admit(time, &state.selection, &state.policy)
            .map_err(Error::Admission)
    }
    fn post(
        &self,
        state: &mut Selected,
        now: &mut impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Option<AdmissionError> {
        let mut hold = now()
            .and_then(|time| self.request.admit(time, &state.selection, &state.policy))
            .err();
        state.status.observed_ref_current =
            match self.repository.named_commit(&self.enrollment.ref_name) {
                Ok(c) => {
                    let current = &c == self.request.commit();
                    if !current && hold.is_none() {
                        hold = Some(AdmissionError::Stale);
                    }
                    Some(current)
                }
                Err(_) => {
                    if hold.is_none() {
                        hold = Some(AdmissionError::Unavailable);
                    }
                    None
                }
            };
        hold
    }
}
mod mutations;
pub use mutations::Mutation;
pub mod batch;
