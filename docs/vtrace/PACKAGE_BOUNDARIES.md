# Package and module boundaries

PKG-001 exists; git-asset-store implements bounded PKG-002/003 publication subsets.
Names/paths for remaining planned packages are controlled allocations,
not permission to create all crates immediately. A module can precede a crate;
cross-boundary work must name its integration test and dependency direction.

| ID | Package/module allocation | Owner | Public seam / allowed dependencies | Forbidden dependency / validation |
| --- | --- | --- | --- | --- |
| PKG-001 | crates/core | identity-namespace | Pure identity/tree/claim/plan types; explicit bounded inputs | No IO, Git, catalog, Art checkout or providers; RUST profile |
| PKG-002 | git-asset-store provider ports/bounded local publication | storage-custody | Exact objects, provider capabilities and observations; core | No semantic selection or hidden root discovery; PROVIDER profile |
| PKG-003 | git-asset-store journals and persistent metadata NodeStore; planned full engine/catalog | concurrency-recovery | Claims/journals/admission/reconcile/index modules; core + store ports | No catalog assertion as custody; no whole-copy global lock; PROVIDER profile |
| PKG-004 | git-asset exact-source/LFS transfer/committed-ref selection and optional native catalog subsets; planned full Git lifecycle | git-adoption | Exact tree/index/source extraction, guarded desired-root adoption, optional AssetResolver and configured transfers; core + store and optional assets | No pointer dialect or remote push equals durability; GIT profile |
| PKG-005 | pure git-asset-core::worktree subset; planned durable git-asset-worktree | git-adoption | Frozen base/private edits/seal/adoption; core + engine/store APIs | No writable shared CAS alias or automatic Git commit; PROVIDER profile |
| PKG-006 | git-asset-model native intake and declared-graph subsets; planned full typed layer | identity-namespace | Exact definition binding, caller-admitted snapshot/exact references and scoped primary deltas; planned full variants/build/decisions | Core/storage may not depend on typed assets; no REEL runtime dependency; RUST then INTEGRATION profiles |
| PKG-007 | planned remote/transport/delivery adapters | provider-privacy | Explicit provider traits and scoped capability/credential access | No public discovery/credentials by default or untested protocol stack; PROVIDER profile |
| PKG-008 | experimental Rust CLIs / local catalogue; planned WASM | git-adoption | Application services and read-only pure resolution; optional engine/assets adapters | No private assets in build outputs; no process-management features as product CLI; UI profile |
| PKG-009 | docs/vtrace, tools/validate.ps1, tests/scenarios | git-adoption | Development-only records, bounded synthetic fixtures and structural checks | Not runtime dependency or new production scripting platform; DOCS profile |

Rust implementation uses normal Cargo dependencies/feature gates; no TRACKER- or
Art-checkout-relative runtime dependency. Extraction pins source and parity
vectors before a shared primitive changes identity semantics. Mature provider
libraries require explicit capability/operational fit checks and license intake.

PKG-002 native boundary: optional crates/windows-process owns named job and
selected storage-device handles through CR-005 FFI contracts; depends on pinned
windows-sys and SHA256. Git's annex adapter consumes safe handle APIs under scoped
admission fences. No core/store/semantic dependency or lifecycle state enters
the platform crate. Actual child/grandchild and adapter-order fixtures qualify
composition; cross-epoch handoff, custody and destructive permission stay separate.

Generated manifests, receipts and test outputs are not authored second inventories.
Never vendor private consumer media. Runtime state and scratch roots must be
explicit. Tests use admitted temporary scopes or pure synthetic data only.
Profiles and commands are controlled in LANGUAGE_PROFILES.md and CODE_RIGOR.md.

## Stage review

Producer-applied six-role lenses: pass_with_risk. P-01: optional semantics could
become a dependency of basic byte access. Resolved by one-way dependency rules
and separate low-level consumer acceptance. Backend/crate splitting stays minimal
until a product slice proves need; no speculative package generation is required.
