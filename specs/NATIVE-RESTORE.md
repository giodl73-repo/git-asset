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
Replacing a worker's stale recipe requires separate admitted orchestration;
ordinary write-once producer records must not be overwritten to hide that change.

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
