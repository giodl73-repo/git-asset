# Native committed source producer review

Inputf6e66f2; controls/scope: [contract](../../specs/ANNEX-COMMITTED-SOURCE.md).
Producer six-role checklists, not independent/human approval.

- Identity: native keys preserve exact hash/size, not Git blob identity; conflicting
  sizes refuse. UTF8 boundary checks prevent malformed-key panics.
- Placement: only complete explicit scope binds; budgets/missing metadata hold.
- Custody: metadata declaration is no payload/custody proof; bare mirror has none.
- Recovery: exact historical capture ignores newer refs; current-ref admission
  remains separate. No operations or claims are allocated by capture.
- Provider/privacy: no symlink following, payload IO, filters or arbitrary URLs;
  existing authenticated HTTP and raw blob hashes remain mandatory.
- Adoption: explicit native adapters/domains preserve old readers/hash encodings.

Verification:43 focused integration +18 GitHub library cases, actual native
emitted-pointer/bare case, strict workspace Clippy/fmt/L0/diff pass. Default native
entry remains ignored and is not counted as execution. Native fixture commit
needed its explicitly admitted annex PATH; no runtime/global binding was changed.
Full P3 native placement/receiver and P2/P5/P6 adoption remain OPEN.


Current-generation extension, input69a1d5c (same controls): producer review of
Execution::hold_current, its CurrentUseError and real-Git lifecycle fixtures.
Identity/recovery H: use actual current source revision/full Version, including
no-op captures; pre-admission expected_revision is not a live generation token.
Implemented using the existing source_use gate. Placement/custody H: this pins
one source, never bytes, all namespace demand or independent backup; native
controller proofs remain mandatory. Provider H: Git may advance while held;
sample before/after acquisition and require per-effect ref/auth revalidation.
Adoption M: legacy returns explicit Protection(Unsupported), no implicit upgrade.
All findings owned by P3 producer, addressed within this component's contract;
full worker/handoff/revert/crash/outage qualification remains OPEN. Tests below
must pass before the implementation checkpoint; producer checks are not approval.

Current-use qualification:20 focused tests pass (16 lifecycle,4 native-source;
ignored tool entry not counted), strict workspace/all-feature/all-target Clippy,
fmt and L0 pass. Initial fixture expected the outer Busy variant; the actual
journal Busy is now asserted exactly. Full P3 integration remains OPEN.
