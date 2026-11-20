# Public annex read adapter producer review

Input: public git-asset 21a70f5abc4340582cda3a075a3c5fe1cee13e40.
Owned lane: feat/native-annex-adapter. EVID-120.
[Contract](../../specs/ANNEX-READ-ADAPTER.md) names controls, scope, entry/exit
and L0/L1/L2. This is producer review, not independent or human approval.

## Implementation

Add an optional `git-asset/annex-read` Rust provider while retaining public git-asset's
existing schema/history APIs, default features, Layout variants and profile JSON.
The new explicit provider-file port shares the existing guarded-open machinery.
Annex reads require an exact configuration pin, repository UUID, format 10,
SHA256 key, trusted runtime and finite budgets. They hold verified payload and
namespace guards through consumer use, including after the Reader is dropped.

This is an in-process read boundary. No controller/index engine is imported.
There is no new CLI, scan, hydration, publication, deletion, persistent job lease
or retention enrollment. REEL should retain its current history surface and use
this Rust provider only for the supported byte-read operation. Long-running job
protection and retained-publication integration remain separate work against
public git-asset's current contracts.

## Verification

Three provider-file tests, thirteen existing byte-reader integration regressions
and seven adapter/parser/process failure tests pass. The eighth adapter test is
deliberately ignored by ordinary test runs and was explicitly run successfully
with enrolled native Git/annex executables. It creates fresh synthetic ordinary
and bare repositories, sets one opaque 22-byte object, reads exact bytes, checks
held deletion refusal, unchanged configuration and a missing key. Windows path
normalization was corrected after this real fixture exposed native spelling.

Negative cases cover configuration corruption before tool execution, wrong UUID,
unsupported format, missing versus ambiguous observation, corrupt payload,
path escape, request/live-reader/IO budgets, output overflow, timeout, descendant
termination and refusal inside an entered Tokio runtime. Strict workspace
all-feature/all-target Clippy passes. The full locked all-feature workspace suite
passes, including existing schema/history/native publication tests and the
500,000-binding persistence regression. The locked no-default-features git-asset
check passes. Final structure/pin/fmt/diff checks pass separately from runtime
verification. The explicit native fixture remains opt-in in ordinary test runs.
Lockfile audit preserves every existing registry package version and checksum;
29 registry entries are added for the optional process dependencies.

## Intended-use validation

The actual native fixture demonstrates exact local lookup through the original
byte guards, without media generation or existing cache access. Fake executables
exercise bounded failure modes and command vocabulary; they do not qualify all
native annex behavior. No remote, linked-worktree, Linux process containment,
external writer/crash/hardware or full REEL workflow acceptance is claimed.
Windows-only synchronous use requires a dedicated thread outside an entered
Tokio runtime. Configuration/runtime enrollment is trusted, not a sandbox.

## Trace

WP-004/005/007/011 and REQ-001/003/004/006/007/008/010 map through
SPEC-003/004/007/009/011, IF-004/005/009/010/012 and PKG-002/003/004/008
to the explicit provider port, annex reader, contained process and fixtures.
This evidence closes only the scoped local read candidate, not product-level
custody, retained publication or protection lifecycle requirements.

## Producer lenses

- Provider/privacy: construction is pure; exact locations remain provider-owned,
  debug output redacts ProviderPath, and reports/errors omit private paths.
  Subprocesses use explicit executables and sanitized, bounded environments.
- Identity/namespace: exact UUID/configuration and requested Object must match;
  no SHA256E guessing, alternate-store fallback or current/latest resolution.
- Concurrency/recovery: guard ownership follows the returned reader; mutation
  and deletion refusal are exercised. Process restart ends this guard, so callers
  must not treat it as a durable job. Timed-out reads delete no referenced bytes.
- Storage/custody: missing/unknown/corrupt remain distinct; local read success
  proves neither independent backup nor authority to remove another copy.
- Git/adoption: existing schema/history/default surfaces remain additive; native
  writes and creative selection remain outside this adapter.

No blocking finding remains within this read-only scope. Maintainers own the
separate durable lifecycle bridge and its restart/concurrent-owner/publication
fixtures. Independent review, hosted checks and human intended-use approval
remain open.
