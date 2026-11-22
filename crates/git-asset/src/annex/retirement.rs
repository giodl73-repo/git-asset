//! Explicit native retirement assessment/coordinator. No initialization or implicit effects.
use super::{managed, process, publication, snapshot};
use git_asset_core::{
    Digest, Object,
    tree::{NodeStore, TreeLimits},
};
use git_asset_store::{
    existing::VerifiedRead,
    protection::Limits,
    source_claims::admission::{Namespace, SubjectFence},
    store_protection::{self, NamespaceAssessment},
};
pub mod planning;

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
/// Verified source bytes tied to the exact assessment basis. Keep this guard
/// until effect intent is durable; it is not a removal capability.
pub struct SourceRead {
    basis: Digest,
    read: VerifiedRead,
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
        let (configuration, directory, mut namespaces) = self.pin(object)?;
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
    fn pin(&mut self, object: &Object) -> Result<(VerifiedRead, std::fs::File, Vec<Namespace>)> {
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
        let namespaces = self
            .profile
            .members
            .iter()
            .map(|m| Namespace::open(&m.root, &m.minimum).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;
        for (namespace, member) in namespaces.iter().zip(self.profile.binding.members()) {
            if !member.places.iter().all(|place| {
                namespace.policy().classes.values().any(|p| {
                    p.current
                        .iter()
                        .chain(&p.pending)
                        .chain(&p.predecessor)
                        .any(|p| p == place)
                })
            }) {
                return Err(publication::Error::Denied.into());
            }
        }
        Ok((configuration, directory, namespaces))
    }
}
impl Assessment {
    pub fn read_for_plan(&mut self) -> Result<SourceRead> {
        let read = self.read_source()?;
        Ok(SourceRead {
            basis: self.replay_basis()?.id()?,
            read,
        })
    }
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

/// A retained exact member plus pinned keeper bytes. No independence/removal grant.
pub struct RetainedRead<'a> {
    read: VerifiedRead,
    store: Digest,
    snapshot: snapshot::Snapshot,
    metrics: git_asset_core::tree::TreeMetrics,
    _source: &'a Assessment,
    _configuration: VerifiedRead,
    _directory: std::fs::File,
    _fences: Vec<SubjectFence>,
    profile: managed::Profile,
}
impl Assessment {
    /// Overlapping authorities borrow existing fences instead of reacquiring shared uses.
    /// Caller meters tree work across the whole operation; this is one point proof.
    pub fn open_keeper<'a>(
        &'a self,
        profile: managed::Profile,
        snapshot: &snapshot::Snapshot,
        nodes: &mut impl NodeStore,
        limits: TreeLimits,
    ) -> Result<RetainedRead<'a>> {
        self.check_current()?;
        if &snapshot.object != self.facts.object()
            || !profile.binding.members().iter().any(|member| {
                member.namespace == snapshot.namespace && member.places.contains(&snapshot.place)
            })
        {
            return Err(publication::Error::Denied.into());
        }
        if profile.members.iter().any(|member| {
            self.profile.members.iter().any(|source| {
                source.minimum.identity() == member.minimum.identity() && source.root != member.root
            })
        }) {
            return Err(publication::Error::Denied.into());
        }
        let mut keeper = Assessor::new(profile)?;
        if keeper.reader.store() == self.store() {
            return Err(publication::Error::Denied.into());
        }
        let (configuration, directory, mut namespaces) = keeper.pin(&snapshot.object)?;
        let mut fences = Vec::new();
        for namespace in &namespaces {
            if self.fence(namespace.identity()).is_none() {
                fences.push(namespace.fence(&snapshot.object)?);
            }
        }
        let namespace = namespaces
            .iter_mut()
            .find(|n| n.identity() == &snapshot.namespace)
            .ok_or(publication::Error::Denied)?;
        let fence = self
            .fence(&snapshot.namespace)
            .or_else(|| fences.iter().find(|f| f.authority() == &snapshot.namespace))
            .ok_or(publication::Error::Denied)?;
        let proof = snapshot.verify_fenced(namespace, fence, nodes, limits)?;
        let metrics = proof.metrics().clone();
        let read = keeper.reader.open_local(&snapshot.object)?;
        self.check_current()?;
        Ok(RetainedRead {
            read,
            store: keeper.reader.store(),
            snapshot: snapshot.clone(),
            metrics,
            _source: self,
            _configuration: configuration,
            _directory: directory,
            _fences: fences,
            profile: keeper.profile,
        })
    }
}
impl<'source> RetainedRead<'source> {
    /// Selected-device comparison, not owner fault-domain admission or removal permission.
    pub fn compare_source_storage(&self) -> Result<StorageSeparation<'_, 'source>> {
        self._source.check_current()?;
        #[cfg(windows)]
        {
            let view = git_asset_windows_process::storage::Separation::compare_with_limit(
                &self._source._directory,
                &self._directory,
                self._source.profile.pool.command_millis,
            )
            .map_err(storage_error)?
            .ok_or(publication::Error::Denied)?;
            let guard = StorageSeparation { keeper: self, view };
            guard.revalidate()?;
            Ok(guard)
        }
        #[cfg(not(windows))]
        {
            Err(publication::Error::Unsupported.into())
        }
    }
    pub fn store(&self) -> &Digest {
        &self.store
    }
    pub fn snapshot(&self) -> &snapshot::Snapshot {
        &self.snapshot
    }
    pub fn metrics(&self) -> &git_asset_core::tree::TreeMetrics {
        &self.metrics
    }
}
#[cfg(windows)]
fn storage_error(e: std::io::Error) -> Error {
    match e.kind() {
        std::io::ErrorKind::Unsupported => publication::Error::Unsupported,
        std::io::ErrorKind::PermissionDenied => publication::Error::Denied,
        std::io::ErrorKind::TimedOut => publication::Error::Budget,
        _ => publication::Error::Uncertain,
    }
    .into()
}
/// Pinned byte/config/namespace guards plus separated observed device views.
/// Distinct privately admitted direct backing/fault domains remain required.
pub struct StorageSeparation<'read, 'source> {
    keeper: &'read RetainedRead<'source>,
    #[cfg(windows)]
    view: git_asset_windows_process::storage::Separation,
}
impl StorageSeparation<'_, '_> {
    pub fn revalidate(&self) -> Result<()> {
        self.keeper._source.check_current()?;
        #[cfg(windows)]
        {
            if !self.view.revalidate().map_err(storage_error)? {
                return Err(publication::Error::Stale.into());
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Err(publication::Error::Unsupported.into())
        }
    }
    /// Hashed identity only: no raw serial, discovered path or custody grant.
    pub fn id(&self) -> Result<Digest> {
        self.revalidate()?;
        #[cfg(windows)]
        {
            let bytes = git_asset_core::identity::canonical_json(&serde_json::json!([
                "git-asset.native-device-separation.v1",
                self.keeper._source.store(),
                self.keeper.store(),
                self.keeper.snapshot(),
                self.view.fingerprints()
            ]))
            .map_err(|_| publication::Error::Invalid)?;
            if bytes.len() > 16384 {
                return Err(publication::Error::Budget.into());
            }
            Ok(Digest::blob(&bytes))
        }
        #[cfg(not(windows))]
        {
            Err(publication::Error::Unsupported.into())
        }
    }
}
impl std::io::Read for RetainedRead<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        std::io::Read::read(&mut self.read, bytes)
    }
}
impl std::io::Seek for RetainedRead<'_> {
    fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
        std::io::Seek::seek(&mut self.read, position)
    }
}

/// Exact recovery basis, not a physical birth certificate or a removal capability.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ReplayBasis {
    store: Digest,
    configuration: Object,
    binding: Digest,
    locations: Digest,
    object: Object,
    checked: Vec<store_protection::Checked>,
}
impl Assessment {
    /// Captures existing scoped absence evidence for an immutable operation intent.
    pub fn replay_basis(&self) -> Result<ReplayBasis> {
        self.check_current()?;
        let locations = git_asset_core::identity::canonical_json(&serde_json::json!([
            &self.profile.pool.repository,
            self.profile
                .members
                .iter()
                .map(|m| (&m.root, &m.minimum))
                .collect::<Vec<_>>()
        ]))
        .map_err(|_| publication::Error::Invalid)?;
        Ok(ReplayBasis {
            store: self.store(),
            configuration: self.profile.pool.configuration.clone(),
            binding: self.facts.binding().clone(),
            locations: Digest::blob(&locations),
            object: self.facts.object().clone(),
            checked: self.facts.checked().to_vec(),
        })
    }
    /// New explicit assessment is required after restart. Changed basis always holds.
    /// Unrelated progress can conservatively stale recovery; live fences stay scoped.
    pub fn check_replay(&self, expected: &ReplayBasis) -> Result<()> {
        if &self.replay_basis()? != expected {
            return Err(publication::Error::Stale.into());
        }
        Ok(())
    }
}
impl ReplayBasis {
    pub fn id(&self) -> Result<Digest> {
        let bytes = git_asset_core::identity::canonical_json(&serde_json::json!([
            "git-asset.annex-replay-basis.experimental-1",
            self
        ]))
        .map_err(|_| publication::Error::Invalid)?;
        if bytes.len() > 16384 {
            return Err(publication::Error::Budget.into());
        }
        Ok(Digest::blob(&bytes))
    }
}
