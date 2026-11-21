# git-asset resume

Owned feat/annex-protected-eviction; inputa61cb94 (P1/PR71 dependency).
Goal: implement PRODUCT_PLAN.md PR70/P1-P6. PR70 merged a95c9de; inspect live
P1 qualification/CI before dependent integration. P1's worktree and live runner
remain separate; never restart it from a log/timeout alone.
P2: specs/ANNEX-PROTECTED-EVICTION.md; freeze lifecycle/incarnation,
independent custody and actor handoff before deletion. No new engine,
per-asset namespace workaround or raw hash-wide retry.
Snapshot4/protection18/assessment1 and keeper2 pass. ReplayBasis pins full source
checkpoints; actual same-hash reingest/claim settlement stales the old basis while
preserving new bytes. Live fences permit unrelated progress; recovery may hold.
No removal API exists. Next: fully qualify all-writer incarnation proof, independent
custody, native intent/drop/completion recovery, bounded CLI and actor handoff.
Build/temp/evidence outside source; preserve other lanes and runtime bindings.
Resume starts no scan, cache, hydration, transfer, render or deletion.
