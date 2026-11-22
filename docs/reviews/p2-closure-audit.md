# P2 closure audit

Input9042dff392f776d0e547a06634eb20daacfef308; PR72. Producer-applied six
[roles](../../.roles/ROLE.md), not independent/human approval. Controls and
L0/L1/L2 are in the [contract](../../specs/ANNEX-PROTECTED-EVICTION.md).
This audits the required end state, not just the available passing tests.

| Required exit | Authoritative implementation/evidence | Disposition |
| --- | --- | --- |
| Preserve identity, bounded flat snapshots and old encodings | annex::snapshot point proofs;5001-member fixture; planning canonical64KiB plan; legacy publication encoding fixture | Component-qualified; no full catalogue-scale claim |
| All-namespace protection, jobs/shared use/unknown coverage | store_protection aggregate assessment; managed read guards; source/keeper namespace admission fixtures | Implemented; exact native failure-family audit remains |
| Independently retained bytes and device/domain checks | open_keeper, StorageSeparation, original ProducerCompletion; actual F:/M: fixture and device counterexamples | Component-qualified; owner direct-backing/domain admission is separate |
| Intent, original producer quiescence and incarnation | control current-origin gate; prepared one-shot publication; existing retirement birth/restore; actual six executor exits | Component-qualified, including unknown effect hold and same-hash restore/replay |
| Concurrent conflict fencing and progress | two paused native cases: same-object job selection Busy, Git config replacement fails, unrelated enrollment succeeds | Qualifies these participating actors/metadata; full unrelated payload publication not shown |
| Bounded CLI and safe recovery | storage native plan/status/apply/resume/cancel; default-false mutation flags; saved ID/typed errors; actual facade cases | Implemented; current-head CI required |
| Copy through retained backup, eviction and restore | Joined controlled Copier with missing keeper input; exact copied producer proof, retained snapshot, Controller/CLI eviction and restore | Two real ordinary/bare F:/M: cases passed; owner/epoch migration remains separate |
| Owner/roster/epoch handoff, paused old actors | Profile only admits NamespaceOwned; MixedLegacy returns Unsupported. Namespace policy is immutable per epoch. Config pin prevents participating edits while held | OPEN: no migration protocol/provenance bridge; declaration/config edit is insufficient |
| Shared native legacy/new production pruning | No qualified old Index/WorkRef-to-Namespace authority adapter in public source | Unsupported; preserve legacy obligations and isolation. No production cutover |
| Default/all-feature workspace profiles and exact hosted head | Earlierc05a69d full hosted pass; later heads require their own runs | OPEN until current-head required jobs pass |
| Implementation/verification/validation/trace/review closeout | Public SDK/CLI, contract, component manifests and producer reviews | OPEN for full P2; producer review is not independent or owner acceptance |

The handoff gap cannot be solved by removing the MixedLegacy refusal, creating a
new acknowledgement for visible bytes, accepting an alias as physical producer,
or editing only a configuration hash. An already-open legacy writer/pruner may
ignore new metadata. A real transition must fence that actor, preserve all jobs/
drafts/retained claims and unsettled effects, then bind old physical provenance
to the newly admitted authority without permitting stale removals. Current config
changes also stale original producer profiles; weakening this check is unsafe.

The joined copy fixture closes one integration seam, not this authority gap.
P2 stays draft until all contract exits are proven. P3 merge placement, P4 REEL,
P5 pinned candidate and P6 private owner acceptance remain in the
[full PR series](../../PRODUCT_PLAN.md). No new cache/journal engine, inventory
scan, private source import or live operation is authorized by this audit.
