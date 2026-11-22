# Trusted native backup producer review

Input b009a2a5ebdf015a0a632ec1ce3b4e9a7f9744ac; controls and scope in
[contract](../../specs/TRUSTED-NATIVE-BACKUP.md). Six .roles producer checklists,
not independent reviewers or owner approval. Producer owns all dispositions.

| Role | Finding; severity | Disposition / remaining proof |
| --- | --- | --- |
| Identity | H: a completed backup has a different recipe from prospective M ingress. | Check only the two exact frozen recipes; forward reads use the actual completed one. Backup and publication IDs remain distinct. |
| Placement | H: reverse copying must not grant arbitrary destinations or bypass dry run. | Swap the existing profiles/controls/domains only; require allow_copy plus default-denied backup. No new allocation before authenticated desired demand. |
| Custody | H: failed backup must leave F readable. | Existing Copier owns both guarded replicas through acknowledgement; offline/expired/denied tests preserve F, retained reopen precedes ingest handoff. No physical eviction here. |
| Recovery | H: an uncertain backup must not become another import. | Pending original recipe holds with its backup-plan ID; repeated receiver requests issue no further import. Post-effect expiry retains successful receipt and fresh-request recovery. |
| Provider/privacy | M: signed events cannot supply storage or execution inputs. | Same fixed private profiles and strict intake; root/plan lookup stays metadata-only, payload follows admitted native Copier. |
| Git/adoption | M: optional permission must preserve old data and policy wire. | False backup permission omitted; existing copy source/recovery APIs retained. Full handoff/eviction scheduling, root publication and project acceptance remain OPEN. |

Verification and exact source outcomes are recorded separately in external
native-backup-* evidence. Synthetic role checks do not qualify production pruning.

Verification:28 focused cases pass, including denied/dry-run/offline backup,
post-effect expiry with preserved acknowledgement, fresh-request retry and
uncertain backup without repeat import. Three explicit real git-annex cases pass:
new controlled backup -> retained reopen -> ingest handoff, plus both existing
forward-copy/original-ingress regressions. Strict workspace all-feature/all-target
Clippy and standalone github/annex-publish builds pass. L0/format/pins remain
separate source checks; full scheduling and owner acceptance are still OPEN.
