# Recovering original native actions

The experimental SDK can retain a complete frozen handoff or eviction plan before
calling the existing native worker. This closes the gap where an uncertain
operation's ID survived but its original body had to be kept elsewhere.
It uses the existing metadata provider and scoped source claims. It does not
create a cache, daemon, separate journal or eviction authority.

`github::native::Actions.pending` is optional. Its `annex::pending::Scope` binds
an existing fenced metadata namespace, metadata-provider identity, owner,
source-key prefix and metadata-only place. `allow_store` defaults false. The
namespace must be separate from all media-pool members; its job policy must retain
current/pending/predecessor metadata on that one place. The native receiver can bind that namespace in its private profile and run
durable ingest handoff after verified copy/backup/presence. Eviction scheduling
remains separate work.

`Execution::handoff_durable(HandoffRequest, namespace, working, retained, clock)`
looks for the original saved slot before constructing a new handoff plan. After
ingest release, the same slot still supplies the exact original plan and producer
recipes, so a retry does not need to reconstruct them from current claims.
`Execution::evict_durable(plan, namespace, sources, nodes, clock)` likewise uses
the first saved original instead of overwriting it with a newer preview.

Both methods return `Durable`: an exact `saved` handle plus either the existing
worker `mutation` or a structured `failure`. Errors before durable saving return
normally without executing the action. After saving, native/admission failures
retain the handle. Current authentication, Main selection and all native
protection/custody/control checks still apply; saved metadata is not a replay grant.

The lower-level `annex::pending` APIs are:

| API | Purpose |
| --- | --- |
| `save` | Write canonical plan/producer blobs, finish/read back metadata, then atomically adopt a retaining source; existing slot wins. |
| `find` | Find one exact kind/Object/original-working-producer slot. Missing means no saved entry, not absence of native bytes. |
| `reopen` | Load the exact Saved Version/envelope with a live source-generation guard and cumulative tree/blob budgets. |
| `Loaded.action` | Typed original handoff or eviction plus both publication recipes for original backend status/resume/cancel. |

All four referenced objects are retained by the saved Tree: envelope, action body,
working publication and retained publication. Each blob is at most64KiB. Existing
metadata quotas and source-count limits still apply; admission may refuse safely.
Sources are scoped per original producer/action rather than locked across an
entire cache. Expiry never deletes or forgets records. This first surface has no
automatic slot replacement, completion cleanup or metadata-forgetting operation.

Persist the `Saved` handle with the consumer's owned task evidence. If a process
exits before it receives the reply, `find` uses the same immutable inputs to
discover the original entry. A competing plan cannot overwrite it. Reopen the
stored body and use its backend status/resume/cancel interface; never replace an
uncertain native operation with a new ID. Store permission may be disabled after
capture while the same scope remains readable for recovery.

Component fixtures cover a separate-process reopen, competing preview, denied
store, bounded/corrupt/foreign handles, scoped blocking with unrelated progress,
and durable native handoff/eviction paths. These are producer tests, not independent
review, complete receiver eviction scheduling or production adoption. See the
[contract](../specs/NATIVE-PENDING-ACTIONS.md) and
[native receiver](NATIVE-RECEIVER.md).
