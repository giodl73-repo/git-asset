# Experimental native eviction SDK

`git_asset::annex::retirement::planning::Controller` is available with
`annex-publish`. Construction only borrows a trusted owner `Profile`; it performs
no IO. Operations are explicit. This is an experimental namespace-owned Windows
adapter, not mixed legacy pruning or production acceptance. The
[contract](../../specs/ANNEX-PROTECTED-EVICTION.md) lists remaining P2 gates.

Use the existing publication SDK for ingress/copy. The controller coordinates
existing lifecycle and protection engines; it introduces no content store.

| Method | Inputs and behavior |
| --- | --- |
| `plan` | `Request { operation, snapshot, grant }`, `Sources` and `Nodes`; returns canonical immutable `Plan` after actual source/keeper/producer/device checks. |
| `apply` | Frozen `Plan`, fresh `Sources` and `Nodes`; reconstructs all proofs and requires the identical plan before any native removal. Fresh attempts only. |
| `status` | Frozen plan and original source publication profile/plan; readonly bounded metadata/effect/presence report, no removal authority. |
| `resume` | Same original context plus source metadata reader/limits; reconciles original completion/cancellation without native removal. |
| `cancel` | Same original context plus source metadata reader/limits; retains verified surviving bytes and refuses any native effect intent/completion. |

`Sources` contains original physical source and keeper `Producer` resources:
publication profile/plan, existing witness store and mutable capacity ledger.
`Nodes` supplies source and keeper metadata readers, protection limits and tree
limits. Reuse existing readonly `MetadataReader`s; no caller must manufacture a
`Live` token or independently orchestrate guard release. Source/keeper resources
must match the frozen control identities and private runtime policy. A hash,
visible file, alias acknowledgement or protocol flag alone is insufficient.

The owner profile defaults deny. Initial admission requires namespace-owned
actors, independently admitted unshared direct backing, distinct failure domains,
actual distinct device observations and exactly one retained keeper. Stronger
quorums cannot be silently reduced. The source's actual configuration must seal
the exact policy/runtime before original publication. Mixed legacy actors are
held pending a qualified [existing-cache handoff](../../specs/ANNEX-PROTECTED-EVICTION.md#existing-cache-handoff-requirements).
A listed or byte-verified backup alone does not establish that handoff. If
`git asset storage native describe` reports `mixed_legacy_supported: false`,
use the catalogue to inspect copies and keep removal held.

Save `Plan::to_json()` and its ID outside Git with operational evidence. Reopen
with `Plan::from_json()` (canonical, at most64KiB), preserving original producer
plans and trusted profile bindings. Do not make a replacement plan/journal to
bypass an uncertain outcome. `Error::code()` distinguishes protection, stale,
conflict, missing/corrupt, admission, unsupported and recovery/uncertain holds.
Busy permits later retry with current proofs. Stale requires reinspection;
needs-recovery/uncertain require original-operation inspection/recovery, never blind apply or
cancellation. Missing acknowledged bytes are an incident, not permission.

A new raw job demand may coexist with historical completion or cancellation;
neither repeats removal. Same-hash restoration creates an explicit new control
incarnation; old completion reports history and current presence separately.
Reports distinguish logical subject bytes, source file allocation (fresh apply
only), metadata receipts and presence. They do not claim net free disk space.

The explicit facade CLI below uses this same controller. An abrupt-exit fixture
covers metadata intent, unacknowledged native removal and durable completion in
ordinary/bare pools, followed by original-operation recovery and retained restore.
This proves process-exit behavior, not power loss. Paused ordinary/bare fixtures
check participating same-object admission/config conflicts while unrelated source
enrollment and controlled payload publication/readback progress in the same pool.
This does not qualify arbitrary legacy exclusion or owner/epoch handoff;
production acceptance is open.
The ordinary/bare synthetic native fixture exercises the SDK preview, fresh
apply, readonly status, historical resume, cancellation and retained restore.
No cache operations run on startup, help or construction.

## Qualify one existing cache scope

Start with one enrolled root and one provider-owned area. Review object-store
eviction and owned job/staging cleanup separately. Record the following in a
private operational record.

| Evidence | What the owner checks |
| --- | --- |
| Scope and identities | Record project, enrolled cache and area, provider/store identity, device, and writer epoch. Include other roots or worktrees that can address the same representation. |
| Writer coverage | Account for services, scheduled tasks, interactive tools and restarted or paused processes. Record the effect boundary that prevents each former writer from mutating the admitted area. A profile declaration alone does not establish this. |
| Work and recovery | Reconcile active jobs, reservations and unsettled operations against their authoritative ledger and owner. For scratch, prove that the exact directory belongs to the closed operation and that no retry or outstanding handoff depends on it. Time since modification is inventory evidence only. |
| Retained data | For working-copy eviction, verify the independent keeper and retention under the current authority. For scratch, trace acknowledged outputs to their retained receipts before considering cleanup. |
| Qualification evidence | Exercise an old actor resuming, a new claim, an alias, and a replaced object or scratch identity. Save the exact scope and evidence used for the decision. Unresolved rows hold that scope. |

The native controller currently accepts namespace-owned, unshared backing.
An owner record documents the checks for a future handoff; it cannot enable
mixed legacy execution. A separately enrolled namespace-owned scope can qualify
on its own evidence while shared scopes remain under review.

### Existing reel-cas caches

The inventory identifies `reel-cas` as an existing byte layout. Its
redundant-copy provider candidate is REEL's `tier-demote`. The provider
takes an owner-scoped request, a pinned tier plan, a bounded set of SHA-256
identities, and an owned operation journal. Its preview and apply use the
same request. Use the original journal to reconcile an interrupted apply.

A git-asset handoff must match SHA-256 and byte length in both reports and
confirm that the provider's configured source is the enrolled cache being
reviewed. Keep provider executable, control roots, policy, tier plan, consumer
roster, and journal bindings in private enrollment. The catalogue report is
candidate evidence only. The provider must recheck live protection, exact
source identity, source aliases and independent custody before any removal.
Leave the provider's off-by-default bypass options disabled in the initial
pilot. Preserve the provider's completion receipt and measure source allocation
separately from changes in total volume free space.

Before enabling the handoff, test the pinned Windows executable on a synthetic
cache with changed scope, a new job, a corrupt keeper, an unknown hard link
and a process interruption. Then qualify one owner-scoped production batch.
Run `git asset catalogue demotion-match DRIVE_REPORT PROVIDER_POLICY PROVIDER_PLAN NEW_REPORT`
with absolute paths to saved evidence. It compares the bounded drive-report
page against the provider's ordinary demotion proposals and records input digests.
The provider plan is limited to 32 MiB for this matcher. Use provider verification
with current controls before relying on any match.

Then use `git asset catalogue demotion-prepare DRIVE_REPORT OWNER_REQUEST NEW_REQUEST`
to check a privately supplied ordinary demotion request. The request names the
exact objects, provider policy and plan, consumer roster, and verification budgets.
The command rechecks the match and writes a new request without widening the
batch. Use that same request with the provider's verification and apply commands.

On Windows with provider-process support, invoke the fixed REEL command through
`git asset catalogue demotion-run verify PROFILE REQUEST JOURNAL`. The private
profile schema is `git-asset.reel-demotion-provider.experimental-1`. It pins
`executable`, `executable_sha256`, `project_root`, `journal` and
`request_sha256`. Use absolute paths in owner-controlled directories.
`allow_apply` defaults to false. An explicitly admitted `apply` uses the pinned
request and its original apply journal. An uncertain outcome requires inspection
of that journal before retry. The provider still performs live checks and writes
the authoritative result. Git-asset validates the returned request and plan IDs.

The ordinary demotion provider requires a keeper. Final retirement of a
rebuildable render has its own execution and approval path.

## From cache review to a saved plan

Start with [the catalogue cache review](CATALOGUE-REVIEW.md#review-a-cache-for-eviction).
Keep the candidate's exact SHA-256, byte length, selected source cache,
verified keeper location, and reviewed project snapshots. These identify the
candidate and the evidence used to review it. An archive byte check alone
does not establish retained custody.

For ordinary or bare Git-annex stores, use the catalogue's explicitly enrolled
native cache place. **Verify bytes at …** verifies the named native copy through
that provider. A legacy alias can share the same underlying file and does not
establish independent retention.

The native planner takes a private profile with the original source and keeper
publication plans. Open **Native plan request field** on the catalogue candidate
and copy its JSON into the request as `reviewed_object`. In API output the
descriptor is `eviction_review.reviewed_object`; use `reviewed_object_json` for
a verbatim copy from JavaScript. Then supply the keeper's exact retained
snapshot and retention grant. The CLI refuses a mismatch against either
original plan or the snapshot before opening stores. The descriptor binds
content identity and length. Runtime placement and removal authority come from
the private profile and revalidated proofs. If the originals,
snapshot, grant, or writer authority cannot be established, leave the candidate
held. Never substitute a different snapshot or keeper for a saved plan.

## Explicit storage commands

Build the facade with `annex-publish`; discovery/help also work without it and
report availability. Existing `git asset storage` commands keep their original
service dispatch. Native commands never initialize providers or discover caches.

```powershell
git asset storage native describe
git asset storage native plan PROFILE.json REQUEST.json > SAVED_PLAN.json
git asset storage native status PROFILE.json SAVED_PLAN.json
git asset storage native apply PROFILE.json SAVED_PLAN.json
git asset storage native resume PROFILE.json SAVED_PLAN.json
git asset storage native cancel PROFILE.json SAVED_PLAN.json
```

Use absolute document paths. These commands require an explicitly qualified
private owner profile; they do not enable production use by themselves. Retain
the saved preview before apply. The full preview is accepted as `SAVED_PLAN`,
as is an equivalent bare plan. The CLI validates the canonical inner plan and
its exact ID; pretty-printing cannot retarget it. Missing/changed profiles or
plans have no latest fallback. Keep runtime profiles/plans/reports asset-side.

The profile schema is `git-asset.native-storage-profile.experimental-1`:

- `owner`: SDK `planning::Profile`, including sealed runtime/control bindings.
- `source`, `keeper`: each contains publication `profile`, original physical
  publication `plan`, `witness` root, `ledger` root and ledger `minimum` checkpoint.
- `source_metadata`, `keeper_metadata`: existing `root` and metadata `minimum`.
- `limits`: existing protection limits. Keeper point reads have fixed ceilings
  of65536 entries/reads,32MiB read bytes and depth64; metadata writers are absent.
- `allow_apply`, `allow_resume`, `allow_cancel`: separate invocation gates,
  default false. They are not epoch/legacy actor fences or revocation leases.

The request schema is `git-asset.native-storage-request.experimental-1`, with
`operation`, exact retained `snapshot` and independently retained metadata `grant`.
Profile/request documents are limited to64KiB, saved plans/reports to128KiB and
canonical inner plans to64KiB. Duplicate keys, extra fields, wrong preview IDs,
relative/traversing paths and non-regular document leaves refuse. Output is one
bounded JSON object; success exits0 and failure exits2. Mutation failures retain
`plan_id`, a retry classification and original-plan status guidance. If stdout
fails after an operation, inspect the original saved plan; do not assume rollback.
Status opens no metadata writer or keeper payload. Cancellation needs surviving
source bytes, not an online keeper. Historical resume uses existing lifecycle
requirements; unavailable required metadata/custody still holds reconciliation.

## Archive one existing cache object

`git asset storage archive describe` discovers the Windows command. Prepare an
absolute private JSON request with schema
`git-asset.cache-archive-request.experimental-1`, exact `object` (SHA-256 and
bytes), `source` and `archive` (`root` and `layout` for each), and
`minimum_free_bytes`. Use existing, distinct cache roots.

1. `git asset storage archive plan REQUEST` verifies the source and reports a
   plan digest. It creates no directories or files.
2. `git asset storage archive apply REQUEST EXPECTED_PLAN` rechecks the exact
   request, checks available capacity, and publishes without overwriting.
3. `git asset storage archive status REQUEST` verifies the exact archive copy
   without rereading the source. Keep the JSON report with the original request.
   Retry with the same request and plan after checking an uncertain outcome.

Verified readback proves the named archive copy at the time of the check.
The report does not establish durable retention, writer handoff or removal
authority. Keep the original working copies until those checks pass.

For a drive-level inventory, run
`git asset catalogue eviction-report ENROLLMENT PROJECT COMMIT CACHE_ROOT NEW_REPORT`.
It refreshes named metadata at the exact enrolled commit, counts all observed
objects in the selected cache, and writes its largest 100 rows to a new report.
The single-link allocation bucket is a verification queue. The report keeps
safe reclaim unknown while current protection and independently retained
custody remain unverified.

`git asset catalogue cache-inventory ENROLLMENT PROJECT CACHE_ROOT NEW_REPORT`
explicitly walks one existing live-probe cache on Windows. It reads file
metadata only, reports physical allocation once per hard-linked file and
stops at two million entries or fifteen minutes. Excluded/unreadable
entries and exhausted budgets mark the observation incomplete. The walk
does not check Git selection, retention, current jobs or reclamation
authorization and does not run on catalogue startup.

`git asset catalogue selection-usage ENROLLMENT PROJECT COMMIT CACHE_ROOT NEW_REPORT`
reports physical allocation in named, size-matching objects at the exact
enrolled snapshot. It separates directly selected objects, declared previous
revisions, other direct bindings and explicit retirement. Current selection
takes precedence when an exact object has multiple bindings. Missing nested
and job coverage does not imply unused bytes.

`git asset storage archive estimate MANIFEST` accepts a private, bounded JSON
document with schema `git-asset.cache-archive-estimate.experimental-1`,
`archive` (root and layout) and an `objects` array of exact SHA-256 and byte
length pairs. It measures free space on that volume and checks only named
object paths. Matching length does not prove integrity. Additional logical
bytes exclude uncertain paths and allocation overhead.

For one report per enrolled Windows cache root, run
`git asset catalogue drive-report ENROLLMENT PROJECT COMMIT CACHE_ROOT NEW_REPORT`.
Use the full commit ID and a new absolute output path. The JSON includes the
physical cache inventory by area, allocation by named selection class, and
the eviction review with candidate states and the largest observed items.
The three observations overlap and can see concurrent changes. Use the
inventory for allocation totals, including distinct allocation of files with multiple hard links. A file can have a link outside the scanned root. Use the eviction review to choose which
candidates to verify. Safe reclaim stays unknown until execution prerequisites
are established. The report does not open payloads or remove cache files.
