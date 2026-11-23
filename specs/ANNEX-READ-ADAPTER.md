# Explicit annex read adapter

Input public git-asset 21a70f5abc4340582cda3a075a3c5fe1cee13e40. Controls
WP-004/005/007/011; REQ-001/003/004/006/007/008/010;
SPEC-003/004/007/009/011; IF-004/005/009/010/012; PKG-002/003/004/008.
PROVIDER/GIT/INTEGRATION profiles. Producer review is not independent approval.

Scope: additive provider-issued file reads and an optional read-only annex
adapter. Preserve all existing layouts, schema/history APIs and defaults.
Do not import the private archive's controller/index or replace public git-asset's
admission, placement, publication or retention engines.

Entry: public existing-byte guard contract and qualified prototype read semantics.
Exit: exact bytes through existing guards, bounded requests/IO/processes,
wrong UUID/config/path/corruption/missing/refusal fixtures and preserved public
history-feature compilation. L0 scope/source/fmt/structure/pins; L1 focused and
existing locked tests/strict Clippy; L2 explicitly enrolled synthetic temporary
annex repository only. No live caches, media generation, service or deployment.

## Provider file port

`ReadCache::provider` admits one provider's shared read/verification limits.
`open_provider_file(root, ProviderPath, Object)` opens one explicit relative
physical suffix with the original file, directory, hash, size and live-reader
guards. It never guesses a layout or falls back. Ordinary `open_verified` on
this provider-only instance refuses as unsupported. Provider calls use place
index zero; this is not a claim about global enrollment or coverage.

`Layout` and existing profile JSON remain unchanged, including exhaustive Rust
matches. Provider paths are separate from authored logical paths: `.git` and
`.tig` segments are allowed, but escapes, separators, aliases and platform
reserved names are rejected. These are private locations, not public reports.

## Annex boundary

An explicit profile admits an initialized ordinary/bare repository, its expected
UUID, an exact bounded repository-config Object, trusted Git/annex runtime paths,
sanitized environment and finite subprocess/read limits. Construction performs
no filesystem or subprocess IO. Configuration is independently verified and
held during UUID/location lookup; returned payload guards preserve exact bytes.
Configuration edits require explicit refreshed provider enrollment, without
changing the requested asset Object or project snapshot.

Only local UUID/version lookup and deterministic SHA256-key content-location
queries are allowed. This first profile supports repository format 10.
No init, scan/info census, fetch, get, copy, publication, drop, hooks,
network or arbitrary extra arguments. No SHA256E key guessing. Missing or corrupt
bytes never select another version or current/latest content. Ambiguous command
failure is an observation error, not proof of absence.

The contained process implementation initially supports Windows only; unsupported
platforms/contexts refuse before IO. Existing cooperative portable byte reads
remain available independently. Async callers use a dedicated thread outside an
entered Tokio runtime; even an entered blocking-pool handle is refused. Runtime
enrollment is trusted, not a sandbox. Linked-worktree repository layouts are not
supported; consumers may enroll a separate initialized ordinary or bare pool.

Publication, durable job/owner registration, slot lifecycle, shared protection
enrollment, custody and removal remain unsupported by this adapter. REEL keeps
its existing public git-asset history APIs; the native lifecycle bridge is subsequent
integration work and must honor public git-asset's current admission contracts.

The [git-annex plumbing contract](https://git-annex.branchable.com/git-annex-contentlocation/)
defines local content lookup without acquisition. Only a quiet exit 1 is treated
as missing; diagnostics or other failures remain observation errors. Native
Windows separators and current-directory components are normalized only at this
provider boundary, before root/prefix/key validation. Generic provider paths and
authored logical paths retain their strict parsers.

## Consumption and retry

Enable `git-asset/annex-read` and consume `annex::Reader` through Rust. Its `open`
keeps typed configuration versus payload failures; its existing `ByteSource`
implementation deliberately projects these into the older byte-port taxonomy.
There is no new CLI, daemon or enrollment discovery. `VerifiedRead` owns the
in-process file/namespace guard even after its Reader is dropped. Dropping that
handle or exiting the process ends protection: this is not a persisted lease.

`Invalid`, `Unsupported` and `WrongIdentity` require corrected admission.
`Configuration` and `Bytes` preserve the underlying missing, corrupt, denied,
unsupported, budget or IO distinction. `Missing` proves only this local lookup
did not locate the requested key; it says nothing about other stores. `Observation`
includes process failure/timeout and is not absence. `Budget` may mean a request,
verification, live-reader or subprocess-output limit; do not retry blindly.
Read failures have no publication outcome to reconcile. After correcting the
cause, retry the same Object within remaining budgets, or explicitly admit a
fresh Reader. Neither timeout nor retry deletes bytes, changes selection or
authorizes hydration. Command deadlines apply per command, with up to five
additional seconds for termination; an open performs at most three commands.

Synthetic coverage and separate verification/validation/review closure are in
the [producer record](../docs/reviews/2026-11-20-annex-read-adapter.md).

## Exact object observation

Input 3cdffe4, issue #78. Uses the provider controls above. An admitted reader
can observe the logical length of a named object under the repository and
provider path guards. The call consumes request admission. It reads repository
configuration and object metadata without reading object payload bytes.
Observation carries no custody, verified hash, allocation, or removal authority.
A subsequent verified read independently rechecks the repository and object.
