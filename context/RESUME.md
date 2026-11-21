# git-asset resume

Owned feat/annex-protected-eviction; inputa61cb94 (P1/PR71 dependency).
Goal: implement PRODUCT_PLAN.md PR70/P1-P6. PR70 merged a95c9de; inspect live
P1 qualification/CI before dependent integration. P1's worktree and live runner
remain separate; never restart it from a log/timeout alone.
P2: specs/ANNEX-PROTECTED-EVICTION.md; freeze lifecycle/incarnation,
independent custody and actor handoff before deletion. No new engine,
per-asset namespace workaround or raw hash-wide retry.
Snapshot/fenced proofs4, namespace protection18, native assessment1 pass.
Keeper read binds exact snapshot to actual pool/place and verified bytes under
all subject/config guards, borrowing overlapping source fences. Keeper2 (including predecessor-retirement refusal) and strict Clippy pass.
Next: enforced incarnation/write model, independent custody, native execution/
recovery/CLI/handoff. No removal API or production activation exists yet.
Build/temp/evidence outside source; preserve other lanes and runtime bindings.
Resume starts no scan, cache, hydration, transfer, render or deletion.
