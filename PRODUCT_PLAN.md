# git-asset roadmap

git-asset is the asset relationship and lifecycle extension of git-annex.
Git keeps small authored definitions, references and selected history. git-annex
owns large immutable file contents, native copies/remotes and transfers.
git-asset coordinates exact identity, versions, dependencies/usage, candidates,
selection, jobs, placement obligations, retention and observed evidence.
Generic byte interfaces and optional typed models remain reusable across projects.

The experimental source baseline is public. Its six-role review and ongoing
release checks are recorded in [public release](docs/PUBLIC-RELEASE.md).
This is source availability, not a stable product or full storage acceptance.

## Implemented experimental slices

- Pure identity/trees, bindings, claim handoff and bounded placement arithmetic.
- Local publication, scoped coordination, capacity journals, retained claims and
  recovery primitives with synthetic and real-client fixtures.
- Exact commit/index extraction, source journals and download-only Git LFS cache
  interoperability. Stores are explicitly enrolled, never discovered.
- Optional exact local annex reads, pool-pinned namespace rosters and compatible
  shared use guards. Optional native candidate publication composes durable
  retention, accounted ingress, original-operation recovery and guarded readback.
  Native pruner execution and production consumer adoption remain open.
- Optional native records, schema admission, graphs, sparse variants, production
  contexts, revision/delivery history and sparse Git proposals.
- Explicit Rust project restore, handoff/progress evidence and history inspection.
- Catalogue: works/navigation, search, exact versions, metadata coverage, copy
  observations, preferences, comparisons and source-fenced impact paging.
  Movement/removal stays held. Cross-project comparisons preserve exact bases.
- Agent queries: bounded JSON inspection/admission, search, comparison and impact
  through admitted graphs/manifests with source/content fences and provenance.
  Missing semantic profiles and excluded inventories remain explicit gaps.

Current contracts and [architecture](docs/ARCHITECTURE.md) limit each claim.
[Evidence](docs/vtrace/EVIDENCE.md) records historical checkpoints rather than
certifying full requirements or every consumer.

## Initial offering: PR finish plan

Input e171da7ebbe181d10b39b8e2a1a4cc78038b971f. PR70 is open; P1-P6 are
proposed labels, not GitHub numbers. Each implementation freezes its own input,
controls, scope and L0/L1/L2 profile before code. Planning proves no runtime gate.

Finish line: an explicitly enrolled Windows team can install a pinned build,
inspect exact history, run protected work, retain candidates, select via Git,
and reconcile working/history/backup placement after a trusted merge, including
eligible working-copy eviction and restoration. Every adopted project records
its supported scope and rollback. Git-annex remains the content engine.

| PR / repository | Scope and dependencies | Required exit evidence |
| --- | --- | --- |
| **PR70: reproducible UX review** / git-asset | Existing two-project sample, README and walkthrough. | Exact-head protected CI; team finds work, compares versions/shared bytes, follows recovery lessons and reopens history. User acceptance is separate from producer checks. |
| **P1: native copy and retained custody** / git-asset | After PR70. Adapt existing bounded placement/capacity/recovery to admitted local annex locations; preserve source. | Copy/hash-length readback and independently retained destination before handoff; interrupted retry, full disk, corrupt/offline destination and lost acknowledgement. Bound work to named objects, not a cache census. |
| **P2: participating native eviction** / git-asset | After P1. Connect existing retirement plans and scoped fences to admitted native removal; expose bounded plan/status/apply/resume through the existing storage entry. | Jobs, drafts, shared use and retained outputs block removal across restart; unknown scope and unavailable required backup hold. Race/stale-plan/process-exit tests, surviving-byte reopen, measured reclaim and exact restore. Raw annex drop is not a participating pruner. |
| **P3: trusted merge-to-placement** / git-asset | After P2. Connect existing committed-source reconciliation to the admitted executor/Action; explicit storage permissions and immutable commit/policy basis. | New golden moves to working; superseded bytes move to retained history before eligible working eviction. Duplicate/missed events, concurrent worktrees, revert, executor crash and offline backup recover without losing job protection. Reviewed dry run precedes activation; no automatic runner installation. |
| **P4: REEL SDK adoption** / reel-workflow | May start after PR70 using the qualified publication SDK; P2 required before shared production pruning. Add an isolated provider route, preserving the existing route. | Exact input preparation; durable protection for days-long manual handoffs; immutable unselected candidates; retained reopen before task completion; conflict, uncertain-publication and restart recovery. No silent latest fallback, scratch-before-retention deletion or duplicate registration. REEL owns semantic/task decisions. |
| **P5: project enrollment and cutover** / private project repositories | After P3/P4. One reusable procedure, with project-owned enrollment changes for every current project; private paths/evidence stay private. | Per-project source/package pins, current AND historical coverage, actual configured storage roles, large-binary copy/read/restore, protected jobs, backup evidence and reversible binding switch. Unknown entries remain unadopted. Preserve legacy names/data and record owner acceptance; never infer all projects migrated from one pilot. |
| **P6: pinned operational release** / git-asset | After P5 acceptance. Versioned Windows bundle, checksums/licenses, clean-machine setup, CLI/SDK support matrix and upgrade/rollback runbook. | Exercise published setup/sample and one accepted real workflow on the pinned build; retain restart/offline/full-disk recovery handles. Exact-head CI, release evidence and owner acceptance precede tag/artifact publication. Linux CI does not qualify Windows-only native APIs on Linux. |

Dependency chain: PR70 -> P1 -> P2 -> P3 -> P5 -> P6; PR70 -> P4 -> P5.
P5 is a project-owned change set, not a giant cross-repository public PR.
Close implementation, verification, validation, trace and review separately.
CLI queries/review plus SDK protected publication suffice; no new CLI publisher.

Control anchors: P1/P2 WP-003/004/007, P3 WP-005/006/009, P4 WP-011/016,
P5 WP-016, P6 WP-000/015; REQ-001..010 and existing SPEC/IF/PKG assignments in
[work packages](docs/vtrace/WORK_PACKAGES.md). Reuse
[candidate publication](specs/ANNEX-CANDIDATE-PUBLICATION.md),
[managed reads](specs/ANNEX-MANAGED-READ.md),
[placement](specs/PLACEMENT-RECONCILIATION.md) and
[adoption gates](docs/CONSUMER-ADOPTION.md); native gaps require their own contract
review, not assumptions that local-provider fixtures qualify native custody.

Later releases: bulk/age-based final deletion, eventual metadata forgetting,
private journal-controller activation, additional cloud providers, full teaching
curriculum and scale expansion. Initial eviction removes eligible working copies;
it never authorizes last-retained-copy deletion. No deadline is implied before
the native execution gaps are qualified.

## Placement, history and lifetime

Union branch/draft/job/retention obligations after resolving policy precedence.
Unknown coverage holds removal. Fresh copy/readback, independent retained custody
and current authority precede eviction. Final deletion and metadata forgetting
need separate consent and stale-replay/recovery fences. Physical reclaim differs
from estimated headroom. Synthetic lessons and private controller prototypes do
not qualify native production removal or activate their default-off gates.

Contracts: [placement](specs/PLACEMENT-RECONCILIATION.md),
[roster precedence](specs/ROSTER-PRECEDENCE.md),
[final retirement](specs/PAGED-RETIREMENT.md),
[journal activation](specs/JOURNAL-ACTIVATION.md),
[successor](specs/JOURNAL-SUCCESSOR.md),
[metadata capacity](specs/METADATA-CAPACITY.md),
[teaching](specs/TEACHING-CATALOGUE.md).

## Continuing obligations

The [17 work packages](docs/vtrace/WORK_PACKAGES.md),
[catalogue roadmap](docs/CATALOGUE-UI-PLAN.md) and
[adoption gates](docs/CONSUMER-ADOPTION.md) retain the broader backlog:
consumer completeness, guarded lifecycle/custody, final deletion and metadata
lifetime, catalogue usability, and provider capability/license/transport review.
Unknown consumed inputs and inventories stay explicit. Qualify optional remote
adapters separately; no second cache engine or global lock. Estimates, observed
copies, protection and removal authority remain distinct.

Full prior progress/proof history and designed fixture matrix: [immutable roadmap](https://github.com/giodl73-repo/git-asset/blob/9658c13f9c4a169e468a99feb30b4e4fe8d11351/PRODUCT_PLAN.md).
