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
- Local catalogue: work counts, nested navigation, search, versions, guarded
  metadata expansion, coverage, copy observations, placement preferences and
  work/version/object comparisons. Movement/removal remains a held review.
  Record impact exposes admitted consumer paths, coverage gaps and exact selected
  navigation through explicit relationship selection and source-fenced paging.
  Record comparison uses an exact in-memory baseline across projects, exposing
  independent identity/field/slot differences without opening payloads.
- Agent queries: versioned JSON discovery, inspection, admission diagnostics,
  search, byte/record comparison and explained impact through admitted graphs.
  Explicit work-scoped manifest references reuse nested adapters and verified
  readers, with source/content fences and document/field provenance.
  Manifest impact joins all matching owners to one admitted graph, separating
  association evidence from conditional native consumers and coverage gaps.
  Missing semantic profiles and excluded inventories remain explicit gaps.

Current contracts and [architecture](docs/ARCHITECTURE.md) limit each claim.
[Evidence](docs/vtrace/EVIDENCE.md) records historical checkpoints rather than
certifying full requirements or every consumer.

## Next delivery sequence

Continue native integration using existing mechanisms: qualify downstream
adoption of retained candidates and participating-pruner enforcement.
Keep registration, creative selection,
approval and delivery separate. Do not replace git-annex's content engine.
Team-facing commands and teaching fixtures must describe only qualified behavior.

Current owned slice: [native candidate publication](specs/ANNEX-CANDIDATE-PUBLICATION.md).
Metadata capacity transitions (#64/#67) are merged; consumer adoption stays explicit.
The candidate SDK has synthetic and actual ordinary/bare native qualification;
consumer profile adoption and production acceptance remain separate gates.

## Delivery priorities


| Priority | Milestone | Exit condition |
| --- | --- | --- |
| 1 | Finish current catalogue changes and user UX test | Protected checks pass; user can find a work, compare versions, trace an owner and return without losing context; task failures become regression fixtures |
| 2 | Unified storage plan and agent contract | Immutable plan/operation identities, enrolled place generations, object set, source/policy/claim basis, capabilities, budgets, authority and holds; explicit complete protection scope before removal; same plan powers CLI and UI |
| 3 | Recoverable local copy orchestration | Plan/apply/status/resume in temporary stores; bounded streaming, destination staging, hash/length readback, retained destination and restart-safe acknowledgement; source preserved |
| 4 | Guarded local eviction | Require complete scoped source/job/draft claims from priority 2; hold overlapping protection through revalidation/removal, retain required independent domains, and prove exact-copy recovery; report file/allocation results separately |
| 5 | Explicit cloud and Git LFS upload adapters | One selected remote provider first, then LFS upload; retry, failed/lost acknowledgement, corrupt/missing readback, credentials and unsupported retention capabilities exercised; no remote retirement from mere upload success |
| 6 | Git-driven lifecycle reconciliation | Commit/branch/merge/revert produce idempotent desired-placement deltas; protect drafts/jobs and retained revisions; event replay and missed events converge without global locks |
| 7 | Bulk asset retirement and final deletion | Block/age/category selection of exact superseded versions; preview shared/protected content and affected stores; separately authorize eligible last-copy deletion; resume batches and measure reclaim |
| 8 | Metadata retirement and operational lifetime | Bounded deletion receipts and retention windows; authority/replay fences permit eventual record retirement; journal continuation preserves witnesses, generations and original retry outcomes |
| 9 | Scale, onboarding and release | Derived incremental indexes keyed to exact sources/rules, measured warm lookup and sparse updates, generic sample enrollment and operator/agent examples; rebuild survives deleted derived state |

UX testing can run in parallel with priorities 2-3. Storage selection should show
unique content, copy bytes, scratch peak, retained obligations, estimated eligible
bytes and measured results separately. Add grouping/filter changes only when they
help answer a concrete task such as why one work grew or what blocks reclamation.
Never count prospective eviction as free space for an admitted job.

## Placement, history and lifetime

A trusted merge request pins an exact commit and operator policy. Working,
history and backup roles are configurable. Resolve precedence before unioning all
branch/draft/job/retention requirements; unknown coverage holds removal. Old
versions still needed by any job remain protected. Copy/readback and independently
retained custody precede eligible cache eviction. Final deletion and eventual
metadata forgetting require separate consent, stale-replay/authority fences and
recoverable original outcomes. Physical reclaim and estimated headroom differ.

The local teaching fixtures cover rename, final loss, reacquisition, Git return,
restored working copies and remote publication. They do not qualify native
production writes or removal. Private journal/history/checkpoint/continuation
prototypes retain default-off controller, admission and lifetime gates. Their
original proofs remain historical, not current activation authority.

Contracts: [placement](specs/PLACEMENT-RECONCILIATION.md),
[roster precedence](specs/ROSTER-PRECEDENCE.md),
[final retirement](specs/PAGED-RETIREMENT.md),
[journal activation](specs/JOURNAL-ACTIVATION.md),
[successor](specs/JOURNAL-SUCCESSOR.md),
[metadata capacity](specs/METADATA-CAPACITY.md),
[teaching](specs/TEACHING-CATALOGUE.md).

## Full outstanding obligations

| Milestone | Exit condition |
| --- | --- |
| Consumer completeness | Explainable declared versus consumed inputs, opaque gaps, active jobs and exact profile/source coverage |
| Lifecycle reconciliation | Durable scoped demands and idempotent plans across Git changes, interruption and concurrent work |
| Guarded execution | Independent retained custody, provider revalidation and authority before retirement; physical reclaim measured in isolated fixtures |
| Final deletion and metadata lifetime | Exact bulk scope, protected shared content, explicit last-copy authority, resumable outcomes and eventual metadata retirement with stale-replay fencing |
| Catalogue usability | Comparable coverage, shared bytes, versions and primary subtypes; explicit enrollment and tool handoff |
| Providers/transfers | Capability/operational/license review and separate recovery/custody proof for remote or optional torrent adapters |

No deadline or completed consumer migration is implied. Catalogue work can advance
independently of deletion. Do not create a second cache engine or global cache
lock to complete a milestone. Estimates, observations, protection and authority
remain distinct.

The [17 work packages](docs/vtrace/WORK_PACKAGES.md),
[catalogue roadmap](docs/CATALOGUE-UI-PLAN.md) and
[adoption gates](docs/CONSUMER-ADOPTION.md) provide finer allocation.


Full prior progress/proof history and designed fixture matrix: [immutable roadmap](https://github.com/giodl73-repo/git-asset/blob/9658c13f9c4a169e468a99feb30b4e4fe8d11351/PRODUCT_PLAN.md).
