# Resume the initial-offering PR series

Goal: PRODUCT_PLAN.md P1-P6. Owned lane: feat/trusted-native-placement,
draft PR74 based on PR73. Verify live head and CI. Use only synthetic fixtures.

P1/PR71 merged. P2/PR72 and source fence PR73 remain upstream of P3/PR74.
P4 consumer acceptance, P5 Windows RC, and P6 private rollout stay open.
The independent issue #75/U4 plan merged in PR76.
[Upstream qualification notes](https://github.com/giodl73-repo/git-asset/blob/2a6c157/context/RESUME.md).

P4 consumer branch `b3babf6cc5f6daad60b46ca3db4a8c6510dca172` passed its
[Windows and Ubuntu adapter jobs](https://github.com/giodl73-repo/REEL/actions/runs/45003799082).
This covers the isolated adapter checkpoint. Consumer integration and owner
acceptance remain open.

## Current P3 mechanisms and next work

[Previous checkpoint](https://github.com/giodl73-repo/git-asset/blob/48753f10a412b6d538c8c30804c7c80842c10e0a/context/RESUME.md)
captures exact source protection, owner registration, archive index and
release fencing qualification. Current fixture adds authenticated transfer of
Git-selected working bytes to the retained pool before historical release.

Local HTTP-to-annex and [hosted issuer qualification](https://github.com/giodl73-repo/git-asset/actions/runs/45119397190) passed at 6932a30. Hosted project-storage execution remains open.

The receiver supports explicit saved registration and archive index successors. Full HTTP,
scale, mixed-authority handoff, consumer acceptance and rollout remain open.

Before committing, run repository validation and the applicable Rust checks.
