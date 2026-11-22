# Exact historical-root metadata archival

Input b02d5ad7e5a373bc672e528d52589162a3d9ad74; P3 WP-004/005/006/007/009;
REQ-001/002/004/006/007/008/010; SPEC-002/003/004/005/007/008/010/011;
IF-002/004/005/008/010/012; PKG-001/003/004/006. RUST/PROVIDER/INTEGRATION.
Allowed: preserve exact historical branch/draft metadata trees and origin plans
through existing NodeStore/MetadataStore and source claims; explicitly release
that historical working obligation under a source-revision fence. No media
transfer, physical removal, final deletion, new journal/cache or production setup.
Entry: admitted owner/namespace/provider identities, existing fenced namespaces
and an exact predecessor Version. Exit: retained metadata/origin plus a bounded
report, or explicit refusal with working history preserved. L0 pins/roles/source/
format; L1 locked permissions, budgets, races and refresh; L2 synthetic restart
and native per-member retained readback. Independent acceptance stays separate.

`annex::history` separates metadata obligations from observed media custody.
Publishing an archived tree declares retained placement and retains its metadata;
it does not assert that every payload is available at that location. Release
removes one explicitly authorized historical branch/draft obligation after complete
retained metadata readback. `media_custody_verified` is null and
`authorizes_eviction` is false. Actual cache eviction still needs the existing
native controller's fresh all-authority protection and independent keeper checks.
Unverified/missing members never become disposable solely through this metadata
handoff. Active jobs, other branches, drafts and explicit retention remain intact.

The profile binds exact working namespace/source/owner/class, retained namespace/
owner/place/key prefix, and an existing metadata-only `pending::Scope` for records.
All three namespaces are distinct. Working class is Branch or Draft, never Job or
Retention. The record namespace's Job policy retains current/pending/predecessor
metadata on its one declared alias. Target Retention policy must include its
retained place for every role. Namespace source-use and provider identity are
mandatory; unknown/legacy capability refuses. `allow_publish`, record `allow_store`
and `allow_release` default denied. Location binding excludes permissions/budgets
so read-only recovery remains possible under current explicit limits.

The canonical plan captures the original Version (revision, root Object and
context), exact expected current Version/revision, profile binding and stable
archive identity. It never substitutes another commit/tree/current selection.
Origin metadata preserves logical paths and object hash/length. No Git checkout,
project-specific media interpretation or physical payload path is part of a saved
record. The original source context is preserved as an identity, not fabricated
into a recovered Git commit string.

Publication holds the working source generation while traversing that exact
metadata closure. Each requested tree/map node is hash/length verified and mirrored
unchanged to the retained MetadataStore. Payload leaves are not opened/copied.
The plan and one-entry record tree are written there, finished and read back;
the complete retained tree must equal the original bindings. Cumulative tree/
record reads and bytes, write attempts/bytes and phase deadlines are finite.
Write caps use the profile's read-count/read-byte ceilings; provider quotas also
apply. The per-record plan is at most64KiB. No runtime admission limit is raised.

Only after readback are the exact media Retention source and metadata-only Job
record adopted. Every referenced plan is a retained leaf, not merely an orphan
node. A failure before adoption grants nothing; a failure between adoptions can
leave extra retained demand but cannot release working history. Deterministic
archive/enroll/selection IDs permit interrupted retry without duplicate logical
registration. An enrolled empty record slot can finish after Main advances, using
fresh current-generation capture; no older release could execute without its
adopted saved record. Complete existing records win over ordinary republishing.

Loading requires the exact retained Saved Version and canonical plan hash/length,
record tree/context, owner/class/provider and archive-key matching. A live record
source-generation guard pins its metadata history. `member` performs a bounded
point lookup on the frozen archived root and returns an exact Snapshot; it does
not scan/hydrate other payloads or infer custody. Existing Publisher.open_retained
can separately verify and open one such member after original ingress is released.

Before a new release, complete retained metadata is re-read under a live retained
source-generation guard. The original expected working generation is checked;
its shared use is dropped immediately before the owner-authorized Retire command,
whose expected revision fences the race. Host admission runs before that boundary
and again before the command. Drift refuses rather than retargeting. The command
retires only the named predecessor revision; current selection remains. Historical
command inspection returns the original receipt, never new release authority.

`refresh` is explicit and consumes the current saved-record handle. It requires
that the original release is unrecorded, Main has strictly advanced, and the same
old Version remains a predecessor. A fresh current-source guard prevents a race
while adopting the new record. The old expected revision cannot execute after
that monotonic advance. Source CAS replaces only the metadata record's current
plan; all older record Versions remain retained and exactly reopenable. The
archived media root/Version is unchanged. Contention refuses without a global
lock. This rule does not replace/cancel an uncertain native eviction operation.

No timeout forgets roots or plans. Existing source-count, predecessor, provider
and IO bounds may refuse safely; unlimited history is not claimed. Receiver phase
scheduling, producer-root advancement/retention, full HTTP qualification and
project convergence/acceptance remain OPEN. This component does not close P3-P6.
