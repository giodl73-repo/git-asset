# git-asset roadmap

git-asset is the asset relationship and lifecycle extension of git-annex.
Git keeps small authored definitions, references and selected history. git-annex
owns large immutable file contents, native copies/remotes and transfers.
git-asset coordinates exact identity, versions, dependencies/usage, candidates,
selection, jobs, placement obligations, retention and observed evidence.
Generic byte interfaces and optional typed models remain reusable across projects.

Public experimental source; see [release checks](docs/PUBLIC-RELEASE.md).

## Implemented experimental slices

Exact identity/trees, bounded metadata, Git snapshots, local publication,
capacity/recovery, optional typed graphs/proposals, restore, catalogue and agent
queries exist. Optional Windows annex adapters provide managed local reads and
retained candidate publication; native copy/pruner execution and consumer adoption
remain open. Existing flat/CAS/LFS adapters remain supported by their contracts.
See [architecture](docs/ARCHITECTURE.md) and [evidence](docs/vtrace/EVIDENCE.md)
for component scope; this is not production acceptance.

## Initial offering: PR finish plan

Input e171da7ebbe181d10b39b8e2a1a4cc78038b971f. PR70 is open; P1-P6 are
labels, not GitHub numbers. Before code freeze input, controls, scope, L0/L1/L2.

Finish: pinned Windows history, protected jobs/candidates, Git selection,
merge-driven placement, eviction/restore; per-project acceptance and rollback.

| PR / repository | Scope and dependencies | Required exit evidence |
| --- | --- | --- |
| **PR70: reproducible UX review** / git-asset | Existing two-project sample, README and walkthrough. | Exact-head protected CI; team finds work, compares versions/shared bytes, follows recovery lessons and reopens history. User acceptance is separate from producer checks. |
| **P1: native copy and retained custody** / git-asset | After PR70. Adapt existing bounded placement/capacity/recovery to admitted local annex locations; preserve source. | Copy/hash-length readback and independently retained destination before handoff; interrupted retry, full disk, corrupt/offline destination and lost acknowledgement. Bound payload/scratch/metadata/process IO; no census or spending prospective reclaim. |
| **P2: participating native eviction** / git-asset | After P1. Connect existing retirement plans and scoped fences to admitted native removal; expose bounded plan/status/apply/resume through the existing storage entry. | Jobs, drafts, shared use and retained outputs block removal across restart; unknown scope and unavailable required backup hold. Race/stale-plan/process-exit tests, surviving-byte reopen, measured reclaim and exact restore. Reject alias-as-independent custody, incomplete rosters and stale incarnation retries. Qualify roster/epoch handoff before mixed legacy/new pruning; raw annex drop does not participate. |
| **P3: trusted merge-to-placement** / git-asset | After P2. Connect existing committed-source reconciliation to the admitted executor/Action; explicit storage permissions and immutable commit/policy basis. | New golden moves to working; superseded bytes move to retained history before eligible working eviction. Duplicate/missed events, concurrent worktrees, revert, executor crash and offline backup recover without losing job protection. Use a fixed trusted worker/config; events supply commit IDs, never paths/code/credentials. Authenticate, revalidate source and reject fork/untrusted/stale triggers. Dry run precedes activation. |
| **P4: REEL SDK adoption** / reel-workflow | May start after PR70 using the qualified publication SDK; P2 required before shared production pruning. Add an isolated provider route, preserving the existing route. | Exact input preparation; durable protection for days-long manual handoffs; immutable unselected candidates; retained reopen before task completion; conflict, uncertain-publication and restart recovery. No silent latest fallback, scratch-before-retention deletion or duplicate registration. Reject traversal/collisions and wrong hash/length; timeout/cancel never deletes bytes. REEL owns semantic/task decisions. |
| **P5: pinned release candidate** / git-asset | After P3/P4. Build the versioned Windows bundle, checksums/licenses, CLI/SDK matrix, clean-machine setup and upgrade/rollback runbook BEFORE cutover. | Pin source, SDK, binaries and enrollment format; exercise setup/sample and restart/offline/full-disk recovery. Measure declared workload/metadata/journal limits; prove supported continuation or safe refusal/recovery before exhaustion. Acceptance precedes production installation. |
| **P6: project cutover and official use** / private projects | After P5. Reuse one procedure for every project; enrollments/evidence remain private. | Current/historical coverage, large-binary restore, jobs, independent backups and reversible bindings. Preserve legacy names/obligations; mixed authorities require P2's qualified handoff or stay isolated/non-pruning. Owner accepts each project; unknown entries stay unadopted. Promote the same accepted RC bytes to official release after gates pass. |

Dependencies: PR70 -> P1 -> P2 -> P3; PR70 -> P4; P3/P4 -> P5 -> P6.
P6 is a project-owned change set. Freeze candidate bytes before any production
switch; final release promotes those bytes. Linux CI does not qualify native
Windows APIs. Close implementation, verification, validation, trace and review
separately. CLI queries/review plus SDK publication suffice; no new publisher CLI.

Control anchors: P1/P2 WP-003/004/007, P3 WP-005/006/009, P4 WP-011/016,
P5 WP-000/015, P6 WP-016; REQ-001..010 and existing SPEC/IF/PKG assignments in
[work packages](docs/vtrace/WORK_PACKAGES.md). Reuse
[candidate publication](specs/ANNEX-CANDIDATE-PUBLICATION.md),
[managed reads](specs/ANNEX-MANAGED-READ.md),
[placement](specs/PLACEMENT-RECONCILIATION.md) and
[adoption gates](docs/CONSUMER-ADOPTION.md); native gaps require their own contract
review, not assumptions that local-provider fixtures qualify native custody.

## Six-role review of this plan

Target1135a2b0c6ec090214ed7d9942c7934e82cda579; producer-applied
[.roles](.roles/ROLE.md), MODEL/SAFETY, foundation scenarios and existing native/
placement/adoption contracts. H=high, M=medium; corrections incorporated above.
Producer review only; no runtime/deployment approval.

| Finding / role owner | Severity; plan correction | Remaining proof |
| --- | --- | --- |
| R1 identity-namespace | H: P4 explicitly rejects traversal/collisions and digest/length mismatches. | Exact historical inputs, malformed paths and wrong content fixtures. |
| R2 placement-policy | H: P1 bounds peak resources; P5 declares capacity/journal lifetime instead of deferring exhaustion to an unfinished prototype. | T15/T19/T20, growth/limit and supported continuation/refusal tests. |
| R3 storage-custody | H: P2 separates failure domains from drive aliases; retained custody stays fresh under removal fences. | T07-T10/T16, offline/corrupt/same-domain survivors, measured reclaim/restore. |
| R4 concurrency-recovery | H: P2/P6 require legacy/roster/epoch handoff and restored-copy incarnation fencing; isolation cannot be called shared pruning adoption. | T12-T14/T24, restart/claim races, stale deletion replay and handoff failure. |
| R5 provider-privacy | H: P3 accepts authenticated commit IDs through fixed trusted code/config, not event-supplied paths or execution. | Fork/untrusted/stale event, path escape, credentials/redaction and unsupported-capability negatives. |
| R6 git-adoption | M: swap former P5/P6: qualify pinned candidate BEFORE project cutover, then promote the same bytes. | Clean-machine candidate setup, per-project reversible acceptance and rollback with obligations preserved. |

Disposition: plan corrections incorporated; implementation/adoption tests remain
OPEN at their owning PRs. L0 docs/link/hash/budget checks verify this revision;
L1/L2 are not executed by this prose review. Implementation may not waive a
failed gate to keep the six-label schedule.

## Beyond the initial offering

Bulk/age-based final deletion, metadata forgetting, private journal activation,
more cloud/transport providers, full teaching and scale expansion follow later.
Initial eviction never deletes the last retained copy. Unknown demand holds
removal; policy precedence precedes obligation union. Retained custody, authority
and physical reclaim stay separate from planned placement and estimated savings.
The [work packages](docs/vtrace/WORK_PACKAGES.md),
[placement contract](specs/PLACEMENT-RECONCILIATION.md),
[roster precedence](specs/ROSTER-PRECEDENCE.md),
[final retirement](specs/PAGED-RETIREMENT.md) and
[adoption gates](docs/CONSUMER-ADOPTION.md) retain broader obligations. No new
cache engine/global lock or automatic activation is authorized by this plan.

Full prior progress/proof history and designed fixture matrix: [immutable roadmap](https://github.com/giodl73-repo/git-asset/blob/9658c13f9c4a169e468a99feb30b4e4fe8d11351/PRODUCT_PLAN.md).
