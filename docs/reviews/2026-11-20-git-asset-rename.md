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
and three Python restore-path cases pass. Final broad Rust, feature-profile,
actual native fixture and L0 outcomes are recorded below when complete.
The first broad run exposed mismatched renamed LFS fixture agent keys: the
selected agent was git-asset but its customtransfer configuration used asset.
Align both download/upload fixture keys with their selected agent; no transfer
implementation, protection or byte checks are relaxed. All six real-client and thirteen upload cases pass after correction.
The two-thread workspace then reached 160 passing native-source cases, one
ignored explicit-artifact case and a failure in the existing seventeen-root
aged-retirement case: Age(Admission(Budget)) at paged_cases.rs:1019. The unchanged
case passes alone in 73.41s. A complete one-thread workspace rerun is required
before closure; no filters, assertions or production budgets are changed. The
parallel failures remain open and do not prove a particular underlying cause.
Hosted exact-head Windows/Linux/secrets checks remain a separate publication gate.

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
  Require the complete one-thread harness before closure; real concurrent actor
  fixtures and production budgets are unchanged. Parallel qualification remains
  open. Scheduling sensitivity is an inference, not a confirmed root cause.
- Provider/privacy: sibling allowlist rejects traversal and shell metacharacters;
  no private archive, assets or cache binding enters this public source. Resolved.
- Git/adoption: historical CI artifacts retain their actual old paths/names;
  current packages and front door use git-asset. Old workflow trust strings and
  consumer pins require explicit consumer updates; existing feature PRs need
  rebasing. Maintainers own those follow-ups; no external deployment is claimed.

Source naming/identity review is complete; broad runtime qualification remains
open. The serial run also reports a failure in aged_native_cli_holds_unknown_
and_recent_then_emits_only_exact_proven_block; await its detailed output and
retain both failures rather than assuming a scheduling diagnosis. Publish only
a draft until local findings and required exact-head hosted checks are settled.
Independent review, human acceptance and full native lifecycle qualification
remain open.
