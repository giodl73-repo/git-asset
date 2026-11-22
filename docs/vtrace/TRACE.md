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

## Current native placement

Full component trace through bounded restoration is preserved at [source3d024fa](https://github.com/giodl73-repo/git-asset/blob/3d024fab8c54a223863acff5f6f9163bd8c1f6b1/docs/vtrace/TRACE.md).
LF SHA-256: `d9a3427970ce5e3c40fb75ad7abf5d844e72bf57b2d7124bc5587ed37c44d209`. Current normative controls remain in the linked contracts.

Native exact copy/history/eviction/restoration use signed fixed profiles, current
source and byte guards, immutable saved actions and expected-baseline index CAS.
[Restore controls](../../specs/NATIVE-RESTORE.md) distinguish component qualification
from HTTP, scale, mixed-authority and project-owner gates, which remain OPEN.

Explicit stale-index successor -> fenced restore::prepare_replan -> sealed original
binding -> pending-source expected-version CAS retaining all predecessors ->
authenticated saved apply. Fixture obligations: unchanged-head refusal, stale
ordinary apply, denied save, exact retry, parent conflict, no native reimport,
fresh-process old/new reopen and completed/historical original preservation.
[Controls/review](../../specs/NATIVE-RESTORE.md#explicit-stale-index-successor).
Native ordinary/bare and four fresh-process reopens pass, including stale,
competing and denied successor cases. Two pending/eight index tests pass before
field-shorthand-only cleanup; strict final-source Clippy/standalone/fmt pass.
Interface/worker regressions precede argument-bundle-only cleanup. Source/log pins
remain external restore-replan-source.json. Full intended-use/owner gates stay OPEN.


Completed-publication recording -> native::batch::registration plus existing
ordinary root updater and RegisterWorking/Retained pending bodies -> fixed signed
permission and side, current head/closure, real completion evidence, saved sole
upsert before CAS. Joined synthetic/native backup -> registration -> archive/read
fixtures include preview, denied writer, stale cursor, duplicate and cold Saved
reopen. [Controls/review](../../specs/NATIVE-PRODUCER-ROOTS.md#authenticated-completed-publication-registration).
Joined inert/actual-native cases pass for retained backup/archive/read and
working copy/history-rebind reuse. Eight interface, twelve worker, two pending
and eight index regressions pass; final working-fixture replacement has its own
focused/native and strict-source checks. Clippy/standalone/fmt, L0/pins/branding/
diff pass. Exact source/log/test-binary evidence remains external in
publication-registration-source.json. Full merge-stage orchestration, stale
ordinary-index replan, initial producer admission, HTTP/scale and owner acceptance
remain OPEN.
