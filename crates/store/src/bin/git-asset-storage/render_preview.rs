use super::{Result, document, report};
use git_asset_core::{Digest, LogicalPath, Object};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const REQUEST: &str = "git-asset.render-retirement-preview-request.experimental-1";
const MAX_VERSIONS: usize = 128;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    predecessor: String,
    replacement: String,
    observed_at_ms: u64,
    minimum_age_ms: u64,
    versions: Vec<Version>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Version {
    binding: LogicalPath,
    object: Object,
    superseded_at_ms: Option<u64>,
    derived_render: Option<bool>,
    protected: Option<bool>,
    rebuild_evidence: Option<Digest>,
}

fn exact_commit(commit: &str) -> bool {
    matches!(commit.len(), 40 | 64)
        && commit
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && commit.bytes().any(|b| b != b'0')
}

fn preview(request: Request) -> Result<Vec<u8>> {
    if request.schema != REQUEST
        || !exact_commit(&request.predecessor)
        || !exact_commit(&request.replacement)
        || request.predecessor == request.replacement
        || request.predecessor.len() != request.replacement.len()
        || request.minimum_age_ms == 0
        || request.versions.len() > MAX_VERSIONS
    {
        return Err("invalid_request");
    }
    let mut bindings = BTreeSet::new();
    let mut objects = BTreeMap::new();
    let mut rows = Vec::new();
    for version in request.versions {
        if !bindings.insert(version.binding.clone()) {
            return Err("duplicate_binding");
        }
        let mut holds = Vec::new();
        if version.derived_render != Some(true) {
            holds.push("derived_render_unconfirmed");
        }
        if version.protected != Some(false) {
            holds.push("protection_present_or_unknown");
        }
        match version
            .superseded_at_ms
            .and_then(|time| request.observed_at_ms.checked_sub(time))
        {
            None => holds.push("supersession_age_unknown"),
            Some(age) if age < request.minimum_age_ms => holds.push("grace_period"),
            Some(_) => {}
        }
        if version.rebuild_evidence.is_none() {
            holds.push("rebuild_evidence_missing");
        }
        let entry = objects
            .entry(version.object.sha256.clone())
            .or_insert((version.object.bytes, false));
        if entry.0 != version.object.bytes {
            return Err("conflicting_object_length");
        }
        entry.1 |= !holds.is_empty();
        rows.push((version, holds));
    }
    let mut eligible_logical_bytes = 0u64;
    for (bytes, held) in objects.values() {
        if !held {
            eligible_logical_bytes = eligible_logical_bytes
                .checked_add(*bytes)
                .ok_or("byte_overflow")?;
        }
    }
    let rows: Vec<_> = rows
        .into_iter()
        .map(|(version, mut holds)| {
            if holds.is_empty() && objects[&version.object.sha256].1 {
                holds.push("shared_object_held");
            }
            json!({"binding":version.binding,"object":version.object,"holds":holds,
            "eligible_under_supplied_evidence":holds.is_empty()})
        })
        .collect();
    report(json!({
        "schema":"git-asset.render-retirement-preview.experimental-1",
        "predecessor":request.predecessor,"replacement":request.replacement,
        "observed_at_ms":request.observed_at_ms,"minimum_age_ms":request.minimum_age_ms,
        "rows":rows,"eligible_unique_logical_bytes":eligible_logical_bytes,
        "physical_reclaim_bytes":null,"archive_destination":null,
        "evidence_verified_by_command":false,"removal_authority":false
    }))
}

pub(super) fn run(args: &[String]) -> Result<Vec<u8>> {
    match args {
        [action] if action == "describe" => report(json!({
            "operations":["preview REQUEST"],"request_schema":REQUEST,
            "max_document_bytes":65536,"max_versions":MAX_VERSIONS,
            "removal_authority":false
        })),
        [action, path] if action == "preview" => preview(document(Path::new(path))?),
        _ => Err("usage"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn version(binding: &str, bytes: &[u8]) -> Version {
        Version {
            binding: serde_json::from_value(json!(binding)).unwrap(),
            object: Object::from_bytes(bytes),
            superseded_at_ms: Some(50),
            derived_render: Some(true),
            protected: Some(false),
            rebuild_evidence: Some(Digest::blob(b"retained rebuild evidence")),
        }
    }

    fn request(versions: Vec<Version>) -> Request {
        Request {
            schema: REQUEST.to_owned(),
            predecessor: "a".repeat(40),
            replacement: "b".repeat(40),
            observed_at_ms: 100,
            minimum_age_ms: 50,
            versions,
        }
    }

    #[test]
    fn counts_shared_eligible_bytes_once_and_holds_protected_aliases() {
        let mut protected = version("active", b"held");
        protected.protected = Some(true);
        let rows = vec![
            version("old/a", b"render"),
            version("old/b", b"render"),
            version("old/c", b"held"),
            protected,
        ];
        let result: Value = serde_json::from_slice(&preview(request(rows)).unwrap()).unwrap();
        assert_eq!(result["eligible_unique_logical_bytes"], 6);
        assert_eq!(result["rows"][2]["holds"], json!(["shared_object_held"]));
        assert_eq!(result["archive_destination"], Value::Null);
        assert_eq!(result["physical_reclaim_bytes"], Value::Null);
        assert_eq!(result["removal_authority"], false);
    }

    #[test]
    fn unknown_evidence_and_future_dates_hold() {
        let mut unknown = version("unknown", b"render");
        unknown.protected = None;
        unknown.rebuild_evidence = None;
        unknown.superseded_at_ms = Some(101);
        let result: Value =
            serde_json::from_slice(&preview(request(vec![unknown])).unwrap()).unwrap();
        assert_eq!(result["eligible_unique_logical_bytes"], 0);
        assert_eq!(
            result["rows"][0]["holds"],
            json!([
                "protection_present_or_unknown",
                "supersession_age_unknown",
                "rebuild_evidence_missing"
            ])
        );
    }
}
