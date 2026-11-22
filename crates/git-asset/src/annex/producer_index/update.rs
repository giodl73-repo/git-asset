//! Retained producer roots over existing metadata and atomic source selection.
//! Caller persists the exact plan before apply; no new journal or native effects.
use super::*;
use crate::annex::pending::Scope;
use git_asset_core::tree::{TreeError, TreeResult};
use git_asset_store::{
    Key,
    metadata::{MetadataReader, MetadataStore},
    protection::Limits,
    source_claims::{
        Kind, Receipt, Selection, Version,
        admission::{Namespace, Select, source_use},
    },
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub scope: Scope,
    pub limits: Limits,
}
pub struct Change {
    /// None requires an absent key. Some requires the exact previous record blob.
    pub expected_record: Option<Object>,
    pub record: Record,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    schema: String,
    scope: Digest,
    revision: u64,
    expected: Option<Version>,
    root: Object,
    records: Vec<(Object, Object)>,
}
#[derive(Serialize)]
pub struct Report {
    pub root: Object,
    pub version: Version,
    pub receipt: Receipt,
    pub current: Option<bool>,
    pub post_effect_hold: Option<&'static str>,
    pub media_custody_verified: Option<bool>,
    pub authorizes_eviction: bool,
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    let raw = identity::canonical_json(&serde_json::to_value(value).map_err(|_| Error::Invalid)?)
        .map_err(|_| Error::Invalid)?;
    if raw.len() > 65536 {
        return Err(Error::Budget);
    }
    Ok(raw)
}
fn limits(p: &Profile) -> TreeLimits {
    TreeLimits {
        max_entries: p.limits.bindings,
        max_reads: p.limits.reads,
        max_writes: p.limits.reads,
        max_read_bytes: p.limits.read_bytes,
        max_write_bytes: p.limits.read_bytes,
        ..TreeLimits::default()
    }
}
struct Meter<'a> {
    nodes: &'a mut MetadataStore,
    used: RefCell<TreeMetrics>,
    cap: TreeLimits,
    start: Instant,
    millis: u64,
}
impl Meter<'_> {
    fn timed(&self) -> TreeResult<()> {
        if self.start.elapsed() >= Duration::from_millis(self.millis) {
            return Err(TreeError::Budget);
        }
        Ok(())
    }
}
impl NodeStore for Meter<'_> {
    fn get(&self, object: &Object) -> TreeResult<Option<Vec<u8>>> {
        self.timed()?;
        let bytes = usize::try_from(object.bytes).map_err(|_| TreeError::Budget)?;
        let mut m = self.used.borrow_mut();
        if m.reads >= self.cap.max_reads
            || bytes > self.cap.max_read_bytes.saturating_sub(m.read_bytes)
        {
            return Err(TreeError::Budget);
        }
        m.reads += 1;
        m.read_bytes += bytes;
        let raw = self.nodes.get(object)?;
        if raw
            .as_ref()
            .is_some_and(|b| Object::from_bytes(b) != *object)
        {
            return Err(TreeError::Integrity);
        }
        self.timed()?;
        Ok(raw)
    }
    fn put(&mut self, raw: &[u8]) -> TreeResult<Object> {
        self.timed()?;
        let m = self.used.get_mut();
        if m.writes >= self.cap.max_writes
            || raw.len() > self.cap.max_write_bytes.saturating_sub(m.write_bytes)
        {
            return Err(TreeError::Budget);
        }
        m.writes += 1;
        m.write_bytes += raw.len();
        let object = self.nodes.put(raw)?;
        if object != Object::from_bytes(raw) {
            return Err(TreeError::Integrity);
        }
        self.timed()?;
        Ok(object)
    }
}
struct Verify<'a, 'b>(&'a mut Meter<'b>);
impl NodeStore for Verify<'_, '_> {
    fn get(&self, object: &Object) -> TreeResult<Option<Vec<u8>>> {
        self.0.get(object)
    }
    fn put(&mut self, raw: &[u8]) -> TreeResult<Object> {
        let object = Object::from_bytes(raw);
        if self.0.get(&object)?.as_deref() != Some(raw) {
            return Err(TreeError::Integrity);
        }
        Ok(object)
    }
}
impl Profile {
    fn check(&self, ns: &Namespace, nodes: &MetadataStore) -> Result<()> {
        self.limits.validate().map_err(|_| Error::Budget)?;
        self.scope.check(ns, nodes)
    }
    fn state(&self, ns: &mut Namespace) -> Result<git_asset_store::source_claims::Source> {
        let state = ns.source(&self.scope.prefix)?.ok_or(Error::Missing)?;
        if state.owner != self.scope.owner || state.kind != Some(Kind::Job) {
            return Err(Error::Denied);
        }
        Ok(state)
    }
}
impl Plan {
    pub fn root(&self) -> &Object {
        &self.root
    }
    pub fn expected(&self) -> Option<&Version> {
        self.expected.as_ref()
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        encode(self)
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > 65536 {
            return Err(Error::Budget);
        }
        let plan: Self = serde_json::from_slice(raw).map_err(|_| Error::Invalid)?;
        plan.valid()?;
        if plan.to_json()? != raw {
            return Err(Error::Integrity);
        }
        Ok(plan)
    }
    fn valid(&self) -> Result<()> {
        if self.schema != "git-asset.producer-root-update.experimental-1"
            || self.revision == 0
            || self.root.bytes == 0
            || self.records.len() > 32
            || self
                .expected
                .as_ref()
                .is_some_and(|v| v.revision == 0 || v.revision > self.revision)
            || self
                .records
                .windows(2)
                .any(|r| r[0].0.sha256 >= r[1].0.sha256)
            || self
                .records
                .iter()
                .any(|(_, row)| row.bytes == 0 || row.bytes > MAX_RECORD as u64)
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    fn selection(&self, p: &Profile) -> Result<Select> {
        self.valid()?;
        if self.scope != p.scope.id()? {
            return Err(Error::Stale);
        }
        let id = Digest::blob(&self.to_json()?);
        let key = |prefix: &str| {
            Key::parse(&format!("{prefix}-{}", id.as_str())).map_err(|_| Error::Invalid)
        };
        Ok(Select {
            prepare: key("producer-prepare")?,
            adopt: key("producer-adopt")?,
            source: p.scope.prefix.clone(),
            owner: p.scope.owner.clone(),
            expected_revision: self.revision,
            selection: Selection {
                root: self.root.clone(),
                context: id,
            },
        })
    }
}
/// Source must already be enrolled as metadata-only Job. No removal edits exist.
/// None means every requested canonical record is already unchanged.
pub fn prepare(
    p: &Profile,
    expected: Option<&Version>,
    changes: &[Change],
    ns: &mut Namespace,
    nodes: &mut MetadataStore,
) -> Result<Option<Plan>> {
    p.check(ns, nodes)?;
    if !p.scope.allow_store {
        return Err(Error::Denied);
    }
    if changes.len() > 32 {
        return Err(Error::Budget);
    }
    let state = p.state(ns)?;
    if state.pending.is_some() || state.current.as_ref() != expected {
        return Err(Error::Stale);
    }
    let mut meter = Meter {
        nodes,
        used: RefCell::new(TreeMetrics::default()),
        cap: limits(p),
        start: Instant::now(),
        millis: p.limits.millis,
    };
    let tree = match expected {
        Some(v) => Tree::from_root(v.selection.root.clone()),
        None => Tree::empty(&mut meter)?,
    };
    let mut edits = BTreeMap::new();
    let mut records = BTreeMap::new();
    for change in changes {
        let path = key(&change.record.object)?;
        if edits.contains_key(&path) {
            return Err(Error::Conflict);
        }
        let (previous, _) = tree.get(&path, &mut meter, limits(p))?;
        if previous != change.expected_record {
            return Err(Error::Conflict);
        }
        if let Some(old) = previous {
            let old = read(&path, &old, &meter, limits(p), &mut TreeMetrics::default())?;
            if old.producer != change.record.producer {
                return Err(Error::Conflict);
            }
        }
        let (path, record) = change.record.store(&mut meter)?;
        records.insert(
            change.record.object.sha256.clone(),
            (change.record.object.clone(), record.clone()),
        );
        edits.insert(path, Some(record));
    }
    let (updated, _) = tree.edit(tree.root(), &edits, &mut meter, limits(p))?;
    meter.nodes.finish().map_err(|_| Error::NeedsRecovery)?;
    if expected.is_some_and(|v| &v.selection.root == updated.root()) {
        meter.timed()?;
        return Ok(None);
    }
    let index = Index::from_root(updated.root().clone());
    for (object, raw) in records.values() {
        let (row, _) = index.get(object, &mut meter, limits(p))?;
        if row.ok_or(Error::Missing)?.metadata_object()? != *raw {
            return Err(Error::Integrity);
        }
    }
    meter.timed()?;
    let plan = Plan {
        schema: "git-asset.producer-root-update.experimental-1".into(),
        scope: p.scope.id()?,
        revision: state.revision,
        expected: expected.cloned(),
        root: updated.root().clone(),
        records: records.into_values().collect(),
    };
    plan.valid()?;
    plan.to_json()?;
    Ok(Some(plan))
}
fn report(
    p: &Profile,
    plan: &Plan,
    selection: &Select,
    ns: &mut Namespace,
    receipt: Receipt,
    hold: Option<&'static str>,
) -> Report {
    let version = Version {
        revision: receipt.source_revision,
        selection: selection.selection.clone(),
    };
    let current = p
        .state(ns)
        .ok()
        .map(|s| s.current.as_ref() == Some(&version));
    Report {
        root: plan.root.clone(),
        version,
        receipt,
        current,
        post_effect_hold: hold.or(if current.is_none() {
            Some("needs_recovery")
        } else {
            None
        }),
        media_custody_verified: None,
        authorizes_eviction: false,
    }
}
/// Read original status without reselecting its historical root.
pub fn status(
    p: &Profile,
    plan: &Plan,
    ns: &mut Namespace,
    nodes: &MetadataStore,
) -> Result<Option<Report>> {
    p.check(ns, nodes)?;
    let selection = plan.selection(p)?;
    let receipt = ns.selection_status(&selection)?;
    Ok(receipt.map(|r| report(p, plan, &selection, ns, r, None)))
}
/// Persist the original Plan in caller-owned durable evidence before invoking.
pub fn apply(
    p: &Profile,
    plan: &Plan,
    ns: &mut Namespace,
    nodes: &mut MetadataStore,
    mut admit: impl FnMut() -> Result<()>,
) -> Result<Report> {
    if let Some(old) = status(p, plan, ns, nodes)? {
        return Ok(old);
    }
    if !p.scope.allow_store {
        return Err(Error::Denied);
    }
    let selection = plan.selection(p)?;
    let state = p.state(ns)?;
    if state.pending.is_some() || state.revision != plan.revision || state.current != plan.expected
    {
        return Err(Error::Stale);
    }
    let mut meter = Meter {
        nodes,
        used: RefCell::new(TreeMetrics::default()),
        cap: limits(p),
        start: Instant::now(),
        millis: p.limits.millis,
    };
    let mut edits = BTreeMap::new();
    for (object, raw) in &plan.records {
        let (record, _) = Index::from_root(plan.root.clone()).get(object, &mut meter, limits(p))?;
        let record = record.ok_or(Error::Missing)?;
        if record.metadata_object()? != *raw {
            return Err(Error::Integrity);
        }
        if let Some(base) = &plan.expected {
            let (previous, _) =
                Index::from_root(base.selection.root.clone()).get(object, &mut meter, limits(p))?;
            if previous.is_some_and(|old| old.producer != record.producer) {
                return Err(Error::Conflict);
            }
        }
        edits.insert(key(object)?, Some(raw.clone()));
    }
    // Rebuild only the declared upserts. This detects hidden edits/removals in
    // a parsed plan, using read-only verification of nodes already finished.
    let mut verify = Verify(&mut meter);
    let base = match &plan.expected {
        Some(v) => Tree::from_root(v.selection.root.clone()),
        None => Tree::empty(&mut verify)?,
    };
    let (rebuilt, _) = base.edit(base.root(), &edits, &mut verify, limits(p))?;
    if rebuilt.root() != &plan.root {
        return Err(Error::Integrity);
    }
    admit()?;
    meter.timed()?;
    let receipt = ns.select(&selection, &meter, &p.limits)?;
    let hold = admit().err().map(Error::code);
    Ok(report(p, plan, &selection, ns, receipt, hold))
}

/// Live generation guard for one explicitly enrolled metadata index head.
/// Dropping this handle never withdraws the durable index/history obligations.
pub struct HeldRoot {
    generation: source_use::Use,
}
impl HeldRoot {
    pub fn version(&self) -> &Version {
        self.generation.version()
    }
    pub fn revision(&self) -> u64 {
        self.generation.revision()
    }
}
impl Profile {
    /// Read-only named head resolution; no lookup of another source or latest asset.
    pub fn hold_current(&self, ns: &mut Namespace, nodes: &MetadataReader) -> Result<HeldRoot> {
        self.limits.validate().map_err(|_| Error::Budget)?;
        self.scope
            .check_metadata(ns, nodes.checkpoint().identity())?;
        let state = self.state(ns)?;
        if state.pending.is_some() {
            return Err(Error::Stale);
        }
        let current = state.current.as_ref().ok_or(Error::ProtectionLost)?;
        if current.selection.root.bytes == 0 || current.selection.root.bytes > 65536 {
            return Err(Error::Budget);
        }
        let generation = ns
            .source_use(
                &self.scope.prefix,
                &self.scope.owner,
                state.revision,
                current,
            )
            .map_err(|e| match e {
                source_use::UseError::Unsupported => Error::Unsupported,
                source_use::UseError::Owner => Error::Denied,
                source_use::UseError::Inactive => Error::ProtectionLost,
                source_use::UseError::Stale => Error::Stale,
                source_use::UseError::Admission(e) => Error::from(e),
            })?;
        Ok(HeldRoot { generation })
    }
}
