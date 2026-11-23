# Resume the initial-offering PR series

Goal: PRODUCT_PLAN.md P1-P6. Owned lane: feat/trusted-native-placement,
draft PR74 based on PR73. Verify live head and CI. Use only synthetic fixtures.

P1/PR71 merged. P2/PR72 and source fence PR73 remain upstream of P3/PR74.
P4 consumer acceptance, P5 Windows RC, and P6 private rollout stay open.
Issue75/U4 is the independent planning lane in PR76.

## Current P3 mechanisms and next work

[Previous checkpoint](https://github.com/giodl73-repo/git-asset/blob/48753f10a412b6d538c8c30804c7c80842c10e0a/context/RESUME.md)
captures exact source protection, owner registration, archive index and
release fencing qualification. Current fixture adds authenticated transfer of
Git-selected working bytes to the retained pool before historical release.

Next: integrate ordinary index successor recovery into the receiver. Full HTTP,
scale, mixed-authority handoff, consumer acceptance and rollout remain open.

Before committing, run repository validation and the applicable Rust checks.
