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

Current refinement from2f4f5fce48f34e4e96d21bae6352cb38447248c7 adds the
metadata-only authenticated history receiver. Optional signed Actions.history and
private history_records dispatch history_preview/history_apply without payload
resource opens. Exact predecessor pages compose archive/refresh/release; cursors
bind current Version, and removed after-keys need saved original release proof.
Post-release expiry preserves receipt/handle and a usable continuation. Preview
may reconcile authenticated Git selection, but never archives/releases history.
No media custody/convergence is inferred. See HISTORICAL-ROOT-ARCHIVE and
HISTORICAL-ROOTS. External history-receiver-* logs pin terminal qualification;
check their outcomes and exact committed source before inheriting evidence.

Next obligations: safe stale-plan replacement after original status/cancel;
producer-root publication/retention and incremental advancement, including
retained producer member references for archived Main roots; missing deprecated
backup repair and complete controlled reacquisition; full native HTTP/account/TLS
qualification; larger-catalogue lifetime and measured bounds. Then close P3/P4
acceptance, qualify P5 and seek concrete P6 owner acceptance. No timeout forgetting,
final archival deletion or automatic production enrollment is introduced.

Run L0/pins/branding/diff, fmt, relevant locked tests, strict workspace/all-feature/
all-target Clippy and standalone feature checks before commits. Build/temp roots
must be explicit and outside source. Source-only ceiling is8.5MiB; runtime limits
are unchanged. Current relevant files plus tests/contracts must stay reviewed.

Terminal local qualification:28 GitHub,2 index,51 retention and1 CLI cases pass;
2 registered native cases plus the SDK native eviction/restore case pass. Strict
workspace Clippy, standalone annex-publish, fmt and actual describe discovery pass.
Exact source/log hashes live in receiver-eviction-source.json externally. Hosted
qualification for the committed refinement must be checked separately.

Historical-root SDK refinement (input b02d5ad; hosted45036579080 passed):
annex::history captures exact predecessor/current generations, mirrors complete
metadata into an existing retained provider, and adopts retained media/metadata
record claims before explicit working-history release. media_custody_verified
stays null and authorizes_eviction false. Actual native eviction still needs
fresh keeper/protection proof. member is an exact point lookup; find/load retain
saved originals. Explicit refresh after monotonic Main advance keeps older saved
plans and the same archived root; it does not replace native eviction plans.
Portable history cases cover restart, partial adoption/enrollment, budgets,
Main-race refusal/refresh and unrelated progress. Native composition reads an
archived exact member after ingress release and an offline working repository;
unavailable siblings remain unavailable. Qualification uses history-* externally.
Next: connect receiver history phase and advance retained producer references;
qualify producer-root lifecycle, native stale-plan replacement, deprecated backup
repair and controlled reacquisition. Full HTTP/scale/project acceptance stay OPEN.
Issue75 review is separate docs/issue75-tool-preview/PR76 at9fcabb7; no U4 runtime
API was implemented there. Keep that main-based documentation lane separate.

Historical-root terminal qualification:8 metadata cases including separate-child
reopen,1 fake native adapter case and1 explicit real-annex case pass. Workspace
all-target/all-feature Clippy, standalone annex-publish, fmt, L0/pins/branding/diff
pass. These are SDK/component results, not full receiver or production acceptance.

History receiver local qualification:8 archive,10 default worker and8 GitHub/CLI
tests pass (6 explicit native worker cases skipped). The separate actual git-annex
historical-member reopen case passes. Strict workspace/all-feature/all-target
Clippy, standalone annex-publish and fmt pass. Final source/log hashes are external
history-receiver-source.json; derive current-head hosted status separately.
Next P3 slice: retain/publish producer roots and update archived member references,
then missing-backup/reacquisition and full transport/recovery qualification.
