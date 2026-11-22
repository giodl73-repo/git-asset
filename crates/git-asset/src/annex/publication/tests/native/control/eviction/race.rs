//! Parent assertions while a separate executor holds real native scopes.
use super::*;
pub(super) fn check(f: &mut Fixture, original: &Plan, control: &mut Control) {
    f.namespaces[0]
        .apply(
            k("race-enroll"),
            Command::Enroll {
                source: k("race-job"),
                owner: k("owner"),
                class: Kind::Job,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    assert!(matches!(
        f.namespaces[0].select(
            &Select {
                prepare: k("race-prepare"),
                adopt: k("race-adopt"),
                source: k("race-job"),
                owner: k("owner"),
                expected_revision: 1,
                selection: original.selection.selection.clone(),
            },
            &f.metadata,
            &Fixture::limits()
        ),
        Err(git_asset_store::source_claims::admission::Error::Busy)
    ));
    f.namespaces[0]
        .apply(
            k("race-unrelated"),
            Command::Enroll {
                source: k("race-unrelated-source"),
                owner: k("owner"),
                class: Kind::Branch,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    publish_other(f, original, control);
    let pool = &f.profile.pool.pool;
    let config = pool
        .repository
        .join(if pool.bare { "config" } else { ".git/config" });
    let before = fs::read(&config).unwrap();
    let output = std::process::Command::new(&pool.git)
        .arg("-C")
        .arg(&pool.repository)
        .args([
            "config",
            "--local",
            "--replace-all",
            "gitasset.evictionpolicy",
            Digest::blob(b"not admitted").as_str(),
        ])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", &pool.empty_config)
        .output()
        .unwrap();
    assert!(output.stdout.len() + output.stderr.len() < 16384);
    assert!(
        !output.status.success(),
        "configuration changed across live native guard"
    );
    assert_eq!(fs::read(&config).unwrap(), before);
    assert_eq!(Object::from_bytes(&before), pool.configuration);
}

fn publish_other(f: &mut Fixture, original: &Plan, control: &mut Control) {
    let bytes = vec![92; 256 * 1024];
    let mut request = original.request.clone();
    request.operation = k("parallel-publication");
    request.source = k("parallel-asset");
    request.expected_revision = 1;
    request.candidate.logical_id = "parallel/asset".into();
    request.candidate.object = Object::from_bytes(&bytes);
    f.namespaces[0]
        .apply(
            k("parallel-enroll"),
            Command::Enroll {
                source: request.source.clone(),
                owner: request.owner.clone(),
                class: Kind::Retention,
            },
            &f.metadata,
            &Fixture::limits(),
        )
        .unwrap();
    let mut metadata =
        MetadataStore::open(&f.root.join("metadata"), f.metadata.checkpoint()).unwrap();
    let selection = freeze(&request, &mut metadata, TreeLimits::default()).unwrap();
    let cp = metadata.finish().unwrap();
    drop(metadata);
    f.metadata = MetadataReader::open(&f.root.join("metadata"), &cp).unwrap();
    let data = control::document(&request.candidate.object).unwrap();
    let metadata_request =
        control::initial_request(&request.candidate.object, k("controller")).unwrap();
    let publication = accounted::plan_bound(
        &control.store,
        &metadata_request,
        control::writer_context(&f.profile, &request).unwrap(),
    )
    .unwrap();
    accounted::publish_accounted(
        &mut control.ledger,
        &control.store,
        &control.job,
        &publication,
        &mut Cursor::new(&data),
    )
    .unwrap();
    let plan = Plan::controlled(
        &f.profile,
        request,
        selection,
        original.admission.clone(),
        f.control_profile.as_ref().unwrap(),
        control::Plan {
            publication,
            job: control.job.clone(),
        },
    )
    .unwrap();
    fs::write(
        f.root
            .join("input/objects")
            .join(plan.candidate().object.sha256.as_str()),
        &bytes,
    )
    .unwrap();
    let mut input = f.input();
    let mut publisher = f.publisher();
    let mut ready = publisher
        .apply(&plan, &mut f.resources(), &mut input)
        .unwrap();
    let mut actual = Vec::new();
    ready.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
    drop(ready);
    let mut reopened = publisher.reopen(&plan, &mut f.resources()).unwrap();
    let mut actual = Vec::new();
    reopened.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes);
    drop(reopened);
    assert_eq!(f.plan, *original);
    assert!(
        f.namespaces[0]
            .source(&k("creative"))
            .unwrap()
            .unwrap()
            .current
            .is_none()
    );
}
