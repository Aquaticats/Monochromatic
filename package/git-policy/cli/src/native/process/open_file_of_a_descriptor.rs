//! What:
//!  The device number and the inode of the file an open descriptor of a process refers to.
//! Why:
//!  A lock is matched to the descriptor that holds it by the device number and the inode the file has,
//!  and the descriptor's own record is only the cheap test that comes first.
//!  Reading those attributes through the path `/proc/<pid>/fd/<n>` walks that path,
//!  and the last step of the walk asks the file system to revalidate its own directory entry before any attribute is read.
//!  A file system served from user space or over the network that stops answering leaves that walk in uninterruptible
//!  sleep for as long as it takes:
//!  measured on this host on 2026-11-09,
//!  a `statx` of a `/proc/<pid>/fd` link stayed asleep for the whole of a 25-minute window with the FUSE server stopped,
//!  and `SIGTERM` from `timeout(1)` did not end it.
//!  A descriptor names the file without any of that:
//!  `pidfd_getfd(2)` copies the open file's descriptor into this process without walking a path,
//!  and `statx(2)` on the copy with `AT_EMPTY_PATH` and `AT_STATX_DONT_SYNC` asks for the attributes the kernel already
//!  holds.
//!  FUSE answers that request from its cached inode without sending anything to the server
//!  (`fs/fuse/dir.c`,
//!  `fuse_update_get_attr`:
//!  `AT_STATX_DONT_SYNC` sets `sync = false` and the attributes come from `generic_fillattr`),
//!  and NFS answers without a round trip
//!  (`fs/nfs/inode.c`,
//!  `nfs_getattr`:
//!  `(query_flags & AT_STATX_DONT_SYNC) && !force_sync` jumps to `out_no_revalidate`).
//!  The measurements and the sources are written up in `doc/troubleshooting/fuse-descriptor-attributes.md`.
//!  A descriptor that cannot be copied says so and is never guessed at from the link's text:
//!  `readlink` of a `/proc/<pid>/fd` entry names a path that a bind mount,
//!  another mount namespace or a rename makes ambiguous.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const opened = openedFileOf({ process: '/proc/1234', fd: '3' }); // { device, inode }
//! ```

/// `OsStr` is a borrowed file name.
use std::ffi::OsStr;
/// The descriptor types a copied descriptor is returned and closed as.
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The device number and the inode number of a file a descriptor holds open.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenedFile {
    /// The device number of the file system that holds the file (`st_dev`).
    pub device: u64,
    /// The inode number of the file on that device.
    pub inode: u64,
}

/// What:
///  The process number a process directory of a process table names.
/// Why:
///   `pidfd_open(2)` takes the number, not the directory,
///  and a table that holds other files (`uptime`) or a number that is not one is refused rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(basename(process)); // NaN when it is not a number
/// ```
#[cfg(target_os = "linux")]
fn pid_of(process: &Path) -> std::io::Result<libc::pid_t> {
    let name: &str = match process.file_name().and_then(OsStr::to_str) {
        Some(found) => found,
        None => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "the process directory has no name")),
    };
    let number: i64 = match name.parse::<i64>() {
        Ok(found) => found,
        Err(_) => {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("the process directory {name:?} is not a process number")));
        }
    };
    return match libc::pid_t::try_from(number) {
        Ok(fits) => Ok(fits),
        Err(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("process {number} is not one this kernel numbers"))),
    };
}

/// What:
///  The descriptor number a name inside a descriptor directory gives.
/// Why:
///   `pidfd_getfd(2)` takes the number,
///  and a name that is not one is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(fd); // NaN when it is not a number
/// ```
#[cfg(target_os = "linux")]
fn descriptor_number(fd: &OsStr) -> std::io::Result<libc::c_int> {
    let name: &str = match fd.to_str() {
        Some(found) => found,
        None => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "the descriptor name is not text")),
    };
    return match name.parse::<i32>() {
        Ok(number) => Ok(number),
        Err(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("the descriptor name {name:?} is not a number"))),
    };
}

/// What:
///  A call's descriptor or zero, with the failure as an error.
/// Why:
///   Every call here returns a descriptor or `-1`,
///  which is the same shape as the helper the subvolume code uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (opened < 0) throw new Error(os.strerror(errno));
/// ```
#[cfg(target_os = "linux")]
fn checked_descriptor(opened: libc::c_long) -> std::io::Result<RawFd> {
    let number: libc::c_int = match libc::c_int::try_from(opened) {
        Ok(found) => found,
        Err(_) => return Err(std::io::Error::from_raw_os_error(libc::EMFILE)),
    };
    if number < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(number);
}

/// What:
///  A process descriptor of `pid`, owned by this process.
/// Why:
///   `pidfd_getfd(2)` needs one,
///  and the call itself reads no file system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pidfdOpen({ pid });
/// ```
#[cfg(target_os = "linux")]
fn pidfd_of(pid: libc::pid_t) -> std::io::Result<OwnedFd> {
    // SAFETY: `pidfd_open` reads its arguments only and, on success, returns a descriptor this process owns.
    let opened: std::io::Result<RawFd> = checked_descriptor(unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0 as libc::c_uint) });
    // SAFETY: a descriptor the call just created is owned by this process and by nothing else.
    return Ok(unsafe { OwnedFd::from_raw_fd(opened?) });
}

/// What:
///  A copy of the descriptor `number` that the process whose descriptor is `pidfd` holds,
///  owned by this process.
/// Why:
///   The copy names the same open file without walking any path,
///  so no file system is asked to revalidate anything.
///  The kernel allows it for a process this one may trace (the same account, and not one that hid itself with
///  `PR_SET_DUMPABLE`), which is the same permission the scan of `/proc/<pid>/fd` needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pidfdGetfd({ pidfd, fd: number });
/// ```
#[cfg(target_os = "linux")]
fn copied_descriptor_of(pidfd: RawFd, number: libc::c_int) -> std::io::Result<OwnedFd> {
    // SAFETY: `pidfd_getfd` reads its arguments only and, on success, returns a descriptor this process owns.
    let copied: std::io::Result<RawFd> = checked_descriptor(unsafe { libc::syscall(libc::SYS_pidfd_getfd, pidfd, number, 0 as libc::c_uint) });
    // SAFETY: a descriptor the call just created is owned by this process and by nothing else.
    return Ok(unsafe { OwnedFd::from_raw_fd(copied?) });
}

/// What:
///  The device and inode of the file the descriptor names,
///  from what the kernel already holds.
/// Why:
///   `AT_EMPTY_PATH` names the descriptor itself and `AT_STATX_DONT_SYNC` forbids a round trip,
///  which is what keeps a stopped FUSE server or an unreachable NFS server out of the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statx({ fd, flags: [AT_EMPTY_PATH, AT_STATX_DONT_SYNC] });
/// ```
#[cfg(target_os = "linux")]
fn attributes_in_memory(descriptor: RawFd) -> std::io::Result<OpenedFile> {
    // SAFETY: an all-zero `statx` is a valid value for the call to fill.
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    let empty: [u8; 1] = [0];
    // SAFETY: the empty name outlives the call, `found` is a live `statx`, and `descriptor` is open for the length of
    // the call.
    let answer: libc::c_int = unsafe {
        libc::statx(
            descriptor,
            empty.as_ptr().cast(),
            libc::AT_EMPTY_PATH | libc::AT_STATX_DONT_SYNC,
            libc::STATX_BASIC_STATS,
            &mut found,
        )
    };
    if answer < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(OpenedFile { device: libc::makedev(found.stx_dev_major, found.stx_dev_minor), inode: found.stx_ino });
}

/// What:
///  The device and inode of the file the descriptor `fd` of the process whose table directory is `process` holds open.
/// Why:
///   The one way a holder of a file is identified here,
///  and the reason no file system is asked to answer anything it must fetch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openedFileOf({ process: '/proc/1234', fd: '3' });
/// ```
#[cfg(target_os = "linux")]
pub fn opened_file_of(process: &Path, fd: &OsStr) -> std::io::Result<OpenedFile> {
    let pid: libc::pid_t = pid_of(process)?;
    let number: libc::c_int = descriptor_number(fd)?;
    let pidfd: OwnedFd = pidfd_of(pid)?;
    let copied: OwnedFd = copied_descriptor_of(pidfd.as_raw_fd(), number)?;
    return attributes_in_memory(copied.as_raw_fd());
}

/// What:
///  The same on a platform that has no `pidfd_getfd(2)`.
/// Why:
///   Nothing is guessed at where the kernel cannot copy a descriptor,
///  and a caller that needs the answer treats this as a process it could not look into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error('no descriptor copy on this platform');
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn opened_file_of(process: &Path, fd: &OsStr) -> std::io::Result<OpenedFile> {
    let _ = (process, fd);
    return Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "this platform has no `pidfd_getfd`, so a descriptor's file is not read without walking its path",
    ));
}

/// Controls on the descriptor reads stay out of the release executable.
#[cfg(test)]
#[path = "open_file_of_a_descriptor_tests.rs"]
mod tests;
