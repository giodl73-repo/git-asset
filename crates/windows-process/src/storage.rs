//! Selected native directory device views, not custody or fault-domain authority.
//! No enumeration. Unknown backing is unsupported, never inferred independent.
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io,
    mem::{offset_of, size_of},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    ptr,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Storage::FileSystem::{
        BusTypeAta, BusTypeNvme, BusTypeSata, BusTypeUsb, GetFinalPathNameByHandleW,
        IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS, VOLUME_NAME_GUID,
    },
    System::{
        IO::DeviceIoControl,
        Ioctl::{
            DISK_EXTENT, IOCTL_STORAGE_QUERY_PROPERTY, STORAGE_DEVICE_DESCRIPTOR,
            VOLUME_DISK_EXTENTS,
        },
    },
};
const BUFFER: usize = 4096;
const MAX_DISKS: usize = 16;
struct Budget {
    start: Instant,
    limit: Duration,
}
impl Budget {
    fn new(millis: u64) -> io::Result<Self> {
        if !(1..=120000).contains(&millis) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Ok(Self {
            start: Instant::now(),
            limit: Duration::from_millis(millis),
        })
    }
    fn check(&self) -> io::Result<()> {
        if self.start.elapsed() >= self.limit {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
}
fn unsupported() -> io::Error {
    io::Error::from(io::ErrorKind::Unsupported)
}
fn hash(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    Sha256::new()
        .chain_update(domain)
        .chain_update(bytes)
        .finalize()
        .into()
}
/// Query a named regular file handle. Reparse points are outside this view.
pub fn hard_links(file: &File) -> io::Result<u64> {
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
        GetFileInformationByHandle,
    };
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: the borrowed file remains open and info is valid writable storage.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if info.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
        || info.nNumberOfLinks == 0
    {
        return Err(unsupported());
    }
    Ok(u64::from(info.nNumberOfLinks))
}

fn volume_name(directory: &File) -> io::Result<String> {
    let mut buffer = [0u16; 1024];
    // SAFETY: live borrowed file handle and initialized output buffer with exact u16 count.
    let n = unsafe {
        GetFinalPathNameByHandleW(
            directory.as_raw_handle(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            VOLUME_NAME_GUID,
        )
    } as usize;
    if n == 0 {
        return Err(io::Error::last_os_error());
    }
    if n >= buffer.len() {
        return Err(unsupported());
    }
    root_name(&String::from_utf16(&buffer[..n]).map_err(|_| unsupported())?)
}
fn root_name(path: &str) -> io::Result<String> {
    let prefix = "\\\\?\\volume{";
    let path = path.to_ascii_lowercase();
    let bytes = path.as_bytes();
    let end = prefix.len() + 36;
    if !path.starts_with(prefix) || bytes.get(end..end + 2) != Some(b"}\\") {
        return Err(unsupported());
    }
    for (i, b) in bytes[prefix.len()..end].iter().enumerate() {
        if if [8, 13, 18, 23].contains(&i) {
            *b != b'-'
        } else {
            !b.is_ascii_hexdigit()
        } {
            return Err(unsupported());
        }
    }
    Ok(path[..end + 1].into())
}
fn open_device(path: &str) -> io::Result<File> {
    OpenOptions::new().access_mode(0).share_mode(7).open(path)
}
fn ioctl(file: &File, code: u32, input: &[u8], budget: &Budget) -> io::Result<Vec<u8>> {
    budget.check()?;
    let mut output = [0u8; BUFFER];
    let mut returned = 0;
    // SAFETY: live owned handle; initialized bounded input/output slices and byte count.
    // The device handle is synchronous: null OVERLAPPED, so buffers outlive the operation.
    let ok = unsafe {
        DeviceIoControl(
            file.as_raw_handle(),
            code,
            if input.is_empty() {
                ptr::null()
            } else {
                input.as_ptr().cast()
            },
            input.len() as u32,
            output.as_mut_ptr().cast(),
            output.len() as u32,
            &mut returned,
            ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    budget.check()?;
    let data = output.get(..returned as usize).ok_or_else(unsupported)?;
    Ok(data.to_vec())
}
fn u32_at(bytes: &[u8], offset: usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(unsupported)?
            .try_into()
            .map_err(|_| unsupported())?,
    ))
}
fn i64_at(bytes: &[u8], offset: usize) -> io::Result<i64> {
    Ok(i64::from_le_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or_else(unsupported)?
            .try_into()
            .map_err(|_| unsupported())?,
    ))
}
fn extents(data: &[u8]) -> io::Result<Vec<u32>> {
    let n = u32_at(data, 0)? as usize;
    if !(1..=MAX_DISKS).contains(&n) {
        return Err(unsupported());
    }
    let base = offset_of!(VOLUME_DISK_EXTENTS, Extents);
    if base + n * size_of::<DISK_EXTENT>() > data.len() {
        return Err(unsupported());
    }
    let mut disks = Vec::new();
    for i in 0..n {
        let at = base + i * size_of::<DISK_EXTENT>();
        let disk = u32_at(data, at + offset_of!(DISK_EXTENT, DiskNumber))?;
        let start = i64_at(data, at + offset_of!(DISK_EXTENT, StartingOffset))?;
        let length = i64_at(data, at + offset_of!(DISK_EXTENT, ExtentLength))?;
        if disk > 65535 || start < 0 || length <= 0 || start.checked_add(length).is_none() {
            return Err(unsupported());
        }
        disks.push(disk);
    }
    disks.sort_unstable();
    disks.dedup();
    Ok(disks)
}
#[derive(Clone, PartialEq, Eq)]
struct Device {
    bus: u32,
    serial: [u8; 32],
}
fn descriptor(data: &[u8]) -> io::Result<Device> {
    let minimum = offset_of!(STORAGE_DEVICE_DESCRIPTOR, RawDeviceProperties);
    let version = u32_at(data, 0)? as usize;
    let length = u32_at(data, 4)? as usize;
    if version != size_of::<STORAGE_DEVICE_DESCRIPTOR>()
        || version > length
        || length > BUFFER
        || length > data.len()
        || data.get(8) != Some(&0)
    {
        return Err(unsupported());
    }
    let bus = u32_at(data, offset_of!(STORAGE_DEVICE_DESCRIPTOR, BusType))?;
    if ![BusTypeAta, BusTypeSata, BusTypeNvme, BusTypeUsb]
        .into_iter()
        .any(|b| b as u32 == bus)
    {
        return Err(unsupported());
    }
    let offset = u32_at(
        data,
        offset_of!(STORAGE_DEVICE_DESCRIPTOR, SerialNumberOffset),
    )? as usize;
    if offset < minimum || offset >= length {
        return Err(unsupported());
    }
    let tail = &data[offset..length];
    let end = tail.iter().position(|b| *b == 0).ok_or_else(unsupported)?;
    if end > 128 {
        return Err(unsupported());
    }
    let serial = tail[..end].trim_ascii();
    if serial.len() < 8
        || !serial.iter().all(|b| b.is_ascii_graphic() || *b == b' ')
        || serial.iter().all(|b| *b == serial[0])
    {
        return Err(unsupported());
    }
    Ok(Device {
        bus,
        serial: hash(
            b"git-asset.windows-device-serial.v1\0",
            &serial.to_ascii_lowercase(),
        ),
    })
}
fn overlaps<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    a.iter().any(|x| b.contains(x))
}
struct Volume {
    id: [u8; 32],
    handle: File,
    numbers: Vec<u32>,
    devices: Vec<(File, Device)>,
}
impl Volume {
    fn open(name: &str, budget: &Budget) -> io::Result<Self> {
        budget.check()?;
        let handle = open_device(name)?;
        let numbers = extents(&ioctl(
            &handle,
            IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
            &[],
            budget,
        )?)?;
        Ok(Self {
            id: hash(b"git-asset.windows-volume.v1\0", name.as_bytes()),
            handle,
            numbers,
            devices: Vec::new(),
        })
    }
    fn describe(&mut self, budget: &Budget) -> io::Result<()> {
        for number in &self.numbers {
            budget.check()?;
            let handle = open_device(&format!("\\\\.\\PhysicalDrive{number}"))?;
            // StorageDeviceProperty=0, PropertyStandardQuery=0, zero additional parameter/padding.
            let device = descriptor(&ioctl(
                &handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                &[0; 12],
                budget,
            )?)?;
            if self
                .devices
                .iter()
                .any(|(_, prior)| prior.serial == device.serial)
            {
                return Err(unsupported());
            }
            self.devices.push((handle, device));
        }
        Ok(())
    }
    fn check(&self, budget: &Budget) -> io::Result<bool> {
        if extents(&ioctl(
            &self.handle,
            IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
            &[],
            budget,
        )?)? != self.numbers
        {
            return Ok(false);
        }
        for (handle, expected) in &self.devices {
            if &descriptor(&ioctl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                &[0; 12],
                budget,
            )?)? != expected
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn fingerprint(&self) -> [u8; 32] {
        let mut identities: Vec<_> = self
            .devices
            .iter()
            .map(|(_, d)| (d.serial, d.bus))
            .collect();
        identities.sort_unstable();
        let mut hash = Sha256::new()
            .chain_update(b"git-asset.windows-device-view.v1\0")
            .chain_update(self.id);
        for (serial, bus) in identities {
            hash.update(serial);
            hash.update(bus.to_le_bytes());
        }
        hash.finalize().into()
    }
}
/// Live separated device views. Owner admission of direct backing/fault domains and
/// retained byte guards remain necessary. No removal permission is granted here.
pub struct Separation {
    source: Volume,
    keeper: Volume,
    millis: u64,
}
impl Separation {
    /// Bounded selected-handle queries; driver calls are synchronous, not a hard IO deadline.
    pub fn compare(source: &File, keeper: &File) -> io::Result<Option<Self>> {
        Self::compare_with_limit(source, keeper, 10000)
    }
    /// Cooperative checks surround selected queries; a blocked driver call is not preempted.
    pub fn compare_with_limit(
        source: &File,
        keeper: &File,
        millis: u64,
    ) -> io::Result<Option<Self>> {
        let budget = Budget::new(millis)?;
        let source_name = volume_name(source)?;
        budget.check()?;
        let keeper_name = volume_name(keeper)?;
        budget.check()?;
        if source_name == keeper_name {
            return Ok(None);
        }
        let mut source = Volume::open(&source_name, &budget)?;
        let mut keeper = Volume::open(&keeper_name, &budget)?;
        if overlaps(&source.numbers, &keeper.numbers) {
            return Ok(None);
        }
        source.describe(&budget)?;
        keeper.describe(&budget)?;
        let source_serials: Vec<_> = source.devices.iter().map(|(_, d)| d.serial).collect();
        let keeper_serials: Vec<_> = keeper.devices.iter().map(|(_, d)| d.serial).collect();
        if overlaps(&source_serials, &keeper_serials) {
            return Ok(None);
        }
        budget.check()?;
        Ok(Some(Self {
            source,
            keeper,
            millis,
        }))
    }
    pub fn fingerprints(&self) -> [[u8; 32]; 2] {
        [self.source.fingerprint(), self.keeper.fingerprint()]
    }
    pub fn revalidate(&self) -> io::Result<bool> {
        let budget = Budget::new(self.millis)?;
        Ok(self.source.check(&budget)? && self.keeper.check(&budget)?)
    }
}
#[cfg(test)]
mod tests;
