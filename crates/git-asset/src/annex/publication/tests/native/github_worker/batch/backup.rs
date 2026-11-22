use super::*;
pub(in super::super) fn exercise(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    m: &mut Fixture,
    f: &mut Fixture,
    project: &Path,
    signer: &github::fixture::Fixture,
) {
    let object = f.plan.candidate().object.clone();
    fs::remove_file(m.root.join("input/objects").join(object.sha256.as_str())).unwrap();
    let reverse = replication::Profile {
        source: worker.policy.copy.destination.clone(),
        destination: worker.policy.copy.source.clone(),
        source_domain: worker.policy.copy.destination_domain.clone(),
        destination_domain: worker.policy.copy.source_domain.clone(),
    };
    let plan = replication::Plan::new(&reverse, f.plan.clone(), m.plan.clone()).unwrap();
    let budget = github::placement::Budget {
        pages: 1,
        objects: 1,
        stream_bytes: 1024 * 1024,
        millis: 60000,
    };
    let policy = worker.policy.clone();
    let trust = worker.trust.clone();
    let mut dry_policy = policy.clone();
    dry_policy.allow_copy = false;
    let mut dry_trust = trust.clone();
    dry_trust.placement_enrollment = dry_policy.id().unwrap();
    let dry_request = github::verify(
        &dry_trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    let dry = configured(worker, &dry_policy, &dry_trust, &dry_request)
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(dry.outcomes[0].state, "planned");
    assert_eq!(dry.outcomes[0].backup, Some(plan.id().unwrap()));
    assert!(dry.outcomes[0].publication.is_none());
    assert!(dry.outcomes[0].retained_publication.is_none());
    assert!(
        !plan
            .destination()
            .progress(&m.witness, &mut m.ledger)
            .unwrap()
            .intent
    );
    assert!(
        m.namespaces[0]
            .source(&k("asset"))
            .unwrap()
            .unwrap()
            .current
            .is_none()
    );
    let mut denied_policy = policy.clone();
    denied_policy.actions.as_mut().unwrap().backup = false;
    let wire = serde_json::to_value(&denied_policy).unwrap();
    assert!(wire["actions"].get("backup").is_none());
    let mut denied_trust = trust.clone();
    denied_trust.placement_enrollment = denied_policy.id().unwrap();
    let denied_request = github::verify(
        &denied_trust,
        &signer.keys,
        &signer.token(&signer.claims),
        worker.request.commit(),
        1000,
    )
    .unwrap();
    let checkpoint = worker.namespace.checkpoint();
    assert!(matches!(
        configured(worker, &denied_policy, &denied_trust, &denied_request).backup(
            &plan,
            &mut f.resources(),
            &mut m.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Admission(github::Error::Disabled))
    ));
    assert_eq!(checkpoint, worker.namespace.checkpoint());
    assert!(
        worker
            .copy(&plan, &mut f.resources(), &mut m.resources(), || Ok(1000))
            .is_err()
    );
    assert_eq!(checkpoint, worker.namespace.checkpoint());
    // Offline destination refuses before native intent, retaining the original F bytes.
    let config = m.profile.pool.pool.repository.join(".git/config");
    let offline = config.with_extension("offline");
    fs::rename(&config, &offline).unwrap();
    let held = worker
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(held.outcomes[0].state, "held");
    assert!(held.outcomes[0].hold.is_some());
    fs::rename(offline, config).unwrap();
    assert!(
        !plan
            .destination()
            .progress(&m.witness, &mut m.ledger)
            .unwrap()
            .intent
    );
    let mut actual = Vec::new();
    f.publisher()
        .reopen(&f.plan.clone(), &mut f.resources())
        .unwrap()
        .read_to_end(&mut actual)
        .unwrap();
    assert_eq!(Object::from_bytes(&actual), object);
    // Successful backup survives admission expiry at the post-effect boundary.
    let mut calls = 0;
    let report = worker
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || {
                calls += 1;
                Ok(if calls == 5 { 1300 } else { 1000 })
            },
        )
        .unwrap();
    assert_eq!(
        report.outcomes[0].state,
        "backed_up",
        "{}",
        serde_json::to_string(&report).unwrap()
    );
    assert_eq!(calls, 5);
    assert_eq!(report.post_effect_hold, Some(github::Error::Expired));
    assert_eq!(report.selection.observed_ref_current, Some(true));
    assert_eq!(report.outcomes[0].backup, Some(plan.id().unwrap()));
    assert_eq!(report.outcomes[0].publication, Some(f.plan.id().unwrap()));
    assert_eq!(
        report.outcomes[0].retained_publication,
        Some(plan.destination().id().unwrap())
    );
    assert_eq!(report.storage_converged, None);
    let state = plan
        .destination()
        .progress(&m.witness, &mut m.ledger)
        .unwrap();
    assert!(state.completion && state.settled);
    let renewed = signer.verify(&signer.claims, 1000).unwrap();
    let mut active = configured(worker, &policy, &trust, &renewed);
    let duplicate = active
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(
        duplicate.outcomes[0].state,
        "present",
        "{}",
        serde_json::to_string(&duplicate).unwrap()
    );
    assert_eq!(duplicate.outcomes[0].backup, Some(plan.id().unwrap()));
    assert_eq!(
        duplicate.outcomes[0].retained_publication,
        Some(plan.destination().id().unwrap())
    );
    // The original fixed roots suffice to recover the actual backup recipe.
    let snapshot = retirement::snapshot(m);
    let handoff = crate::annex::publication::handoff::Plan::new(
        &f.plan,
        plan.destination(),
        snapshot,
        &mut f.namespaces[0],
    )
    .unwrap();
    let f_plan = f.plan.clone();
    let handed = active
        .handoff(
            &handoff,
            &f_plan,
            plan.destination(),
            &mut f.resources(),
            &mut m.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert!(handed.result.status.retire.is_some());
    assert!(handed.post_effect_hold.is_none());
    let after = active
        .reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000),
        )
        .unwrap();
    assert_eq!(
        after.outcomes[0].state,
        "present",
        "{}",
        serde_json::to_string(&after).unwrap()
    );
    assert_eq!(after.outcomes[0].publication, Some(f_plan.id().unwrap()));
    let mut copier = replication::Copier::controlled(
        reverse,
        f.control_profile.clone(),
        m.control_profile.clone(),
    )
    .unwrap();
    let mut retained = copier.reopen(&plan, &mut m.resources()).unwrap();
    let mut bytes = Vec::new();
    retained.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, actual);
    drop(retained);
    // A stale event cannot restart the backup after Main advances.
    git(
        project,
        &[
            "commit",
            "--allow-empty",
            "-m",
            "main advances after backup",
        ],
    );
    assert!(matches!(
        active.reconcile(
            None,
            &budget,
            &mut m.resources(),
            &mut f.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Selection(lifecycle::Error::Stale))
    ));
}
#[test]
fn fixed_root_backup_permission_offline_expiry_retry_and_ingest_handoff() {
    matrix(false, true, false, false, 4);
}
pub(in super::super) fn uncertain(
    worker: &mut Execution<'_, impl lifecycle::CommittedSource>,
    m: &mut Fixture,
    f: &mut Fixture,
) {
    let initial = m.plan.progress(&m.witness, &mut m.ledger).unwrap();
    assert!(
        !initial.intent && !initial.completion,
        "fixture must start without an M publication"
    );
    let reverse = replication::Profile {
        source: worker.policy.copy.destination.clone(),
        destination: worker.policy.copy.source.clone(),
        source_domain: worker.policy.copy.destination_domain.clone(),
        destination_domain: worker.policy.copy.source_domain.clone(),
    };
    let plan = replication::Plan::new(&reverse, f.plan.clone(), m.plan.clone()).unwrap();
    let mut copier = replication::Copier::controlled(
        reverse,
        f.control_profile.clone(),
        m.control_profile.clone(),
    )
    .unwrap();
    assert!(
        copier
            .apply_hook(&plan, &mut f.resources(), &mut m.resources(), |stage| {
                if stage == Stage::Imported {
                    Err(Error::NeedsRecovery)
                } else {
                    Ok(())
                }
            })
            .is_err()
    );
    let calls = m.profile.pool.pool.repository.join("calls");
    let imports = || {
        fs::read_to_string(&calls)
            .unwrap_or_default()
            .lines()
            .filter(|s| s.contains("reinject"))
            .count()
    };
    let count = imports();
    assert!(count > 0);
    let budget = github::placement::Budget {
        pages: 1,
        objects: 1,
        stream_bytes: 1024 * 1024,
        millis: 60000,
    };
    for _ in 0..2 {
        let report = worker
            .reconcile(
                None,
                &budget,
                &mut m.resources(),
                &mut f.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert_eq!(report.outcomes[0].state, "held");
        assert_eq!(report.outcomes[0].backup, Some(plan.id().unwrap()));
        assert!(matches!(
            report.outcomes[0].hold.as_ref().unwrap().code.as_str(),
            "uncertain" | "needs_recovery"
        ));
        assert!(report.outcomes[0].publication.is_none());
    }
    assert_eq!(imports(), count);
    let original = f.plan.clone();
    f.publisher().reopen(&original, &mut f.resources()).unwrap();
}
#[test]
fn uncertain_backup_preserves_original_plan_and_never_reimports() {
    matrix(false, true, false, false, 5);
}
#[test]
#[ignore = "explicit native tools; synthetic pools only"]
fn actual_fixed_root_backup_permission_offline_expiry_retry_and_ingest_handoff() {
    matrix(true, true, false, false, 4);
}
