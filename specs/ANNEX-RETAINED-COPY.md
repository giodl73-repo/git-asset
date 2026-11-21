# Native retained copy (P1)

Input8ded690793b37b7132eec1c9e912e944f3b2e286. WP-003/004/007/008/009;
REQ-001/002/003/004/005/006/007/008/009; SPEC-002/003/004/005/007/008/009/013;
IF-002/003/004/005/006/008/010/012; PKG-002/003/004/008/009.
RUST/PROVIDER/GIT/INTEGRATION. Entry: exact managed source and destination profiles,
existing candidate/provenance, immutable metadata, retained namespace, capacity
job and witness. P1 composes existing managed Reader and Publisher; no second
payload engine, remote configuration, census, native removal or creative selection.
Native execution is Windows-only; fixtures are new external synthetic pools.

The copy plan binds source UUID/store, exact config, roster and repository
location, declared failure domains and destination publication plan. Its context
is included in the existing immutable accounted publication plan, preserving old
plan encodings when absent. Existing witness inspection checks original completion
identity before looking for a different object fence; conflicting reuse remains
conflict instead of ambiguous recovery, with guarded revalidation unchanged. Conflicting reuse of an operation refuses rather
than changing source/basis. Copying preserves candidate identity/object/provenance;
its destination registration is location retention, not creative selection.

Apply holds a fresh source byte/all-namespace guard through destination accounted
staging, native reinject, fresh readback and durable retention acknowledgement.
It reuses original publication registration/intent/completion/recovery minima.
Source bytes are never handed to a consuming native command. Pure constructors
perform no IO. Apply requires two admitted source verifications and bounded
source/destination command, byte, elapsed, scratch and metadata limits; reclaim
is never admitted capacity. Existing-root data is never initialized or repaired.

Status/reopen/resume address the exact destination operation. Explicit resume can
finish a verified owned stage or reopen completed bytes without source; an
incomplete stage needs apply with the original source. Missing acknowledged bytes
remain an incident. Timeout/cancel/receipt serialization never withdraw protection.
The returned destination guard proves retained local bytes under the admitted
cooperative boundary; it does not prove power-loss durability, independent backup
or eviction authority. Failure domains are explicitly admitted, never inferred
from letters or UUIDs. Equal domains count once; same-store copy refuses. P2 must
freshly qualify surviving custody and all removal fences, including authority
handoff; no P1 receipt is a deletion grant.

Exit L0: fmt/source/link/hash/budget/diff and six-role review. L1: pure identity/
context/conflicting-source/budget refusals plus locked optional/default and strict
Clippy. L2: two actual temporary ordinary/bare annex pools with32MiB bytes,
exact identity/provenance, source preservation and durable destination reopen;
interrupted staging/native completion, offline/corrupt source/destination, denied
capacity and original-operation retry. Verification, validation, trace/review and
implementation close separately; hosted and production acceptance are distinct.
Historical progress summaries retain immutable source/hash; the8 MiB cap stays fixed.

SDK: `git_asset::annex::replication::{Profile,Plan,Copier,Ready,Receipt}` under
`annex-publish`. Freeze source and destination publication plans first; source
must already reopen as retained, and candidates must match exactly. Destination
metadata must preserve the original provenance. This is location retention,
not another creative selection. Copier apply takes both resource sets; status,
resume and reopen take only destination resources. Persist the copy plan plus
original metadata/namespace/capacity minima. Ready exposes destination recovery
and optional source_recovery; source_verified=false after destination-only
recovery, so no two-copy coverage may be inferred. Declared domains are admitted
policy, not automatic hardware verification. No copy CLI is added in P1.

## EVID-145 producer checkpoint

Implementation: replication::Plan/Copier compose existing publication/managed
read engines. Optional copy context preserves absent-field legacy encodings;
completion inspection reports conflicting operation reuse before payload lookup.
Verification:10 focused entries pass (one child entry); three real child exits
recover staging/import/completion. Actual native32MiB ordinary/bare copies and
source-offline destination reopen pass; strict all-feature/all-target Clippy
passes. Full workspace/default/standalone and hosted gates remain pending.
Validation is synthetic copy/recovery only, never production custody/removal.

Six lenses: identity pins both plans/locations/provenance; placement bounds peak
without reclaim credit; custody retains source/destination guards and reports
DECLARED domains only; concurrency preserves original retry/incarnation scopes;
privacy uses fixed local plumbing/no private data; adoption preserves old APIs
and keeps pruning/legacy authority handoff at P2. Producer PASS_WITH_RISK for
this checkpoint, not independent or human approval. Remaining tests/gates above
stay OPEN. Trace: named controls -> replication/context/witness inspection ->
pure, fake protocol, native and child-exit fixtures. Broad verification/validation
and production acceptance close separately.
