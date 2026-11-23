//! Delegate fixed operations to the admitted native SDK controller.
use super::{
    PROFILE, REQUEST, RESPONSE, failure,
    profile::{self, Profile},
};
use git_asset::annex::{
    self,
    retirement::planning::{self, Controller, Nodes, Sources},
};
use git_asset_core::{Digest, Object, identity};
use git_asset_store::{
    Key, accounted, ledger::CapacityLedger, local::LocalStore, retention::RetentionGrant,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{ffi::OsString, path::Path};
pub(super) type Result<T> = std::result::Result<T, &'static str>;
pub(super) type Owner = planning::Profile;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    operation: Key,
    snapshot: annex::snapshot::Snapshot,
    grant: RetentionGrant,
    #[serde(default)]
    reviewed_object: Option<Object>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Preview {
    schema: String,
    ok: bool,
    operation: String,
    plan_id: Digest,
    plan: planning::Plan,
    removal_authority: bool,
}
fn plan(path: &Path) -> Result<planning::Plan> {
    let raw = profile::raw(path, 131072)?;
    let value = identity::parse_json(&raw).map_err(|_| "invalid_document")?;
    if value.get("schema").and_then(Value::as_str) == Some(RESPONSE) {
        let preview: Preview = serde_json::from_value(value).map_err(|_| "invalid_document")?;
        if preview.schema != RESPONSE
            || !preview.ok
            || preview.operation != "plan"
            || preview.removal_authority
            || preview.plan.id().map_err(|e| e.code())? != preview.plan_id
        {
            return Err("invalid_document");
        }
        return Ok(preview.plan);
    }
    planning::Plan::from_json(&identity::canonical_json(&value).map_err(|_| "invalid_document")?)
        .map_err(|e| e.code())
}
fn admit(p: &Profile) -> Result<()> {
    let source = annex::Reader::new(p.source.profile.pool.pool.clone())
        .map_err(|_| "invalid_document")?
        .store();
    let keeper = annex::Reader::new(p.keeper.profile.pool.pool.clone())
        .map_err(|_| "invalid_document")?
        .store();
    p.owner.admit(&source, &keeper).map_err(|e| e.code())?;
    Ok(())
}
fn fresh(p: &Profile, request: Option<Request>, frozen: Option<&planning::Plan>) -> Result<Value> {
    admit(p)?;
    let sw = LocalStore::open(&p.source.witness)
        .map_err(|e| annex::publication::Error::from(accounted::AccountedError::Store(e)).code())?;
    let kw = LocalStore::open(&p.keeper.witness)
        .map_err(|e| annex::publication::Error::from(accounted::AccountedError::Store(e)).code())?;
    let mut sl = CapacityLedger::open(&p.source.ledger, &p.source.minimum).map_err(|e| {
        annex::publication::Error::from(accounted::AccountedError::Ledger(e)).code()
    })?;
    let mut kl = CapacityLedger::open(&p.keeper.ledger, &p.keeper.minimum).map_err(|e| {
        annex::publication::Error::from(accounted::AccountedError::Ledger(e)).code()
    })?;
    let sn = p.source_metadata.open()?;
    let mut kn = p.keeper_metadata.open()?;
    let sources = Sources {
        source: planning::Producer {
            profile: &p.source.profile,
            plan: &p.source.plan,
            witness: &sw,
            ledger: &mut sl,
        },
        keeper: planning::Producer {
            profile: &p.keeper.profile,
            plan: &p.keeper.plan,
            witness: &kw,
            ledger: &mut kl,
        },
    };
    let nodes = Nodes {
        source: &sn,
        keeper: &mut kn,
        limits: &p.limits,
        tree_limits: profile::tree_limits(),
    };
    let controller = Controller::new(&p.owner);
    if let Some(r) = request {
        let preview = controller
            .plan(
                planning::Request {
                    operation: r.operation,
                    snapshot: r.snapshot,
                    grant: r.grant,
                },
                sources,
                nodes,
            )
            .map_err(|e| e.code())?;
        Ok(
            json!({"schema":RESPONSE,"ok":true,"operation":"plan","plan_id":preview.id().map_err(|e| e.code())?,
            "plan":preview,"removal_authority":false}),
        )
    } else {
        let report = controller
            .apply(frozen.ok_or("invalid_document")?, sources, nodes)
            .map_err(|e| e.code())?;
        Ok(
            json!({"schema":RESPONSE,"ok":true,"operation":"apply","plan_id":report.plan,"report":report}),
        )
    }
}
fn execute(op: &str, p: &Profile, frozen: &planning::Plan) -> Result<Value> {
    let controller = Controller::new(&p.owner);
    let report = if op == "status" {
        serde_json::to_value(
            controller
                .status(frozen, &p.source.profile, &p.source.plan)
                .map_err(|e| e.code())?,
        )
    } else if op == "apply" {
        return fresh(p, None, Some(frozen));
    } else {
        let nodes = p.source_metadata.open()?;
        let report = if op == "resume" {
            controller.resume(frozen, &p.source.profile, &p.source.plan, &nodes, &p.limits)
        } else {
            controller.cancel(frozen, &p.source.profile, &p.source.plan, &nodes, &p.limits)
        };
        serde_json::to_value(report.map_err(|e| e.code())?)
    }
    .map_err(|_| "output_unavailable")?;
    Ok(
        json!({"schema":RESPONSE,"ok":true,"operation":op,"plan_id":frozen.id().map_err(|e| e.code())?,"report":report}),
    )
}
pub(super) fn run(args: &[OsString]) -> Result<Value> {
    if args.len() != 3 {
        return Err("usage");
    }
    let op = args[0].to_str().ok_or("usage")?;
    if !["plan", "status", "apply", "resume", "cancel"].contains(&op) {
        return Err("unsupported");
    }
    let p: Profile = profile::parse(&profile::raw(Path::new(&args[1]), 65536)?)?;
    if p.schema != PROFILE {
        return Err("unsupported");
    }
    let permitted = match op {
        "apply" => p.allow_apply,
        "resume" => p.allow_resume,
        "cancel" => p.allow_cancel,
        _ => true,
    };
    if op == "plan" {
        let request: Request = profile::parse(&profile::raw(Path::new(&args[2]), 65536)?)?;
        if request.schema != REQUEST {
            return Err("unsupported");
        }
        if let Some(reviewed) = &request.reviewed_object
            && (reviewed != &request.snapshot.object
                || reviewed != &p.source.plan.candidate().object
                || reviewed != &p.keeper.plan.candidate().object)
        {
            return Err("stale");
        }
        return fresh(&p, Some(request), None);
    }
    let frozen = plan(Path::new(&args[2]))?;
    let id = frozen.id().map_err(|e| e.code())?;
    if !permitted {
        return Ok(failure("denied", Some(&id)));
    }
    Ok(execute(op, &p, &frozen).unwrap_or_else(|code| failure(code, Some(&id))))
}
