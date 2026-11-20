# README narrative and CI scope

Input e78cabff8bb6d49f80065b924c2f685c6a3e3060. PKG-009 / SPEC-015;
DOCS profile. Narrative traces WP-006/009/010/011/012/015,
REQ-001/006/007/010, SPEC-002/010/011, IF-010/013 to existing capabilities.
No runtime, persistent format, placement authority or readiness contract changes.

Scope: a README-only CI fast path, its synthetic Git fixtures, development notes
and producer evidence; then README prose/diagrams in a separate one-file change.
Keep the existing required secrets and Windows/Linux check names. Keep workflow
triggers unconditional; gate expensive Rust steps inside the matrix jobs.
The existing documentation/source checks and secret scan still run.

Only an ordinary root README.md blob changed between the declared immutable
commits qualifies. PR comparisons use their merge base; main pushes use before
and after. Any other file, rename, deletion, mode change, unknown event, invalid
commit or Git failure selects full validation. Read-only Git calls disable
external diff/text conversion and have finite timeouts. No API file-list limits,
third-party filtering action, asset cache operation or new Cargo dependency.

README.md is mutable product narrative. Remove its current row from the producer
foundation hash inventory, while retaining ordinary-file/link/branding checks,
Git history and every other reviewed target. Future README-only edits therefore
need no companion inventory edit. Formal specs and implementation stay pinned.
Historical review commits retain their original README hashes and proof scope.

Entry: the merged input, clean owned worktree and unchanged protected-main checks.
Exit: real Git fixtures distinguish README-only from code/mixed/unknown changes;
required matrix jobs still report while Rust setup/build/test/legacy checks skip.
L0: structure/links/pins/branding/diff. L1: Python classifier Git fixtures.
L2: hosted required checks for the CI change, then an actual README-only PR.
No Rust implementation changes; local Rust requalification is not required.
Keep implementation, verification, intended-use validation, trace and producer
review separate; roles are checklists, not independent or human acceptance.
