use super::demotion_provider::{guarded_file, run_provider};
use git_asset_core::Digest;
use serde::Deserialize;
use serde_json::Value;
use std::{
    fs,
    io::ErrorKind,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    schema: String,
    executable: PathBuf,
    executable_sha256: Digest,
    job_profile_sha256: Digest,
    cache_root: PathBuf,
    workspace: PathBuf,
    max_millis: u64,
    #[serde(default)]
    allow_apply: bool,
}

pub(super) fn run(action: &str, profile_path: &Path, job: &Path) -> Result<Value, String> {
    let apply = match action {
        "verify" => false,
        "apply" => true,
        _ => return Err("Expected verify or apply".into()),
    };
    let (_profile_hold, raw) = guarded_file(profile_path, 65536)?;
    let profile: Profile =
        serde_json::from_slice(&raw).map_err(|_| "Invalid scratch provider profile")?;
    if profile.schema != "git-asset.reel-scratch-provider.experimental-1"
        || !profile.cache_root.is_absolute()
        || !profile.workspace.is_absolute()
        || !(1..=3_600_000).contains(&profile.max_millis)
        || (apply && !profile.allow_apply)
    {
        return Err("Scratch provider profile does not admit this operation".into());
    }
    let (_job_hold, raw) = guarded_file(job, 128 * 1024)?;
    if Digest::blob(&raw) != profile.job_profile_sha256 {
        return Err("Job profile pin mismatch".into());
    }
    let job_document: Value = serde_json::from_slice(&raw).map_err(|_| "Invalid job profile")?;
    let job_id = job_document["job_id"]
        .as_str()
        .ok_or("Missing job identity")?;
    let workspace = job_document["retirement"]["workspace"]
        .as_str()
        .map(Path::new);
    if job_document["schema"] != "reel.tig-workspace-retirement-profile.experimental-1"
        || workspace != Some(profile.workspace.as_path())
        || profile.workspace.file_name().and_then(|name| name.to_str()) != Some(job_id)
        || profile.workspace.parent() != Some(profile.cache_root.join("staging/caminos").as_path())
    {
        return Err("Job profile does not name the pinned owned workspace".into());
    }
    let canonical_cache = profile
        .cache_root
        .canonicalize()
        .map_err(|_| "Cannot resolve enrolled cache")?;
    let canonical_parent = profile
        .workspace
        .parent()
        .ok_or("Missing workspace parent")?
        .canonicalize()
        .map_err(|_| "Cannot resolve workspace parent")?;
    if canonical_parent != canonical_cache.join("staging/caminos") {
        return Err("Resolved workspace parent is outside enrolled staging".into());
    }
    match fs::symlink_metadata(&profile.workspace) {
        Ok(metadata)
            if metadata.is_dir()
                && metadata.file_attributes() & 0x400 == 0
                && profile.workspace.canonicalize().ok() == Some(canonical_parent.join(job_id)) => {
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        _ => return Err("Resolved workspace is not the exact enrolled job directory".into()),
    }
    let (_executable_hold, executable) = guarded_file(&profile.executable, 128 * 1024 * 1024)?;
    if Digest::blob(&executable) != profile.executable_sha256 {
        return Err("Scratch executable pin mismatch".into());
    }
    drop(executable);
    let result = run_provider(
        &profile.executable,
        Duration::from_millis(profile.max_millis),
        |command| {
            command
                .current_dir(canonical_cache)
                .arg("retire-workspace")
                .arg(job);
            if apply {
                command.arg("--apply");
            }
        },
    );
    result
        .filter(|report| {
            report["schema"] == "reel.tig-workspace-retirement.experimental-1"
                && report["job_id"] == job_id
                && report["dry_run"] == !apply
                && report["staging_retired"] == apply
                && report["cache_objects_removed"] == 0
        })
        .ok_or_else(|| {
            if apply {
                "Scratch outcome uncertain; inspect the original retirement transaction".into()
            } else {
                "Scratch verification did not complete".into()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn refuses_workspace_outside_staging_before_opening_the_provider() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let workspace = root.join("unowned/job");
        let request = serde_json::to_vec(&json!({
            "schema":"reel.tig-workspace-retirement-profile.experimental-1",
            "job_id":"job","retirement":{"workspace":workspace}
        }))
        .unwrap();
        let job = root.join("job.json");
        fs::write(&job, &request).unwrap();
        let profile = root.join("provider.json");
        fs::write(
            &profile,
            serde_json::to_vec(&json!({
                "schema":"git-asset.reel-scratch-provider.experimental-1",
                "executable":root.join("unavailable.exe"),
                "executable_sha256":Digest::blob(b"unused executable"),
                "job_profile_sha256":Digest::blob(&request),
                "cache_root":root,"workspace":workspace,
                "max_millis":1000,"allow_apply":true
            }))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            run("apply", &profile, &job).unwrap_err(),
            "Job profile does not name the pinned owned workspace"
        );
    }
}
