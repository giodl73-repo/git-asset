# Native pending actions producer review

Input 1c0f870f8efa8a538fc0948139f62f31ea3e625b; controls and allowed scope in
[contract](../../specs/NATIVE-PENDING-ACTIONS.md). Six producer .roles checklists;
no independent review, human acceptance or production permission. Producer owns
dispositions and remaining proof.

| Role | Finding; severity | Disposition / remaining proof |
| --- | --- | --- |
| Identity | H: save full originals, not only their IDs. | Separate canonical body/producer blobs, strict envelope and exact retained Version; hash/length/key matching. |
| Placement | H: plan persistence must not become media demand or a global lock. | Separate metadata-only namespace, source gate per action/producer; metadata quotas and source-count refusal remain. |
| Custody | H: a saved plan must not imply custody/removal authority. | Existing worker/controller revalidates; full readback and retaining source precede effects; no physical execution in registry. |
| Recovery | H: a later preview cannot overwrite uncertain work. | Existing slot wins; durable worker returns original handle on post-save failure, original backend APIs remain authoritative. |
| Provider/privacy | M: expanding plans belong asset-side with finite reads. | Existing NodeStore/MetadataStore, four referenced blobs, cumulative budgets, no request paths or new journal. |
| Git/adoption | M: compatibility and acceptance must remain explicit. | Optional scope omitted when absent; old action/policy wire preserved; no source cleanup or production inference. |

Verification/validation and exact-source terminal outcomes are recorded in external
native-pending-* evidence. Registry-wide scheduling, stale-plan replacement,
larger-catalogue continuation and owner adoption remain OPEN.

Verification:the registry fixture passes a separate child-process reopen,
competing preview, invalid-admission/no-write, corrupt/foreign/budget handles and
per-source blocking with unrelated metadata progress.28 focused GitHub/native
worker cases pass. Two explicit native cases pass: controlled backup/handoff
with saved-plan replay, and distinct E:/F: eviction/restore with complete original
plan readback after removal. Strict workspace all-feature/all-target Clippy and
standalone annex-publish build pass. Exact-source L0 and producer evidence remain
separate from independent review or production acceptance.
