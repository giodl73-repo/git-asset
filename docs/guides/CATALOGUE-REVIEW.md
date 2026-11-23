# Review the git-asset catalogue

Use a fresh synthetic sample for a repeatable review. Configure external build
and temporary roots as described in [development setup](../DEVELOPMENT.md).
`NEW_SAMPLE` must be a new absolute directory outside every source checkout;
its parent must already exist. No production enrollment or media is needed.

```text
cargo run -p git-asset --features schema --bin git-asset-sample --locked -- rehearse-review NEW_SAMPLE
cargo run -p git-asset --features schema --bin git-asset-catalogue --locked -- check NEW_SAMPLE/catalogue.json
cargo run -p git-asset --features schema --bin git-asset-catalogue --locked -- serve NEW_SAMPLE/catalogue.json 8801
```

Installed service binaries also accept `git asset sample rehearse-review NEW_SAMPLE`.
Generation runs the existing 7- and 11-checkpoint rehearsals and can take a minute
or two. Progress labels go to stderr; one bounded JSON result goes to stdout.
The new workspace opens at **All projects**, with independent intro-v2 and
recovery-v1 projects, one shared selected 25-byte object and eleven recorded
recovery lessons. Each original catalogue/trace remains in its own subdirectory;
`review.json` and the child result receipts pin completion. No viewer starts
automatically. A failed/interrupted root is preserved; retry in a new root.

Keep `review.json`, `intro-result.json` and `recovery-result.json` with the
workspace: the review report pins the child receipts and combined catalogue by
hash and byte length. It records one shared selected object, 25 bytes, eleven
recovery checkpoints and `viewer_started: false`. Stop the server with Ctrl+C.


Open `http://127.0.0.1:8801` on the same computer. Stop the serving process to
stop this explicitly launched preview. The sample uses tiny synthetic
bytes, real Git commits and local fixture providers; it is not native-annex
production acceptance. Its cache labels are symbolic, configurable policy roles,
not permission to operate the machine's actual drives.

## Five-minute walkthrough

1. Open **Intro sample · current v2**, then **Series study**. Browse its source record, versions and related files.
   Confirm that work counts describe works rather than every rendition/record.
2. Close the inspector with Escape, its close button or the surrounding backdrop.
   Focus should return to the original card. Tab and Shift+Tab should stay inside
   an open inspector, including after switching sections or opening a source.
3. Open **Objects · placement & versions**. Compare content size, observations
   and primary-cache preference. **Refresh observed copies** explicitly checks
   the registered synthetic scope. **Not observed** means coverage is unknown;
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

**Shared assets** compares the intro and recovery projects and finds their shared
25-byte reference; their selected series versions differ. The original
`rehearse-ordinary` still creates just one project and comparison must explain
the need for two. Cross-project comparison,
duplicate titles, stale replies, partial coverage and failed requests also have
shipped synthetic controller fixtures; they do not establish private-project or
screen-reader acceptance.

## Review the later advances too

The review command includes the introduction and return/restoration course. Other
teaching modes are already in the same source; each needs its own new absolute
output root. Use the resulting catalogue.json with the same check/serve commands.
`git-asset-sample describe` lists operations supported by the compiled features.

| Sample operation | Additional review scope |
| --- | --- |
| `rehearse-exchange` | Push/fetch/pull, local metadata exchange and exact selection |
| `rehearse-worktree` | Linked worktree/index changes and independent active-job protection |
| `rehearse-backup-outage` | Missing backup observations and held convergence |
| `rehearse-stale-plan` | Source changes invalidate an old plan |
| `rehearse-rename` | Logical rename with retained byte identity |
| `rehearse-return-restore` | Eleven checkpoints: final loss, historical Git return, missing bytes, explicit reacquisition and verified working restoration |
| `rehearse-aged-category` | Explicit aged-category block and separate final-loss authority |
| `rehearse-remote-publication` | Bounded loopback upload/readback/uncertain observation; requires `schema,upload` and its fixed sibling tools |

See the [full sample course](../../examples/catalogue/README.md) for each mode's
limits, consent, simulated clock and provider boundary. Advanced synthetic
deletion/publication lessons are not permission to operate live caches. No single
enrollment or build automatically enables every optional provider or experiment.

## Review a cache for eviction

1. In **Objects**, select an observed cache such as the sample's working cache and choose **Review this cache for eviction**. The review uses recorded observations across all enrolled projects.
2. Read the decision and holds for each exact hash. **Keep: current declared use** identifies a current reference. **Check archive copy** identifies an older copy with a matching listing elsewhere. **Unknown: keep for now** means the recorded evidence cannot support a candidate decision. Use **Inspect references** to see the declared consumers.
3. For a candidate, choose **Verify bytes at …** to read the listed archive copy and verify its hash and length. A failed read leaves the copy unverified. A successful read establishes byte integrity at that check. Independent retained custody still requires its own proof.
4. Check the reclaim estimate before planning capacity. If other hard links exist, removing one path is estimated to free zero bytes. With insufficient physical evidence, reclaim remains unknown. Use **Next review page** to continue on the same evidence basis. If that basis changes, start the review again.

The viewer does not authorize or perform eviction. See the [native eviction
guide](NATIVE-EVICTION.md) for explicitly enrolled participating pools and the
requirements for protected execution.

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
