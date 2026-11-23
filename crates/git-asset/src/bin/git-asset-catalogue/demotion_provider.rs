use git_asset_core::Digest;
use process_wrap::tokio::{CommandWrap, JobObject, KillOnDrop};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::Read,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    schema: String,
    executable: PathBuf,
    executable_sha256: Digest,
    project_root: PathBuf,
    journal: PathBuf,
    request_sha256: Digest,
    drive_report: PathBuf,
    drive_report_sha256: Digest,
    #[serde(default)]
    allow_apply: bool,
}

fn guarded_file(path: &Path, limit: u64) -> Result<(File, Vec<u8>), String> {
    if !path.is_absolute() {
        return Err("Provider inputs require absolute paths".into());
    }
    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .custom_flags(0x0020_0000)
        .open(path)
        .map_err(|_| "Cannot hold provider input")?;
    let metadata = file
        .metadata()
        .map_err(|_| "Cannot inspect provider input")?;
    if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 || metadata.len() > limit {
        return Err("Provider input is outside file or byte limits".into());
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read provider input")?;
    if bytes.len() as u64 > limit {
        return Err("Provider input exceeds byte limit".into());
    }
    Ok((file, bytes))
}

pub(super) fn run(
    action: &str,
    profile: &Path,
    request: &Path,
    target: &Path,
) -> Result<Value, String> {
    let status = action == "status";
    let apply = match action {
        "verify" | "status" => false,
        "apply" => true,
        _ => return Err("Expected verify, apply or status".into()),
    };
    let (_profile_hold, profile_raw) = guarded_file(profile, 65536)?;
    let profile: Profile =
        serde_json::from_slice(&profile_raw).map_err(|_| "Invalid provider profile")?;
    if profile.schema != "git-asset.reel-demotion-provider.experimental-1"
        || !profile.project_root.is_absolute()
        || !profile.journal.is_absolute()
        || (!status && profile.journal != target)
        || (apply && !profile.allow_apply)
    {
        return Err("Provider profile does not admit this operation".into());
    }
    let (_request_hold, request_raw) = guarded_file(request, 256 * 1024)?;
    if Digest::blob(&request_raw) != profile.request_sha256 {
        return Err("Provider request pin mismatch".into());
    }
    let document: Value =
        serde_json::from_slice(&request_raw).map_err(|_| "Invalid provider request")?;
    let (_drive_hold, drive_raw) = guarded_file(&profile.drive_report, 16 * 1024 * 1024)?;
    if Digest::blob(&drive_raw) != profile.drive_report_sha256 {
        return Err("Drive report pin mismatch".into());
    }
    drop(drive_raw);
    let objects = super::demotion::validate_prepared(&profile.drive_report, &document)?;
    let plan = &document["plan_sha256"];
    if status {
        return inspect_saved(&profile, target, plan, &objects);
    }
    let (_executable_hold, executable) = guarded_file(&profile.executable, 128 * 1024 * 1024)?;
    if Digest::blob(&executable) != profile.executable_sha256 {
        return Err("Provider executable pin mismatch".into());
    }
    drop(executable);
    let seconds = document["max_seconds"]
        .as_u64()
        .filter(|n| (1..=3600).contains(n))
        .ok_or("Invalid provider time budget")?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Cannot start provider runtime")?;
    let result = runtime
        .block_on(execute(&profile, request, target, apply, seconds))
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok())
        .filter(|result| matches_receipt(result, &profile.request_sha256, plan, &objects, apply));
    result.ok_or_else(|| {
        if apply {
            "Provider outcome uncertain; inspect the original journal before retrying".into()
        } else {
            "Provider verification did not complete; inspect the original journal".into()
        }
    })
}

fn inspect_saved(
    profile: &Profile,
    result_path: &Path,
    plan: &Value,
    objects: &BTreeMap<String, u64>,
) -> Result<Value, String> {
    if result_path.parent() != Some(profile.journal.as_path())
        || result_path
            .file_name()
            .and_then(|s| s.to_str())
            .is_none_or(|name| !name.starts_with("result-"))
    {
        return Err("Select a result file in the pinned original journal".into());
    }
    let (_stamp_hold, stamp_raw) = guarded_file(&profile.journal.join("request.json"), 65536)?;
    let stamp: Value =
        serde_json::from_slice(&stamp_raw).map_err(|_| "Malformed journal request stamp")?;
    let apply = stamp["apply"].as_bool().ok_or("Missing journal action")?;
    if stamp["request_sha256"] != profile.request_sha256.as_str() || stamp["plan_sha256"] != *plan {
        return Err("Journal request stamp conflicts with pinned request".into());
    }
    let (_result_hold, result_raw) = guarded_file(result_path, 1024 * 1024)?;
    let result: Value =
        serde_json::from_slice(&result_raw).map_err(|_| "Malformed provider receipt")?;
    if !matches_receipt(&result, &profile.request_sha256, plan, objects, apply) {
        return Err("Receipt conflicts with pinned request or journal action".into());
    }
    Ok(json!({
        "schema":"git-asset.reel-demotion-status.experimental-1","historical_receipt":result,
        "receipt_sha256":Digest::blob(&result_raw),"current_storage_checked":false,
        "removal_authorized":false
    }))
}

fn matches_receipt(
    result: &Value,
    request: &Digest,
    plan: &Value,
    objects: &BTreeMap<String, u64>,
    apply: bool,
) -> bool {
    result["schema"] == "reel.asset-demotion-result.v1"
        && result["apply"] == apply
        && result["request_sha256"] == request.as_str()
        && result["plan_sha256"] == *plan
        && result["last_copy_retired"] == false
        && result["archive_probed"] == false
        && exact_results(result, objects, apply)
}

fn exact_results(report: &Value, expected: &BTreeMap<String, u64>, apply: bool) -> bool {
    let Some(rows) = report["objects"].as_array() else {
        return false;
    };
    if rows.len() != expected.len() {
        return false;
    }
    let received: Option<BTreeMap<_, _>> = rows
        .iter()
        .map(|row| {
            let removed = row["hot_links_removed"].as_u64()?;
            if !apply && removed != 0 {
                return None;
            }
            Some((row["sha256"].as_str()?.to_owned(), row["bytes"].as_u64()?))
        })
        .collect();
    received.as_ref() == Some(expected)
}

async fn capture(reader: impl AsyncRead + Unpin) -> std::io::Result<Vec<u8>> {
    const LIMIT: usize = 1024 * 1024;
    let mut bytes = Vec::new();
    reader
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .await?;
    if bytes.len() > LIMIT {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "provider output limit",
        ));
    }
    Ok(bytes)
}

async fn execute(
    profile: &Profile,
    request: &Path,
    journal: &Path,
    apply: bool,
    seconds: u64,
) -> Option<Vec<u8>> {
    let mut command = CommandWrap::with_new(&profile.executable, |command| {
        command
            .current_dir(&profile.project_root)
            .arg("--root")
            .arg(&profile.project_root)
            .arg("tier-demote")
            .arg(request)
            .arg(journal)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if apply {
            command.arg("--apply");
        }
    });
    command.wrap(KillOnDrop).wrap(JobObject);
    let mut child = command.spawn().ok()?;
    let out = child.stdout().take()?;
    let err = child.stderr().take()?;
    let (out, err, status) = tokio::time::timeout(Duration::from_secs(seconds + 5), async {
        tokio::try_join!(capture(out), capture(err), child.wait())
    })
    .await
    .ok()?
    .ok()?;
    (status.success() && err.is_empty()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn checks_exact_receipt_and_refuses_a_verify_removal() {
        let expected = BTreeMap::from([("first".to_owned(), 10), ("second".to_owned(), 20)]);
        let first = json!({"sha256":"first","bytes":10,"hot_links_removed":0});
        let second = json!({"sha256":"second","bytes":20,"hot_links_removed":0});
        let mut receipt = json!({"objects":[first, second]});
        assert!(exact_results(&receipt, &expected, false));
        receipt["objects"][1]["hot_links_removed"] = json!(1);
        assert!(!exact_results(&receipt, &expected, false));
        assert!(exact_results(&receipt, &expected, true));
        receipt["objects"][1]["bytes"] = json!(21);
        assert!(!exact_results(&receipt, &expected, true));
        receipt["objects"][1] = first;
        assert!(!exact_results(&receipt, &expected, true));
    }
}
