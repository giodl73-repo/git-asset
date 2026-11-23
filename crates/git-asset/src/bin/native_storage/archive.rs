//! Explicit copy of one existing cache object to an existing archive namespace.
use git_asset::restore;
use git_asset_core::{Digest, Object};
use git_asset_store::existing::{
    ByteSource, Limits, Place, Profile, ReadCache, RequiredProtection,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{ffi::OsString, fs, path::Path, process::ExitCode};

const SCHEMA: &str = "git-asset.cache-archive-request.experimental-1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    object: Object,
    source: Place,
    archive: Place,
    minimum_free_bytes: u64,
}

fn reader(place: Place) -> Result<ReadCache, String> {
    ReadCache::new(Profile {
        places: vec![place],
        required: RequiredProtection::PreventMutation,
        limits: Limits::default(),
    })
    .map_err(|error| error.to_string())
}

fn run(args: &[OsString]) -> Result<Value, String> {
    if args.len() == 1 && args[0] == "describe" {
        return Ok(
            json!({"schema":SCHEMA,"operations":["plan REQUEST","apply REQUEST EXPECTED_PLAN","status REQUEST","estimate MANIFEST"],"available":cfg!(windows),"removal_authorized":false}),
        );
    }
    let op = args.first().and_then(|a| a.to_str()).ok_or("usage")?;
    if !matches!(
        (op, args.len()),
        ("plan" | "status" | "estimate", 2) | ("apply", 3)
    ) {
        return Err(
            "usage: git asset storage archive {plan|status|estimate} REQUEST or apply REQUEST EXPECTED_PLAN"
                .into(),
        );
    }
    if !cfg!(windows) {
        return Err("unsupported: Windows guarded cache reader required".into());
    }
    let raw = super::profile::raw(Path::new(&args[1]), 65536)?;
    if op == "estimate" {
        return estimate(super::profile::parse(&raw)?);
    }
    let req: Request = super::profile::parse(&raw)?;
    let plan = Digest::blob(&raw);
    if req.schema != SCHEMA || req.object.bytes > Limits::default().object_bytes {
        return Err("invalid archive request schema or object size".into());
    }
    if op == "apply" && args[2].to_str() != Some(plan.as_str()) {
        return Err("stale plan".into());
    }
    for root in [&req.source.root, &req.archive.root] {
        if !root.is_absolute()
            || root.components().any(|c| {
                matches!(
                    c,
                    std::path::Component::CurDir | std::path::Component::ParentDir
                )
            })
            || !restore::ordinary(root)?.is_dir()
        {
            return Err("existing ordinary absolute cache roots required".into());
        }
    }
    let source_root = fs::canonicalize(&req.source.root).map_err(|_| "source root unavailable")?;
    let archive_root =
        fs::canonicalize(&req.archive.root).map_err(|_| "archive root unavailable")?;
    if source_root.starts_with(&archive_root) || archive_root.starts_with(&source_root) {
        return Err("overlapping cache roots refused".into());
    }
    let destination = req
        .archive
        .layout
        .object_path(&req.archive.root, &req.object.sha256);
    let existing = restore::check_destination(&destination, &req.object, true)?;
    let mut archive = reader(req.archive)?;
    let mut copied_bytes = 0;
    if op != "status" {
        let mut source = reader(req.source)?;
        if op == "apply" && !existing {
            let held = source
                .open_verified(&req.object)
                .map_err(|e| e.to_string())?;
            let required = req
                .minimum_free_bytes
                .checked_add(req.object.bytes)
                .ok_or("capacity overflow")?;
            if fs2::available_space(&archive_root).map_err(|_| "archive capacity unavailable")?
                < required
            {
                return Err("archive capacity hold".into());
            }
            restore::create_parents(&destination)?;
            drop(held);
            let outcome = restore::restore(&mut source, &req.object, &destination, true)
                .map_err(|e| format!("archive apply outcome may require status: {e}"))?;
            copied_bytes = outcome.copied_bytes;
        } else {
            source
                .open_verified(&req.object)
                .map_err(|e| e.to_string())?;
        }
    }
    let verified = existing || op == "apply";
    if verified {
        archive
            .open_verified(&req.object)
            .map_err(|e| format!("archive readback: {e}"))?;
    }
    Ok(
        json!({"schema":"git-asset.cache-archive-report.experimental-1","ok":true,"operation":op,"plan":plan,"object":req.object,"archive_copy_verified":verified,"copied_bytes":copied_bytes,"retained_custody_established":false,"removal_authorized":false}),
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Estimate {
    schema: String,
    archive: Place,
    objects: Vec<Object>,
}

fn estimate(req: Estimate) -> Result<Value, String> {
    if req.schema != "git-asset.cache-archive-estimate.experimental-1"
        || req.objects.is_empty()
        || req.objects.len() > 1024
        || !req.archive.root.is_absolute()
        || req.archive.root.components().any(|c| {
            matches!(
                c,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )
        })
    {
        return Err("Invalid archive estimate admission".into());
    }
    for dir in req.archive.root.ancestors() {
        if !restore::ordinary(dir)?.is_dir() {
            return Err("Existing ordinary archive root required".into());
        }
    }
    let mut unique = std::collections::BTreeMap::new();
    for object in req.objects {
        if unique
            .insert(object.sha256.clone(), object.bytes)
            .is_some_and(|prior| prior != object.bytes)
        {
            return Err("Conflicting byte lengths for one object".into());
        }
    }
    let mut totals = std::collections::BTreeMap::<&str, (u64, u64)>::new();
    for (sha256, bytes) in &unique {
        let path = req.archive.layout.object_path(&req.archive.root, sha256);
        let mut state = "unknown";
        if let Some(parent) = path.parent() {
            let mut ready = true;
            for dir in parent.ancestors().take_while(|p| *p != req.archive.root) {
                match fs::symlink_metadata(dir) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        state = "absent_at_named_path";
                        ready = false;
                    }
                    _ if restore::ordinary(dir).is_ok_and(|m| m.is_dir()) => {}
                    _ => {
                        state = "unknown";
                        ready = false;
                        break;
                    }
                }
            }
            if ready {
                state = match fs::symlink_metadata(&path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        "absent_at_named_path"
                    }
                    Ok(_)
                        if restore::ordinary(&path)
                            .is_ok_and(|m| m.is_file() && m.len() == *bytes) =>
                    {
                        "length_matches_unverified"
                    }
                    Ok(_) => "conflicting_or_unsafe_leaf",
                    Err(_) => "unknown",
                };
            }
        }
        let bucket = totals.entry(state).or_default();
        bucket.0 += 1;
        bucket.1 = bucket
            .1
            .checked_add(*bytes)
            .ok_or("Archive estimate overflow")?;
    }
    let groups: Vec<_> = totals
        .iter()
        .map(|(state, (objects, bytes))| json!({"state":state,"objects":objects,"bytes":bytes}))
        .collect();
    Ok(
        json!({"schema":"git-asset.cache-archive-estimate-report.experimental-1",
        "objects":unique.len(),"groups":groups,"additional_logical_bytes_at_named_paths":totals.get("absent_at_named_path").map_or(0,|v|v.1),
        "volume_total_bytes":fs2::total_space(&req.archive.root).ok(),"volume_free_bytes":fs2::available_space(&req.archive.root).ok(),
        "payloads_read":0,"removal_authorized":false}),
    )
}

pub(crate) fn main(args: Vec<OsString>) -> ExitCode {
    match run(&args) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!(
                "{}",
                json!({"ok":false,"error":error,"removal_authorized":false})
            );
            ExitCode::from(2)
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn estimate_counts_exact_missing_bytes_once_without_reading_payload() {
        let root = tempfile::tempdir().unwrap();
        let objects = root.path().join("objects");
        fs::create_dir(&objects).unwrap();
        let listed = Object::from_bytes(b"named");
        let absent = Object::from_bytes(b"not in archive");
        // Matching length alone is deliberately not reported as verification.
        fs::write(objects.join(listed.sha256.as_str()), b"wrong").unwrap();
        let report = estimate(Estimate {
            schema: "git-asset.cache-archive-estimate.experimental-1".into(),
            archive: Place {
                root: root.path().to_owned(),
                layout: git_asset_store::existing::Layout::Flat,
            },
            objects: vec![listed, absent.clone(), absent.clone()],
        })
        .unwrap();
        assert_eq!(report["objects"], 2);
        assert_eq!(
            report["additional_logical_bytes_at_named_paths"],
            absent.bytes
        );
        assert_eq!(report["payloads_read"], 0);
        assert_eq!(report["removal_authorized"], false);
    }

    #[test]
    fn archive_plan_apply_status_and_exact_retry_preserve_the_source() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        let archive = root.path().join("archive");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&archive).unwrap();
        let object = Object::from_bytes(b"synthetic historical delivery");
        let source_file =
            git_asset_store::existing::Layout::Flat.object_path(&source, &object.sha256);
        fs::create_dir(source_file.parent().unwrap()).unwrap();
        fs::write(&source_file, b"synthetic historical delivery").unwrap();
        let request = root.path().join("request.json");
        fs::write(
            &request,
            serde_json::to_vec(&json!({"schema":SCHEMA,"object":object,
                "source":{"root":source,"layout":"flat"},"archive":{"root":archive,"layout":"reel-cas"},
                "minimum_free_bytes":0})).unwrap(),
        ).unwrap();
        let invoke = |op: &str, plan: Option<&str>| {
            let mut args = vec![op.into(), request.clone().into_os_string()];
            args.extend(plan.map(OsString::from));
            run(&args)
        };
        let preview = invoke("plan", None).unwrap();
        assert_eq!(preview["archive_copy_verified"], false);
        assert!(!archive.join("objects").exists());
        assert!(invoke("apply", Some("wrong")).is_err());
        assert!(!archive.join("objects").exists());
        let plan = preview["plan"].as_str().unwrap();
        assert_eq!(
            invoke("apply", Some(plan)).unwrap()["copied_bytes"],
            object.bytes
        );
        assert_eq!(
            invoke("status", None).unwrap()["archive_copy_verified"],
            true
        );
        assert_eq!(invoke("apply", Some(plan)).unwrap()["copied_bytes"], 0);
        assert_eq!(
            fs::read(&source_file).unwrap(),
            b"synthetic historical delivery"
        );
        let target =
            git_asset_store::existing::Layout::ReelCas.object_path(&archive, &object.sha256);
        fs::write(&target, b"unrelated destination content").unwrap();
        assert!(invoke("apply", Some(plan)).is_err());
        assert_eq!(fs::read(target).unwrap(), b"unrelated destination content");
    }
}
