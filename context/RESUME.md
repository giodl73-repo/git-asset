# git-asset resume

Owned feat/annex-protected-eviction. Goal: PRODUCT_PLAN.md PR70/P1-P6.
PR70 merged a95c9de; P1/PR71 merged f12a163 (tree equals qualifieda61cb94).
P1 all-feature/default workspace, standalone, native32MiB and18 hosted checks pass.
No P1 runner remains live. Keep worktrees/runtime bindings intact.
P2: specs/ANNEX-PROTECTED-EVICTION.md; freeze lifecycle/incarnation,
independent custody and actor handoff before deletion. No new engine,
per-asset namespace workaround or raw hash-wide retry.
Snapshot4/protection18/assessment1, keeper2 and explicit real ordinary/bare keeper pass. ReplayBasis pins full source
checkpoints; actual same-hash reingest/claim settlement stales the old basis while
preserving new bytes. Live fences permit unrelated progress; recovery may hold.
No removal API exists. ReplayBasis is NOT an incarnation protocol. Next: reuse
existing per-key pending/retirement/birth state and gate EVERY native writer,
including uncertain child commands; then independent custody, native recovery,
bounded CLI and actor handoff. P2 remains incomplete; P3-P6 still follow.
Build/temp/evidence outside source; preserve other lanes and runtime bindings.
Resume starts no scan, cache, hydration, transfer, render or deletion.
