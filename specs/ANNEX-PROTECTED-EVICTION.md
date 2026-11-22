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

Incarnation factor entry e493bac: WP-003/004/007/008/009, REQ-001/003/004/005/
007/009, SPEC-003/004/005/008/009, IF-003/004/005/006/008/012, PKG-002/003/004.
RUST/PROVIDER. Extract existing Birth codec/transitions for reuse by local and
native adapters, preserving all old schemas/hash domains and MAX_BIRTHS16.
No new journal, IO, authority or deletion entry. Entry: existing local lifecycle
and retirement recovery fixtures. Exit L0 pins/fmt/links/diff; L1 independent
literal wire vectors, cancellation/reacquisition/bounds and existing guarded
retirement/final-state regressions; L2 existing temporary-store process exits.
Native durable gate integration remains required before P2 execution closes.

Writer-gate API: expose LocalStore::read_published over its existing guarded
completed_reader. Exact original publication, completion and current incarnation
are checked under a shared object guard; pending retirement holds even when bytes
remain readable. No repair, fallback, initialization or new persisted state.
This can protect native operation metadata; metadata bytes are NOT native custody.
Fixture: live guard blocks removal; reopened pending state refuses; cancellation
preserves identity; retired/old publication refuses after same-hash restoration.

Incarnation checkpoint: shared lifecycle::Incarnation now constructs the same
initial/cancel/reacquire/final-reacquire records used by local replay and writers.
Two independent-vector/boundary fixtures pass; ordinary retirement24 active and
final provider28 regressions pass, including actual child exits. read_published's
new reopen/cancel/restore fixture passes and checks conflict before a different
object's missing fence; unknown operation is needs_recovery. Exact guarded reads
reuse completed_reader, with no second hash pass. Both identifier audits pass;
strict all-feature/all-target Clippy and standalone pass. Full P2 broad/native
execution/authority/independent-custody/handoff acceptance remains OPEN.

Native writer connection input528fd27: same P2 controls, RUST/PROVIDER/GIT.
Use one small genuine content-addressed control document per raw native object,
managed by the existing LocalStore/accounted/retirement engine; no native payload
enters that store, no per-asset namespace and no second journal. Its place binds
one native UUID. Private control profile pins root/store, ledger identity/minimum
and owner. Actual config gitasset.lifecyclecontrol binds that profile and native
UUID; every Publisher entry refuses omitted/mismatched control on enrolled pools.
Absent optional fields preserve old plan encodings; old config pins stay frozen.
Read-only accounted reopen checks original plan/job/settlement and publication
incarnation under its guard. Only the physical placement's original control
publication is admitted; aliases cannot invent another producer context. A missing
native body is writable only by the bound original producer; other logical
registrations require existing native bytes. Returned guards span native IO.
Initial/control publication remains explicit accounted SDK work, not resume/init.
Metadata retirement may expose a validated pre-unlink effect callback after durable
intent/current keeper revalidation. Failure leaves intent pending; historical replay
never repeats the effect. Callback is not native custody, authority or quiescence.
L0 fmt/pins/identifier/diff; L1 exact binding/origin/context/unknown/pending/stale
refusals and existing engine regressions; L2 owned fake/actual ordinary/bare native
publication and an actual child reopening pending metadata. Native deletion,
independence, cancellation effects, actor handoff and command quiescence stay OPEN.

### Native control SDK checkpoint

Implementation: `annex::control::{Profile,Plan,document,place,initial_request,
writer_context}` maps genuine small control documents to the existing managed
LocalStore. `accounted::reopen` requires the exact settled plan/job and returns
an incarnation-checked guard without admission or repair. Native
`publication::{Plan,Publisher}::controlled` and `replication::Copier::controlled`
retain that guard through IO/readback. Existing public profile structs and
uncontrolled plan encodings remain unchanged. Enrolled native configuration
refuses omitted/mismatched control. Only the original bound producer may supply
absent bytes; unfinished aliases need recovery, acknowledged absence stays an
error. `retirement::Execution::apply_with_effect` calls an adapter after durable
intent under live metadata fences, then revalidates before metadata unlink.
Unsettled retries may repeat the callback; completed historical replay skips it.
This hook grants no external custody, command-quiescence or deletion authority.

Verification/validation: owned synthetic publication/copy fixtures exercise
guard lifetime, pending restart in an actual child, cancellation, unrelated-key
progress and omitted-control/alias refusals. A separately selected real-native
fixture exercises ordinary and bare pools. Metadata-only retirement deliberately
leaves native bytes untouched. Engine/accounting regressions and exact reopen
negative bindings qualify this seam only; full P2 remains OPEN.

Producer `.roles` dispositions: identity checks bind original placement/context;
placement keeps configured admission explicit and limits unchanged; custody
rejects counting control documents as native replicas; recovery requires pending
fences and still holds command quiescence; provider preserves existing legacy
config spelling without adding dependencies/unsafe; Git/adoption preserves old
public profile literals and requires separate production handoff. Owners are the
corresponding six producer checklists, not independent people or approval.
High-severity native custody/quiescence/handoff findings remain P2 blockers.
Trace: same controls/input528fd27 above; full exact-source logs live outside Git.
Local metadata bounds remain16 births,16384 commands and16MiB journal; this is
not a continuation/scale qualification. No native removal API is exposed.

Native command fence entry input9e86635: same WP/REQ/SPEC/IF/PKG controls above,
RUST/PROVIDER, CR-005 exception isolated to crates/windows-process. Existing core,
store and Git adapter retain forbid(unsafe_code). The platform crate owns only
Windows named-job handles; no asset state, cache, custody or removal policy.
Fixed Global namespace plus a64-hex scope binds native store/object, not a PID.
Default owner security and no inherited handles require the admitted execution
identity; access denial has no Local/anonymous fallback. New jobs enable only
kill-on-last-handle-close, never breakaway; existing jobs must match those limits.
OwnedHandle closes each successful native handle exactly once. Every FFI call
has a fixed initialized repr(C) buffer/size, valid nul-terminated UTF16 name or
borrowed live process handle. No raw handles escape; no unsafe Send/Sync impl.
Assignment occurs before process-wrap's pinned JobObject resumes the suspended
native child. Scoped admission/control guards span assignment and native IO.
Recovery must hold existing pending metadata and native protection/authority
fences. Neither a missing name nor a reopened empty job proves the original
command stopped: after last-handle loss the name can disappear or be reused
while old process references remain. Only continuously retained original handles
or an admitted durable completion proof can establish that command's quiescence.
Query failure/active descendants/unknown continuity hold recovery. Timeout or
parent death alone is never empty evidence. Durable cross-restart continuity and
old/legacy-epoch handoff remain separate blockers; no public quiescence grant is
exposed by this adapter. Native ingress uses the scoped job while its original
handle is live. It explicitly polls active descendants within the existing
command timeout, independent of the dependency's child-wait/notification result.
L0 pins/fmt/unsafe inventory; L1 name/limits/handle/error/refusal fixtures; L2
actual child/grandchild attachment, parent exit, lost-name/recreated-job
counterexample and bounded live-handle descendant completion checks.
No native deletion is authorized by this platform primitive.
References: [Windows jobs](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects),
[job query](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-queryinformationjobobject).

One-shot effect witness entry inputacd7338: same P2 controls, RUST/PROVIDER.
Reuse upload_witness::Packet's two existing accounted documents. Add guarded
exact reopen and a one-shot callback path: reserve both records, persist intent,
then invoke a fresh effect at most once. A prior intent without completion is
uncertain and never invokes another callback. Completed replay reopens exact
settled bytes; completion settlement may resume without repeating the effect.
Callback failure, timeout or process exit retains intent and all external
pending protection; absence/name reuse cannot clear it. Completion publication
is the admitted caller's assertion, not native custody or quiescence by itself.
The native adapter must hold its actual original empty command group through
completion publication and bind source/config/object, metadata retirement plan
(including incarnation), owner/authority and command into these documents.
No unrecorded-effect repair or native deletion entry is added by this seam.
Allowed scope: existing witness API/error mapping and synthetic callback/crash
fixtures. L0 pins/fmt/links/branding/diff; L1 original context/job binding,
guarded corrupt/missing refusal, pending no-repeat and completed read-only
reopen; L2 actual exit from inside a callback, then original-operation reopen
and refusal without a second effect. Full P2 custody/handoff/CLI stays OPEN.

Native device-view entry input9427fdf: same P2 controls, RUST/PROVIDER, CR-005
exception scoped to windows-process::storage. Generic core/store/Git crates keep
unsafe forbidden. Compare only the already pinned source/keeper directory
handles: normalized volume GUID, at most16 physical extents per volume and
bounded4096-byte direct-device descriptors. No disk/volume/cache enumeration,
transfer or deletion. Same volume, overlapping disk numbers or equal normalized
serial identities refuse separation. Unknown/virtual/RAID/network/opaque buses,
missing/ambiguous serials, truncated/oversized responses and denied access hold.
Admit only ATA/SATA/NVMe/USB device views; this does not prove what hardware or
fault domains lie behind firmware/bridges. Private owner admission of unshared
direct backing and distinct failure domains remains mandatory before eviction.
Kernel observations never substitute for that declaration or native byte custody.
Keep volume/device handles live and recheck the exact observation before effect;
hash identifiers in reports, never export raw serials or discovered paths.
FFI contract: live borrowed handles, fixed initialized buffers and exact bounded
byte counts; parse returned bytes with checked offsets, not typed pointer casts.
Safe std File owns every device handle; no raw handles or unsafe Send/Sync escape.
L0 fmt/pins/unsafe/dependency/identifier checks; L1 byte-layout/truncation/count/
bus/serial/alias negatives; L2 owned temporary directory same-volume refusal and
native keeper comparison under actual namespace/config/byte guards. Distinct-
device positive qualification and owner authority/handoff remain required before
any native removal capability. No admission or observation runs on construction.
Device calls are synchronous/cooperatively budgeted, not preemptible kernel IO
deadlines. Unknown descriptor versions and duplicate device serials also hold.
The explicit F:/M: device-view fixture and native keeper fixture use only newly
owned temporary directories. No existing cache or asset is opened by those tests.
References: [selected volume extents](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-ioctl_volume_get_volume_disk_extents),
[device descriptors](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-storage_device_descriptor).

Connected-fixture entry inputa588ddd: same P2 WP/REQ/SPEC/IF/PKG controls and
RUST/PROVIDER/INTEGRATION profiles. Allowed scope: synthetic composition of the
existing metadata retirement callback, exact native keeper/device guards,
one-shot effect witness and native same-hash reacquisition; additive readonly
reacquisition payload-plan accessor. No public native removal entry or production
admission. Explicitly selected fresh F:/M: pools model namespace-owned actors;
this does not qualify private owner policy or mixed legacy handoff. L0 pins,
fmt/links/budget/branding; L1 existing witness/retirement regressions; L2 exact
native key removal, retained byte reopen, metadata pending writer refusal,
completed retry without repeated effect, same-hash restoration and old-plan
replay preserving restored bytes in ordinary/bare arrangements, including callback failure after actual removal
without a completion record: backup access survives and repeated effect is
Uncertain. Interruption here is injected error, not process exit. Process-exit
and historical-new-demand recovery qualification remain open for the controller.
The pinned native tool refuses dropkey without --force. This fixture alone uses
fixed dropkey --force --json --json-error-messages plus one exact SHA256 key,
under the retained native guards, and validates its bounded response and absence.
The native plumbing supplies no other-copy guarantee; no production command or
owner admission is inferred. See [upstream semantics](https://git-annex.branchable.com/git-annex-dropkey/).

Producer closure entry input36b7de1: same P2 controls, plus candidate publication's
WP/REQ/SPEC/IF/PKG controls; RUST/PROVIDER/INTEGRATION. Before native eviction,
source producer completion must prove the original command finished. Earlier
publication recovery inferred completion from visible bytes after incomplete
intent, although a lost job name cannot prove quiescence. Replace that path with
prepared one-shot publication under the existing packet operation fence: staging
precedes effect intent, actual original group spans durable completion, and an
old intent without completion is Uncertain without another import or implicit
acknowledgement. Preserve scoped staging retry, old-plan readability and defaults;
new plans explicitly identify guarded effect semantics. Old completion records
are historical evidence, not retroactive process-closure qualification. No new
cache/journal engine. L0 pins/fmt/links/budget/audits; L1 guarded preparation,
peer exclusion and unknown no-repeat; L2 real exits before intent/during staging/
after import/after completion, native ordinary/bare and connected eviction/restore.
Old acknowledgements stay readable; new eviction admission must refuse legacy
unqualified producer completion. Producer roles remain non-independent checklists.

Admitted planning entry inputd2ea6ff: same P2 controls/profiles above. Replace
fixture-built intent with a reusable readonly planner. Entry: actual scoped
source no-demand, exact retained native keeper/device guard, guarded original
producer completions and actual control enrollments. Explicit private policy
has eviction/direct-backing flags default false and actor boundary unqualified;
unqualified/mixed legacy authority refuses. Initial plan admits one independent
keeper/domain; broader quorum execution is unsupported, never silently reduced.
Bind owner policy, both exact producer plans/current physical control origins,
source checkpoint basis, keeper namespace/repository locations and exact retained
snapshot/device view before existing incarnation-bound metadata retirement plan.
Metadata grants remain separate from native media custody; reverify actual grant.
Allowed: readonly planning/completion proof API, existing store opener factoring,
bounded canonical plan and synthetic fixtures. No native apply/initialization.
L0 pins/fmt/budget/links/audits; L1 denied/legacy/alias/unknown/wrong-profile proofs,
plan canonical/binding and released/missing grant refusal; L2 actual F:/M: plans
feed the connected eviction/restore fixture. Frozen plan is not removal authority:
apply must re-admit/reopen all current proofs. Full owner/actor handoff, historical
recovery/cancellation and CLI remain P2 gates, not waived by planning admission.

SDK planning: obtain actual source Assessment, then read_for_plan before keeper
borrows; its SourceRead binds verified bytes to that assessment basis. Open the
exact keeper and StorageSeparation. Plan::producer_completion reopens both
original guarded producers; it keeps physical control origins and exact settled
witnesses live, without treating them as media custody. A completed alias cannot
supply this proof. StorageSeparation::plan consumes these live references plus
private planning::Profile and retained metadata grant. It opens the configured
control stores internally and freezes one canonical planning::Plan (64KiB max).
No deserialized plan/token or owner declaration substitutes for fresh apply
proofs. NamespaceOwned policy input is not mixed/old-actor handoff qualification.

Execution entry input9c1ce35: same P2 WP/REQ/SPEC/IF/PKG controls and profiles.
Allowed: internal native controller over the frozen planner/core retirement/
one-shot witnesses; fresh synthetic tests only. Runtime admission binds effect
store/ledger/job, source metadata job and full metadata namespace locations to
owner policy. Source native config gitasset.evictionpolicy seals that policy;
actual config guards span execution, so stale policy cannot survive committed
revocation. Configure before initial native publication; epoch changes require
handoff. Existing runtime-absent preview encodings retain their original policy
identity and remain non-executable. Public controller entry follows below.
Fresh apply reopens/rebuilds exact plan under live source/keeper/producer/device
proofs, releases source control read only after birth pinning, persists metadata
and effect intents before releasing source bytes, then fixed exact-key native
removal/absence check. Original command guard spans witness completion. Historical
completion uses full native scope fences without evaluating today's demand; it
never repeats removal or deletes reappeared bytes. Unknown intent remains held.
L0 pins/fmt/links/budget/audits; L1 runtime/authority/profile mismatch and replay
holds; L2 actual F:/M: fresh application, completed/unknown interruption, new demand
before historical completion and same-hash restore/stale replay. Public CLI,
cancellation and authority/legacy handoff qualification remain P2 gates.

Cancellation entry inputc3a3ab7: same P2 controls and RUST/PROVIDER/INTEGRATION
profiles. Add existing retirement cancellation guard under the exact operation
attempt fence after original intent/plan/job validation and before abort/birth
continuation. Existing unguarded core API behavior/wire stays unchanged. Native
cancel pins current full source scope/config/authority and verified surviving
bytes, then inspects the frozen effect packet inside that guard. Any intent or
completion refuses cancellation; absence/name reuse/timeouts cannot clear it.
Only a pending metadata operation with no native effect may continue cancelled.
L0 pins/fmt/budget/links/audits; L1 guard failure preserves pending/body and the
same operation excludes peers; L2 F:/M: pre-effect cancellation preserves bytes/
control origin, unknown/completed effects refuse and new demand remains intact.
Source/root/owner handoff, native executor actual exits and public CLI remain
separate P2 gates. No production operation is enabled by these synthetic tests.

Public controller entry input24c9ed4: same P2 WP/REQ/SPEC/IF/PKG controls;
RUST/PROVIDER/INTEGRATION. Allowed: an explicit experimental SDK coordinator
assembling existing source/keeper/producer/device proofs and delegating to the
qualified internal plan/apply/status/resume/cancel engines. No new storage,
authority grant, cache initialization or implicit operation on construction.
Fresh plan/apply share proof acquisition; apply must rebuild the frozen plan.
Recovery uses original source context and no fresh absence-of-demand assertion.
L0 source pins/fmt/size/links/audits; L1 pure constructor/default-denied owner and
mismatched source/keeper; L2 ordinary/bare native fixture through public fresh
plan/apply and historical status/resume/cancel. Actual executor process exits,
CLI, epoch/legacy handoff and production acceptance remain separate gates.

CLI entry input61816a0: same P2 WP/REQ/SPEC/IF/PKG controls; RUST/PROVIDER/GIT/
INTEGRATION. Allowed: facade storage native describe/plan/status/apply/resume/
cancel over the public Controller, preserving ordinary storage sibling dispatch.
Trusted private profile only, no event-supplied roots/code; explicit mutation
flags default false. Profile/request <=64KiB; saved preview/report <=128KiB;
canonical inner plan <=64KiB. Validate duplicate keys/schema/arity before native
operations. Saved previews carry exact plan ID; errors retain that handle and
explicit retry/reconciliation semantics. Status opens no metadata writer or
keeper payload; cancellation opens no keeper. L0 pins/fmt/links/size/audits;
L1 discovery/default-disabled/invalid/oversize/duplicate/mismatched preview and
ordinary dispatch; L2 new ordinary/bare F:/M: pools via real freshly built facade
for preview/apply/status/resume/cancel. Actual executor exit/race, epoch handoff
and P2 acceptance remain separate. No live cache operation is authorized.

Executor exit entry inputc05a69d: same P2 WP/REQ/SPEC/IF/PKG controls and
RUST/PROVIDER/INTEGRATION profiles. Test-only abrupt child exits at metadata
intent before native intent, native removal before completion, and durable effect
completion before metadata unlink. No production environment-controlled fault
hook. Parent owns fresh ordinary/bare F:/M: fixtures and exact saved profile/plan;
child reopens existing resources and executes the same Controller. L0 pins/fmt/
links/size/audits; L1 scoped platform compilation and unchanged defaults; L2 six
actual exits: no-effect cancellation preserves bytes, unknown effects stay held,
completed effects resume with new demand, retained bytes reopen and old history
spares restored same-hash content. This is process-exit evidence, not power-loss
or legacy/epoch handoff qualification. No production operation is enabled.

Native race entry input4c35c15: same P2 controls and RUST/PROVIDER/INTEGRATION.
Test-only bounded pause after metadata intent/before native effect while actual
source/keeper scopes are held. Parent attempts same-object job admission and
native config replacement, permits unrelated source enrollment, then releases
child to abrupt exit. Original cancellation with new demand preserves bytes.
L0 pins/fmt/size/links/audits; L1 platform guards/no production hooks; L2 actual
ordinary/bare F:/M: races. This checks participating actor fencing and metadata
progress, not arbitrary legacy exclusion or completed epoch handoff. Owner/
roster/epoch qualification and full P2 closeout remain required.

Closure audit/pipeline entry input9042dff: same P2 controls and RUST/PROVIDER/
INTEGRATION profiles. The existing connected fixture published its keeper from
independent fixture input; copy and eviction were qualified separately. Add a
joined controlled Copier -> native retained snapshot -> source claim settlement
-> Controller/CLI eviction -> managed restoration case in ordinary/bare pools.
Remove keeper input before copying, verify destination/original bytes and keep
creative selection unmodified. No production code/protocol change. L0 current
source pins/docs/size/links; L1 strict compilation/annex regression; L2 two real
F:/M: joined cases. Closure audit keeps actual owner/legacy/epoch migration and
remaining whole-PR profiles open; separate component proofs do not close them.
