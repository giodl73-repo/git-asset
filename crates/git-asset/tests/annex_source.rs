//! Exact metadata only; synthetic Git roots have no annex payload store.
mod common;
use common::{git, git_executable};
use git_asset::{
    lifecycle::{self, CommittedSource},
    source::{
        self, GitId, Limits, Repository, Scope,
        annex::{Source, pointer},
    },
};
use git_asset_core::{LogicalPath, Object};
use std::{fs, path::Path};
fn path(s: &str) -> LogicalPath {
    LogicalPath::parse(s).unwrap()
}
fn key(object: &Object, extension: bool) -> String {
    format!(
        "SHA256{}-s{}--{}{}",
        if extension { "E" } else { "" },
        object.bytes,
        object.sha256.as_str(),
        if extension { ".wav" } else { "" }
    )
}
fn unlocked(object: &Object) -> Vec<u8> {
    format!("/annex/objects/{}\n", key(object, true)).into_bytes()
}
fn locked(object: &Object) -> Vec<u8> {
    let k = key(object, false);
    format!("../.git/annex/objects/aBc/X1z/{k}/{k}").into_bytes()
}
fn budget() -> lifecycle::Budget {
    lifecycle::Budget {
        entries: 128,
        nodes: 1024,
        node_bytes: 4 * 1024 * 1024,
        millis: 30000,
    }
}
#[test]
fn canonical_native_keys_preserve_hash_and_size_without_path_lookup() {
    for bytes in [b"".as_slice(), b"synthetic native content".as_slice()] {
        let object = Object::from_bytes(bytes);
        for extension in [false, true] {
            for ending in ["", "\n", "\r\n"] {
                let raw = format!("/annex/objects/{}{ending}", key(&object, extension));
                for mode in ["100644", "100755"] {
                    assert_eq!(
                        pointer(&path("assets/item"), mode, raw.as_bytes()).unwrap(),
                        object
                    );
                }
            }
        }
        for mode in ["120000", "100644"] {
            assert_eq!(
                pointer(&path("assets/item"), mode, &locked(&object)).unwrap(),
                object
            );
        }
    }
}
#[test]
fn malformed_backend_path_extension_and_utf8_refuse_without_panics() {
    let object = Object::from_bytes(b"one");
    let k = key(&object, false);
    let examples = vec![
        format!("/annex/objects/{}", k.replace("SHA256-s", "SHA1-s")),
        format!("/annex/objects/{}", k.replace("-s3--", "-s03--")),
        format!(
            "/annex/objects/SHA256-s18446744073709551616--{}",
            object.sha256.as_str()
        ),
        format!("/annex/objects/{k}.wav"),
        format!("/annex/objects/{}", k.to_uppercase()),
        format!("/annex/objects/{k}\n/annex/extra\n"),
        format!("../.git/annex/objects/../abc/{k}/{k}"),
        format!(".git/annex/objects/abc/def/{k}/{k}"),
        format!("../../.git/annex/objects/abc/def/{k}/{k}"),
        format!("../.git/annex/objects/abc/def/{k}/{k}x"),
        format!(
            "/annex/objects/SHA256E-s3--{}.wav/escape",
            object.sha256.as_str()
        ),
        format!("/annex/objects/SHA256-s3--{}é", "a".repeat(63)),
        "ordinary non-annex content".into(),
    ];
    for raw in examples {
        assert!(
            pointer(&path("assets/item"), "100644", raw.as_bytes()).is_err(),
            "accepted malformed metadata"
        );
    }
    assert!(pointer(&path("assets/item"), "120000", &unlocked(&object)).is_err());
    let mut line = locked(&object);
    line.push(b'\n');
    assert!(pointer(&path("assets/item"), "120000", &line).is_err());
    assert!(pointer(&path("assets/item"), "160000", &locked(&object)).is_err());
    assert!(matches!(
        pointer(&path("assets/item"), "100644", &vec![b'x'; 1025]),
        Err(source::Error::Budget)
    ));
}
fn init() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    git(t.path(), &["init", "-b", "main"]);
    git(t.path(), &["config", "user.name", "Synthetic"]);
    git(
        t.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    t
}
fn stage(root: &Path, logical: &str, mode: &str, raw: &[u8]) {
    fs::write(root.join("metadata-input"), raw).unwrap();
    let oid = String::from_utf8(git(root, &["hash-object", "-w", "--", "metadata-input"]).stdout)
        .unwrap();
    git(
        root,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("{mode},{},{logical}", oid.trim()),
        ],
    );
}
fn commit(root: &Path) -> GitId {
    git(root, &["commit", "--quiet", "-m", "synthetic exact source"]);
    GitId::parse(
        String::from_utf8(git(root, &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim(),
    )
    .unwrap()
}
#[test]
fn exact_native_capture_ignores_head_worktree_and_excluded_payloads_in_ordinary_and_bare_git() {
    let t = init();
    let old = Object::from_bytes(b"old content");
    let new = Object::from_bytes(b"new content");
    stage(t.path(), "assets/locked", "120000", &locked(&old));
    stage(t.path(), "assets/unlocked", "100644", &unlocked(&old));
    // The excluded ordinary Git blob is deliberately far above the metadata cap.
    stage(
        t.path(),
        "excluded-large",
        "100644",
        &vec![1; 2 * 1024 * 1024],
    );
    let original = commit(t.path());
    stage(t.path(), "assets/locked", "120000", &locked(&new));
    stage(t.path(), "assets/unlocked", "100644", &unlocked(&new));
    let current = commit(t.path());
    let bare = t.path().join("copy.git");
    git(
        t.path(),
        &[
            "clone",
            "--bare",
            "--quiet",
            t.path().to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
    );
    for root in [t.path(), bare.as_path()] {
        let repository = Repository::new(git_executable(), root.into(), Limits::default()).unwrap();
        let source = Source {
            repository: &repository,
        };
        let scope = vec![path("assets")];
        let historical = source
            .flat_capture(original.clone(), &scope, &budget())
            .unwrap();
        assert_eq!(historical.bindings[&path("assets/locked")], old);
        assert_eq!(historical.bindings[&path("assets/unlocked")], old);
        assert_eq!(historical.bindings.len(), 2);
        assert_eq!(source.named_commit("refs/heads/main").unwrap(), current);
        let latest = source
            .flat_capture(current.clone(), &scope, &budget())
            .unwrap();
        assert_eq!(latest.bindings[&path("assets/locked")], new);
        assert_ne!(historical.listing, latest.listing);
        let snapshot = repository.commit(original.clone()).unwrap();
        let empty_scope = || Scope::new([path("absent")]).unwrap();
        let legacy_empty = repository.extract(&snapshot, empty_scope()).unwrap();
        let annex_empty = repository.extract_annex(&snapshot, empty_scope()).unwrap();
        assert_ne!(
            legacy_empty.selection_digest().unwrap(),
            annex_empty.selection_digest().unwrap()
        );
        assert!(
            !repository
                .extract(&snapshot, Scope::new(scope).unwrap())
                .unwrap()
                .complete()
        );
        assert!(
            source
                .flat_capture(
                    GitId::parse(&"f".repeat(40)).unwrap(),
                    &[path("assets")],
                    &budget()
                )
                .is_err()
        );
        let mut small = budget();
        small.entries = 1;
        assert!(
            source
                .flat_capture(original.clone(), &[path("assets")], &small)
                .is_err()
        );
    }
    assert!(!t.path().join(".git/annex").exists());
    assert!(!t.path().join("assets/locked").exists());
}
#[test]
fn unsupported_selected_entries_and_contradictory_sizes_hold_the_entire_source() {
    let t = init();
    let a = Object::from_bytes(b"a");
    stage(t.path(), "assets/one", "100644", &unlocked(&a));
    let mut b = a.clone();
    b.bytes += 1;
    stage(t.path(), "assets/two", "100644", &unlocked(&b));
    let c = commit(t.path());
    let repository = Repository::new(git_executable(), t.path().into(), Limits::default()).unwrap();
    let source = Source {
        repository: &repository,
    };
    assert!(matches!(
        source.flat_capture(c, &[path("assets")], &budget()),
        Err(lifecycle::Error::Pending)
    ));
    stage(t.path(), "assets/two", "120000", b"../../outside");
    let c = commit(t.path());
    assert!(matches!(
        source.flat_capture(c, &[path("assets")], &budget()),
        Err(lifecycle::Error::Pending)
    ));
    let mut empty = budget();
    empty.nodes = 0;
    assert!(matches!(
        source.flat_capture(
            GitId::parse(&"a".repeat(40)).unwrap(),
            &[path("assets")],
            &empty
        ),
        Err(lifecycle::Error::Budget)
    ));
}

#[test]
#[ignore = "explicit native tools; fresh synthetic annex repository only"]
fn actual_annex_emitted_metadata_reopens_as_exact_content_in_a_bare_mirror() {
    use std::{
        io::{Read, Seek, SeekFrom},
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let root = init();
    let annex = std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE").expect("explicit annex executable");
    let tools = std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").expect("explicit native PATH");
    fs::create_dir(root.path().join("hooks")).unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    fs::write(root.path().join("empty-global"), b"").unwrap();
    git(
        root.path(),
        &[
            "config",
            "core.hooksPath",
            root.path().join("hooks").to_str().unwrap(),
        ],
    );
    git(root.path(), &["config", "core.symlinks", "false"]);
    let native = |args: &[&str]| {
        let output = tempfile::tempfile().unwrap();
        let error = tempfile::tempfile().unwrap();
        let mut command = Command::new(&annex);
        command
            .args(args)
            .current_dir(root.path())
            .env_clear()
            .env("PATH", &tools)
            .env("TEMP", root.path())
            .env("TMP", root.path())
            .env("USERPROFILE", root.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", root.path().join("empty-global"))
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ALLOW_PROTOCOL", "file")
            .stdin(Stdio::null())
            .stdout(output.try_clone().unwrap())
            .stderr(error.try_clone().unwrap());
        if let Some(system) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", system);
        }
        let mut child = command.spawn().unwrap();
        let start = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if start.elapsed() > Duration::from_secs(60) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("owned annex fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        fn tail(mut file: fs::File) -> String {
            file.seek(SeekFrom::Start(0)).unwrap();
            let mut bytes = vec![];
            file.take(16385).read_to_end(&mut bytes).unwrap();
            assert!(bytes.len() <= 16384);
            String::from_utf8_lossy(&bytes).into_owned()
        }
        assert!(
            status.success(),
            "owned annex fixture: {} {}",
            tail(output),
            tail(error)
        );
    };
    native(&["init", "synthetic metadata source"]);
    let body = b"synthetic annex payload, not actual audio";
    fs::write(root.path().join("assets/clip.wav"), body).unwrap();
    native(&["add", "--backend=SHA256E", "--", "assets/clip.wav"]);
    let committed = common::command(
        root.path(),
        &["commit", "--quiet", "-m", "synthetic native source"],
    )
    .env("PATH", &tools)
    .output()
    .unwrap();
    assert!(
        committed.status.success(),
        "native commit: {}",
        String::from_utf8_lossy(&committed.stderr)
    );
    let original = GitId::parse(
        String::from_utf8(git(root.path(), &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim(),
    )
    .unwrap();
    let raw = git(
        root.path(),
        &["show", &format!("{}:assets/clip.wav", original.as_str())],
    )
    .stdout;
    assert!(raw.starts_with(b"/annex/objects/SHA256E-s"));
    assert_eq!(
        pointer(&path("assets/clip.wav"), "100644", &raw).unwrap(),
        Object::from_bytes(body)
    );
    let bare = root.path().join("mirror.git");
    git(
        root.path(),
        &[
            "clone",
            "--bare",
            "--quiet",
            root.path().to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
    );
    let repository = Repository::new(git_executable(), bare.clone(), Limits::default()).unwrap();
    let result = Source {
        repository: &repository,
    }
    .flat_capture(original, &[path("assets")], &budget())
    .unwrap();
    assert_eq!(
        result.bindings[&path("assets/clip.wav")],
        Object::from_bytes(body)
    );
    assert!(!bare.join("annex/objects").exists());
}
