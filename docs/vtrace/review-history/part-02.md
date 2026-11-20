# Producer review history

Closed capacity reservation producer review: entry84136b; exact four source/test
files plus CAPACITY-RETIREMENT.md. Six roles are producer-applied checklists,
not independent reviewers. Identity: old tokens stay generation-fenced; a
retirement-operation namespace collision refuses and preserves the old record.
Placement: retirement cannot refund unobserved allocation debits or mutate a
sample; the debit map keeps its existing-size bound. Storage/custody: immutable
historic dispositions and all old journal bytes remain; no physical data is
removed. Concurrency/recovery: model changes adopt only with SQL commit, actual
child exits cover both commit sides, peer tails synchronize and retirement
invalidates handle freshness. Provider/privacy: all fixtures are synthetic
private temporary roots, with no real inventory or implicit probes. Git/adoption:
new canonical commands are explicit reader-version boundaries; unchanged old
prefixes and replayed outcomes do not grant new IO. Disposition pass_with_risk:
namespace collisions fail closed, journal limits/compaction and external latest-
witness authority remain separate constraints. Consumer rollout is not this
component's validation or proof of complete artifact retention.

EVID-083 producer review (input0ceade): identity verifies exact snapshots/all nodes;
placement requires explicit distinct providers; storage retains source and failed
target; concurrency refuses drift without long transactions; privacy uses only
synthetic fixtures and redacts summaries; adoption stays caller-owned.
Pass_with_risk: partial/publication failure leaves an unselected target. No
independent/human approval, capacity reservation, custody or hosted qualification.

Agent manifest-query producer review (input0f292ba): identity, provider/privacy,
Git/adoption and concurrency lenses apply to explicit work-scoped named JSON
reads, document provenance and source/content paging fences. See the exact scope
and fixture evidence in [the review](../../reviews/2026-11-07-manifest-queries.md).
Disposition pass_with_risk: associations do not establish semantic consumers,
full project coverage or eviction authority. No independent/human acceptance.

Manifest impact producer review (input954af65): identity, provider/privacy,
Git/adoption and concurrency lenses cover explicit association evidence joined
to one admitted native graph. See [scope and fixtures](../../reviews/2026-11-07-manifest-impact.md).
Pass_with_risk: potential consumers remain conditional on owner-record changes;
retained associations do not become production edges. Exact old pins and missing
coverage stay distinct. No independent/human acceptance or retirement authority.

Catalogue impact producer review (input520531b): identity, privacy/provider,
Git/adoption and concurrency lenses applied to a metadata-only authenticated
impact endpoint and inspector controls. Exact selected navigation, explicit kinds,
source-fenced pages, stale/late reply handling and unknown coverage are covered
by [scoped fixtures and review](../../reviews/2026-11-08-catalogue-impact.md).
Pass_with_risk; visual/browser qualification and complete project/job/custody
coverage remain open. No independent/human acceptance or eviction authority.

Integrated UX fixture producer review (inputf072d1b): identity, privacy/provider,
adoption and concurrency lenses cover shipped-script controller flows, stable
identities, exact snapshot/progress, sample disclosures and four stale-response
guards. See [scope, reproductions and evidence](../../reviews/2026-11-08-fixture-ux-flows.md).
Pass_with_risk: synthetic DOM/fetch fixtures supplement native API checks and
cannot establish browser layout, live-project or custody acceptance.

Catalogue record-comparison producer review (input4129764): identity, privacy/
provider, adoption and concurrency lenses cover two exact source snapshots,
agent-operation parity, an ephemeral baseline, independent comparison facts and
stale-response fences. See [scope and fixtures](../../reviews/2026-11-08-catalogue-record-compare.md).
Pass_with_risk: no historical resolver, retention claim, decoded equivalence,
complete source/job/custody or independent browser acceptance is established.

Object-to-record producer review (input2894138): identity, privacy/provider,
adoption and concurrency lenses cover explicit declaring-owner shortcuts,
captured source/scope, return navigation and object/review request fencing.
See [scope, regression and fixture evidence](../../reviews/2026-11-08-object-record-workflows.md).
Pass_with_risk: retained/nested byte associations are not production dependencies;
browser, complete coverage and custody acceptance remain open. No independent
review or eviction authority is established.

Record-inspector producer review (input5710677): four producer lenses cover
same-card request isolation, exact response sources, pending action clearing and
dependency render isolation. See [scope and regression evidence](../../reviews/2026-11-08-record-inspector-fences.md).
Pass_with_risk: fixtures check shipped controllers and native API integration;
browser, live-project, complete coverage and custody acceptance remain open.

Storage roadmap producer review (inputf742abf): DOCS profile, WP-003/004/005/
006/007/008/009/011/012/013/015/016, applicable source/storage/catalogue
contracts. Scope: PRODUCT_PLAN.md next-delivery sequence, current-fixture matrix
and proposed local/provider lifecycle fixture. Identity lens requires immutable
objects and exact source/policy basis; provider/privacy requires explicit scopes,
capabilities and isolated fixtures; adoption reuses existing Rust mechanisms;
concurrency requires scoped revalidation, reader/claim races and restart recovery.
No blocking design finding within this planning scope; owner: git-asset maintainer.
Pass_with_risk: this is producer review of a plan, not implementation approval,
independent review or live operation authority. Runtime exits remain unproven.
L0 source/hash/link/diff checks required; L1/L2 not applicable to prose changes.
Existing replica and retention-handoff tests were rerun to verify the inventory
of component evidence; their 20 and 16 entries pass, including helper entries.
All six real Git LFS client cases also pass in temporary repositories. No cloud
upload, final-payload eviction or complete orchestration proof follows.

Six-role roadmap follow-up (input44b7dab): all parliament checklists applied to
the storage sequence and self-contained development startup. Five high-severity
design omissions and one medium adoption omission are corrected in the plan;
see [findings, dispositions and remaining proof](../../reviews/2026-11-08-storage-roadmap-roles.md).
Pass_with_risk for revised prose only. Executor/remote/eviction proof remains
open; no independent review, live operation or runtime deployment is authorized.

Lifecycle feedback follow-up (input43adabd): DOCS profile; WP-003/004/006/007/
008/009/013/015. The roadmap and catalogue/resume plans now cover policy-enabled
merge reconciliation/eviction, finite backup retention, bulk final deletion and
eventual metadata retirement. All six producer lenses additionally require
cross-store custody fencing, coverage invalidation, authorized-absence replay,
large-object qualification and journal lifetime. See [LF-01..06 dispositions and
planned proof](../../reviews/2026-11-08-storage-roadmap-roles.md). Pass_with_risk for
prose only. L0 structure/hash/link/diff checks apply; L1/L2 are not runtime claims.
Implementation, verification, validation and trace remain open separately; no
independent or human acceptance, live deletion or deployment follows.

Second six-role roadmap pass (input7979190): rechecked all .roles against exact
source, model/safety/source-journal contracts and foundation cases. RR-01..06
correct same-hash re-ingest fencing, owned historical-claim settlement, truthful
provider deletion scope, pre-removal evidence budget, trusted Action/policy
admission and lifetime/audit sequencing. See [findings and remaining gates](../../reviews/2026-11-08-storage-roadmap-roles.md).
Producer disposition: pass_with_risk for planning only; runtime implementation,
verification, validation and trace closure remain separate and open. No
independent review or live-operation approval. DOCS L0 required; no L1/L2 claim.

Storage orchestration implementation review, inputbb51c5a: WP-003/004/006/007/
008; REQ-001/002/003/004/005/007/009; SPEC-002/003/004/005/008/009;
IF-002/003/004/005/006/012; PKG-002/003, plus PKG-008 CLI adapter.
Producer-applied six lenses: identity requires canonical exact request/context
binding; policy exposes unknown protection/removal holds; custody preserves source
and verifies retained destination; recovery uses existing journal commands and
fresh status; provider/privacy limits local process-restart and redacts paths;
adoption preserves legacy None context bytes while old readers reject new fields.
During implementation, review caught cleanup validation losing the new context;
corrected to preserve it. Resume uses destination recovery without requiring a
new source byte read when publication is already complete. Existing provider
power-loss, hostile path mutation, global coverage and removal guarantees are not
expanded. Pass_with_risk for bounded implementation: focused fixtures, default/
all-feature regressions, strict Clippy, fmt, standalone Git and L0 checks pass;
no independent review, consumer acceptance or full WP closure. See
[contract and required tests](../../../specs/STORAGE-ORCHESTRATION.md).

Protection-demand snapshot review, inputd18afba: WP-004/006/008/009;
REQ-001/002/004/005/007/009; SPEC-002/004/005/006/008;
IF-002/004/005/006/008; PKG-001/003. One producer applied all six .roles lenses
against model/safety contracts and foundation T01/02/03/08/11/12/15/17/20/24/25.
Owner: git-asset maintainer. Scope is exact observed demand; no removal implementation.

| Lens | Finding / disposition | Remaining proof |
| --- | --- | --- |
| Identity | Bind profile, source owners/kinds, root interpretation, all journal checkpoints and shared demand provenance; normalized ordering and hash/size conflicts checked. | Frozen admission capabilities and stale-operation fencing before removal. |
| Placement | Selected source queries could omit other registered jobs/branches. Added bounded read-only observe_all and expected/actual comparison; all root roles contribute. | Enforced registration for new sources and policy/constraint adoption. |
| Custody | Resolved metadata is not byte custody. Summary and View always deny complete protection/removal; local replica grants remain provider obligations. | Independent retained copies and effective destructive fences. |
| Concurrency | Metadata IO cannot hold a SQL transaction. Close observation first, then recheck heads; mutation during NodeStore reads becomes a gap. | A recheck is not a lease; effective admission/removal guard still required. |
| Provider/privacy | Opaque/missing/corrupt roots, unknown clients, replaced/rolled-back journals and foreign owners stay gaps; metadata reads and summaries bounded. | Admitted native extraction, latest-witness authority and hostile-provider qualification. |
| Git/adoption | Explicitly retired sources can stay registered with no roots; retained jobs keep their own demand. Large snapshot-to-View conversion refuses existing limits instead of truncating. | Source/runtime adoption and scalable executor integration. |

Producer disposition: pass_with_risk for this read-only prerequisite. No new
journal command or persistent schema; no consumer deployment, independent review,
full requirement validation or permission to remove bytes. Focused and workspace
verification is recorded separately under EVID-085 after final checks.

Final resource review found a missing elapsed-time bound in the new projection.
Added a 1..600,000 ms cooperative budget checked between/after calls and before
return, with explicit slow-provider and invalid-window cases. It does not promise
interruption of a blocked provider. Ordinary fixtures use 60 seconds; the 33,000-
binding case uses 180 seconds. All sixteen focused cases pass on the correction.

Input bounds now precede profile cloning/sorting, and oversized View projections
refuse before expanding placement copies. Valid snapshot identities are unchanged;
the existing budget/large-view fixtures exercise refusal without truncation.

Identity follow-up: identical historical/current roots must not collapse distinct
source selections. Origin identity now includes the exact selected revision or
pending prepare reference; a repeated identical-root fixture checks both versions.
This is the first unshipped projection format, not a change to journal encodings.

The selection-provenance change initially missed the predecessor tuple constructor;
compilation refused it before tests. Corrected to carry the same exact revision
shape as current selections. The repeated-root regression covers that constructor.

## Mandatory admission producer review

Input355aad7fc6ab964527b46492cdb6f98a395f58c1; roadmap review85ac433;
implementation0b75664e63ca3807096fb9e1dfdafd7faac275ce. Controls and RUST/PROVIDER
entry/exit/L0/L1/L2 scope are in specs/MANDATORY-SOURCE-ADMISSION.md. Method:
one producer applying all six .roles checklists against model/safety contracts,
T01/02/03/08/11/12/13/15/20/24/25 and the corrected storage roadmap. Owner of
each disposition: git-asset maintainer. Independent review and human acceptance remain
separate. This is admission qualification, not authority for live provider IO.

| Lens | Finding and disposition | Remaining qualification |
| --- | --- | --- |
| Identity/namespace | Guarded marker design-2 binds immutable placement policy to a fresh authority identity. Owner/class enrollment is immutable; exact operation replay and source revisions bind the compiled permit. Legacy design-1 omits new fields and refuses Enroll. | Store-scope binding across every applicable namespace, epoch handoff, latest-authority and rollback-resistant restore. |
| Placement | Fence raw active/history/retained bucket changes, including role-only changes. Configurable active precedence avoids unnecessary history copies; retention-class obligations remain additive. The older snapshot collector remains an additive copy-oriented projection and cannot stand in for effective namespace policy. | Provider constraints, namespace-wide ownership admission and mixed-root retained-subset handoff before final retirement. |
| Custody | Scoped absence borrows a live subject fence and always denies removal authority. No custody or physical availability is inferred from roots or places. | Independent retained destination, store binding and physical removal/reclaim tests. |
| Concurrency/recovery | Generic guarded mutations and preview refuse bypass. Shared admission/exclusive assessment use 256 ordered nonblocking partitions; collisions can conservatively contend. No SQL spans tree reads. Changed source revision refuses stale compilation; unrelated heads can advance during subject assessment. Actual child exit releases OS locks and exact receipts recover. | Epoch transitions and intentional-absence/incarnation protocol before any payload unlink; hostile anchor/root replacement remains outside the cooperative filesystem contract. |
| Provider/privacy | Missing/corrupt closure, wrong sizes, marker changes, missing anchors and finite work/deadline limits refuse. Static errors omit private paths. No live source/store is opened by resume. | Native/opaque extraction and external writer/pruner boundary admission; power-loss/provider-specific guarantees. |
| Git/adoption | Legacy journals remain usable without migration. Guarded sources require explicit enrollment; no hooks, daemon, consumer switch or automatic merge workflow is installed. | Trusted Git event integration, branch/job registration and bounded recovery under actual intended use. |

Disposition: pass_with_risk for the cooperative local namespace prerequisite.
Implementation, scoped verification, intended-use validation, trace and this
producer review are distinct records; full work packages and all 110 scenario
statuses remain open/designed. EVID-086 records executed versus pending checks.

Source budget review, PKG-009: the 4 MiB ceiling was nearly exhausted by generic
implementation/tests (4,165,590 bytes at the input checkpoint). Admit 5 MiB for
this code and synthetic fault/process fixtures. Identity keeps persistent runtime
encodings/limits separate; placement/custody grant no extra cache capacity;
concurrency retains bounded journals/work; provider/privacy keeps every private,
media/binary/runtime exclusion; Git/adoption retains 128 KiB per file and excludes
generated logs/builds/databases. This changes no runtime storage/resource limit.
The validator measures 4,231,791 ordinary bytes before this prose closeout.

Failure memory: the initial Enroll field named kind conflicted with serde's
internally tagged command discriminator. Compilation refused the derive; the
field is class, preserving the existing kind discriminator. Focused tests cover
new marker/command compatibility, immutable enrollment and legacy encoding.

## Store-bound protection producer review

Input83d3ec950e3145f0d0a8b358ff4480dd943aafe8;
implementationbe6f946cc48c0ca72da641a72b55a86190985bba. Controls:
WP-003/004/006/008/009; REQ-001/002/003/004/005/006/007/009;
SPEC-002/003/004/005/006/008/009; IF-002/003/004/005/006/008/012;
PKG-002/003; RUST/PROVIDER profiles. Scope/entry/exit and L0/L1/L2 are in
specs/STORE-PROTECTION-BOUNDARY.md. IF-003 is explicit for new marker compatibility.
Method: one producer applying all six parliament checklists against model/safety,
foundation T01/03/08/10/12/13/15/16/20/24/25 and the reviewed storage roadmap.
Owner of each disposition: git-asset maintainer. No independent review or live-operation
authority is asserted. No source-budget or runtime-cap increase.

| Lens | Counterexample and disposition | Remaining qualification |
| --- | --- | --- |
| Identity/namespace | An absence fact in one journal can omit another project. The actual store marker now pins every enrolled namespace and assigned place alias; exact roster, provider instance, generation and binding identity are checked. Omitted/foreign/duplicate namespaces refuse. | Latest-authority recovery and object incarnation before removal; identical restored metadata is not automatically current. |
| Placement | Checking only the first alias could miss history or backup demand at the same provider. Every mapped alias contributes after each namespace's admitted precedence. One namespace's current claim cannot erase another's hard obligation. | Native closure, external actor enrollment, policy/namespace handoff and mixed-root retained-subset retirement. |
| Custody | An all-namespace absence result still says nothing about surviving independent bytes. Existing local reader/retention guards participate, and summary removal authority stays false. | Actual retained independent-domain custody, authorized intent and physical remove/reclaim verification. |
| Concurrency/recovery | All subject fences are acquired in authority order, nonblocking, then the existing object guard; failures unwind all acquisitions. Work shares one aggregate budget outside SQL transactions. Child process termination and failed revalidation exercise actual guard lifetime; unrelated enrollment progresses. | Epoch transitions with old-executor fencing and authorized-absence/incarnation recovery before physical retirement. |
| Provider/privacy | Missing anchors must not cause inspection to allocate metadata. The provider uses an existing-anchor exclusive path and exact cached binding checks. Strict bounds precede creation; errors omit private paths and malformed/unknown state refuses. | Hostile filesystem, unmanaged pruning, full restore authority and provider power-loss guarantees. |
| Git/adoption | A new manifest must not silently migrate running stores or impersonate the old provider contract. Only explicit fresh-store creation uses design-6; old bytes remain unchanged and old readers reject it. Actual copy descriptors bind its contract/generation and reuse existing accounting/retention. | Live project enrollment, trusted Git lifecycle execution and dynamic roster/policy/epoch adoption remain separate. |

Disposition: pass_with_risk for the bounded cooperative store-enrollment/assessment
slice. EVID-087 records actual versus pending verification. Full requirements,
work packages, 110 intended workflows, independent review and human acceptance
remain open. Immutable first enrollment is not epoch handoff; no marker replacement,
deletion API, implicit migration or custody permission is offered as a substitute.

Failure memory: compilation caught an omitted protection field in the internal
LocalAttempt store clone. It now preserves the exact binding; the new accounted
copy/retention integration case exercises that path. The first retention fixture
also attempted a private limit field; it uses the existing required_peak accessor.

## Managed ordinary retirement producer review

Inputfcb7009f82b725e16e35eb1f2d1a8deaebe328e1;
implementationfe88c7cb9277d5e2d1c93935cd98932a0d153984. Controls:
WP-003/004/006/007/008/009; REQ-001/002/003/004/005/006/007/009;
SPEC-002/003/004/005/006/008/009; IF-002/003/004/005/006/008/012;
PKG-002/003, RUST/PROVIDER profiles. Entry/exit and L0/L1/L2 are in
specs/MANAGED-REPLICA-RETIREMENT.md. Scope: fresh synthetic managed stores,
ordinary replica unlink/cancellation and explicit reacquisition. Method: one
producer applies all six .roles checklists against model/safety, foundation
T01/02/03/07/08/09/10/12/13/14/15/16/20/24/25 and the storage roadmap. Owner of
each finding: git-asset maintainer. No independent or human approval is implied.

| Finding / lens | Severity and disposition | Remaining qualification |
| --- | --- | --- |
| MR-01 Identity | High: hash-wide deletion evidence could remove a later same-hash placement or resurrect old publication receipts. New records bind physical incarnation; cancellation advances only action metadata, while reacquisition also advances placement. Old terminal retries are historical only. Forged index/version/instance and spent operation refuse. | Full restore/latest-authority fencing and eventual receipt expiry; no per-object prefix is discarded here. |
| MR-02 Placement | High: a dropped label or old dry run is insufficient. Actual all-namespace/place absence is held through the step; live readers/claims and retained source commitments block it. Execution compares private context/domain enrollments, and explicit plan authorization adopts its requested coverage. | A policy selector/controller must independently derive/admit constraints before automatic apply. Opaque/native completeness, mixed-root subset retirement and final-copy authority remain open. |
| MR-03 Custody | High: a keeper may become unavailable after initial verification. Retained guards span removal and actual keeper authority/bytes are reverified after intent, immediately before unlink. Same-domain, missing/foreign or insufficient custody refuses. Allocation-before, logical unlink and historical replay remain separate from whole-volume reclaim. | Native domain independence, device/site/key failures, real full-disk/quota and large-file provider qualification. |
| MR-04 Concurrency/recovery | High: persisted intent precedes unlink; exact recovery after absent bytes performs metadata only. Eight actual child exits cover the boundaries. New demand before unlink can cancel and preserve original receipts. Settled ledger plus missing terminal provider evidence holds instead of repeating old IO. Unrelated physical publication and source commit progress under target fences, with no SQL transaction spanning IO. | Whole-authority restore, journal continuation, partial-attempt cleanup/adoption and dynamic epoch/roster handoff; current per-object history exhausts safely. |
| MR-05 Provider/privacy | High: older writers cannot know intentional absence. Explicit fresh design-7 markers reject them; bare managed mutation refuses, and lifecycle metadata is budgeted through existing allocations. Missing known anchors are not repaired by ordinary reads/writes. Names/records are bounded and private profiles stay external. | Initial unacknowledged bootstrap interruption can need owner recovery; hostile filesystem and power-loss guarantees are not offered. |
| MR-06 Git/adoption | Medium: merging source or opening a repo must not activate removal. The new executor is an explicit library profile; current copy CLI only reports distinct absence/recovery errors. No native consumer, cache, daemon, runner or workflow is attached or migrated. The byte rehearsal publishes replacement/backup, holds old job use, evicts and restores exact previous bytes. | Shared CLI/agent retirement surface, Git event convergence/trusted policy admission, remote upload, user UX and private adoption. |

Disposition: pass_with_risk for scoped cooperative local mechanism verification.
EVID-088 separates executed/pending checks from implementation; full work packages,
requirements, all 110 designed workflows, intended-use acceptance and independent
review remain open. Scope tokens do not authenticate callers. This is not final
deletion, metadata forgetting, automatic policy selection or production readiness.

Failure memory: compile checks caught an over-constrained keeper borrow lifetime;
executor and keeper borrows now have independent lifetimes. A forged maximum index
reached canonical JSON before validation; shape bounds now precede hashing and a
negative case confirms no admission. On Windows held reader handles prevented
renaming an entire keeper directory, so the fault fixture withdraws its authority
marker and claims that narrower failure only. Provider-prefix replay review added
the settled-ledger/missing-terminal refusal and an actual restored-prefix case.

## Retirement CLI/agent producer review

Input3e300ca; implementation19346c45930b575c1bbda5a7ee618fb4101d9d98.
WP-003/004/006/007/008/009/015; REQ-001/002/003/004/005/006/007/008/009;
SPEC-002/003/004/005/006/007/008/009/013;
IF-002/003/004/005/006/008/012/013; PKG-002/003/008, with PKG-009 CI qualification.
Entry/exit, allowed scope and RUST/PROVIDER L0/L1/L2 are in
[the contract](../../../specs/RETIREMENT-AGENT-CLI.md). One producer reapplied all six
.roles checklists against source/model/safety contracts and foundation scenarios;
[RC-01..06](../../reviews/2026-11-08-retirement-agent-cli.md) record findings,
dispositions and remaining gates. Owner: git-asset maintainer. No independent or human
approval is implied.

Review required independent private survivor/domain/context/role/budget admission,
separate execution flags, canonical input-bound plans and no implicit provider
initialization. Status separates historical outcome from fresh integrity and
cannot reconstruct removal authority. The review caught missing lifecycle-side
allocation checks in reacquisition status; both lifecycle and payload bindings
are now validated, sharing the library's payload-shape validation. Cancellation
uses the existing kernel without depending on available keeper/tree/namespaces.
The explicit L2 case runs actual CLI processes after a real child interruption;
CI builds the artifact and invokes it deliberately on Windows/Linux rather than
claiming the ordinary ignored library entry passed. Original copy commands and
wire encodings stay compatible; no catalogue/runner/consumer is deployed.

Disposition: pass_with_risk for this bounded explicit local interface. EVID-089
records executed local checks separately from hosted qualification, validation
rehearsal, trace and review. All full requirements, independent review and live
adoption remain open, as do remote/LFS upload, trusted Git lifecycle convergence,
bulk final deletion, metadata expiry and operational lifetime qualification.

## Source-preserving S3 producer review

Input3fd9918; implementation4fc3060c9c6ecb262cd1b964146ea5087af9a397.
WP-003/005/006/007/014/015; REQ-001/003/004/005/006/007/009;
SPEC-002/003/004/007/008/009/013; IF-002/003/004/005/006/012/013;
PKG-002/003/004/008, RUST/PROVIDER profiles. Allowed scope and entry/exit/L0/L1/L2
are in [S3-PUBLICATION](../../../specs/S3-PUBLICATION.md). One producer reapplied all
six parliament checklists to model/safety/publication/ledger/orchestration contracts;
[S3-01..06](../../reviews/2026-11-08-s3-publication.md) record exact findings,
dispositions and remaining qualification. Owner: git-asset maintainer; no independent
or human approval is implied.

Review required explicit private target/access admission, scoped operation/source
guards, bounded metadata and transport, conditional no-clobber and full readback,
with source preserved and no retention/removal claim. It caught semantic query
selectors hidden by credential exclusion and a concurrent writer/status sampling
gap. Known authentication fields alone may vary; existing completion-attempt
admission now spans remote IO and stale provider/ledger samples are rechecked.
Actual peer contention and another job's progress verify that correction. The
fixture's inherited Windows nonblocking mode was fixed rather than adding a
production retry loop to mask connection aborts. Optional HTTP/TLS dependencies
stay outside default Git/LFS, and private access is neither serialized into plans
nor exposed in errors/Client Debug. Library hosts must disable sensitive transport
trace logging; the CLI installs none.

Disposition: pass_with_risk for bounded source-preserving S3 protocol delivery.
EVID-090 records final corrected-source local tests and separates hosted checks,
verification, intended-use rehearsal and trace. Actual cloud IAM/TLS/account,
provider retained custody/durability/quota, large-file/native/lifetime qualification,
independent review and live adoption remain open, as do LFS upload and subsequent
Git convergence, bulk final-deletion and metadata-retirement milestones.

## Standalone LFS upload producer review

Input9b8a689; implementation2f1173bd2991af429911b251ceaa173916077ee3.
Scope/control entry/exit and GIT/PROVIDER L0/L1/L2 are in
[LFS-UPLOAD](../../../specs/LFS-UPLOAD.md). One producer reapplied all six parliament
checklists; [LU-01..06](../../reviews/2026-11-08-lfs-upload.md) record severity,
disposition, exact evidence and remaining qualification. Owner: git-asset maintainer.
This is not independent review or human/live-operation approval.

Review caught readable-but-unregistered content skipping a required verify on
retry; unfinished operations now perform upload admission/verify and fresh full
readback. Explicit action credentials replace any implicit Batch-token forwarding,
and duplicate/conflicting/routing/media headers, expiry and ambiguous scopes hold.
Shared accounted witnesses preserve S3 wires and scoped progress. Actual process
exit/retry and source-preserving real Git LFS push qualify the bounded client seam.
EVID-091 separately records local verification, synthetic validation and trace.
Disposition: pass_with_risk for this 8 MiB experimental adapter only; hosted checks,
actual server/custody/large-file/lifetime, trusted Git convergence, bulk final
deletion, metadata retirement and private adoption remain separate obligations.

## Committed-ref selection producer review

Inputd81cabe; implementationfc3c57f6f17f1711b27131b75911d72a6c644e72.
Controls/scope/entry/exit and GIT/PROVIDER L0/L1/L2 are in
[GIT-LIFECYCLE-RECONCILIATION](../../../specs/GIT-LIFECYCLE-RECONCILIATION.md).
One producer reapplied all six checklists; [GS-01..06](../../reviews/2026-11-08-git-lifecycle-selection.md)
record severity, owner/disposition and remaining qualification. These are review
lenses, not independent people, user acceptance or live-operation authority.

Review required exact before-state/revision and private source/policy admission,
metadata finish/readback, atomic existing-command composition and separate current
ref observations. It corrected storage convergence to unknown rather than
asserting an unobserved outcome. A no-op selection may still need physical work
when jobs/providers change; copy-created working retention needs authorized
handoff before eviction. Those integration obligations remain explicit. Existing
event encodings are preserved and a pinned pre-pair binary reads/replays new pairs.

EVID-092 separately records local verification, synthetic validation and trace.
Disposition: pass_with_risk for committed-ref selection only. Full automatic
copy/retention/eviction, authenticated trigger, branch/draft/index/native/provider
adoption and large-file/lifetime/subset/authority/final-deletion work remain open.

## Local placement reconciliation producer review

Inputa678381; implementation74327391d8c53e3fe9295ba5a19c3a34b7dbe985.
Controls, exact targets, allowed scope and GIT/PROVIDER L0/L1/L2 are in
[PLACEMENT-RECONCILIATION](../../../specs/PLACEMENT-RECONCILIATION.md). Targets:
requirements projection, placement plans/executor, existing receipt/origin
inspection helpers, explicit committed-ref composition and synthetic fixtures.
Method: one producer applies all six .roles checklists against current model,
safety, source/provider contracts and executed evidence. Owner of each finding:
git-asset maintainer using the named role. No independent review, user acceptance or
live-operation authority is implied.

| Finding / role | Severity | Disposition and evidence | Remaining gate |
| --- | --- | --- | --- |
| LP-01 Identity and Namespace | High, resolved for bounded placement | Exact object/provider/generation/authority/job and source-policy pins; current plan rows are recomputed. Run budgets/order cannot mint new copies. Unexplained managed placements and changed plans hold. | Authority/namespace migration and complete restore witnesses. |
| LP-02 Placement Policy | High, resolved for declared scope | Resolve active/history precedence per namespace, then physical union; shared current/job bytes avoid advisory history. Aggregate compiler work spans all namespaces. A snapshot is never a removal lease. | Full branch/draft/index/native source integration and incremental scale. |
| LP-03 Storage and Custody | Critical, resolved for ordinary local removal | Backup retention precedes cache ingest handoff; required cache readers and existing retained keeper/all-namespace fences span source removal. Offline or missing acknowledged backup blocks eviction. Retained-role copies are never automatically deleted. | Real provider/domain qualification, large files, measured reclaim and separate final-deletion authority. |
| LP-04 Concurrency and Recovery | Critical, corrected for process restart | Late claims/readers block removal while other objects progress. Actual child exit resumes from backup. Cancellation preserves original placement/publication; effects now report the actual birth transition. Keeper choice is stable across provider reordering. | Unattended private-partial recovery, sustained journals, epoch handoff and power-loss proof. |
| LP-05 Provider and Privacy | High, retained external boundary | Private strict profiles, independent default-disabled permissions and source-profile digest; requests supply no machine paths or authority. Missing entry roots hold without initialization. No runner/hook/daemon is installed. | Authenticated external trigger and independent executor OS authority; offline enrollment and live adoption. |
| LP-06 Git and Adoption | High, corrected convergence scope | Real main merge/copy/demotion/revert CLI flow and no-op provider retry pass. Historical selection, observed ref, desired basis and physical page remain separate. Cursor pages report global convergence unknown. | Complete same-basis orchestration, user/UX acceptance and remaining priority-6/later milestones. |

EVID-093 separately records verification, synthetic validation and trace. L0 and
local scoped qualification pass with the exact broad/final-focused timing in
that evidence. Sixteen placement entries include one child helper and one actual
exit; two new projection cases and twelve real Git/CLI cases pass. Both ignored
artifact qualifications are executed explicitly, not credited from ignored rows.
Disposition: pass_with_risk for the 8 MiB/process-restart local composition only.
Hosted current-head verification, independent/human review and full requirements
remain open. Total hash IO/free-space reclaim are not fabricated from stream or
unlink counts. This checkpoint does not close the whole product plan.

## Trusted GitHub request producer review

Input1c253ea, implementation503adca/fixb3aced0: all six producer checklists and
GT-01..06 are recorded in ../reviews/2026-11-09-github-trigger.md. EVID-094
separates passing broad/final-focused/default/Clippy/artifact results from pending
hosted Windows/current-head checks, synthetic nonempty HTTP/store rehearsal and actual hosted issuer
empty-scope qualification. Corrected findings include lost partial progress on a
late metadata failure, accumulating duplicate holds, oversized blob responses,
cross-platform fixture pins and invalid coupled JWT time semantics. Disposition:
pass_with_risk for bounded implementation; no independent/human acceptance or
live-operation authority. Full roadmap and intended-use acceptance remain open.

## Staged-index lifecycle producer review

Input195bd0a, implementation51dd2b2; all six producer lenses and IL-01..06 are
recorded in ../reviews/2026-11-09-index-lifecycle.md. EVID-095 separates focused
Git/CLI/provider/abrupt-exit checks, broader qualification, synthetic validation
and trace. Resolved counterexamples include fabricated merged conflict bindings,
unsupported capture shrinking protection, stale index adoption, lost selection
truth on provider failure and mismatched namespace roots. No new source event
encoding, cache engine, global lock, hook or final-copy authority is introduced.
Pass_with_risk for the bounded implementation and local qualification: default/
all-feature workspace, strict Clippy, standalone Git, Node/restore, both explicit
artifact qualifiers and L0 pass. Hosted Linux/secrets pass at51dd2b2; Windows/
later current-head checks, independent review, real-provider/user adoption and
full roadmap remain open.

Owned settlement review, input9c2a29f/implementation51c5acc: six producer lenses
SS-01..06 are in ../reviews/2026-11-09-source-settlement.md. EVID-096 separates
local verification, synthetic intended-use rehearsal, hosted issuer/check scope
and trace. Successor revision is checked inside the existing transaction; source
and Git observations both gate convergence. Historical roots, keeper fences and
default-disabled execution remain. Source/docs budget rises to 6 MiB for required
generic code/tests/review, with 128 KiB/file and runtime/media exclusions unchanged.
Pass_with_risk for bounded local qualification; hosted current-head, independent/
human acceptance and the full remaining product plan stay open.

Retained-subset review, input63f2c96/implementation2fecfab/corrective002a850:
RS-01..06 in ../reviews/2026-11-09-retained-subset.md apply six producer lenses.
Policy review corrected transient active-use suppression with atomic historical
adoption; observation/fault/process cases verify the boundary without new event
bytes. EVID-097 separates focused verification, synthetic validation and trace.
Pass_with_risk for bounded claim settlement and named local checks. Corrected
locked default/all-feature workspace suites and explicit retirement/old-reader
artifact qualifiers pass. Hosted current-head checks, independent/human acceptance,
complete final deletion, metadata retirement and full-product gates remain open.

Final-retirement claim review, input59174b1/implementation89d260e: FC-01..06
in ../reviews/2026-11-09-final-retirement-claims.md apply six producer lenses.
Place-local surplus is insufficient for final loss; live content membership and
stricter minimum ancestry now hold across an independently admitted whole scope.
EVID-098 separates final-runtime workspace, focused/vector/old-reader verification
and synthetic validation from implementation. Pass_with_risk for this bounded
guard; design-only for provider release/unlink and later FR-06..15. Hosted exact
head, independent/human acceptance and the complete plan remain open.

Final-provider review, input273e639/implementation9f5e8c0: FP-01..06 in
../reviews/2026-11-09-final-provider.md apply six producer lenses. Separate
incarnation/context and portable design-8 contract, exact grant release and
historical outcome prevent stale replay from removing fresh identical bytes.
EVID-099 records local runtime/fault/vector/old-writer qualification, default/all-
feature/Clippy/standalone/L0 and public placement integration. S3 contention now
allows only success/Busy and requires serial convergence, with a deterministic
held-reader fixture. Pass_with_risk for bounded synthetic local use; hosted exact
head, independent/human acceptance and full bulk/provider/authority/lifetime
requirements remain open. No live upgrade, final CLI or deployment is enabled.
