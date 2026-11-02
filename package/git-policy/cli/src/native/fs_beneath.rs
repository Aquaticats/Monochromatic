//! What:
//!  Directory-relative file operations that never follow a symbolic link: open a directory, a file or a link below a held
//!  directory handle one name at a time, create entries there, set times on them, and remove them.
//! Why:
//!  A path string is resolved again by every call that takes it, so a directory swapped for a link between two calls
//!  redirects the second one. The clone rung and the snapshot's stub removal work on a tree another process can rename
//!  into; holding the handle of each directory and naming only one component at a time keeps every call on the inode
//!  that was looked at, and a link in the way is an error, not a detour.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const dir = await openDirectoryBeneath(root, ['a', 'b']); // throws when a component is a link
//! const file = await openFileBeneath(dir, 'c.txt');
//! ```

use std::ffi::{CString, OsStr, OsString};
use std::fs::File;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path};

/// The components of a relative path, each refused unless it is a plain name.
///
/// Why: `..`, `.`, a root or a prefix would leave the held directory or name it twice.
pub fn plain_components(relative: &Path) -> std::io::Result<Vec<OsString>> {
    let mut names: Vec<OsString> = Vec::new();
    for part in relative.components() {
        match part {
            Component::Normal(name) => names.push(name.to_os_string()),
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} is not a plain relative path", relative.display()))),
        }
    }
    return Ok(names);
}

/// A NUL-terminated copy of one name, refused when it is empty, holds a slash or a NUL byte.
fn name_of(name: &OsStr) -> std::io::Result<CString> {
    let bytes: &[u8] = name.as_bytes();
    if bytes.is_empty() || bytes.contains(&b'/') {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "a name must be one plain component"));
    }
    return CString::new(bytes).map_err(|_error| { return std::io::Error::new(std::io::ErrorKind::InvalidInput, "a name holds a NUL byte"); });
}

/// Turn a C result that is `-1` on failure into the thread's error.
fn checked(result: libc::c_int) -> std::io::Result<libc::c_int> {
    return if result < 0 { Err(std::io::Error::last_os_error()) } else { Ok(result) };
}

/// Open the directory `path` itself, refusing a link as its last component.
pub fn open_directory(path: &Path) -> std::io::Result<File> {
    let name = CString::new(path.as_os_str().as_bytes()).map_err(|_error| { return std::io::Error::new(std::io::ErrorKind::InvalidInput, "a path holds a NUL byte"); })?;
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let descriptor: libc::c_int = checked(unsafe { libc::open(name.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) })?;
    // SAFETY: `open` returned a new descriptor that nothing else owns.
    return Ok(File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// Open the subdirectory `name` of `parent`, refusing a link.
pub fn open_directory_at(parent: &File, name: &OsStr) -> std::io::Result<File> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call and `parent` is an open descriptor.
    let descriptor: libc::c_int = checked(unsafe { libc::openat(parent.as_raw_fd(), c_name.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) })?;
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// Open the directory at `names` below `root`, one component at a time, refusing a link or a mount-crossing directory:
///  every directory on the way must be on the device of `root`.
pub fn open_directory_beneath(root: &File, names: &[OsString]) -> std::io::Result<File> {
    let device: u64 = device_of(root)?;
    let mut current: File = root.try_clone()?;
    for name in names {
        let next: File = open_directory_at(&current, name.as_os_str())?;
        if device_of(&next)? != device {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} is on another device", name.to_string_lossy())));
        }
        current = next;
    }
    return Ok(current);
}

/// The device number of an open file.
pub fn device_of(file: &File) -> std::io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    return Ok(file.metadata()?.dev());
}

/// Open the regular file `name` of `dir` for reading, refusing a link; a fifo or a device is refused later by its type.
pub fn open_file_at(dir: &File, name: &OsStr) -> std::io::Result<File> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call and `dir` is an open descriptor; `O_NONBLOCK` keeps a swapped-in fifo from
    // holding the call.
    let descriptor: libc::c_int = checked(unsafe { libc::openat(dir.as_raw_fd(), c_name.as_ptr(), libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC) })?;
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// Create the file `name` in `dir` with `mode`, failing when anything is there, a link included.
pub fn create_file_at(dir: &File, name: &OsStr, mode: u32) -> std::io::Result<File> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call and `dir` is an open descriptor.
    let descriptor: libc::c_int = checked(unsafe {
        libc::openat(dir.as_raw_fd(), c_name.as_ptr(), libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC, mode as libc::c_uint)
    })?;
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// Create the directory `name` in `dir` with `mode`; `Ok(true)` when it was made, `Ok(false)` when a directory was
///  already there (a link or a file in the way is an error).
pub fn make_directory_at(dir: &File, name: &OsStr, mode: u32) -> std::io::Result<bool> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call and `dir` is an open descriptor.
    let result: libc::c_int = unsafe { libc::mkdirat(dir.as_raw_fd(), c_name.as_ptr(), mode as libc::mode_t) };
    if result == 0 {
        return Ok(true);
    }
    let error: std::io::Error = std::io::Error::last_os_error();
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        drop(open_directory_at(dir, name)?);
        return Ok(false);
    }
    return Err(error);
}

/// Create the link `name` in `dir` pointing at `target`.
pub fn symlink_at(target: &Path, dir: &File, name: &OsStr) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    let c_target = CString::new(target.as_os_str().as_bytes()).map_err(|_error| { return std::io::Error::new(std::io::ErrorKind::InvalidInput, "a link target holds a NUL byte"); })?;
    // SAFETY: both strings outlive the call and `dir` is an open descriptor.
    checked(unsafe { libc::symlinkat(c_target.as_ptr(), dir.as_raw_fd(), c_name.as_ptr()) })?;
    return Ok(());
}

/// The target of the link `name` in `dir`.
pub fn read_link_at(dir: &File, name: &OsStr) -> std::io::Result<std::path::PathBuf> {
    // The magic link of an open descriptor names that directory itself, so the lookup cannot be redirected.
    return std::fs::read_link(descriptor_path(dir.as_raw_fd(), name));
}

/// The metadata of the entry `name` in `dir`, without following a link.
pub fn entry_metadata_at(dir: &File, name: &OsStr) -> std::io::Result<std::fs::Metadata> {
    return std::fs::symlink_metadata(descriptor_path(dir.as_raw_fd(), name));
}

/// The path of an entry below an open directory, through the descriptor's magic link.
fn descriptor_path(descriptor: RawFd, name: &OsStr) -> std::path::PathBuf {
    return Path::new("/proc/self/fd").join(descriptor.to_string()).join(name);
}

/// Set the access and modification times of the entry `name` in `dir`, without following a link.
pub fn set_times_at(dir: &File, name: &OsStr, times: [libc::timespec; 2]) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call, `times` holds the two timestamps the call reads, and `dir` is an open descriptor.
    checked(unsafe { libc::utimensat(dir.as_raw_fd(), c_name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) })?;
    return Ok(());
}

/// Remove the entry `name` from `dir`: a directory when `directory` is set (it must be empty), else a file or a link.
pub fn remove_at(dir: &File, name: &OsStr, directory: bool) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    let flags: libc::c_int = if directory { libc::AT_REMOVEDIR } else { 0 };
    // SAFETY: `c_name` outlives the call and `dir` is an open descriptor.
    checked(unsafe { libc::unlinkat(dir.as_raw_fd(), c_name.as_ptr(), flags) })?;
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_beneath_tests.rs"]
mod tests;
