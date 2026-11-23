# Update a retained producer index

After authorized eviction and completed restore, use the separate Rust
`producer_index::update::restore` transaction described in
[controlled restoration](../specs/NATIVE-RESTORE.md#restore-specific-index-transaction).
Supply the exact old index Version/record and saved restore Plan. It holds fresh
restored bytes through one expected-baseline row replacement; old roots retain
old recipes. Ordinary edits still refuse producer replacement. Persist the wrapper
Plan before apply; recover its receipt before attempting any new native action.

Use Rust package `git-asset`, feature `annex-publish`, through
`annex::producer_index::update`. This experimental SDK composes existing metadata
and source transactions; it does not transfer or remove media.

1. Bind an existing fenced metadata-only namespace and MetadataStore through
   `update::Profile.scope`. Explicitly enroll its prefix as an owned Job source
   using the existing namespace API. Current, pending and predecessor placements
   must all be the scope's one metadata alias.
2. Read the exact current Version. For each of at most32 records, provide its
   `expected_record` canonical Object or None for an absent key. Obtain a record's
   metadata identity through `Record::metadata_object`; its media Object is a
   different value.
3. Call `prepare`. It returns a finished, read-back immutable index Plan, or None
   when an existing root is unchanged. An empty first request publishes an explicit
   empty index; absence of a head is not silently treated as that index. It does not change the source head. Preserve
   `Plan::to_json` durably with owned asset-side operation evidence before apply.
4. Call `apply` with current admission checks. Preserve the returned root, exact
   Version and receipt even when post_effect_hold is present. A competing writer
   fails the expected-revision transaction; it does not overwrite the newer root.
5. After interrupted delivery or restart, load that exact canonical Plan and call
   `status`/`apply`. Completed work returns its historical receipt without selecting
   its old root again. Permission can be disabled for this read-only recovery.

For an unfinished one-row plan whose head advanced, call `prepare_successor`
with the original plan. Preserve both plans before applying the successor.
If the row already matches, there is no edit. A changed row requires review.
Use `status` to recover a completed original.

Old versions remain predecessor obligations and old index roots remain readable.
The API has no removal, expiry cleanup or metadata-forgetting operation. Root
history and protection traversal remain subject to existing namespace bounds;
passing one bounded edit is not episode-scale/lifetime qualification.

## Register a newly published producer

For owner-managed admission, use `annex::producer_index::update::initial`.
After publication, pass the original producer Plan, its enrolled Publisher and
resources, and the expected index Version to `initial::prepare`. It reopens
the exact protected bytes and prepares an insert for an absent row. Save both
original plans before `initial::apply`. Apply checks their binding and recovers
a completed receipt first. Otherwise it reopens the publication and holds its
verified bytes through the index update. A competing advance requires a fresh
baseline after checking the original receipt. Keep the ingest claim active
until a separate retained custody handoff has completed.

## Connect archived history to its original producer

Use `annex::history::find` or `load`, then `Loaded::producer_record` with
the exact archive member and original record. See the
[archive binding contract](../specs/NATIVE-PRODUCER-ROOTS.md).
Older examples remain in the [prior guide](https://github.com/giodl73-repo/git-asset/blob/2a19956/docs/NATIVE-PRODUCER-ROOTS.md).

## Let a worker follow an enrolled index

Enroll `policy.producer_heads.source` and `.destination`, each containing:

| Field | Binding |
| --- | --- |
| root, minimum | Existing metadata-only namespace path and checkpoint floor. |
| profile.scope | Exact namespace/provider/owner/source prefix and metadata alias from the publisher profile. |
| profile.limits | Existing finite metadata/protection limits. |

The source prefix must already be an owned Job with a current index. No namespace
is created by a receiver request. The read-side profile can have allow_store false;
keep the writer's authorization separate. Bind the actual source/destination
metadata providers, not a similarly named drive/path. Missing head/provider refuses
without using the previous static root or another source.

After this deliberate trust enrollment, the owner can publish another index with
the expected-baseline SDK without changing the worker policy for every edit. A
copy/eviction pass freezes both index generations until it returns, reports both
exact pins and binds its cursor to them. A concurrent edit of either named source
is Busy and can retry the same prepared update after the reader ends. Other source
slots remain available. Restart a pass when an index advancement makes its cursor
stale; it never silently switches roots mid-page.

This changes producer metadata lookup, not which asset version a Git commit
selects. Saved native original plans/receipts remain authoritative after a head
advances; a new registry Version is not permission to repeat uncertain native work.
Check compiled support with git-asset-trigger describe. Runtime root creation,
producer registration/archival orchestration, backup repair, controlled
reacquisition, full HTTP and project acceptance still need their own qualification.

## Record the recipe that actually completed

Use `Record::with_completed_publication` with the original witness and
ledger, then save and apply an expected-row update. The worker freshly
opens the unique completed publication. Missing or conflicting evidence
holds. Archive membership preserves the recorded recipe.
