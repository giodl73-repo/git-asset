# Native receiver and bounded copy reconciliation

Input e082c3133c2514155cb6ebf3d3c41ef7a1de3653; P3 WP-005/006/009 with
WP-004/007; REQ-001/002/004/006/007/008/010; SPEC-002/003/004/005/007/008/010/011;
IF-002/004/005/008/010/012; PKG-001/003/004/006. GIT/RUST/PROVIDER/INTEGRATION.
Allowed: fixed receiver profile dispatch and bounded native copy reconciliation
using qualified exact source, producer index, native copier and existing guards.
Entry: authenticated immutable commit and fixed admitted policy with exact index
roots. Exit: bounded reports with original plans/receipts and explicit holds,
never whole-placement convergence. L0 pins/source/roles/format; L1 locked tests,
strict Clippy and feature matrix; L2 synthetic native/Git/receiver fixtures only.
Independent/human acceptance and production activation remain separate.

Optional policy producer roots preserve the absent-field policy wire/hash.
Both roots participate in the signed enrollment identity. Exact hash and length
lookup uses source/destination metadata providers and a shared bounded lookup
meter; records do not authorize native effects. Reconstructed copy plans retain
original publication IDs, context and optional exact source snapshot.

One request reconciles at most128 required objects at the working alias, under
the actual selected Main generation. Desired requirements are bounded metadata,
never a payload census. Continuation binds policy, commit, admitted source revision
and exact ordered desired objects. A stale/missing after-key refuses. A suffix
does not prove prefix completion. Durable ingest handoff is optional; the separate registered eviction pass is
specified in NATIVE-EVICTION-PASS. Full batch convergence remains separate; storage_converged is always unknown here.

Dry run resolves original plans without native payload observation or effects.
Apply verifies actual managed working bytes and exact retained source custody;
only definitive working absence permits original Copier execution. Corruption,
unknown protection/configuration or offline custody holds. No observed-present
claim depends on a retired per-object ingest claim. Original candidate identity
still must match both planned locations. Reacquisition requires existing native
control admission; no new incarnation is invented automatically.
Publisher::observe_guarded returns fresh publication status while retaining the
existing native byte/roster/control and settled witness guards. It grants no
retention and is distinct from Ready. The copy pass requires original historical
completion and settled accounting before reporting present; an unfinished or
uncertain original publication holds even if payload bytes already exist.
Plan::progress reads only exact original witness/accounting records; it is not
custody. An index may name original working ingress or its frozen pre-copy plan.
The pass checks that recipe and the single deterministic copy-context recipe,
never an arbitrary current/latest producer. A unique completed/settled match
is revalidated by observe_guarded. Incomplete matching intent holds; conflicts
without one exact completed match refuse. This permits original F ingress and
future M-to-F restoration without reimporting or inventing a copy receipt.
Outcomes name the actual observed publication separately from the copy-plan ID.

Admission/ref is rechecked before each native action and after successful work.
Successful receipts survive later drift; failures stop the page at that object
and return its original plan/basis for reconciliation. Unknown native outcomes
must use original Copier status/resume; a new plan is not a safe retry. Durable
claims survive exit; live handles are not persistent jobs. Receiver opens only
explicit existing roots after authentication, installs no daemon/service, and
never reads plans/paths/code/credentials from request bodies. Old local/LFS
receiver dispatch remains supported.

Bounds: existing per-action native IO/copy/publication limits, request object/
prospective-byte/deadline budget, aggregate lookup node/read-byte budget, and
at most128 outcome records. A deadline stops between native operations, not by
interrupting a successful native effect and losing its receipt. Required custody
and Main guards stay held during each operation. Runtime evidence belongs
asset-side; this adds no queue, mutable cache registry or operation journal.

Optional reverse backup and its separate permission/receipt lifecycle are defined
in [TRUSTED-NATIVE-BACKUP](TRUSTED-NATIVE-BACKUP.md). It uses the same fixed pair
and frozen index records; root publication and full repair/reacquisition remain separate.

With a private pending namespace binding, reconcile_with_actions runs admitted
handoff after a successful copy, backup or verified-present observation. The
original reconcile API remains copy-only. The receiver requires a pending binding
matching signed Actions.pending whenever handoff is enabled; paths come only
from its private profile. Existing namespaces are opened after authentication.
The worker also checks the actual namespace/provider identities before selection.

Handoff uses the actual completed working recipe and exact retained recipe,
never the prospective forward-copy destination when original ingress supplied
the bytes. An explicit producer-index snapshot wins. Otherwise the retained
publication's original Select receipt identifies its exact Version; current/latest
substitution is forbidden. Plan::retained_snapshot is metadata capture only;
the native handoff still validates actual roster, generation, membership and bytes.

Deadline/ref/admission checks precede the phase. A copy's post-effect hold skips
new handoff work without losing its receipt. Durable capture precedes release.
Outcome.handoff returns Saved and the mutation or failure; post-save failure also
holds the page. A fresh authenticated retry discovers the same frozen action,
including after temporary ingest release. No action slot is overwritten and no
media bytes are removed by this phase. Storage convergence remains unknown.

The metadata-only history_preview/history_apply phases are specified in
[HISTORICAL-ROOT-ARCHIVE](HISTORICAL-ROOT-ARCHIVE.md#authenticated-receiver-scheduling).
They use optional signed Actions.history plus the private history_records locator,
without producer-index or payload resource opens. Their cursor binds current
Version rather than the source revision changed by predecessor release. These
phases grant no native copy/removal authority and report custody as unknown.
