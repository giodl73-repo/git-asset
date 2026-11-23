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

2026-11-22 controlled allowance, input f12a1634a8833f822edf2a78b4df75a2b3851538:
tracked main source totals8,388,156 bytes,452 bytes below its8MiB ceiling.
The authored issue75 interface audit/PR plan and reviewed target pins need more
room. Reuse the finite8.25MiB source allowance already reviewed in the P3 lane,
so the independent U4 review can land on main without a storage dependency.
This is source/documentation only:128KiB/file, binary/private/runtime exclusions
and all metadata, journal, payload, response, admission and cache limits remain.
WP-000/015/016; DOCS/UI/PROVIDER/INTEGRATION. Six producer checks: identity/wire
unchanged; placement/custody limits intact; original recovery unaffected; no
private/generated source; explicit source pins; independent acceptance OPEN.
L0 source/links/pins/branding/diff proves this bounded documentation change only.
