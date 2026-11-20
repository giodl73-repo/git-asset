# Native job protection projection

Input 6eaf75904868a86201eeb55fc61e10b414fa7759; feat/native-job-protection.
WP-004/008/009/010/011; REQ-001/003/004/006/007/008/010;
SPEC-002/003/004/005/007/008/010/011; IF-002/004/005/008/010/012;
PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION profiles.

Allowed scope: optional native job metadata projection into existing flat byte
bindings, exact readback, synthetic guarded-namespace lifecycle fixtures and
bounded controls. No REEL dependency, new journal, lease service, provider engine,
cache enrollment, annex writes, media production, selection or physical removal.
Entry: an admitted JobContext plus public guarded Namespace. Exit: every declared
job Object is represented exactly in bounded flat bindings, independently read
back before namespace adoption, and remains an obligation through restart,
concurrent source changes and explicit withdrawal. Full custody and publication
acceptance remain separate.

L0: source/scope/roles, fmt, structure/hash pins and diff. L1: deterministic
projection, exact membership/context, substitution/missing/corrupt metadata and
aggregate read/write/count budgets; locked focused/workspace all-feature tests,
strict all-target Clippy and standalone tig-git check. L2: temporary real Git,
finished/reopened metadata and actual guarded Namespace restart/retry/stale/
cancellation fixtures. No live caches or renders. Producer review is not
independent or human approval.

## Contract

Consume `tig_git::source::native::job::protection` through the optional
`tig-git/assets` Rust surface; schema/history callers may keep their existing
features. No new CLI or consumer-specific dependency is introduced.
Byte-only jobs may use their admitted flat Tree and Namespace directly; this
adapter does not require them to construct a typed native JobContext.

`job::protection::freeze` writes a candidate flat Selection with one binding per
deduplicated `JobContext::objects()` Object plus a domain-separated header that
pins the exact source job Selection. It does not recursively interpret opaque
payloads or assert metadata-provider custody. Original job roots/encodings and
all existing APIs stay unchanged. The header and full declared resource set,
including interpretation, runtime, composition, input/output evidence and claimed
outputs, are preserved. Aliases deduplicate by exact Object, not logical asset ID.

Finish the metadata provider and independently reopen it, then call `verify`
against the exact admitted/restored JobContext. Verification requires identical
header, context and flat entry map; it writes nothing and opens no payloads.
Whole entry bounds are checked before provider IO. Header and tree IO share one
call's TreeLimits; failed writes may leave unreferenced immutable prefixes.
At most 65,536 flat entries are admitted, including the header. Existing journal
limits remain 1,024 sources, 64 predecessors per source, 16,384 commands and
16 MiB of command bytes per namespace; exhaustion refuses, preserving old claims.
Original logical byte totals exclude projection/tree/provider overhead; this
metadata operation reserves no physical capacity.

Enroll each independent attempt as Kind::Job in the existing guarded Namespace,
then use Namespace::select with explicit prepare/adopt operation IDs, expected
source revision and the verified flat Selection. Native job Selection itself is
opaque and must not be submitted as if it were flat bindings. Namespace policy
chooses places; projection invents no cache or maturity policy. Adoption records
desired protection only, never availability, independent custody or complete
global enrollment.

## Long-running work and recovery

Keep the exact namespace identity/minimum checkpoint, source/owner, operation
IDs, expected revision, original admitted job and flat Selection in the caller's
asset-side operation record. The namespace owns durable obligations. Process exit
and worktree closure do not release them; reopening validates minimum ancestry.
An independent source advancing cannot overwrite this attempt. A stale revision
refuses; an identical operation replay returns its historical receipt. Check
current source state separately before treating that receipt as current demand.

There is no timeout/expiry deletion and no renewal timer in this profile.
Manual handoffs remain protected until an explicit owner transition. Cancellation
uses Namespace::apply(Withdraw), preserving the former selection as a predecessor
under declared history placement. It neither retires that predecessor nor deletes
bytes. Retire, settlement, final loss and metadata forgetting require their own
existing authority/contracts. New claimed outputs may be adopted as a later job
selection, retaining the previous job as a predecessor. This is separate from
creative selection, approval and delivery.

On an uncertain selection reply, use Namespace::selection_status with the exact
Select request. Retry is safe only with unchanged operation/request identity;
conflicting reuse must not be repaired by overwriting records. Missing/corrupt
metadata blocks new admission without releasing earlier obligations. Namespace
Budget/Busy/Stale/Owner/Conflict remain distinct from projection Budget,
Incomplete/Integrity/Metadata errors. There is no implicit recovery write.

## Remaining integration

REEL can consume these Rust interfaces alongside its existing history surface and
the optional annex read provider. Binding an actual annex pool to a complete
protection roster and contained publication/readback/retention still requires
separate qualification. These metadata obligations do not constrain unmanaged
`git annex drop`, raw filesystem deletion or other nonparticipating pruners.
