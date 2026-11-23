# Source-generation use producer review

Input94865c07928275bb130cf3e002dc48ac82d9cfce; branch
feat/source-generation-use. Contract and L0/L1/L2 controls:
[scoped use](../../specs/SOURCE-GENERATION-USE.md). Producer applied all six
[roles](../../.roles/ROLE.md); this is not independent/human acceptance.

| Role / finding | Disposition / evidence boundary |
| --- | --- |
| Identity: source revision and selected-version revision can differ after history maintenance. | Check expected source revision separately from full selected Version; regression fixture required. Owner is a bound identifier, not authentication. |
| Placement: lock expiry must not remove durable job demand. | No clock/TTL or renewal writes. Process exit releases only live use; cancellation remains explicit Withdraw and retains predecessor claims. |
| Custody: source stability is not byte retention. | Use grants no availability, independently retained custody, complete-roster or deletion authority. REEL must also retain real byte guards and durable claims. |
| Concurrency: all new source effects must honor the same gate. | Central journal commit path locks enrollment-order slots; retries return original receipts without advancing state. No SQLite transaction spans long work. Separate-source progress and child-exit fixtures required. |
| Provider: paths/files are not secure authority. | Trusted stable control roots, ordinary zero-size anchors, Windows physical directory guards. Unix advisory-lock stability and network filesystem qualification remain explicit limits. No arbitrary-writer/rollback defense claimed. |
| Adoption: existing caches must remain readable. | Explicit fresh design-3 capability; design-1/2 creation/open and hash domains preserved. No automatic conversion. Old tools reject new tag; actor/epoch admission stays separate. |

Implementation, verification, validation, trace and review are distinct gates.
The immutable runtime qualification is recorded below; this does not close the
full product/adoption gates. Full REEL Lifecycle,
legacy handoff, workload scale and production acceptance remain open.

Completed older review prose is kept via immutable94865 source links and LF
SHA-256 stubs. Active contracts and P2 closure audit remain in this checkout;
no source budget or safety limit is raised to fit this slice.


At b9f9d84, six focused Windows fixtures and L0/branding passed; the full
combined local suite remained running. Hosted44993567982 used stable Rust1.100
and rejected an explicit unit expression in the new schema-match arm. Replace
it with an empty unit arm (no runtime/encoding change); do not waive strict lint
or qualify that failed head. REEL candidate integration has passing source-use
and actual ordinary/bare native protection fixtures; full Lifecycle remains open.


Code0ccb137cf5a78416e780f8a3372017f642a5cb71 passed full hosted44995031081
on Windows/Linux. Verification: six focused Windows cases, strict all-feature/
all-target workspace Clippy, standalone check, fmt/L0/branding pass; full local
regression passes on the behavior-equivalent pre-lint runtime. Validation:
actual process exit and physical Windows paths; REEL bridgebe3bec8 passes30
Windows tests, four real ordinary/bare native cases and hosted44995403521.
Trace closes this source-use component through the contract and original-receipt/
real-roster guard fixtures. Producer six-role findings addressed; no independent,
human or production approval. Full Lifecycle and legacy adoption stay OPEN.
