# Durable original native actions

Input 1c0f870f8efa8a538fc0948139f62f31ea3e625b; P3 WP-004/005/006/007/009;
REQ-001/002/004/006/007/008/010; SPEC-002/003/004/005/007/008/010/011;
IF-002/004/005/008/010/012; PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION.
Allowed: immutable original handoff/eviction plans and producer recipes in the
existing metadata provider, retained through existing scoped source claims;
authenticated SDK execution from those saved originals. No new journal, cache,
daemon, media census, automatic forgetting or production enrollment. Entry:
existing fenced metadata namespace/provider and admitted operation. Exit: save,
hash/length readback and source adoption before any effect; restart finds that
same original plan or an explicit hold. L0 pins/roles/source/format; L1 locked
failure/concurrency and strict Clippy; L2 synthetic native/process recovery.
Independent review, scheduling and owner acceptance remain separate.

The private pending scope binds namespace, metadata identity, owner, source-key
prefix and metadata-only alias. It must be separate from all media pool members.
Store permission defaults denied; readback/recovery remains possible without it.
Each action slot is derived from kind, exact Object and original working producer
ID, with a scoped source gate rather than a cache-wide lock. Existing slots always
return their stored original; a fresh preview cannot overwrite pending history.

Plan bodies and both original publication recipes are separate canonical blobs
(each at most64KiB) in the existing NodeStore. A small strict envelope pins their
Objects, captured policy and commit. A four-entry Tree references all those blobs
and the envelope. Metadata finish/readback precedes atomic source adoption.
The job-class source retains current/pending/predecessor metadata on its admitted
alias with no timeout. Native bytes, custody and removal authority are not inferred
from this metadata. Namespace/provider quotas, source count and cumulative tree/
record read bounds may safely refuse; no hidden expansion or retention waiver.

The saved handle pins namespace/source, exact Version and envelope Object. Loading
requires that exact retained version, a live source-generation guard and canonical
hash/length verification; no current/latest replacement. A missing/corrupt/stale
record holds. Source CAS conflicts adopt no alternative. Orphan immutable nodes
before source adoption grant no operation and are not mistaken for pending work.
No action executes before its saved handle is durably discoverable.

Durable worker entries authenticate first, find an existing slot before freezing
a new handoff, and invoke the existing qualified worker with saved plan/recipes.
Fresh authenticated requests may recover original work after Git advances; each
new command/effect still checks current admitted Main, native protection, retained
custody and actual configured profiles. Captured policy/commit are provenance,
never authority to replay an old event. Post-save/effect errors preserve the saved
handle and explicit failure. Native uncertain intent never authorizes another plan.

Ordinary save does not retire or replace saved slots. The explicit restore-index
successor below is separate from ordinary retry. Status/resume/
cancel use the original backend APIs and saved bodies; metadata/byte forgetting
needs a separate explicit contract. Registry-wide paging and other stale-plan replacement,
large-catalogue continuation and production remain
OPEN. Growing records stay asset-side, not in Git or mutable per-object files.

Registered receiver eviction is refined in
[NATIVE-EVICTION-PASS](NATIVE-EVICTION-PASS.md). Its durable worker inspects original
native status before apply: completion resumes accounting only, uncertain native
intent refuses another effect. Saved originals and current admission remain
mandatory; stale-plan replacement is not inferred from this recovery surface.

## Explicit restore-index successor

The restore contract's [successor rules](NATIVE-RESTORE.md#explicit-stale-index-successor)
add only `save_restore_index_successor`. It accepts a sealed replan proposal and
exact Saved baseline. The same source CAS retains the prior version as a
predecessor; its original four blobs/schema/key stay readable. The successor body
changes only the index plan, preserving the original native restore/publications.
An identical saved successor is recoverable without store permission; a different
current proposal or parent mismatch refuses. Ordinary save never supersedes.
Original handles/receipts must remain recorded; there is no timeout or cleanup.

## Completed-publication index actions

RegisterWorking/RegisterRetained phases retain Registration bodies containing
side, one-record ordinary index Plan and exact completed Record, alongside the
original working/retained publisher Plans. Pure validation binds side/producer,
Object, canonical record and the sole upsert; it grants no native authority.
Existing phase encodings and four-blob records remain readable. Load verifies
that the decoded action phase equals its envelope. Ordinary save-original and
source-history rules apply. The authenticated worker separately checks actual
completion and permits only the completion-field change before index apply;
original receipts are inspected before fresh completion checks. There is no cleanup.
