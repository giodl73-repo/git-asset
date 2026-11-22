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
durable ingest handoff after verified copy/backup/presence. The eviction receiver uses the same saved-action API;
complete repair/replacement/root acceptance remains separate.

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

The eviction receiver now pages registered working candidates and calls the
durable API. Before fresh removal, `evict_durable` inspects the original native
controller status: completed work reconciles through `resume`; uncertain intent
holds. This permits completed accounting recovery after physical absence without
substituting a new plan. The keeper's backend metadata may still be required.
See [registered eviction](../specs/NATIVE-EVICTION-PASS.md) for fixed profile phases,
bounds, grant admission and remaining repair/replacement obligations.

After completed authorized eviction, the additive Rust
`annex::publication::restore` SDK prepares an exact saved restore Plan, reuses
metadata reacquisition, and delegates native import/reopen to the controlled
Publisher. Save before apply; retry the same Plan after an interrupted metadata
birth or uncertain reply. A managed retained reader supplies the old bytes.
Completed replay returns retained readback without another registration. This
does not retarget creative selection, repair unexplained loss, or add a receiver
restore phase. See [controlled restoration](../specs/NATIVE-RESTORE.md) for
permissions, crash boundaries, limits and remaining qualification gates.

The same four-blob pending mechanism now retains `Restore` and `RestoreIndex`
actions. Native bodies preserve the exact eviction plus restore Plan; index bodies
also preserve the qualified index Plan and require the same native restore recipe.
Both original producer publications remain retained separately. Save/read back
before execution, then execute the loaded body rather than a later preview.
Existing phase/object/original-producer slots win on retry, including when new
storage is disabled. A stale index Plan is not overwritten; use its original
status and explicit SDK replan. These metadata records alone prove no media
custody and do not add a receiver restore phase. The [saved-action contract](../specs/NATIVE-RESTORE.md#saved-restore-actions)
defines bounds and the remaining authenticated orchestration gate.

Rust package `git-asset` with `github` and `annex-publish` also exposes
`github::native::Execution::{restore_saved,register_restore_saved}`. Pass an
existing Saved handle and fixed enrolled resources. The worker loads its original
body, checks the signed policy, exact current committed-source guard and working
demand before new effects. Restore returns owned Ready bytes plus its receipt;
index execution requires the signed destination head and preserves completed
receipts with index writes disabled and native storage offline. Renew expired
credentials/key snapshots and retry the same handle. Post-effect holds do not
erase success. These SDK operations do not create a receiver restore phase or
replace a stale plan; see the [execution contract](../specs/NATIVE-RESTORE.md#authenticated-saved-execution).
