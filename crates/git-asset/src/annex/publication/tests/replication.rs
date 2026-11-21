use super::*;
use crate::annex::replication as copy;
fn pair(root: &Path) -> (copy::Profile, Plan, Plan) {
    let a = profile(&root.join("source"));
    let mut b = profile(&root.join("destination"));
    b.pool.pool.uuid = "00000000-0000-0000-0000-000000000002".into();
    let mut nodes = MemoryNodes::default();
    let r = request(&mut nodes);
    let source = plan(&mut nodes, &a, r.clone());
    let target = plan(&mut nodes, &b, r);
    (
        copy::Profile {
            source: a,
            destination: b,
            source_domain: k("domain"),
            destination_domain: k("domain"),
        },
        source,
        target,
    )
}
#[test]
fn copy_plan_pins_locations_context_and_preserves_legacy_publication_encoding() {
    let temp = tempfile::tempdir().unwrap();
    let (profile, source, target) = pair(temp.path());
    let original = target.id().unwrap();
    let value = serde_json::to_value(&target).unwrap();
    assert!(value.get("context").is_none());
    let legacy: Plan = serde_json::from_value(value).unwrap();
    assert_eq!(legacy.id().unwrap(), original);
    let p = copy::Plan::new(&profile, source.clone(), target.clone()).unwrap();
    assert_ne!(p.destination().id().unwrap(), original);
    assert_eq!(p.source().candidate(), p.destination().candidate());
    assert_eq!(
        p,
        serde_json::from_slice::<copy::Plan>(&serde_json::to_vec(&p).unwrap()).unwrap()
    );
    let mut other = profile.clone();
    other.source.pool.pool.repository = temp.path().join("replaced-source");
    let changed = copy::Plan::new(&other, source.clone(), target.clone()).unwrap();
    assert_ne!(changed.id().unwrap(), p.id().unwrap());
    assert_ne!(
        changed.destination().id().unwrap(),
        p.destination().id().unwrap()
    );
    let mut other = profile.clone();
    other.destination_domain = k("independently-admitted-domain");
    let changed = copy::Plan::new(&other, source, target).unwrap();
    assert_ne!(changed.id().unwrap(), p.id().unwrap());
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}
#[test]
fn copy_alias_conflicting_candidate_and_source_peak_refuse_purely() {
    let temp = tempfile::tempdir().unwrap();
    let (mut p, source, _target) = pair(temp.path());
    assert_eq!(
        copy::Plan::new(&p, source.clone(), source.clone()).unwrap_err(),
        Error::Stale
    );
    p.destination = p.source.clone();
    assert_eq!(
        copy::Plan::new(&p, source.clone(), source.clone()).unwrap_err(),
        Error::Invalid
    );
    let (mut p, source, target) = pair(temp.path());
    let mut wrong = target.clone();
    wrong.request.candidate.logical_id = "different asset".into();
    assert_eq!(
        copy::Plan::new(&p, source.clone(), wrong).unwrap_err(),
        Error::Conflict
    );
    p.source.pool.pool.limits.live_readers = 1;
    assert_eq!(
        copy::Plan::new(&p, source, target).unwrap_err(),
        Error::Budget
    );
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}
