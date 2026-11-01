//! What:
//!  Copying the attributes of one directory entry onto its copy: owner, permission bits, access and modification
//!  times to the nanosecond, and extended attributes; and recreating a named pipe.
//! Why:
//!  `cp --archive` and `rsync --archive --xattrs` did this in the migration script. The index stat rewrite trusts
//!  that sizes and modification times survive the copy, and Git compares the mode and owner of each entry, so the
//!  copy must carry them. Failures that an unprivileged user must expect are tolerated and counted, never silent: an
//!  owner other than the caller's cannot be set (`EPERM`), and a security label (`security.selinux`) is set by the
//!  policy for a new file, not by the caller.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await chown(copy, st.uid, st.gid); await chmod(copy, st.mode & 0o7777); await utimes(copy, st.atime, st.mtime);
//! ```

use super::fs_primitive_error::c_path;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  How many attribute operations could not be applied, by kind.
/// Why:
///   The report says so, so a person knows what the copy does not carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeLosses = { owner: number; extendedAttributes: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AttributeLosses {
    /// Entries whose owner or group could not be set.
    pub owner: u64,
    /// Extended attributes that could not be copied.
    pub extended_attributes: u64,
}

/// What:
///  Set the owner and group of the entry itself (not what a symbolic link points at).
/// Why:
///   Only when they differ from what a new file gets, so the common case costs no call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lchown(path, uid, gid);
/// ```
fn set_owner(path: &Path, source: &Metadata, losses: &mut AttributeLosses) {
    let Ok(name) = c_path("lchown", path) else { return };
    // SAFETY: `name` is a NUL-terminated string that outlives the call; `getuid` and `getgid` take no arguments.
    let (own_uid, own_gid): (u32, u32) = unsafe { (libc::getuid(), libc::getgid()) };
    if source.uid() == own_uid && source.gid() == own_gid {
        return;
    }
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::lchown(name.as_ptr(), source.uid(), source.gid()) };
    if result != 0 {
        losses.owner += 1;
    }
}

/// What:
///  Set the access and modification times of the entry itself, to the nanosecond.
///  `utimensat` with `AT_SYMLINK_NOFOLLOW` acts on a symbolic link, not its target.
/// Why:
///   The index stat rewrite and Git's own stat check compare modification times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lutimes(path, atime, mtime);
/// ```
fn set_times(path: &Path, source: &Metadata) {
    let Ok(name) = c_path("utimensat", path) else { return };
    let times: [libc::timespec; 2] = [
        libc::timespec { tv_sec: source.atime(), tv_nsec: source.atime_nsec() },
        libc::timespec { tv_sec: source.mtime(), tv_nsec: source.mtime_nsec() },
    ];
    // SAFETY: `name` is a NUL-terminated string and `times` is two `timespec` values, both alive for the call.
    let _result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
}

/// What:
///  Copy every extended attribute of `source` onto `destination` (the entries themselves, not link targets).
/// Why:
///   User attributes belong to the file. A refused attribute is counted, because the caller is not the privileged
///  user that may set every namespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await listxattr(source)) await setxattr(destination, name, await getxattr(source, name));
/// ```
fn copy_extended_attributes(source: &Path, destination: &Path, losses: &mut AttributeLosses) {
    let (Ok(from), Ok(to)) = (c_path("llistxattr", source), c_path("lsetxattr", destination)) else { return };
    // SAFETY: a null buffer with size 0 asks for the length of the list.
    let length: isize = unsafe { libc::llistxattr(from.as_ptr(), std::ptr::null_mut(), 0) };
    if length <= 0 {
        return;
    }
    let mut names: Vec<u8> = vec![0; length as usize];
    // SAFETY: `names` holds `length` writable bytes.
    let filled: isize = unsafe { libc::llistxattr(from.as_ptr(), names.as_mut_ptr().cast(), names.len()) };
    if filled <= 0 {
        return;
    }
    names.truncate(filled as usize);
    for name in names.split(|byte: &u8| { return *byte == 0; }).filter(|name: &&[u8]| { return !name.is_empty(); }) {
        let Ok(attribute) = std::ffi::CString::new(name) else { continue };
        // SAFETY: a null buffer with size 0 asks for the length of the value.
        let size: isize = unsafe { libc::lgetxattr(from.as_ptr(), attribute.as_ptr(), std::ptr::null_mut(), 0) };
        if size < 0 {
            losses.extended_attributes += 1;
            continue;
        }
        let mut value: Vec<u8> = vec![0; size as usize];
        // SAFETY: `value` holds `size` writable bytes.
        let read: isize = unsafe { libc::lgetxattr(from.as_ptr(), attribute.as_ptr(), value.as_mut_ptr().cast(), value.len()) };
        if read < 0 {
            losses.extended_attributes += 1;
            continue;
        }
        // SAFETY: `value` holds `read` initialised bytes; the names are NUL-terminated strings.
        let set: libc::c_int = unsafe { libc::lsetxattr(to.as_ptr(), attribute.as_ptr(), value.as_ptr().cast(), read as usize, 0) };
        if set != 0 {
            losses.extended_attributes += 1;
        }
    }
}

/// What:
///  Make the copy at `destination` carry the attributes of `source`: extended attributes, owner, permission bits (not
///  for a symbolic link, which has none of its own) and times, in this order.
/// Why:
///   Setting the owner clears the set-user-ID bits, so the permission bits come after it; the times come last,
///  because the earlier calls change the change time but not the modification time, and writing the file's content
///  must already be finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await preserveAttributes(source, destination, metadata);
/// ```
pub fn preserve_attributes(source: &Path, destination: &Path, metadata: &Metadata, losses: &mut AttributeLosses) {
    copy_extended_attributes(source, destination, losses);
    set_owner(destination, metadata, losses);
    if !metadata.file_type().is_symlink() {
        let _result = std::fs::set_permissions(destination, std::os::unix::fs::PermissionsExt::from_mode(metadata.mode() & 0o7777));
    }
    set_times(destination, metadata);
}

/// What:
///  Create a named pipe (FIFO) with the permission bits of the original.
///  `mkfifo(3)` is the call; the umask applies, and `preserve_attributes` sets the exact bits afterwards.
/// Why:
///   A FIFO is part of the tree; sockets and device nodes are not recreated (the report lists them).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkfifo(path, mode);
/// ```
pub fn make_fifo(path: &Path, mode: u32) -> std::io::Result<()> {
    let name = c_path("mkfifo", path).map_err(|error| { return std::io::Error::from_raw_os_error(error.errno); })?;
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), mode & 0o7777) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}
