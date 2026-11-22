//! Same worker over fake providers by default and explicitly admitted real annex.
use super::*;
use crate::github::native::{Execution, Policy as WorkerPolicy};
use crate::{github, lifecycle, source};
pub(super) fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let executable = std::env::var_os("GIT_ASSET_TEST_GIT_EXE").unwrap_or_else(|| "git".into());
    let output = Process::new(executable)
        .current_dir(root)
        .args(["-c", "core.hooksPath=NUL"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
pub(super) mod batch;
mod history;
fn matrix(native: bool, demanded: bool, snapshot_mode: bool, action_mode: bool, batch_mode: u8) {
    let bytes = vec![45; 256 * 1024];
    let tools = native.then(|| {
        (
            std::env::var_os("GIT_ASSET_TEST_GIT_EXE").unwrap().into(),
            std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE").unwrap().into(),
            std::env::split_paths(&std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").unwrap())
                .collect(),
        )
    });
    let mut source = Fixture::build_setup(
        &bytes,
        tools.clone(),
        false,
        None,
        snapshot_mode || action_mode || batch_mode >= 4,
    );
    let mut target = Fixture::build_setup(&bytes, tools, true, None, true);
    if batch_mode >= 4 {
        super::control::enable(&mut source, native);
        super::control::enable(&mut target, native);
    }
    if batch_mode < 4 {
        retirement::publish(&mut source);
    }
    if batch_mode == 3 || batch_mode >= 4 {
        retirement::publish(&mut target);
    }
    let profile = crate::annex::replication::Profile {
        source: source.profile.clone(),
        destination: target.profile.clone(),
        source_domain: k("retained-domain"),
        destination_domain: k("working-domain"),
    };
    let plan = if snapshot_mode {
        let snapshot = retirement::snapshot(&mut source);
        crate::annex::replication::Plan::from_snapshot(
            &profile,
            source.plan.clone(),
            target.plan.clone(),
            snapshot,
        )
        .unwrap()
    } else {
        crate::annex::replication::Plan::new(&profile, source.plan.clone(), target.plan.clone())
            .unwrap()
    };
    let target_plan = target.plan.clone();
    let mut pending = (batch_mode == 4).then(|| batch::pending_namespace(&target));
    let policy = WorkerPolicy {
        copy: profile,
        source_control: source.control_profile.clone(),
        destination_control: target.control_profile.clone(),
        namespace: target.namespaces[0].identity().clone(),
        required_place: k("pool"),
        allow_selection: true,
        allow_copy: true,
        producers: (batch_mode != 0).then(|| github::native::ProducerRoots {
            source: batch::index(
                &mut source,
                plan.source().clone(),
                plan.source_snapshot().cloned(),
            ),
            destination: batch::index(&mut target, target_plan.clone(), None),
        }),
        actions: if batch_mode >= 4 {
            Some(github::native::Actions {
                backup: true,
                history: None,
                pending: pending.as_ref().map(|ns| batch::pending_scope(&target, ns)),
                handoff: true,
                eviction: None,
            })
        } else {
            action_mode.then_some(github::native::Actions {
                handoff: true,
                eviction: None,
                backup: false,
                history: None,
                pending: None,
            })
        },
    };
    let policy: WorkerPolicy =
        serde_json::from_slice(&serde_json::to_vec(&policy).unwrap()).unwrap();
    if policy.producers.is_none() {
        // Freeze the pre-index policy wire/domain independently of Policy's serializer.
        let mut old = serde_json::json!({"copy":policy.copy,"source_control":policy.source_control,
            "destination_control":policy.destination_control,"namespace":policy.namespace,
            "required_place":policy.required_place,"allow_selection":policy.allow_selection,
            "allow_copy":policy.allow_copy});
        if let Some(actions) = &policy.actions {
            old["actions"] = serde_json::to_value(actions).unwrap();
        }
        let expected = Digest::blob(
            &identity::canonical_json(&serde_json::json!({
            "schema":"git-asset.trusted-native-copy-policy.experimental-1","policy":old}))
            .unwrap(),
        );
        assert_eq!(policy.id().unwrap(), expected);
    }
    let project = target.root.join("selected-project");
    fs::create_dir_all(project.join("assets")).unwrap();
    git(&project, &["init", "-b", "main"]);
    git(&project, &["config", "user.name", "Synthetic"]);
    git(
        &project,
        &["config", "user.email", "fixture@example.invalid"],
    );
    let object = source.plan.candidate().object.clone();
    let selected_object = if demanded {
        object.clone()
    } else {
        Object::from_bytes(b"different selected content")
    };
    fs::write(
        project.join("assets/clip"),
        format!(
            "/annex/objects/SHA256-s{}--{}\n",
            selected_object.bytes,
            selected_object.sha256.as_str()
        ),
    )
    .unwrap();
    git(&project, &["add", "assets"]);
    git(&project, &["commit", "-m", "selected native asset"]);
    let commit = source::GitId::parse(
        String::from_utf8(git(&project, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim(),
    )
    .unwrap();
    let executable = std::env::var_os("GIT_ASSET_TEST_GIT_EXE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::split_paths(&std::env::var_os("PATH").unwrap())
                .map(|d| d.join("git.exe"))
                .find(|p| p.is_file())
                .unwrap()
        });
    let repository =
        source::Repository::new(executable, project.clone(), source::Limits::default()).unwrap();
    let native_source = source::annex::Source {
        repository: &repository,
    };
    let enrollment = lifecycle::Enrollment {
        repository: Digest::blob(b"synthetic project"),
        authority: Digest::blob(b"worker authority"),
        namespace: policy.namespace.clone(),
        policy: Digest::blob(
            &identity::canonical_json(
                &serde_json::to_value(target.namespaces[0].policy()).unwrap(),
            )
            .unwrap(),
        ),
        source: k("creative"),
        owner: k("owner"),
        class: Kind::Branch,
        ref_name: "refs/heads/main".into(),
        scope: vec![LogicalPath::parse("assets").unwrap()],
        budget: lifecycle::Budget {
            entries: 128,
            nodes: 1024,
            node_bytes: 4 * 1024 * 1024,
            millis: 60000,
        },
    };
    let mut signer = github::fixture::Fixture::new();
    signer.commit = commit.clone();
    signer.claims["sha"] = serde_json::json!(commit.as_str());
    signer.trust.selection_enrollment = enrollment.id().unwrap();
    signer.trust.placement_enrollment = policy.id().unwrap();
    let request = signer.verify(&signer.claims, 1000).unwrap();
    let mut namespace = Namespace::open(
        &target.root.join("namespace-0"),
        &target.namespaces[0].checkpoint(),
    )
    .unwrap();
    let mut metadata =
        MetadataStore::open(&target.root.join("metadata"), target.metadata.checkpoint()).unwrap();
    let limits = Fixture::limits();
    let mut worker = Execution {
        trust: &signer.trust,
        request: &request,
        policy: &policy,
        enrollment: &enrollment,
        repository: &native_source,
        namespace: &mut namespace,
        metadata: &mut metadata,
        limits: &limits,
    };
    if batch_mode != 0 {
        fs::remove_file(
            target
                .root
                .join("input/objects")
                .join(object.sha256.as_str()),
        )
        .unwrap();
        if batch_mode == 6 {
            history::exercise(&mut worker, &mut source, &mut target, &project, &signer);
            return;
        }
        if batch_mode == 5 {
            batch::backup::uncertain(&mut worker, &mut source, &mut target);
            return;
        }
        if batch_mode >= 4 {
            batch::backup::exercise(
                &mut worker,
                &mut source,
                &mut target,
                &project,
                &signer,
                pending.as_mut().unwrap(),
            );
            return;
        }
        if batch_mode == 3 {
            batch::existing(&mut worker, &mut source, &mut target, &plan);
            return;
        }
        if batch_mode == 2 {
            batch::uncertain(&mut worker, &mut source, &mut target, &plan);
            return;
        }
        batch::exercise(
            &mut worker,
            &mut source,
            &mut target,
            &plan,
            &project,
            &signer,
        );
        return;
    }
    let before = worker.namespace.checkpoint();
    // Policy drift cannot use an otherwise valid signed event.
    let mut wrong = policy.clone();
    wrong.allow_copy = false;
    worker.policy = &wrong;
    assert!(matches!(
        worker.copy(
            &plan,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Admission(github::Error::Trust))
    ));
    assert_eq!(worker.namespace.checkpoint(), before);
    worker.policy = &policy;
    let mut foreign_profile = policy.copy.clone();
    foreign_profile.source_domain = k("foreign-domain");
    let foreign_plan = crate::annex::replication::Plan::new(
        &foreign_profile,
        source.plan.clone(),
        target.plan.clone(),
    )
    .unwrap();
    assert!(matches!(
        worker.copy(
            &foreign_plan,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Native(Error::Conflict))
    ));
    assert_eq!(worker.namespace.checkpoint(), before);
    let mut disabled_trust = signer.trust.clone();
    disabled_trust.placement_enrollment = wrong.id().unwrap();
    let disabled_request = github::verify(
        &disabled_trust,
        &signer.keys,
        &signer.token(&signer.claims),
        &commit,
        1000,
    )
    .unwrap();
    worker.trust = &disabled_trust;
    worker.request = &disabled_request;
    worker.policy = &wrong;
    assert!(matches!(
        worker.copy(
            &plan,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Admission(github::Error::Disabled))
    ));
    assert_eq!(worker.namespace.checkpoint(), before);
    worker.trust = &signer.trust;
    worker.request = &request;
    worker.policy = &policy;
    fs::remove_file(
        target
            .root
            .join("input/objects")
            .join(object.sha256.as_str()),
    )
    .unwrap();
    if !demanded {
        assert!(matches!(
            worker.copy(
                &plan,
                &mut source.resources(),
                &mut target.resources(),
                || Ok(1000)
            ),
            Err(github::native::Error::Admission(github::Error::Denied))
        ));
        assert!(
            target.namespaces[0]
                .source(&k("asset"))
                .unwrap()
                .unwrap()
                .current
                .is_none()
        );
        assert!(
            target
                .ledger
                .allocation_observation(
                    &key(&plan.destination().scope().unwrap(), "payload").unwrap()
                )
                .unwrap()
                .is_none()
        );
        return;
    }
    let mut copied = worker
        .copy(
            &plan,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000),
        )
        .unwrap();
    let mut actual = vec![];
    copied.retained().read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
    assert_eq!(copied.report().copy.plan, plan.id().unwrap());
    assert!(copied.report().selection.desired_current);
    assert_eq!(copied.report().selection.observed_ref_current, Some(true));
    assert_eq!(copied.report().storage_converged, None);
    assert!(copied.report().post_copy_hold.is_none());
    let selected = worker.namespace.source(&k("creative")).unwrap().unwrap();
    assert!(matches!(
        worker.namespace.apply(
            k("blocked-advance"),
            Command::Withdraw {
                source: k("creative"),
                owner: k("owner"),
                expected_revision: selected.revision
            },
            worker.metadata,
            &limits
        ),
        Err(admission::Error::Journal(
            git_asset_store::source_claims::Error::Busy
        ))
    ));
    drop(copied);
    if action_mode {
        let snapshot = retirement::snapshot(&mut source);
        let handoff = crate::annex::publication::handoff::Plan::new(
            plan.destination(),
            plan.source(),
            snapshot,
            &mut target.namespaces[0],
        )
        .unwrap();
        let checkpoint = target.namespaces[0].checkpoint();
        let mut copy_only = policy.clone();
        copy_only.actions = None;
        let mut copy_trust = signer.trust.clone();
        copy_trust.placement_enrollment = copy_only.id().unwrap();
        let copy_request = github::verify(
            &copy_trust,
            &signer.keys,
            &signer.token(&signer.claims),
            &commit,
            1000,
        )
        .unwrap();
        worker.policy = &copy_only;
        worker.trust = &copy_trust;
        worker.request = &copy_request;
        assert!(matches!(
            worker.handoff(
                &handoff,
                plan.destination(),
                plan.source(),
                &mut target.resources(),
                &mut source.resources(),
                || Ok(1000)
            ),
            Err(github::native::Error::Admission(github::Error::Disabled))
        ));
        assert_eq!(target.namespaces[0].checkpoint(), checkpoint);
        worker.policy = &policy;
        worker.trust = &signer.trust;
        worker.request = &request;
        let mut calls = 0;
        assert!(matches!(
            worker.handoff(
                &handoff,
                plan.destination(),
                plan.source(),
                &mut target.resources(),
                &mut source.resources(),
                || {
                    calls += 1;
                    Ok(if calls == 3 { 1300 } else { 1000 })
                }
            ),
            Err(github::native::Error::Admission(github::Error::Expired))
        ));
        assert_eq!(calls, 3);
        assert_eq!(target.namespaces[0].checkpoint(), checkpoint);
        let retry_request = signer.verify(&signer.claims, 1000).unwrap();
        worker.request = &retry_request;
        let mut calls = 0;
        assert!(matches!(
            worker.handoff(
                &handoff,
                plan.destination(),
                plan.source(),
                &mut target.resources(),
                &mut source.resources(),
                || {
                    calls += 1;
                    if calls == 4 {
                        git(
                            &project,
                            &[
                                "commit",
                                "--allow-empty",
                                "-m",
                                "head advances between handoff commands",
                            ],
                        );
                    }
                    Ok(1000)
                }
            ),
            Err(github::native::Error::Admission(github::Error::Stale))
        ));
        assert_eq!(calls, 4);
        let partial = handoff.status(&mut target.namespaces[0]).unwrap();
        assert!(partial.withdraw.is_some() && partial.retire.is_none());
        let next = source::GitId::parse(
            String::from_utf8(git(&project, &["rev-parse", "HEAD"]))
                .unwrap()
                .trim(),
        )
        .unwrap();
        let mut claims = signer.claims.clone();
        claims["sha"] = serde_json::json!(next.as_str());
        let next_request = github::verify(
            &signer.trust,
            &signer.keys,
            &signer.token(&claims),
            &next,
            1000,
        )
        .unwrap();
        worker.request = &next_request;
        let done = worker
            .handoff(
                &handoff,
                plan.destination(),
                plan.source(),
                &mut target.resources(),
                &mut source.resources(),
                || Ok(1000),
            )
            .unwrap();
        assert!(done.result.status.retire.is_some());
        assert!(done.post_effect_hold.is_none());
        assert!(done.selection.desired_current);
        assert_eq!(
            crate::annex::retirement::Assessor::new(target.profile.pool.clone())
                .unwrap()
                .assess(&object, &target.metadata, &limits)
                .err()
                .unwrap()
                .code(),
            "protected"
        );
        return;
    }
    let checkpoint = worker.namespace.checkpoint();
    let mut calls = 0;
    let duplicate = worker
        .copy(
            &plan,
            &mut source.resources(),
            &mut target.resources(),
            || {
                calls += 1;
                if calls == 3 {
                    git(
                        &project,
                        &["commit", "--allow-empty", "-m", "head advances during copy"],
                    );
                }
                Ok(1000)
            },
        )
        .unwrap();
    assert_eq!(checkpoint, worker.namespace.checkpoint());
    assert_eq!(duplicate.report().copy.plan, plan.id().unwrap());
    assert_eq!(
        duplicate.report().selection.observed_ref_current,
        Some(false)
    );
    assert!(matches!(
        duplicate.report().post_copy_hold,
        Some(github::Error::Stale)
    ));
    drop(duplicate);
    assert!(matches!(
        worker.copy(
            &plan,
            &mut source.resources(),
            &mut target.resources(),
            || Ok(1000)
        ),
        Err(github::native::Error::Selection(lifecycle::Error::Stale))
    ));
    assert_eq!(checkpoint, worker.namespace.checkpoint());
    // Historical destination reopens with original handles despite a newer ref.
    let mut copier = crate::annex::replication::Copier::new(policy.copy.clone()).unwrap();
    let mut retained = copier.reopen(&plan, &mut target.resources()).unwrap();
    let mut actual = vec![];
    retained.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
}
#[test]
fn authenticated_native_copy_uses_exact_selected_demand_and_original_retry() {
    matrix(false, true, false, false, 0);
}
#[test]
#[ignore = "explicit native tools; synthetic pools only"]
fn actual_authenticated_native_copy_reopens_original_after_head_advance() {
    matrix(true, true, false, false, 0);
}

#[test]
fn authenticated_native_copy_refuses_undemanded_candidate_before_publication() {
    matrix(false, false, false, false, 0);
}

#[test]
fn authenticated_native_snapshot_copy_keeps_exact_git_admission() {
    matrix(false, true, true, false, 0);
}
#[test]
#[ignore = "explicit native tools; fresh synthetic pools only"]
fn actual_authenticated_native_snapshot_copy_keeps_exact_git_admission() {
    matrix(true, true, true, false, 0);
}

#[test]
fn authenticated_handoff_revalidates_after_custody_and_between_commands() {
    matrix(false, true, true, true, 0);
}
#[test]
#[ignore = "explicit native tools; fresh synthetic pools only"]
fn actual_authenticated_handoff_revalidates_after_custody_and_between_commands() {
    matrix(true, true, true, true, 0);
}

#[test]
fn authenticated_history_pass_preserves_exact_roots_and_continues_after_release() {
    matrix(false, true, false, false, 6);
}
