# Controlled native restoration

Input2eae14c95886767b36a62cffd48bf49017021f01; P3 WP-004/005/006/007/009;
REQ-001/002/004/006/007/008/010; SPEC-002/003/004/005/007/008/010/011;
IF-002/004/005/008/010/012; PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION.
Entry: exact original controlled publication and completed native/metadata
eviction, existing enrolled stores/namespace/ledger/job and explicit permissions.
Allowed: additive Rust composition over existing reacquisition and Publisher,
canonical bounded saved plans and synthetic fixtures. No discovery, new journal,
enrollment, creative selection, raw filesystem repair or production activation.
Exit: exact retained bytes reopened under a new native/control incarnation,
idempotent original-plan recovery, or explicit refusal preserving the plan.
L0: pins/links/budget/fmt; L1: locked tests and strict workspace Clippy;
L2: ordinary/bare native copy/evict/restore, interruption and stale eviction replay.

## SDK contract

Use `annex::publication::restore::{prepare,apply,resume}` with Context, Original,
Operations and Plan. The Context supplies actual private native/control/retirement
profiles, the existing Namespace and MetadataStore, and protection limits.
`allow_publish` and `allow_reacquire` must both be explicit. The existing eviction
status contract also requires its admitted retirement profile; inspection here
does not perform a removal. No new CLI or receiver phase is exposed.

`prepare` verifies original plan/object/namespace/control bindings and completed
native removal plus metadata retirement. Uncertain removal returns Uncertain;
incomplete or unexplained loss returns NeedsRecovery. Verified-present bytes
return Conflict. The destination must be an existing owned, empty Retention
source with no pending selection. Fresh native/control operation IDs are required.
Original candidate identity, content hash/length, provenance and admission remain
unchanged. New registration uses the captured source revision and existing CAS;
Branch/Main creative selection is never advanced.

Preparation writes and finishes immutable metadata, reopens it, and constructs
the existing metadata ReacquisitionPlan. It does not import media or create a
new incarnation. The caller must durably save the exact canonical Plan before
apply. JSON is strict and capped at16KiB; original metadata/tree/native/capacity
limits remain in force, including the existing16-incarnation lifecycle bound.
The new read-only `ReacquisitionPlan::validate_retirement` binds a parsed plan to
the exact completed predecessor; it grants no external-byte authority.

`apply` revalidates bindings and permissions, runs the existing metadata
reacquisition, then calls the controlled Publisher with an explicitly supplied
ByteSource. A managed retained native reader can supply the old bytes without
the original input or a consumer checkout. Hash/length verification, capacity,
staging, protection, native intent/completion and retained readback remain owned
by that Publisher. Return Ready must be reopened/read before consumer completion.

The admission callback receives BeforeControl and ControlReady. These are retry
checkpoints, not receipts. Refusal at ControlReady can follow a durable new
metadata birth; preserve the Plan and retry it. `resume` reuses that same plan
and the native publisher's original outcome without a ByteSource. Missing original
native completion may require apply with the same plan/source; uncertainty remains
a hold. Never generate a replacement operation merely because an acknowledgement
was lost. Completed replay must not register another version or reimport bytes.
After restoration the old completed eviction is historical and cannot remove the
new same-hash incarnation. Restoring a missing backup without matching authorized
retirement is a different, still-open repair operation.

Keep `Plan::publication()` as the exact reopened producer handle. Its new
operation/incarnation is not a copy-context annotation of the old producer row.
Use the restore-specific index transaction below to register that successor;
ordinary write-once edits cannot hide the change. Receiver scheduling is separate.

## Producer review and qualification

Six .roles are producer checklists, not independent review or owner acceptance.
Identity: preserve the exact candidate and canonical original-plan chain.
Placement: retain existing bounded admission and explicit permissions.
Custody: consume retained bytes; never treat absent media as eviction authority.
Recovery: save before effect, retry after ControlReady, replay without reimport,
and fence old deletion against the new birth. Privacy: private profiles own paths;
events supply no execution or discovery. Adoption: SDK-only qualification leaves
receiver orchestration, unexplained-loss repair and project acceptance OPEN.

| Finding / owner | Severity / disposition | Evidence or remaining work |
| --- | --- | --- |
| Recovery: sequence alone does not bind a predecessor. | High; bind complete derived birth, terminal, exact retirement and ledger. | Wrong terminal/retirement/ledger tests; validate again after restore. |
| Placement: serialized wrapper inherits the publication encoder cap. | Medium; declare16KiB and reject oversized input. | Existing encoder and canonical saved-plan roundtrip; no limit increase. |
| Custody: missing bytes are not proof of authorized removal. | High; require completed original native and metadata retirement. | Existing original status; unexplained-loss repair remains separate. |
| Adoption: SDK proof does not qualify automated restore scheduling. | Medium; keep receiver/owner gates open. | No receiver restore phase or automatic activation. |

The explicit synthetic test
`actual_native_controlled_copy_retained_eviction_and_restore_pipeline` covers
ordinary/bare pools, disabled permission, canonical-plan reload, post-control
interruption/retry, retained-source hash/length readback and completed replay.
It also resumes the old eviction and checks that restored bytes survive.
External native-restore-* logs record terminal results and exact source; default
ignored native cases are not successes. Full P2/P3, P4/P5/P6 and human validation
remain separate from this component's implementation and verification.

## Restore-specific index transaction

Inputc10b4682ce0042494a8f490044be4eeada7d1a26; same controls/profiles above.
Allowed: extend the existing producer-root updater with one typed restore
transition. Entry: existing index Job source, exact committed Version/record,
saved restore Plan and original publication/eviction. No generic recipe editor,
new cache/journal, automatic enrollment, payload transfer or creative selection.
Exit: retained expected-baseline index receipt for the new publication, or hold.
L0: pins/links/fmt; L1: strict Clippy, immutable ordinary updates and parsed-plan
validation; L2: native restore/readback, saved index update, interruption/retry,
stale baseline and old/new root lookup. These are producer component checks.

`publication::restore::reopen` is read-only: verify the exact original completed
retirement and saved restore chain, then reopen the restored publication through
the controlled Publisher. It creates no metadata birth/import or selection.

`producer_index::update::restore::{prepare,apply,status}` composes this readback
with the existing metered root editor/atomic selection. Baseline names the exact
index Version and previous record Object. Only one row may change. The old row
must declare the exact retired publication, and the new row is exactly the restored
candidate's publication, without inherited old retained-reference/control grant
or a copy-context annotation. Candidate/provenance remain identical. Old roots
and their recipes remain in the existing source predecessor history; all namespace
closure/history/lifetime bounds remain. Ordinary prepare/apply still refuse swaps.

Persist the canonical wrapper Plan before apply. It embeds the restore Plan and
root-update Plan, is capped at64KiB, and authorizes no native transfer/removal.
The inner root Plan has a distinct restore-root schema/selection identity;
ordinary apply returns Unsupported for it. An ordinary insertion receipt cannot
be reused as evidence that a restore-specific transition completed.
New selection reopens and holds restored native bytes through root adoption and
revalidates the exact old/new records while rebuilding declared edits. Concurrent
index writers cause stale/CAS refusal. Post-selection refusal keeps the receipt.
Completed original status/replay returns that historical receipt before native
access, without reselecting an old root or requiring the media to remain online.
An incomplete index update preserves the original restore and root Plans; retry
the same operation after admission/recovery. Do not restore again merely because
the index reply is lost. Receiver scheduling and unexplained-loss repair stay open.

Producer role review: recovery (high) requires an existing old row even for parsed
plans, keeps ordinary apply strict and checks original receipt before native IO;
the separate restore-root identity also prevents cross-operation receipt reuse;
implemented in the private transition verifier and native fixture. Custody (high)
requires held fresh native readback through selection, not metadata alone. Identity
keeps candidate/provenance fixed and old root blobs unchanged. Placement retains
all existing budgets and separate permission. Privacy adds no event paths or new
execution. Adoption leaves automation/owner acceptance open. These dispositions
are producer checks, not independent review or authorization to enroll caches.

## Saved restore actions

Inputec8bf476a629f61015599615f74ddf5f03eee59d; same P3 controls above.
Entry: saved exact retirement/publication identities, explicit existing pending
Scope/namespace/metadata and bounded restore or restore-index Plan. Allowed:
additive phases in the existing four-blob pending-action mechanism, pure binding
accessors and synthetic fixtures. No new journal/cache, media mutation, enrollment
of cache locations, automatic expiry/forgetting or receiver activation.
Exit: original action and both producer recipes retained/read back before effect,
stable lookup/reopen across retries, or explicit refusal. L0 pins/links/budget;
L1 strict Clippy, existing pending/index regressions and feature checks; L2 native
restore pipeline with saved restore and index actions and wrong-chain refusal.

`pending::Phase::{Restore,RestoreIndex}` reuse existing Scope/Saved/find/save/reopen.
Restore bodies hold the original eviction and native restore Plan. Index bodies
add the restore-specific index Plan. Original working and retained publications
remain separate retained blobs, just as for eviction. Pure validation binds the
exact eviction producer IDs, original restore digests and unchanged candidate;
index actions must embed the same restore Plan. Canonical body/readback and all
existing per-blob64KiB, aggregate metadata and namespace lifetime limits apply.
These records assert no current media custody and grant no native authority.

Find the original phase/object/working-publication slot before preparing another
operation. Existing saved slots win over later previews; immutable old plans are
not overwritten by retry, timeout or changed index baseline. Native uncertainty
must use the same saved restore; index stale/conflict needs explicit original
status and replan through the index SDK. Completed index receipts remain readable
without native IO. Retaining these bodies does not itself execute, replace or
schedule an operation. Authenticated worker/receiver restore orchestration stays
open until it actually consumes these saved actions under current demand guards.

Producer review: recovery/identity (high) require exact original producer IDs and
the same embedded restore recipe, plus fresh-process reopen and immutable retry.
Custody (high) forbids treating saved metadata as completed media work. Placement
retains existing bounds/refusal. Privacy adds no execution or event paths. Git/
adoption keeps expanding evidence asset-side and receiver/owner gates open.

## Authenticated saved execution

Inputaf878de168aea7c980d7bde1cf6e59b657a4a3e3; same P3 controls/profiles above.
Allowed: trusted Execution methods consuming existing saved bodies, current-source
guards, original Publisher and qualified index SDK. No event-supplied plans/paths,
new queue/cache, automatic enrollment or receiver phase. Entry: exact saved handle,
fixed signed policy, existing namespaces/providers and explicit copy/reacquire
permissions. Exit: retained readback or index receipt with post-effect hold, or
explicit refusal preserving originals. L0 pins/source/fmt; L1 signed admission,
profile/demand/refusal regressions and strict Clippy; L2 ordinary/bare saved-action
restore after interruption, index adoption and historical receipt recovery.

`Execution::restore_saved` loads the retained original, verifies its publications
against the fixed pair and private Context against signed profiles, holds exact
current committed-source generation and requires working-place demand. Both
allow_copy and destination-control allow_reacquire are required. Admission/ref
checks run before metadata birth and native import. Input comes only from the
fixed retained managed pool. The result owns Ready bytes and the original receipt;
post-effect expiry/ref drift is reported without discarding that receipt.

`Execution::register_restore_saved` loads the saved index body, binds Target to the
explicit signed destination producer head and checks original status before fresh
native IO. Completed receipts recover with metadata writes disabled. A new index
selection requires admitted head write permission, current working-place demand,
fresh restore readback and current admission under the SDK's expected-baseline CAS.
No producer-head read guard is held across its own update. Stale/uncertain originals
remain held; these methods neither prepare replacement plans nor forget old ones.
They do not imply receiver scheduling or whole-project convergence.

Producer review: recovery/custody (high) require a fenced committed-source
namespace; unsupported legacy sources are not upgraded or bypassed. The native
fixture explicitly creates that fenced boundary and selects the restored object
in its Git commit. Expired requests require fresh token and public-key snapshot
verification at later time; a Verified clock cannot move backwards. Post-selection
expiry keeps the index receipt. Identity/privacy bind caller contexts and index
Target to the exact signed profiles; no supplied plan/path becomes authority.
Placement preserves existing admission budgets. Adoption keeps receiver phases,
explicit stale-index replan and all private owner gates open. These are producer
checks, not independent or human approval.
