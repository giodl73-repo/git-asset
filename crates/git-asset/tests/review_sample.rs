#![cfg(feature = "schema")]
mod common;
use git_asset::catalogue::{Config, shared};
use git_asset_core::Object;
use serde_json::Value;
use std::{fs, path::Path, process::Command};
#[path = "common/catalogue_server.rs"]
mod catalogue_server;

fn run(command: &str, path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_git-asset-sample"))
        .arg(command)
        .arg(path)
        .output()
        .unwrap()
}

#[test]
fn front_door_creates_independent_review_projects_and_preserves_original_receipts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("review workspace");
    let out = Command::new(env!("CARGO_BIN_EXE_git-asset"))
        .args(["sample", "rehearse-review"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["viewer_started"], false);
    assert_eq!(result["complete_course"], false);
    assert_eq!(result["visible_recovery_checkpoints"], 11);
    let saved: Value =
        serde_json::from_slice(&fs::read(root.join("review.json")).unwrap()).unwrap();
    assert_eq!(saved, result);
    for (index, id, checkpoints) in [(0, "intro", 7), (1, "recovery", 11)] {
        assert!(root.join(id).join("project/.git").is_dir());
        let receipt = &result["projects"][index];
        let bytes = fs::read(root.join(receipt["path"].as_str().unwrap())).unwrap();
        let object: Object = serde_json::from_value(receipt["object"].clone()).unwrap();
        assert_eq!(Object::from_bytes(&bytes), object);
        let original: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(original["completed_checkpoints"], checkpoints);
        let enrollment: Value =
            serde_json::from_slice(&fs::read(root.join(id).join("catalogue.json")).unwrap())
                .unwrap();
        assert_eq!(enrollment["projects"][0]["id"], "sample");
    }
    let catalogue_bytes = fs::read(root.join("catalogue.json")).unwrap();
    assert_eq!(
        Object::from_bytes(&catalogue_bytes),
        serde_json::from_value::<Object>(result["catalogue"]["object"].clone()).unwrap()
    );
    let config: Value = serde_json::from_slice(&catalogue_bytes).unwrap();
    assert!(config["default_project"].is_null());
    assert_eq!(config["teaching"]["project"], "recovery");
    let intersection = shared::compare(
        Config::load(&root.join("catalogue.json")).unwrap(),
        "intro",
        "recovery",
        "",
    )
    .unwrap();
    assert_eq!(intersection["shared_objects"], 1);
    assert_eq!(intersection["shared_unique_bytes"], 25);
    let server = catalogue_server::Server::new(&root.join("catalogue.json"));
    let (code, book) = server.json("/api/lessons");
    assert_eq!(code, 200);
    assert_eq!(book["lessons"].as_array().unwrap().len(), 11);
    assert_eq!(
        book["lessons"][10]["restoration"]["phase"],
        "working_restored"
    );
    drop(server);
    let before = fs::read(root.join("review.json")).unwrap();
    assert!(!run("rehearse-review", &root).status.success());
    assert_eq!(fs::read(root.join("review.json")).unwrap(), before);
}

#[test]
fn discovery_and_invalid_review_destinations_have_no_fixture_effects() {
    let out = Command::new(env!("CARGO_BIN_EXE_git-asset-sample"))
        .arg("describe")
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        v["operations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|op| { op.as_str() == Some("rehearse-review NEW_ABSOLUTE_ROOT") })
    );
    let temp = tempfile::tempdir().unwrap();
    let existing = temp.path().join("existing");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("canary"), b"preserve").unwrap();
    assert!(!run("rehearse-review", &existing).status.success());
    assert_eq!(fs::read(existing.join("canary")).unwrap(), b"preserve");
    assert_eq!(fs::read_dir(&existing).unwrap().count(), 1);
    let missing = temp.path().join("missing-parent/review");
    assert!(!run("rehearse-review", &missing).status.success());
    assert!(!missing.parent().unwrap().exists());
    let relative = Path::new("relative review");
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_git-asset-sample"))
            .arg("rehearse-review")
            .arg(relative)
            .current_dir(temp.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!temp.path().join(relative).exists());
    let checkout = temp.path().join("checkout");
    fs::create_dir(&checkout).unwrap();
    common::git(&checkout, &["init", "--quiet"]);
    let nested = checkout.join("review");
    assert!(!run("rehearse-review", &nested).status.success());
    assert!(!nested.exists());
}
