//! One authenticated demanded native copy; no new publisher, journal or pruner.
use super::{Error as AdmissionError, Trust, Verified};
use crate::{
    annex::{control, managed, publication, replication},
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
}
impl Policy {
    /// Pure enrollment identity; never opens a pool or discovers locations.
    pub fn id(&self) -> Result<Digest, Error> {
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
        mut now: impl FnMut() -> Result<u64, AdmissionError>,
    ) -> Result<Copied, Error> {
        let policy = self.policy.id()?;
        let selection = self.enrollment.id().map_err(Error::Selection)?;
        let admit = |time| {
            self.request
                .admit(time, &selection, &policy)
                .map_err(Error::Admission)
        };
        if self.request.provenance().trust != self.trust.id().map_err(Error::Admission)?
            || self.policy.namespace != self.enrollment.namespace
        {
            return Err(Error::Admission(AdmissionError::Trust));
        }
        admit(now().map_err(Error::Admission)?)?;
        if !self.policy.allow_selection || !self.policy.allow_copy {
            return Err(Error::Admission(AdmissionError::Disabled));
        }
        if !self.namespace.supports_source_use() {
            return Err(Error::Generation(lifecycle::CurrentUseError::Protection(
                source_use::UseError::Unsupported,
            )));
        }
        self.limits
            .validate()
            .map_err(|_| Error::Admission(AdmissionError::Budget))?;
        let mut copier = replication::Copier::controlled(
            self.policy.copy.clone(),
            self.policy.source_control.clone(),
            self.policy.destination_control.clone(),
        )?;
        // Validate the frozen per-object plan against the enrolled pair before
        // any selection mutation; do not substitute a new publication attempt.
        let expected = match plan.source_snapshot() {
            Some(snapshot) => replication::Plan::from_snapshot(
                &self.policy.copy,
                plan.source().clone(),
                plan.destination().clone(),
                snapshot.clone(),
            )?,
            None => replication::Plan::new(
                &self.policy.copy,
                plan.source().clone(),
                plan.destination().clone(),
            )?,
        };
        if expected != *plan {
            return Err(Error::Native(publication::Error::Conflict));
        }
        let mut lifecycle = lifecycle::Execution {
            enrollment: self.enrollment,
            repository: self.repository,
            namespace: self.namespace,
            metadata: self.metadata,
            allow_selection: true,
        };
        let selected = lifecycle
            .plan(Some(self.request.commit()))
            .map_err(Error::Selection)?;
        let mut status = lifecycle.apply(&selected).map_err(Error::Selection)?;
        let generation = lifecycle
            .hold_current(&selected)
            .map_err(Error::Generation)?;
        let nodes = self
            .metadata
            .finished_reader()
            .map_err(|_| Error::Selection(lifecycle::Error::Metadata))?;
        let required = self
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
        if self
            .repository
            .named_commit(&self.enrollment.ref_name)
            .map_err(Error::Selection)?
            != *self.request.commit()
        {
            return Err(Error::Admission(AdmissionError::Stale));
        }
        admit(now().map_err(Error::Admission)?)?;
        let ready = copier.apply(plan, source, destination)?;
        let mut hold = None;
        status.observed_ref_current = match self.repository.named_commit(&self.enrollment.ref_name)
        {
            Ok(c) => {
                let current = &c == self.request.commit();
                if !current {
                    hold = Some(AdmissionError::Stale);
                }
                Some(current)
            }
            Err(_) => {
                hold = Some(AdmissionError::Unavailable);
                None
            }
        };
        match now().and_then(|time| self.request.admit(time, &selection, &policy)) {
            Ok(()) => (),
            Err(e) => hold = Some(e),
        }
        Ok(Copied {
            report: Report {
                schema: "git-asset.trusted-native-copy-report.experimental-1",
                policy,
                provenance: self.request.provenance().clone(),
                selection: status,
                copy: ready.receipt().clone(),
                post_copy_hold: hold,
                storage_converged: None,
            },
            ready,
            _generation: generation,
        })
    }
}
