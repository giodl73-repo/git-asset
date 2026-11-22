# Producer-root update review

Input5c4228a0e96aab41cbc225599efebdada5ad1a21; controls, boundaries and exit
profiles are in [the contract](../../specs/NATIVE-PRODUCER-ROOTS.md). Six .roles
producer checklists applied by the implementing maintainer; no independent
reviewer or human approval is represented.

| Role | Severity / finding | Disposition |
| --- | --- | --- |
| Identity | H: media identity, record identity and original producer differ. | Canonical metadata Object, exact row baseline, immutable native recipe, archived member equality and strict plan format. |
| Placement | H: incremental edits do not eliminate full protection costs. | Shared read/write/time meter,32-row cap,64KiB plan, existing namespace closure/history bounds; no lifetime/coverage claim. |
| Custody | H: retained index metadata is not media custody. | Metadata-only Job source and separate member Snapshot; unknown custody/no eviction grant; actual native retained reopen remains mandatory. |
| Recovery | H: competing writers, lost replies and stale originals must not roll back newer roots. | Expected Version/revision, original selection_status, deterministic Select, post-effect receipt preservation and read-only historical recovery. |
| Provider/privacy | H: parsed plans must not hide edits or trigger native work. | Read-only reconstruction of declared upserts and bounded hash/readback; only existing namespace/metadata roots, no native payload API or discovery. |
| Git/adoption | M: operational index growth stays asset-side; trust must not move implicitly. | Existing Tree/metadata/source engines; caller persists exact plan before apply; receiver policy stays explicitly pinned/reenrolled. |

Implementation: producer-root preparation/status/apply and exact archive record
rebinding. Verification: focused locked metadata/race/recovery fixtures, archive
regressions, actual native indexed historical reopen and strict feature/Clippy
checks must be terminal before commit. Validation: historical bytes must reopen
through the updated Record after old ingress release and working repo loss.
Trace: contract -> producer_index::update/history::producer_record -> metadata
and native history fixtures; source/log hashes are external producer-update-*.
Review: producer findings incorporated; independent/owner acceptance remains open.

The initial project case distinguishes an explicitly retained empty index from
an unenrolled/absent source. Repeating its empty update is a no-op. Read-only
historical recovery is also exercised in a separate child process using only
saved plan, profile and checkpoint files, with old index row readback.

Terminal local verification:7 producer-index cases pass, including the inert
worker entry and parent invoking a separate-process original reopen;8 archive
regressions pass. The fake history adapter passes (its native case is explicitly
ignored there), and that actual git-annex case passes when invoked separately.
The authenticated history receiver regression, strict workspace/all-feature/
all-target Clippy, standalone annex-publish and fmt pass. L0/branding/diff and
exact source/log pins are recorded separately before commit. Validation confirms
indexed historical bytes after original ingress retirement and working-repository
loss, while unavailable sibling media stays missing. Independent review, trusted
worker root adoption, full HTTP/scale and owner acceptance remain OPEN.
