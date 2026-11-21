use super::*;
use git_asset_core::tree::MemoryNodes;
#[cfg(windows)]
use git_asset_store::source_claims::Places;
use git_asset_store::{
    source_claims::Checkpoint,
    store_protection::{Binding, Member},
};
use std::{io::Cursor, path::Path};
#[cfg(windows)]
mod native;
fn k(s: &str) -> Key {
    Key::parse(s).unwrap()
}
fn request(nodes: &mut impl NodeStore) -> Request {
    Request {
        operation: k("attempt"),
        owner: k("owner"),
        source: k("asset"),
        expected_revision: 1,
        candidate: Candidate {
            logical_id: "episode/E01/image".into(),
            object: Object::from_bytes(b"original result"),
            provenance: nodes.put(b"opaque provenance packet").unwrap(),
        },
    }
}
fn profile(root: &Path) -> Profile {
    let ns = Digest::blob(b"namespace");
    let minimum: Checkpoint = serde_json::from_value(
        serde_json::json!({"genesis":ns,"sequence":0,"hash":Digest::blob(b"header")}),
    )
    .unwrap();
    Profile {
        pool: managed::Profile {
            pool: super::super::Profile {
                git: root.join("git.exe"),
                annex: root.join("annex.exe"),
                repository: root.join("pool"),
                bare: false,
                uuid: "00000000-0000-0000-0000-000000000001".into(),
                configuration: Object::from_bytes(b"config"),
                tools: vec![root.into()],
                empty_config: root.join("empty-config"),
                empty_hooks: root.join("hooks"),
                temporary: root.join("tmp"),
                command_millis: 3000,
                output_bytes: 16384,
                limits: existing::Limits {
                    object_bytes: 64 * 1024 * 1024,
                    total_bytes: 1024 * 1024 * 1024,
                    requests: 64,
                    live_readers: 8,
                    verify_millis: 300000,
                },
            },
            binding: Binding::new(
                1,
                Digest::blob(b"participating boundary"),
                vec![Member {
                    namespace: ns,
                    places: vec![k("pool")],
                }],
            )
            .unwrap(),
            members: vec![managed::Member {
                root: root.join("namespace"),
                minimum,
            }],
        },
        retained_place: k("pool"),
        staging: root.join("staging"),
        staging_marker: Object::from_bytes(b"stage marker"),
        budget: Budget {
            object_bytes: 64 * 1024 * 1024,
            transfer_bytes: 256 * 1024 * 1024,
            millis: 300000,
        },
        allow_publish: true,
    }
}
fn plan(nodes: &mut impl NodeStore, p: &Profile, r: Request) -> Plan {
    let frozen = freeze(&r, nodes, TreeLimits::default()).unwrap();
    let capacity = Digest::blob(b"capacity");
    let job =
        serde_json::from_value(serde_json::json!({"genesis":capacity,"operation":"job"})).unwrap();
    Plan::new(
        p,
        r,
        frozen,
        Admission {
            namespace: p.pool.members[0].minimum.identity().clone(),
            witness: Digest::blob(b"witness"),
            capacity,
            job,
        },
    )
    .unwrap()
}
#[test]
fn plan_freeze_exact_membership_and_constructors_are_pure() {
    let root = tempfile::tempdir().unwrap();
    let p = profile(root.path());
    let mut nodes = MemoryNodes::default();
    let r = request(&mut nodes);
    let q = plan(&mut nodes, &p, r);
    verify(&q, &mut nodes, TreeLimits::default()).unwrap();
    assert_eq!(bindings(&q.request).unwrap().len(), 1);
    assert_eq!(q, serde_json::from_slice(&encode(&q).unwrap()).unwrap());
    assert_eq!(q.id().unwrap(), digest(&q).unwrap());
    Publisher::new(p.clone()).unwrap();
    assert!(!p.pool.pool.repository.exists());
    assert!(!p.staging.exists());
    let mut bad = q.clone();
    bad.request.candidate.logical_id = "replacement".into();
    assert_eq!(
        verify(&bad, &mut nodes, TreeLimits::default()).unwrap_err(),
        Error::Integrity
    );
}
#[test]
fn provider_failures_remain_distinct_from_corruption_and_admission() {
    use git_asset_core::tree::{TreeError, TreeResult};
    struct Failing(TreeError);
    impl NodeStore for Failing {
        fn get(&self, _: &Object) -> TreeResult<Option<Vec<u8>>> {
            Err(self.0.clone())
        }
        fn put(&mut self, _: &[u8]) -> TreeResult<Object> {
            Err(self.0.clone())
        }
    }
    let root = tempfile::tempdir().unwrap();
    let mut nodes = MemoryNodes::default();
    let request = request(&mut nodes);
    let p = plan(&mut nodes, &profile(root.path()), request.clone());
    for (failure, expected) in [
        (
            TreeError::Store("provider unavailable".into()),
            Error::NeedsRecovery,
        ),
        (TreeError::Budget, Error::Budget),
        (TreeError::Integrity, Error::Integrity),
    ] {
        let mut failing = Failing(failure);
        assert_eq!(
            freeze(&request, &mut failing, TreeLimits::default()).unwrap_err(),
            expected
        );
        assert_eq!(
            verify(&p, &mut failing, TreeLimits::default()).unwrap_err(),
            expected
        );
    }
}
#[test]
fn missing_provenance_and_finite_metadata_and_payload_budgets_refuse() {
    let root = tempfile::tempdir().unwrap();
    let mut p = profile(root.path());
    let mut nodes = MemoryNodes::default();
    let mut r = request(&mut nodes);
    let q = plan(&mut nodes, &p, r.clone());
    assert_eq!(
        verify(
            &q,
            &mut nodes,
            TreeLimits {
                max_reads: 1,
                ..TreeLimits::default()
            }
        )
        .unwrap_err(),
        Error::Budget
    );
    r.candidate.provenance = Object::from_bytes(b"missing");
    assert_eq!(
        freeze(&r, &mut nodes, TreeLimits::default()).unwrap_err(),
        Error::Missing
    );
    p.budget.transfer_bytes = 1;
    assert_eq!(
        Plan::new(
            &p,
            q.request.clone(),
            q.selection.selection.clone(),
            q.admission.clone()
        )
        .unwrap_err(),
        Error::Budget
    );
}
#[test]
fn staging_recovers_exact_prefix_without_duplicate_files_or_consuming_source() {
    let root = tempfile::tempdir().unwrap();
    let p = profile(root.path());
    fs::create_dir(&p.staging).unwrap();
    let bytes = vec![37u8; 150000];
    let mut nodes = MemoryNodes::default();
    let mut r = request(&mut nodes);
    r.candidate.object = Object::from_bytes(&bytes);
    let q = plan(&mut nodes, &p, r);
    let mut once = true;
    assert_eq!(
        staging::prepare(
            &p,
            &q,
            &mut Work::new(p.budget),
            &mut || Ok(Cursor::new(bytes.clone())),
            &mut |s| {
                if s == Stage::StagingChunk && once {
                    once = false;
                    Err(Error::Uncertain)
                } else {
                    Ok(())
                }
            }
        )
        .unwrap_err(),
        Error::Uncertain
    );
    let path = staging::path(&p, &q).unwrap();
    assert_eq!(fs::read(&path).unwrap(), bytes[..65536]);
    staging::prepare(
        &p,
        &q,
        &mut Work::new(p.budget),
        &mut || Ok(Cursor::new(bytes.clone())),
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), bytes);
    staging::prepare::<Cursor<Vec<u8>>>(
        &p,
        &q,
        &mut Work::new(p.budget),
        &mut || panic!("complete stage must not open input"),
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
}
#[test]
fn corrupt_and_oversize_staging_are_preserved_and_budgets_stop_growth() {
    let root = tempfile::tempdir().unwrap();
    let p = profile(root.path());
    fs::create_dir(&p.staging).unwrap();
    let mut nodes = MemoryNodes::default();
    let r = request(&mut nodes);
    let q = plan(&mut nodes, &p, r);
    let path = staging::path(&p, &q).unwrap();
    fs::create_dir(path.parent().unwrap()).unwrap();
    for bad in [
        b"bad".as_slice(),
        b"too many bytes in this stage".as_slice(),
    ] {
        fs::write(&path, bad).unwrap();
        assert_eq!(
            staging::prepare(
                &p,
                &q,
                &mut Work::new(p.budget),
                &mut || Ok(Cursor::new(b"original result")),
                &mut |_| Ok(())
            )
            .unwrap_err(),
            Error::Integrity
        );
        assert_eq!(fs::read(&path).unwrap(), bad);
    }
    fs::write(&path, b"").unwrap();
    let mut budget = p.budget;
    budget.transfer_bytes = 1;
    assert_eq!(
        staging::prepare(
            &p,
            &q,
            &mut Work::new(budget),
            &mut || Ok(Cursor::new(b"original result")),
            &mut |_| Ok(())
        )
        .unwrap_err(),
        Error::Budget
    );
    assert_eq!(fs::metadata(path).unwrap().len(), 0);
}

#[cfg(windows)]
pub(super) fn fake_tool() -> &'static Path {
    use std::sync::OnceLock;
    static TOOL: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    &TOOL.get_or_init(|| {
        let root=tempfile::tempdir().unwrap();let source=root.path().join("fixture.rs");let exe=root.path().join("fixture.exe");
        fs::write(&source,r#"
use std::{fs,io::Write,path::Path};
fn main(){
 let a:Vec<_>=std::env::args().skip(1).collect();
 let mut log=fs::OpenOptions::new().create(true).append(true).open("calls").unwrap();writeln!(log,"{}",a.join(" ")).unwrap();
 let mode=fs::read_to_string("mode").unwrap_or_default();
 if a==["config","--local","--no-includes","--get-all","annex.uuid"] {println!("{}",fs::read_to_string("uuid").unwrap_or("00000000-0000-0000-0000-000000000001".into()));return;}
 if a==["config","--local","--no-includes","--get-all","annex.version"] {println!("10");return;}
 if a==["config","--local","--no-includes","--get-all","tig.protectionbinding"] {println!("{}",fs::read_to_string("binding").unwrap());return;}
 if a.len()==5 && a[..4]==["config","--local","--no-includes","--get-regexp"] {
  println!("tig.protectionbinding {}",fs::read_to_string("binding").unwrap());
  if let Ok(c)=fs::read_to_string("control") {println!("gitasset.lifecyclecontrol {}",c);}return;
 }
 let prefix=if std::env::var("GIT_DIR").unwrap()=="." {"annex/objects"}else{".git/annex/objects"};
 if a.len()==2 && a[0]=="contentlocation" {
  let p=format!("{prefix}/a/b/{0}/{0}",a[1]);if !Path::new(&p).exists(){std::process::exit(1);}println!("{p}");return;
 }
 if a.len()==6 && a[..5]==["reinject","--guesskeys","--json","--json-error-messages","--"] {
  let src=Path::new(&a[5]);let key=src.file_name().unwrap().to_str().unwrap();let dir=Path::new(prefix).join("a/b").join(key);
  fs::create_dir_all(&dir).unwrap();fs::rename(src,dir.join(key)).unwrap();
  if mode=="lost-reply" {eprintln!("synthetic lost response");std::process::exit(2);}
  if mode=="bad-bytes" {fs::write(dir.join(key),b"corrupt").unwrap();}
  println!("{{\"success\":true,\"key\":\"{key}\"}}");return;
 }
 eprintln!("inadmitted command");std::process::exit(2);
}
"#).unwrap();
        let out=std::process::Command::new("rustc").arg("--edition=2024").arg(&source).arg("-o").arg(&exe).output().unwrap();
        assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stderr));(root,exe)
    }).1
}

mod replication;
