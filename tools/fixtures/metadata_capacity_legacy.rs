//! Compile against pinned pre-capacity SDK, never against the candidate SDK.
use git_asset_core::tree::NodeStore;
use git_asset_store::metadata::{Checkpoint, Error, MetadataStore};
use std::{env, fs, path::Path, thread, time::{Duration, Instant}};
fn main() {
    let a: Vec<_> = env::args().collect();
    assert_eq!(a.len(), 5);
    let root = Path::new(&a[1]);
    let cp: Checkpoint = serde_json::from_slice(&fs::read(&a[2]).unwrap()).unwrap();
    let mut old = MetadataStore::open(root, &cp).unwrap();
    old.put(b"buffered by the pinned historical writer").unwrap();
    fs::write(&a[3], b"ready").unwrap();
    let start = Instant::now();
    while !Path::new(&a[4]).exists() {
        assert!(start.elapsed() < Duration::from_secs(45));
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(old.finish().unwrap_err(), Error::Corrupt);
    assert_eq!(MetadataStore::open(root, &cp).err(), Some(Error::Foreign));
    println!("cached_writer_fenced;old_profile_reopen_refused");
}
