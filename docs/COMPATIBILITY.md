# Rename and existing-data compatibility

The current product, repository, commands and Rust workspace are named git-asset.
The rename preserves existing data readability. It does not rewrite Git history,
move cache roots, update deployed runtimes or alter stored objects and journals.

## SDK and command names

| Previous package | Current package | Source path |
| --- | --- | --- |
| tig-core | git-asset-core | crates/core |
| tig-store | git-asset-store | crates/store |
| tig-assets | git-asset-model | crates/model |
| tig-git | git-asset | crates/git-asset |

Rust imports use `git_asset_core`, `git_asset_store`, `git_asset_model` and
`git_asset`. Consumers adopt an exact reviewed supplier commit and update their
Cargo package/path references deliberately; pinned older checkouts remain intact.
Cargo dependency aliases can stage a consumer's import migration separately.

The front door is `git asset`/`git-asset`, with sibling service executables named
`git-asset-query`, `git-asset-history`, `git-asset-storage` and so on. Build/install
the workspace binaries together. Development environment variables use the
`GIT_ASSET_` prefix. Existing installed executables, cache profiles and runtime
bindings are not replaced by this source change.

New teaching records use `.asset`. The native resolver continues accepting
legacy `.tig` records in explicitly declared record scopes. Logical record
identity and content hashes are not inferred from the filename extension.

## Frozen identifiers intentionally retained

The following old spellings are data/protocol compatibility, not current branding:

- Versioned `tig.*` schema tags, hash domains and schema URNs such as
  `urn:tig:definition:design-1`. Changing these bytes changes identity or prevents
  existing records from being decoded.
- Binary hash framing beginning `TIG\0`, and historical stream slots such as
  `tig-revision-history`, `tig-selected-revision`, `tig-delivery-policy` and
  `tig-delivery-history`.
- Existing proposal references under `refs/tig/proposals`, the native pool
  configuration key `tig.protectionbinding`, reserved `.tig` private namespace
  handling and narrowly admitted legacy `.tig-restore-` recovery names.
- Pinned pre-rename CI readers/writers still build the historical
  `crates/tig-store` paths and `tig-source`/`tig-storage` executables present in
  those exact commits. These are explicit compatibility fixtures.

Do not conceal or mechanically replace these identifiers. Future format changes
need explicit versioning, migration and old-data vectors. The rename inventory
checks their original byte spellings separately from product branding.

Safety doctrine aliases now use `GIT-ASSET-` in this checkout; historical
`TIG-P`, `TIG-I`, `TIG-F` and `TIG-R` citations refer to the same obligations,
not different safety promises. Historical source SHAs and evidence remain
historical; renaming prose does not requalify an old implementation.

GitHub repository identity, existing issues and pull requests are preserved by
renaming the existing repository. Source/package moves require other active
branches to rebase and consumers to update paths before adopting this revision.
No branch or worktree is reset as part of that coordination.

Repository-scoped workflow allowlists, OIDC subjects and external consumer pins
that include the old repository name need an explicit consumer update. GitHub
redirects do not rewrite trust-policy strings. Existing open feature branches
need rebasing onto the renamed source layout before their changes can merge.

Pinned activation legacy-writer qualification retains its original crate paths,
source template and TIG_ACTIVATION_CASE/READY/RELEASE handshake. They target the
exact historical writer, not the current SDK. Historical development artifact
paths and commit-specific source URLs also retain their actual old locations.

Imported PR51..60 records originally used EVID-120..129 at source 66f8b4b. Their
current aliases are EVID-130..139, avoiding collisions with already-landed native
and rename records. Original qualified commits, artifact paths and data identities
remain unchanged; the integration itself uses EVID-140.
