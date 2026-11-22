# Native producer index producer review

Input0b73387e26e3b747611c824d387bb58c482a0430; controls and allowed scope in
[contract](../../specs/NATIVE-PRODUCER-INDEX.md). Six .roles checklists applied
by producer; no independent review, human approval or production acceptance.

| Role | Finding; severity | Disposition / remaining proof |
| --- | --- | --- |
| Identity | H: same digest with wrong byte length must refuse. | Exact Object comparison, canonical record/key/plan/snapshot consistency; immutable roots. |
| Placement | H: paging must not scan an entire asset inventory. | Persistent map range walk, finite page and aggregate tree/record read budgets;10,000-row fixture. |
| Custody | H: producer registration cannot imply live custody. | API returns declarations only; native origin/bytes/protection independently required. No eviction entry here. |
| Recovery | H: a root change or arbitrary suffix must not imply convergence. | Root-bound cursor rejects drift; absent after-key refuses; unseen prefix and original uncertain operations stay unresolved. |
| Provider/privacy | M: no path discovery or payload hydration. | Existing metadata NodeStore only,32KiB rows; no event paths, subprocess or machine roots in API. |
| Git/adoption | M: growing inventory must stay asset-side. | Existing immutable tree publication, no per-object Git file or new journal; receiver/batch/root-retention qualification OPEN. |

Producer owns all dispositions. Verification:120 core tests (including three
new paging cases and doctests) and two producer-index cases passed; all synthetic.
Strict workspace all-feature/all-target Clippy, standalone annex-publish feature
and L0 pass; terminal outcomes remain in external producer-index-* logs. Source pins
and exact terminal outcomes must be checked separately. This component does not
close P3 integration or any per-project adoption gate.
