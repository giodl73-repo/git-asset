use super::*;
use crate::annex::replication as copy;
fn pair(bytes: &[u8]) -> (Fixture, Fixture, copy::Profile, copy::Plan) {
    let mut source = Fixture::new(bytes);
    let target = Fixture::new(bytes);
    assert_ne!(source.profile.pool.pool.uuid, target.profile.pool.pool.uuid);
    source.refresh();
    let source_plan = source.plan.clone();
    let mut input = source.input();
    source
        .publisher()
        .apply(&source_plan, &mut source.resources(), &mut input)
        .unwrap();
    let profile = copy::Profile {
        source: source.profile.clone(),
        destination: target.profile.clone(),
        source_domain: k("one-synthetic-volume"),
        destination_domain: k("one-synthetic-volume"),
    };
    let plan = copy::Plan::new(&profile, source_plan, target.plan.clone()).unwrap();
    (source, target, profile, plan)
}
#[test]
fn retained_copy_preserves_source_and_reopens_without_it_after_restart() {
    let bytes = vec![73; 256 * 1024];
    let (mut source, mut target, profile, plan) = pair(&bytes);
    target.refresh();
    let source_checkpoint = source.namespaces[0].checkpoint();
    let observer = Namespace::open(&source.root.join("namespace-0"), &source_checkpoint).unwrap();
    let mut copier = copy::Copier::new(profile.clone()).unwrap();
    let mut ready = copier
        .apply_hook(
            &plan,
            &mut source.resources(),
            &mut target.resources(),
            |stage| {
                if stage == Stage::Imported {
                    assert!(matches!(
                        observer.fence(&plan.source().candidate().object),
                        Err(admission::Error::Busy)
                    ));
                }
                Ok(())
            },
        )
        .unwrap();
    let mut actual = vec![];
    ready.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
    assert!(ready.receipt().source_verified);
    assert_eq!(ready.receipt().declared_domains, 1);
    let receipt = ready.receipt().destination.clone();
    assert_eq!(receipt.candidate, source.plan.candidate().clone());
    assert_eq!(source.namespaces[0].checkpoint(), source_checkpoint);
    assert_eq!(fs::read(source.output_path()).unwrap(), bytes);
    assert!(
        source.namespaces[0]
            .source(&k("creative"))
            .unwrap()
            .unwrap()
            .current
            .is_none()
    );
    assert!(
        target.namespaces[0]
            .source(&k("creative"))
            .unwrap()
            .unwrap()
            .current
            .is_none()
    );
    drop(ready);
    drop(copier);
    let checkpoint = target.namespaces[0].checkpoint();
    fs::rename(
        &profile.source.pool.pool.repository,
        source.root.join("offline-pool"),
    )
    .unwrap();
    let mut copier = copy::Copier::new(profile).unwrap();
    let mut ready = copier.resume(&plan, &mut target.resources()).unwrap();
    assert!(!ready.receipt().source_verified);
    assert_eq!(ready.receipt().destination, receipt);
    assert_eq!(target.namespaces[0].checkpoint(), checkpoint);
    let mut actual = vec![];
    ready.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
}
#[test]
fn retained_copy_import_uncertainty_preserves_bytes_without_claiming_completion() {
    let bytes = vec![91; 128 * 1024];
    let (mut source, mut target, profile, plan) = pair(&bytes);
    target.refresh();
    let mut copier = copy::Copier::new(profile.clone()).unwrap();
    let error = copier.apply_hook(
        &plan,
        &mut source.resources(),
        &mut target.resources(),
        |stage| {
            if stage == Stage::Imported {
                Err(Error::Uncertain)
            } else {
                Ok(())
            }
        },
    );
    assert!(matches!(error, Err(Error::Uncertain)));
    let checkpoint = target.namespaces[0].checkpoint();
    fs::rename(
        &profile.source.pool.pool.repository,
        source.root.join("offline-pool"),
    )
    .unwrap();
    assert!(matches!(
        copy::Copier::new(profile)
            .unwrap()
            .resume(&plan, &mut target.resources()),
        Err(Error::Uncertain)
    ));
    let mut ready = crate::annex::managed::Reader::new(target.profile.pool.clone())
        .unwrap()
        .open(&plan.source().candidate().object)
        .unwrap();
    let mut actual = vec![];
    ready.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
    assert_eq!(target.namespaces[0].checkpoint(), checkpoint);
}
#[test]
fn denied_or_corrupt_source_copy_has_no_destination_effects() {
    let (mut source, mut target, mut profile, plan) = pair(b"retained source");
    profile.destination.allow_publish = false;
    let before = target.ledger.checkpoint();
    assert!(matches!(
        copy::Copier::new(profile.clone()).unwrap().apply(
            &plan,
            &mut source.resources(),
            &mut target.resources()
        ),
        Err(Error::Denied)
    ));
    assert_eq!(target.ledger.checkpoint(), before);
    assert!(!target.profile.pool.pool.repository.join("calls").exists());
    profile.destination.allow_publish = true;
    fs::write(source.output_path(), b"corrupt").unwrap();
    assert!(matches!(
        copy::Copier::new(profile).unwrap().apply(
            &plan,
            &mut source.resources(),
            &mut target.resources()
        ),
        Err(Error::Integrity)
    ));
    assert!(!target.profile.pool.pool.repository.join("calls").exists());
}

#[test]
#[ignore = "requires explicit native tools and new synthetic pools"]
fn actual_native_retained_copy_ordinary_bare_large_binary() {
    let git: PathBuf = std::env::var_os("GIT_ASSET_TEST_GIT_EXE")
        .expect("git")
        .into();
    let annex: PathBuf = std::env::var_os("GIT_ASSET_TEST_ANNEX_EXE")
        .expect("annex")
        .into();
    let paths =
        std::env::split_paths(&std::env::var_os("GIT_ASSET_TEST_ANNEX_PATH").expect("tools"))
            .collect::<Vec<_>>();
    let bytes = vec![61; 32 * 1024 * 1024];
    for bare in [false, true] {
        let tools = || Some((git.clone(), annex.clone(), paths.clone()));
        let mut source = Fixture::build(&bytes, tools(), bare);
        let mut target = Fixture::build(&bytes, tools(), !bare);
        let mut input = source.input();
        source.refresh();
        let original = source.plan.clone();
        source
            .publisher()
            .apply(&original, &mut source.resources(), &mut input)
            .unwrap();
        let profile = copy::Profile {
            source: source.profile.clone(),
            destination: target.profile.clone(),
            source_domain: k("synthetic-volume"),
            destination_domain: k("synthetic-volume"),
        };
        let plan = copy::Plan::new(&profile, original, target.plan.clone()).unwrap();
        target.refresh();
        let mut copier = copy::Copier::new(profile.clone()).unwrap();
        let mut ready = copier
            .apply(&plan, &mut source.resources(), &mut target.resources())
            .unwrap();
        let mut readback = vec![];
        ready.read_to_end(&mut readback).unwrap();
        assert_eq!(readback, bytes);
        assert_eq!(ready.receipt().declared_domains, 1);
        let receipt = ready.receipt().destination.clone();
        drop(ready);
        drop(copier);
        assert_eq!(
            source
                .publisher()
                .reopen(plan.source(), &mut source.resources())
                .unwrap()
                .receipt()
                .candidate,
            receipt.candidate
        );
        fs::rename(
            &profile.source.pool.pool.repository,
            source.root.join("offline-native-pool"),
        )
        .unwrap();
        let ready = copy::Copier::new(profile)
            .unwrap()
            .reopen(&plan, &mut target.resources())
            .unwrap();
        assert_eq!(ready.receipt().destination, receipt);
        assert!(!ready.receipt().source_verified);
    }
}
#[test]
fn source_basis_change_conflicts_with_existing_destination_operation() {
    let (mut source, mut target, profile, plan) = pair(b"original exact candidate");
    target.refresh();
    let mut copier = copy::Copier::new(profile.clone()).unwrap();
    let ready = copier
        .apply(&plan, &mut source.resources(), &mut target.resources())
        .unwrap();
    drop(ready);
    let before = target.namespaces[0].checkpoint();
    let mut changed = profile;
    changed.source_domain = k("changed-admission");
    let changed_plan =
        copy::Plan::new(&changed, plan.source().clone(), target.plan.clone()).unwrap();
    let error = copy::Copier::new(changed)
        .unwrap()
        .apply(
            &changed_plan,
            &mut source.resources(),
            &mut target.resources(),
        )
        .err()
        .unwrap();
    assert_eq!(error, Error::Conflict);
    assert_eq!(target.namespaces[0].checkpoint(), before);
}
#[test]
fn incomplete_copy_stage_requires_original_source_and_resumes_exactly() {
    let bytes = vec![18; 256 * 1024];
    let (mut source, mut target, profile, plan) = pair(&bytes);
    target.refresh();
    let mut copier = copy::Copier::new(profile).unwrap();
    assert!(matches!(
        copier.apply_hook(
            &plan,
            &mut source.resources(),
            &mut target.resources(),
            |s| if s == Stage::StagingChunk {
                Err(Error::Uncertain)
            } else {
                Ok(())
            }
        ),
        Err(Error::Uncertain)
    ));
    let checkpoint = target.namespaces[0].checkpoint();
    assert!(matches!(
        copier.resume(&plan, &mut target.resources()),
        Err(Error::Missing)
    ));
    let mut ready = copier
        .apply(&plan, &mut source.resources(), &mut target.resources())
        .unwrap();
    let mut actual = vec![];
    ready.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
    assert_eq!(target.namespaces[0].checkpoint(), checkpoint);
}

#[test]
fn retained_copy_crash_worker() {
    let Some(source_root) = std::env::var_os("GIT_ASSET_COPY_SOURCE") else {
        return;
    };
    let mut source = Fixture::load(source_root.into());
    let mut target = Fixture::load(std::env::var_os("GIT_ASSET_COPY_TARGET").unwrap().into());
    let profile = copy::Profile {
        source: source.profile.clone(),
        destination: target.profile.clone(),
        source_domain: k("one-synthetic-volume"),
        destination_domain: k("one-synthetic-volume"),
    };
    let plan: copy::Plan =
        serde_json::from_slice(&fs::read(target.root.join("copy-plan.json")).unwrap()).unwrap();
    let mode = std::env::var("GIT_ASSET_COPY_MODE").unwrap();
    let mut copier = copy::Copier::new(profile).unwrap();
    if mode == "uncertain" {
        assert!(matches!(
            copier.resume(&plan, &mut target.resources()),
            Err(Error::Uncertain)
        ));
        return;
    }
    let mut ready = if mode == "resume" {
        copier.resume(&plan, &mut target.resources()).unwrap()
    } else if mode == "apply" {
        copier
            .apply(&plan, &mut source.resources(), &mut target.resources())
            .unwrap()
    } else {
        copier
            .apply_hook(
                &plan,
                &mut source.resources(),
                &mut target.resources(),
                |s| {
                    if format!("{s:?}") == mode {
                        std::process::exit(73);
                    }
                    Ok(())
                },
            )
            .unwrap();
        panic!("copy crash boundary not reached");
    };
    let mut bytes = vec![];
    ready.read_to_end(&mut bytes).unwrap();
    assert_eq!(Object::from_bytes(&bytes), plan.source().candidate().object);
    fs::write(
        target.root.join("copy-recovered.json"),
        serde_json::to_vec(ready.receipt()).unwrap(),
    )
    .unwrap();
}
#[test]
fn actual_copy_child_exits_preserve_retention_and_original_operation() {
    for stage in [Stage::StagingChunk, Stage::Imported, Stage::Completed] {
        let (source, mut target, _profile, plan) = pair(&vec![36; 256 * 1024]);
        target.refresh();
        source.save();
        target.save();
        fs::write(
            target.root.join("copy-plan.json"),
            serde_json::to_vec(&plan).unwrap(),
        )
        .unwrap();
        let run = |mode: &str| {
            Process::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "annex::publication::tests::native::replication::retained_copy_crash_worker",
                    "--test-threads=1",
                ])
                .env("GIT_ASSET_COPY_SOURCE", &source.root)
                .env("GIT_ASSET_COPY_TARGET", &target.root)
                .env("GIT_ASSET_COPY_MODE", mode)
                .output()
                .unwrap()
        };
        let out = run(&format!("{stage:?}"));
        assert_eq!(out.status.code(), Some(73), "{out:?}");
        let recover = if stage == Stage::StagingChunk {
            "apply"
        } else {
            fs::rename(
                &source.profile.pool.pool.repository,
                source.root.join("offline-pool"),
            )
            .unwrap();
            "resume"
        };
        if stage == Stage::Imported {
            for _ in 0..2 {
                let out = run("uncertain");
                assert!(out.status.success(), "{out:?}");
            }
            assert!(!target.root.join("copy-recovered.json").exists());
            continue;
        }
        let out = run(recover);
        assert!(out.status.success(), "{out:?}");
        let receipt: copy::Receipt =
            serde_json::from_slice(&fs::read(target.root.join("copy-recovered.json")).unwrap())
                .unwrap();
        assert_eq!(receipt.plan, plan.id().unwrap());
        assert_eq!(receipt.destination.candidate, *plan.source().candidate());
    }
}
#[test]
fn copy_capacity_denial_keeps_source_and_creates_no_native_object() {
    let (mut source, mut target, profile, _old_plan) = pair(&vec![47; 128 * 1024]);
    target.refresh();
    let job = target
        .ledger
        .reserve(k("tiny-copy-job"), k("owner"), 1024)
        .unwrap()
        .reference()
        .clone();
    target.plan.admission.job = job;
    let plan = copy::Plan::new(&profile, source.plan.clone(), target.plan.clone()).unwrap();
    target.refresh();
    assert!(matches!(
        copy::Copier::new(profile).unwrap().apply(
            &plan,
            &mut source.resources(),
            &mut target.resources()
        ),
        Err(Error::Budget)
    ));
    assert!(!target.output_path().exists());
    assert_eq!(
        fs::read(source.output_path()).unwrap(),
        vec![47; 128 * 1024]
    );
}
