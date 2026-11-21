# Catalogue UX review readiness

EVID-143; input96da4503aea1b290fd1cefcdad621d607ec0f52a;
[controls](../../specs/CATALOGUE-REVIEW-UX.md). Producer-run review, not independent
or human approval. Uses a freshly generated public seven-checkpoint sample only.

## Implementation

One inspector entry helper preserves the original invoking control across nested
work/source/object navigation, focuses Close and makes the background inert.
The keyboard loop excludes hidden/disabled controls, contains forward/backward
Tab and returns focus on dismissal; a removed opener falls back to the heading.
Narrow-screen Browse exposes collections and technical records previously hidden
by CSS. View selection collapses that menu; cards become single-column and wide
tables scroll locally. Controls/metadata have larger type, stronger text contrast
and clearer responsive wrapping. Current g. branding and a same-origin bundled
SVG icon replace the old mark and failed favicon request. CSP stays unchanged.

## Verification

- All90 shipped controller cases pass, including five new keyboard/menu cases.
  New cases are included in both hosted JavaScript check paths.
- All13 actual catalogue integration tests and the hostile HTTP/static-icon
  regression pass; strict workspace all-feature/all-target Clippy passes.
- Actual browser: work Overview/Versions/Files, nested source Impact entry,
  object block review, lessons, exact historical selection/refresh, shared-assets
  one-project refusal, compact navigation, inert background, forward/backward
  Tab and Escape/return all exercised on the rebuilt application.
- Actual CSS viewport widths320,390,614,1024 and1440 have no page-wide horizontal
  overflow; wide lesson tables remain in scroll containers. Browser/OS scaling
  is measured via innerWidth, not assumed from the requested outer window size.
- Final browser session has zero console errors/warnings. Earlier favicon data
  URI violated the existing CSP; replaced with the fixed same-origin resource,
  not a relaxed policy. Before/after images and runtime/log hashes stay external.

Hosted exact-head checks, independent accessibility/screen-reader acceptance and
team review remain separate. No claim of comprehensive accessibility compliance.

## Intended-use validation

The saved merge checkpoint keeps original and fixed bytes near while the old
job lives. The following job-ended checkpoint shows original moving from Working/F
to History/D with Backup/M retained; fixed stays Working/F plus Backup/M.
The UI distinguishes before, desired and recorded observations. These are
symbolic synthetic roles, not verification of current disks or hardware custody.

Object review reports verified reclaim as Unknown, exposes missing consumer/job
coverage and explicitly states that no executor or movement/deletion occurred.
Historical refresh preserves the selected old commit. A one-project shared-assets
comparison explains the need for two projects rather than claiming no sharing.
The [walkthrough](../guides/CATALOGUE-REVIEW.md) makes these review tasks repeatable.

## Trace and producer lenses

WP-015 / REQ-006/007/010 / SPEC-014 / IF-010/011/013 / PKG-004/006/008
map to bundled HTML/CSS/controllers, the fixed HTTP icon route, keyboard/controller
fixtures, native catalogue/HTTP tests and browser tasks above.

| Lens / finding | Disposition / remaining scope |
| --- | --- |
| Identity: history must not silently become latest | Existing snapshot fences retained; old-commit refresh exercised |
| Placement/custody: observed zero or a selected block must not imply safe reclaim | Unknown/held explanations verified; no executor or new storage authority |
| Recovery: modal dismissal must not admit a late response | Existing response tokens retained; all stale/race regressions pass |
| Privacy/provider: visual review must not import project data or relax HTTP policy | Public synthetic fixture only, external images/logs, existing CSP/auth unchanged |
| Adoption: small DOM double is not browser acceptance | Actual keyboard/layout tasks recorded separately; team, assistive-technology and production acceptance remain open |

Owner: git-asset maintainers. Media previews/playback, interactive enrollment,
authoring-tool handoff, real-project adoption and native eviction remain OPEN.
