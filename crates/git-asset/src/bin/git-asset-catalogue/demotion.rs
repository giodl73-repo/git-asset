use git_asset_core::{Digest, Object};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

fn read(path: &Path, limit: u64) -> Result<(Value, Digest), String> {
    if !path.is_absolute() {
        return Err("Evidence paths must be absolute".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(limit + 1).read_to_end(&mut bytes))
        .map_err(|_| "Cannot read evidence")?;
    if bytes.len() as u64 > limit {
        return Err("Evidence exceeds byte budget".into());
    }
    let value = serde_json::from_slice(&bytes).map_err(|_| "Malformed evidence JSON")?;
    Ok((value, Digest::blob(&bytes)))
}

pub(super) fn compare(drive: &Path, policy: &Path, plan: &Path) -> Result<Value, String> {
    let (drive, drive_digest) = read(drive, 16 * 1024 * 1024)?;
    let (policy, policy_digest) = read(policy, 256 * 1024)?;
    let (plan, plan_digest) = read(plan, 32 * 1024 * 1024)?;
    let mut report = match_candidates(&drive, &policy, &policy_digest, &plan)?;
    report["drive_report_digest"] = json!(drive_digest);
    report["provider_plan_digest"] = json!(plan_digest);
    Ok(report)
}

fn match_candidates(
    drive: &Value,
    policy: &Value,
    policy_digest: &Digest,
    plan: &Value,
) -> Result<Value, String> {
    if drive["schema"] != "git-asset.drive-report.experimental-1"
        || drive["inventory"]["layout"] != "reel-cas"
        || policy["schema"] != "reel.asset-tier-policy.v1"
        || plan["schema"] != "reel.asset-tier-plan.v1"
        || plan["policy_sha256"] != policy_digest.as_str()
        || policy["project_id"].as_str().is_none_or(str::is_empty)
        || policy["project_id"] != plan["project_id"]
    {
        return Err("Provider policy or report basis mismatch".into());
    }
    let root = drive["inventory"]["cache_root"]
        .as_str()
        .ok_or("Missing cache root")?;
    let selected = policy["selected_tier"]
        .as_str()
        .ok_or("Missing selected tier")?;
    let tiers = policy["tiers"].as_array().ok_or("Missing policy tiers")?;
    let mut sources = tiers
        .iter()
        .filter(|tier| tier["id"] == selected && tier["storage"] == "active");
    let tier = sources
        .next()
        .and_then(|tier| tier["index"].as_u64())
        .ok_or("Missing active source tier")?;
    let observations = plan["tier_observations"]
        .as_array()
        .ok_or("Missing tier observations")?;
    let mut observed = observations.iter().filter(|row| row["tier"] == tier);
    if !Path::new(root).is_absolute()
        || sources.next().is_some()
        || observed
            .next()
            .and_then(|row| row["root"].as_str())
            .map(Path::new)
            != Some(Path::new(root))
        || observed.next().is_some()
    {
        return Err("Provider source tier does not match reviewed root".into());
    }
    let review = drive["eviction_review"]["largest_observed"]
        .as_array()
        .ok_or("Missing eviction review")?;
    let rows = plan["objects"]
        .as_array()
        .ok_or("Missing provider objects")?;
    let mut proposed = BTreeMap::new();
    for row in rows {
        if row["class"] == "previously-used" && row["proposal"] == "verify-and-demote" {
            let hash = row["sha256"]
                .as_str()
                .ok_or("Missing provider object digest")?;
            let bytes = row["bytes"]
                .as_u64()
                .ok_or("Missing provider object length")?;
            if proposed.insert(hash, bytes).is_some() {
                return Err("Duplicate provider candidate".into());
            }
        }
    }
    let mut matches = Vec::new();
    for row in review {
        if row["eviction_review"]["state"] != "verify_candidate"
            || row["has_current_revision"] != false
        {
            continue;
        }
        let object: Object =
            serde_json::from_value(row["eviction_review"]["reviewed_object"].clone())
                .map_err(|_| "Invalid reviewed object")?;
        if let Some(bytes) = proposed.remove(object.sha256.as_str()) {
            if bytes != object.bytes {
                return Err("Provider object length differs from review".into());
            }
            matches.push(object);
        }
    }
    let plan_id: Digest = serde_json::from_value(plan["plan_sha256"].clone())
        .map_err(|_| "Invalid provider plan digest")?;
    Ok(json!({
        "schema":"git-asset.reel-demotion-match.experimental-1","cache_root":root,
        "project":drive["inventory"]["project"],"snapshot":drive["selection_usage"]["snapshot"],
        "provider_project":plan["project_id"],"provider_plan_sha256":plan_id,
        "provider_policy_sha256":policy_digest,"matched":matches.len(),"objects":matches,
        "scope":"saved_review_page","provider_verified":false,"removal_authorized":false
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_exact_candidate_and_refuses_wrong_root_or_length() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let object = Object::from_bytes(b"old asset");
        let drive = json!({
            "schema":"git-asset.drive-report.experimental-1",
            "inventory":{"cache_root":root,"project":"fixture","layout":"reel-cas"},
            "eviction_review":{"largest_observed":[{"has_current_revision":false,
                "eviction_review":{"state":"verify_candidate","reviewed_object":object}}]}
        });
        let digest = Digest::blob(b"policy");
        let policy = json!({
            "schema":"reel.asset-tier-policy.v1","project_id":"fixture","selected_tier":"working",
            "tiers":[{"id":"working","storage":"active","index":0}]
        });
        let mut plan = json!({
            "schema":"reel.asset-tier-plan.v1","project_id":"fixture","policy_sha256":digest,
            "plan_sha256":Digest::blob(b"provider plan"),
            "tier_observations":[{"tier":0,"root":root}],
            "objects":[{"class":"previously-used","proposal":"verify-and-demote",
                "sha256":object.sha256,"bytes":object.bytes}]
        });
        let matched = match_candidates(&drive, &policy, &digest, &plan).unwrap();
        assert_eq!(matched["objects"], json!([object]));
        assert_eq!(matched["removal_authorized"], false);
        plan["objects"][0]["bytes"] = json!(object.bytes + 1);
        assert!(match_candidates(&drive, &policy, &digest, &plan).is_err());
        plan["objects"][0]["bytes"] = json!(object.bytes);
        plan["tier_observations"][0]["root"] = json!(root.join("other"));
        assert!(match_candidates(&drive, &policy, &digest, &plan).is_err());
    }
}
