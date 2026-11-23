//! Child-only abrupt exits. This module never exists in a production build.
use super::*;
use git_asset_store::{local::LocalStore, metadata::MetadataReader};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::io::Read;
fn field<T: DeserializeOwned>(v: &Value, key: &str) -> T {
    serde_json::from_value(v.get(key).unwrap().clone()).unwrap()
}
fn read(path: &std::path::Path) -> Value {
    let mut raw = Vec::new();
    fs::File::open(path)
        .unwrap()
        .take(131073)
        .read_to_end(&mut raw)
        .unwrap();
    assert!(raw.len() <= 131072);
    git_asset_core::identity::parse_json(&raw).unwrap()
}
fn metadata(v: &Value) -> MetadataReader {
    MetadataReader::open(&field::<PathBuf>(v, "root"), &field(v, "minimum")).unwrap()
}
#[test]
fn eviction_exit_worker() {
    let Some(path) = std::env::var_os("GIT_ASSET_TEST_EVICTION_PROFILE") else {
        return;
    };
    let profile = read(std::path::Path::new(&path));
    assert_eq!(
        profile["schema"],
        "git-asset.native-storage-profile.experimental-1"
    );
    assert_eq!(profile["allow_apply"], true);
    let saved = read(std::path::Path::new(
        &std::env::var_os("GIT_ASSET_TEST_EVICTION_PLAN").unwrap(),
    ));
    let plan =
        NativePlan::from_json(&git_asset_core::identity::canonical_json(&saved["plan"]).unwrap())
            .unwrap();
    assert_eq!(
        saved["plan_id"],
        serde_json::to_value(plan.id().unwrap()).unwrap()
    );
    assert_eq!(plan.object(), &Object::from_bytes(&vec![91; 256 * 1024]));
    let owner: OwnerProfile = field(&profile, "owner");
    let source: crate::annex::publication::Profile = field(&profile["source"], "profile");
    let keeper: crate::annex::publication::Profile = field(&profile["keeper"], "profile");
    let source_plan: Plan = field(&profile["source"], "plan");
    let keeper_plan: Plan = field(&profile["keeper"], "plan");
    let sw = LocalStore::open(&field::<PathBuf>(&profile["source"], "witness")).unwrap();
    let kw = LocalStore::open(&field::<PathBuf>(&profile["keeper"], "witness")).unwrap();
    let mut sl = CapacityLedger::open(
        &field::<PathBuf>(&profile["source"], "ledger"),
        &field(&profile["source"], "minimum"),
    )
    .unwrap();
    let mut kl = CapacityLedger::open(
        &field::<PathBuf>(&profile["keeper"], "ledger"),
        &field(&profile["keeper"], "minimum"),
    )
    .unwrap();
    let sn = metadata(&profile["source_metadata"]);
    let mut kn = metadata(&profile["keeper_metadata"]);
    let limits: git_asset_store::protection::Limits = field(&profile, "limits");
    let cut = match std::env::var("GIT_ASSET_TEST_EVICTION_CUT")
        .unwrap()
        .as_str()
    {
        "before" => Cut::ExitBeforeEffect,
        "removed" => Cut::ExitAfterRemoval,
        "completed" => Cut::ExitAfterCompletion,
        _ => panic!("invalid child exit boundary"),
    };
    let result = Controller::new(&owner).apply_hook(
        &plan,
        Sources {
            source: Producer {
                profile: &source,
                plan: &source_plan,
                witness: &sw,
                ledger: &mut sl,
            },
            keeper: Producer {
                profile: &keeper,
                plan: &keeper_plan,
                witness: &kw,
                ledger: &mut kl,
            },
        },
        Nodes {
            source: &sn,
            keeper: &mut kn,
            limits: &limits,
            tree_limits: TreeLimits::default(),
        },
        cut,
    );
    panic!("expected abrupt exit, got {result:?}");
}
