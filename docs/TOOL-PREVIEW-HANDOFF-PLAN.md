# Tool handoff and guarded previews: issue75 review

Proposed delivery plan for [issue75](https://github.com/giodl73-repo/git-asset/issues/75),
not an implemented API or accepted preview contract. Input: public main
f12a1634a8833f822edf2a78b4df75a2b3851538; REEL requirements
[fb8ac7b](https://github.com/giodl73-repo/REEL/blob/fb8ac7b2f1b435f08d5e304fd0b82b3148ec01b6/docs/workflow-ux-requirements.md).
PR74 source b02d5ad7e5a373bc672e528d52589162a3d9ad74 was separately inspected;
its storage receiver/action recovery does not supply a catalogue media API.

git-asset should provide exact references, generic tool registration, bounded
preview access and candidate/observation interfaces. REEL provides the production
board, film/comic interpretation, language clocks, provider attempts and review.
REEL's standalone read-only board can progress independently. This U4 sequence
does not add a storage-engine replacement or production-adoption prerequisite to
the existing P1-P6 finish plan.

Review controls: WP-015/016; REQ-006/007/010; SPEC-014 and existing source,
publication/protection contracts; IF-013 with IF-007/008/010; PKG-008 with
PKG-004/006 at the SDK seams. DOCS/UI/PROVIDER/INTEGRATION. This change records
the audit/proposed sequence and a controlled ordinary-source allowance; no runtime
API is added. Implementation PRs must freeze their own input, specific interface
controls, bounds, entry/exit and L0/L1/L2 profiles.

## Current interfaces and gaps

Available means the named component exists at the stated source, not that the
combined REEL experience is qualified. Needs qualification means reusable
mechanisms exist but their required integration is missing.

| Issue requirement | Assessment | Existing surface | Missing contract / qualification |
| --- | --- | --- | --- |
| 1. Tool/view registration | Missing | [catalogue Enrollment](../crates/git-asset/src/catalogue/mod.rs) has display, fields, graph, collections and storage; U4 is open. | Trusted versioned tool/capability registry, fixed local connection, explicit Open action, bounded supported-context negotiation. No executable asset fields. |
| 2. Exact launch context | Needs qualification | `catalogue::Project::inspect`, `check_snapshot`; `catalogue::query::Request.expected_snapshots/expected_basis`; record pins, named streams, work IDs and existing UI request/return tokens. | One versioned launch/return descriptor, exact record-or-candidate selector, explicit current versus historical mode and variant context; server and tool both validate it. Existing stale checks do not constitute a tool handshake. |
| 3. Guarded previews | Missing HTTP surface; SDK needs qualification | [managed annex Reader](../crates/git-asset/src/annex/managed.rs) returns a guarded `Read + Seek`; `catalogue::Project::verification`/`catalogue::verify` explicitly check named existing bytes. | Named preview descriptors/MIME, provider capability admission, scoped byte serving, range/verification budgets, bounded concurrent readers, cancellation and typed availability/errors. Current HTTP routes return metadata/verification, not media bodies. |
| 4. Unselected candidates | SDK available; catalogue discovery missing | [annex publication](../crates/git-asset/src/annex/publication.rs) provides Candidate, retained publication/reopen and original recovery. Registered named streams are inspectable. [native job protection](../crates/git-asset/src/source/native/job/protection.rs) supplies existing job protection mechanisms. | General exact candidate discovery and preview handles for active-work outputs, with owning operation/attempt provenance and protection scope. A named stream is not a generic candidate feed. Discovery must neither register duplicates nor select. |
| 5. Observation/navigation | Navigation available; change stream missing | Work/asset queries, compare, impact, references, snapshot/basis checks and bounded offset pages; [agent queries](AGENT-QUERIES.md). | Scoped cursor/change or explicit invalidation/resync contract for registered subjects and claims. Current paging/basis framing is not a durable workflow-event service. |
| 6. Action/result fencing | SDK available; tool-session bridge missing | Existing protected-job and retained-publication/original-operation recovery; PR73 adds exact current-source generation guards, PR74 adds saved native storage actions. | Session handles must preserve original task/attempt/operation and candidate identity across reconnect. Native storage actions are not REEL provider attempts; no UI lifecycle may cancel/release/retry them implicitly. |

The catalogue's `/api/storage/preview`, `/api/storage/batch-preview` and agent
`preview_admission` are planning previews. They do not serve images, audio or
video. Reuse their source/basis discipline, not their names as evidence of media
support. `ResolverProfile.byte_source` currently enrolls the existing byte-source
profile; a guarded native preview adapter must be explicitly admitted and tested.

Catalogue failures currently include string errors. New tool/preview endpoints
need structured error codes at their boundary; this need not rewrite every
existing catalogue route. Distinguish missing, corrupt, stale/conflicting, denied,
unsupported and uncertain. Availability and observed integrity remain separate
from retained custody, review, approval and selection.

## Contract decisions to freeze before implementation

The following names/fields are proposals, not current schemas or command names.

- Tool registration is private enrollment: tool ID/version, protocol version,
  supported asset/work capabilities, and a fixed trusted connection mechanism.
  An initial already-running loopback tool with a scoped URL is sufficient.
  Asset records may name declared content/capabilities; they cannot supply a
  process, command line, script, filesystem root or arbitrary destination URL.
- Launch captures project/work, exact record pin or candidate handle, immutable
  commit/snapshot/stream, source basis and bounded opaque variant context. Return
  navigation captures exact catalogue IDs and view state. It is not a write grant,
  retention lease or proof that a provider attempt completed.
- Current-mode stale bases refuse. Historical mode explicitly names the old
  snapshot; it never resolves current/latest. Existing mutable catalogue snapshot
  switching must not silently retarget another tab's launch or historical view.
  Reuse inspector/detail request tokens to discard late responses.
- Preview descriptors name a specific Object (SHA-256 and length), MIME and role
  through an admitted adapter. No filenames, newest-object guessing, media census
  or automatic derivative generation. REEL registers thumbnails, waveforms, proxy
  movies and composed pages through normal jobs.
- The broker opens the exact object with the provider's byte/configuration and
  source guards, retaining them through the response. It supports only admitted
  image/audio/video formats and bounded range behavior. Do not render active
  HTML/script content under the catalogue origin or expose provider paths/tokens.
- `Read + Seek` does not prove efficient partial integrity. The native reader can
  verify a whole object before a small range. Initial qualification should admit
  bounded named proxies, and reject a large master unless an adapter proves
  bounded/chunk integrity. Never hide a whole-episode hash/copy behind a range.
  Charge verification and response reads to the budget. Reuse only provider-
  qualified immutable handles within finite reader leases; an open UI must not
  hold Main's source-generation gate for its entire lifetime and stall production.
- Candidate handles bind logical identity, exact bytes, origin operation and
  owning protection. Active-job access uses an admitted SDK bridge, not a raw
  client-supplied journal/claim. Returning a candidate leaves older selection
  intact. Selection remains a distinct expected-baseline transaction.
- Start observation with bounded snapshot/basis polling and explicit invalidation.
  Any later change cursor binds scope and source epoch; restart/gaps request resync.
  Do not invent a monotonic event history from UI memory. REEL owns job progress.
- Preview capabilities may expire and be renewed after revalidation. Their expiry,
  browser close or disconnect affects access only; durable job/retention state
  persists. Reconnect recovers the original operation. Uncertain provider work
  stays uncertain; UI refresh cannot issue another provider effect.

Candidate discovery and media access must disclose coverage and supported provider
capabilities. Empty, unavailable, unknown and unsupported are different outcomes.
Use named reads only; neither opening the catalogue nor requesting a missing
preview may hydrate, render, upload or mutate selection.

Starting bounds to validate in the implementation contracts:16KiB launch/context,
64KiB descriptor pages,25 candidate rows, one range per response,8MiB response
bytes and a64MiB whole-object verification ceiling for initial proxy providers.
Add finite per-session bytes/readers/time and explicit admission refusal. These
are proposed test targets, not existing supported limits. Larger media and any
chunk-verified adapter need their own measured qualification.

## Four implementation PRs

Each PR includes its own failure fixtures, source pins, documentation and producer
review. Schema/CLI discovery precedes UI wiring. Do not defer correctness tests to
the final browser PR.

| PR | Concrete deliverable and dependency | Exit evidence |
| --- | --- | --- |
| U4-A: enrolled tools and exact launch | Add bounded registry/context schemas and library/CLI discovery over existing exact catalogue queries. Validate version/capability, explicit current/historical mode and return context. No general command execution or embedded third-party UI. Can start from main. | Synthetic exact work/version launch descriptor and return; unknown tool/capability, tampered candidate, wrong project, stale basis, same-ID/different-snapshot and late-response refusal. No byte/provider work merely to discover a tool. |
| U4-B: named guarded preview broker | After A. Add explicit preview adapters/descriptors and scoped local byte serving over existing readers. Use exact selected/named streams first; add no preview renderer. Capability, MIME, range and object-verification limits are part of the contract. | Real synthetic image/audio/video HTTP bytes; hash/length, range boundaries, unsupported/missing/offline/corrupt, unsafe MIME, traversal, concurrent readers and disconnect. Demonstrate a large master is refused without reading it wholesale; qualified small proxy works. Native guard/race tests remain separate from local/CAS/LFS qualification. |
| U4-C: candidate and observation bridge | After A; reuse B's exact byte path. Add bounded discovery of registered and active-work candidate handles and scoped polling/invalidation. Reuse job/publication recovery through the qualified SDK; do not add REEL semantics or effect endpoints. | Returned candidate appears with original operation/attempt while prior selection is unchanged. Process restart/reconnect preserves protection and original identity; expiry/close do not release it. Stale cursor/epoch resync, bounded pages, late events and uncertain work do not cause another effect. |
| U4-D: catalogue UX and REEL acceptance | After A-C. Add explicit Open in enrolled tool, lazy named previews, unavailable states and exact history/impact/return navigation. Integrate with REEL's separately owned read-only board; reuse current catalogue controllers. | Actual synthetic desktop and narrow-browser walkthrough with visible image and audio/video controls, exact historical return, unselected candidate, missing preview, reconnect and stale launch. Demonstrate actual playback/seeking; DOM doubles alone cannot close visual/playback acceptance. |

The REEL board PR can run in parallel with A-C using existing qualified exact
assets/workflow journals. Its frozen-export/1GiB host limit remains its own bound;
it does not qualify live episode-scale access. No private production installation
or asset adoption is authorized by these PRs.

A can initially enable only qualified record/work capabilities. Candidate launch
remains explicitly unsupported until C admits its protected-handle adapter; a
syntactically valid opaque handle alone is not a qualified candidate context.

## Source/package pin and acceptance

Use Rust package `git-asset` (`git_asset` in Rust), with exact Git source and
required feature set. U4-A's audited main source is f12a163 above. Do not invent
a released SDK version from a draft branch. The separate P4 REEL route qualified
source-use SDK0ccb137cf5a78416e780f8a3372017f642a5cb71; it is a component pin,
not main or a tool/preview interface. Native source-generation-dependent use
must qualify that prerequisite; U4 does not require unrelated placement adoption.
Freeze the final tool/preview source, protocol and binary pins after A-D acceptance.

Issue75 acceptance maps to A (exact launch/return), B (declared media and bounded
large reads), C (candidate, job protection, reconnect/invalidation) and D (combined
historical/visible browser experience). Preserve the original six scenarios;
metadata tests or storage CI alone do not close this issue.

Six producer .roles findings, owned by the implementing maintainer: H identity
requires exact current/historical context (A); H placement/custody forbids preview
availability as retention authority (B/C); H recovery preserves original operations
and long-lived claims (C); H provider/privacy rejects executable records, leaked
paths and hidden master hydration (A/B); M Git/adoption keeps authored definitions
in Git and operational state asset-side, with independent source/acceptance pins
(all). These are producer checklists, not independent reviewers or approval.

Implementation: not started by this documentation review. Verification: source
audit and documentation checks only. Validation: synthetic HTTP/provider/browser
scenarios above remain required. Trace: issue75 -> A-D and the cited existing
contracts/source seams. Review: producer-applied; independent/human acceptance OPEN.
