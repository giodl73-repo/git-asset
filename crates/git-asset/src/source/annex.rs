//! Explicit native-annex content declarations. No provider/path lookup or custody.
use super::*;
use crate::lifecycle::{self, CommittedSource, FlatCapture};
use git_asset_core::identity;
/// Bounded canonical single-line SHA256/SHA256E subset, not all annex backends.
/// Relative targets are checked as metadata and NEVER passed to filesystem IO.
pub fn pointer(path: &LogicalPath, mode: &str, raw: &[u8]) -> Result<Object> {
    if raw.len() > 1024 {
        return Err(Error::Budget);
    }
    if !matches!(mode, "100644" | "100755" | "120000") {
        return Err(Error::Invalid);
    }
    let text = std::str::from_utf8(raw).map_err(|_| Error::Invalid)?;
    let line = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(text);
    if line.contains(['\n', '\r']) || (mode == "120000" && text != line) {
        return Err(Error::Invalid);
    }
    let key = if let Some(key) = line.strip_prefix("/annex/objects/") {
        if mode == "120000" {
            return Err(Error::Invalid);
        }
        key
    } else {
        let prefix = format!(
            "{}.git/annex/objects/",
            "../".repeat(path.as_str().split('/').count() - 1)
        );
        let parts = line
            .strip_prefix(&prefix)
            .ok_or(Error::Invalid)?
            .split('/')
            .collect::<Vec<_>>();
        if parts.len() != 4
            || parts[2] != parts[3]
            || parts[..2].iter().any(|p| {
                p.is_empty() || p.len() > 8 || !p.bytes().all(|b| b.is_ascii_alphanumeric())
            })
        {
            return Err(Error::Invalid);
        }
        parts[2]
    };
    let (rest, extension) = if let Some(s) = key.strip_prefix("SHA256-s") {
        (s, false)
    } else if let Some(s) = key.strip_prefix("SHA256E-s") {
        (s, true)
    } else {
        return Err(Error::Invalid);
    };
    let (size, hash) = rest.split_once("--").ok_or(Error::Invalid)?;
    let bytes = size.parse::<u64>().map_err(|_| Error::Invalid)?;
    if bytes.to_string() != size || hash.len() < 64 {
        return Err(Error::Invalid);
    }
    let suffix = hash.get(64..).ok_or(Error::Invalid)?;
    let hash = hash.get(..64).ok_or(Error::Invalid)?;
    if (!extension && !suffix.is_empty())
        || (!suffix.is_empty() && !suffix.starts_with('.'))
        || suffix
            .bytes()
            .any(|b| !(33..=126).contains(&b) || b == b'/' || b == b'\\')
    {
        return Err(Error::Invalid);
    }
    Ok(Object {
        sha256: Digest::parse(hash).map_err(|_| Error::Invalid)?,
        bytes,
    })
}
/// Actual fixed Git repository; exact historical capture never retargets a ref.
/// Lifecycle still separately checks current ref and namespace admission.
pub struct Source<'a> {
    pub repository: &'a Repository,
}
fn mapped(error: Error) -> lifecycle::Error {
    match error {
        Error::Budget => lifecycle::Error::Budget,
        Error::Incomplete => lifecycle::Error::Pending,
        _ => lifecycle::Error::Git,
    }
}
impl CommittedSource for Source<'_> {
    fn named_commit(&self, name: &str) -> lifecycle::Result<GitId> {
        self.repository.named_commit(name).map_err(mapped)
    }
    fn flat_capture(
        &self,
        commit: GitId,
        scope: &[LogicalPath],
        budget: &lifecycle::Budget,
    ) -> lifecycle::Result<FlatCapture> {
        budget.validate()?;
        let start = Instant::now();
        // Bound successful listing/blob IO conservatively within the requested
        // metadata allowance, in addition to the original repository limits.
        let per_half = budget.node_bytes as usize / 2;
        let mut limits = self.repository.limits;
        limits.listing_bytes = limits.listing_bytes.min(per_half);
        limits.entries = limits.entries.min(budget.nodes as usize);
        limits.pointer_blobs = limits
            .pointer_blobs
            .min(budget.entries as usize)
            .min(per_half / 2048);
        limits.command_millis = limits.command_millis.min(budget.millis);
        limits.validate().map_err(mapped)?;
        let repository = Repository {
            enrollment: self.repository.enrollment.clone(),
            bare: self.repository.bare,
            executable: self.repository.executable.clone(),
            root: self.repository.root.clone(),
            limits,
        };
        let snapshot = repository.commit(commit).map_err(mapped)?;
        let extraction = repository
            .extract_annex(&snapshot, Scope::new(scope.to_vec()).map_err(mapped)?)
            .map_err(mapped)?;
        if !extraction.complete() {
            return Err(lifecycle::Error::Pending);
        }
        if extraction.bindings().len() > budget.entries as usize {
            return Err(lifecycle::Error::Budget);
        }
        let context = identity::canonical_json(&serde_json::json!([
            "git-asset.local-annex-listing.experimental-1",
            snapshot.checkpoint().listing,
            scope
        ]))
        .map_err(|_| lifecycle::Error::Invalid)?;
        if context.len() > budget.node_bytes as usize {
            return Err(lifecycle::Error::Budget);
        }
        budget.timed(start)?;
        Ok(FlatCapture {
            listing: Digest::blob(&context),
            bindings: extraction
                .bindings()
                .iter()
                .map(|b| (b.path.clone(), b.object.clone()))
                .collect(),
        })
    }
}
