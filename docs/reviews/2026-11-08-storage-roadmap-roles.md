# Six-role storage roadmap reviews: retained findings

DOCS-only producer checklist reviews; no independent approval, runtime evidence
or storage authority. Inputs:44b7dabf,43adabd5,79791904 and355aad7f. Exact original
scopes, controls, dispositions and counterexamples remain at the
[immutable full record](https://github.com/giodl73-repo/git-asset/blob/8ded690793b37b7132eec1c9e912e944f3b2e286/docs/reviews/2026-11-08-storage-roadmap-roles.md).
Original SHA256: `df556add3e351bff90cf53a8bc62f452884594df27b4856ff673aaa56cb220fc`.

Controls: WP-003/004/005/006/007/008/009/011/012/013/015/016;
REQ-001..010; SPEC-002..014; IF-002/003/004/005/006/008/012/013;
PKG-001/002/003/004/007/008/009. Targets were roadmap, startup/resume, MODEL,
SAFETY, source-claim/protection contracts and catalogue planning. All findings
owned by the maintainer applying the named six-role lenses. Severity H=high,
M=medium. Corrections were incorporated in prose; implementation gates stay open.

## Earlier retained findings

SR-01..06: exact immutable operations/store generations; complete scoped claims
before eviction; independent retained custody rather than availability; scoped
fences through removal; provider capability/credential/path admission; bounded
self-contained startup. Identity, placement, custody, recovery and privacy are
high severity; adoption medium. No live hooks or initialization follow.

LF-01..06: authorized absence versus corruption/replay; generic age/category
selection; separate final-loss consent and simultaneous-retirement safety;
coverage/recovery/continuation; large-object/locked/offline resource proof;
trusted exact-merge executor with policy-enabled routine eviction. First five
high, adoption medium; final deletion remains separately authorized.

RR-01..06: incarnation-bound retirement; owner-only historical claim settlement;
logical unlink versus representations/physical reclaim (no secure erasure);
admitted durable intent/completion space before removal; trigger/policy authority
separation; lifetime/audit/aggregate fences before forgetting. Custody/adoption
medium, others high. Same-hash new content, shared/foreign claims, exhaustion,
crashes, stale peers and untrusted events remain required counterexamples.

## Current architecture re-review

| Findings | Severity / owner | Required retained disposition and remaining proof |
| --- | --- | --- |
| AR-01 | H identity | Store-scope/generation binds EVERY protection namespace; other-namespace demand blocks removal, unknown roster holds exact bytes/incarnation. |
| AR-02 | M placement | Active-use precedence over advisory history prevents copying unchanged shared bytes into history; hard/additive retention remains; old jobs protect working bytes. |
| AR-03 | H custody | Acknowledged retained-subset replacement BEFORE broad-root retirement; mixed eligible/held/shared siblings and crash boundaries preserve survivors; payload authority stays separate. |
| AR-04 | H recovery | Explicit epoch handoff and old-removal fencing; fresh epochs alone are unsafe; admission/removal races, old-peer replay and restart must refuse unresolved authority while unrelated work progresses. |
| AR-05 | H privacy | Drive letters/upload replies/unlink do not prove independent custody/reclaim; alias/offline/locked/versioned providers, actual large-file restore and allocation tests remain mandatory. |
| AR-06 | M adoption | Merge success is not convergence or deletion authority; trusted event/policy/executor separation; wrong source, offline backup and policy escalation leave explained pending work. |

Planning disposition PASS_WITH_RISK; L0 structure/link/hash/budget/diff passed at
those prose checkpoints. L1/L2 proposed scenarios were NOT executed by these
reviews. Historical20 replica,16 handoff and six real-LFS component cases remain
scoped facts, not the planned lifecycle proof. AR-01/03/04 and claim settlement,
final-loss consent, audit retention and rollback-resistant forgetting remain
implementation entry gates. No full requirements, private adoption, other-lane
admission implementation, independent/human acceptance or live caches were closed.
