//! Bounded private owner input. No initialization or path discovery.
use super::operations::{Owner, Result};
use git_asset::annex::publication;
use git_asset_core::{identity, tree::TreeLimits};
use git_asset_store::{ledger, metadata, protection};
use serde::Deserialize;
use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Location {
    pub root: PathBuf,
    pub minimum: metadata::Checkpoint,
}
impl Location {
    pub fn open(&self) -> Result<metadata::MetadataReader> {
        metadata::MetadataReader::open(&self.root, &self.minimum).map_err(|e| match e {
            metadata::Error::Busy => "busy",
            metadata::Error::Corrupt => "corrupt",
            metadata::Error::Budget => "admission_denied",
            metadata::Error::Rollback | metadata::Error::Foreign => "stale",
            metadata::Error::NeedsRecovery => "needs_recovery",
            _ => "metadata_unavailable",
        })
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Producer {
    pub profile: publication::Profile,
    pub plan: publication::Plan,
    pub witness: PathBuf,
    pub ledger: PathBuf,
    pub minimum: ledger::Checkpoint,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Profile {
    pub schema: String,
    pub owner: Owner,
    pub source: Producer,
    pub keeper: Producer,
    pub source_metadata: Location,
    pub keeper_metadata: Location,
    pub limits: protection::Limits,
    #[serde(default)]
    pub allow_apply: bool,
    #[serde(default)]
    pub allow_resume: bool,
    #[serde(default)]
    pub allow_cancel: bool,
}
pub(super) fn raw(path: &Path, cap: u64) -> Result<Vec<u8>> {
    if !path.is_absolute()
        || path
            .components()
            .any(|p| matches!(p, Component::CurDir | Component::ParentDir))
    {
        return Err("invalid_document");
    }
    let before = fs::symlink_metadata(path).map_err(|_| "document_unavailable")?;
    if !before.is_file() || before.file_type().is_symlink() {
        return Err("invalid_document");
    }
    if before.len() > cap {
        return Err("admission_denied");
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1).custom_flags(0x00200000);
    }
    let file = options.open(path).map_err(|_| "document_unavailable")?;
    let current = file.metadata().map_err(|_| "document_unavailable")?;
    if !current.is_file() || current.len() > cap {
        return Err("invalid_document");
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if current.file_attributes() & 0x400 != 0 {
            return Err("invalid_document");
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != current.dev() || before.ino() != current.ino() {
            return Err("invalid_document");
        }
    }
    let mut bytes = Vec::new();
    file.take(cap + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "document_unavailable")?;
    if bytes.len() as u64 > cap {
        return Err("admission_denied");
    }
    Ok(bytes)
}
pub(super) fn parse<T: serde::de::DeserializeOwned>(raw: &[u8]) -> Result<T> {
    serde_json::from_value(identity::parse_json(raw).map_err(|_| "invalid_document")?)
        .map_err(|_| "invalid_document")
}
pub(super) fn tree_limits() -> TreeLimits {
    TreeLimits {
        max_writes: 0,
        max_write_bytes: 0,
        ..TreeLimits::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn document_boundary_rejects_oversize_duplicate_keys_relative_paths_and_directories() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("profile");
        fs::write(&path, vec![b' '; 65537]).unwrap();
        assert_eq!(raw(&path, 65536), Err("admission_denied"));
        assert_eq!(raw(root.path(), 65536), Err("invalid_document"));
        assert_eq!(raw(Path::new("relative"), 65536), Err("invalid_document"));
        assert_eq!(
            parse::<serde_json::Value>(br#"{"allow_apply":false,"allow_apply":true}"#),
            Err("invalid_document")
        );
        fs::write(&path, br#"{"exact":true}"#).unwrap();
        assert_eq!(raw(&path, 65536).unwrap(), br#"{"exact":true}"#);
    }
    #[cfg(unix)]
    #[test]
    fn leaf_symlink_is_not_an_owner_document() {
        let root = tempfile::tempdir().unwrap();
        let body = root.path().join("body");
        fs::write(&body, b"{}").unwrap();
        let link = root.path().join("link");
        std::os::unix::fs::symlink(&body, &link).unwrap();
        assert_eq!(raw(&link, 65536), Err("invalid_document"));
    }
}
