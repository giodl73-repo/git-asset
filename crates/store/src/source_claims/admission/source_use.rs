//! Scoped current source stability, independent of byte custody and removal.
use super::*;
use crate::source_claims::{MAX_SOURCES, Selection, Version};
pub(in crate::source_claims) const SCHEMA: &str = "git-asset.source-claims.design-3";
pub(in crate::source_claims) fn create_anchors(root: &Path) -> super::super::Result<()> {
    fs::create_dir(root.join("source-gates"))?;
    for slot in 0..MAX_SOURCES {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("source-gates").join(format!("{slot:04x}")))?;
    }
    Ok(())
}
#[derive(Debug, PartialEq, Eq)]
pub enum UseError {
    Unsupported,
    Admission(super::Error),
    Owner,
    Stale,
    Inactive,
}
impl std::fmt::Display for UseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for UseError {}
impl From<super::Error> for UseError {
    fn from(e: super::Error) -> Self {
        Self::Admission(e)
    }
}
/// No serialization/clone/timeout. Keeps only the exact source slot stable.
pub struct Use {
    namespace: Digest,
    source: Key,
    owner: Key,
    version: Version,
    revision: u64,
    _lock: Gate,
}
impl Use {
    pub fn namespace(&self) -> &Digest {
        &self.namespace
    }
    pub fn source(&self) -> &Key {
        &self.source
    }
    pub fn owner(&self) -> &Key {
        &self.owner
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn version(&self) -> &Version {
        &self.version
    }
    pub fn selection(&self) -> &Selection {
        &self.version.selection
    }
}
pub(in crate::source_claims) struct Gate {
    _file: File,
    _directories: Vec<File>,
}
fn gate(root: &Path, slot: usize, exclusive: bool) -> Result<Gate> {
    if slot >= MAX_SOURCES {
        return Err(Error::WrongAuthority);
    }
    let directory = root.join("source-gates");
    local::directory(&directory)?;
    let mut directories = Vec::new();
    // Pin the actual Windows control path while the slot is in use. These are
    // metadata handles only, not a cache lock or a recursive inventory.
    #[cfg(windows)]
    {
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        for parent in root
            .ancestors()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .chain(std::iter::once(directory.as_path()))
        {
            local::directory(parent)?;
            let held = OpenOptions::new()
                .read(true)
                .share_mode(3)
                .custom_flags(0x0220_0000)
                .open(parent)?;
            let m = held.metadata()?;
            if !m.is_dir() || m.file_attributes() & 0x400 != 0 {
                return Err(Error::WrongAuthority);
            }
            directories.push(held);
        }
    }
    #[cfg(not(windows))]
    let _ = &mut directories;
    let path = directory.join(format!("{slot:04x}"));
    local::regular(&path)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(3).custom_flags(0x0020_0000);
    }
    let file = options.open(&path)?;
    let m = file.metadata()?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(Error::WrongAuthority);
        }
    }
    if !m.is_file() || m.len() != 0 {
        return Err(Error::WrongAuthority);
    }
    let locked = if exclusive {
        file.try_lock()
    } else {
        file.try_lock_shared()
    };
    locked.map_err(|e| match e {
        fs::TryLockError::WouldBlock => Error::Busy,
        fs::TryLockError::Error(_) => Error::Io,
    })?;
    Ok(Gate {
        _file: file,
        _directories: directories,
    })
}
pub(in crate::source_claims) fn mutations(
    root: &Path,
    state: &super::super::State,
    requests: &[super::super::Request],
) -> super::super::Result<Vec<Gate>> {
    let mut slots = BTreeSet::new();
    for request in requests {
        let key = request.command.source();
        if let Some(slot) = state.source_slots.get(key.as_str()) {
            slots.insert(*slot);
        } else if state.sources.contains_key(key.as_str()) {
            return Err(super::super::Error::Corrupt);
        } else if !requests
            .iter()
            .any(|r| matches!(&r.command,Command::Enroll{source,..} if source==key))
        {
            return Err(super::super::Error::Missing);
        }
    }
    slots
        .into_iter()
        .map(|slot| {
            gate(root, slot, true).map_err(|e| match e {
                Error::Busy => super::super::Error::Busy,
                Error::Io => super::super::Error::Io,
                Error::Journal(e) => e,
                _ => super::super::Error::NeedsRecovery,
            })
        })
        .collect()
}
impl Namespace {
    /// Explicit fresh capability only; does not upgrade an existing namespace.
    pub fn create_fenced(root: &Path, project: Key, epoch: Key, policy: Policy) -> Result<Self> {
        let policy = policy.normalized()?;
        Ok(Self {
            journal: SourceJournal::create_scoped(root, project, epoch, Some(policy), true)?,
        })
    }
    pub fn supports_source_use(&self) -> bool {
        self.journal.config.schema == SCHEMA
    }
    /// Hold exact owner/current version after checking it again under the gate.
    /// Historical receipts and pending transitions never become current use.
    pub fn source_use(
        &mut self,
        source: &Key,
        owner: &Key,
        expected_revision: u64,
        version: &Version,
    ) -> std::result::Result<Use, UseError> {
        if !self.supports_source_use() {
            return Err(UseError::Unsupported);
        }
        self.journal
            .check_guarded_marker()
            .map_err(super::Error::from)?;
        self.journal.sync().map_err(super::Error::from)?;
        let slot = *self
            .journal
            .state
            .source_slots
            .get(source.as_str())
            .ok_or(UseError::Inactive)?;
        let held = gate(&self.journal.root, slot, false)?;
        let current = self.source(source)?.ok_or(UseError::Inactive)?;
        if &current.owner != owner {
            return Err(UseError::Owner);
        }
        if current.pending.is_some() || current.current.is_none() {
            return Err(UseError::Inactive);
        }
        if current.revision != expected_revision || current.current.as_ref() != Some(version) {
            return Err(UseError::Stale);
        }
        self.journal
            .check_guarded_marker()
            .map_err(super::Error::from)?;
        Ok(Use {
            namespace: self.identity().clone(),
            source: source.clone(),
            owner: owner.clone(),
            version: version.clone(),
            revision: expected_revision,
            _lock: held,
        })
    }
}
