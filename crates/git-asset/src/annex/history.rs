//! Exact historical-tree metadata archival. No media transfer or removal authority.
use super::{
    pending,
    publication::{Error, Result},
    snapshot::Snapshot,
};
use git_asset_core::{
    Digest, LogicalPath, Object, identity,
    tree::{NodeStore, Tree, TreeError, TreeLimits, TreeMetrics, TreeResult},
};
use git_asset_store::{
    Key,
    metadata::MetadataStore,
    protection::Limits,
    source_claims::{
        Command, Kind, Receipt, Selection, Version,
        admission::{Namespace, Select, source_use},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Working {
    pub namespace: Digest,
    pub source: Key,
    pub owner: Key,
    pub class: Kind,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub namespace: Digest,
    pub owner: Key,
    pub place: Key,
    pub prefix: Key,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub working: Working,
    pub target: Target,
    pub records: pending::Scope,
    pub limits: Limits,
    #[serde(default)]
    pub allow_publish: bool,
    #[serde(default)]
    pub allow_release: bool,
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    let raw = identity::canonical_json(&serde_json::to_value(value).map_err(|_| Error::Invalid)?)
        .map_err(|_| Error::Invalid)?;
    if raw.len() > 65536 {
        return Err(Error::Budget);
    }
    Ok(raw)
}
fn digest(value: &impl Serialize) -> Result<Digest> {
    Ok(Digest::blob(&encode(value)?))
}
fn key(prefix: &Key, id: &Digest) -> Result<Key> {
    Key::parse(&format!("{}-{}", prefix.as_str(), id.as_str())).map_err(|_| Error::Invalid)
}
fn path() -> LogicalPath {
    LogicalPath::parse("history").unwrap()
}
fn use_error(e: source_use::UseError) -> Error {
    match e {
        source_use::UseError::Unsupported => Error::Unsupported,
        source_use::UseError::Admission(e) => e.into(),
        source_use::UseError::Owner => Error::Denied,
        source_use::UseError::Inactive => Error::ProtectionLost,
        source_use::UseError::Stale => Error::Stale,
    }
}
fn tree_limits(limits: &Limits, used: &TreeMetrics) -> Result<TreeLimits> {
    Ok(TreeLimits {
        max_entries: limits.bindings,
        max_reads: limits.reads.checked_sub(used.reads).ok_or(Error::Budget)?,
        max_read_bytes: limits
            .read_bytes
            .checked_sub(used.read_bytes)
            .ok_or(Error::Budget)?,
        max_writes: limits.reads.checked_sub(used.writes).ok_or(Error::Budget)?,
        max_write_bytes: limits
            .read_bytes
            .checked_sub(used.write_bytes)
            .ok_or(Error::Budget)?,
        ..TreeLimits::default()
    })
}
fn add(a: &mut TreeMetrics, b: &TreeMetrics) {
    a.reads += b.reads;
    a.read_bytes += b.read_bytes;
    a.writes += b.writes;
    a.write_bytes += b.write_bytes;
}
impl Profile {
    /// Stable location/owner binding; read-only recovery can disable permissions.
    pub fn binding(&self) -> Result<Digest> {
        self.limits.validate().map_err(|_| Error::Budget)?;
        if !matches!(self.working.class, Kind::Branch | Kind::Draft)
            || self.target.prefix.as_str().len() > 31
            || self.target.namespace == self.working.namespace
            || [
                self.target.namespace.clone(),
                self.working.namespace.clone(),
            ]
            .contains(&self.records.namespace)
        {
            return Err(Error::Denied);
        }
        digest(&(
            "git-asset.history-binding.experimental-1",
            &self.working,
            &self.target,
            self.records.id()?,
        ))
    }
    pub fn archive_id(&self, origin: &Version) -> Result<Digest> {
        digest(&(
            "git-asset.history-root.experimental-1",
            self.binding()?,
            origin,
        ))
    }
    fn check_target(&self, media: &Namespace) -> Result<()> {
        if media.identity() != &self.target.namespace {
            return Err(Error::Stale);
        }
        if !media.supports_source_use() {
            return Err(Error::Unsupported);
        }
        let policy = media
            .policy()
            .classes
            .get(&Kind::Retention)
            .ok_or(Error::Denied)?;
        if [&policy.current, &policy.pending, &policy.predecessor]
            .iter()
            .any(|p| !p.contains(&self.target.place))
        {
            return Err(Error::Denied);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    schema: String,
    profile: Digest,
    origin: Version,
    revision: u64,
    current: Version,
    archive: Digest,
}
impl Plan {
    pub fn capture(profile: &Profile, working: &mut Namespace, origin: &Version) -> Result<Self> {
        let binding = profile.binding()?;
        if working.identity() != &profile.working.namespace {
            return Err(Error::Denied);
        }
        if !working.supports_source_use() {
            return Err(Error::Unsupported);
        }
        let source = working
            .source(&profile.working.source)?
            .ok_or(Error::Missing)?;
        if source.owner != profile.working.owner
            || source.kind.as_ref() != Some(&profile.working.class)
        {
            return Err(Error::Denied);
        }
        if source.pending.is_some() {
            return Err(Error::NeedsRecovery);
        }
        if !source.predecessors.contains(origin) {
            return Err(Error::ProtectionLost);
        }
        let plan = Self {
            schema: "git-asset.history-plan.experimental-1".into(),
            profile: binding,
            origin: origin.clone(),
            revision: source.revision,
            current: source.current.ok_or(Error::ProtectionLost)?,
            archive: profile.archive_id(origin)?,
        };
        plan.to_json()?;
        Ok(plan)
    }
    pub fn origin(&self) -> &Version {
        &self.origin
    }
    pub fn expected_revision(&self) -> u64 {
        self.revision
    }
    pub fn id(&self) -> Result<Digest> {
        Ok(Digest::blob(&self.to_json()?))
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        if self.schema != "git-asset.history-plan.experimental-1"
            || self.origin.revision == 0
            || self.current.revision <= self.origin.revision
            || self.revision < self.current.revision
        {
            return Err(Error::Invalid);
        }
        encode(self)
    }
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > 65536 {
            return Err(Error::Budget);
        }
        let p: Self =
            serde_json::from_value(identity::parse_json(raw).map_err(|_| Error::Invalid)?)
                .map_err(|_| Error::Invalid)?;
        if p.to_json()? != raw {
            return Err(Error::Integrity);
        }
        Ok(p)
    }
    fn bound(&self, profile: &Profile) -> Result<()> {
        self.to_json()?;
        if self.profile != profile.binding()? || self.archive != profile.archive_id(&self.origin)? {
            return Err(Error::Stale);
        }
        Ok(())
    }
    fn operation(&self, phase: &str) -> Result<Key> {
        key(&Key::parse(phase).map_err(|_| Error::Invalid)?, &self.id()?)
    }
    fn select_media(&self, p: &Profile) -> Result<Select> {
        Ok(Select {
            prepare: key(&Key::parse("history-prepare").unwrap(), &self.archive)?,
            adopt: key(&Key::parse("history-adopt").unwrap(), &self.archive)?,
            source: key(&p.target.prefix, &self.archive)?,
            owner: p.target.owner.clone(),
            expected_revision: 1,
            selection: Selection {
                root: self.origin.selection.root.clone(),
                context: self.archive.clone(),
            },
        })
    }
    fn retire(&self, p: &Profile) -> Command {
        Command::Retire {
            source: p.working.source.clone(),
            owner: p.working.owner.clone(),
            expected_revision: self.revision,
            predecessor_revision: self.origin.revision,
        }
    }
}
pub struct Destinations<'a> {
    pub media: &'a mut Namespace,
    pub records: &'a mut Namespace,
}
/// Owns record-generation protection. Durable claims outlive this live handle.
pub struct Loaded {
    saved: pending::Saved,
    plan: Plan,
    media: Version,
    _record: source_use::Use,
}
#[derive(Serialize)]
pub struct Report {
    pub archive: Digest,
    pub receipt: Receipt,
    pub historical: bool,
    pub media_custody_verified: Option<bool>,
    pub authorizes_eviction: bool,
}
fn retained_version(p: &Profile, plan: &Plan, media: &mut Namespace) -> Result<Version> {
    p.check_target(media)?;
    let select = plan.select_media(p)?;
    let receipt = media
        .selection_status(&select)?
        .ok_or(Error::NeedsRecovery)?;
    let version = Version {
        revision: receipt.source_revision,
        selection: select.selection,
    };
    let source = media.source(&select.source)?.ok_or(Error::ProtectionLost)?;
    if source.owner != p.target.owner
        || source.kind != Some(Kind::Retention)
        || !source
            .current
            .iter()
            .chain(&source.predecessors)
            .any(|v| v == &version)
    {
        return Err(Error::ProtectionLost);
    }
    Ok(version)
}
pub fn load(
    p: &Profile,
    saved: &pending::Saved,
    d: Destinations<'_>,
    nodes: &MetadataStore,
) -> Result<Loaded> {
    load_meter(p, saved, d, nodes, &mut TreeMetrics::default())
}
fn load_meter(
    p: &Profile,
    saved: &pending::Saved,
    d: Destinations<'_>,
    nodes: &MetadataStore,
    used: &mut TreeMetrics,
) -> Result<Loaded> {
    p.binding()?;
    p.records.check(d.records, nodes)?;
    if saved.namespace != p.records.namespace {
        return Err(Error::Stale);
    }
    let state = d.records.source(&saved.source)?.ok_or(Error::Missing)?;
    if state.owner != p.records.owner || state.kind != Some(Kind::Job) {
        return Err(Error::Denied);
    }
    if state.pending.is_some()
        || !state
            .current
            .iter()
            .chain(&state.predecessors)
            .any(|v| v == &saved.version)
    {
        return Err(Error::Stale);
    }
    let held = d
        .records
        .source_use(
            &saved.source,
            &p.records.owner,
            state.revision,
            state.current.as_ref().ok_or(Error::ProtectionLost)?,
        )
        .map_err(use_error)?;
    let mut read = nodes.finished_reader().map_err(|_| Error::NeedsRecovery)?;
    let (entries, metrics) = Tree::from_root(saved.version.selection.root.clone())
        .entries(&mut read, tree_limits(&p.limits, used)?)?;
    add(used, &metrics);
    if entries.len() != 1
        || entries.get(&path()) != Some(&saved.record)
        || saved.version.selection.context != saved.record.sha256
    {
        return Err(Error::Integrity);
    }
    let len = usize::try_from(saved.record.bytes).map_err(|_| Error::Budget)?;
    if len > 65536
        || used.reads >= p.limits.reads
        || len > p.limits.read_bytes.saturating_sub(used.read_bytes)
    {
        return Err(Error::Budget);
    }
    let raw = read.get(&saved.record)?.ok_or(Error::Missing)?;
    if Object::from_bytes(&raw) != saved.record {
        return Err(Error::Integrity);
    }
    used.reads += 1;
    used.read_bytes += raw.len();
    let plan = Plan::from_json(&raw)?;
    plan.bound(p)?;
    if key(&p.records.prefix, &plan.archive)? != saved.source {
        return Err(Error::Integrity);
    }
    let media = retained_version(p, &plan, d.media)?;
    Ok(Loaded {
        saved: saved.clone(),
        plan,
        media,
        _record: held,
    })
}
pub fn find(
    p: &Profile,
    origin: &Version,
    d: Destinations<'_>,
    nodes: &MetadataStore,
) -> Result<Option<Loaded>> {
    find_archive(p, &p.archive_id(origin)?, d, nodes)
}
/// Point recovery by stable archive ID, including after its working predecessor retired.
pub fn find_archive(
    p: &Profile,
    archive: &Digest,
    d: Destinations<'_>,
    nodes: &MetadataStore,
) -> Result<Option<Loaded>> {
    p.binding()?;
    p.records.check(d.records, nodes)?;
    let name = key(&p.records.prefix, archive)?;
    let Some(state) = d.records.source(&name)? else {
        return Ok(None);
    };
    if state.owner != p.records.owner || state.kind != Some(Kind::Job) {
        return Err(Error::Denied);
    }
    if state.pending.is_some() {
        return Err(Error::NeedsRecovery);
    }
    let Some(version) = state.current else {
        if state.revision == 1 && state.predecessors.is_empty() {
            return Ok(None);
        }
        return Err(Error::ProtectionLost);
    };
    let mut reader = nodes.finished_reader().map_err(|_| Error::NeedsRecovery)?;
    let (object, mut used) = Tree::from_root(version.selection.root.clone()).get(
        &path(),
        &mut reader,
        tree_limits(&p.limits, &TreeMetrics::default())?,
    )?;
    let saved = pending::Saved {
        namespace: p.records.namespace.clone(),
        source: name,
        version,
        record: object.ok_or(Error::Integrity)?,
    };
    let loaded = load_meter(p, &saved, d, nodes, &mut used)?;
    if &loaded.plan.archive != archive {
        return Err(Error::Integrity);
    }
    Ok(Some(loaded))
}
struct Mirror<'a, N> {
    source: &'a N,
    destination: RefCell<&'a mut MetadataStore>,
    writes: Cell<usize>,
    bytes: Cell<usize>,
    limits: &'a Limits,
    start: Instant,
}
impl<N: NodeStore> NodeStore for Mirror<'_, N> {
    fn get(&self, object: &Object) -> TreeResult<Option<Vec<u8>>> {
        let size = usize::try_from(object.bytes).map_err(|_| TreeError::Budget)?;
        if self.start.elapsed() >= Duration::from_millis(self.limits.millis)
            || self.writes.get() >= self.limits.reads
            || size > self.limits.read_bytes.saturating_sub(self.bytes.get())
        {
            return Err(TreeError::Budget);
        }
        let raw = self
            .source
            .get(object)?
            .ok_or_else(|| TreeError::MissingNode(object.clone()))?;
        if Object::from_bytes(&raw) != *object {
            return Err(TreeError::Integrity);
        }
        if self.destination.borrow_mut().put(&raw)? != *object {
            return Err(TreeError::Integrity);
        }
        self.writes.set(self.writes.get() + 1);
        self.bytes.set(self.bytes.get() + size);
        Ok(Some(raw))
    }
    fn put(&mut self, _: &[u8]) -> TreeResult<Object> {
        Err(TreeError::Store("read-only traversal".into()))
    }
}
#[derive(Clone, Copy)]
pub enum Stage {
    MetadataFinished,
    MediaAdopted,
    RecordEnrolled,
    Recorded,
}
pub fn publish<N: NodeStore>(
    p: &Profile,
    plan: &Plan,
    working: &mut Namespace,
    d: Destinations<'_>,
    input: &N,
    nodes: &mut MetadataStore,
    mut admit: impl FnMut(Stage) -> Result<()>,
) -> Result<Loaded> {
    plan.bound(p)?;
    p.records.check(d.records, nodes)?;
    p.check_target(d.media)?;
    let start = Instant::now();
    nodes.finished_reader().map_err(|_| Error::NeedsRecovery)?;
    if let Some(old) = find(
        p,
        &plan.origin,
        Destinations {
            media: d.media,
            records: d.records,
        },
        nodes,
    )? {
        return Ok(old);
    }
    if !p.allow_publish || !p.records.allow_store {
        return Err(Error::Denied);
    }
    if working.identity() != &p.working.namespace {
        return Err(Error::Denied);
    }
    let _working = working
        .source_use(
            &p.working.source,
            &p.working.owner,
            plan.revision,
            &plan.current,
        )
        .map_err(use_error)?;
    let state = working.source(&p.working.source)?.ok_or(Error::Missing)?;
    if state.kind.as_ref() != Some(&p.working.class) || !state.predecessors.contains(&plan.origin) {
        return Err(Error::Stale);
    }
    let (original, mut used) = {
        let mut mirror = Mirror {
            source: input,
            destination: RefCell::new(&mut *nodes),
            writes: Cell::new(0),
            bytes: Cell::new(0),
            limits: &p.limits,
            start,
        };
        let (original, mut used) = Tree::from_root(plan.origin.selection.root.clone()).entries(
            &mut mirror,
            tree_limits(&p.limits, &TreeMetrics::default())?,
        )?;
        used.writes = mirror.writes.get();
        used.write_bytes = mirror.bytes.get();
        (original, used)
    };
    let raw = plan.to_json()?;
    if used.writes >= p.limits.reads
        || raw.len() > p.limits.read_bytes.saturating_sub(used.write_bytes)
    {
        return Err(Error::Budget);
    }
    let object = nodes.put(&raw)?;
    if object != Object::from_bytes(&raw) {
        return Err(Error::Integrity);
    }
    used.writes += 1;
    used.write_bytes += raw.len();
    let (record, metrics) = Tree::build(
        &BTreeMap::from([(path(), object.clone())]),
        nodes,
        tree_limits(&p.limits, &used)?,
    )?;
    add(&mut used, &metrics);
    nodes.finish().map_err(|_| Error::NeedsRecovery)?;
    let mut read = nodes.finished_reader().map_err(|_| Error::NeedsRecovery)?;
    let (checked, metrics) = Tree::from_root(plan.origin.selection.root.clone())
        .entries(&mut read, tree_limits(&p.limits, &used)?)?;
    add(&mut used, &metrics);
    if checked != original {
        return Err(Error::Integrity);
    }
    let (checked, metrics) = record.entries(&mut read, tree_limits(&p.limits, &used)?)?;
    add(&mut used, &metrics);
    if checked.get(&path()) != Some(&object)
        || used.reads >= p.limits.reads
        || raw.len() > p.limits.read_bytes.saturating_sub(used.read_bytes)
    {
        return Err(Error::Budget);
    }
    if read.get(&object)?.as_deref() != Some(&raw) {
        return Err(Error::Integrity);
    }
    used.reads += 1;
    used.read_bytes += raw.len();
    if start.elapsed() >= Duration::from_millis(p.limits.millis) {
        return Err(Error::Budget);
    }
    admit(Stage::MetadataFinished)?;
    let selection = plan.select_media(p)?;
    d.media.apply(
        key(&Key::parse("history-enroll").unwrap(), &plan.archive)?,
        Command::Enroll {
            source: selection.source.clone(),
            owner: p.target.owner.clone(),
            class: Kind::Retention,
        },
        &read,
        &p.limits,
    )?;
    d.media.select(&selection, &read, &p.limits)?;
    admit(Stage::MediaAdopted)?;
    if start.elapsed() >= Duration::from_millis(p.limits.millis) {
        return Err(Error::Budget);
    }
    let name = key(&p.records.prefix, &plan.archive)?;
    d.records.apply(
        key(&Key::parse("history-record-enroll").unwrap(), &plan.archive)?,
        Command::Enroll {
            source: name.clone(),
            owner: p.records.owner.clone(),
            class: Kind::Job,
        },
        &read,
        &p.limits,
    )?;
    admit(Stage::RecordEnrolled)?;
    let selection = Select {
        prepare: plan.operation("history-record-prepare")?,
        adopt: plan.operation("history-record-adopt")?,
        source: name.clone(),
        owner: p.records.owner.clone(),
        expected_revision: 1,
        selection: Selection {
            root: record.root().clone(),
            context: object.sha256.clone(),
        },
    };
    let receipt = d.records.select(&selection, &read, &p.limits)?;
    let saved = pending::Saved {
        namespace: p.records.namespace.clone(),
        source: name,
        version: Version {
            revision: receipt.source_revision,
            selection: selection.selection,
        },
        record: object,
    };
    admit(Stage::Recorded)?;
    load_meter(p, &saved, d, nodes, &mut used)
}
/// Explicitly refresh an unexecuted metadata release after Main advanced.
/// Old records stay retained; this is not replacement of a native eviction plan.
pub fn refresh(
    p: &Profile,
    previous: Loaded,
    working: &mut Namespace,
    d: Destinations<'_>,
    nodes: &mut MetadataStore,
    mut admit: impl FnMut(Stage) -> Result<()>,
) -> Result<Loaded> {
    let start = Instant::now();
    previous.plan.bound(p)?;
    p.records.check(d.records, nodes)?;
    if previous._record.version() != &previous.saved.version {
        return Err(Error::Stale);
    }
    if working.identity() != &p.working.namespace {
        return Err(Error::Denied);
    }
    if working
        .command_status(
            previous.plan.operation("history-release")?,
            previous.plan.retire(p),
        )?
        .is_some()
    {
        return Ok(previous);
    }
    if !p.allow_publish || !p.records.allow_store {
        return Err(Error::Denied);
    }
    let plan = Plan::capture(p, working, &previous.plan.origin)?;
    if plan.revision == previous.plan.revision {
        return Ok(previous);
    }
    if plan.revision < previous.plan.revision {
        return Err(Error::Stale);
    }
    let _working = working
        .source_use(
            &p.working.source,
            &p.working.owner,
            plan.revision,
            &plan.current,
        )
        .map_err(use_error)?;
    retained_version(p, &previous.plan, d.media)?;
    let media_select = plan.select_media(p)?;
    let media_state = d
        .media
        .source(&media_select.source)?
        .ok_or(Error::ProtectionLost)?;
    let _media = d
        .media
        .source_use(
            &media_select.source,
            &p.target.owner,
            media_state.revision,
            media_state.current.as_ref().ok_or(Error::ProtectionLost)?,
        )
        .map_err(use_error)?;
    nodes.finished_reader().map_err(|_| Error::NeedsRecovery)?;
    let raw = plan.to_json()?;
    if p.limits.reads < 1 || raw.len() > p.limits.read_bytes {
        return Err(Error::Budget);
    }
    let object = nodes.put(&raw)?;
    if object != Object::from_bytes(&raw) {
        return Err(Error::Integrity);
    }
    let mut used = TreeMetrics {
        writes: 1,
        write_bytes: raw.len(),
        ..TreeMetrics::default()
    };
    let (tree, metrics) = Tree::build(
        &BTreeMap::from([(path(), object.clone())]),
        nodes,
        tree_limits(&p.limits, &used)?,
    )?;
    add(&mut used, &metrics);
    nodes.finish().map_err(|_| Error::NeedsRecovery)?;
    let mut reader = nodes.finished_reader().map_err(|_| Error::NeedsRecovery)?;
    let (entries, metrics) = tree.entries(&mut reader, tree_limits(&p.limits, &used)?)?;
    add(&mut used, &metrics);
    if entries.get(&path()) != Some(&object)
        || used.reads >= p.limits.reads
        || raw.len() > p.limits.read_bytes.saturating_sub(used.read_bytes)
    {
        return Err(Error::Budget);
    }
    if reader.get(&object)?.as_deref() != Some(&raw) {
        return Err(Error::Integrity);
    }
    used.reads += 1;
    used.read_bytes += raw.len();
    if start.elapsed() >= Duration::from_millis(p.limits.millis) {
        return Err(Error::Budget);
    }
    admit(Stage::MetadataFinished)?;
    let revision = previous._record.revision();
    let name = previous.saved.source.clone();
    drop(previous);
    let select = Select {
        prepare: plan.operation("history-record-prepare")?,
        adopt: plan.operation("history-record-adopt")?,
        source: name.clone(),
        owner: p.records.owner.clone(),
        expected_revision: revision,
        selection: Selection {
            root: tree.root().clone(),
            context: object.sha256.clone(),
        },
    };
    let receipt = d.records.select(&select, &reader, &p.limits)?;
    let saved = pending::Saved {
        namespace: p.records.namespace.clone(),
        source: name,
        version: Version {
            revision: receipt.source_revision,
            selection: select.selection,
        },
        record: object,
    };
    admit(Stage::Recorded)?;
    load_meter(p, &saved, d, nodes, &mut used)
}
impl Loaded {
    pub fn saved(&self) -> &pending::Saved {
        &self.saved
    }
    pub fn plan(&self) -> &Plan {
        &self.plan
    }
    pub fn release_status(&self, p: &Profile, working: &mut Namespace) -> Result<Option<Receipt>> {
        self.plan.bound(p)?;
        if working.identity() != &p.working.namespace {
            return Err(Error::Denied);
        }
        working
            .command_status(self.plan.operation("history-release")?, self.plan.retire(p))
            .map_err(Error::from)
    }
    /// One exact historical binding, with no current/latest substitution or payload IO.
    pub fn member(
        &self,
        p: &Profile,
        path: LogicalPath,
        nodes: &mut impl NodeStore,
    ) -> Result<Snapshot> {
        self.plan.bound(p)?;
        let (object, _) = Tree::from_root(self.media.selection.root.clone()).get(
            &path,
            nodes,
            tree_limits(&p.limits, &TreeMetrics::default())?,
        )?;
        Ok(Snapshot {
            namespace: p.target.namespace.clone(),
            source: self.plan.select_media(p)?.source,
            owner: p.target.owner.clone(),
            version: self.media.clone(),
            place: p.target.place.clone(),
            path,
            object: object.ok_or(Error::Missing)?,
        })
    }
    /// Release only this old metadata obligation after full retained-tree readback.
    /// Native bytes remain unknown; this receipt never grants physical eviction.
    pub fn release<F: NodeStore, M: NodeStore>(
        &self,
        p: &Profile,
        working: &mut Namespace,
        media: &mut Namespace,
        nodes: (&F, &mut M),
        mut admit: impl FnMut() -> Result<()>,
    ) -> Result<Report> {
        self.plan.bound(p)?;
        let start = Instant::now();
        if self.saved.record != Object::from_bytes(&self.plan.to_json()?)
            || self._record.namespace() != &self.saved.namespace
            || self._record.source() != &self.saved.source
            || self.saved.version.selection.context != self.saved.record.sha256
            || working.identity() != &p.working.namespace
        {
            return Err(Error::Integrity);
        }
        let operation = self.plan.operation("history-release")?;
        let command = self.plan.retire(p);
        if let Some(receipt) = working.command_status(operation.clone(), command.clone())? {
            return Ok(Report {
                archive: self.plan.archive.clone(),
                receipt,
                historical: true,
                media_custody_verified: None,
                authorizes_eviction: false,
            });
        }
        if !p.allow_release {
            return Err(Error::Denied);
        }
        let version = retained_version(p, &self.plan, media)?;
        let select = self.plan.select_media(p)?;
        let state = media.source(&select.source)?.ok_or(Error::ProtectionLost)?;
        let held = media
            .source_use(
                &select.source,
                &p.target.owner,
                state.revision,
                state.current.as_ref().ok_or(Error::ProtectionLost)?,
            )
            .map_err(use_error)?;
        Tree::from_root(version.selection.root)
            .entries(nodes.1, tree_limits(&p.limits, &TreeMetrics::default())?)?;
        if start.elapsed() >= Duration::from_millis(p.limits.millis) {
            return Err(Error::Budget);
        }
        let current = working
            .source_use(
                &p.working.source,
                &p.working.owner,
                self.plan.revision,
                &self.plan.current,
            )
            .map_err(use_error)?;
        admit()?;
        drop(current);
        admit()?;
        let receipt = working.apply(operation, command, nodes.0, &p.limits)?;
        drop(held);
        Ok(Report {
            archive: self.plan.archive.clone(),
            receipt,
            historical: false,
            media_custody_verified: None,
            authorizes_eviction: false,
        })
    }
}
#[cfg(test)]
mod tests;
