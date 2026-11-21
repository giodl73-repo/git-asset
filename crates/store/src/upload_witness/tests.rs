use super::*;
use crate::ledger::Checkpoint;
use std::{fs, path::Path, process::Command};
fn key(s: &str) -> Key {
    Key::parse(s).unwrap()
}
fn packet(store: &LocalStore, context: &[u8]) -> Packet {
    Packet::new(
        store,
        &Digest::blob(context),
        &key("owner"),
        [
            Document {
                operation: key("effect-intent"),
                bytes: br#"{"phase":"intent"}"#.to_vec(),
            },
            Document {
                operation: key("effect-done"),
                bytes: br#"{"phase":"done"}"#.to_vec(),
            },
        ],
    )
    .unwrap()
}
struct Fixture {
    root: tempfile::TempDir,
    minimum: Checkpoint,
    job: JournalRef,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let s = LocalStore::create(&root.path().join("witness"), key("witness")).unwrap();
        let p = packet(&s, b"original birth and native operation");
        let mut l =
            CapacityLedger::create(&root.path().join("ledger"), key("volume"), key("epoch"), 0)
                .unwrap();
        let minimum = l.checkpoint();
        let probe = l.begin_probe(key("probe")).unwrap().reference().clone();
        l.observe(key("sample"), &probe, 512 * 1024 * 1024).unwrap();
        let job = l
            .reserve(key("job"), key("owner"), p.required_peak().unwrap())
            .unwrap()
            .reference()
            .clone();
        fs::write(
            root.path().join("saved.json"),
            serde_json::to_vec(&(&minimum, &job)).unwrap(),
        )
        .unwrap();
        Self { root, minimum, job }
    }
    fn open(&self) -> (LocalStore, CapacityLedger, Packet) {
        let s = LocalStore::open(&self.root.path().join("witness")).unwrap();
        let l = CapacityLedger::open(&self.root.path().join("ledger"), &self.minimum).unwrap();
        let p = packet(&s, b"original birth and native operation");
        (s, l, p)
    }
}
#[test]
fn exact_completed_reopen_is_readonly_and_rejects_wrong_context_job_missing_and_corrupt_bytes() {
    let f = Fixture::new();
    let (s, mut l, p) = f.open();
    assert!(p.reopen(&s, &mut l, &f.job).is_err());
    p.apply_once(&s, &mut l, &f.job, || Ok(())).unwrap();
    let probe = l
        .begin_probe(key("other-probe"))
        .unwrap()
        .reference()
        .clone();
    l.observe(key("other-sample"), &probe, 512 * 1024 * 1024)
        .unwrap();
    let other_job = l
        .reserve(key("other-job"), key("owner"), p.required_peak().unwrap())
        .unwrap()
        .reference()
        .clone();
    l.finish(key("close"), &f.job).unwrap();
    let before = l.checkpoint();
    let held = p
        .apply_once(&s, &mut l, &f.job, || -> Result<()> {
            panic!("completed replay must not run effect")
        })
        .unwrap();
    assert_eq!(l.checkpoint(), before);
    let path = f
        .root
        .path()
        .join("witness/objects")
        .join(held.completion().request().object.sha256.as_str());
    assert!(matches!(
        s.existing_exclusive_object(&held.completion().request().object),
        Err(crate::Error::Busy)
    ));
    drop(held);
    s.existing_exclusive_object(&p.entries[1].request.object)
        .unwrap();
    assert!(
        packet(&s, b"other birth")
            .reopen(&s, &mut l, &f.job)
            .is_err()
    );
    // A different settled job binding cannot authenticate the same packet.
    assert!(p.reopen(&s, &mut l, &other_job).is_err());
    assert_eq!(l.checkpoint(), before);
    fs::write(&path, b"corrupt").unwrap();
    assert!(p.reopen(&s, &mut l, &f.job).is_err());
    fs::remove_file(&path).unwrap();
    assert!(p.reopen(&s, &mut l, &f.job).is_err());
    assert_eq!(l.checkpoint(), before);
    assert!(!path.exists());
}
#[test]
fn failed_effect_is_uncertain_and_never_repeated_even_after_external_bytes_change() {
    let f = Fixture::new();
    let (s, mut l, p) = f.open();
    let mut calls = 0;
    assert!(matches!(
        p.apply_once(&s, &mut l, &f.job, || -> Result<()> {
            calls += 1;
            Err(Error::NeedsRecovery)
        }),
        Err(OnceError::Uncertain)
    ));
    assert_eq!(calls, 1);
    fs::write(
        f.root.path().join("external-bytes"),
        b"different later bytes",
    )
    .unwrap();
    let before = l.checkpoint();
    assert!(matches!(
        p.apply_once(&s, &mut l, &f.job, || {
            calls += 1;
            Ok(())
        }),
        Err(OnceError::Uncertain)
    ));
    assert_eq!(calls, 1);
    // Reinspection/apply may revisit existing admissions, but never clears intent.
    assert!(p.inspect(&s, &mut l, &f.job).unwrap().intent);
    assert!(!p.inspect(&s, &mut l, &f.job).unwrap().completion);
    assert_eq!(l.checkpoint(), before);
}
#[test]
fn returned_effect_guard_outlives_settlement_and_exact_reopen() {
    struct Guard<'a> {
        store: &'a LocalStore,
        packet: &'a Packet,
        root: &'a Path,
        minimum: &'a Checkpoint,
        job: &'a JournalRef,
    }
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            let mut l = CapacityLedger::open(&self.root.join("ledger"), self.minimum).unwrap();
            self.packet.reopen(self.store, &mut l, self.job).unwrap();
            fs::write(self.root.join("guard-dropped-after-reopen"), b"observed").unwrap();
        }
    }
    let f = Fixture::new();
    let (s, mut l, p) = f.open();
    p.apply_once(&s, &mut l, &f.job, || {
        Ok(Guard {
            store: &s,
            packet: &p,
            root: f.root.path(),
            minimum: &f.minimum,
            job: &f.job,
        })
    })
    .unwrap();
    assert!(f.root.path().join("guard-dropped-after-reopen").is_file());
}
#[test]
fn competing_original_attempt_is_busy_and_never_runs_a_second_effect() {
    let f = Fixture::new();
    let (s, mut l, p) = f.open();
    let (other_store, mut other_ledger, other_packet) = f.open();
    let mut calls = 0;
    p.apply_once(&s, &mut l, &f.job, || {
        calls += 1;
        assert!(matches!(
            other_packet.apply_once(&other_store, &mut other_ledger, &f.job, || -> Result<()> {
                panic!("competing callback must not run")
            }),
            Err(OnceError::Witness(Error::Busy))
        ));
        Ok(())
    })
    .unwrap();
    assert_eq!(calls, 1);
    other_packet
        .reopen(&other_store, &mut other_ledger, &f.job)
        .unwrap();
}
#[test]
fn recorded_completion_can_finish_settlement_without_repeating_effect() {
    let f = Fixture::new();
    let (s, mut l, p) = f.open();
    drop(p.begin(&s, &mut l, &f.job).unwrap());
    let done = &p.entries[1];
    let allocation = accounted::admit(&mut l, &s, &f.job, &done.plan).unwrap();
    let mut attempt = accounted::prepare(&mut l, &s, &f.job, &done.plan, &allocation).unwrap();
    // Reproduce a crash after completion bytes were published, before ledger settlement.
    crate::publish(
        &mut attempt.store,
        &done.request,
        &mut Cursor::new(&done.bytes),
    )
    .unwrap();
    drop(attempt);
    let state = p.inspect(&s, &mut l, &f.job).unwrap();
    assert!(state.completion && !state.settled);
    p.apply_once(&s, &mut l, &f.job, || -> Result<()> {
        panic!("completion settlement must not repeat the external effect")
    })
    .unwrap();
    assert!(p.inspect(&s, &mut l, &f.job).unwrap().settled);
}
#[test]
fn actual_exit_inside_effect_preserves_original_intent_without_second_effect() {
    let f = Fixture::new();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "upload_witness::tests::exit_worker",
            "--nocapture",
        ])
        .env("GIT_ASSET_EFFECT_FIXTURE", f.root.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(77));
    assert_eq!(
        fs::read(f.root.path().join("effect-count")).unwrap(),
        b"one"
    );
    let (s, mut l, p) = f.open();
    assert!(matches!(
        p.apply_once(&s, &mut l, &f.job, || -> Result<()> {
            panic!("never repeat an uncertain effect")
        }),
        Err(OnceError::Uncertain)
    ));
    assert_eq!(
        fs::read(f.root.path().join("effect-count")).unwrap(),
        b"one"
    );
}
#[test]
fn exit_worker() {
    let Some(root) = std::env::var_os("GIT_ASSET_EFFECT_FIXTURE") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let (minimum, job): (Checkpoint, JournalRef) =
        serde_json::from_slice(&fs::read(root.join("saved.json")).unwrap()).unwrap();
    let s = LocalStore::open(&root.join("witness")).unwrap();
    let mut l = CapacityLedger::open(&root.join("ledger"), &minimum).unwrap();
    let p = packet(&s, b"original birth and native operation");
    let _ = p.apply_once(&s, &mut l, &job, || -> Result<()> {
        fs::write(root.join("effect-count"), b"one").unwrap();
        std::process::exit(77);
    });
    panic!("worker must exit inside the effect");
}
