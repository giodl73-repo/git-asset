# git-asset resume

Issue75 review lane: docs/issue75-tool-preview, input main
f12a1634a8833f822edf2a78b4df75a2b3851538. TOOL-PREVIEW-HANDOFF-PLAN maps the
six proposed requirements to current APIs/gaps and four bounded U4 PRs. It is
documentation, not a tool/preview implementation or browser qualification.
REEL requirements pin fb8ac7b2f1b435f08d5e304fd0b82b3148ec01b6. U4 remains
independent of P1-P6 storage/production adoption. Do not post issue comments or
launch tools/servers as a side effect. Existing backend work remains in the
owned feat/trusted-native-placement lane; derive its head/PR74 CI from GitHub.
Keep both lanes separate. Resume their actual next steps after this review.

Owned feat/annex-retained-copy; input8ded690793b37b7132eec1c9e912e944f3b2e286.
Active goal: implement the reviewed PR70/P1-P6 series in PRODUCT_PLAN.md.
PR70 merged a95c9de. P1 draft/full checks remain in progress; inspect live state.
P1 controls: specs/ANNEX-RETAINED-COPY.md. Compose managed source/retained
publisher with exact copy-context pin; source and destination guards bridge
copy/readback. No removal grant, new engine or production enrollment.
Finish implementation, exact synthetic/native/recovery tests, producer review,
source-budget/target pins and hosted checks before P1 integration.
P2/P3 follow storage; P4 REEL is consumer-owned; P5 freezes RC before P6 cutover.

Keep builds/temp/evidence outside source, existing dirty/private/other-lane
worktrees and runtimes untouched. Resume starts no cache, scan, hydration,
transfer, render or deletion. Owner acceptance remains distinct from checks.
