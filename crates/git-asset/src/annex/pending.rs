//! Original immutable action bodies over existing metadata and scoped source claims.
//! No native effect or deletion authority, independent log, timeout or cache.
use super::{
    publication::{self, Error, Result},
    retirement::planning,
};
use git_asset_core::{
    Digest, LogicalPath, Object, identity,
    tree::{NodeStore, Tree, TreeLimits, TreeMetrics},
};
use git_asset_store::{
    Key,
    metadata::MetadataStore,
    protection::Limits,
    source_claims::{
        Kind, Version,
        admission::{Namespace, Select, source_use},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub namespace: Digest,
    pub metadata: Digest,
    pub owner: Key,
    pub prefix: Key,
    pub place: Key,
    #[serde(default)]
    pub allow_store: bool,
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    let bytes = identity::canonical_json(&serde_json::to_value(value).map_err(|_| Error::Invalid)?)
        .map_err(|_| Error::Invalid)?;
    if bytes.len() > 65536 {
        return Err(Error::Budget);
    }
    Ok(bytes)
}
impl Scope {
    pub fn id(&self) -> Result<Digest> {
        if self.prefix.as_str().len() > 31 {
            return Err(Error::Invalid);
        }
        Ok(Digest::blob(&encode(&(
            "git-asset.native-pending-scope.experimental-1",
            &self.namespace,
            &self.metadata,
            &self.owner,
            &self.prefix,
            &self.place,
        ))?))
    }
    pub(super) fn check(&self, ns: &Namespace, nodes: &MetadataStore) -> Result<()> {
        self.check_metadata(ns, nodes.checkpoint().identity())
    }
    pub(super) fn check_metadata(&self, ns: &Namespace, metadata: &Digest) -> Result<()> {
        self.id()?;
        if ns.identity() != &self.namespace || metadata != &self.metadata {
            return Err(Error::Stale);
        }
        if !ns.supports_source_use() {
            return Err(Error::Unsupported);
        }
        let policy = ns.policy().classes.get(&Kind::Job).ok_or(Error::Denied)?;
        if [&policy.current, &policy.pending, &policy.predecessor]
            .iter()
            .any(|p| p.len() != 1 || p[0] != self.place)
        {
            return Err(Error::Denied);
        }
        Ok(())
    }
    fn key(&self, kind: Phase, object: &Object, producer: &Digest) -> Result<Key> {
        let id = Digest::blob(&encode(&(self.id()?, kind, object, producer))?);
        Key::parse(&format!("{}-{}", self.prefix.as_str(), id.as_str())).map_err(|_| Error::Invalid)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Handoff,
    Eviction,
    Restore,
    RestoreIndex,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreAction {
    pub eviction: planning::Plan,
    pub restore: publication::restore::Plan,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreIndexAction {
    pub original: RestoreAction,
    pub index: super::producer_index::update::restore::Plan,
}
#[derive(Clone, Debug)]
pub enum Operation {
    Handoff(Box<publication::handoff::Plan>),
    Eviction(Box<planning::Plan>),
    Restore(Box<RestoreAction>),
    RestoreIndex(Box<RestoreIndexAction>),
}
#[derive(Clone, Debug)]
pub struct Action {
    pub operation: Operation,
    pub working: publication::Plan,
    pub retained: publication::Plan,
}
impl Action {
    pub fn phase(&self) -> Phase {
        match self.operation {
            Operation::Handoff(_) => Phase::Handoff,
            Operation::Eviction(_) => Phase::Eviction,
            Operation::Restore(_) => Phase::Restore,
            Operation::RestoreIndex(_) => Phase::RestoreIndex,
        }
    }
    pub fn object(&self) -> &Object {
        match &self.operation {
            Operation::Handoff(p) => p.object(),
            Operation::Eviction(p) => p.object(),
            Operation::Restore(p) => &p.restore.publication().candidate().object,
            Operation::RestoreIndex(p) => &p.original.restore.publication().candidate().object,
        }
    }
    fn body(&self) -> Result<Vec<u8>> {
        if self.working.candidate().object != *self.object()
            || self.retained.candidate().object != *self.object()
        {
            return Err(Error::Integrity);
        }
        let working = self.working.id()?;
        let retained = self.retained.id()?;
        match &self.operation {
            Operation::Handoff(p) => {
                if p.source_plan() != &working || p.keeper_plan() != &retained {
                    return Err(Error::Integrity);
                }
                p.to_json()
            }
            Operation::Eviction(p) => {
                if p.source_producer() != &working || p.keeper_producer() != &retained {
                    return Err(Error::Integrity);
                }
                p.to_json().map_err(|e| match e {
                    super::retirement::Error::Operation(e) => e,
                    _ => Error::Invalid,
                })
            }
            Operation::Restore(p) => {
                self.restore_binding(p)?;
                encode(p)
            }
            Operation::RestoreIndex(p) => {
                self.restore_binding(&p.original)?;
                if p.index.restoration().to_json()? != p.original.restore.to_json()? {
                    return Err(Error::Integrity);
                }
                p.index.to_json()?;
                encode(p)
            }
        }
    }
    fn restore_binding(&self, p: &RestoreAction) -> Result<()> {
        if p.eviction.source_producer() != &self.working.id()?
            || p.eviction.keeper_producer() != &self.retained.id()?
        {
            return Err(Error::Integrity);
        }
        p.restore.check_original(publication::restore::Original {
            eviction: &p.eviction,
            publication: &self.working,
        })?;
        p.restore.to_json()?;
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    scope: Digest,
    policy: Digest,
    commit: String,
    phase: Phase,
    object: Object,
    producer: Digest,
    body: Object,
    working: Object,
    retained: Object,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Saved {
    pub namespace: Digest,
    pub source: Key,
    pub version: Version,
    pub record: Object,
}
pub struct Loaded {
    pub saved: Saved,
    pub policy: Digest,
    pub commit: String,
    pub action: Action,
    _generation: source_use::Use,
}
fn path(s: &str) -> LogicalPath {
    LogicalPath::parse(s).expect("fixed pending metadata path")
}
fn tree_limits(limits: &Limits) -> TreeLimits {
    TreeLimits {
        max_entries: 8,
        max_reads: limits.reads,
        max_read_bytes: limits.read_bytes,
        max_writes: 32,
        max_write_bytes: 65536,
        ..TreeLimits::default()
    }
}
fn read(
    nodes: &MetadataStore,
    object: &Object,
    meter: &mut TreeMetrics,
    limits: &Limits,
) -> Result<Vec<u8>> {
    let len = usize::try_from(object.bytes).map_err(|_| Error::Budget)?;
    if len > 65536
        || meter.reads >= limits.reads
        || len > limits.read_bytes.saturating_sub(meter.read_bytes)
    {
        return Err(Error::Budget);
    }
    let bytes = nodes.get(object)?.ok_or(Error::Missing)?;
    if Object::from_bytes(&bytes) != *object {
        return Err(Error::Integrity);
    }
    meter.reads += 1;
    meter.read_bytes += len;
    Ok(bytes)
}
fn put(nodes: &mut MetadataStore, bytes: &[u8]) -> Result<Object> {
    if bytes.len() > 65536 {
        return Err(Error::Budget);
    }
    let expected = Object::from_bytes(bytes);
    let actual = nodes.put(bytes)?;
    if expected != actual {
        return Err(Error::Integrity);
    }
    Ok(actual)
}
/// Read an exact retained handle, without selecting the current plan as fallback.
pub fn reopen(
    scope: &Scope,
    saved: &Saved,
    ns: &mut Namespace,
    nodes: &MetadataStore,
    limits: &Limits,
) -> Result<Loaded> {
    limits.validate().map_err(|_| Error::Budget)?;
    scope.check(ns, nodes)?;
    if saved.namespace != scope.namespace {
        return Err(Error::Stale);
    }
    let source = ns.source(&saved.source)?.ok_or(Error::Missing)?;
    if source.owner != scope.owner || source.kind != Some(Kind::Job) {
        return Err(Error::Denied);
    }
    if !source
        .current
        .iter()
        .chain(&source.predecessors)
        .any(|v| v == &saved.version)
    {
        return Err(Error::ProtectionLost);
    }
    let current = source.current.as_ref().ok_or(Error::ProtectionLost)?;
    let generation = ns
        .source_use(&saved.source, &scope.owner, source.revision, current)
        .map_err(|e| match e {
            source_use::UseError::Unsupported => Error::Unsupported,
            source_use::UseError::Admission(e) => e.into(),
            _ => Error::Stale,
        })?;
    let mut borrowed = nodes.finished_reader().map_err(|_| Error::NeedsRecovery)?;
    let (entries, mut meter) = Tree::from_root(saved.version.selection.root.clone())
        .entries(&mut borrowed, tree_limits(limits))?;
    if entries.len() != 4
        || entries.get(&path("record")) != Some(&saved.record)
        || saved.version.selection.context != saved.record.sha256
    {
        return Err(Error::Integrity);
    }
    let raw = read(nodes, &saved.record, &mut meter, limits)?;
    let env: Envelope = serde_json::from_slice(&raw).map_err(|_| Error::Integrity)?;
    if encode(&env)? != raw
        || env.schema != "git-asset.native-pending-action.experimental-1"
        || crate::source::GitId::parse(&env.commit).is_err()
        || env.scope != scope.id()?
        || scope.key(env.phase, &env.object, &env.producer)? != saved.source
        || entries.get(&path("body")) != Some(&env.body)
        || entries.get(&path("working")) != Some(&env.working)
        || entries.get(&path("retained")) != Some(&env.retained)
    {
        return Err(Error::Integrity);
    }
    let body = read(nodes, &env.body, &mut meter, limits)?;
    let working = decode_publication(&read(nodes, &env.working, &mut meter, limits)?)?;
    let retained = decode_publication(&read(nodes, &env.retained, &mut meter, limits)?)?;
    let operation = match env.phase {
        Phase::Handoff => {
            Operation::Handoff(Box::new(publication::handoff::Plan::from_json(&body)?))
        }
        Phase::Eviction => Operation::Eviction(Box::new(
            planning::Plan::from_json(&body).map_err(|_| Error::Integrity)?,
        )),
        Phase::Restore => Operation::Restore(Box::new(
            serde_json::from_slice(&body).map_err(|_| Error::Integrity)?,
        )),
        Phase::RestoreIndex => Operation::RestoreIndex(Box::new(
            serde_json::from_slice(&body).map_err(|_| Error::Integrity)?,
        )),
    };
    let action = Action {
        operation,
        working,
        retained,
    };
    if action.body()? != body
        || action.object() != &env.object
        || action.working.id() != Ok(env.producer)
    {
        return Err(Error::Integrity);
    }
    Ok(Loaded {
        saved: saved.clone(),
        policy: env.policy,
        commit: env.commit,
        action,
        _generation: generation,
    })
}
fn decode_publication(raw: &[u8]) -> Result<publication::Plan> {
    let p: publication::Plan = serde_json::from_slice(raw).map_err(|_| Error::Integrity)?;
    if encode(&p)? != raw || p.id()? != Object::from_bytes(raw).sha256 {
        return Err(Error::Integrity);
    }
    Ok(p)
}
pub fn find(
    scope: &Scope,
    phase: Phase,
    object: &Object,
    producer: &Digest,
    ns: &mut Namespace,
    nodes: &MetadataStore,
    limits: &Limits,
) -> Result<Option<Loaded>> {
    limits.validate().map_err(|_| Error::Budget)?;
    scope.check(ns, nodes)?;
    let key = scope.key(phase, object, producer)?;
    let Some(source) = ns.source(&key)? else {
        return Ok(None);
    };
    if source.pending.is_some() {
        return Err(Error::NeedsRecovery);
    }
    if source.owner != scope.owner || source.kind != Some(Kind::Job) {
        return Err(Error::Denied);
    }
    let Some(version) = source.current else {
        return if source.predecessors.is_empty() {
            Ok(None)
        } else {
            Err(Error::ProtectionLost)
        };
    };
    // The context pins the envelope hash. Its length is read from the retained tree.
    let mut borrowed = nodes.finished_reader().map_err(|_| Error::NeedsRecovery)?;
    let (record, metrics) = Tree::from_root(version.selection.root.clone()).get(
        &path("record"),
        &mut borrowed,
        tree_limits(limits),
    )?;
    let mut remaining = limits.clone();
    remaining.reads = remaining
        .reads
        .checked_sub(metrics.reads)
        .ok_or(Error::Budget)?;
    remaining.read_bytes = remaining
        .read_bytes
        .checked_sub(metrics.read_bytes)
        .ok_or(Error::Budget)?;
    let saved = Saved {
        namespace: scope.namespace.clone(),
        source: key,
        version,
        record: record.ok_or(Error::Integrity)?,
    };
    reopen(scope, &saved, ns, nodes, &remaining).map(Some)
}
/// Store once; an existing slot always wins over a newly computed preview.
pub fn save(
    scope: &Scope,
    policy: Digest,
    commit: &str,
    action: &Action,
    ns: &mut Namespace,
    nodes: &mut MetadataStore,
    limits: &Limits,
) -> Result<Saved> {
    limits.validate().map_err(|_| Error::Budget)?;
    scope.check(ns, nodes)?;
    let producer = action.working.id()?;
    if let Some(old) = find(
        scope,
        action.phase(),
        action.object(),
        &producer,
        ns,
        nodes,
        limits,
    )? {
        return Ok(old.saved);
    }
    if !scope.allow_store {
        return Err(Error::Denied);
    }
    store(
        scope,
        policy,
        commit,
        StoreAction {
            action,
            expected: None,
        },
        ns,
        nodes,
        limits,
    )
}
pub struct RestoreIndexSuccessor<'a> {
    pub expected: &'a Saved,
    pub proposal: super::producer_index::update::restore::Replan,
}
/// Explicit metadata successor; the prior Saved version remains a retained predecessor.
pub fn save_restore_index_successor(
    scope: &Scope,
    policy: Digest,
    commit: &str,
    successor: RestoreIndexSuccessor<'_>,
    ns: &mut Namespace,
    nodes: &mut MetadataStore,
    limits: &Limits,
) -> Result<Saved> {
    let RestoreIndexSuccessor { expected, proposal } = successor;
    let old = reopen(scope, expected, ns, nodes, limits)?;
    let mut action = old.action.clone();
    let Operation::RestoreIndex(body) = &mut action.operation else {
        return Err(Error::Invalid);
    };
    if Digest::blob(&body.index.to_json()?) != proposal.original {
        return Err(Error::Integrity);
    }
    body.index = proposal.replacement;
    let raw = action.body()?;
    drop(old);
    let current = find(
        scope,
        Phase::RestoreIndex,
        action.object(),
        &action.working.id()?,
        ns,
        nodes,
        limits,
    )?
    .ok_or(Error::ProtectionLost)?;
    if &current.saved != expected {
        return if current.action.body()? == raw {
            Ok(current.saved)
        } else {
            Err(Error::Conflict)
        };
    }
    drop(current);
    if !scope.allow_store {
        return Err(Error::Denied);
    }
    store(
        scope,
        policy,
        commit,
        StoreAction {
            action: &action,
            expected: Some(expected),
        },
        ns,
        nodes,
        limits,
    )
}
struct StoreAction<'a> {
    action: &'a Action,
    expected: Option<&'a Saved>,
}
fn store(
    scope: &Scope,
    policy: Digest,
    commit: &str,
    input: StoreAction<'_>,
    ns: &mut Namespace,
    nodes: &mut MetadataStore,
    limits: &Limits,
) -> Result<Saved> {
    let StoreAction { action, expected } = input;
    let producer = action.working.id()?;
    if crate::source::GitId::parse(commit).is_err() {
        return Err(Error::Invalid);
    }
    let body = put(nodes, &action.body()?)?;
    let working = put(nodes, &encode(&action.working)?)?;
    let retained = put(nodes, &encode(&action.retained)?)?;
    let env = Envelope {
        schema: "git-asset.native-pending-action.experimental-1".into(),
        scope: scope.id()?,
        policy,
        commit: commit.into(),
        phase: action.phase(),
        object: action.object().clone(),
        producer: producer.clone(),
        body: body.clone(),
        working: working.clone(),
        retained: retained.clone(),
    };
    let record = put(nodes, &encode(&env)?)?;
    let bindings = BTreeMap::from([
        (path("record"), record.clone()),
        (path("body"), body),
        (path("working"), working),
        (path("retained"), retained),
    ]);
    let (tree, _) = Tree::build(&bindings, nodes, tree_limits(limits))?;
    nodes.finish().map_err(|_| Error::NeedsRecovery)?;
    // Read back every stored body before exposing a source head or executing work.
    let mut meter = TreeMetrics::default();
    for object in bindings.values() {
        read(nodes, object, &mut meter, limits)?;
    }
    let key = scope.key(action.phase(), action.object(), &producer)?;
    let enrol = Key::parse(&format!(
        "pending-enrol-{}",
        Digest::blob(key.as_str().as_bytes()).as_str()
    ))
    .map_err(|_| Error::Invalid)?;
    ns.apply(
        enrol,
        git_asset_store::source_claims::Command::Enroll {
            source: key.clone(),
            owner: scope.owner.clone(),
            class: Kind::Job,
        },
        nodes,
        limits,
    )?;
    let source = ns.source(&key)?.ok_or(Error::NeedsRecovery)?;
    if source.pending.is_some() || source.current.as_ref() != expected.map(|s| &s.version) {
        return Err(Error::Conflict);
    }
    let selection = git_asset_store::source_claims::Selection {
        root: tree.root().clone(),
        context: record.sha256.clone(),
    };
    let id = Digest::blob(&encode(&(&key, &selection, source.revision))?);
    let receipt = ns.select(
        &Select {
            prepare: Key::parse(&format!("pending-prepare-{}", id.as_str()))
                .map_err(|_| Error::Invalid)?,
            adopt: Key::parse(&format!("pending-adopt-{}", id.as_str()))
                .map_err(|_| Error::Invalid)?,
            source: key.clone(),
            owner: scope.owner.clone(),
            expected_revision: source.revision,
            selection: selection.clone(),
        },
        nodes,
        limits,
    )?;
    Ok(Saved {
        namespace: scope.namespace.clone(),
        source: key,
        version: Version {
            revision: receipt.source_revision,
            selection,
        },
        record,
    })
}
