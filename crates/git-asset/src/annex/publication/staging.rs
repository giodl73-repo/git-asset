//! One exact operation scope; no global scan, cleanup or payload cache.
use super::*;
use sha2::{Digest as _, Sha256};
use std::{
    fs::OpenOptions,
    io::{SeekFrom, Write},
    path::Path,
};

pub(super) fn path(profile: &Profile, p: &Plan) -> Result<PathBuf> {
    Ok(profile.staging.join(p.scope()?.as_str()).join(format!(
        "SHA256-s{}--{}",
        p.candidate().object.bytes,
        p.candidate().object.sha256.as_str()
    )))
}
fn regular(path: &Path) -> Result<()> {
    let m = fs::symlink_metadata(path)?;
    if !m.is_file() || reparse(&m) {
        return Err(Error::Denied);
    }
    Ok(())
}
fn reparse(m: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        m.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        m.file_type().is_symlink()
    }
}
pub(super) fn footprint(profile: &Profile, p: &Plan) -> Result<u64> {
    let path = path(profile, p)?;
    if let Ok(m) = fs::symlink_metadata(path.parent().ok_or(Error::Invalid)?)
        && (!m.is_dir() || reparse(&m))
    {
        return Err(Error::Denied);
    }
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e.into()),
        Ok(_) => {
            regular(&path)?;
            Ok(fs2::FileExt::allocated_size(&File::open(path)?)?)
        }
    }
}
pub(super) fn prepare<R: Read + Seek>(
    profile: &Profile,
    p: &Plan,
    work: &mut Work,
    source: &mut impl FnMut() -> Result<R>,
    hook: &mut impl FnMut(Stage) -> Result<()>,
) -> Result<PathBuf> {
    let path = path(profile, p)?;
    let parent = path.parent().ok_or(Error::Invalid)?;
    match fs::create_dir(parent) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => return Err(e.into()),
    }
    let m = fs::symlink_metadata(parent)?;
    if !m.is_dir() || reparse(&m) {
        return Err(Error::Denied);
    }
    for e in fs::read_dir(parent)?.take(2) {
        if e?.path() != path {
            return Err(Error::NeedsRecovery);
        }
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0).custom_flags(0x00200000);
    }
    let mut file = match fs::symlink_metadata(&path) {
        Ok(_) => {
            regular(&path)?;
            options.open(&path)?
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            options.create_new(true).open(&path)?
        }
        Err(e) => return Err(e.into()),
    };
    if reparse(&file.metadata()?) {
        return Err(Error::Denied);
    }
    let object = &p.candidate().object;
    let prefix = file.metadata()?.len();
    if prefix > object.bytes {
        return Err(Error::Integrity);
    }
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut input = if prefix < object.bytes {
        let mut r = source()?;
        r.seek(SeekFrom::Start(0))?;
        Some(r)
    } else {
        None
    };
    let mut checked = 0;
    while checked < prefix {
        let n = (prefix - checked).min(buffer.len() as u64) as usize;
        work.consume(n as u64)?;
        file.read_exact(&mut buffer[..n])?;
        if let Some(input) = input.as_mut() {
            let mut comparison = [0u8; 65536];
            work.consume(n as u64)?;
            input.read_exact(&mut comparison[..n])?;
            if comparison[..n] != buffer[..n] {
                return Err(Error::Integrity);
            }
        }
        hash.update(&buffer[..n]);
        checked += n as u64;
    }
    if let Some(mut input) = input {
        // Stream under the admitted peak; preallocation that extends EOF would
        // make a crash indistinguishable from a complete zero-filled suffix.
        file.seek(SeekFrom::Start(prefix))?;
        while checked < object.bytes {
            work.check()?;
            let n = (object.bytes - checked).min(buffer.len() as u64) as usize;
            let n = input.read(&mut buffer[..n])?;
            if n == 0 {
                return Err(Error::Integrity);
            }
            work.consume((n as u64).checked_mul(2).ok_or(Error::Budget)?)?;
            file.write_all(&buffer[..n])?;
            hash.update(&buffer[..n]);
            checked += n as u64;
            hook(Stage::StagingChunk)?;
        }
        let mut extra = [0u8; 1];
        work.consume(1)?;
        if input.read(&mut extra)? != 0 {
            return Err(Error::Integrity);
        }
    }
    if Digest::from_sha256(hash.finalize().into()) != object.sha256 {
        return Err(Error::Integrity);
    }
    file.sync_all()?;
    drop(file);
    Ok(path)
}
