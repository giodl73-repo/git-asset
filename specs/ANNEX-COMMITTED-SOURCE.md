# Exact native-annex committed source

P3 inputf6e66f2b0d44356443f8cd82dedb9ed8367f6514.
WP-004/005/006/009; REQ-001/002/004/006/007/008/010;
SPEC-002/003/004/005/007/008/010/011; IF-002/004/005/008/010/012;
PKG-001/003/004/006. GIT/PROVIDER/INTEGRATION.
Allowed: explicit metadata-only annex CommittedSource adapters, bounded key/blob
parsing, shared existing extraction/HTTP machinery and synthetic source fixtures.
No new publisher/cache/journal, payload lookup/hydration, filter/checkout execution,
authority/epoch transition, native placement effects or workflow activation.
Entry: existing exact Git/HTTP captures, issuer trust and committed-source seam.
Exit: immutable native-key bindings with no latest fallback, unsupported coverage
refused, bounded source IO and old extraction/listing encodings unchanged.
L0 source/format/diff/budget/pins; L1 locked source/GitHub tests and strict Clippy;
L2 actual Git commits, bare metadata and explicitly admitted synthetic annex tools.
Producer checks remain separate from independent/owner acceptance and full P3.

Existing Repository::extract and github::source::Source remain the original
pointer contract. Native annex capture is explicit, with distinct listing domains
and separate enrollment/adoption. No default interpretation or old hash domain
changes. Source declarations are content identity, never provider paths/custody.

The bounded single-line subset accepts SHA256 and SHA256E keys with canonical
u64 byte length and lowercase SHA256. SHA256E's optional printable ASCII extension (no slash/backslash) does not
alter content identity. Missing size, extra key fields and chunked keys refuse. Regular/executable pointer blobs accept /annex/objects/KEY followed by
EOF, LF or CRLF. Canonical relative .git/annex/objects/A/B/KEY/KEY declarations
accept ordinary pointer blobs or Git symlink mode; the repeated key must match,
shard components are bounded alphanumeric and parent depth matches the logical
Git path. No link is followed and no declared target is passed to filesystem IO.
Other backends, unknown modes, extra lines and metadata above1024 bytes refuse
complete selection. This is deliberately narrower than all annex pointer forms.

Repository::extract_annex uses the existing bounded cat-file/listing paths.
source::annex::Source implements CommittedSource for the exact Git repository;
Github AnnexSource reuses fixed-origin authenticated HTTP, raw Git blob hashes,
explicit scopes and cumulative metadata/time budgets. Neither fetches Git packs
or payloads. Historical captures are exact; lifecycle planning independently
requires the current enrolled ref and original namespace baseline. Signed event
provenance is still separate from source currentness and storage authority.

Managed native pools still resolve their admitted canonical SHA256 keys. A
project SHA256E declaration supplies content identity; it does not grant arbitrary
key/pool discovery or imply those bytes are present. Missing bytes hold execution.
The new adapters do not themselves connect native copy/eviction to the receiver;
that P3 integration and its fork/stale/revert/duplicate/outage/crash proof remain
OPEN. P2 mixed legacy refusal and P5/P6 release/adoption gates remain unchanged.

Primary format references: [pointer files](https://git-annex.branchable.com/internals/pointer_file/),
[keys](https://git-annex.branchable.com/internals/key_format/),
[locked/unlocked files](https://git-annex.branchable.com/tips/unlocked_files/).

## Current-generation provider-work guard

Input69a1d5c7f78d8154aaead4002246e2a92e9519ca; same WP/REQ/SPEC/IF/PKG
controls above, plus existing source-generation-use contract. Allowed extension:
Execution::hold_current and synthetic committed-source race fixtures. Entry:
exact admitted lifecycle Plan and design-3 source gate. Exit: original owner,
full current Version and actual source revision held without namespace mutation;
legacy Unsupported, wrong/historical generation Stale, conflicting writer Busy.
L0 pins/format/source budget; L1 locked lifecycle/source fixtures and Clippy;
L2 real synthetic Git commits with native pointer blobs and scoped journal gates.
No native payload effects, receiver activation or custody qualification here.

hold_current samples the enrolled Git ref before and after acquiring the existing
source-use guard. It works with ordinary and native CommittedSource adapters,
including duplicate/no-op plans; it uses the actual source revision, not a plan's
pre-admission revision. Selection failures and source-use failures have distinct
CurrentUseError variants. Legacy namespaces refuse without migration or writes.
A returned guard blocks participating changes to that source generation, not
unrelated sources or Git commits. Callers must sample the ref/authorization again
between bounded effects; remote Git refs cannot be locked by this local guard.
No TTL, serialization or durable worker receipt is introduced. Releasing a guard
or process exit never withdraws claims or removes bytes. Native copy/eviction
must still acquire their own actual custody, roster and physical-origin proofs.
