//! Explicit native storage route. Discovery never opens a profile/provider.
use serde_json::{Value, json};
use std::{ffi::OsString, io::Write, process::ExitCode};
#[cfg(feature = "annex-publish")]
mod operations;
#[cfg(feature = "annex-publish")]
mod profile;
const RESPONSE: &str = "git-asset.native-storage-response.experimental-1";
const PROFILE: &str = "git-asset.native-storage-profile.experimental-1";
const REQUEST: &str = "git-asset.native-storage-request.experimental-1";
fn retry(code: &str) -> &'static str {
    match code {
        "busy" => "retry_same_operation_with_current_proofs",
        "stale" | "conflict" => "inspect_original_before_replanning",
        "uncertain" | "needs_recovery" | "output_unavailable" => "inspect_original_operation",
        "missing_acknowledged" | "protection_lost" => "incident_recovery",
        "missing" | "corrupt" => "inspect_and_repair_under_existing_protection",
        _ => "correct_input_or_admission",
    }
}
fn failure(code: &str, plan: Option<&git_asset_core::Digest>) -> Value {
    json!({"schema":RESPONSE,"ok":false,"code":code,"retry":retry(code),
        "plan_id":plan,"reconcile":"storage native status PROFILE SAVED_PLAN"})
}
fn describe() -> Value {
    json!({"schema":"git-asset.native-storage-interface.experimental-1",
        "profile_schema":PROFILE,"request_schema":REQUEST,"response_schema":RESPONSE,
        "available":cfg!(feature="annex-publish"),"experimental":true,
        "operations":["plan PROFILE REQUEST","status PROFILE SAVED_PLAN",
            "apply PROFILE SAVED_PLAN","resume PROFILE SAVED_PLAN","cancel PROFILE SAVED_PLAN"],
        "optional_reviewed_object":true,"profile_request_bytes":65536,"saved_plan_response_bytes":131072,
        "canonical_plan_bytes":65536,"initializes_providers":false,
        "automatic_operations":false,"mixed_legacy_supported":false,
        "mutations_default_enabled":false,"production_qualified":false})
}
pub(super) fn main(args: Vec<OsString>) -> ExitCode {
    let value = if args.len() == 1 && args[0] == "describe" {
        describe()
    } else if args.is_empty()
        || (args.len() == 1 && matches!(args[0].to_str(), Some("help" | "--help" | "-h")))
    {
        json!({"schema":RESPONSE,"ok":true,"usage":"git asset storage native describe | plan PROFILE REQUEST | status/apply/resume/cancel PROFILE SAVED_PLAN"})
    } else {
        run(&args).unwrap_or_else(|code| failure(code, None))
    };
    let (success, mut bytes) = match serde_json::to_vec(&value) {
        Ok(bytes) if bytes.len() < 131072 =>
            (value.get("ok").and_then(Value::as_bool).unwrap_or(true), bytes),
        _ => (false, br#"{"schema":"git-asset.native-storage-response.experimental-1","ok":false,"code":"output_unavailable","retry":"inspect_original_operation"}"#.to_vec()),
    };
    bytes.push(b'\n');
    if std::io::stdout().lock().write_all(&bytes).is_err() {
        let _ = writeln!(
            std::io::stderr().lock(),
            "{{\"schema\":\"{RESPONSE}\",\"ok\":false,\"code\":\"output_unavailable\",\"retry\":\"inspect_original_operation\"}}"
        );
        return ExitCode::from(2);
    }
    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}
#[cfg(feature = "annex-publish")]
fn run(args: &[OsString]) -> Result<Value, &'static str> {
    operations::run(args)
}
#[cfg(not(feature = "annex-publish"))]
fn run(_: &[OsString]) -> Result<Value, &'static str> {
    Err("unsupported")
}

#[cfg(feature = "annex-publish")]
mod archive;
#[cfg(feature = "annex-publish")]
pub(super) use archive::main as archive;
#[cfg(not(feature = "annex-publish"))]
pub(super) fn archive(_: Vec<OsString>) -> ExitCode {
    println!("{}", json!({"ok":false,"code":"unsupported"}));
    ExitCode::from(2)
}
