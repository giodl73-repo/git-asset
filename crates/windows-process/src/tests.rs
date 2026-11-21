use super::*;
use std::{
    fs,
    io::{Read, Write},
    os::windows::{io::AsHandle, process::CommandExt},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn command(root: &std::path::Path, mode: &str) -> Command {
    let mut c = Command::new(std::env::current_exe().unwrap());
    c.args(["--exact", "tests::worker", "--nocapture"])
        .env("GIT_ASSET_JOB_FIXTURE", root)
        .env("GIT_ASSET_JOB_MODE", mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x08000000);
    c
}
fn until(mut predicate: impl FnMut() -> bool) {
    let start = Instant::now();
    while !predicate() {
        assert!(
            start.elapsed() < Duration::from_secs(8),
            "bounded job fixture expired"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
fn scope(root: &std::path::Path) -> String {
    // tempfile basename is unique; fixed-width lowercase scope without another hash dependency.
    let bytes = root.file_name().unwrap().to_str().unwrap().as_bytes();
    let mut name = String::new();
    for b in bytes.iter().copied().cycle().take(32) {
        name.push_str(&format!("{b:02x}"));
    }
    name
}
#[test]
fn invalid_names_refuse_and_queries_never_create_jobs() {
    for invalid in ["", "../scope", "a\0b", &"a".repeat(65), &"A".repeat(64)] {
        assert!(Job::join(invalid).is_err());
        assert!(Job::open(invalid).is_err());
    }
    let root = tempfile::tempdir().unwrap();
    let scope = scope(root.path());
    assert!(Job::open(&scope).unwrap().is_none());
    let job = Job::join(&scope).unwrap();
    assert_eq!(job.active().unwrap(), 0);
    assert_eq!(Job::open(&scope).unwrap().unwrap().active().unwrap(), 0);
    drop(job);
    assert!(Job::open(&scope).unwrap().is_none());
}
#[test]
fn existing_foreign_limits_are_refused_without_repair() {
    let root = tempfile::tempdir().unwrap();
    let scope = scope(root.path());
    let job = Job::join(&scope).unwrap();
    let info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    // SAFETY: owned empty fixture job, initialized matching structure and exact ABI size.
    assert_ne!(
        unsafe {
            SetInformationJobObject(
                job.0.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of_val(&info) as u32,
            )
        },
        0
    );
    assert_eq!(
        Job::open(&scope).unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        Job::join(&scope).unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        job.active().unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
}
#[test]
fn lost_name_and_recreated_empty_job_do_not_prove_old_process_exit() {
    let root = tempfile::tempdir().unwrap();
    let scope = scope(root.path());
    let job = Job::join(&scope).unwrap();
    let mut child = command(root.path(), "blocked").spawn().unwrap();
    job.assign(child.as_handle()).unwrap();
    let info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    // SAFETY: owned fixture job, exact initialized structure/size; remove kill-on-close
    // only to expose name lifetime independently from the asynchronous termination request.
    assert_ne!(
        unsafe {
            SetInformationJobObject(
                job.0.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of_val(&info) as u32,
            )
        },
        0
    );
    drop(job);
    assert!(child.try_wait().unwrap().is_none());
    assert!(Job::open(&scope).unwrap().is_none());
    let replacement = Job::join(&scope).unwrap();
    assert_eq!(replacement.active().unwrap(), 0);
    assert!(child.try_wait().unwrap().is_none());
    child.kill().unwrap();
    child.wait().unwrap();
}
#[test]
fn reopened_job_counts_grandchild_after_immediate_parent_exits() {
    let root = tempfile::tempdir().unwrap();
    let scope = scope(root.path());
    let job = Job::join(&scope).unwrap();
    let mut child = command(root.path(), "spawn").spawn().unwrap();
    job.assign(child.as_handle()).unwrap();
    child.stdin.take().unwrap().write_all(b"G").unwrap();
    until(|| root.path().join("grandchild").exists());
    assert!(child.wait().unwrap().success());
    let reopened = Job::open(&scope).unwrap().unwrap();
    drop(job);
    assert!(reopened.active().unwrap() > 0);
    reopened.terminate().unwrap();
    until(|| reopened.active().unwrap() == 0);
    drop(reopened);
    assert!(Job::open(&scope).unwrap().is_none());
}
#[test]
fn recovering_handle_survives_actual_controller_exit_and_fences_descendants() {
    let root = tempfile::tempdir().unwrap();
    let scope = scope(root.path());
    fs::write(root.path().join("scope"), &scope).unwrap();
    let mut controller = command(root.path(), "controller").spawn().unwrap();
    until(|| root.path().join("grandchild").exists());
    let recovery = Job::open(&scope).unwrap().unwrap();
    assert!(recovery.active().unwrap() > 0);
    controller.stdin.take().unwrap().write_all(b"X").unwrap();
    assert_eq!(controller.wait().unwrap().code(), Some(77));
    assert!(recovery.active().unwrap() > 0);
    recovery.terminate().unwrap();
    until(|| recovery.active().unwrap() == 0);
}
#[test]
fn worker() {
    let Some(root) = std::env::var_os("GIT_ASSET_JOB_FIXTURE") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    match std::env::var("GIT_ASSET_JOB_MODE").unwrap().as_str() {
        "blocked" => {
            std::io::stdin().read_exact(&mut [0]).unwrap();
        }
        "controller" => {
            let scope = fs::read_to_string(root.join("scope")).unwrap();
            let job = Job::join(&scope).unwrap();
            let mut child = command(&root, "spawn").spawn().unwrap();
            job.assign(child.as_handle()).unwrap();
            child.stdin.take().unwrap().write_all(b"G").unwrap();
            until(|| root.join("grandchild").exists());
            assert!(child.wait().unwrap().success());
            std::io::stdin().read_exact(&mut [0]).unwrap();
            std::process::exit(77);
        }
        "spawn" => {
            std::io::stdin().read_exact(&mut [0]).unwrap();
            // Descendant deliberately outlives its immediate parent, inheriting the job.
            let child = command(&root, "grandchild")
                .stdin(Stdio::null())
                .spawn()
                .unwrap();
            drop(child);
        }
        "grandchild" => {
            fs::write(root.join("grandchild"), b"owned synthetic process").unwrap();
            thread::sleep(Duration::from_secs(30));
        }
        _ => panic!("unknown owned fixture mode"),
    }
}
