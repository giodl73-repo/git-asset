//! Operator-requested metadata walk of one enrolled local root.
use super::*;
use crate::restore::ordinary;
use fs2::FileExt;
use std::{
    fs,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    time::{Duration, Instant},
};

const MAX_ENTRIES: u64 = 2_000_000;
const MAX_ELAPSED: Duration = Duration::from_secs(900);

#[derive(Default, Serialize)]
struct Counts {
    files: u64,
    logical_path_bytes: u64,
    unique_files: u64,
    unique_allocated_bytes: u64,
    linked_paths: u64,
    excluded_or_unreadable: u64,
}

pub fn run(config: Config, project: &str, root: &Path) -> Result<Value> {
    let project = config
        .projects
        .iter()
        .find(|entry| entry.id == project)
        .ok_or("Project not enrolled")?;
    let profile: ResolverProfile = read(&project.profile, 65536)?;
    let places = profile.byte_source.ok_or("No cache locations")?.places;
    let storage = project.storage.as_ref().ok_or("No storage admission")?;
    if !storage.places.iter().any(|entry| {
        entry.live_probe
            && places
                .get(entry.place_index)
                .is_some_and(|place| place.root == root)
    }) {
        return Err("Root is not enrolled for live observation".into());
    }
    if !root.is_absolute()
        || root.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err("Absolute ordinary root required".into());
    }
    for path in root.ancestors() {
        if !ordinary(path)?.is_dir() {
            return Err("Non-directory cache ancestor".into());
        }
    }

    let started = Instant::now();
    let free_before = fs2::available_space(root).ok();
    let mut queue = vec![(root.to_owned(), String::new())];
    let mut identities = BTreeSet::new();
    let mut total = Counts::default();
    let mut areas = BTreeMap::<String, Counts>::new();
    let mut entries = 0;
    let mut budget_exhausted = false;
    'walk: while let Some((directory, area)) = queue.pop() {
        if !ordinary(&directory).is_ok_and(|metadata| metadata.is_dir()) {
            total.excluded_or_unreadable += 1;
            continue;
        }
        let Ok(iter) = fs::read_dir(&directory) else {
            total.excluded_or_unreadable += 1;
            continue;
        };
        for entry in iter {
            if entries == MAX_ENTRIES || started.elapsed() >= MAX_ELAPSED {
                budget_exhausted = true;
                break 'walk;
            }
            entries += 1;
            let Ok(entry) = entry else {
                total.excluded_or_unreadable += 1;
                continue;
            };
            let name = if area.is_empty() {
                entry.file_name().to_string_lossy().into_owned()
            } else {
                area.clone()
            };
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                total.excluded_or_unreadable += 1;
                continue;
            };
            if metadata.file_attributes() & 0x400 != 0 {
                total.excluded_or_unreadable += 1;
                continue;
            }
            if metadata.is_dir() {
                queue.push((path, name));
                continue;
            }
            let area_key = if directory == root {
                "(root files)".to_owned()
            } else if areas.len() < 128 || areas.contains_key(&name) {
                name
            } else {
                "(other directories)".to_owned()
            };
            let bucket = areas.entry(area_key).or_default();
            for count in [&mut total, &mut *bucket] {
                count.files += 1;
                count.logical_path_bytes = count
                    .logical_path_bytes
                    .checked_add(metadata.len())
                    .ok_or("Logical size overflow")?;
            }
            let physical = fs::OpenOptions::new()
                .access_mode(0x80)
                .share_mode(7)
                .custom_flags(0x0020_0000)
                .open(&path)
                .and_then(|file| {
                    let (volume, index, links) =
                        git_asset_windows_process::storage::file_identity(&file)?;
                    Ok(((volume, index), links, file.allocated_size()?))
                });
            let Ok((identity, links, allocation)) = physical else {
                total.excluded_or_unreadable += 1;
                bucket.excluded_or_unreadable += 1;
                continue;
            };
            let unique = links == 1 || identities.insert(identity);
            for count in [&mut total, &mut *bucket] {
                count.linked_paths += u64::from(links > 1);
                if unique {
                    count.unique_files += 1;
                    count.unique_allocated_bytes = count
                        .unique_allocated_bytes
                        .checked_add(allocation)
                        .ok_or("Allocation overflow")?;
                }
            }
        }
    }
    Ok(
        json!({"schema":"git-asset.cache-inventory.experimental-1","project":project.id,"cache_root":root,
        "entries_examined":entries,"budget_exhausted":budget_exhausted,
        "all_entries_accounted_for":!budget_exhausted && total.excluded_or_unreadable==0,
        "elapsed_seconds":started.elapsed().as_secs(),"counts":total,"areas":areas,
        "volume_total_bytes":fs2::total_space(root).ok(),"volume_free_before":free_before,
        "volume_free_after":fs2::available_space(root).ok(),"payloads_read":0,
        "removal_authorized":false,"limitations":"Live metadata walk; concurrent changes can affect counts. Hard-linked allocation is credited to the first path seen. Physical inventory does not establish current use, custody or safe reclaim."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrolled_walk_counts_two_names_as_one_allocation() {
        let test = tempfile::tempdir().unwrap();
        let root = test.path().join("store");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("original"), b"synthetic allocated bytes").unwrap();
        fs::hard_link(root.join("original"), root.join("alias")).unwrap();
        let profile = test.path().join("profile.json");
        fs::write(&profile, serde_json::to_vec(&json!({
            "schema":"tig.asset-resolver-profile.experimental-1",
            "git_executable":"unused","repository":"unused","commit":"unused",
            "records":[],"definitions":[],
            "byte_source":{"places":[{"root":root,"layout":"flat"}],"required":"cooperative-immutable"}
        })).unwrap()).unwrap();
        let config: Config = serde_json::from_value(json!({
            "schema":"tig.catalogue-enrollment.experimental-1",
            "projects":[{"id":"sample","label":"Synthetic store","profile":profile,
                "storage":{"schema":"tig.catalogue-storage.experimental-1","encoding":"json","max_decoded_bytes":1024,
                    "places":[{"label":"Working","place_index":0,"live_probe":true}]}}]
        })).unwrap();
        let measured = run(config, "sample", &root).unwrap();
        assert_eq!(measured["all_entries_accounted_for"], true);
        assert_eq!(measured["counts"]["files"], 2);
        assert_eq!(measured["counts"]["linked_paths"], 2);
        assert_eq!(measured["counts"]["unique_files"], 1);
        assert_eq!(measured["payloads_read"], 0);
        assert_eq!(measured["removal_authorized"], false);
    }
}
