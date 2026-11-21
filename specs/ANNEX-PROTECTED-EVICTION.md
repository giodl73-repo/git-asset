# P2: participating native ordinary eviction

Inputa61cb94c9e9eb351025f059ca5c5a645035934a0 (PR71 dependency).
WP-003/004/006/007/008/009/015/016; REQ-001..010;
SPEC-002/003/004/005/006/007/008/009/011/013/014;
IF-002/003/004/005/006/008/009/010/012/013; PKG-002/003/004/008/009.
RUST/PROVIDER/GIT/INTEGRATION. This freezes entry/scope and proof obligations;
provider wire/authority seams below must resolve before destructive code.

Entry: qualified native copy/retained readback, exact immutable provider and
full-roster enrollment, source claims and explicit independently retained custody.
Allowed: native adapter over existing store protection, retirement/incarnation,
capacity and intent/completion machinery; bounded existing storage CLI extension;
synthetic fake/native/crash fixtures. No new cache engine/journal, final-copy
loss, remote provider, renderer, automatic activation or production migration.

## Required implementation seams

- Factor the existing per-object LocalStore lifecycle/retirement mechanism for
  a provider-owned native representation. Preserve old formats and defaults.
  A source hash alone is not an incarnation: old removal must reject later
  same-hash reingest. Do not invent a second lifecycle controller.
- Support exact objects in existing flat retained snapshots. P1's retained-
  candidate convenience API must not force one mutable source per asset.
  MAX_SOURCES1024, MAX_COMMANDS16384 and MAX_PREDECESSORS64 remain admission
  limits, not permission for hidden whole-manifest rewrites. Reuse exact tree
  membership, immutable provenance, generic root claims and acknowledged subset
  handoff. Working placement and independent retained backup are distinct proofs.
- Reuse one aggregate protection budget and canonical all-namespace fences;
  unknown/foreign/opaque scope holds removal. Fresh independent retained byte
  guards span native removal, alongside pinned source config/authority/birth.
  Drop the source verification reader only while those fences remain held.
- Authenticate private policy/owner authority; copy/evict/reacquire flags are
  separate and default false. Native command is fixed and exact-key scoped;
  no event/path-supplied execution, wildcard deletion or filesystem fallback.
- Persist admitted original intent BEFORE removal and reserve completion/recovery
  metadata first. Native uncertainty inspects the same key/incarnation, never
  repeats a hash-wide deletion against a later placement. Cancellation preserves
  surviving bytes and reports historical effects honestly. Missing acknowledged
  bytes without matching authorized absence are incidents, not orphan evidence.
- Old/new roster and epoch handoff must actually fence old actors, including a
  paused legacy writer/pruner. Config edits or a receipt alone are insufficient.
  Private Index/WorkRef authority is not Namespace authority; mixed pruning stays
  denied without qualified owner-side handoff. Preserve originals, active jobs,
  minima and pending outcomes. No private implementation enters public ancestry.

## Exit and named proof families

L0: controls/roles/format/source hash/link/budget checks; old identifiers frozen.
L1: exact-basis/policy/protection/alias/unsupported/metadata-budget refusal;
copy and snapshot membership preserve logical identity/provenance/selection;
locked default/all-feature workspace, strict all-target Clippy and standalone.
L2: new temporary ordinary/bare native pools: copy -> retained backup -> owner
handoff -> eligible working eviction -> exact restoration. Active jobs, shared
consumers/readers and restart protection block removal; offline/corrupt/same-domain
survivors refuse. Actual exits at intent/native removal/completion reconcile the
original operation. Same-hash restore survives stale retry. Unrelated publishers
progress. Paused old-epoch actors cannot cross handoff. CLI plan/status/apply/
resume reports bounded typed holds/errors and logical versus allocated reclaim;
no net free-space or secure-erasure claim follows from native unlink.

Implementation, verification, validation, trace and six producer lenses close
separately. P2 merges only after P1 and all gates. Private/human production
acceptance remains P6; this contract authorizes only owned synthetic fixtures.

First implementation seam: annex::snapshot verifies one exact path/object in a
current or predecessor Retention root, hard place floor and owner/namespace;
shared subject use spans the bounded tree point read and source recheck. It
opens no native bytes, creates no per-asset head and grants no deletion/custody.
Fixtures: shared_root_point_proof_preserves_history_and_never_reads_payloads;
wrong_namespace_owner_kind_place_path_and_object_refuse;
retirement_during_metadata_read_cannot_return_a_retained_proof. The coordinator
must enforce one aggregate operation budget; returned point metrics are not that
whole-operation meter.

A keeper can share a namespace with its source. Snapshot verification must accept
that exact already-held exclusive subject fence; reacquiring a shared namespace
use would self-deadlock. The fenced proof borrows the real guard and verifies its
namespace/object before point IO. Test:
shared_namespace_keeper_verifies_under_existing_exclusive_fence_without_self_deadlock.

## EVID-146 metadata seam checkpoint

Four snapshot fixtures pass: one5001-member retained root uses a bounded point
lookup, preserves predecessor access and opens no payload; namespace/owner/kind/
place/path/object/version/budget failures refuse; retirement during node IO
invalidates the proof; a shared source/keeper namespace verifies under an existing
exclusive fence without self-deadlock. Fixture transition admission uses20000
bindings for before+after5001-member roots, within unchanged production limits.
Producer identity/placement/custody/concurrency/privacy/adoption lenses remain
OPEN for native execution; this metadata guard is not byte custody or removal.
This seam passes focused L1 and strict all-feature/all-target Clippy; P2/L2/hosted
qualification remains outstanding. Historical summaries pin original full records
and hashes to keep the unchanged8 MiB source ceiling.

Namespace seam input715b98c: factor canonical roster fencing and shared-meter
absence evaluation from store_protection::assess without changing local-provider
ordering or frames. The provider-neutral result owns subject fences/checkpoints;
it grants no provider/custody/deletion authority. Native code must separately hold
and revalidate actual pool configuration, bytes and authorization. Fixtures:
namespace_only_assessment_reuses_guards_and_allows_unrelated_progress;
namespace_only_assessment_refuses_incomplete_required_and_exhausted_scope.

Native inspection seam: annex::retirement::Assessor is pure at construction.
Explicit assessment pins initialized pool config/directory/UUID/current binding,
opens exactly enrolled namespace minima and holds the provider-neutral subject
fences. Native source readback occurs under those fences, never via a managed
reader that reacquires shared locks. No mutation/removal method is exposed yet.
Fixture native_retirement_assessment_holds_configuration_and_all_subjects tests
required-claim refusal, explicit claim settlement, readonly source bytes and
configuration/fence lifetime. Invalid/disabled runtime remains no-IO.

Assessment: protection18/snapshot4/native-readonly1 pass; strict all-feature/
all-target Clippy passes. Tests cover refusal/unrelated progress, root settlement,
pinned config, two exclusive fences and exact bytes; protected has its own code. This adds no eviction
method; keeper custody, incarnation model, handoff and operator execution remain
OPEN. Local source-provider exclusion ordering and old record formats are intact.

Keeper seam inputd3f3c3b: Assessment::open_keeper pins the exact native pool,
complete roster, snapshot namespace/place and verified bytes. All keeper subjects
are fenced; overlaps borrow source fences only at the same admitted location.
The returned read borrows the source assessment and owns keeper fences/config/
namespace/byte guards. A different UUID alone is NOT independent custody; this
adds no eviction grant. Tree metrics remain caller-metered across the operation.
Keeper2/assessment1 and strict all-feature/all-target Clippy pass (fake native
protocol). Actual native keeper/incarnation/handoff/CLI and full P2 remain OPEN.

Replay seam input0e30be2: ReplayBasis binds source pool/config/locations/binding,
exact object and every Checked checkpoint to an immutable intent. A fresh explicit
assessment rejects changed bases. Live scoped fences allow unrelated progress;
after restart unrelated progress may conservatively stale the old operation.
This is NOT physical birth or removal authority. A candidate namespace-based
incarnation proof requires EVERY admitted writer to hold the same subject uses
and establish effective source-place demand before IO. Current Publisher uses
hard Retention; generic future writers must preserve that rule. Unmanaged or
legacy writers, metadata rollback/current minima and old-actor handoff remain
separate unresolved gates. Local lifecycle/retirement reuse remains required
unless this alternative is fully qualified; no destructive execution is added.

Replay fixture passes: same-hash reingest remains after stale check; old publisher
refuses missing acknowledged bytes; live fences block writes but allow unrelated
admission. Initial log equality wrongly counted readonly commands; only imports
and capacity are effect measures. Physical birth/old-actor/native-crash proof OPEN.

Native keeper qualification: one explicit ignored entry runs ordinary/bare keeper
pools (256KiB each), actual publication/readback, protected predecessor-retirement
refusal, config/byte guard lifetimes and corrupt/missing-byte holds. Both shapes
pass. Fixture paths come from the verified provider, never assumed annex layout.
A Windows-only scoped lint exception clears the owned fixture readonly attribute
so write refusal proves handle exclusion, with a successful write-open beforehand.
Fake keeper/replay3 pass; full P2 execution/independence/handoff remains OPEN.

Pending-command gate: ReplayBasis is supplemental stale-plan evidence, not the
retirement/incarnation protocol. Every native writer must honor durable per-key
pending retirement before any effect. Parent exit/timeout or a namespace vector
alone cannot prove an earlier native child has stopped. Recovery must fence or
prove completion of that command before admitting the next incarnation. Factor
existing pending/intent/completion/birth machinery; do not promote this replay
fixture into destructive permission. This proof remains OPEN.

Dependency closed: PR71 merged f12a1634a8833f822edf2a78b4df75a2b3851538;
its tree equals qualifieda61cb94. Both full workspace profiles, standalone and18
hosted checks pass. P1's EVID-145 pending paragraph is a historical checkpoint;
final source qualification is in PR71 and its external exact-source packet.
