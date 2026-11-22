# git-asset resume

Goal: PRODUCT_PLAN.md PR70/P1-P6. Synthetic fixtures only; no production IO on
resume. Preserve other worktrees. Component checks are not release/project
acceptance; keep the full goal active.

Active owned lane: F:/src/git-asset-worktrees/trusted-native-placement,
feat/trusted-native-placement, draft PR74 based on feat/source-generation-use.
Input352f17cd3ce1c8339475d4a392d1bf22672720e9; derive newer source/CI from Git.
PR70 and P1/PR71 merged. P2 PR72 and source-generation prerequisite PR73 remain
stack dependencies; P2 mixed legacy/owner/roster/epoch handoff is OPEN, never
inferred from NamespaceOwned policy. See docs/reviews/p2-closure-audit.md.
Native controller qualification is component-only; no legacy migration authority.

P3 implemented: exact Git/HTTP annex metadata source, hold_current generation,
authenticated demanded native copy, temporary ingest handoff to exact retained
snapshots/cohorts. Strict original reopen, actual roster-root generation,
newer-version/job protection and original recovery remain mandatory. Controls:
specs/NATIVE-SNAPSHOT-COPY.md and its prerequisite contracts.

Retained-snapshot copy/restore now shares Publisher::open_retained with handoff,
uses cumulative metadata budgets and preserves original copy wire hashes. Explicit
Plan::from_snapshot works through the authenticated copy worker. Next: connect
trusted handoff/eviction/receiver, exact per-object producer lookup,
revert/reacquisition and duplicate/missed/outage/crash qualification. No global
convergence from one copy or stale event. No new publisher/cache/journal.

P4: REEL main-based PR401, qualified89a875c6c34ed84b1e890520db996d55cffe2bd7,
ready for review. SDK pin0ccb137cf5a78416e780f8a3372017f642a5cb71 unchanged;
Isolated public route/lifecycle/retention and nine child exits qualified;
existing route/vendor untouched. Consumer snapshot access must qualify before releasing a
claim that its old Ready recovery still needs. No production adoption implied.
P5 pinned Windows RC/limits/recovery/upgrade/rollback and P6 private per-project
current/history coverage, backup/restore and owner acceptance remain OPEN.

External evidence: F:/src/tig-dev-temp/git-asset-protected-eviction-20261121.
Use *-source.json and logs, not narrative as terminal proof. Copy head8124cd4
passed hosted45012037613; handoff352f17c passed hosted45014039566.
Latest snapshot-copy:37 focused tests (includes inert worker), five explicit
real-annex cases pass; shared-reader Clippy/feature/L0 gates recorded externally.
Independent review and full P3 acceptance separate. Development Cargo/TEMP roots remain external;
see docs/DEVELOPMENT.md. No budget increase or fixture removal.

Prior detailed component/source progress remains in [immutable resume](https://github.com/giodl73-repo/git-asset/blob/352f17cd3ce1c8339475d4a392d1bf22672720e9/context/RESUME.md).
LF SHA-256: `354164d5498b0fa90251245b91f6c871c1190eecfb11aadcdce86cb1a58862c5`.
