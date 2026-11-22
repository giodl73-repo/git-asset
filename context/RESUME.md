# Resume the initial-offering PR series

Goal remains PRODUCT_PLAN.md P1-P6, not a single component checkpoint. Use actual
Git/GitHub state for branch, head, dirty files, PR and CI; do not treat this file
as a substitute. Preserve other sessions/worktrees and shared runtime bindings.
No production caches, scans, hydration, transfers, pruning, renders or deployment
on resume. Native work uses explicitly owned synthetic fixtures only.

Current lane: feat/trusted-native-placement, draft PR74, based on
feat/source-generation-use/PR73. Latest refinement input is
0bc08b666268df815dad1e04530d42e1ad45b232. That input's hosted45033636660 passed.
Derive the current head and its separate hosted run after any later commit.

P1/PR71 is merged. P2/PR72 has qualified native eviction SDK/CLI components;
mixed legacy actors, roster/epoch handoff and owner acceptance remain OPEN.
P4's main-based REEL PR401 has a component route; its source-use SDK pin remains
0ccb137cf5a78416e780f8a3372017f642a5cb71. Consumer/owner acceptance is separate.
P5 pinned Windows RC/bounds/setup/upgrade/rollback and P6 private project cutover
remain after P3/P4. Do not promote an unaccepted candidate to production.

P3 implemented: exact local/HTTP annex source resolution and current source-use
protection; authenticated fixed-profile copy, backup, handoff and eviction APIs;
exact retained snapshots; bounded immutable producer lookup; native receiver
copy/backup and durable handoff; full original-action persistence/restart recovery.
Existing publication, metadata, capacity, control and retirement engines remain
authoritative. Old local/LFS dispatch and absent optional wire fields remain.
SDK original reconcile is copy-only; reconcile_with_actions adds durable handoff.

Current registered eviction refinement: native::eviction::Pass and
reconcile_evictions page the signed working index. Exact current Main plus fresh
complete-roster controller checks protect required bytes. Producer records may
embed canonical control_retention grants for native control metadata; this never
substitutes for independent retained native bytes. Private receiver phase defaults
to copy, with eviction_preview/eviction_apply opt-ins. Preview never saves/releases/
removes; apply persists the full original first. Durable eviction inspects original
status: completed work resumes accounting; uncertain intent never repeats removal.
Saved direct-ingress actions can be found before keeper index/payload opens.

Contracts/runbooks: specs/{NATIVE-RECEIVER,NATIVE-PENDING-ACTIONS,
NATIVE-EVICTION-PASS,TRUSTED-NATIVE-BACKUP}.md and docs/{NATIVE-RECEIVER,
NATIVE-ACTION-RECOVERY}.md. Six .roles are producer checklists, not independent
reviewers or human approval. Implementation, verification, validation, trace and
review stay separate; no global convergence or production acceptance is claimed.

External evidence uses receiver-eviction-* under the owned development evidence
root. Earlier receiver-handoff-*, native-pending-*, native-backup-* and producer
index logs retain their exact source/component scope. New native cases exercise
preview, late job protection, admitted removal/post-expiry, completed saved replay
with keeper payload offline, and uncertain original intent with unchanged effect
ledger. Derive terminal outcomes from actual logs before claiming qualification.

Next obligations: safe stale-plan replacement after original status/cancel;
producer-root publication/retention and incremental advancement, including
explicit historical Main snapshot custody/handoff before releasing predecessors; missing deprecated
backup repair and complete controlled reacquisition; full native HTTP/account/TLS
qualification; larger-catalogue lifetime and measured bounds. Then close P3/P4
acceptance, qualify P5 and seek concrete P6 owner acceptance. No timeout forgetting,
final archival deletion or automatic production enrollment is introduced.

Run L0/pins/branding/diff, fmt, relevant locked tests, strict workspace/all-feature/
all-target Clippy and standalone feature checks before commits. Build/temp roots
must be explicit and outside source. Source-only ceiling is8.25MiB; runtime limits
are unchanged. Current relevant files plus tests/contracts must stay reviewed.

Terminal local qualification:28 GitHub,2 index,51 retention and1 CLI cases pass;
2 registered native cases plus the SDK native eviction/restore case pass. Strict
workspace Clippy, standalone annex-publish, fmt and actual describe discovery pass.
Exact source/log hashes live in receiver-eviction-source.json externally. Hosted
qualification for the committed refinement must be checked separately.
