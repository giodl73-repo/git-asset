//! Copy must consume the acknowledged source, not another fixture input.
use super::*;
use crate::annex::replication;
pub(super) fn copy(source: &mut Fixture, keeper: &mut Fixture, owner: &OwnerProfile, bytes: &[u8]) {
    source.refresh();
    keeper.refresh();
    let profile = replication::Profile {
        source: source.profile.clone(),
        destination: keeper.profile.clone(),
        source_domain: owner.source_domain.clone(),
        destination_domain: owner.keeper_domain.clone(),
    };
    let plan = replication::Plan::new(&profile, source.plan.clone(), keeper.plan.clone()).unwrap();
    let unavailable = keeper
        .root
        .join("input/objects")
        .join(keeper.plan.candidate().object.sha256.as_str());
    fs::remove_file(&unavailable).unwrap();
    let mut copier = replication::Copier::controlled(
        profile,
        source.control_profile.clone(),
        keeper.control_profile.clone(),
    )
    .unwrap();
    let mut retained = copier
        .apply(&plan, &mut source.resources(), &mut keeper.resources())
        .unwrap();
    let mut actual = Vec::new();
    retained.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
    assert!(retained.receipt().source_verified);
    assert_eq!(retained.receipt().declared_domains, 2);
    assert_eq!(
        retained.receipt().destination.candidate,
        source.plan.candidate().clone()
    );
    assert!(!unavailable.exists());
    drop(retained);
    drop(copier);
    keeper.plan = plan.destination().clone();
    keeper.refresh();
    // Later eviction planning must reopen THIS copy's exact physical-origin proof.
    assert!(keeper.plan.has_guarded_effect());
    let proof = keeper
        .plan
        .producer_completion(
            &keeper.profile,
            keeper.control_profile.as_ref().unwrap(),
            &keeper.witness,
            &mut keeper.ledger,
        )
        .unwrap();
    drop(proof);
    for f in [source, keeper] {
        assert!(
            f.namespaces[0]
                .source(&k("creative"))
                .unwrap()
                .unwrap()
                .current
                .is_none()
        );
    }
}
