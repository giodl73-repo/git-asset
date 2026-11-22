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

## Enrolled head-consumer review

Inputffccf8cb206077908a3d667863acb5737189af65; same six producer checklists and
implementing-maintainer ownership. H identity: asset commits stay immutable while
the two explicitly enrolled index sources are mutable; each pass captures exact
Versions and binds them to continuation. H recovery: both live generations remain
held through effects; readers drop on exit without withdrawing durable history;
old cursor/unknown head refuses rather than falling back. H custody: head/record
availability never establishes native bytes or independent keeper custody.
H provider/privacy: fixed signed namespace/provider/owner/source, separate from
native pools; pure policy validation opens nothing; report pins omit paths.
H placement: original shared lookup and namespace/source bounds remain; no census
or global lock. M Git/adoption: optional absent fields preserve legacy wire/basis,
explicit enrollment replaces per-edit reenrollment, and no request gains an index
writer or native stale-plan replacement authority. Independent/human review is open.

Qualification targets: signed immutable Git source plus retained/working index
heads; metadata update without changing enrollment; changed-head cursor refusal;
blocked writer during copy and successful update after exit; missing/wrong provider
without fallback; exact copied/present recipe retry; real registered eviction with
late protection, expiry and saved original replay using head enrollment. Full
transport/account/TLS, orchestration and production adoption remain separate.

Head-consumer terminal local verification:11 default worker cases pass, with7
explicit native cases ignored in that default run;7 producer-index and2 actual
pending-action metadata cases pass (child entries and separate-process parents
retain their stated scope).8 GitHub/authentication/CLI-discovery cases pass.
Two explicitly invoked actual git-annex cases pass: enrolled-head copy/update/
writer fencing/present retry and distinct-volume registered eviction with late
protection, expiry and saved replay. Strict workspace/all-feature/all-target
Clippy, standalone annex-publish and fmt pass. Validation separates exact producer
pins from byte custody and demonstrates no policy reenrollment per index edit.
Trace/source/log pins use producer-heads-source.json externally; L0/branding/diff
close separately before commit. Review remains producer-applied; full HTTP,
publication orchestration, scale and owner adoption remain OPEN.

## Completed-recipe refinement

Inputcb798cbc9993cbb8b6649476e6fe7c9ae66c7154; same six producer lenses and owner.
H identity: record original and completed copy-context recipe separately; normalized
origin equality prevents candidate/request/control changes. H recovery: original
intents remain checked; an installed recipe is write-once through preparation and
parsed-plan apply, and cannot be hidden by metadata replacement. H custody: exact
historical completion/accounting is not current byte presence; fresh native reads
and keeper proofs remain mandatory. H placement: at most two declared plus one
prospective recipe, shared existing lookup/status bounds and32KiB record ceiling;
no census or budget increase. H provider/privacy: only fixed original witness,
ledger and native publisher ports; no event-supplied recipe writer. M Git/adoption:
absent field preserves old bytes and index roots stay immutable/retained. Producer
review only; full owner workflow, HTTP/scale and independent acceptance stay open.

Qualification targets: reject foreign/uncompleted/unsettled/uncertain recipes and
replacement; preserve legacy serialization; root registration before source rebind;
original present reuse without new allocation; indexed reverse-backup read after
both temporary ingests retire and the working repository becomes unavailable.

Completed-recipe terminal local verification:12 default worker cases pass, with8
explicit native cases ignored;8 producer-index and8 archive cases pass. Three
explicit actual git-annex cases pass: copy registration/rebind reuse without new
allocation; recorded reverse-backup archive read after both ingests retire and
working-repository loss; distinct-volume eviction using a recorded publication
that differs from its prospective context, including expiry and saved replay.
Strict workspace/all-feature/all-target Clippy, standalone annex-publish and fmt
pass. Validation preserves original identity, legacy absent-field bytes, exact
completed recipes and fresh native checks; metadata does not assert custody.
L0/pins/branding/diff close separately. Source/log/test-binary hashes are external
producer-recipe-source.json. Review is producer-applied; full orchestration,
repair/reacquisition, transport/scale and owner acceptance remain OPEN.
