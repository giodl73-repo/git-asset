//! Restore one exact cohort member after the original ingest claim was released.
use super::*;
use crate::annex::replication;
fn matrix(native: bool) {
    let (old, mut keeper, snapshot, _) = super::handoff::cohort_setup(native);
    retirement::settle(&mut keeper);
    let original = keeper.plan.clone();
    assert!(matches!(
        keeper
            .publisher()
            .reopen(&original, &mut keeper.resources()),
        Err(Error::ProtectionLost)
    ));
    // The previous cache/input and unrelated cohort payloads are unavailable.
    let config = old.profile.pool.pool.repository.join(".git/config");
    fs::rename(&config, config.with_extension("offline")).unwrap();
    let tools = native.then(|| {
        (
            keeper.profile.pool.pool.git.clone(),
            keeper.profile.pool.pool.annex.clone(),
            keeper.profile.pool.pool.tools.clone(),
        )
    });
    let mut target = Fixture::build_setup(&vec![83; 256 * 1024], tools, false, None, false);
    let profile = replication::Profile {
        source: keeper.profile.clone(),
        destination: target.profile.clone(),
        source_domain: k("retained"),
        destination_domain: k("new-working"),
    };
    let plain = replication::Plan::new(&profile, original.clone(), target.plan.clone()).unwrap();
    // Compatibility: encode the original five-field plan independently.
    let old_value = serde_json::to_value(&plain).unwrap();
    let legacy = serde_json::json!({"source":plain.source(),"destination":plain.destination(),"source_domain":profile.source_domain,"destination_domain":profile.destination_domain,"locations":old_value["locations"]});
    assert_eq!(
        identity::canonical_json(&old_value).unwrap(),
        identity::canonical_json(&legacy).unwrap()
    );
    assert_eq!(
        plain.id().unwrap(),
        Digest::blob(&identity::canonical_json(&legacy).unwrap())
    );
    let plan = replication::Plan::from_snapshot(
        &profile,
        original.clone(),
        target.plan.clone(),
        snapshot.clone(),
    )
    .unwrap();
    let plan: replication::Plan =
        serde_json::from_slice(&serde_json::to_vec(&plan).unwrap()).unwrap();
    assert_ne!(plain.id().unwrap(), plan.id().unwrap());
    assert_eq!(plan.source_snapshot(), Some(&snapshot));
    fs::remove_file(
        target
            .root
            .join("input/objects")
            .join(original.candidate().object.sha256.as_str()),
    )
    .unwrap();
    let before = target.namespaces[0].checkpoint();
    let mut wrong = snapshot.clone();
    wrong.path = LogicalPath::parse("cohort/unrelated").unwrap();
    let wrong =
        replication::Plan::from_snapshot(&profile, original.clone(), target.plan.clone(), wrong)
            .unwrap();
    assert!(matches!(
        replication::Copier::new(profile.clone()).unwrap().apply(
            &wrong,
            &mut keeper.resources(),
            &mut target.resources()
        ),
        Err(Error::Conflict)
    ));
    assert_eq!(target.namespaces[0].checkpoint(), before);
    // Both metadata stages separately fit this read count; their sum does not.
    let (_, metrics) = Tree::from_root(original.selection.selection.root.clone())
        .entries(&mut keeper.metadata, TreeLimits::default())
        .unwrap();
    let mut resources = keeper.resources();
    resources.limits.reads = metrics.reads + 2;
    assert!(matches!(
        Publisher::new(profile.source.clone())
            .unwrap()
            .open_retained(&original, &snapshot, &mut resources),
        Err(Error::Budget)
    ));
    let mut copier = replication::Copier::new(profile.clone()).unwrap();
    let root = keeper.root.join("namespace-0");
    let mut peer = Namespace::open(&root, &keeper.namespaces[0].checkpoint()).unwrap();
    let peer_metadata =
        MetadataReader::open(&keeper.root.join("metadata"), keeper.metadata.checkpoint()).unwrap();
    let revision = peer.source(&k("cohort")).unwrap().unwrap().revision;
    let calls = keeper.profile.pool.pool.repository.join("calls");
    let lookups = || {
        fs::read_to_string(&calls)
            .unwrap()
            .lines()
            .filter(|line| line.contains("contentlocation"))
            .count()
    };
    let before_lookups = if native { 0 } else { lookups() };
    let mut copied = copier
        .apply_hook(
            &plan,
            &mut keeper.resources(),
            &mut target.resources(),
            |stage| {
                if stage == Stage::Imported {
                    assert!(matches!(
                        peer.apply(
                            k("blocked-cohort-withdraw"),
                            Command::Withdraw {
                                source: k("cohort"),
                                owner: k("owner"),
                                expected_revision: revision
                            },
                            &peer_metadata,
                            &Fixture::limits()
                        ),
                        Err(admission::Error::Journal(
                            git_asset_store::source_claims::Error::Busy
                        ))
                    ));
                }
                Ok(())
            },
        )
        .unwrap();
    if !native {
        assert_eq!(lookups() - before_lookups, 1);
    }
    let mut bytes = vec![];
    copied.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, vec![83; 256 * 1024]);
    assert!(copied.receipt().source_verified);
    assert!(copied.source_recovery().is_some());
    assert!(copied.source_snapshot_minimum().is_some());
    drop(copied);
    // Source-free exact reopen remains available after source config goes offline.
    let config = keeper.profile.pool.pool.repository.join("config");
    fs::rename(&config, config.with_extension("offline")).unwrap();
    let mut restored = copier.reopen(&plan, &mut target.resources()).unwrap();
    assert!(!restored.receipt().source_verified);
    assert!(restored.source_snapshot_minimum().is_none());
    let mut bytes = vec![];
    restored.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, vec![83; 256 * 1024]);
}
#[test]
fn snapshot_copy_restores_exact_member_without_original_ingest_or_old_cache() {
    matrix(false);
}
#[test]
#[ignore = "explicit native tools; fresh synthetic pools only"]
fn actual_native_snapshot_copy_restores_from_bare_cohort() {
    matrix(true);
}
