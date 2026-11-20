# Annex managed object use

Input a16dc61d3c551dc686bfae9a566a040dd0069e0a; feat/annex-managed-read.
WP-004/007/008/009/011; REQ-001/003/004/006/007/008/010;
SPEC-003/004/005/007/008/009/011; IF-002/004/005/010/012;
PKG-002/003/004. RUST/PROVIDER/INTEGRATION profiles.

Scope: additive shared namespace object-use guards and an optional managed annex
read adapter, with synthetic/native fixtures. Preserve existing APIs, marker
formats, default features and dependency pins. No private controller/index import,
new persistent engine, native writes/removal, cache migration or live enrollment.
Entry: exact guarded annex reads, public guarded namespaces and canonical provider
Binding. Exit: actual pool configuration pins the admitted full roster; a read
holds use guards for every member while claim adoption remains possible and
exclusive retirement fences are blocked. This is a publication prerequisite,
not a retained-publication implementation.

L0: scope/roles/fmt/structure/hash pins/diff. L1: shared/exclusive compatibility,
same-hash conflicts, marker/identity/roster/alias failures, legacy read regression,
locked workspace all-feature tests, strict all-target Clippy and standalone Git.
L2: isolated temporary namespaces and explicitly admitted synthetic native annex
ordinary/bare fixtures. No existing caches, production assets or renders.
Producer review is neither independent review nor human approval.

## Shared use boundary

`Namespace::hold` returns a nonserializable/noncloneable `SubjectUse` for one exact
Object. It holds a shared lock on the existing hash partition and records the
namespace authority. Shared uses and ordinary guarded claim admission coexist;
`Namespace::fence` cannot acquire that partition exclusively while a use lives.
This allows later publication readback and owner adoption to overlap safely.
The guard survives dropping the Namespace handle. Drop/process exit releases
only the live lock, never a durable job claim or byte. A use grants no claim,
custody, current demand, expiry, release or removal permission. Partition
collisions may conservatively contend; there is no global cache lock.

## Pool enrollment and reads

Enable existing `git-asset/annex-read`; use `annex::managed::Reader` when participating
pruners must be fenced. Its pure constructor accepts the existing annex Profile,
a canonical store_protection::Binding, and exact namespace locations/minimum
checkpoints. It admits exactly the Binding's full roster (1..16 authorities) and
its existing place aliases, generation, writer/pruner boundary and history scope.
There is no directory discovery or inferred membership.

An explicitly enrolled initialized ordinary/bare pool records the Binding's
canonical digest in local Git configuration key `tig.protectionbinding`.
The existing exact raw config Object pins that value along with UUID/version.
Managed open verifies the repository/configuration, queries that one local key
without includes, requires the exact binding ID, then opens every enrolled
Namespace at its minimum ancestry, validates authority/place aliases and holds
its shared SubjectUse before native content lookup and exact byte verification.
Partial/wrong rosters cannot silently replace the actual pool binding. Fields
are trusted explicit enrollment, not discovery of unmanaged clients. No API here
creates or edits that configuration or initializes a pool.

The returned guard owns both the Windows byte/namespace guard and all shared
uses through Read/Seek/path access, even if its Reader is dropped. Construction
and unsupported-platform/context refusal perform no IO. Failed member acquisition
drops only newly acquired live guards. Errors retain distinct annex byte/config,
namespace admission, invalid roster and wrong-binding categories; they never
become Missing or trigger another store/version. Each read adds one bounded Git
configuration query to the original three-command local read path. All original
annex read budgets and platform/runtime limits continue to apply.
The existing ByteSource port is also implemented; its coarse byte error taxonomy
never converts namespace/roster uncertainty to Missing. Use typed `open` when the
consumer needs the full admission diagnosis.
Each read currently reopens its declared namespaces through their existing
bounded journal protocol. No large-batch metadata throughput or namespace-handle
caching is qualified by this slice.
Busy may be retried for the same Object after contention clears, within remaining
budgets. Invalid/wrong binding requires corrected enrollment; namespace corruption,
rollback or recovery errors require explicit recovery. None of these read errors
releases claims, changes selection or authorizes acquisition/deletion.

## Participating pruner obligation and limits

An admitted pruner must validate the current pool/configuration and complete
binding, acquire exclusive fences for every enrolled authority in canonical
order, and perform current protection/independent custody/authorization checks
under the appropriate provider boundary. An unheld gate is not deletion
permission. This slice adds no pruner, absence grant or removal API.

Raw filesystem deletion and standalone `git annex drop` do not participate in
this protocol. The existing Windows read guard still blocks mutation while held;
durable job protection after process exit relies on retained namespace claims
and participating pruners. Replacing a roster/epoch/boundary requires a separately
qualified handoff, not editing config to abandon old claims. No handoff or
production custody is qualified here. Native publication, uncertain-result
reconciliation and retained-output handoff remain the next integration work.
