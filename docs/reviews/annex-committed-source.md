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
