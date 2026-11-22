# Retained producer-root updates

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
