# Historical-root producer review

Input b02d5ad7e5a373bc672e528d52589162a3d9ad74; controls and scope in
[contract](../../specs/HISTORICAL-ROOT-ARCHIVE.md). Six .roles producer checklists,
owned by the implementing maintainer; no independent review or human approval.

| Role | Finding / severity | Disposition and proof boundary |
| --- | --- | --- |
| Identity | H: history must preserve the original Version/tree/context. | Canonical saved plan, unchanged node/root hashes, exact member lookup, separate-process reopen; no inferred Git commit or latest fallback. |
| Placement | H: metadata archive is not observed payload coverage. | Explicit unknown custody/no eviction authority. Node/record budgets are cumulative; physical eviction remains separately admitted. |
| Custody | H: retiring the old obligation must not erase its only definition. | Complete retained-tree finish/readback plus hard media/metadata claims precede working release; actual media stays subject to native keeper checks. |
| Recovery | H: interrupted adoption and Main races need original handles. | Deterministic adoption, source-generation guards, revision CAS; explicit monotonic refresh keeps older saved definitions and the same archived tree. No uncertain native-plan replacement. |
| Provider/privacy | M: avoid filesystem census and app-specific payload processing. | Only exact tree nodes and record blobs traverse NodeStore; point member API opens no payload; caller-owned fixed provider/namespace identities. |
| Git/adoption | M: source/acceptance bounds must remain honest. | Generic Rust metadata facade over existing engines; controlled8.5MiB source-only allowance, runtime caps unchanged; receiver/full project acceptance remains OPEN. |

Implementation, verification, validation, trace and review are separate. Exact
source and terminal checks belong to external history-* evidence. Portable cases
exercise denied/corrupt/bounded input, adoption interruption, empty-enrollment
restart, Main-advance refusal/refresh, older-plan reopen and scoped unrelated
progress. The native case composes archive/member access with original producer
retained readback after ingress release and an unavailable working repository.
These scenarios do not qualify all project assets or full receiver scheduling.

Verification:8 metadata cases pass (including the inert child entry and the parent
that invokes its separate-process reopen), plus the fake native adapter case and
one explicitly invoked real git-annex case. Strict workspace/all-feature/all-target
Clippy, standalone annex-publish and fmt pass. L0, branding and diff checks pass.
Validation:synthetic source-advance refusal/refresh preserves old record access;
partial enrollment resumes, exact native member readback works with the old repo
offline, working bytes survive, and unavailable siblings stay unavailable.
Trace:contract controls/source seams and reviewed target pins are recorded;
exact committed source/log hashes are external history-source.json.
Review:producer findings incorporated. Independent review, receiver integration,
full HTTP/scale and owner acceptance remain OPEN.

## Receiver refinement producer review

Input2f4f5fce48f34e4e96d21bae6352cb38447248c7; same six producer checklists and
implementing-maintainer ownership. H identity: cursor binds current Version and
immutable commit, not a revision changed by its own release. Removed keys require
positive original record/receipt; arbitrary unknown keys refuse. H recovery:
post-release expiry keeps successful receipts and advances only that completed
archive's continuation; renewal uses a newly verified request, never clock rewind.
H custody: metadata-only transport declares no payload coverage/removal right.
H provider/privacy: fixed signed profile and separate metadata namespace; dispatch
returns before payload/witness/capacity opens. H placement:32-root ceiling,
outer deadline and explicit per-root metadata bounds; no claim that stream-byte
budget meters metadata. M Git/adoption: old absent optional fields retain policy
identity, preview can reconcile exact Main, and full HTTP/production acceptance
remains distinct. Source/trace/pins and final checks are separate from this review.

Fixture: real synthetic Git advances through three immutable selections; native
repositories remain offline during preview/apply. Preview preserves archive
namespaces/metadata, unsigned policy and wrong source refuse, unknown cursor
refuses, expired post-release receipt survives, renewed admission resumes the
removed cursor entry, both historical records reopen, duplicate suffix is empty
and newer Git Main rejects the old request. Library/native media qualification
from the preceding SDK slice remains scoped to those unchanged mechanisms.

Receiver verification:8 archive tests,10 default worker tests and8 GitHub/CLI
cases pass;6 explicit native worker cases are skipped, not counted as successes.
The separate real git-annex historical-member reopen case passes on this source.
Strict workspace/all-feature/all-target Clippy, standalone annex-publish and fmt
pass. Validation: exact historical receipt continuation survives expiry/renewal
with native configurations offline; no media reads or removal are admitted by the
receiver history phase. Trace: source controls and reviewed targets map the public
API/configuration to these fixtures; final L0/source/log pins are external
history-receiver-* evidence. Review: producer-applied only; hosted current-head,
full native HTTP/account/TLS and independent/owner acceptance remain separate.
