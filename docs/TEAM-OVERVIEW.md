# git-asset: the team overview

git-asset is a CRM-like relationship and lifecycle layer for git-annex. Think of
an asset record the way you would think of a relationship record: it has a stable
identity, history, links, state, obligations and evidence. The file bytes are
managed by git-annex. The selected definitions and decisions are reviewed in Git.

## The three responsibilities

| Layer | Owns |
| --- | --- |
| Git | Authored definitions, small references, commits, branches, review and selected history |
| git-annex | Large file contents, content keys, physical copies, remotes and native transfers |
| git-asset | Asset identity and versions, dependency/usage relationships, candidates, selected state, jobs, placement/retention obligations and observations |

Projects can add their own asset types and workflow rules. git-asset does not
interpret prompts, provider attempts, language timing, creative approval or
issue closure. Those belong to the consuming workflow. Small authored definitions
stay in Git; media, prompt packets, raw responses, journals and growing evidence
stay asset-side and are linked by exact identity.

## A concrete lifecycle

1. Register a returned file as an immutable candidate, preserving its original
   bytes and provenance. Registration does not select or approve it.
2. Propose a selection against an expected committed baseline. A second writer
   advancing that baseline causes a conflict rather than a silent overwrite.
3. A merge can change desired placement: the selected version belongs in a fast
   working location, older retained versions in an archive, and required backups
   in an independently admitted vault. Locations and maturity rules are policy.
4. Active jobs retain their exact input versions even after another branch
   advances. Outstanding manual work does not lose bytes on a timeout.
5. Movement or removal needs current protection, verified destination/custody
   evidence and explicit authority. Old-version eviction and authorized final
   deletion are distinct. Metadata retirement is another explicit policy.

This is the product contract and delivery direction. It is not a claim that all
five steps have already been qualified against every native storage provider.

## What we can demonstrate today

The Rust workspace has exact Git snapshot and history interfaces, declared asset
graphs, a catalogue, bounded queries, sparse proposals, durable namespace/journal
primitives and synthetic lifecycle teaching fixtures. The native-annex adapter
has verified local reads from explicitly enrolled ordinary/bare pools on Windows,
including exact pool configuration, UUID and namespace-roster checks. Held read
guards coexist with owner adoption and block participating exclusive fences.

The next native work is positive file ingress, uncertain-result recovery,
retained publication/readback and participating-pruner execution. A general
production service, automatic merge worker and complete consumer workflow are
not yet delivered. Existing local/LFS adapters and their fixtures remain useful
compatibility and qualification components.

## Where to start

- Run `git asset describe` for the bounded command surface after building the
  workspace binaries together.
- Use the [sample course](../examples/catalogue/README.md) for reproducible public
  synthetic evidence; its existing provider rehearsals are not native production
  deployment instructions.
- Read [architecture](ARCHITECTURE.md), [roadmap](../PRODUCT_PLAN.md),
  [data compatibility](COMPATIBILITY.md) and [evidence](vtrace/EVIDENCE.md).

Opening the repository or requesting help performs no cache initialization,
hydration, transfer or deletion. Evidence of a record or planned copy is not
evidence that independently retained bytes currently exist.
