//! What:
//!  One directory opened beneath a held one,
//!  with the four resolution refusals the wrapper's walk uses:
//!  no symbolic link,
//!  no `/proc` magic link,
//!  no mount crossing and nothing outside the held folder.
//! Why:
//!  The probe's refusal controls need the kernel's own errno for a `..` escape,
//!  which is `EXDEV`,
//!  and the wrapper's walker refuses `..` as a name before the kernel sees it,
//!  so this call is the control's own
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms").
//!  The flag bits are the wrapper's own four,
//!  repeated here because that module keeps them for its walker alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const opened = openat2(held, '../escape', { flags: O_DIRECTORY, resolve: NO_XDEV | NO_MAGICLINKS | NO_SYMLINKS | BENEATH });
//! ```

use super::probe_call_error::CallError;
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

/// `RESOLVE_NO_XDEV`:
/// no mount point is crossed.
pub const RESOLVE_NO_XDEV: u64 = 0x01;
/// `RESOLVE_NO_MAGICLINKS`:
/// no `/proc` style link is followed.
pub const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
/// `RESOLVE_NO_SYMLINKS`:
/// no symbolic link is followed.
pub const RESOLVE_NO_SYMLINKS: u64 = 0x04;
/// `RESOLVE_BENEATH`:
/// the lookup stays below the directory it starts from.
pub const RESOLVE_BENEATH: u64 = 0x08;

/// The arguments of `openat2`,
/// laid out as the kernel's `struct open_how`.
#[repr(C)]
struct OpenHow {
    /// The `open` flags.
    flags: u64,
    /// The mode of a created file.
    mode: u64,
    /// The `RESOLVE_*` bits.
    resolve: u64,
}

/// What:
///  Open one relative path beneath a held directory,
///  refusing every link,
///  every mount crossing and everything outside.
/// Why:
///  The probe's refusal controls assert the kernel's answer,
///  so the call is the control's own and the errno is the answer it asserts.
///
/// # Errors
/// The call refused,
///  with the errno the kernel answered.
pub fn open_directory_beneath(held: &std::fs::File, relative: &str) -> Result<OwnedFd, CallError> {
    let path: CString = CString::new(relative).expect("no NUL in a relative path");
    let how: OpenHow = OpenHow { flags: (libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC) as u64, mode: 0, resolve: RESOLVE_NO_XDEV | RESOLVE_NO_MAGICLINKS | RESOLVE_NO_SYMLINKS | RESOLVE_BENEATH };
    // SAFETY: the held descriptor is the caller's, the path and the structure outlive the call, and the answer is owned once.
    let opened: libc::c_int = unsafe {
        libc::syscall(libc::SYS_openat2, held.as_raw_fd(), path.as_ptr(), (&how as *const OpenHow).cast::<libc::c_void>(), std::mem::size_of::<OpenHow>()) as libc::c_int
    };
    if opened < 0 {
        return Err(CallError { call: "openat2", errno: std::io::Error::last_os_error().raw_os_error().unwrap_or(libc::EIO) });
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(opened) });
}

/// What:
///  One directory named by path as the handle a mount move can land on,
///  which is an `O_PATH` descriptor rather than an opened folder.
/// Why:
///  A mount placed onto a descriptor needs the kind of handle that names the
///  folder without opening it,
///  and the product's own walk holds its slot the same way.
///
/// # Errors
/// The open refused.
pub fn open_path_handle(path: &std::path::Path) -> Result<OwnedFd, CallError> {
    use std::os::unix::ffi::OsStrExt;
    let name: CString = CString::new(path.as_os_str().as_bytes()).expect("a path holds no NUL");
    // SAFETY: the path is this call's own and outlives it; the descriptor it answers is owned once.
    let opened: libc::c_int = unsafe { libc::open(name.as_ptr(), libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC) };
    if opened < 0 {
        return Err(CallError { call: "open O_PATH", errno: std::io::Error::last_os_error().raw_os_error().unwrap_or(libc::EIO) });
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(opened) });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_open_beneath_tests.rs"]
mod tests;
