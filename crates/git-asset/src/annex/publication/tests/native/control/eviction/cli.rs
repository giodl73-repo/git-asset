//! Real facade boundary on owned synthetic fixture profiles only.
use super::*;
use serde_json::{Value, json};
pub(super) struct Client {
    exe: PathBuf,
    profile: PathBuf,
    saved: PathBuf,
}
fn publication_profile(f: &Fixture) -> Value {
    let p = &f.profile;
    let n = &p.pool.pool;
    json!({"pool":{"pool":{"git":n.git,"annex":n.annex,"repository":n.repository,
        "bare":n.bare,"uuid":n.uuid,"configuration":n.configuration,"tools":n.tools,
        "empty_config":n.empty_config,"empty_hooks":n.empty_hooks,"temporary":n.temporary,
        "command_millis":n.command_millis,"output_bytes":n.output_bytes,"limits":{"object_bytes":n.limits.object_bytes,"total_bytes":n.limits.total_bytes,
        "requests":n.limits.requests,"live_readers":n.limits.live_readers,"verify_millis":n.limits.verify_millis}},
        "binding":p.pool.binding,"members":p.pool.members.iter().map(|m|json!({"root":m.root,"minimum":m.minimum})).collect::<Vec<_>>()},
        "retained_place":p.retained_place,"staging":p.staging,"staging_marker":p.staging_marker,
        "budget":p.budget,"allow_publish":p.allow_publish})
}
fn producer(f: &Fixture, plan: &Plan) -> Value {
    json!({"profile":publication_profile(f),"plan":plan,"witness":f.root.join("witness"),
        "ledger":f.root.join("ledger"),"minimum":f.ledger.checkpoint()})
}
impl Client {
    pub fn new(
        f: &Fixture,
        b: &Fixture,
        owner: &OwnerProfile,
        plan: &NativePlan,
        pin: &crate::annex::snapshot::Snapshot,
    ) -> Option<Self> {
        let exe: PathBuf = std::env::var_os("GIT_ASSET_TEST_FRONT_DOOR_EXE")?.into();
        assert!(exe.is_absolute() && exe.is_file());
        let profile = f.root.join("cli-profile.json");
        let saved = f.root.join("cli-plan.json");
        let value = json!({"schema":"git-asset.native-storage-profile.experimental-1","owner":owner,
            "source":producer(f,&f.plan),"keeper":producer(b,&b.plan),
            "source_metadata":{"root":f.root.join("metadata"),"minimum":f.metadata.checkpoint()},
            "keeper_metadata":{"root":b.root.join("metadata"),"minimum":b.metadata.checkpoint()},
            "limits":Fixture::limits(),"allow_apply":true,"allow_resume":true,"allow_cancel":true});
        let data = serde_json::to_vec(&value).unwrap();
        assert!(data.len() <= 65536);
        fs::write(&profile, data).unwrap();
        let request = f.root.join("cli-request.json");
        fs::write(&request,serde_json::to_vec(&json!({"schema":"git-asset.native-storage-request.experimental-1",
            "operation":"native-eviction","snapshot":pin,"grant":plan.metadata_plan().request().keepers[0].grant})).unwrap()).unwrap();
        let this = Self {
            exe,
            profile,
            saved,
        };
        let out = this.invoke("plan", &this.profile, &request, true);
        assert_eq!(
            out["plan_id"],
            serde_json::to_value(plan.id().unwrap()).unwrap()
        );
        assert_eq!(out["removal_authority"], false);
        fs::write(&this.saved, serde_json::to_vec(&out).unwrap()).unwrap();
        // CLI-only mutation permissions default false, independent of sealed owner policy.
        let mut denied = value;
        for k in ["allow_apply", "allow_resume", "allow_cancel"] {
            denied.as_object_mut().unwrap().remove(k);
        }
        let denied_path = f.root.join("cli-disabled.json");
        fs::write(&denied_path, serde_json::to_vec(&denied).unwrap()).unwrap();
        for op in ["apply", "resume", "cancel"] {
            let out = this.invoke(op, &denied_path, &this.saved, false);
            assert_eq!(out["code"], "denied");
            assert_eq!(
                out["plan_id"],
                serde_json::to_value(plan.id().unwrap()).unwrap()
            );
        }
        let mut bad = out;
        bad["plan_id"] = json!(Digest::blob(b"wrong preview"));
        let bad_path = f.root.join("cli-wrong-plan.json");
        fs::write(&bad_path, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert_eq!(
            this.invoke("status", &this.profile, &bad_path, false)["code"],
            "invalid_document"
        );
        Some(this)
    }
    fn invoke(
        &self,
        op: &str,
        profile: &std::path::Path,
        document: &std::path::Path,
        success: bool,
    ) -> Value {
        let out = std::process::Command::new(&self.exe)
            .args(["storage", "native", op])
            .arg(profile)
            .arg(document)
            .output()
            .unwrap();
        assert!(out.stdout.len() <= 131072);
        assert!(
            out.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let value: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(out.status.success(), success, "{value}");
        assert_eq!(value["ok"], success, "{value}");
        value
    }
    fn child(&self, cut: Cut) -> std::process::Command {
        let stage = match cut {
            Cut::BeforeEffect => "before",
            Cut::AfterRemoval => "removed",
            Cut::AfterCompletion => "completed",
            _ => panic!("unsupported child exit boundary"),
        };
        let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
        cmd.args([
            "--exact",
            "annex::publication::tests::native::control::eviction::process::eviction_exit_worker",
            "--nocapture",
        ])
        .env("GIT_ASSET_TEST_EVICTION_PROFILE", &self.profile)
        .env("GIT_ASSET_TEST_EVICTION_PLAN", &self.saved)
        .env("GIT_ASSET_TEST_EVICTION_CUT", stage);
        cmd
    }
    pub fn exit(&self, cut: Cut) {
        assert_eq!(self.child(cut).status().unwrap().code(), Some(73));
    }
    pub fn race(&self, check: impl FnOnce()) {
        let root = self.profile.parent().unwrap().join("race-pause");
        fs::create_dir(&root).unwrap();
        let mut child = self
            .child(Cut::BeforeEffect)
            .env("GIT_ASSET_TEST_EVICTION_PAUSE", &root)
            .spawn()
            .unwrap();
        let started = std::time::Instant::now();
        while !root.join("ready").exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "child exited before pause"
            );
            if started.elapsed() > std::time::Duration::from_secs(180) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("child readiness timeout");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let checked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(check));
        fs::write(root.join("release"), b"continue_to_exit").unwrap();
        assert_eq!(child.wait().unwrap().code(), Some(73));
        if let Err(error) = checked {
            std::panic::resume_unwind(error);
        }
    }
    pub fn call(&self, op: &str, success: bool) -> Value {
        self.invoke(op, &self.profile, &self.saved, success)
    }
}
