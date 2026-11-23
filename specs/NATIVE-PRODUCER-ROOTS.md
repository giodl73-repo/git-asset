# Retained producer-root updates

Ordinary updates below preserve producer recipes. Completed authorized
restoration has a separate [typed successor transaction](NATIVE-RESTORE.md#restore-specific-index-transaction)
with fresh native readback and exact old-row validation; it does not relax them.

Input5c4228a0e96aab41cbc225599efebdada5ad1a21; P3 WP-004/005/006/007/009;
REQ-001/002/004/006/007/008/010; SPEC-002/003/004/005/007/008/010/011;
IF-002/004/005/008/010/012; PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION.
Allowed: exact canonical producer records, existing Tree edits, metadata finish/
readback and atomic namespace selection. No native bytes, discovery, removal,
new journal/cache, dynamic trust enrollment or production installation.
Entry: existing fenced metadata-only namespace and explicitly enrolled Job source,
admitted MetadataStore identity, exact expected current Version and at most32
declared record upserts (including an explicit empty initial root). Exit: retained immutable root/receipt, no-op, or explicit
refusal without changing the source. L0 pins/links/roles/source/fmt; L1 locked
canonicality/budget/permission/conflict/race/recovery fixtures and strict Clippy;
L2 synthetic archive/index/native retained member reopen. Independent acceptance
remains separate.

`annex::producer_index::update::Profile` reuses `pending::Scope` for provider,
namespace, owner and metadata-only Job placement checks. The scope prefix names
the already enrolled root source. No enrollment or namespace creation occurs.
`allow_store` is required for preparation and new selection; false permits exact
original status/recovery. The profile's ordinary protection limits remain finite.
Location identity excludes permissions/budgets, so disabling writes does not erase
the identity of previously completed updates.

`prepare` requires the exact current Version, and captures its source revision.
None requires an enrolled source with no selection. Pending, wrong owner/class or
changed baseline refuses. Each Change requires an absent key or the exact previous
canonical record Object. Duplicate keys refuse; replacing a row cannot change its
original native publication Plan. Existing control-retention metadata is preserved
by archive rebinding. No record deletion operation is exposed.

Immutable tree editing touches only declared upserts; unmodified rows and old
roots remain unchanged. Metadata record writes, tree reads/writes, finish and
changed-row readback precede return. A shared meter bounds cumulative count/bytes
and elapsed time, including record reads and edits. Any failure can leave orphan
immutable metadata but cannot select a new root. All unchanged records return
None after finishing provider state and do not create another source Version.
This is incremental metadata editing, not a claim that source protection projects
an unbounded index/history incrementally: namespace selection retains its existing
complete closure, association, predecessor and lifetime limits.

The strict canonical Plan is at most64KiB and contains location binding, expected
Version/revision, new root and at most32 ordered media/record Objects. The caller
must durably preserve this exact Plan in owned asset-side evidence BEFORE apply.
Returning a prepared Plan is not durable retention or a publication receipt.
Plan selection IDs/context are deterministic. There is no mutable per-object
registry or second operation log. Descriptor/plan blobs and expanding root history
stay asset-side; small authored configuration remains in Git.

`apply` first checks original namespace selection status. Completed originals
return their receipt without invoking admission, rewriting metadata or selecting
an older root. An unexecuted original needs current permission, exact baseline/
revision and fresh admission. Apply verifies each declared canonical row, preserves
existing original recipes, then reconstructs the declared edits using read-only
verification of already finished tree nodes. A parsed Plan cannot hide extra rows,
removals or unrelated edits in its new root. The existing atomic Select transaction
fences a concurrent writer immediately before adoption. No global lock is added.

After successful selection the post-effect callback may refuse, but the report
keeps its receipt and exact Version plus post_effect_hold. Current selection is a
separate optional observation; unknown is not false or whole-catalogue completion.
`status` is read-only; a lost reply must use the original Plan, not a newly prepared
replacement. Missing metadata/provider failures require original provider recovery;
corrupt metadata requires investigation; stale/conflicting state requires a fresh
explicit baseline after original status is known. A denied/busy callback before
selection can retry the original after admission is restored. No timeout deletes
metadata or bytes.

`history::Loaded::producer_record` resolves one exact archived path, checks media
Object equality and copies the original Record while replacing only its retained
Snapshot. It does not invent a new publication recipe, hydrate a sibling or prove
native media custody. `Record::metadata_object` identifies its canonical metadata
blob separately from its media Object. Original/native identities stay unchanged.

Root claims protect index definitions on the metadata alias. Retained member
references do not acquire/renew media claims or assert actual bytes. Native
open_retained and eviction keep their original fresh integrity, control, roster,
generation and independent custody checks. Reports explicitly return unknown media
custody and no physical eviction authority.

The receiver still consumes explicitly signed immutable producer roots. This SDK
does not silently adopt a mutable registry head or update a trusted policy. Root
reenrollment/worker integration, missing-backup repair, full transport/account/TLS,
larger-catalogue lifetime and owner cutover acceptance remain separate obligations.

## Explicitly enrolled head consumers

Refinement inputffccf8cb206077908a3d667863acb5737189af65; the same P3 controls
and no-native-effect authority boundary apply. Optional signed
`github::native::Policy.producer_heads` enrolls source/destination Head locators:
fixed namespace root/minimum plus root Profile. It is mutually exclusive with
`producers` immutable roots. Absent optional fields preserve older policy bytes.
Each minimum must identify its profile namespace; head namespaces must be separate
from both native pools, and the pair cannot name the same namespace/source slot.
Pure policy identity validation opens nothing. A new enrollment is explicit;
there is no fallback from an invalid head to a static root or another registry.

`update::Profile::hold_current` validates the real namespace/provider, fenced
capability, owner/Job class and metadata-only placement. It refuses pending or
missing current selection and holds its exact source revision/Version through
the existing source-use gate. Root read permission does not require allow_store;
the worker acquires no index-write or metadata-deletion authority. Dropping the
handle releases only live source stability, never durable current/history claims.
The source minimum is an admitted checkpoint floor, not an inventory certificate.

Copy and registered eviction passes resolve both named heads after authenticated
admission, using the actual source/destination MetadataReaders. They retain both
generation guards through the pass, including native actions and post-effect
checks. Index advancement on those exact slots is Busy while a reader participates;
unrelated namespace/source slots remain usable. There is no cache-wide lock.
Failed resolution drops any earlier live guard and returns a structured refusal.
Required metadata still goes through the original aggregate bounded index lookup.
Two source lookups and their gates retain the existing namespace/provider bounds;
no runtime limit is raised or payload census introduced.

Dynamic reports include producer_heads pins: namespace, source key, source revision
and exact Version for each index. No physical paths or credentials are returned.
Cursor basis includes both pins in addition to its existing policy, immutable Git
commit/Main source and requested-object/page basis. Advancing either index requires
a new pass and rejects an old cursor even when the media hash or root content is
unchanged. Missing/current/malformed metadata remains explicit and never means
latest asset selection. Project asset identity still comes from the exact admitted
Git commit; only the explicitly enrolled producer metadata heads are mutable.

The private receiver accepts this signed option on its existing copy/eviction
phases and strict commit/cursor request. Events cannot enroll heads or write rows.
`git-asset-trigger describe` advertises native_producer_head_resolution when built
with annex-publish. Static-root reports/bases retain their previous behavior.
History scheduling does not need producer lookup and remains independently usable.

This consumes already published roots; it does not automatically create a root,
register a new producer, repair backups or replace an uncertain native plan. The
original native completion/control/roster/independent-custody checks remain
mandatory. Worker publication/archival orchestration, controlled reacquisition,
full HTTP/account/TLS, scale/lifetime and owner acceptance remain separate.

## Preserve a completed copy or backup recipe

Refinement inputcb798cbc9993cbb8b6649476e6fe7c9ae66c7154; same P3 WP/REQ/SPEC/IF/
PKG controls and L0/L1/L2 profiles. Optional Record.publication names at most one
additional immutable publication recipe. It defaults absent and is omitted when
None, preserving old canonical record bytes. The recipe must equal producer after
removing only copy context: candidate, provenance, request, selection, namespace,
control, witness/capacity admission, configuration and budget cannot change.
The original producer remains mandatory and immutable. Record cap stays32KiB;
large two-recipe records refuse rather than raising bounds.

Record::with_completed_publication checks the exact recipe's original witness and
ledger status. An unfinished intent is Uncertain; missing completion/unsettled
accounting is NeedsRecovery. It never starts/resumes native work or checks current
payload presence. Its output declares historical completion only, not byte custody.
Once an additional recipe is recorded, ordinary expected-baseline index edits and
parsed-plan apply cannot clear or replace it, including with another copy context.
Original native ambiguity must not be hidden by changing metadata. Controlled
reacquisition/stale-plan replacement requires separate qualified admission.

Record::publications enumerates the original and optional recorded recipe (at most
two). Copy/backup/eviction checks those plus the one currently derived prospective
recipe (at most three), deduplicating exact plans. A known unfinished intent in any
candidate holds; one unique completed/settled recipe must be freshly verified.
Conflicting/ambiguous completions are not resolved by last writer or presence alone.
The declaration is never sufficient to import or remove bytes. Saved eviction can
be found by its recorded exact publication before keeper payload opens; original
saved action/status checks remain authoritative.

This preserves a completed working copy when a retained membership reference later
changes, and preserves a completed reverse backup after old ingest is released.
Archival rebinding copies the optional publication unchanged alongside the original
producer/control grant. Historical readers can pass that exact recorded recipe
and retained Snapshot to open_retained; it verifies real claims and bytes again.
No working checkout or source-pool fallback is necessary for that named read.

Owner sequence: complete original native work; inspect exact completion/settlement;
construct the publication-bearing Record; prepare expected-old-record root update;
durably preserve its exact canonical Plan before apply; select through existing
namespace admission and preserve the returned receipt. Archive/ingest release
remains separately admitted. A metadata failure after native success keeps the
native original/receipt; it never authorizes another transfer. No new queue,
operation journal, RPC producer editor or automatic cache adoption is introduced.

## Authenticated completed-publication registration

Input7bd82ba90b1d9ba7c66a2ace5da8027ebfb6eb8e; P3 WP-004/005/006/007/009;
REQ-001/002/004/006/007/008/010; SPEC-002/003/004/005/007/008/010/011;
IF-002/004/005/008/010/012; PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION.
Allowed: existing completed native copy/backup recipes into their current declared
index records, immutable pending root plans and fixed receiver registration phase.
No media import/removal, census, new engine or production enrollment. Entry:
signed current source, explicitly enabled registration, enrolled index heads,
existing owned pending namespace and actual completion witness/ledger. Exit:
read-only named eligibility, held original, or saved exact index plan before CAS.
L0 reviewed pins/links/budget; L1 locked failures/strict checks; L2 synthetic native
backup -> registration -> archive/read, retry, stale/denied and historical receipt.

Only the completion field may be added to an existing original record; native
producer, retained member and control grant remain unchanged. Witness completion
and settled accounting are required, never inferred from presence or a new plan.
This metadata registration proves no current byte availability or custody.
Ordinary root apply still reconstructs exactly the declared upsert. Original saved
plans precede any index write and remain authoritative after uncertain delivery.
At most one eligible object changes per pass; an index advancement needs a fresh
basis. Current root/head, exact source, explicit permission and quotas are checked.
Preview saves nothing. Historical receipts never reselect old roots. Full stage
orchestration, missing registrations/repair, HTTP/scale/owner acceptance remain OPEN.

Producer review: identity binds the sole index upsert to the exact stored Record
and original working/retained producers; decoded phase must match its envelope.
Recovery retains Saved/receipt across post-effect refusal and never reconstructs
another native effect. Placement/custody keeps recorded completion separate from
availability and fresh survivor proof. Privacy uses only fixed enrolled locations.
Adoption requires default-denied registration and explicit index write admission;
source/profile/hash domains stay readable, with no new dependency or state engine.
Six producer checklists do not constitute independent or human acceptance.

## Archive membership indexing before working release

Input6503813b12114758645abf13dcb98875382e1586; existing P3
WP-004/005/006/007/009; REQ-001/002/004/006/007/008/010;
SPEC-002/003/004/005/007/008/010/011; IF-002/004/005/008/010/012;
PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION. Allowed: exact archived
member enumeration, retained-row upserts with saved originals, and opt-in trusted
history/ordered-driver integration. No native import/deletion, new engine,
control-grant invention or production enrollment. Entry: published retained tree,
enrolled producer heads, existing pending namespace and explicit archive-index
permission. Exit: all distinct archived Objects indexed before working release,
or a hold preserving working/history/index originals. L0 pins/links/budget;
L1 malformed bindings, refusals, retry, cumulative lookup and strict checks;
L2 synthetic committed predecessors -> archive -> index -> working release/read.

Enumerate the exact retained metadata tree once; no payload census. Deduplicate
Objects deterministically. Each upsert preserves original native producer,
completed recipe and control grant, changing only retained membership. Save the
exact root plan before CAS. Slots bind archived source plus original working
producer/Object, so another archive cannot replace an uncertain original. Old
versions remain retained. Completed receipts never reselect old roots. Missing
registration/control custody remains unknown; metadata indexing grants no native
availability/eviction authority. Existing archive-root and per-row index limits,
cumulative lookup limits and the outer deadline can hold partial work. Restart
reuses current exact rows/saved originals; it does not trust a client prefix.

Producer review (same author, not independent approval): identity binds exact
archive source, member/Object and sole upsert while preserving original wire
identities. Recovery saves before CAS and retains partial handles; completed
historical receipts never reselect. Placement holds missing/unknown producers;
custody/control are not fabricated by metadata. Privacy uses fixed enrolled
namespaces and synthetic fixtures. Adoption keeps the flag default-denied/omitted
and preserves previous policy hashes; P2-P6 acceptance gates remain OPEN.

Release fence refinement, input54a74e49d38791df3983b4baf490214401855493.
Same WP/REQ/SPEC/IF/PKG and RUST/PROVIDER/INTEGRATION controls as above.
Enter with enrolled retained head and exact archive. Exit with a contiguous
sequence of verified index versions and its final generation held through
working release, or a stale hold. L0 pins/diff, L1 focused worker/strict checks,
L2 synthetic archive and release. Scope is metadata fencing only.

Initial owner registration, input687d5e89077a7b30c9b20097a0f1e87260825144.
Existing WP-004/005/006/007/009, REQ-001/002/004/006/007/008/010,
SPEC-002/003/004/005/007/008/010/011, IF-002/004/005/008/010/012,
PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION. Entry is an explicitly
admitted publisher and owned producer index. Prepare only an absent-row insert
for an exact reopened producer. Save the plan before apply. Apply recovers past
receipts first, otherwise holds original protected bytes through index CAS.
Exit is the original receipt or an explicit hold. L0 pins/diff, L1 locked and
strict checks, L2 synthetic guarded publication and registration. No selection
of creative assets, native import, deletion or control-retention grant.

Different-byte registration fixture, input bff913f89dbaf409300f5336d23b99a4c9bb2597.
Same controls and synthetic scope as initial owner registration. Verify a later
protected publication adds its exact Object to the same index while retaining
the earlier index root and producer row. L0 pins/diff, L1 focused/strict, L2 synthetic
publication, replay and exact readback. Merge-driven placement is separate.

Historical byte reopen fixture, input8137cb6. Same archive-index controls,
RUST/PROVIDER/INTEGRATION and synthetic scope. After indexed working release,
reopen the exact archived member with its retained publisher and verify content.
L0 pins/diff, L1 strict checks, L2 ordinary and bare native retained reads.

Distinct Git selection fixture, input b0418b0. Existing archive-index controls
apply. With distinct later Git Objects, recover the registered predecessor from
the retained pool while the working repository is offline. Missing producers
still hold. L0 pins/diff, L1 focused/strict, L2 synthetic native historical read.

Golden publication fixture, input60b3a5f. Existing owner/archival controls and
synthetic scope apply. Publish distinct new bytes in the working pool, register
against its enrolled index, select that Object through Git, and verify both the
working golden bytes and the separate historical retained read. L0/L1/L2 as above.

Golden backup fixture, input48753f1. Same owner, archive and bounded receiver
controls. Enter with registered working publication and an admitted retained
recipe. Exit after authenticated backup and exact retained readback. L0 pins,
L1 worker/strict checks, L2 synthetic native backup and historical reopen.

Golden handoff fixture, input7e6bbcd. Same controls and synthetic scope.
Qualify saved custody handoff after authenticated backup, and exact retry
with original receipt. L0 pins/diff, L1 locked/strict, L2 native byte reopen.
