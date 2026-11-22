# Native copy receiver

Experimental P3 copy and backup reconciliation over Git and git-annex. This extends the
existing foreground receiver; it does not install a service or activate caches.
Full superseded-asset handoff/eviction scheduling and project acceptance remain
open. Use synthetic enrolled locations before any owner-approved installation.

Build with `github,annex-publish`, then inspect `git-asset-trigger describe`.
`native_copy_reconciliation` reports the compiled entry;
`native_payload_supported` reports native execution on this platform (Windows).
Metadata-only dry runs do not claim payload capability. Apply on an unsupported
platform refuses before source selection or native work.

The outer `tig.github-receiver.design-1` profile and `serve PROFILE` command are
unchanged. Point its private `placement_profile` at a document with these fields:

| Field | Meaning |
| --- | --- |
| `schema` | `git-asset.native-receiver-placement.experimental-1` |
| `policy` | `github::native::Policy`: fixed source/retained and destination/working profiles, controls, distinct admitted domains, required namespace/alias, permissions and exact producer roots |
| `source`, `destination` | Existing resource bindings: `namespace` (`root`,`minimum`), `metadata_root`,`metadata_minimum`,`witness_root`,`ledger_root`,`capacity_minimum` |
| `limits` | Existing protection bounds: `millis`,`roots`,`bindings`,`associations`,`reads`,`read_bytes` |

No roots are initialized or discovered. Provider identities, actual native control
origins, publication resources and byte integrity are checked by the existing
adapters. Domains declared in a profile do not prove independent hardware.
The outer namespace/metadata belong to the selected project; each location's
publication namespace belongs to its original candidate producer. A wrong
binding holds rather than choosing a different namespace.

Publish immutable producer records through
`annex::producer_index::Record::store` and the existing Tree/metadata mechanisms.
`policy.producers.source` and `.destination` pin those exact tree Objects.
They contain original publication plans and optional exact retained snapshots,
not commands supplied by a webhook. Publishing/retaining these roots is an
explicit operator/SDK responsibility; no automatic registry editor exists here.
Changing a root or permission changes the policy identity and requires explicit
trust reenrollment. Existing policies without producer roots retain their hashes
and continue to support the single-object worker.

Start with outer `allow_receive` and `allow_selection` deliberately enabled,
`policy.allow_selection=true` and `policy.allow_copy=false`. A dry run reconciles
the committed desired selection and resolves plans, but performs no native
payload observation or transfer. Review `planned` outcomes and bounds. Activation
sets `allow_copy=true` in a newly admitted policy; an event cannot enable it.

The existing strict POST `/v1/reconcile` body remains a schema, immutable commit
ID and optional `{basis,after}` cursor. The bearer must pass the existing signed
OIDC trust checks. No paths, provider plans, executable code or permission changes
are accepted in that body. Keep credentials in the existing private token input;
incoming OIDC credentials are not forwarded to GitHub's metadata API.

One request handles at most `min(budget.objects,128)` desired objects, with the
existing prospective-byte and deadline bounds. Lookup reads share `limits.reads`
and `limits.read_bytes`; selection/protection and each native operation retain
their own explicit limits. `budget.pages` remains validated; this native entry
performs one page per request. Supply `next` to continue on the same basis. An
empty suffix is not proof that its prefix ran. A changed job/input set, selected
generation, policy or commit invalidates continuation.

Outcomes are `planned`, `copied`, `backed_up`, `present` or `held`. `present` requires verified
working bytes, original completed/settled publication records and retained source
custody; it works after the temporary working ingest claim is retired. Forward
copying requires the exact admitted retained publication or snapshot. With both
`allow_copy=true` and `actions.backup=true`, the receiver can copy original F
ingress to the enrolled retained location when that backup has not started.
Both producer plans must already exist in the pinned indexes. The reverse
profile, controls and domains come from the same fixed pair; there is no
event-supplied destination or automatic incarnation creation.
`storage_converged` remains unknown because retention/eviction phases are separate.

The outcome's `plan` is the deterministic copy-plan handle; `publication` names
the actual completed working publication that was verified. Direct ingress on F
and a later M-to-F copy have different publication recipes. The receiver checks
only those two frozen recipes and requires one exact completed/settled match,
then verifies actual bytes with its guards held. It never invents a successful
copy receipt merely because an earlier ingress already supplies the bytes.
`Plan::progress` is metadata-only; `Publisher::observe_guarded` supplies the
separate fresh byte/configuration/control observation. Neither creates retention.

`backup` names the original reverse copy plan, and `retained_publication` names
the verified retained publication. A `backed_up` result holds the existing
copier's verified F source and new M destination through its post-effect check.
It does not repeat a forward transfer. Duplicate requests recognize that exact
backup recipe even while the index still names its frozen pre-copy plan. Backup
permission is separately denied by default; false is omitted from old policy
serialization. Dry run reports both plan handles without native payload work.
See [backup contract](../specs/TRUSTED-NATIVE-BACKUP.md).

Preserve the report's policy, basis, object and copy-plan ID. The same pinned
index records reconstruct that original plan; native receipts remain in the
existing witness/ledger. A post-effect hold preserves completed progress. If a
reply is lost, start the same pass again: completed bytes can be observed, while
an unfinished native attempt stops for original Copier status/resume. Never
replace an uncertain operation with a new publication attempt.

| Failure | Next step |
| --- | --- |
| `missing` | Restore exact metadata/source availability; missing registration is unknown, not absent bytes. |
| `corrupt` | Investigate hash/length/canonical metadata or bytes; do not overwrite them as an ordinary retry. |
| `stale`, `conflict`, selection drift | Resolve the committed/policy basis and obtain a fresh authenticated request. |
| `admission_denied`, `busy` | Fix the declared capacity/budget or wait for scoped contention, then retry the original basis if still current. |
| `unsupported`, `denied` | Correct capabilities or explicit operator permissions; retries do not grant authority. |
| `uncertain`, `needs_recovery`, `missing_acknowledged` | Use original publication/copy status and recovery handles; preserve claims and bytes. |

See the [contract](../specs/NATIVE-RECEIVER.md),
[producer index](../specs/NATIVE-PRODUCER-INDEX.md) and
[native worker](../specs/TRUSTED-NATIVE-WORKER.md). This is not a renderer, task
scheduler, production cutover or approval of mixed legacy pruning.
