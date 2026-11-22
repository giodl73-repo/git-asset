# Experimental native eviction SDK

`git_asset::annex::retirement::planning::Controller` is available with
`annex-publish`. Construction only borrows a trusted owner `Profile`; it performs
no IO. Operations are explicit. This is an experimental namespace-owned Windows
adapter, not mixed legacy pruning or production acceptance. The
[contract](../../specs/ANNEX-PROTECTED-EVICTION.md) lists remaining P2 gates.

Use the existing publication SDK for ingress/copy. The controller coordinates
existing lifecycle and protection engines; it introduces no content store.

| Method | Inputs and behavior |
| --- | --- |
| `plan` | `Request { operation, snapshot, grant }`, `Sources` and `Nodes`; returns canonical immutable `Plan` after actual source/keeper/producer/device checks. |
| `apply` | Frozen `Plan`, fresh `Sources` and `Nodes`; reconstructs all proofs and requires the identical plan before any native removal. Fresh attempts only. |
| `status` | Frozen plan and original source publication profile/plan; readonly bounded metadata/effect/presence report, no removal authority. |
| `resume` | Same original context plus source metadata reader/limits; reconciles original completion/cancellation without native removal. |
| `cancel` | Same original context plus source metadata reader/limits; retains verified surviving bytes and refuses any native effect intent/completion. |

`Sources` contains original physical source and keeper `Producer` resources:
publication profile/plan, existing witness store and mutable capacity ledger.
`Nodes` supplies source and keeper metadata readers, protection limits and tree
limits. Reuse existing readonly `MetadataReader`s; no caller must manufacture a
`Live` token or independently orchestrate guard release. Source/keeper resources
must match the frozen control identities and private runtime policy. A hash,
visible file, alias acknowledgement or protocol flag alone is insufficient.

The owner profile defaults deny. Initial admission requires namespace-owned
actors, independently admitted unshared direct backing, distinct failure domains,
actual distinct device observations and exactly one retained keeper. Stronger
quorums cannot be silently reduced. The source's actual configuration must seal
the exact policy/runtime before original publication. Mixed legacy actors are
unsupported until the separate owner-side epoch/handoff protocol is qualified.

Save `Plan::to_json()` and its ID outside Git with operational evidence. Reopen
with `Plan::from_json()` (canonical, at most64KiB), preserving original producer
plans and trusted profile bindings. Do not make a replacement plan/journal to
bypass an uncertain outcome. `Error::code()` distinguishes protection, stale,
conflict, missing/corrupt, admission, unsupported and recovery/uncertain holds.
Busy permits later retry with current proofs. Stale requires reinspection;
needs-recovery/uncertain require original-operation inspection/recovery, never blind apply or
cancellation. Missing acknowledged bytes are an incident, not permission.

A new raw job demand may coexist with historical completion or cancellation;
neither repeats removal. Same-hash restoration creates an explicit new control
incarnation; old completion reports history and current presence separately.
Reports distinguish logical subject bytes, source file allocation (fresh apply
only), metadata receipts and presence. They do not claim net free disk space.

The native CLI and actual executor process-exit/epoch qualification remain open.
The ordinary/bare synthetic native fixture exercises the SDK preview, fresh
apply, readonly status, historical resume, cancellation and retained restore.
No cache operations run on startup, help or construction.
