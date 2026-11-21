//! Bounded named Windows jobs, not asset authority or proof of byte custody.
//! Safety contract: specs/ANNEX-PROTECTED-EVICTION.md, native command fence entry.
#![cfg(windows)]
#![deny(unsafe_op_in_unsafe_fn)]
use std::{
    io,
    os::windows::io::{AsRawHandle, BorrowedHandle, FromRawHandle, OwnedHandle},
    ptr,
};
use windows_sys::Win32::System::SystemServices::{
    JOB_OBJECT_ASSIGN_PROCESS, JOB_OBJECT_QUERY, JOB_OBJECT_TERMINATE,
};
use windows_sys::Win32::{
    Foundation::{ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, GetLastError, SetLastError},
    System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation, OpenJobObjectW,
        QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
    },
};

fn name(scope: &str) -> io::Result<Vec<u16>> {
    if scope.len() != 64
        || !scope
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    Ok(format!("Global\\git-asset-native-v1-{scope}\0")
        .encode_utf16()
        .collect())
}

/// Owned, noninheritable named job. No raw handle or PID is a durable proof.
#[derive(Debug)]
pub struct Job(OwnedHandle);
impl Job {
    /// Explicit creation/join; access or foreign-limit failures have no fallback.
    pub fn join(scope: &str) -> io::Result<Self> {
        let name = name(scope)?;
        // SAFETY: bounded live nul-terminated UTF16; null attributes mean default
        // owner security and a noninheritable handle. Capture last error immediately.
        let (raw, error) = unsafe {
            SetLastError(0);
            let raw = CreateJobObjectW(ptr::null(), name.as_ptr());
            (raw, GetLastError())
        };
        if raw.is_null() {
            return Err(io::Error::from_raw_os_error(error as i32));
        }
        // SAFETY: successful CreateJobObjectW returns a fresh owned kernel handle.
        let job = Self(unsafe { OwnedHandle::from_raw_handle(raw) });
        if error != ERROR_ALREADY_EXISTS {
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            // SAFETY: valid owned job, initialized ABI-matching structure and exact size.
            let ok = unsafe {
                SetInformationJobObject(
                    raw,
                    JobObjectExtendedLimitInformation,
                    (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of_val(&info) as u32,
                )
            };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        job.check_limits()?;
        Ok(job)
    }

    /// Query/recovery does not create a new job. Absence alone grants no action.
    pub fn open(scope: &str) -> io::Result<Option<Self>> {
        let name = name(scope)?;
        // SAFETY: bounded live nul-terminated UTF16; returned handle is not inherited.
        let raw = unsafe {
            OpenJobObjectW(
                JOB_OBJECT_QUERY | JOB_OBJECT_ASSIGN_PROCESS | JOB_OBJECT_TERMINATE,
                0,
                name.as_ptr(),
            )
        };
        if raw.is_null() {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(ERROR_FILE_NOT_FOUND as i32) {
                Ok(None)
            } else {
                Err(error)
            };
        }
        // SAFETY: successful OpenJobObjectW returns one new owned handle.
        let job = Self(unsafe { OwnedHandle::from_raw_handle(raw) });
        job.check_limits()?;
        Ok(Some(job))
    }

    fn check_limits(&self) -> io::Result<()> {
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        // SAFETY: live owned job, initialized output of the requested ABI type/size.
        let ok = unsafe {
            QueryInformationJobObject(
                self.0.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&mut info as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of_val(&info) as u32,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        if info.BasicLimitInformation.LimitFlags != JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        Ok(())
    }

    /// Caller assigns before resuming a suspended command. This method does not suspend it.
    pub fn assign(&self, process: BorrowedHandle<'_>) -> io::Result<()> {
        self.check_limits()?;
        // SAFETY: both borrowed process and owned job handles remain live during call.
        // A wrong-kind borrowed handle is rejected by the kernel, never dereferenced here.
        if unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), process.as_raw_handle()) } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Includes descendants; zero is meaningful only under the caller's admission fences.
    pub fn active(&self) -> io::Result<u32> {
        self.check_limits()?;
        let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        // SAFETY: live owned job and initialized fixed-size requested output structure.
        let ok = unsafe {
            QueryInformationJobObject(
                self.0.as_raw_handle(),
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of_val(&info) as u32,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(info.ActiveProcesses)
    }

    /// Requests group termination; success does not mean all processes have exited.
    pub fn terminate(&self) -> io::Result<()> {
        self.check_limits()?;
        // SAFETY: valid owned job; scalar exit code, no borrowed memory.
        if unsafe { TerminateJobObject(self.0.as_raw_handle(), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
