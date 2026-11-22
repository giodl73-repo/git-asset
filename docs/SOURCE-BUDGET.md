# Source and runtime data budgets

Current ordinary source/documentation ceiling: **8.25 MiB** (8,650,752 bytes),
with **128 KiB** per file, enforced by tools/validate.ps1. Earlier 1, 2, 3, 3.25, 3.5 and 4 MiB ceilings were historical checkpoints. This covers implemented Rust,
locked dependencies, synthetic fixtures and docs; it does not allocate cache
space or allow runtime evidence in Git.

Media, builds, databases, runtime logs, inventories, observations and growing
receipts belong outside ordinary Git. Tests generate synthetic temporary bytes.
Machine profiles, credentials and project acceptance also remain external.
Keep meaningful generic regression fixtures.

Budget changes require a concrete reason and recorded review. Do not weaken
checks, disguise generated data as source, or accumulate operational journals
in docs. Replace private notes with public summaries only after verified private
preservation.

2026-11-08 controlled increase (input355aad7): the 4 MiB implementation had
4,165,590 ordinary bytes before mandatory source-admission code and its process/
fault tests. The 5 MiB ceiling admits actual generic Rust implementation/tests;
per-file limits and all runtime/private/binary exclusions remain enforced. This
changes no payload, journal, inventory or capacity limit. DOCS/PROVIDER review is
recorded with the admission slice; generated fixture bytes remain external.

Native publication source9658c13 replaces duplicated historical startup/progress
prose with immutable Git links; no ceiling increase or runtime data enters Git.

2026-11-22 controlled source allowance, input6082aac75ce96550a538b8282fb59bded5bf6ad2:
qualified source measured8,388,383 bytes,225 bytes below the8MiB ceiling after
preserving completed reviews/resume in immutable Git. Add256KiB for the required
P3 authenticated native handoff/eviction integration, admission fencing and
synthetic stale/denied/crash fixtures. Keep current contracts and regression tests;
no runtime records, payloads, generated evidence or new dependencies enter Git.
128KiB/file and every cache/metadata/journal/payload/admission limit stay unchanged.
WP-000/005/007/009/015, SPEC-007/008/011/015, IF-008/010/012, PKG-006/009;
DOCS/PROVIDER/INTEGRATION. L0 budget/roles/hash/diff, L1 locked focused/Clippy,
L2 explicit synthetic native pools. Six-role producer review: identity/data wire
unchanged; placement/custody limits not relaxed; original recovery preserved;
no private/generated source; release/adoption gates remain independent and OPEN.
This source allowance closes no P2-P6 runtime/acceptance gate.
