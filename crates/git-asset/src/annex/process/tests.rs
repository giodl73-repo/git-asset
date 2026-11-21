use super::*;
use std::{
    fs, io::Read, os::windows::io::AsHandle, os::windows::process::CommandExt, process::Command,
    thread, time::Instant,
};
fn profile(root: &std::path::Path) -> Profile {
    let identity = Digest::blob(root.to_string_lossy().as_bytes());
    let id = identity.as_str();
    fs::write(root.join("empty"), b"").unwrap();
    fs::create_dir(root.join("hooks")).unwrap();
    Profile {
        git: std::env::current_exe().unwrap(),
        annex: std::env::current_exe().unwrap(),
        repository: root.into(),
        bare: false,
        uuid: format!(
            "{}-{}-{}-{}-{}",
            &id[..8],
            &id[8..12],
            &id[12..16],
            &id[16..20],
            &id[20..32]
        ),
        configuration: Object::from_bytes(b"owned process fixture"),
        tools: vec![root.into()],
        empty_config: root.join("empty"),
        empty_hooks: root.join("hooks"),
        temporary: root.into(),
        command_millis: 5000,
        output_bytes: 4096,
        limits: Limits::default(),
    }
}
fn fixture_root() -> Option<std::path::PathBuf> {
    let root = std::path::PathBuf::from(std::env::var_os("USERPROFILE")?);
    root.join("job-scope").is_file().then_some(root)
}
#[test]
fn assignment_precedes_first_effect_and_wait_includes_pipe_detached_grandchild() {
    let root = tempfile::tempdir().unwrap();
    let profile = profile(root.path());
    let object = Object::from_bytes(b"owned source");
    let scope = object_scope(&profile, &object).unwrap();
    fs::write(root.path().join("job-scope"), scope.as_str()).unwrap();
    let output = run_for_object(
        &profile,
        &object,
        false,
        &["--exact", "annex::process::tests::worker", "--nocapture"],
    )
    .unwrap();
    assert!(root.path().join("attached-before-effect").exists());
    assert!(root.path().join("grandchild-finished").exists());
    assert_eq!(
        git_asset_windows_process::Job::open(scope.as_str())
            .unwrap()
            .unwrap()
            .active()
            .unwrap(),
        0
    );
    drop(output);
    assert!(
        git_asset_windows_process::Job::open(scope.as_str())
            .unwrap()
            .is_none()
    );
}
#[test]
fn original_command_group_guard_composes_with_durable_one_shot_witness() {
    use git_asset_store::{
        Key,
        ledger::CapacityLedger,
        local::LocalStore,
        upload_witness::{Document, Packet},
    };
    let root = tempfile::tempdir().unwrap();
    let profile = profile(root.path());
    let object = Object::from_bytes(b"owned source");
    let scope = object_scope(&profile, &object).unwrap();
    fs::write(root.path().join("job-scope"), scope.as_str()).unwrap();
    let key = |s: &str| Key::parse(s).unwrap();
    let witness = LocalStore::create(&root.path().join("witness"), key("witness")).unwrap();
    let packet = Packet::new(
        &witness,
        &scope,
        &key("owner"),
        [
            Document {
                operation: key("intent"),
                bytes: br#"{"phase":"intent"}"#.to_vec(),
            },
            Document {
                operation: key("done"),
                bytes: br#"{"phase":"done"}"#.to_vec(),
            },
        ],
    )
    .unwrap();
    let mut ledger = CapacityLedger::create(
        &root.path().join("ledger"),
        key("capacity"),
        key("epoch"),
        0,
    )
    .unwrap();
    let probe = ledger
        .begin_probe(key("probe"))
        .unwrap()
        .reference()
        .clone();
    ledger
        .observe(key("sample"), &probe, 64 * 1024 * 1024)
        .unwrap();
    let job = ledger
        .reserve(key("job"), key("owner"), packet.required_peak().unwrap())
        .unwrap()
        .reference()
        .clone();
    let completed = packet
        .apply_once(&witness, &mut ledger, &job, || {
            let output = run_for_object(
                &profile,
                &object,
                false,
                &["--exact", "annex::process::tests::worker", "--nocapture"],
            )
            .map_err(|_| git_asset_store::upload_witness::Error::NeedsRecovery)?;
            assert_eq!(
                git_asset_windows_process::Job::open(scope.as_str())
                    .unwrap()
                    .unwrap()
                    .active()
                    .unwrap(),
                0
            );
            Ok(output)
        })
        .unwrap();
    assert_eq!(
        completed.completion().request().object,
        Object::from_bytes(br#"{"phase":"done"}"#)
    );
    assert!(
        git_asset_windows_process::Job::open(scope.as_str())
            .unwrap()
            .is_none()
    );
    let before = ledger.checkpoint();
    packet
        .apply_once(
            &witness,
            &mut ledger,
            &job,
            || -> git_asset_store::upload_witness::Result<()> {
                panic!("recovered completion cannot rerun the native command")
            },
        )
        .unwrap();
    assert_eq!(ledger.checkpoint(), before);
}
#[test]
fn live_old_command_blocks_retry_and_object_scoped_observations_stay_distinct() {
    let root = tempfile::tempdir().unwrap();
    let profile = profile(root.path());
    let object = Object::from_bytes(b"owned source");
    let scope = object_scope(&profile, &object).unwrap();
    let job = git_asset_windows_process::Job::join(scope.as_str()).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "annex::process::tests::paused_worker",
            "--nocapture",
        ])
        .env("USERPROFILE", root.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x08000000)
        .spawn()
        .unwrap();
    job.assign(child.as_handle()).unwrap();
    let held = git_asset_windows_process::Job::open(scope.as_str())
        .unwrap()
        .unwrap();
    assert!(held.active().unwrap() > 0);
    let unrelated = object_scope(&profile, &Object::from_bytes(b"other")).unwrap();
    assert!(
        git_asset_windows_process::Job::open(unrelated.as_str())
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        run_for_object(
            &profile,
            &object,
            false,
            &["--exact", "annex::process::tests::worker", "--nocapture"]
        ),
        Err(Error::Observation)
    ));
    assert!(!root.path().join("attached-before-effect").exists());
    job.terminate().unwrap();
    child.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while held.active().unwrap() != 0 {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn detached_descendant_timeout_is_not_acknowledged_as_command_completion() {
    let root = tempfile::tempdir().unwrap();
    let mut profile = profile(root.path());
    profile.command_millis = 1500;
    let object = Object::from_bytes(b"owned source");
    let scope = object_scope(&profile, &object).unwrap();
    fs::write(root.path().join("job-scope"), scope.as_str()).unwrap();
    fs::write(root.path().join("delay-ms"), "5000").unwrap();
    assert!(matches!(
        run_for_object(
            &profile,
            &object,
            false,
            &["--exact", "annex::process::tests::worker", "--nocapture"]
        ),
        Err(Error::Observation)
    ));
    assert!(root.path().join("attached-before-effect").exists());
    assert!(!root.path().join("grandchild-finished").exists());
}
#[test]
fn worker() {
    let Some(root) = fixture_root() else {
        return;
    };
    let scope = fs::read_to_string(root.join("job-scope")).unwrap();
    // This is the child's first observable effect. No other process uses this fresh scope.
    assert!(
        git_asset_windows_process::Job::open(&scope)
            .unwrap()
            .unwrap()
            .active()
            .unwrap()
            > 0
    );
    fs::write(root.join("attached-before-effect"), b"assigned").unwrap();
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "annex::process::tests::grandchild",
            "--nocapture",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x08000000)
        .spawn()
        .unwrap();
    drop(child);
}
#[test]
fn grandchild() {
    let Some(root) = fixture_root() else {
        return;
    };
    let delay = fs::read_to_string(root.join("delay-ms"))
        .ok()
        .map(|v| v.parse::<u64>().unwrap().min(5000))
        .unwrap_or(200);
    thread::sleep(Duration::from_millis(delay));
    fs::write(root.join("grandchild-finished"), b"finished").unwrap();
}
#[test]
fn paused_worker() {
    if std::env::var_os("USERPROFILE")
        .is_some_and(|p| std::path::Path::new(&p).join("empty").is_file())
    {
        std::io::stdin().read_exact(&mut [0]).unwrap();
    }
}
