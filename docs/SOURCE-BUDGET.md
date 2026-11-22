# Source and runtime data budgets

Current ordinary source/documentation ceiling: **8.625 MiB** (9,043,968 bytes),
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

2026-11-22 controlled allowance, input b02d5ad7e5a373bc672e528d52589162a3d9ad74:
ordinary source measured8,638,221 bytes, leaving12,531 bytes under8.25MiB.
Exact historical-root archival, retained metadata readback, revision-fenced
release and their restart/race/native-member fixtures exceed that remainder.
Add256KiB for generic implementation/contracts/fixtures and the independent U4
review integration. Keep128KiB/file and every binary/private/generated exclusion.
No runtime cache, metadata, journal, response, payload, admission or source-count
limit changes. No dependency or production enrollment is added.
WP-000/004/005/007/009/015, SPEC-003/005/007/008/010/011/015,
IF-004/005/008/010/012, PKG-001/003/004/006/009; RUST/PROVIDER/INTEGRATION.
Six producer checks: exact identities/data wire stay stable; metadata placement
is not byte custody; revision fencing and original recovery remain; scoped locks
permit unrelated progress; only authored code/tests/docs enter Git; acceptance
remains separate. L0 budget/pins/branding/diff, L1 locked/Clippy/features and L2
synthetic restart/native readback qualify component scope, not P2-P6 completion.

Explicit index successors, input3d024fab8c54a223863acff5f6f9163bd8c1f6b1:
the implementation checkpoint measured8,921,847 bytes after preserving the complete prior trace in an immutable
hash-pinned Git link, already8,951 bytes above8.5MiB. Admit128KiB of authored
Rust, synthetic failure/restart fixtures and contracts. No runtime/generated or
private data enters source; per-file128KiB and all runtime, metadata, journal,
response, provider and cache limits remain unchanged. Existing P3 controls apply.
Producer roles: identity preserves canonical old records; recovery retains exact
predecessors/receipts; placement/custody adds no authority; privacy adds no paths
or credentials; adoption keeps native/HTTP/owner acceptance separate. L0 pins and
budget plus L1/L2 source-qualified fixtures are mandatory, not waived by allowance.
