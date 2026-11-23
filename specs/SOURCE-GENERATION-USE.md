# Scoped source-generation use

Input94865c07928275bb130cf3e002dc48ac82d9cfce; P4 prerequisite.
WP-004/008/009/011/016; REQ-001/002/003/004/006/007/008/010;
SPEC-002/003/004/005/007/008/010/011; IF-002/004/005/008/010/012;
PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION profiles.
Allowed: existing source journal/Namespace, scoped revision-use guard, explicit
new-epoch capability, bounded synthetic locks/restart/failure fixtures. No new
cache/journal engine, clock lease, consumer activation, native command/deletion,
legacy migration or weakening of retained custody. Entry: exact source revisions
and compatible object-use admission. Exit: original current source remains stable
while guarded; unrelated sources continue and restart releases only live guards.
L0 roles/source/hash/format/budget/diff; L1 locked focused/workspace all-feature
checks and strict Clippy; L2 temporary namespaces, child exit and cross-handle
contention. Producer review is not independent or owner acceptance.

Object SubjectUse blocks removal fences but intentionally permits claims to
change. It is not a source-generation lease. Add Namespace::source_use for an
exact owner, source revision and current Version, returning a noncloneable live Use.
Verify no pending transition, exact source revision and full selected Version; never
substitute a historical receipt for current state. This proves source stability,
not byte custody, complete roster coverage or authorization by an owner string.

Fresh Namespace::create_fenced explicitly creates source-claims design-3 using
the same bounded journal commands/receipts and fixed policy. Legacy create and
all design-1/2 tags/hash domains remain unchanged and readable. Older tools do
not recognize design-3; owner enrollment must admit the participating writer
boundary. No marker edit/automatic upgrade or migration on open/resume. Existing
namespace source_use returns Unsupported rather than a weak status substitute.

Design-3 precreates 1,024 source-gate anchors, one for each maximum source slot,
in addition to existing 256 object partitions. Slot assignment is derived from
first enrollment order during journal replay, never names/lexical order or reuse.
No expanding per-operation side log. Source Use holds a shared slot lock; every
new effecting transition acquires that source's exclusive slot lock through the
journal commit. Multiple uses coexist, same-source mutation returns Busy, and
distinct source slots do not contend. No SQLite transaction/cache-global lock
spans the use or payload IO. Exact already-recorded retries are read-only history
and may return their receipt; this never advances/revives the source.

Control roots/anchors are owned trusted metadata: participating actors never
replace/delete them during an operation. This is not an ACL or defense against
arbitrary filesystem writers/rollback. Windows holds non-delete-sharing directory
handles through the actual control path, plus the slot file. Other platforms use
cooperative advisory locks and require stable roots; network lock semantics are
not qualified by local filesystem tests. Anchor bytes are not durable identity.

Acquire the slot then recheck current owner/source revision/full Version under it.
Expected source revision is separate from selected Version.revision: retiring
history can advance the source without changing its selected version. Missing,
wrong-kind or nonempty gate files refuse without recreation. New source
enrollment is atomic in the existing journal; a previously missing non-enrollment
source cannot bypass a gate by racing enrollment. Sources/slots never disappear
or get reused. Existing 1,024-source, 64-predecessor, 16,384-command and 16 MiB
bounds remain. Initial anchor allocation needs separately admitted filesystem
headroom, not inferred quota or future reclaim.

Drop/process exit releases only the live slot lock; durable current/pending/
predecessor claims stay. There is no TTL, renewal, expiry deletion, cancellation
hook or task decision. Callers reopen original minimum/owner/version and actual
bytes independently. SourceUse error taxonomy preserves Unsupported, Busy,
owner/stale/inactive and underlying namespace/recovery failures. Original operation
IDs and admission remain mandatory after uncertainty. Roster/epoch/legacy handoff,
complete REEL Lifecycle and production acceptance remain separate gates.

Retry: Busy has made no new journal transition; retry the original operation and
baseline after contention ends. Unsupported requires explicit capability/epoch
admission, never an implicit upgrade. Owner/Stale/Inactive require caller
reconciliation, not latest substitution. IO/NeedsRecovery preserve original
operation IDs and recovery evidence; never create another operation to guess an
unknown outcome. Durable claims are not renewed by status/use and are not lost
when a lock disappears. No byte materialization occurs in this API.
