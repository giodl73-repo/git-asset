# Bounded native producer lookup

Input0b73387e26e3b747611c824d387bb58c482a0430; P3 WP-004/005/006/007/009;
REQ-001/002/004/006/007/008/010; SPEC-002/003/004/005/007/008/010/011;
IF-002/004/005/008/010/012; PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION.
Allowed: existing immutable Tree/NodeStore producer records, bounded exact lookup
and root-bound paging; synthetic read/budget/corruption fixtures. No new cache,
journal, physical discovery, mutation authority or production enrollment.
Entry: independently admitted immutable index root and metadata provider. Exit:
resolve only the exact requested content identity/length and original producer
plan, or explicit missing/corrupt/stale/budget refusal; no current/latest fallback.
L0 source/format/pins/roles/budgets; L1 locked tree/index tests and Clippy;
L2 synthetic metadata roots, no production data. Independent review separate.

Records live asset-side in the existing metadata provider. A flat Tree maps the
canonical content SHA256 to a bounded canonical record, containing exact Object,
original native publication Plan and optional retained Snapshot. Record bytes are
hash/length verified; key/object/plan/snapshot identities must agree. This is a
declared original producer, never proof of current physical incarnation, presence,
quiescence, custody or authority. Existing native APIs revalidate those separately.
Old roots and original plans remain immutable; callers explicitly publish/select
new roots through existing metadata/source mechanisms. No mutable per-object file
or new operation log is introduced. Missing registration is unknown, not absent
payload or permission to prune. An uncertain operation must retain its original
plan; a new index version does not authorize repeating unknown native effects.
Each index root describes one explicitly enrolled location/producer stream;
the consumer binds it to its fixed policy. It is not a global SHA-to-current-pool
directory. Record::store returns a content-addressed record and logical key;
existing Tree build/edit and source selection publish a new root explicitly.
An old root remains readable after publishing a replacement. This API does not
select a root, retain its closure or authorize physical effects on its own.

Tree::flat_page traverses existing ordered entry-map nodes under shared aggregate
node/byte/depth limits. It returns only files directly in the root directory;
encountered directories refuse. Limit1..4096 and one lookahead bound output memory.
Cursor continuation must retain the exact Tree root; no automatic head lookup.
The supplied after key must exist. Route ordering/priority/count checks remain;
missing required nodes refuse. Skipped prefix subtrees are not verified by a
suffix request, and an arbitrary cursor is not a complete-coverage certificate.
An arbitrary resumed cursor cannot prove that an unobserved prefix was checked.

Index pages share a cumulative budget across tree and record reads. Bounds hold
even with huge metadata inventories; no payload is read or hydrated. Index
availability or a full metadata pass is not whole-placement convergence. Receiver,
batch planning/persistence and project acceptance remain OPEN.

SDK: annex::producer_index::{Record,Index,Cursor,Page}, annex-publish feature;
core::tree::{Tree::flat_page,FlatPage}. Record cap32KiB, page1..4096; existing
TreeLimits inventory/depth/read-count/read-byte bounds apply, with row reads
charged to the same cumulative metrics as traversal. The NodeStore provider
must honor bounded metadata retrieval; this API never opens asset payloads.
Missing registration returns None (unknown); missing referenced metadata is
Missing; malformed/canonical/hash/length/key mismatch is Integrity; changed
cursor root is Stale; an absent after-key is Stale; resource refusal is Budget;
provider failure is NeedsRecovery. Reads may be retried against the same root
after fixing availability/admission; integrity failures require investigation.
No read retry mutates state or repeats a native operation.

Fixtures: core/tests/tree_pages.rs covers10,000-entry paging, aggregate node/byte
bounds, directories, missing/corrupt nodes and forged counts/order. Native
publication/tests/producer_index.rs uses synthetic plans/metadata without native
processes: exact size/hash/key, missing/noncanonical rows, stale roots and shared
tree-plus-record budgets. Native custody/executor tests remain prerequisites,
not re-proven by an index declaration.

[Producer-root updates](NATIVE-PRODUCER-ROOTS.md) add explicit metadata finish/
readback, expected-baseline source selection and original status recovery over
these immutable records. Root publication is not native custody or policy adoption.
