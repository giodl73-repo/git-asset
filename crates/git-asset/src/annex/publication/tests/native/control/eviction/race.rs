//! Parent assertions while a separate executor holds real native scopes.
use super::*;
pub(super) fn check(f: &mut Fixture, original: &Plan) {
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
