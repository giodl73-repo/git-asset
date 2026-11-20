# Explicit metadata capacity transitions

Issue64; input 0bb2b20ee7a00c4e92a1d84779024cd17542a385.
WP-001/003/005/008/009/014/015; REQ-001/003/005/007/009;
SPEC-002/003/005/006/008/009/013; IF-002/003/005/008/012;
PKG-002/003/004/008. RUST/PROVIDER/INTEGRATION profiles. Entry: owned existing
provider, exact checkpoint/configuration. Exit: explicit bounded capacity change
with preserved history, writer fencing and recoverable original operation.
Synthetic only; no cache setup, payload census, profile mutation or new engine.
Actor16/path128/node65536 bounds stay unchanged.

## SDK contract

Package git-asset-store, git_asset_store::metadata::capacity. The CLI remains
provider-copy-only. plan(root,Request{operation,expected,expected_limits,limits})
is read-only; Preview headroom is observed, never reserved. apply(root,&plan)
executes or recovers the same operation. status(root,&plan) never repairs and
returns NotApplied/Pending/Applied plus original receipt where committed.

Limits stay identity-bound. Even contraction derives a fresh instance. The plan
and counter identity commit atomically in existing SQLite; no node changes. A
second nonblocking writer fence publishes config without obsolete replay over a
later transition. SQL committed/config old means Pending; crash temps survive.
No automatic growth, retry, cleanup or latest fallback.

Cached writers fail their existing identity check; immutable readers may finish.
Old runtimes/profiles cannot reopen the changed provider. REEL must pin the new
SDK and CAS/adopt receipt.after and related enrollment, preserving original
handles/receipt. Consumer coordination/profile CAS remains REEL-owned.
rebind(root,&historical,&current,max_hops) explicitly maps old minima through
1..1024 bounded lineage steps; exact tree/object roots and selections stay intact.

Contraction refuses contents/allocated pages above its ceiling; no vacuum or
history deletion. Supported Limits/free-space guards recheck on apply. Indexed
history stays asset-side; plans are bounded8192B with no paths.

Errors: busy retries the same operation; uncertain requires status/original-plan
recovery. needs_recovery requires its original pending plan or explicit SQL
recovery. stale requires admitted replanning; conflict forbids changed-plan reuse.
foreign/corrupt/rollback hold for investigation; admission_denied needs explicit
resource revision; unavailable implies no success. Never guess a new operation
from a lost response. Existing opens remain strict.

## Qualification

L0 fmt/diff/pins; L1 locked metadata/capacity tests and strict Clippy; L2 process
exits and Windows/NTFS qualification. Build the [legacy fixture](../tools/fixtures/metadata_capacity_legacy.rs)
against input core/store with serde_json1.0.145; pin source/lock/exe and set
GIT_ASSET_METADATA_LEGACY_EXE. Run the exact ignored near-production-capacity test.
Evidence separates implementation/V/V/trace/producer review. Hosted, hardware
power-loss and independent/human production acceptance stay separate.
