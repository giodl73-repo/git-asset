# Review the git-asset catalogue

Use a fresh synthetic sample for a repeatable review. Configure external build
and temporary roots as described in [development setup](../DEVELOPMENT.md).
`NEW_SAMPLE` must be a new absolute directory outside every source checkout;
its parent must already exist. No production enrollment or media is needed.

```text
cargo build -p git-asset --features schema --bin git-asset-sample --bin git-asset-catalogue --locked
git-asset-sample rehearse-ordinary NEW_SAMPLE
git-asset-catalogue check NEW_SAMPLE/catalogue.json
git-asset-catalogue serve NEW_SAMPLE/catalogue.json 8801
```

Open `http://127.0.0.1:8801` on the same computer. Stop the serving process to
stop this explicitly launched preview. The seven-step sample uses tiny synthetic
bytes, real Git commits and local fixture providers; it is not native-annex
production acceptance. Its cache labels are symbolic, configurable policy roles,
not permission to operate the machine's actual drives.

## Five-minute walkthrough

1. Open **Series study**. Browse its source record, versions and related files.
   Confirm that work counts describe works rather than every rendition/record.
2. Close the inspector with Escape, its close button or the surrounding backdrop.
   Focus should return to the original card. Tab and Shift+Tab should stay inside
   an open inspector, including after switching sections or opening a source.
3. Open **Objects · placement & versions**. Compare content size, observations
   and primary-cache preference. **Not observed** means coverage is unknown;
   zero observed bytes is not proof that no copy exists. Review controls produce
   held dry runs; they do not move or delete assets.
4. Open **Learn git-asset · sample lessons**. Compare **release v1**, **draft v2**,
   **merge v2 job active**, **job ended**, **revert v1** and **return v2**. The tables
   separate before, desired and recorded observed placement. Ending an old job
   changes its protection independently of merging or closing a branch. The exact
   Git checkpoint and recorded evidence explain each transition.
5. Return to **Inspect sample catalogue**. Select another explicitly listed
   snapshot, then refresh it. Verify that historical inspection does not quietly
   retarget to latest. Refresh redraws the selected snapshot; it does not mean
   pull, hydrate, rebuild or evict.
6. Narrow the window. **Browse** exposes projects, collections, storage, lessons,
   coverage, shared assets and technical records when applicable. Selecting a
   view collapses that menu. Narrow inspectors use the full width; wide tables
   scroll within their own region rather than widening the page.

The ordinary sample has one project. **Shared assets** requires two different
enrolled projects and should explain that requirement. Cross-project comparison,
duplicate titles, stale replies, partial coverage and failed requests also have
shipped synthetic controller fixtures; they do not establish private-project or
screen-reader acceptance.

## Review boundaries

This is a metadata/library and recorded-lifecycle review. Media previews/playback,
interactive enrollment, authoring-tool handoff and native copy/eviction execution
are separate work. A green status, selected record, observed copy, candidate or
historical receipt does not grant creative approval or independent backup custody.
Report the view, exact snapshot, steps and expected/actual behavior. Keep private
screenshots, enrollment paths, prompts and media out of public issues/source.

The [UI plan](../CATALOGUE-UI-PLAN.md), [catalogue contract](../../specs/CATALOGUE.md)
and [review scope](../../specs/CATALOGUE-REVIEW-UX.md) distinguish implemented,
verified and still-open capabilities.
