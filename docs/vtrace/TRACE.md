# git-asset trace

Historical component/checkpoint trace is preserved in [immutable source](https://github.com/giodl73-repo/git-asset/blob/4a153bf487b5f37553d70ebbb941b2aa5666b452/docs/vtrace/TRACE.md).
LF SHA-256: `8a638ba3ea87d3ad7c098942cbc662b707b8616d8c2beb403ebce439e4845783`. The baseline requirement allocation below remains OPEN;
current component progress is separate from full intended-use acceptance.
See EVIDENCE.md and PRODUCT_PLAN.md for evidence scope and remaining gates.

All requirements are OPEN; scenario coverage below is designed, not proof.

| Requirement | Obligation | Spec | Planned cases | Owner |
| --- | --- | --- | --- | --- |
| GIT-ASSET-R01 | Exact identity and snapshot-qualified logical paths | specs/MODEL.md | T01/T04/T05/T21 | Identity |
| GIT-ASSET-R02 | Explicit constraints and complete conservative demand | specs/MODEL.md | T03/T06/T19/T20/T24 | Policy |
| GIT-ASSET-R03 | Independent custody and fresh safe removal | specs/SAFETY.md | T07-T10/T12/T23 | Custody |
| GIT-ASSET-R04 | Concurrent recovery without a global lock | specs/SAFETY.md | T11-T14 | Recovery |
| GIT-ASSET-R05 | Bounded incremental manifests, transfers and physical accounting | specs/MODEL.md | T15/T17/T18/T22 | Store |
| GIT-ASSET-R06 | Honest portable providers and private generic integration | specs/SAFETY.md | T16/T21/T25 | Provider |
| GIT-ASSET-R07 | Indexed shared lookup, incremental publication and honest stale/offline state | specs/CACHE-AND-LFS.md | CL01-CL06 | Store/recovery |
| GIT-ASSET-R08 | Standard Git LFS pointer/transfer integration without Art, with full copy accounting | specs/CACHE-AND-LFS.md | CL07-CL10 | Git/provider |
| GIT-ASSET-R09 | Git/worktree source transitions and recoverable durable protection independent of derived indexes | specs/GIT-WORKFLOW.md | GW01-GW15 | Git/recovery |
| GIT-ASSET-R10 | Optional typed assets incorporate Art value without duplicating identity/storage or requiring typed records for Git/LFS | docs/UNIFIED-DESIGN.md | Art parity and semantic cases planned; T25/BN10 cover generic boundary intent only | Core/adoption |

Need: small versioned logical records, predictable storage, recoverable parallel
operations, independent backend choice and explicit adoption from Art/other clients.

All 110 cases have per-case requirement, principle, invariant, pitfall, mitigation
and owner links in tests/scenarios/learning-links.json. CR/RW/LF/SP/BN/LC add
60 design walkthroughs covering recovery, races, Git/LFS, space, representations
and event-driven proximity/replication lifecycle. See
[review learnings](../scenarios/LEARNINGS.md) and [mitigations](../scenarios/MITIGATIONS.md).
These extend planned coverage, not verified requirement dispositions.

specs/BYTE-WORKTREE.md refines GIT-ASSET-R01/R04/R05/R09/R10 with pinned-base overlay,
explicit removals, immutable candidates, Git selection and same-path conflict
rules. Related existing cases and required new executable fixtures are listed
there; no overlay or tree-merge behavior is yet verified.

## Complete V chain

REQ aliases map one-to-one to the stable GIT-ASSET-R table above. Every full requirement
is OPEN. Implementation/evidence columns distinguish a partial current mechanism
from target packages; designed acceptance is not a verified product disposition.

| Requirement ID | Need / CONOPS | Specification Item | Architecture / Interface / Design | Work Package | Implementation | Verification | Validation | Evidence Pointer | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| REQ-001 | NEED-001; CON-001/CON-002 | SPEC-002/SPEC-006/SPEC-010 | ARCH-001/ARCH-004/ARCH-005; IF-002/IF-007/IF-010; DES-001/DES-006/DES-009 | WP-001/WP-002/WP-010 | Pure canonical/raw identity and persistent trees implemented; private binding views; provider/materialization target | EVID-007/008/009 vectors and bounded sparse-edit API tests passed | VAL-001/VAL-005 planned | EVID-001, EVID-004, EVID-007, EVID-008, EVID-009 | OPEN; pure WP-001/002 verified |
| REQ-002 | NEED-002/NEED-004; CON-002/CON-004 | SPEC-001/SPEC-004/SPEC-008 | ARCH-001/ARCH-002; IF-001/IF-004/IF-006; DES-002/DES-004 | WP-004/WP-006/WP-009 | Pure assigned union/candidates; durable/constraint implementation target | Partial handoff tests; owner/constraint integration planned | VAL-003/VAL-004 planned | EVID-001, EVID-003 | OPEN; partial mechanism |
| REQ-003 | NEED-002; CON-003/CON-004 | SPEC-003/SPEC-004/SPEC-008/SPEC-009/SPEC-013 | ARCH-002/ARCH-007; IF-003/IF-004/IF-006/IF-012; DES-003/DES-004/DES-008/DES-011 | WP-003/WP-004/WP-006/WP-007/WP-013/WP-014 | Bounded admitted publication and restart-persistent reservations; eviction absent | EVID-010/014/015 restart/transaction/publication tests passed; power-loss/custody/full restore planned | VAL-002/VAL-003 planned | EVID-005, EVID-010 | OPEN; publication subset |
| REQ-004 | NEED-004; CON-003/CON-005 | SPEC-001/SPEC-003/SPEC-004/SPEC-009 | ARCH-002; IF-004/IF-006; DES-003/DES-004 | WP-003/WP-004/WP-007/WP-009 | Pure snapshot/leaf checks plus local object OS fences | EVID-009/013 stale-target, budget and cross-process local exclusion; full claim/provider proof planned | VAL-002/VAL-004 planned | EVID-001, EVID-013 | OPEN; partial mechanism |
| REQ-005 | NEED-001/NEED-003; CON-004 | SPEC-001/SPEC-002/SPEC-005/SPEC-008/SPEC-009/SPEC-011/SPEC-013 | ARCH-001/ARCH-003/ARCH-007; IF-002/IF-005/IF-006/IF-012; DES-001/DES-005/DES-008/DES-011 | WP-001/WP-006/WP-007/WP-008/WP-011/WP-013/WP-014 | Capacity model/journal and admitted file-data publication plus bounded tree work | EVID-008/012/014/015/016 sparse edits/accounting/publication/local samples passed; native domain authority/whole-volume enforcement/reclaim/catalog planned | VAL-001/VAL-003 planned | EVID-001, EVID-003, EVID-008, EVID-012, EVID-014, EVID-015, EVID-016 | OPEN; partial mechanism |
| REQ-006 | NEED-005; CON-001/CON-006 | SPEC-002/SPEC-007/SPEC-010/SPEC-012/SPEC-013/SPEC-014/SPEC-015 | ARCH-004/ARCH-005/ARCH-006/ARCH-007/ARCH-008; IF-008/IF-010/IF-011/IF-012/IF-013; DES-007/DES-009/DES-010/DES-011/DES-012 | WP-000/WP-001/WP-005/WP-010/WP-012/WP-013/WP-014/WP-015/WP-016 | Generic pure core and source checks only | Capability/privacy/non-media integration planned | VAL-001/VAL-005 planned | EVID-002, EVID-004, EVID-006 | OPEN |
| REQ-007 | NEED-003; CON-001/CON-005 | SPEC-005/SPEC-014 | ARCH-003/ARCH-008; IF-005/IF-013; DES-005/DES-012 | WP-008/WP-015 | Object catalog/inspector target; bounded reservation replay implemented | EVID-014 indexed-tail replay; full cache lookup/unknown-state fixtures planned | VAL-001/VAL-004 planned | EVID-003, EVID-014 | OPEN |
| REQ-008 | NEED-001/NEED-005; CON-001/CON-005 | SPEC-007/SPEC-013 | ARCH-004/ARCH-007; IF-008/IF-009/IF-012; DES-007/DES-011 | WP-005/WP-013 | No LFS adapter | Pinned real-client/server/extra-copy fixtures planned | VAL-001/VAL-004 planned | EVID-003, EVID-004 | OPEN |
| REQ-009 | NEED-001/NEED-004; CON-002/CON-003/CON-005 | SPEC-001/SPEC-003/SPEC-006/SPEC-007/SPEC-008/SPEC-012 | ARCH-002/ARCH-004/ARCH-006; IF-004/IF-007/IF-008/IF-011; DES-003/DES-006/DES-007/DES-010 | WP-002/WP-003/WP-005/WP-009/WP-012 | Pure desired transition/private bindings plus bounded publication protocol | EVID-009/010 API/restart tests passed; actual Git/index/ownership protocol planned | VAL-001/VAL-002/VAL-004 planned | EVID-001, EVID-003, EVID-009, EVID-010 | OPEN; partial mechanism |
| REQ-010 | NEED-005; CON-006 | SPEC-010/SPEC-011/SPEC-012/SPEC-014 | ARCH-005/ARCH-006/ARCH-008; IF-010/IF-011/IF-013; DES-009/DES-010/DES-012 | WP-010/WP-011/WP-012/WP-015/WP-016 | Canonicalizer extracted; typed graph/model/variants target | Ten donor canonical vectors passed; full semantic integrations planned | VAL-005 planned | EVID-003, EVID-004, EVID-007 | OPEN |

Current SPEC-001/SPEC-015 evidence is source-bound, not a waiver of target rows.
Package closeout updates this chain and the named VAL/EVID rows together; do not
turn absence of a failing test into evidence of full coverage.

## Stage review

Producer-applied six-role lenses: pass_with_risk. TR-01: requirement/spec/work
mapping could omit current versus target or validation source. Resolved by a
complete ten-row V chain and six resolvable evidence IDs, with full requirements
OPEN and future integrations explicitly planned. No invented user acceptance.

TR-02 structural-assessment feedback: abbreviated grouped spec IDs hid exact
identifiers from canonical trace discovery. Expanded every grouped V-chain ID
explicitly; no spec was dropped, evidence upgraded or scope weakened.

EVID-017 refines REQ-003/004/005/007/009 with exact private-attempt preview,
authorized discard, durable cancellation and bounded remaining-footprint
settlement. Stale publisher status is rechecked under the attempt fence. Full
requirements remain OPEN: this does not implement final-object retirement,
independent custody, actual volume reclaim or complete restore authority.

EVID-018 refines REQ-003/004/006/007 with exact completed-byte verification,
held object guards, monotonic session freshness and explicit declared-domain
coverage. Portable observations cannot restore live evidence; reports explicitly
deny retention/eviction authority. All full requirements and validations stay OPEN.

EVID-019 refines REQ-002/003/004/005/007/009 with budgeted immutable local
retention, exact grant verification and persistent exclusive-access refusal.
Three child exits and concurrent slot admission exercise this component only.
All full requirements stay OPEN: release/handoff, native identity, restore,
claim completeness and final-source retirement are not implemented here.

EVID-020 refines REQ-002/003/004/005/007/009 with exact authorized handoff,
protected successor overlap, immutable release history and typed accounting.
EVID-021 scopes actual legacy consumer compatibility/scale and the source-only budget.
Full requirements and validations remain OPEN; no payload eviction or cutover.

## Product rename trace

GIT-ASSET-RENAME.md controls -> current crate/service/docs naming, bounded
front door, .asset/.tig compatibility and immutable-input identity audit ->
front_door.rs/native_source.rs/check_branding.py -> EVID-123 and separate
verification, intended-use validation and six producer lenses. Full native
retained-publication and provider acceptance remain open.

## README-only CI trace

PKG-009 / SPEC-015 -> README-CI.md -> exact immutable Git comparison and
ordinary README edit classification -> gated Rust steps with required matrix
checks retained -> eight real Git histories and EVID-124. No runtime semantics.

## Open-PR integration trace

OPEN-PR-INTEGRATION.md -> original PR51..60 contracts and aliases EVID-130..139
plus current main interfaces -> reconciled source/partitioned CI -> main891 and
stack1030 audits, locked check and exact-source qualification -> EVID-140.
No runtime format migration or operational activation is introduced.

METADATA-CAPACITY.md -> metadata::capacity/tests/legacy fixture -> EVID-141.

ANNEX-CANDIDATE-PUBLICATION.md -> annex::publication, managed guards, exact ledger
observation and synthetic/native fixtures -> EVID-142; no creative-selection authority.

CATALOGUE-REVIEW-UX.md -> web controllers/HTML/CSS/fixed icon, controller/native
fixtures and actual browser walkthrough -> EVID-143; no storage authority.

CATALOGUE-REVIEW-SAMPLE.md -> existing rehearsals plus derived enrollment/receipt
and front-door discovery/help -> review_sample tests and EVID-144.

ANNEX-RETAINED-COPY.md -> replication/publication/local witness -> copy fixtures; EVID-145.

ANNEX-PROTECTED-EVICTION.md -> snapshot -> four fixture families; EVID-146.

NATIVE-EVICTION-PASS.md -> native::eviction, saved original status/resume and
producer control-retention declarations -> registered eviction native fixtures.
WP-004/005/006/007/009 and contract REQ/SPEC/IF/PKG anchors apply. Component
verification is external receiver-eviction-* evidence; full HTTP, replacement,
root lifecycle, independent review and project acceptance remain OPEN.

HISTORICAL-ROOT-ARCHIVE.md -> annex::history and existing MetadataStore/source
claims -> bounded, restart, Main-race/refresh and native exact-member fixtures.
Controls are in the contract; external history-* evidence pins verification.
Metadata obligation handoff grants no media custody or native removal authority.
Receiver history scheduling -> github::native::history and private receiver phases
-> synthetic immutable-Git preview, paged release/receipt continuation, expiry,
stale-policy/cursor and historical-reopen fixtures. Same archive controls apply;
external history-receiver-* evidence records terminal checks. Producer-root
lifecycle, full HTTP/account/TLS and project acceptance remain OPEN.

NATIVE-PRODUCER-ROOTS.md -> producer_index::update, Record::metadata_object and
history::Loaded::producer_record -> metadata conflict/race/recovery and native
indexed historical reopen fixtures. Same P3 controls as the contract. External
producer-update-* evidence records source/log identities and terminal checks;
root policy adoption, full HTTP and project acceptance remain separate.

Explicit producer-head consumers -> update::Profile::hold_current, shared metadata
Scope checks, native::producers resolution, copy/eviction basis/pins and receiver
admission -> synthetic signed head update/stale cursor/racing writer/no fallback
and actual native copy/eviction fixtures. Same producer-root P3 controls; external
producer-heads-* logs pin qualification. Full HTTP/scale/owner acceptance stays open.

Completed producer recipes -> Record.publication/with_completed_publication,
write-once root update and bounded native recipe lookup -> origin/canonicality/
uncertainty refusal, copy reuse after rebind, backup-index/archive after both
ingests retire and independent working-repository loss. Controls in
NATIVE-PRODUCER-ROOTS; producer-recipe-* external evidence. Full owner orchestration,
backup repair/reacquisition and production acceptance remain separate.

Controlled native restore -> publication::restore and read-only metadata
ReacquisitionPlan::validate_retirement -> exact predecessor/permission/canonical
plan checks, post-control interruption/retry, ordinary/bare retained-source reopen
and historical eviction replay. Controls and producer review: [contract](../../specs/NATIVE-RESTORE.md).
Full owner orchestration, unexplained-loss repair and production acceptance OPEN.

Restore-specific producer registration -> update::restore plus read-only
publication::restore::reopen -> exact old row and retirement chain, fresh held
restored bytes, metered one-row edit/CAS, saved-plan retry and offline historical
receipt. Ordinary updates still reject swaps; original roots remain retained.
Controls/review: [restoration contract](../../specs/NATIVE-RESTORE.md#restore-specific-index-transaction).
Receiver scheduling, missing-backup repair and production acceptance remain OPEN.

Saved restore/index actions -> pending Restore/RestoreIndex phases, pure original
chain binding and index recipe equality -> wrong-chain/cross-link refusal,
fresh-process reopen, denied-store retry preserving original and native lifecycle
fixtures. [Controls](../../specs/NATIVE-RESTORE.md#saved-restore-actions). Existing
four-blob source mechanism only; no native authority or receiver activation.

Authenticated saved execution -> github::native::restore -> signed fixed-context
binding, fenced exact-source generation/working demand, native original-status
reopen, index original receipt before media access and current admission gates.
Native lifecycle exercises disabled/foreign contexts, expiry, renewed proof/keys,
post-selection retained receipt and offline read-only recovery. [Controls/review](../../specs/NATIVE-RESTORE.md#authenticated-saved-execution).
Receiver scheduling and stale-index replan remain OPEN.

Bounded restore scheduling -> native::restore::pass and fixed restore_preview/
restore_apply dispatch -> current-demand/registered-original lookup, read-only
preview, save-before-effect, one eligible apply, released head/read guards and
expected-baseline registration. Native revert fixture checks bytes, fresh basis,
stale cursor, duplicate no-op and index-only historical receipt recovery without
retargeting. [Controls/review](../../specs/NATIVE-RESTORE.md#bounded-receiver-restoration).
Full HTTP, scale, stale-index replan and private owner acceptance remain OPEN.
