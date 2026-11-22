# Update a retained producer index

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

Old versions remain predecessor obligations and old index roots remain readable.
The API has no removal, expiry cleanup or metadata-forgetting operation. Root
history and protection traversal remain subject to existing namespace bounds;
passing one bounded edit is not episode-scale/lifetime qualification.

## Connect archived history to its original producer

Resolve an existing archive with `annex::history::find`/`load`, then call
`Loaded::producer_record(profile, logical_path, original_record, nodes)`.
This produces a Record containing the exact archived member Snapshot while
preserving the original publication Plan and control-retention grant. Wrong
object/path pairs refuse; missing members never fall back to latest/current.
Use that record as an expected-old-record update above.

Reopen the selected index's exact Record through `Index::get`, then use its
original `producer` and `retained` with the existing `Publisher::open_retained`.
That API verifies the real retained claim, original completion, current native
configuration/protection and actual bytes. The index can outlive an original
ingest claim and working checkout; availability still requires those byte-level
checks to pass. Registration does not select a creative version or establish
independent custody.

The native fixture advances an index while its archive record guard is live,
retains the old index, retires the original keeper ingress, releases the old
working obligation and reopens indexed historical bytes with the working repo
offline. Unavailable sibling media remains missing.

The current receiver pins producer roots in its signed policy. Use explicit
trusted reenrollment to consume another root; this SDK does not rewrite policy
or accept root changes from events. Automatic worker integration, missing-backup
repair, full HTTP/account/TLS and production acceptance remain open. See the
[contract](../specs/NATIVE-PRODUCER-ROOTS.md) and
[history guide](HISTORICAL-ROOTS.md).
