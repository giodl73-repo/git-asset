# Native receiver producer review

Inputd43c71e382579383e0549758e1d589f807a64b42; controls and scope in
[contract](../../specs/NATIVE-RECEIVER.md). Producer applied all six .roles
checklists; no independent review, human approval or production acceptance.
Producer owns dispositions and remaining qualification.

| Role | Finding; severity | Disposition / remaining proof |
| --- | --- | --- |
| Identity | H: root changes must invalidate signed enrollment and continuation. | Optional exact roots hashed into policy; absent-field wire retained; pass basis includes commit, source revision and ordered Objects. |
| Placement | H: a successful page cannot imply global convergence. | One bounded page, explicit cursor/holds; all global storage convergence unknown; retention/eviction scheduling OPEN. |
| Custody | H: present bytes can hide uncertain original publication or cause duplicate ingress. | Metadata progress selects only one exact completed original-ingress/copy-context recipe; guarded observation verifies it and keeper custody. Uncertainty holds without reimport; actual publication ID stays separate from prospective copy-plan ID. |
| Recovery | H: retired ingest must not make current desired bytes unreadable, or stale events overwrite progress. | Actual Main generation plus original byte/witness/control guards; current-demand observation permits retired ingest; post-effect drift retains original plan outcome. |
| Provider/privacy | H: receiver must not accept arbitrary storage instructions. | Existing strict authenticated body; explicit path/plan/cursor-field negatives; private fixed profile, regular-file read guards, no service installation. |
| Git/adoption | M: native receiver scope must remain truthful. | Old local/LFS dispatch retained; compiled/platform capabilities separate, unsupported apply refuses; full native HTTP/account/TLS/production acceptance separate. |

Tests and terminal outcomes are external native-receiver-* evidence. Qualify
implementation, verification, validation, trace and independent review separately.

Verification:26 focused GitHub/native-worker/failure-code cases pass, plus two
explicitly invoked real git-annex cases (copy/continuation/released ingest/offline
custody/drift; original working ingress without republication). One private-file
document test and one actual CLI discovery/default-denial case pass. Strict
workspace all-feature/all-target Clippy, standalone github receiver and standalone
annex-publish builds pass. Formatting/L0/branding/source pins are separate checks.
Full authenticated native HTTP/account/TLS acceptance, retention/eviction phase
persistence and production activation remain OPEN.
