//! What:
//!  The descriptor-relative system calls the folder of saved copies is read and removed through:
//!  open a name inside an open directory without following a link,
//!  ask what a name is,
//!  read a directory's names,
//!  and remove one name.
//! Why:
//!  A path string is resolved again by every call that takes it,
//! so a link or a rename between the listing and the removal would send a removal
//!  somewhere the pass never judged.
//! Every call here acts in the directory whose descriptor it is given,
//! which is the descriptor the folder was opened on (`discarded_store.rs`).
//! A name that is gone is answered as gone and never as a failure,
//! because another pass removing it first is the aim reached.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fd = openInside(parentFd, 'name'); // never follows a link at that name
//! ```
/// The identity a `stat` describes.
use crate::discarded_store_records::Identity;
/// `CString` is a NUL-terminated name,
/// and `OsStr`/`OsString` are borrowed/owned names in bytes.
use std::ffi::{
    CString,
    OsStr,
    OsString,
};
/// `OwnedFd` closes a descriptor when dropped;
/// `RawFd` is the plain number a call takes.
use std::os::fd::{
    FromRawFd,
    OwnedFd,
    RawFd,
};
/// `OsStrExt`/`OsStringExt` read and build a name's raw bytes.
use std::os::unix::ffi::{
    OsStrExt,
    OsStringExt,
};
/// The name as the C string the system calls take.
pub(crate) fn c_name(name: &OsStr) -> std::io::Result<CString> {
    return match CString::new(name.as_bytes()) {
        Ok(text) => Ok(text),
        Err(_) => Err(std::io::Error::from_raw_os_error(libc::EINVAL)),
    };
}

/// Takes ownership of a descriptor a system call returned,
/// or the error it left.
pub(crate) fn owned(descriptor: libc::c_int) -> std::io::Result<OwnedFd> {
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the call returned a new descriptor that nothing else owns.
    return Ok(unsafe { OwnedFd::from_raw_fd(descriptor) });
}

/// Opens the directory `name` inside `parent` for reading,
/// never following a link at `name`.
pub(crate) fn open_inside(
    parent: RawFd,
    name: &OsStr,
) -> std::io::Result<OwnedFd> {
    let text: CString = c_name(name)?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `parent` is an open descriptor.
    return owned(unsafe {
        libc::openat(
            parent,
            text.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    });
}

/// What `fstatat` says of `name` inside `parent`,
/// without following a link at `name`.
pub(crate) fn status_of(
    parent: RawFd,
    name: &OsStr,
) -> std::io::Result<libc::stat> {
    let text: CString = c_name(name)?;
    // SAFETY: `stat` is plain data for which all zero bytes is a valid value.
    let mut status: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `status` is valid for one `stat`.
    let result: libc::c_int = unsafe {
        libc::fstatat(
            parent,
            text.as_ptr(),
            &raw mut status,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(status);
}

/// The identity a `stat` describes.
pub(crate) fn identity_in(status: &libc::stat) -> Identity {
    return Identity {
        device: u64::from(status.st_dev),
        inode: u64::from(status.st_ino),
        changed_seconds: i64::from(status.st_ctime),
        changed_nanos: i64::from(status.st_ctime_nsec),
    };
}

/// Whether a `stat` describes a directory.
pub(crate) fn is_directory(status: &libc::stat) -> bool {
    return status.st_mode & libc::S_IFMT == libc::S_IFDIR;
}

/// Sets the thread's error number to zero,
/// so that the end of a directory can be told from a failed read.
fn clear_error_number() {
    // SAFETY: the location is the calling thread's own error number, valid for a write.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    unsafe {
        *libc::__errno_location() = 0;
    }
    // SAFETY: the location is the calling thread's own error number, valid for a write.
    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "ios"))]
    unsafe {
        *libc::__error() = 0;
    }
}

/// The names inside the directory open on `descriptor`,
/// `.` and `..` left out.
pub(crate) fn names_inside(descriptor: RawFd) -> std::io::Result<Vec<OsString>> {
    // SAFETY: `descriptor` is open; the duplicate is owned by the stream `fdopendir` makes of it.
    let duplicate: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_DUPFD_CLOEXEC, 0) };
    if duplicate < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `duplicate` is a directory descriptor nothing else uses.
    let stream: *mut libc::DIR = unsafe { libc::fdopendir(duplicate) };
    if stream.is_null() {
        let error: std::io::Error = std::io::Error::last_os_error();
        // SAFETY: `fdopendir` failed, so `duplicate` is still ours to close.
        unsafe { libc::close(duplicate) };
        return Err(error);
    }
    // The duplicate shares its position with the descriptor, so an earlier read would leave this one at the end.
    // SAFETY: `stream` is an open directory stream used only by this thread.
    unsafe { libc::rewinddir(stream) };
    let mut names: Vec<OsString> = Vec::new();
    let mut failure: Option<std::io::Error> = None;
    loop {
        clear_error_number();
        // SAFETY: `stream` is an open directory stream used only by this thread.
        let entry: *mut libc::dirent = unsafe { libc::readdir(stream) };
        if entry.is_null() {
            let error: std::io::Error = std::io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(number) if number != 0) {
                failure = Some(error);
            }
            break;
        }
        // SAFETY: `readdir` returned a valid entry whose name is NUL-terminated; it is copied before the next call.
        let name: Vec<u8> = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }
            .to_bytes()
            .to_vec();
        if name != b"." && name != b".." {
            names.push(OsString::from_vec(name));
        }
    }
    // SAFETY: `stream` is open and closed once; this also closes the duplicate.
    unsafe { libc::closedir(stream) };
    return match failure {
        Some(error) => Err(error),
        None => Ok(names),
    };
}

/// What:
///  Removes the entry `name` of `parent`:
/// a directory when `directory`,
/// otherwise anything that is not one.
/// Whether this
///  call removed it.
/// Why:
///   The kernel lets exactly one `unlinkat` of a name succeed,
/// so the caller that gets `true` is the entry's one owner and
///  the only one that may count it.
/// An entry another pass removed first is gone,
/// which is the aim reached and no error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mine = await unlinkAt(parent, name, directory) // false: somebody else removed it first
/// ```
pub(crate) fn unlink_inside(
    parent: RawFd,
    name: &OsStr,
    directory: bool,
) -> std::io::Result<bool> {
    let text: CString = c_name(name)?;
    let flags: libc::c_int = if directory { libc::AT_REMOVEDIR } else { 0 };
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `parent` is an open descriptor.
    let result: libc::c_int = unsafe { libc::unlinkat(parent, text.as_ptr(), flags) };
    if result != 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        // Gone already (another pass removed it): the aim is reached, and the other pass owns the count.
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(false);
        }
        return Err(error);
    }
    return Ok(true);
}
