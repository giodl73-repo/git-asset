# Managed annex read producer review

Input a16dc61d3c551dc686bfae9a566a040dd0069e0a; feat/annex-managed-read.
EVID-122. [Contract](../../specs/ANNEX-MANAGED-READ.md) names controls,
allowed scope, entry/exit and L0/L1/L2. Owner: maintainers. This is producer
review, not independent review, human approval or deployment authority.

## Implementation

Add Namespace::hold/SubjectUse on the existing shared admission partitions.
These live uses permit ordinary owner adoption while preventing overlapping
exclusive retirement fences. No journal record, marker change, expiry timer or
new persistent engine is introduced. Durable job obligations remain separate.

Add optional annex::managed::Reader/Read using the existing ByteSource port and
typed open errors. An exact raw pool config pins a canonical existing provider
Binding digest under tig.protectionbinding. Every admitted namespace identity,
minimum ancestry and place alias must match before content lookup. A successful
read owns both the exact Windows byte guard and all namespace uses, even after
its Reader is dropped. Refactor the legacy Reader into private admission/local
lookup helpers while preserving its public profile and three-command behavior.

Native publication needs shared use to overlap owner adoption; the exclusive
fence would block that handoff. This slice qualifies that necessary pool-use
boundary first. It adds no publication, retained-output acknowledgement, native
pruner, roster handoff or private archive controller/index dependency.

## Verification

Two focused store tests pass. They cover compatible adoption, multiple uses,
same-hash/different-length exclusion, unrelated partition progress, namespace
handle drop, exclusive-versus-shared refusal and actual forced process death.
After child termination the live lock is released while job history remains.
Existing marker-replacement/missing-anchor tests now also refuse shared use.

Eleven annex tests pass (seven existing, four managed). They check pure exact
roster admission, both typed and ByteSource reads, owner adoption while all uses
live, wrong pool binding, unknown place aliases, partial acquisition cleanup,
payload corruption and entered-Tokio refusal. The initial store-test compile
found a fixture borrow conflict; cloning its small Object argument corrected it,
and shared-v2 passes. No production check was relaxed.

The explicit native ordinary/bare fixture passes with fresh temporary Git/annex
repositories and opaque 22-byte data. Each includes a two-namespace binding,
actual native lookup, exact readback, held deletion refusal, exclusive-fence
refusal, guard release and unchanged config during reads. Only fixture setup
writes the enrollment field. It remains deliberately ignored in routine runs.
Strict all-feature/all-target Clippy passes. The full locked all-feature workspace
passes with two test threads, including the 500,000-binding persistence case.
Locked standalone Git/assets-only checks and final fmt/structure/hash/diff pass.
These are separate closure results; dependency pins remain unchanged.

The first four-thread broad run stopped in the existing seventeen-root aged
retirement fixture with Age(Budget), after 159 native-source cases passed. The
unchanged case passes alone in 94.36 seconds. Temporary deadline/read diagnostics
did not reproduce the error in that isolated run and were restored byte-for-byte;
neither age defaults nor resource limits changed. A load-sensitive deadline is an
inference, not a confirmed diagnosis of the original failure. The unchanged case
also passes within the complete two-thread run. Hosted four-thread qualification
and the original parallel budget failure remain open; maintainers own follow-up.

## Intended-use validation

This demonstrates the required read/use/adoption lock relationship and actual
pool-roster verification. A free gate is not absence, custody or removal
authorization. An enrolled pruner must validate current complete binding and
perform the existing independent protection/custody/authorization protocol.
Standalone annex drop and raw filesystem actors do not honor namespace locks.
No production cache, remote custody, roster replacement, physical removal,
retained publication or complete REEL workflow acceptance is qualified.

## Trace and producer lenses

WP-004/007/008/009/011; REQ-001/003/004/006/007/008/010 ->
SPEC-003/004/005/007/008/009/011, IF-002/004/005/010/012, PKG-002/003/004 ->
ANNEX-MANAGED-READ.md -> shared use, pool-pinned full roster and exact guarded
reads -> EVID-122. Full IF-004/provider acceptance stays open.

- Identity: config Object, UUID, binding digest, namespace ancestry and aliases
  are checked; partial or substituted rosters cannot silently replace the pool.
- Placement: use the existing canonical Binding and namespace policy, without
  drive or maturity assumptions. Unknown coverage is never zero demand.
- Custody: no read/use is a durable retention grant or eviction permission.
  Maintainers own native publication and participating-pruner qualification next.
- Concurrency/recovery: shared admission coexists with use; exclusive fences
  conflict. Partial errors release only live guards, not claims or bytes.
- Provider/privacy: finite declared roots, no discovery/init/transfer and no
  private paths in reports. Typed admission errors never become payload absence.
- Git/adoption: existing APIs, defaults, markers and pins stay compatible.
  Operational enrollment remains outside authored Git and is not auto-installed.

No blocking finding remains within the managed-read scope. Independent review,
hosted qualification and human intended-use approval remain open.
