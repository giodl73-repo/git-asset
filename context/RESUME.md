# git-asset resume

Issue #77. Existing P2 checkpoint and operating constraints are recorded in
[upstream resume](https://github.com/giodl73-repo/git-asset/blob/94865c07928275bb130cf3e002dc48ac82d9cfce/context/RESUME.md).
Use synthetic Windows fixtures. Keeper read guarding across a hard link does
not qualify legacy source-writer handoff or grant removal authority.

Issue75 review lane: docs/issue75-tool-preview, input main
f12a1634a8833f822edf2a78b4df75a2b3851538. TOOL-PREVIEW-HANDOFF-PLAN maps the
six proposed requirements to current APIs/gaps and four bounded U4 PRs. It is
documentation, not a tool/preview implementation or browser qualification.
REEL requirements pin fb8ac7b2f1b435f08d5e304fd0b82b3148ec01b6. U4 remains
independent of P1-P6 storage/production adoption. Do not post issue comments or
launch tools/servers as a side effect. Existing backend work remains in the
owned feat/trusted-native-placement lane; derive its head/PR74 CI from GitHub.
Keep both lanes separate. Resume their actual next steps after this review.

Owned worktree: F:/src/git-asset-worktrees/annex-protected-eviction;
branch feat/annex-protected-eviction, draft PR72. Goal: PRODUCT_PLAN.md PR70/P1-P6.
PR70 merged a95c9de; P1/PR71 merged f12a163 (tree equals qualified a61cb94).
P2 remains incomplete; P3-P6 follow. Checkpoint9427fdf passed all18 hosted checks in run44954410238. The next
a588ddd passed hosted run44958115092. Derive newer status from GitHub.

P2 contract/control entry: specs/ANNEX-PROTECTED-EVICTION.md. Current work uses
inputa588ddd for connected native fixtures. Device-view comparison consumes actual pinned native source/keeper
directory handles. Same volume/disk/serial refuses; unknown bus/version/serial
holds. Bounded selected queries, cooperative timeout, no kernel IO preemption.
Raw hardware identifiers/paths are not exported. Fresh F:/M: device and native
ordinary/bare keeper fixtures pass. StorageSeparation retains actual guards and
rechecks observations; it grants no owner/fault-domain or deletion authority.

Existing mechanisms: exact retained snapshot point lookup, aggregate namespace
fences, native keeper guards, shared lifecycle::Incarnation, read_published and
accounted::reopen. Native publication/copy controlled constructors honor the
actual config gate. Packet::apply_once/reopen reuses two accounted documents;
old intent without completion is uncertain and never repeats its effect. The
callback guard survives completion settlement and exact reopen. Native Output
retains the original empty job handle. Missing/recreated job names prove no
quiescence. ReplayBasis alone is not physical birth or removal authority.

Connected synthetic fixture composes real native keeper/device guards, actual
control-root retirement, one-shot native effect and incarnation-bound restore.
It tests ordinary/bare pools, completion-before-metadata interruption, uncertain
intent without completion and stale replay preserving the same-hash restore.
Only new synthetic F:/M: roots; native dropkey requires explicit --force and the
fixture validates its exact-key JSON response. ReacquisitionPlan::payload_plan
is now a readonly public accessor; wire/hash domains and limits are unchanged.
This is fixture orchestration, not a shipped controller/admission/CLI.

Producer-closure correction (input36b7de1): native import recovery formerly
acknowledged visible bytes after incomplete intent without durable quiescence.
Packet::apply_once_prepared now fences retryable staging before effect intent;
native command/read guards span completion. Intent without completion stays
Uncertain, even with visible bytes/offline scratch; registration/credit remain.
New native plans bind effect_protocol=one_shot_guarded_v1 and expose
has_guarded_effect(); absent-field legacy plans/acknowledgements remain readable
but are not qualified eviction closure. Actual producer completion still needs
exact witness reopen and physical-origin control binding, never just the flag. See producer-* logs for this revision.

Readonly native planning (inputd2ea6ff): planning::Profile defaults deny;
mixed legacy authority is unsupported. Plan::producer_completion reopens guarded
original physical producer witnesses/control origins; aliases, legacy and unknown
producers refuse. Assessment::read_for_plan pins source bytes to the exact basis.
StorageSeparation::plan combines those tokens, actual keeper snapshot/devices,
private policy, configured control roots and a reverified metadata grant into a
canonical64KiB native plan over the existing metadata retirement plan. Source/
keeper proof guards must be dropped before metadata execution; source byte guard
only after durable effect intent. Only the owned fixture executes native removal.
This is readonly planning, not the native apply/status/cancel/restore controller.
Initial planner has one explicit keeper/domain; no silent quorum reduction.

Internal execution (input9c1ce35) now consumes the frozen plan. Optional runtime
binds effect store/ledger/job, source metadata job and metadata namespace roots;
None preserves legacy preview policy identity and remains non-executable. Source
native config gitasset.evictionpolicy seals runtime policy before publication.
Fresh apply rebuilds proofs, persists both intents, holds original command group
through completion and reports source file allocation separately from metadata.
Resume uses full source namespace/config fences without today's no-demand check;
new raw job demand does not repeat removal. Unknown effect never runs again;
historical old completion preserves restored same-hash media. Bounded status is
readonly. Controller is internal, not a public removal SDK/CLI yet. Guarded cancellation (inputc3a3ab7) now rechecks native effect state inside
the original metadata operation fence before abort/continuation. It retains
surviving bytes, tolerates new raw demand and refuses unknown/completed effects.
Cancelled historical resume cannot run removal. Native crash qualification and
remaining authority/epoch handoff stay open; see cancel-* evidence.

Public controller (input24c9ed4): planning::Controller assembles fresh source/
keeper bytes, original producer completion, snapshot/device and metadata proofs;
public plan/apply share acquisition and apply compares the entire frozen plan.
Status/resume/cancel delegate the same internal recovery engines. Construction
performs no IO; denied owner input holds, mixed legacy remains Unsupported.
See docs/guides/NATIVE-EVICTION.md and controller-* qualification evidence.
CLI entry (input61816a0): git asset storage native describe/plan/status/apply/
resume/cancel delegates Controller via a bounded trusted profile. Mutation flags
default false; saved preview ID and original-operation recovery are explicit.
Ordinary storage sibling dispatch remains. See native-cli-* external evidence.
Abrupt exits (inputc05a69d): test-only child stops without Rust cleanup at
metadata intent/no native effect, post-removal unknown, and durable completion.
Parent exercises original CLI/SDK recovery, new demand, retained reopen and
same-hash restore/history. See exit-* source/terminal evidence; no production
fault hook. Paused native races (input4c35c15) now exercise actual same-object
job admission/config replacement refusal while unrelated source enrollment
progresses, then new-demand cancellation after exit; race-* evidence is external.
Unrelated payload (input3ddeab5) also publishes/reopens through controlled
Publisher in that same pool/namespace/witness/ledger during the pause; parallel-*
evidence. Legacy actor-control boundary was asked asynchronously, not inferred.
P2 closure audit: docs/reviews/p2-closure-audit.md. Joined controlled copier
(input9042dff) removes keeper input, uses source bytes, retains exact destination
producer, then native eviction/restore. See pipeline-* terminal evidence.
Next: authority/roster/epoch handoff and remaining P2 coverage/closeout;
production activation/acceptance remains unavailable.
Core limits remain16 births,16384 commands,16MiB journal; P5 scale gates stay open.

Evidence: F:/src/tig-dev-temp/git-asset-protected-eviction-20261121.
Build: F:/src/tig-dev-builds/annex-protected-eviction-20261121;
CARGO_HOME F:/src/tig-dev-cargo; TEMP/TMP F:/TigQ/c177; one build job.
Preserve dirty C:/src/Tig, F:/src/Tig, other worktrees/previews/runtime bindings.
Resume starts no cache, census, hydration, transfer, render or deletion.
Synthetic fixtures only; no production changes, independent or human approval.

[Prior complete startup history](https://github.com/giodl73-repo/git-asset/blob/9427fdf982d65a0383580ce42d603464afae7f4f/context/RESUME.md).


P4 source-generation component: feat/source-generation-use, PR73 stacked on
PR72. Code0ccb137cf5a78416e780f8a3372017f642a5cb71 passed hosted44995031081
(Windows/Linux full matrix). Six focused Windows fixtures, strict full-workspace
Clippy, standalone Git check, fmt/branding/L0 pass. The combined local regression
also passed; its compiled b9f9d84 runtime differs only by the removed unit-expression
lint fixed in0ccb137. Contract specs/SOURCE-GENERATION-USE.md; producer review
 docs/reviews/source-generation-use.md. Fresh create_fenced enables exact source
use; old formats stay readable and are not upgraded. No expiry or byte deletion.
REEL bridgebe3bec855820a824e53ab2cffdc2756654d380a8 passed hosted44995403521
(Windows/Linux), 30 default Windows tests and four actual ordinary/bare native
cases. ProtectedInputs owns actual byte/config/namespace and original-generation
guards; copied metadata cannot redirect the guard away from the roster root.
Evidence: source-use-fixed-source.json and reel-public-source-use-source.json in
the external evidence root above. Existing private route/bindings are untouched.
Next P4: original task/request/control retention, full ProtectedJob/Lifecycle and
joined uncertain-publication/crash recovery. Retained verification after explicit
job settlement must not reactivate the old generation. P2 actor/roster/epoch
handoff stays OPEN pending the legacy-boundary answer; MixedLegacy Unsupported.
No production migration, independent or owner acceptance is implied. Derive the
latest documentation-head CI from GitHub; qualified runtime SDK pin stays0ccb137.
