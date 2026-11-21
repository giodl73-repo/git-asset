# Native candidate publication and retained readback

Input 9658c13f9c4a169e468a99feb30b4e4fe8d11351. WP-003/004/005/007/008/009/011/014/015;
REQ-001/003/004/005/006/007/008/009/010; SPEC-002/003/004/005/007/008/009/011/013;
IF-002/003/004/005/006/008/010/012; PKG-002/003/004/006/008.
RUST/PROVIDER/GIT/INTEGRATION profiles. Entry: explicit initialized format10 pool,
complete pool-pinned managed roster, existing metadata/retention namespace and
witness store/ledger/job. Allowed: optional additive Rust publication adapter,
existing guards/journals/accounted witnesses, private bounded staging and public
synthetic fixtures. No cache enrollment/init, native copy/drop/pruner, network,
creative selection, renderer, private-controller import or production deployment.

Exit: one exact externally supplied binary becomes an immutable unselected
candidate, original bytes/provenance survive, durable protection precedes native
write, exact retry/reopen recovers interruptions without duplicate registration,
and retained bytes are independently reopened before consumer completion.
Native byte custody is conditional on the enrolled cooperative protection boundary;
it is neither an independent backup nor protection from unmanaged annex commands.

## Composition

Use git-annex's native ingress; do not add a payload store. The
[reinject plumbing](https://git-annex.branchable.com/git-annex-reinject/) can consume
a private verified staging copy named by the exact SHA256 key. Producer input is
never passed to a command that moves it. No Git worktree/index file is created or
selected. An initialized empty annex root may lack objects/: publication admission
checks the pinned config, UUID, version and full roster while treating that absent
container as missing content without an annex lookup. Only explicit native ingress
creates it. Existing read-only adapter admission remains unchanged. Ordinary and bare pools are separate explicit qualifications.

The caller supplies a bounded provenance metadata Object, logical asset identity,
output Object, original operation/owner, expected retention-source revision and
admitted pool/config/roster/capacity/job/witness identities. Freeze a canonical
descriptor and flat metadata bindings; finish/reopen/verify the metadata provider
before admission. Provenance bytes are immutable metadata; larger prompt/result
packets remain separate caller-protected assets referenced by that provenance.
No provider, prompt, timing or creative semantics are inferred. Only output bytes
are flat pool byte obligations; descriptor/provenance remain in the existing
append-only metadata provider. Selection context and accounted intent pin them;
the handoff returns explicit metadata/namespace/capacity recovery minima.

Use the existing Retention-class Namespace source and Select operation for
candidate registration/protection, separately from creative Branch selection.
Registration can be pending physical availability. Current and predecessor
placement must retain the pool. Persist registration before any native write;
retain older candidate versions. Same operation replay returns original receipts;
changed request/context conflicts. Historical receipts alone cannot recreate
protection explicitly retired by the owner. Recheck the exact registered
selection and current assigned pool obligation on apply and reopen.

The ledger exposes only an additive exact generic-allocation observation for
read-only reconciliation; it appends no commands and grants no fresh admission.
Existing upload_witness::Packet supplies accounted immutable intent/completion and
the scoped attempt fence; no second journal is introduced. Bind the entire plan,
pool UUID/config/roster, namespace, output/provenance and resource identities.
Admit a separate existing-ledger payload allocation before staging/native IO.
Reserve a conservative bounded transient peak; settle observed allocated final
and remaining owned-stage file data. SQL/log/directory overhead and other writers
require separately admitted headroom; no claim of complete physical quota follows.

Hold the exact pool config, initialized annex directory and every namespace SubjectUse through lookup, staging,
native mutation, readback, protection recheck and completion. Preserve command,
elapsed, output, streaming and aggregate verification budgets. Windows process
containment remains mandatory; unsupported platforms/entered runtimes refuse
before IO. Publication defaults disabled; inspection never initializes or repairs.

Staging is one exact operation-owned file under an explicitly existing, pinned
private root. Bounded prefix comparison against the exact source permits resuming
an interrupted staging copy; mismatches hold without overwriting. A complete
verified stage may resume without source availability. No stage census or sweep.
After native uncertainty inspect the exact key, never replace the operation or
guess another key/version. Missing/corrupt acknowledged bytes are incidents,
never an automatic reupload. Completion requires fresh exact native readback and
durable protection; returned live guards bridge downstream owner handoff.

## Rust consumer contract

Enable the `annex-publish` feature on package `git-asset`; consume
`git_asset::annex::publication`. This implies `annex-read` and the existing
accounted witness implementation. No CLI publication command is exposed yet.
Pin the qualified repository commit and package path `crates/git-asset`, together
with its same-revision workspace packages; do not mix private-controller APIs.

1. Admit existing stores, roster, Retention source, capacity job and private stage
   root. Set `allow_publish` explicitly; `Publisher::new` and `Plan::new` do no IO.
2. Put bounded opaque provenance in `MetadataStore`; call `freeze(Request, ... )`,
   finish and reopen it as `MetadataReader`. Create the immutable `Plan` using
   exact `Admission` identities. Persist that plan and enrolled profile before IO.
3. Preview `Plan::candidate`, `payload_peak` and `required_peak(witness)`. The
   closure is exactly one body plus descriptor/provenance metadata; no census,
   directory expansion or implicit additional inputs occur.
4. Call `Publisher::apply(plan, Resources, ByteSource)`. Registration is a
   Retention source transition, separate from creative selection. No repository
   commit/index or selected asset pointer is changed. Original-operation retry
   cannot replace an expected baseline after another writer advances it.
5. Keep returned `Ready` alive while independently adopting the exact candidate
   into the downstream owner's durable claims. Persist `Receipt`, `Recovery`
   minima and original plan/profile handles. Read/seek or use its guarded path;
   confirm adoption before REEL records completion or retires owned scratch.
6. After restart, open the enrolled stores against saved recovery minima and use
   `status`, `resume` or `reopen` on the original plan. `reopen` freshly verifies
   retained bytes without accessing original input or stage. A serialized receipt
   alone is not a byte guard. No CAMINOS/CAIMITOS checkout is required.

Capacity/job finish is separate from retention. The exact candidate remains a
durable current/predecessor obligation across process exit, job finish, closed-job
retirement and days-long manual handoffs. No wall-clock TTL, renewal or budget
timeout withdraws it. Cancellation means stop work, reconcile uncertain outcomes,
then explicitly withdraw/retire the owner's claims under existing admission;
neither cancellation nor timeout deletes bytes. Historical reopen never selects
that candidate as current. Expiry policy and creative approval belong to REEL.

`Stage`/`apply_hook` exposes Registered, Intent, StagingChunk, Staged, Imported,
Completed and Settled boundaries for deterministic interruption/consumer evidence.
An error after any boundary may leave durable partial work; there is no rollback
promise. Registration can precede physical bytes. Witness completion can precede
payload-accounting settlement. Reconcile by original plan/operation, never by
allocating a replacement operation when a command reply is lost.

| Error code | Consumer action |
| --- | --- |
| `busy` | Defer; retry the same plan after the competing attempt releases its scoped fence |
| `uncertain`, `needs_recovery` | Inspect exact state and reconcile/resume the original operation; preserve input/stage and original admission |
| `missing` | Restore the exact declared source for incomplete staging; no latest fallback |
| `missing_acknowledged`, `corrupt` | Hold as a custody/integrity incident; no automatic reupload or completion |
| `stale`, `conflict` | Correct admission/baseline explicitly; do not invent a retry operation to bypass it |
| `denied`, `admission_denied` | Correct explicit permission/capacity or bounded budget; effects may already exist, so inspect first |
| `protection_lost` | Restore legitimate owner protection explicitly; an old receipt cannot resurrect retired claims |
| `invalid`, `unsupported` | Correct caller/platform/runtime admission; blind retry is ineffective |

Reports contain bounded identities/booleans and recovery checkpoints, not an
expanding journal dump or physical path inventory. Limits are independent: the
profile caps a body at 64 GiB, transfer accounting at 1 TiB and cooperative work at
300 seconds; configured lower limits still apply. The local actual-native fixture
qualifies 32 MiB, not those maxima. Source verification, metadata traversal and
native subprocesses also have their own budgets; no lifetime bandwidth or physical
quota guarantee follows. Windows-only native ingress refuses entered Tokio
runtimes before IO; invoke it from a suitable blocking worker. Staging is private
operator-owned storage; hostile filesystem actors and device/power-loss durability
are not qualified by process-exit fixtures.

## Verification and limits

L0: current source/hash/link budgets, frozen formats, fmt/diff and producer lenses.
L1: pure freeze/plan/verify, wrong metadata/UUID/config/roster/path/admission,
disabled/unsupported/busy/stale/conflict, corrupt/missing bytes, bounded IO and
original-source preservation. Locked regressions, strict all-feature/all-target
Clippy and standalone check. L2: explicit synthetic native ordinary/bare pools,
candidate/provenance/history readback, actual process exits around registration,
intent, staging, native write and completion, fresh-process original-operation
recovery, concurrent attempts and owner adoption under live guards.

Producer checklists are not independent/human acceptance. Record implementation,
verification, intended-use validation, trace and review separately. Hosted exact
head, representative large-file/device/crash/power-loss and REEL production
adoption remain distinct gates. CLI expansion and native pruner work follow this
qualified SDK slice; this contract grants no removal authority.

Source budget: replace duplicated historical startup/progress prose with immutable
9658c13 links (including the incremental journal producer history), retaining current priorities/exit gates and all authoritative specs.
No runtime/source-budget ceiling is raised and no original history is deleted.
