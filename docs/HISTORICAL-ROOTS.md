# Preserve a historical tree before releasing its working obligation

The experimental Rust SDK can archive one exact branch/draft predecessor without
turning an asset label change into deletion permission. It preserves the old tree,
object identities and origin record using the existing metadata provider and
source claims. The current branch selection is retained.

Use `git_asset::annex::history` with `annex-publish`:

| API | Purpose |
| --- | --- |
| `Plan::capture(profile, working, origin)` | Freeze one exact predecessor and expected current source generation. |
| `publish(profile, plan, working, destinations, input, metadata, admit)` | Mirror only metadata nodes; finish/read back the exact tree and plan, then adopt media/record obligations. |
| `find` / `find_archive` / `load` | Recover the current saved slot or one exact retained Saved Version with its live metadata-generation guard. |
| `Loaded::saved` / `plan` | Preserve the exact reconciliation handle and original plan with owned workflow evidence. |
| `Loaded::member` | Resolve one logical path to the original Object/Snapshot without current/latest fallback or unrelated payload reads. |
| `Loaded::release_status` / `release` | Recheck metadata retention and working revision, then retire only the old working predecessor. |
| `refresh` | Explicitly update an unexecuted release's baseline after Main advances; retain old plans and the same archived root. |

Profiles use existing fenced namespaces: working branch/draft, retained media
declarations and a separate metadata-only record namespace. Bind the retained
MetadataStore identity in the record scope. Publication/store/release permissions
are separate and denied by default. Runtime creation/enrollment and private paths
are owner-managed; loading this SDK starts nothing.

The archive declares desired retained media placement. Its byte availability and
custody remain unknown until separately verified. Release reports
`media_custody_verified: null` and `authorizes_eviction: false`; it never transfers
or deletes payloads. The existing native eviction controller still refuses
missing/offline/corrupt keeper bytes, active work, unknown authority or stale
native origin. Jobs and other obligations are not retired by this operation.

A metadata failure can leave orphan nodes or extra retained demand; working
history stays protected. Use the caller-owned metadata provider's explicit
finish/recovery rules for pending writes. After the media or record adoption
boundary, retry the same plan/handle. First complete record wins. If Main advanced
before release, use explicit `refresh`; ordinary retry never invents a new plan.
Native uncertain-effect recovery continues through its original separate APIs.

Saved metadata can reopen in another process without the original working Git
checkout. To read actual bytes, obtain the exact member Snapshot and use the
existing admitted Publisher.open_retained with the original producer recipe.
Unavailable siblings remain unavailable; resolving one member does not hydrate
the archived tree. Current/historical registration, availability, custody and
creative selection remain separate.

The [contract](../specs/HISTORICAL-ROOT-ARCHIVE.md) defines failure, fence and budget
rules. The authenticated receiver also exposes an
explicit metadata-only history pass, described below. Advancing producer roots,
payload repair, broader scale and owner adoption remain required for the full
merge-to-placement offering.

## Run a bounded history pass

Use `github::native::Execution::reconcile_history` or the existing private receiver
with `phase: "history_preview"` first, then an explicitly reenrolled
`"history_apply"` policy/profile when approved. Set `policy.actions.history` to the
archive profile and `history_records` to its existing namespace root/checkpoint.
The retained `source.metadata_root/metadata_minimum` must match that profile's
record metadata provider. No producer indexes or payload resource opens are needed
for these phases. They retain the same strict authenticated request body.

Preview can reconcile the exact Git selection; it does not archive or release
historical obligations. Apply only publishes/releases as permitted by the signed
history profile. Archive-only operation uses allow_publish with allow_release
false. Permissions/configuration changes require a newly admitted policy identity.

Save each outcome's archive ID, plan ID, Saved handle and release receipt. Pass
`next` unchanged to continue. Releasing a predecessor changes its source revision;
the cursor remains bound to the unchanged current Version and proves a removed
after-key with its original retained record/release receipt. A changed commit or
policy needs a new pass. Reconnect/expiry never deletes media or forgets history.
A successful receipt remains authoritative even when its outcome also has a late
hold. Missing/corrupt/stale/denied/unsupported holds require repairing that stated
condition; uncertain native effects use their separate original-operation recovery.

Pages contain at most32 roots and the configured lower object limit, under an
outer deadline and disclosed per-root metadata bounds. Media stream bytes are0;
this is not a media availability survey. The report never claims whole-storage
convergence. HTTP/account/TLS and production installation require separate acceptance.

To preserve a usable producer recipe alongside archived membership, call
`Loaded::producer_record` with the original Record and exact historical path.
It preserves the original native Plan/control grant and returns an updated
retained Snapshot. Publish it through the bounded
[producer-root update SDK](NATIVE-PRODUCER-ROOTS.md); lookup/readback and actual
native open_retained remain separate from registration. Old index roots remain
readable and a historical recovery receipt never reselects an old root.
