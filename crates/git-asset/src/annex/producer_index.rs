//! Immutable metadata declarations, never current native custody or mutation authority.
use super::{
    publication::{Error, Plan, Result},
    snapshot::Snapshot,
};
use git_asset_core::{
    Digest, LogicalPath, Object, identity,
    tree::{NodeStore, Tree, TreeLimits, TreeMetrics},
};
use serde::{Deserialize, Serialize};

const SCHEMA: &str = "git-asset.native-producer.experimental-1";
const MAX_RECORD: usize = 32768;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub schema: String,
    pub object: Object,
    pub producer: Plan,
    pub retained: Option<Snapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_retention: Option<git_asset_store::retention::RetentionGrant>,
}
impl Record {
    pub fn new(producer: Plan, retained: Option<Snapshot>) -> Result<Self> {
        let row = Self {
            schema: SCHEMA.into(),
            object: producer.candidate().object.clone(),
            producer,
            retained,
            control_retention: None,
        };
        row.bytes()?;
        Ok(row)
    }
    fn bytes(&self) -> Result<Vec<u8>> {
        if let Some(grant) = &self.control_retention {
            let raw = grant.to_json().map_err(|_| Error::Integrity)?;
            git_asset_store::retention::RetentionGrant::from_json(&raw)
                .map_err(|_| Error::Integrity)?;
            if grant.request().publication.object
                != Object::from_bytes(&super::control::document(&self.object)?)
            {
                return Err(Error::Integrity);
            }
        }
        if self.schema != SCHEMA
            || self.object != self.producer.candidate().object
            || self
                .retained
                .as_ref()
                .is_some_and(|s| s.object != self.object)
        {
            return Err(Error::Integrity);
        }
        self.producer.id()?;
        let bytes =
            identity::canonical_json(&serde_json::to_value(self).map_err(|_| Error::Invalid)?)
                .map_err(|_| Error::Invalid)?;
        if bytes.len() > MAX_RECORD {
            return Err(Error::Budget);
        }
        Ok(bytes)
    }
    /// Writes only immutable metadata; callers explicitly publish the resulting tree.
    pub fn store(&self, nodes: &mut impl NodeStore) -> Result<(LogicalPath, Object)> {
        let bytes = self.bytes()?;
        let object = nodes.put(&bytes)?;
        if object != Object::from_bytes(&bytes) {
            return Err(Error::Integrity);
        }
        Ok((key(&self.object)?, object))
    }
}
fn key(object: &Object) -> Result<LogicalPath> {
    LogicalPath::parse(object.sha256.as_str()).map_err(|_| Error::Invalid)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    pub root: Object,
    pub after: Digest,
}
#[derive(Debug)]
pub struct Page {
    pub records: Vec<Record>,
    pub next: Option<Cursor>,
    pub metrics: TreeMetrics,
}
pub struct Index {
    tree: Tree,
}
impl Index {
    pub fn from_root(root: Object) -> Self {
        Self {
            tree: Tree::from_root(root),
        }
    }
    pub fn root(&self) -> &Object {
        self.tree.root()
    }
    /// None is unknown registration, never evidence of absent or disposable bytes.
    pub fn get(
        &self,
        object: &Object,
        nodes: &mut impl NodeStore,
        limits: TreeLimits,
    ) -> Result<(Option<Record>, TreeMetrics)> {
        let path = key(object)?;
        let (row, mut metrics) = self.tree.get(&path, nodes, limits)?;
        let row = row
            .map(|r| read(&path, &r, nodes, limits, &mut metrics))
            .transpose()?;
        if row.as_ref().is_some_and(|r| r.object != *object) {
            return Err(Error::Integrity);
        }
        Ok((row, metrics))
    }
    /// A resumed suffix does not establish coverage of an unseen prefix.
    pub fn page(
        &self,
        cursor: Option<&Cursor>,
        count: usize,
        nodes: &mut impl NodeStore,
        limits: TreeLimits,
    ) -> Result<Page> {
        if cursor.is_some_and(|c| c.root != *self.root()) {
            return Err(Error::Stale);
        }
        let after = cursor
            .map(|c| LogicalPath::parse(c.after.as_str()).map_err(|_| Error::Invalid))
            .transpose()?;
        let page = self.tree.flat_page(after.as_ref(), count, nodes, limits)?;
        let mut metrics = page.metrics;
        let mut records = Vec::with_capacity(page.entries.len());
        for (path, object) in page.entries {
            records.push(read(&path, &object, nodes, limits, &mut metrics)?);
        }
        let next = page
            .next
            .map(|p| -> Result<Cursor> {
                Ok(Cursor {
                    root: self.root().clone(),
                    after: Digest::parse(p.as_str()).map_err(|_| Error::Integrity)?,
                })
            })
            .transpose()?;
        Ok(Page {
            records,
            next,
            metrics,
        })
    }
}
fn read(
    path: &LogicalPath,
    object: &Object,
    nodes: &impl NodeStore,
    limits: TreeLimits,
    metrics: &mut TreeMetrics,
) -> Result<Record> {
    if object.bytes > MAX_RECORD as u64 {
        return Err(Error::Budget);
    }
    let size = usize::try_from(object.bytes).map_err(|_| Error::Budget)?;
    let reads = metrics.reads.checked_add(1).ok_or(Error::Budget)?;
    let bytes = metrics.read_bytes.checked_add(size).ok_or(Error::Budget)?;
    if reads > limits.max_reads || bytes > limits.max_read_bytes {
        return Err(Error::Budget);
    }
    let data = nodes.get(object)?.ok_or(Error::Missing)?;
    if Object::from_bytes(&data) != *object {
        return Err(Error::Integrity);
    }
    let row: Record = serde_json::from_slice(&data).map_err(|_| Error::Integrity)?;
    if row.bytes()? != data || key(&row.object)? != *path {
        return Err(Error::Integrity);
    }
    metrics.reads = reads;
    metrics.read_bytes = bytes;
    Ok(row)
}
