//! Explicit native ordinary-retirement assessment. No removal/initialization API.
use super::{managed, process, publication};
use git_asset_core::{Digest, Object, tree::NodeStore};
use git_asset_store::{
    existing::VerifiedRead,
    protection::Limits,
    source_claims::admission::{Namespace, SubjectFence},
    store_protection::{self, NamespaceAssessment},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Protected,
    Operation(publication::Error),
}
impl Error {
    pub fn code(self) -> &'static str {
        match self {
            Self::Protected => "protected",
            Self::Operation(e) => e.code(),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for Error {}
impl From<publication::Error> for Error {
    fn from(e: publication::Error) -> Self {
        Self::Operation(e)
    }
}
impl From<super::Error> for Error {
    fn from(e: super::Error) -> Self {
        publication::Error::from(e).into()
    }
}
impl From<git_asset_store::source_claims::admission::Error> for Error {
    fn from(e: git_asset_store::source_claims::admission::Error) -> Self {
        publication::Error::from(e).into()
    }
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct Assessor {
    profile: managed::Profile,
    reader: super::Reader,
}
pub struct Assessment {
    profile: managed::Profile,
    reader: super::Reader,
    _configuration: VerifiedRead,
    _directory: std::fs::File,
    namespaces: Vec<Namespace>,
    facts: NamespaceAssessment,
}
fn protection_error(error: store_protection::Error) -> Error {
    match error {
        store_protection::Error::Required | store_protection::Error::Retained => Error::Protected,
        store_protection::Error::Busy => publication::Error::Busy.into(),
        store_protection::Error::Admission(e) => e.into(),
        store_protection::Error::InvalidBinding => publication::Error::Invalid.into(),
        _ => publication::Error::Stale.into(),
    }
}
impl Assessor {
    pub fn new(mut profile: managed::Profile) -> Result<Self> {
        managed::Reader::new(profile.clone()).map_err(publication::Error::from)?;
        profile
            .members
            .sort_by(|a, b| a.minimum.identity().cmp(b.minimum.identity()));
        let reader = super::Reader::new(profile.pool.clone())?;
        Ok(Self { profile, reader })
    }
    /// Explicit bounded read-only assessment; no source bytes or annex mutation.
    pub fn assess(
        mut self,
        object: &Object,
        nodes: &impl NodeStore,
        limits: &Limits,
    ) -> Result<Assessment> {
        self.reader.preflight(object)?;
        let configuration = self.reader.admit_repository_mode(true)?;
        let directory = self.reader.hold_annex_directory()?;
        let observed = process::run(
            &self.profile.pool,
            false,
            &[
                "config",
                "--local",
                "--no-includes",
                "--get-all",
                "tig.protectionbinding",
            ],
        )?;
        if super::line(&observed.stdout)?
            != self
                .profile
                .binding
                .id()
                .map_err(protection_error)?
                .as_str()
        {
            return Err(publication::Error::Stale.into());
        }
        let mut namespaces = self
            .profile
            .members
            .iter()
            .map(|m| Namespace::open(&m.root, &m.minimum).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;
        let facts = store_protection::assess_namespaces(
            &self.profile.binding,
            object,
            &mut namespaces,
            nodes,
            limits,
        )
        .map_err(protection_error)?;
        Ok(Assessment {
            profile: self.profile,
            reader: self.reader,
            _configuration: configuration,
            _directory: directory,
            namespaces,
            facts,
        })
    }
}
impl Assessment {
    pub fn store(&self) -> Digest {
        self.reader.store()
    }
    pub fn facts(&self) -> &NamespaceAssessment {
        &self.facts
    }
    pub fn fence(&self, namespace: &Digest) -> Option<&SubjectFence> {
        self.facts.fence(namespace)
    }
    pub fn check_current(&self) -> Result<()> {
        self.facts
            .check_current(&self.profile.binding, &self.namespaces)
            .map_err(protection_error)
    }
    /// Verification guard may be dropped before mutation only while this assessment lives.
    pub fn read_source(&mut self) -> Result<VerifiedRead> {
        self.check_current()?;
        self.reader
            .open_local(self.facts.object())
            .map_err(Error::from)
    }
}
