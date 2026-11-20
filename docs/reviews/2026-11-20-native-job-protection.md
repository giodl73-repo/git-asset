# Native job protection producer review

Input 6eaf75904868a86201eeb55fc61e10b414fa7759; feat/native-job-protection.
EVID-121. [Contract](../../specs/NATIVE-JOB-PROTECTION.md) names controls,
scope, entry/exit, bounds and L0/L1/L2. Maintainers own this slice. Producer
checklists do not constitute independent review or human approval.

## Implementation

Add `job::protection::freeze` and `verify` on the existing optional assets Rust
surface. An admitted native JobContext becomes flat byte bindings accepted by
the public guarded Namespace, rather than exposing an opaque job root as if it
were its complete flat resource list. Preserve the original job encoding and
selection, all existing APIs and dependency pins. The domain-separated projection
header pins that exact job Selection; all declared Objects and the header are
bound explicitly. Header/tree IO share the caller's finite TreeLimits.

There is no new persistent engine, timer, authored resource inventory, cache
policy, CLI, native annex writer or REEL dependency. Existing Namespace
Enroll/select/status/Withdraw and their scoped fences own lifecycle changes.
Returned native bytes still require separate publication/readback/retention and
the annex pool's actual participating-pruner boundary.

## Verification

Three focused native fixtures pass: complete declared role membership and
deterministic identity; substitution/missing/corruption and aggregate IO/count
budgets; actual finished/reopened metadata and guarded journal lifecycle with
the synthetic Git checkout unavailable. The lifecycle checks independent peer
advancement, missing-closure refusal preserving old selection, identical replay
without extra commands, stale update refusal, predecessor retention on explicit
cancellation, continued shared-input demand and foreign-owner refusal.

The first compile found an invalid test assumption that Metadata Limits had a
Default implementation. The fixture now uses explicit supported bounds; all
three pass in focused-v2. The locked all-feature workspace suite passes, including
the existing 500,000-binding persistence regression and native history/publication
cases. Strict all-feature/all-target Clippy and locked standalone/assets-only
git-asset checks pass. Final fmt, structure/hash pins and whitespace checks pass
separately from runtime verification. No dependency pins changed.

## Intended-use validation

The fixture demonstrates durable metadata obligations with graceful provider and
journal close/reopen, without consulting a current/latest checkout. It does not
exercise a managed producer, abrupt process death at this new integration seam,
hardware faults, remote custody or native annex pruning. Those remain distinct
acceptance gates; underlying journal recovery fixtures do not prove full REEL
workflow acceptance. The projection APIs perform no payload scanning, hydration,
publication, rendering or removal. Fixtures use only newly created synthetic
inputs. No timeout advances state; cancellation preserves predecessor demand.

## Trace

WP-004/008/009/010/011; REQ-001/003/004/006/007/008/010 ->
SPEC-002/003/004/005/007/008/010/011, IF-002/004/005/008/010/012,
PKG-001/003/004/006 -> NATIVE-JOB-PROTECTION.md -> projection/readback and
three native fixtures -> EVID-121. Full IF-004/provider/custody acceptance stays
open; this scope resolves the opaque-job-to-flat-namespace seam only.

## Producer role checklists

- Identity: exact source job header and every binding must match. Renaming or
  dropping a resource cannot masquerade as the old job. No blocking finding.
- Placement: use the namespace's admitted policy; no hard-coded drives, maturity
  labels or expiry. Missing or conflicting closure refuses admission.
- Custody: obligations are desired state, not copies or removal permission.
  Annex pool/pruner and retained publication qualification remain high-priority
  open integration work owned by maintainers.
- Concurrency/recovery: stale revisions and foreign owners fail; historical
  replay is explicitly distinguished from current demand after cancellation.
  Graceful reopen is tested; abrupt integrated crash boundaries remain open.
- Privacy/provider: construction touches only caller-supplied metadata, with
  finite total IO. No private production assets or installation paths are added.
- Git/adoption: exact historical job roots and existing schema/history remain
  intact; registration does not become creative selection, approval or delivery.

Independent review, hosted qualification and human intended-use acceptance remain
open. This record grants no authority to deploy or change live caches.
