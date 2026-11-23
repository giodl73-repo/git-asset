//! One exact member of an existing retained flat snapshot, without per-asset heads.
//! Metadata obligation/use only; native bytes, full coverage and removal are separate.
use super::publication::{Error, Result};
use git_asset_core::{
    Digest, LogicalPath, Object, identity,
    tree::{NodeStore, Tree, TreeLimits, TreeMetrics},
};
use git_asset_store::{
    Key,
    source_claims::{
        Kind, Version,
        admission::{Namespace, SubjectFence, SubjectUse},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub namespace: Digest,
    pub source: Key,
    pub owner: Key,
    pub version: Version,
    pub place: Key,
    pub path: LogicalPath,
    pub object: Object,
}
pub struct Held {
    snapshot: Snapshot,
    _use: SubjectUse,
    metrics: TreeMetrics,
}
impl Held {
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub fn metrics(&self) -> &TreeMetrics {
        &self.metrics
    }
}
/// Borrows the caller's exact exclusive subject fence; never reacquires a shared lock.
pub struct Fenced<'a> {
    snapshot: Snapshot,
    _fence: &'a SubjectFence,
    metrics: TreeMetrics,
}
impl Fenced<'_> {
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub fn metrics(&self) -> &TreeMetrics {
        &self.metrics
    }
}
impl Snapshot {
    pub fn id(&self) -> Result<Digest> {
        Ok(Digest::blob(
            &identity::canonical_json(
                &serde_json::to_value(("git-asset.annex-retained-snapshot.experimental-1", self))
                    .map_err(|_| Error::Invalid)?,
            )
            .map_err(|_| Error::Invalid)?,
        ))
    }
    fn current(&self, namespace: &mut Namespace) -> Result<()> {
        if namespace.identity() != &self.namespace {
            return Err(Error::Stale);
        }
        let source = namespace
            .source(&self.source)?
            .ok_or(Error::ProtectionLost)?;
        if source.owner != self.owner {
            return Err(Error::Denied);
        }
        if source.kind != Some(Kind::Retention) {
            return Err(Error::Unsupported);
        }
        if !source
            .current
            .iter()
            .chain(&source.predecessors)
            .any(|v| v == &self.version)
        {
            return Err(Error::ProtectionLost);
        }
        let policy = namespace
            .policy()
            .classes
            .get(&Kind::Retention)
            .ok_or(Error::Unsupported)?;
        if ![&policy.current, &policy.pending, &policy.predecessor]
            .iter()
            .all(|places| places.contains(&self.place))
        {
            return Err(Error::Denied);
        }
        Ok(())
    }
    /// Exact current/predecessor root, no fallback to the current selection.
    /// Caller supplies the admitted metadata reader and a shared aggregate meter.
    /// This never opens payloads or writes source state; bytes require native verification.
    pub fn verify(
        &self,
        namespace: &mut Namespace,
        metadata: &mut impl NodeStore,
        limits: TreeLimits,
    ) -> Result<Held> {
        if self.version.revision == 0 {
            return Err(Error::Invalid);
        }
        if namespace.identity() != &self.namespace {
            return Err(Error::Stale);
        }
        let held = namespace.hold(&self.object)?;
        let metrics = self.membership(namespace, metadata, limits)?;
        Ok(Held {
            snapshot: self.clone(),
            _use: held,
            metrics,
        })
    }
    pub fn verify_fenced<'a>(
        &self,
        namespace: &mut Namespace,
        fence: &'a SubjectFence,
        metadata: &mut impl NodeStore,
        limits: TreeLimits,
    ) -> Result<Fenced<'a>> {
        if fence.authority() != &self.namespace || fence.object() != &self.object {
            return Err(Error::Denied);
        }
        if self.version.revision == 0 {
            return Err(Error::Invalid);
        }
        let metrics = self.membership(namespace, metadata, limits)?;
        Ok(Fenced {
            snapshot: self.clone(),
            _fence: fence,
            metrics,
        })
    }
    fn membership(
        &self,
        namespace: &mut Namespace,
        metadata: &mut impl NodeStore,
        limits: TreeLimits,
    ) -> Result<TreeMetrics> {
        self.current(namespace)?;
        let (actual, metrics) = Tree::from_root(self.version.selection.root.clone())
            .get(&self.path, metadata, limits)?;
        match actual {
            None => return Err(Error::Missing),
            Some(object) if object != self.object => return Err(Error::Conflict),
            Some(_) => (),
        }
        // A metadata provider can yield while another actor withdraws the source.
        // Recheck this exact obligation, not an unrelated global head checkpoint.
        self.current(namespace)?;
        Ok(metrics)
    }
}
#[cfg(test)]
mod tests;
