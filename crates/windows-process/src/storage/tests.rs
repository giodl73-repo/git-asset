use super::*;
use std::os::windows::fs::OpenOptionsExt;
#[test]
fn cooperative_query_budget_refuses_invalid_and_expired_limits_without_io() {
    assert!(Budget::new(0).is_err());
    assert!(Budget::new(120001).is_err());
    let expired = Budget {
        start: Instant::now() - Duration::from_secs(1),
        limit: Duration::from_millis(1),
    };
    assert_eq!(expired.check().unwrap_err().kind(), io::ErrorKind::TimedOut);
}
fn device(bus: u32, serial: &[u8]) -> Vec<u8> {
    let mut data = vec![0; 40];
    data[..4].copy_from_slice(&40u32.to_le_bytes());
    data[24..28].copy_from_slice(&40u32.to_le_bytes());
    data[28..32].copy_from_slice(&bus.to_le_bytes());
    data.extend_from_slice(serial);
    data.push(0);
    let n = data.len() as u32;
    data[4..8].copy_from_slice(&n.to_le_bytes());
    data
}
#[test]
fn literal_windows_layout_extents_and_bounded_malformed_frames() {
    assert_eq!(offset_of!(VOLUME_DISK_EXTENTS, Extents), 8);
    assert_eq!(size_of::<DISK_EXTENT>(), 24);
    assert_eq!(
        offset_of!(STORAGE_DEVICE_DESCRIPTOR, SerialNumberOffset),
        24
    );
    assert_eq!(offset_of!(STORAGE_DEVICE_DESCRIPTOR, BusType), 28);
    let raw = [
        1, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0,
        0, 0,
    ];
    assert_eq!(extents(&raw).unwrap(), vec![7]);
    for n in 0..raw.len() {
        assert!(extents(&raw[..n]).is_err());
    }
    for count in [0u32, 17, u32::MAX] {
        let mut data = raw;
        data[..4].copy_from_slice(&count.to_le_bytes());
        assert!(extents(&data).is_err());
    }
    for (offset, value) in [(8, 65536u64), (16, u64::MAX), (24, 0)] {
        let mut data = raw;
        data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        assert!(extents(&data).is_err());
    }
    let mut overflow = raw;
    overflow[16..24].copy_from_slice(&i64::MAX.to_le_bytes());
    assert!(extents(&overflow).is_err());
}
#[test]
fn serials_are_normalized_hashed_and_opaque_or_truncated_devices_refuse() {
    let a = descriptor(&device(BusTypeSata as u32, b"  SYNTHETIC-1234  ")).unwrap();
    let b = descriptor(&device(BusTypeUsb as u32, b"synthetic-1234")).unwrap();
    assert_eq!(a.serial, b.serial);
    assert!(overlaps(&[a.serial], &[b.serial]));
    assert!(!overlaps(
        &[a.serial],
        &[descriptor(&device(BusTypeAta as u32, b"other-serial-1234"))
            .unwrap()
            .serial]
    ));
    for bus in [
        0,
        1,
        2,
        4,
        5,
        6,
        8,
        9,
        10,
        12,
        13,
        14,
        15,
        16,
        18,
        19,
        u32::MAX,
    ] {
        assert!(descriptor(&device(bus, b"synthetic-1234")).is_err());
    }
    for serial in [
        b"".as_slice(),
        b"00000000",
        b"short",
        b"synthetic\n1234",
        b"bad\xffserial",
    ] {
        assert!(descriptor(&device(BusTypeNvme as u32, serial)).is_err());
    }
    let data = device(BusTypeNvme as u32, b"synthetic-1234");
    for version in [0u32, 36, 39, 41, u32::MAX] {
        let mut bad = data.clone();
        bad[..4].copy_from_slice(&version.to_le_bytes());
        assert!(descriptor(&bad).is_err());
    }
    let mut oversized = data.clone();
    oversized[4..8].copy_from_slice(&4097u32.to_le_bytes());
    assert!(descriptor(&oversized).is_err());
    assert!(descriptor(&device(BusTypeNvme as u32, &[b'x'; 129])).is_err());
    for n in 0..data.len() {
        assert!(descriptor(&data[..n]).is_err());
    }
    for offset in [0u32, 20, u32::MAX] {
        let mut bad = data.clone();
        bad[24..28].copy_from_slice(&offset.to_le_bytes());
        assert!(descriptor(&bad).is_err());
    }
    let mut bad = data;
    bad[8] = 5;
    assert!(descriptor(&bad).is_err());
    assert!(overlaps(&[1u32, 3], &[2, 3]));
    assert!(!overlaps(&[1u32], &[2]));
}
#[test]
fn only_exact_volume_guid_roots_are_accepted() {
    let path = "\\\\?\\Volume{01234567-89AB-CDEF-0123-456789ABCDEF}\\owned\\directory";
    assert_eq!(
        root_name(path).unwrap(),
        "\\\\?\\volume{01234567-89ab-cdef-0123-456789abcdef}"
    );
    for bad in [
        "F:\\owned",
        "\\\\server\\share",
        "\\\\?\\Volume{}\\",
        "\\\\?\\Volume{01234567-89ab-cdef-0123-456789abcdef}",
        "\\\\?\\Volume{01234567-89ab-cdef-0123-456789abcdeg}\\",
    ] {
        assert!(root_name(bad).is_err());
    }
}
#[test]
fn actual_owned_directory_and_alias_never_become_separated_devices() {
    let root = tempfile::tempdir().unwrap();
    let child = root.path().join("child");
    std::fs::create_dir(&child).unwrap();
    let open = |p: &std::path::Path| {
        OpenOptions::new()
            .read(true)
            .custom_flags(0x02000000)
            .open(p)
            .unwrap()
    };
    let a = open(root.path());
    let b = open(&child);
    assert!(Separation::compare(&a, &b).unwrap().is_none());
    assert!(Separation::compare(&a, &a).unwrap().is_none());
}

#[test]
#[ignore = "explicit source/keeper volume roots; fresh owned directories and selected device queries only"]
fn actual_declared_distinct_volume_pair_is_revalidated_without_identifier_output() {
    let source = std::path::PathBuf::from(
        std::env::var_os("GIT_ASSET_TEST_SOURCE_VOLUME_ROOT").expect("source root"),
    );
    let keeper = std::path::PathBuf::from(
        std::env::var_os("GIT_ASSET_TEST_KEEPER_VOLUME_ROOT").expect("keeper root"),
    );
    assert!(source.is_absolute() && keeper.is_absolute());
    let source = tempfile::Builder::new()
        .prefix("git-asset-device-view-")
        .tempdir_in(source)
        .unwrap();
    let keeper = tempfile::Builder::new()
        .prefix("git-asset-device-view-")
        .tempdir_in(keeper)
        .unwrap();
    let open = |p: &std::path::Path| {
        OpenOptions::new()
            .read(true)
            .custom_flags(0x02000000)
            .open(p)
            .unwrap()
    };
    let a = open(source.path());
    let b = open(keeper.path());
    let pair = Separation::compare(&a, &b)
        .unwrap()
        .expect("declared pair must have disjoint direct-device views");
    let id = pair.fingerprints();
    assert_ne!(id[0], id[1]);
    assert!(pair.revalidate().unwrap());
    assert_eq!(pair.fingerprints(), id);
}

#[test]
fn counts_links_on_open_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("object");
    let file = File::create(&path).unwrap();
    assert_eq!(hard_links(&file).unwrap(), 1);
    std::fs::hard_link(&path, dir.path().join("alias")).unwrap();
    assert_eq!(hard_links(&file).unwrap(), 2);
}
