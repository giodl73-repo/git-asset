use std::process::Command;
#[test]
fn discovery_names_the_native_backend_without_starting_providers() {
    let out = Command::new(env!("CARGO_BIN_EXE_git-asset"))
        .arg("describe")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["product"], "git-asset");
    assert_eq!(value["backend"], "git-annex");
    assert_eq!(value["automatic_cache_operations"], false);
    assert_eq!(value["native_publication_qualified"], false);
    let help = Command::new(env!("CARGO_BIN_EXE_git-asset"))
        .arg("help")
        .output()
        .unwrap();
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("native Rust adapters"));
    assert!(text.contains("Native-publication CLI execution"));
    for bad in ["../query", "query;echo", "not-a-command"] {
        let out = Command::new(env!("CARGO_BIN_EXE_git-asset"))
            .arg(bad)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
    }
}
#[cfg(feature = "schema")]
#[test]
fn front_door_dispatches_only_its_named_sibling_service() {
    let direct = Command::new(env!("CARGO_BIN_EXE_git-asset-query"))
        .arg("describe")
        .output()
        .unwrap();
    let through = Command::new(env!("CARGO_BIN_EXE_git-asset"))
        .args(["query", "describe"])
        .output()
        .unwrap();
    assert!(direct.status.success());
    assert!(through.status.success());
    assert_eq!(direct.stdout, through.stdout);
    assert_eq!(direct.stderr, through.stderr);
}

#[test]
fn native_storage_discovery_and_arity_never_initialize_a_provider() {
    let root = tempfile::tempdir().unwrap();
    for op in ["describe", "help"] {
        let out = Command::new(env!("CARGO_BIN_EXE_git-asset"))
            .args(["storage", "native", op])
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(out.status.success());
        assert!(out.stderr.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        if op == "describe" {
            assert_eq!(value["available"], cfg!(feature = "annex-publish"));
            assert_eq!(value["initializes_providers"], false);
            assert_eq!(value["production_qualified"], false);
        }
    }
    let out = Command::new(env!("CARGO_BIN_EXE_git-asset"))
        .args(["storage", "native", "apply", "too", "many", "arguments"])
        .current_dir(root.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        value["code"],
        if cfg!(feature = "annex-publish") {
            "usage"
        } else {
            "unsupported"
        }
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}
