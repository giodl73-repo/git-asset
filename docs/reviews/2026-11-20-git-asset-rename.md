# git-asset rename producer review

Input 4ce4da04cd92947a49b8f6c33c0f37ccfe8751b9; branch feat/git-asset-rename.
EVID-123. [Contract](../../specs/GIT-ASSET-RENAME.md) names controls, allowed
scope, entry/exit and L0/L1/L2. Method: producer-applied six-role checklists.
Owner: maintainers. This is not independent review or human acceptance.

## Implementation

Rename the existing public repository in place and preserve visibility/history.
Rename current Rust packages, imports, directories, services, environment names,
UI/session labels, development entry points and authored documentation. Add the
bounded git asset front door, selecting only known sibling executables without
shell dispatch or provider startup. Explain the asset relationship/lifecycle
role over Git and git-annex, including the unqualified native publication work.
Keep local/LFS compatibility and fixture providers; introduce no storage engine.

Preserve schema/hash-domain bytes, stream slots, private namespace handling,
proposal refs and historical artifact builds. New authored records use .asset;
old .tig records and restore partials remain readable. No cache discovery,
migration, runtime installation, transfer, rendering or deletion is performed.
Keep historical reviews in bounded chapters without discarding their evidence.

## Verification

The naming audit compares against the immutable input and preserves 891 frozen
runtime/fixture identifier occurrences. Cargo registry versions/checksums are
unchanged. All-target/all-feature workspace check, warnings-denied Clippy, two
front-door tests, the explicit legacy-record readback test, 82 JavaScript cases
and three Python restore-path cases pass. Standalone and assets-only builds,
fresh real Git external-command discovery and the explicitly admitted native
ordinary/bare annex fixture also pass. Final source/links/hash/diff checks include
an audit that staged Git-tree bytes match the reviewed source hashes.

The first broad run exposed mismatched renamed LFS fixture agent keys: the
selected agent was git-asset but its customtransfer configuration used asset.
Align both download/upload fixture keys with their selected agent; no transfer
implementation, protection or byte checks are relaxed. All six real-client and
thirteen upload cases pass after correction.

The two-thread workspace reached 160 passing native-source cases, one ignored
explicit-artifact case and an existing seventeen-root aged-retirement failure:
Age(Admission(Budget)) at paged_cases.rs:1019. The unchanged case passes alone in
73.41s and in the complete native-source target's one-thread run. That serial
target reached 160 passing cases and one ignored case but failed separately at
age_cases.rs:146 on an explicit clock_held response from the actual enrolled
OS clock. Neither result establishes a particular underlying cause.

The synthetic real-clock fixture now makes at most four fresh CLI attempts after
clock_held, with a five-second retry window checked between calls. Other errors,
final holds and all original state/selection/protection assertions remain.
No production clock policy, floor, admission code or resource budget changes.
The focused corrected fixture passes in 2.75s, and strict all-feature/all-target
Clippy passes afterward. Deterministic clock-rejection tests remain unchanged.

The final local serial native-source target passes all 161 runnable cases, with
one explicitly ignored artifact case. The store library passes 434 cases with
nine ignored explicit-artifact cases. The later fences integration helper
deadlocked: single-thread libtest prefixes its READY line with the test name,
while the parent expects a standalone signal. Stop only the verified owned
hung parent/child and continue the no-fail-fast run. Add --quiet to that child
harness; all eleven corrected fence cases pass in 0.51s, and strict Clippy
passes afterward. This changes test stdout framing only, not production locks
or assertions. Retain the interrupted original local target as a failure; do
not label that original complete command an uninterrupted passing run.

Final complete workspace and exact-head hosted Windows/Linux/secrets
outcomes are recorded on [PR 61](https://github.com/giodl73-repo/git-asset/pull/61).
They are publication gates separate from this producer review; do not infer their
status from a local implementation commit. The CI profile requires the exact heavy seventeen-root case alone first,
checks that it exists exactly once, and stops on failure. The four-thread full
workspace then omits only that already-qualified case. Every routine test and
explicit concurrency actor still runs. This avoids serializing unrelated cases
while retaining the original budgets and assertions. Earlier qualification of
the aged case under inter-test load remains open; no throughput or universal
clock-availability claim follows. Both phases are required before PR readiness.

## Intended-use validation

The team overview and README identify Git decisions/history, git-annex byte
custody, and asset relationships/selection/obligations as separate concerns.
Front-door discovery names git-annex but explicitly reports native publication
as unqualified. Legacy readback proves a committed .tig record resolves to the
same immutable record Object without rewriting its bytes. These are synthetic
fixtures, not production cache migration or full consumer workflow acceptance.

## Trace

The WP/REQ/SPEC/IF/PKG controls in GIT-ASSET-RENAME.md trace to renamed package
boundaries, source/native compatibility, front-door discovery, check_branding.py,
TEAM-OVERVIEW.md and COMPATIBILITY.md, then to EVID-123. Requirements and native
retained-publication/provider acceptance remain open beyond this rename scope.

## Producer lenses and dispositions

- Identity: source names change; exact persisted identities do not. The immutable
  baseline audit and legacy read fixture address accidental rekeying. Resolved.
- Placement: desired/planned/observed separation and current protection rules
  remain. No policy or drive assignment is changed. Resolved within rename scope.
- Custody: branding is not retention or permission to remove bytes. Native
  publication and participating-pruner work remain owned by maintainers. Open
  product work, explicitly excluded from this rename's completion claim.
- Concurrency/recovery: no journal/budget/fencing behavior changes; new partial
  names also recognize the old prefix. The initial two-thread rerun
  also hit Age(Admission(Budget)); the unchanged case passes alone in 73.41s.
  Require both CI phases before closure; real concurrent actor fixtures and
  production budgets are unchanged. Qualification of the heavy aged case under
  inter-test load remains open. Scheduling sensitivity remains an inference.
- Provider/privacy: sibling allowlist rejects traversal and shell metacharacters;
  no private archive, assets or cache binding enters this public source. Resolved.
- Git/adoption: historical CI artifacts retain their actual old paths/names;
  current packages and front door use git-asset. Old workflow trust strings and
  consumer pins require explicit consumer updates; existing feature PRs need
  rebasing. Maintainers own those follow-ups; no external deployment is claimed.

Source naming/identity review is complete, with the runtime qualification gates
and remaining risks recorded separately above. Independent review, human
acceptance and complete native lifecycle qualification remain open.
