# git-asset resume

Owned feat/annex-protected-eviction; inputa61cb94 (P1/PR71 dependency).
Goal: implement PRODUCT_PLAN.md PR70/P1-P6. PR70 merged a95c9de; inspect live
P1 qualification/CI before dependent integration. P1's worktree and live runner
remain separate; never restart it from a log/timeout alone.
P2 entry: specs/ANNEX-PROTECTED-EVICTION.md. Resolve provider lifecycle reuse,
shared flat snapshot retention, aggregate all-namespace assessment, independently
retained custody and old-actor handoff before native deletion code. No new engine,
per-asset namespace workaround, raw hash-wide retry or production activation.
Snapshot/fenced proofs4, protection tests18 and native readonly assessment pass;
strict all-feature/all-target Clippy passes. Source config and namespace fences
span verified bytes. Next: keeper custody and enforced incarnation/write model,
then actual native eviction/recovery/CLI/handoff. No removal API exists yet.
Build/temp/evidence outside source; preserve other lanes and runtime bindings.
Resume starts no scan, cache, hydration, transfer, render or deletion.
