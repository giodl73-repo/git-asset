# Journal activation design review

EVID-139 design only; inputc833add85f9a9c79d3387a78e469bc13a2570403.
[JOURNAL-ACTIVATION](../../specs/JOURNAL-ACTIVATION.md) names controls, full
required outcome, allowed fixture-only entry and L0/L1/L2 gates before code.
Method: producer inspected actual legacy guard/apply_once/catch_up and staged
rollback, current caps, original-history export/lookup and all six checklists.
These are producer lenses, not independent people or human acceptance.

| Lens / target | Finding / severity | Required disposition / evidence |
| --- | --- | --- |
| Identity / ordinary and activation minima | Critical: rotation may keep the same ordinary checkpoint; its minimum cannot detect pre-fence restore | Separate independently retained activation generation/digest minimum; preserve original authority/references, refuse missing/stale minima |
| Placement / restored accounting | Critical: an empty or summary-only successor loses active/in-flight obligations | Full working model/highs and exact typed original dependencies; fresh handle-local probe only, real nonempty multi-rotation fixtures |
| Custody / lifecycle | High: sealing alone could be advertised as permission to reclaim old metadata or bytes | No reclaim on prepare/activate, no seal-only public entry point; expiry and current provider/source/witness authority stay open |
| Concurrency / delayed legacy writer | Critical: old process can pass file guard then begin SQL after rotation | Transactional old events/meta write guards; actual pinned legacy Rust process paused at that exact boundary, older-held transaction and unrelated-domain progress |
| Provider/privacy / finite budgets | High: full journals may have no room for an unknown seal event; naive index editing can consume unbounded staging | Separate fixed bounded fence control row, no journal cap waiver; admitted physical metadata growth and bounded global index/tail proof |
| Git/adoption / proof identity | Critical: current-writer or Python model could be mislabeled legacy qualification | Pin original source/binary and any timing-only patch hashes; unmodified old-binary format refusal plus legacy acceptance, separate full activation/recovery gates |

Historical design findings above remain constraints, not activation approval.
The complete incremental producer/probe/qualification record is preserved at
[immutable source](https://github.com/giodl73-repo/git-asset/blob/9658c13f9c4a169e468a99feb30b4e4fe8d11351/docs/reviews/2026-11-11-journal-activation.md).
Original record SHA256: `1ad455f7089f6c0bec70bca18be9b631578390cae9ce03e9c063dba6d487c9b9`.
This bounded summary does not replace any journal contract or qualify current
source. Manual synthetic publication, controller/recovery, physical admission,
provider authority and expiry gates remain open.

## Continued preparation extraction qualification

Input92a3b7b; owned7184 TERMINAL0/reaped. All Rust/TOML/lock path/raw hashes
checked before every phase and at completion. Focused57 and strict-entry pass;
actual original16384 + two method-written16384 tails + command49153 passes1251.56s
through the shared continued prepare/finish/readback helper. Raw joined full
original+tail passes467.59s:1123422reads447596663B/668057writes258673542B. Full
legacy preparation passes101.44s:114834reads40370841B/49187writes16589897B.
History8/checkpoint3/index8, final strict/fmt and every phase/complete exit0.
External root continued-preparation-owned-ee01ff2856464281a6de846b4c061c77.
Same-source26992 workspace all-feature/all-target strict20.09s and fmt complete0
at continued-preparation-all-strict-ffa79a3270a84fe9bb4ab2c333829d1e.

Verification closes these named component/scale obligations only. Scoped synthetic
validation preserves original/intermediate/current replies, inherited accounting,
Some(0), exact minima and no freshness renewal or cap waiver. Six producer lenses
above remain PASS_WITH_RISK for read-only preparation; no independent/human approval.
Trace uses the extraction entry's unchanged controls and L0/L1/L2 profiles. This
candidate still needs its exact broader/hosted checks. Published92a hosted17 pass
covers the prior helper/scheduler, not this extraction. Manual control publication
remains fixture setup. Genuine physical admission, full controller/recovery,
source/provider/witness/expiry and complete PRODUCT_PLAN/course remain OPEN.
