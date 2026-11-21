//! Compose existing independent rehearsals; never invent observations or custody.
use super::*;
use git_asset::catalogue::{Catalogue, Config};

pub(super) fn run(requested: &Path) -> Result<Value> {
    let root = fresh_root(requested)?;
    let mut projects = Vec::new();
    let mut receipts = Vec::new();
    let mut teaching = Value::Null;
    for (id, label, mode, checkpoints) in [
        ("intro", "Intro sample · current v2", Mode::Ordinary, 7),
        (
            "recovery",
            "Recovery sample · returned v1",
            Mode::ReturnRestore,
            11,
        ),
    ] {
        eprintln!("Preparing {id} rehearsal ({checkpoints} checkpoints)…");
        let child = root.join(id);
        let result = Demo::new(&child, mode)?.run(mode)?;
        if result["completed_checkpoints"] != checkpoints {
            return Err("review rehearsal checkpoint mismatch".into());
        }
        let receipt_name = format!("{id}-result.json");
        let receipt = save(&root.join(&receipt_name), &result)?;
        let mut enrollment: Value = identity::parse_json(&fs::read(child.join("catalogue.json"))?)?;
        let mut project = enrollment["projects"]
            .as_array_mut()
            .filter(|items| items.len() == 1)
            .ok_or("review needs one project per rehearsal")?
            .remove(0);
        project["id"] = json!(id);
        project["label"] = json!(label);
        projects.push(project);
        if id == "recovery" {
            teaching = enrollment["teaching"].take();
            teaching["project"] = json!(id);
        }
        receipts.push(
            json!({"project":id,"checkpoints":checkpoints,"path":receipt_name,"object":receipt}),
        );
    }
    let catalogue = root.join("catalogue.json");
    eprintln!("Checking the combined catalogue and exact shared bytes…");
    let enrollment = save(
        &catalogue,
        &json!({"schema":"tig.catalogue-enrollment.experimental-1","projects":projects,"teaching":teaching}),
    )?;
    let loaded = Catalogue::load(Config::load(&catalogue)?)?;
    let shared = loaded.shared_selected("intro", "recovery", "")?;
    if shared["shared_objects"] != 1 || shared["shared_unique_bytes"] != 25 {
        return Err("independent review shared-object oracle mismatch".into());
    }
    let result = json!({"schema":"git-asset.sample-review.experimental-1","root":root,
        "catalogue":{"path":"catalogue.json","object":enrollment},"projects":receipts,
        "visible_recovery_checkpoints":11,"shared_selected_objects":1,"shared_selected_bytes":25,
        "viewer_started":false,"complete_course":false});
    save(&root.join("review.json"), &result)?;
    Ok(result)
}
