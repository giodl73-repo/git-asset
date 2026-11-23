# Whole historical selections and original-binding pages

Experimental Rust metadata API: `git_asset_store::final_retirement::paged`. This first
layer supports the [paged-retirement contract](../../specs/PAGED-RETIREMENT.md).
The operator controller described below supplies the execution boundary for supported profiles.

Use this layer to freeze a large exact selection before independently deciding
what history and copies to retire. A bounded bulk batch preserves held bindings;
several batches can therefore keep one another held when shared content appears
in more than sixteen historical roots. A whole selection permits an honest
projection of the obligations that would remain after all selected history ends.
That observation does not authorize claim release or final deletion.

## Library sequence

1. Independently admit the metadata identity, complete source roster/current
   minima, owner/classes and authority/epoch. Open existing owned providers only.
2. Build each selected-binding Tree through existing Tree/NodeStore ports. Its
   exact paths and expected objects are checked against the actual historical
   source occurrence by compilation. No payload bodies need to exist in metadata.
3. `paged::request` writes small selection records and an immutable request header.
   A Selection names namespace, source, predecessor_revision and the binding Tree.
   Paging policy is frozen in that request; duplicate occurrences refuse.
4. `paged::build` validates the actual historical sources and projects complete
   admitted protection under one metadata/time budget and the intersection of
   private roster/local source limits. It emits a candidate manifest object.
5. Explicitly finish the owned metadata operation, reopen its read-only provider
   with the acknowledged minimum, then `paged::inspect` the full manifest. Adopt
   retention/custody and current authority independently before relying on it.
6. `paged::page` returns byte/count-bounded original-binding pages. Repeated calls
   with the returned cursor cover that fixed manifest; status/inspection perform
   no source/provider mutation or clock sampling.

Request and manifest headers, cohort/subject records and pages are canonical and
bounded to64KiB. Expanding indexes and binding sets live in asset-side Trees;
save only admitted small references in project Git. All writes are immutable
metadata. A failed builder may leave unacknowledged staged objects; the caller
owns that staging and must not implicitly finish another operation's work.

The initial profile caps4,096 selected occurrences and65,536 selected bindings/
objects, subject to the complete protection/read/write/time/provider budgets.
The full admitted closure, including hypothetically removed bindings, is charged.
Source, journal and provider capacity ceilings remain independent limits. A
limit failure refuses instead of truncating or resetting history.

## Read reports precisely

An occurrence is an exact namespace/source/predecessor/original root and context,
with admitted owner/class/policy. It is not a native asset count. Multiple native
assets may map to one occurrence; their meanings stay adapter-owned. Binding counts
include aliases; unique object/byte counts deduplicate immutable content. They are
not copy counts, file allocation, volume reclaim or proof that bytes are present.

`held_by_surviving_claims` records the hypothetical whole-selection observation.
An unselected alias, predecessor, current/pending root, job, retention obligation
or other admitted owner can keep content held. No remaining claim observed does
not establish provider eligibility, retained custody or deletion authority.
The inventory and every page deny claim-release/removal authority explicitly.

Inspection checks request/cohort/subject/count consistency and original binding
metadata. It reports historical data; a supplied digest/header does not authenticate
its own provenance. Before any future mutation, the independently trusted controller
must reconstruct the exact manifest against current privately admitted sources and
compare the root. A changed source vector or protection observation is a new
candidate requiring a new decision, never an amendment of old approval.

## Pages and later execution

A cursor binds one manifest and an actual original occurrence/path. Invalid
positions or another manifest refuse. It conveys an inspection position only;
skipped pages have no implied completion. The caller must retain acknowledged
execution progress separately through the future durable controller.

Pages name original bindings. After one subset transition retires an original
predecessor, later pages need a verified translation through the actual owned
survivor chain. They cannot be applied repeatedly to the original source. Prove
unchanged paths and actual transitions, preserve unselected members and inherit
age using [the age protocol](AGE-WITNESSES.md). Equal hashes cannot replace lineage.

`paged::preparation::prepare` now stages those conditional future source plans.
Supply independently admitted subset profiles and the current complete roster.
Preparation reconstructs the original manifest, translates every page through the
existing pure journal transition model and reobserves the full source basis. It
changes no source or payload. Explicitly finish and reopen the metadata operation
before `preparation::read`; keep the returned small header reference asset-side.
The header points to an ordered Tree of bounded page records and separate plans.
Readback verifies original pages, survivor lineage, unchanged identities and exact
subset projections under a shared operation budget. Predicted checkpoints are
comparisons for later actual receipts, never acknowledgements of execution.

This preparation has no apply/resume/cancel interface or approval token. It does
not freeze provider births/grants, admit age policy, reserve execution witnesses
or authorize claim release. The remaining controller obligations are controlled
by [paged execution](../../specs/PAGED-EXECUTION.md).

The controller must acknowledge separate exact historical-claim and final-copy
decisions, page intents/plans, recovery admission and completion witnesses. It
must validate only its admitted command history under final live fences, preserve
original held exclusions and provider births/grants, stop on cancellation or stale
approval, and report partial effects honestly. Existing subset/final-provider/
accounted ports supply those primitives; this manifest API does not execute them.

## Manual controller candidate

`paged::control` is the separate experimental Rust execution interface. Admit its
Enrollment and complete current roster privately, persist an exact `copies` index,
then `plan` a candidate Decision using existing providers. Explicitly finish and
read back metadata. `control::inspect` exposes original exclusions and the exact
provider plans without payload IO or new permission. The admitted operator separately approves the decision's
historical-claim scope and original final-copy scope. Portable hashes authenticate
neither permission. The experimental operator CLI exposes apply and resume for privately enrolled profiles. Automatic policy installation remains outside that interface.

`Execution::apply` uses these two exact approvals and private job bindings. Calls
default to16 new pages and16 provider attempts. Call again with the same decision
and approvals to resume; progress comes from fixed witness slots and actual source/
provider receipts. All pages settle before physical copy actions. Changed private
authority, clock/window, protection, metadata identity or unknown intervening
commands can hold work. A changed candidate requires a new decision.

`Inspection` needs a metadata reader and no clock/writer. `Cancellation` can stop
without that reader or fresh time. A missing reader yields null observations and
unknown cleanup debt; later cancellation with metadata can resolve admitted page/
provider work. Stopping does not restore retired history or deleted copies.
Reports distinguish page completion from historical source retirement and original
copy deletion; they do not assert secure erasure or newly available volume space.

This interface accepts exact manual selections. It supplies no age-policy or
native-category admission and must not be presented as qualifying an aged preview.
The [execution contract](../../specs/PAGED-EXECUTION.md) retains age inheritance,
native/operator/agent/UI/Action, actual authority/writer/time/provider/large-file,
continuation/metadata-expiry and human acceptance obligations for the full plan.

The explicit age-aware Rust candidate adds `Enrollment.age`,
`control::aged::plan` and `Execution::apply_aged`. Admit a separate metadata-only
registry boundary, clock and required duration privately. Planning freezes exact
retained original evidence in an asset-side index; manual apply cannot execute
this enrollment. Each page verifies the same original age and acknowledges its
survivor's inherited proof before completion. A new stronger fact cannot change
the approved proof chain. Unknown/recent evidence and lost completed inheritance
hold work. Both exact claim/final permissions remain required.

Plain status has no registry port; `age_inheritance_unknown` reports an
uncompleted aged source retirement rather than inventing its inheritance state.
`control::aged::status` can inspect exact retained facts without a metadata writer
or clock, and can clarify an acknowledged fact after interrupted page completion.
It never creates a missing fact. Cancellation still stops without registry or
time; it cannot undo retired history. Full user-facing selection/execution and
private native/provider/authority/lifetime acceptance remain open.

## Connect a post-merge render policy

The existing operator entry point is `git asset storage paged describe`. With a
compatible private profile, `copies` and `plan` stage exact decision metadata.
Use `inspect`, `subjects` and `actions` to review the resulting decision. These
commands require explicit enrollment and existing roots. Any future merge
preview should keep staged metadata separate from production writer authority.

Use `git asset catalogue selection-changes ENROLLMENT PROJECT PREDECESSOR
REPLACEMENT NEW_REPORT` to derive changed or removed selected objects from
two exact commits in one enrolled repository. The report refuses more than
1,024 changes and flags digests still directly selected or stream-bound in
the replacement snapshot. Other projects, nested bindings, jobs, age and
rebuild evidence require independent checks. The report grants no retirement
authority and does not assert that the commits form a particular merge.

Run `git asset storage render-retirement describe` to discover the experimental
preview interface. Run `git asset storage render-retirement preview REQUEST` against
a bounded JSON request. Supply `schema` as
`git-asset.render-retirement-preview-request.experimental-1`, full Git object IDs
as `predecessor` and `replacement`, and `observed_at_ms` and positive
`minimum_age_ms`. Each of up to 128 `versions` names a logical `binding`, an
`object` with SHA-256 and byte length, and optional `superseded_at_ms`,
`derived_render`, `protected`, and `rebuild_evidence` digest. Supply
`protected: false` only with complete protection coverage for the exact content.
The command evaluates supplied assertions, holds unknown fields and emits no
decision usable by the executor. It does not read Git, resolve the evidence
digest, or open any cache. Use an independently trusted producer for the request.

`git asset storage render-retirement verify-inputs REQUEST` checks a supplied
list of exact retained objects through the local retention engine. Provide
schema `git-asset.render-input-verification-request.experimental-1`,
`store_root`, `store_identity`, `ledger_root`, `minimum`, `max_bytes` and
`inputs` containing `object` and `grant` pairs. Use existing absolute roots
and the trusted ledger minimum. The command verifies up to 32 distinct
objects under the requested byte budget, capped at 1 TiB. Results identify
the exact grants observed. The caller must establish that the list covers
the complete rebuild closure and separately prove the accepted rebuild standard.
The local verifier can refuse a slow read as `retained_input_stale`. The byte
ceiling is an admission limit; large-file verification still needs qualification.
A corrupt, missing or released input requires investigation or replacement custody.

The project adapter must supply the exact old release and replacement commit,
a trusted classification of the old asset as a derived render, the supersession
age, and the retained rebuild evidence. The evidence should bind the exact input,
recipe and toolchain identities plus the project's accepted rebuild standard.
An absent, ambiguous or failed check holds the candidate. The same applies to a
golden retained for delivery or an output still required by any admitted job,
branch, project or other owner. A shared hash is checked across the full scope.

A preview should report eligible and held versions, unique logical bytes and
planned placements. Keep actual reclaim unknown until the provider verifies the
physical representation and shared links. Final retirement of a qualified
render has no archive destination. The approved decision goes through the
existing claim-release and final-loss gates. Use the same saved decision with
`status` after an uncertain outcome and `resume` for authorized pending work.
A changed decision requires fresh approval.

Qualify the adapter and controller together with a synthetic replacement and a
rebuild from retained inputs. Also exercise an active job, a shared hash, a
missing rebuild dependency, an exact-release hold, a stale decision and a
restart after partial completion. Verify that the eligible retirement leaves
retained inputs readable and creates no archive payload. Existing legacy shared
working roots need writer handoff and compatible provider qualification before
this procedure can execute there.
