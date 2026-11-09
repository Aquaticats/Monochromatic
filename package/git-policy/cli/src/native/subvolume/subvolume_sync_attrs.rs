//! What:
//!  Copying the attributes of one directory entry onto its copy:
//! owner,
//! permission bits,
//! access and modification
//!  times to the nanosecond,
//! and extended attributes (user attributes and POSIX ACLs included);
//! and recreating a
//!  named pipe.
//! Why:
//!  `cp --archive` and `rsync --archive --xattrs` did this in the migration script.
//! The index stat rewrite trusts
//!  that sizes and modification times survive the copy,
//! and Git compares the mode and owner of each entry,
//! so the
//!  copy must carry them.
//! Failures that an unprivileged user must expect are tolerated and counted,
//! never silent:
//! an
//!  owner other than the caller's cannot be set (`EPERM`),
//! and a security label (`security.selinux`) is set by the
//!  policy for a new file,
//! not by the caller.
//! A failure that no unprivileged user should meet (a permission change or
//!  a time change refused for lack of space,
//! say) is an error,
//! because the copy would be recorded as faithful when it
//!  is not.
//! The attributes are applied through the descriptor of the copy,
//! so a path swapped for a link while the
//!  pass runs cannot redirect them;
//! only a symbolic link,
//! which cannot be opened,
//! is addressed by its path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await chown(copy, st.uid, st.gid); await chmod(copy, st.mode & 0o7777); await utimes(copy, st.atime, st.mtime);
//! ```

use crate::fs_primitive_error::c_path;
use std::collections::BTreeMap;
use std::ffi::{CStr, CString};
use std::fs::{File, Metadata};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// What:
///  How many attribute operations could not be applied,
/// by kind.
/// Why:
///   The report says so,
/// so a person knows what the copy does not carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeLosses = { owner: number; extendedAttributes: number; mode: number; times: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AttributeLosses {
    /// Entries whose owner or group could not be set.
    pub owner: u64,
    /// Extended attributes that could not be copied.
    pub extended_attributes: u64,
    /// Entries whose permission bits the operating system refused to change for lack of ownership.
    pub mode: u64,
    /// Entries whose times the operating system refused to change for lack of ownership.
    pub times: u64,
}

/// What:
///  An attribute operation that failed in a way the copy cannot tolerate.
/// Why:
///   The caller names the path and turns it into the migration's error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeFailure = { action: string; error: Error };
/// ```
#[derive(Debug)]
pub struct AttributeFailure {
    /// What was being set,
    /// in words for `cannot <action> <path>`.
    pub action: &'static str,
    /// The operating system's refusal.
    pub error: std::io::Error,
}

/// What:
///  Where an attribute operation lands:
/// an open file or directory,
/// or a symbolic link addressed by its path.
/// Why:
///   The `f` calls work on a descriptor and cannot be redirected;
/// a symbolic link cannot be opened for them,
/// so the
///  `l` calls take its path and never follow it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Handle = { kind: 'descriptor'; file: FileHandle } | { kind: 'link'; path: string };
/// ```
#[derive(Clone, Copy)]
enum Handle<'a> {
    /// An open file,
    /// directory or named pipe.
    Descriptor(&'a File),
    /// A symbolic link.
    Link(&'a CStr),
}

/// Whether an attribute is one the copy carries and the equality check compares:
/// user attributes and POSIX ACLs.
///  The security label and the other namespaces belong to the policy and to privileged users.
fn is_carried(name: &[u8]) -> bool {
    return name.starts_with(b"user.") || name == b"system.posix_acl_access" || name == b"system.posix_acl_default";
}

/// The names of the extended attributes of a handle,
/// or nothing when it has none or the filesystem refuses.
fn attribute_names(handle: Handle<'_>) -> Vec<Vec<u8>> {
    // SAFETY: a null buffer with size 0 asks for the length of the list.
    let length: isize = unsafe {
        match handle {
            Handle::Descriptor(file) => libc::flistxattr(file.as_raw_fd(), std::ptr::null_mut(), 0),
            Handle::Link(path) => libc::llistxattr(path.as_ptr(), std::ptr::null_mut(), 0),
        }
    };
    if length <= 0 {
        return Vec::new();
    }
    let mut names: Vec<u8> = vec![0; length as usize];
    // SAFETY: `names` holds `length` writable bytes.
    let filled: isize = unsafe {
        match handle {
            Handle::Descriptor(file) => libc::flistxattr(file.as_raw_fd(), names.as_mut_ptr().cast(), names.len()),
            Handle::Link(path) => libc::llistxattr(path.as_ptr(), names.as_mut_ptr().cast(), names.len()),
        }
    };
    if filled <= 0 {
        return Vec::new();
    }
    names.truncate(filled as usize);
    let mut found: Vec<Vec<u8>> = Vec::new();
    for name in names.split(is_zero_byte) {
        if !name.is_empty() {
            found.push(name.to_vec());
        }
    }
    return found;
}

/// Whether `byte` is zero,
/// the separator between attribute names.
fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// The value of one extended attribute,
/// or nothing when it cannot be read.
fn attribute_value(handle: Handle<'_>, name: &CStr) -> Option<Vec<u8>> {
    // SAFETY: a null buffer with size 0 asks for the length of the value.
    let size: isize = unsafe {
        match handle {
            Handle::Descriptor(file) => libc::fgetxattr(file.as_raw_fd(), name.as_ptr(), std::ptr::null_mut(), 0),
            Handle::Link(path) => libc::lgetxattr(path.as_ptr(), name.as_ptr(), std::ptr::null_mut(), 0),
        }
    };
    if size < 0 {
        return None;
    }
    let mut value: Vec<u8> = vec![0; size as usize];
    // SAFETY: `value` holds `size` writable bytes.
    let read: isize = unsafe {
        match handle {
            Handle::Descriptor(file) => libc::fgetxattr(file.as_raw_fd(), name.as_ptr(), value.as_mut_ptr().cast(), value.len()),
            Handle::Link(path) => libc::lgetxattr(path.as_ptr(), name.as_ptr(), value.as_mut_ptr().cast(), value.len()),
        }
    };
    if read < 0 {
        return None;
    }
    value.truncate(read as usize);
    return Some(value);
}

/// Set one extended attribute;
/// true when the operating system accepted it.
fn set_attribute(handle: Handle<'_>, name: &CStr, value: &[u8]) -> bool {
    // SAFETY: `value` holds `value.len()` initialised bytes; the names are NUL-terminated strings.
    let result: libc::c_int = unsafe {
        match handle {
            Handle::Descriptor(file) => libc::fsetxattr(file.as_raw_fd(), name.as_ptr(), value.as_ptr().cast(), value.len(), 0),
            Handle::Link(path) => libc::lsetxattr(path.as_ptr(), name.as_ptr(), value.as_ptr().cast(), value.len(), 0),
        }
    };
    return result == 0;
}

/// Remove one extended attribute;
/// true when it is gone.
fn remove_attribute(handle: Handle<'_>, name: &CStr) -> bool {
    // SAFETY: the names are NUL-terminated strings.
    let result: libc::c_int = unsafe {
        match handle {
            Handle::Descriptor(file) => libc::fremovexattr(file.as_raw_fd(), name.as_ptr()),
            Handle::Link(path) => libc::lremovexattr(path.as_ptr(), name.as_ptr()),
        }
    };
    return result == 0;
}

/// What:
///  Whether the handle already holds `value` under `name`,
/// so a set the system refused loses nothing.
/// Why:
///   A mount that forces one security context (`context=` on a container's overlay) reports the same label on every
/// inode it creates,
/// and a process may not set that label at all:
/// the copy's creation gives the replacement that value,
/// so counting a refusal as a loss would report data that is present as missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alreadyCarries = (handle, name, value) => bufEquals(getxattr(handle, name), value);
/// ```
fn already_carries(destination: Handle<'_>, name: &CStr, value: &[u8]) -> bool {
    let Some(held) = attribute_value(destination, name) else { return false };
    return held == value;
}

/// What:
///  Copy every extended attribute of `source` onto the handle,
/// then remove the carried ones the source lacks.
/// Why:
///   User attributes and ACLs belong to the file.
/// An attribute the source no longer has must not survive on the copy
///  (a removed ACL entry would keep granting access);
/// only the carried namespaces are removed,
/// because the security
///  label is the policy's.
/// A refused attribute is counted,
/// because the caller is not the privileged user that may set
///  every namespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await listxattr(source)) await setxattr(copy, name, await getxattr(source, name));
/// ```
fn copy_extended_attributes(source: &CStr, destination: Handle<'_>, remove_extras: bool, losses: &mut AttributeLosses) {
    let wanted: Vec<Vec<u8>> = attribute_names(Handle::Link(source));
    for name in &wanted {
        let Ok(attribute) = CString::new(name.as_slice()) else { continue };
        let Some(value) = attribute_value(Handle::Link(source), attribute.as_c_str()) else {
            losses.extended_attributes += 1;
            continue;
        };
        // A refusal is a loss only when the destination does not already hold that value: a mount that forces the label
        //  gives it to the copy by itself, and nothing was lost.
        if !set_attribute(destination, attribute.as_c_str(), value.as_slice()) && !already_carries(destination, attribute.as_c_str(), value.as_slice()) {
            losses.extended_attributes += 1;
        }
    }
    if !remove_extras {
        return;
    }
    for name in attribute_names(destination) {
        if !is_carried(name.as_slice()) || wanted.contains(&name) {
            continue;
        }
        let Ok(attribute) = CString::new(name.as_slice()) else { continue };
        if !remove_attribute(destination, attribute.as_c_str()) {
            losses.extended_attributes += 1;
        }
    }
}

/// The carried extended attributes of a path (a link's own,
/// not its target's),
/// by name.
fn carried_attributes(path: &CStr) -> BTreeMap<Vec<u8>, Vec<u8>> {
    let mut found: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
    for name in attribute_names(Handle::Link(path)) {
        if !is_carried(name.as_slice()) {
            continue;
        }
        let Ok(attribute) = CString::new(name.as_slice()) else { continue };
        if let Some(value) = attribute_value(Handle::Link(path), attribute.as_c_str()) {
            found.insert(name, value);
        }
    }
    return found;
}

/// What:
///  Whether two paths carry the same user attributes and ACLs.
/// Why:
///   Size,
/// time and mode do not change when only an attribute does,
/// so the equality of a copy and its original has to
///  look at them too,
/// or a later pass would call a file with an outdated ACL unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = await carriedAttributesEqual(source, destination);
/// ```
pub fn carried_attributes_equal(source: &Path, destination: &Path) -> bool {
    let (Ok(from), Ok(to)) = (c_path("llistxattr", source), c_path("llistxattr", destination)) else { return false };
    return carried_attributes(from.as_c_str()) == carried_attributes(to.as_c_str());
}

/// A failed owner,
/// permission or time change that only a missing privilege explains.
fn is_missing_privilege(error: &std::io::Error) -> bool {
    return matches!(error.raw_os_error(), Some(libc::EPERM | libc::EACCES));
}

/// What:
///  Decide what a failed permission or time change means:
/// a missing privilege is counted,
/// anything else is an error.
/// Why:
///   An entry owned by someone else cannot be changed by this user,
/// and the report says how many;
/// a refusal for another
///  reason (no space for the metadata,
/// a read-only filesystem) leaves a copy that is not what it is recorded to be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (error?.code === 'EPERM') refused += 1; else if (error) throw error;
/// ```
fn settle(outcome: std::io::Result<()>, action: &'static str, refused: &mut u64) -> Result<(), AttributeFailure> {
    let Err(error) = outcome else { return Ok(()) };
    if !is_missing_privilege(&error) {
        return Err(AttributeFailure { action, error });
    }
    *refused += 1;
    return Ok(());
}

/// What:
///  Whether the copy already has the original's owner and group,
/// each given as (user,
/// group).
/// Why:
///   Both must agree before the change is skipped:
/// a copy that differs in only one of them still needs it,
/// and the pure
///  comparison is tested on every combination without needing the privilege to make files of other owners.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = source.uid === current.uid && source.gid === current.gid;
/// ```
fn same_owner(source: (u32, u32), current: (u32, u32)) -> bool {
    return source.0 == current.0 && source.1 == current.1;
}

/// What:
///  Set the owner and group of the copy to those of the original when they differ from what the copy has now.
/// Why:
///   The copy's owner comes from its creator and its group from the creator's group or the parent's setgid
///  directory,
/// so the destination,
/// not the caller,
/// is what must be compared with the original.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (copyStat.uid !== st.uid || copyStat.gid !== st.gid) await lchown(copy, st.uid, st.gid);
/// ```
fn set_owner(destination: Handle<'_>, current: &Metadata, source: &Metadata, losses: &mut AttributeLosses) {
    if same_owner((source.uid(), source.gid()), (current.uid(), current.gid())) {
        return;
    }
    // SAFETY: the descriptor or the NUL-terminated path is valid for the call.
    let result: libc::c_int = unsafe {
        match destination {
            Handle::Descriptor(file) => libc::fchown(file.as_raw_fd(), source.uid(), source.gid()),
            Handle::Link(path) => libc::lchown(path.as_ptr(), source.uid(), source.gid()),
        }
    };
    if result != 0 {
        losses.owner += 1;
    }
}

/// What:
///  Set the access and modification times of the copy,
/// to the nanosecond.
///  `futimens` acts on the descriptor;
/// `utimensat` with `AT_SYMLINK_NOFOLLOW` acts on a symbolic link,
/// not its target.
/// Why:
///   The index stat rewrite and Git's own stat check compare modification times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lutimes(path, atime, mtime);
/// ```
fn set_times(destination: Handle<'_>, source: &Metadata) -> std::io::Result<()> {
    let times: [libc::timespec; 2] = [
        libc::timespec { tv_sec: source.atime(), tv_nsec: source.atime_nsec() },
        libc::timespec { tv_sec: source.mtime(), tv_nsec: source.mtime_nsec() },
    ];
    // SAFETY: `times` is two `timespec` values alive for the call, and the descriptor or path is valid.
    let result: libc::c_int = unsafe {
        match destination {
            Handle::Descriptor(file) => libc::futimens(file.as_raw_fd(), times.as_ptr()),
            Handle::Link(path) => libc::utimensat(libc::AT_FDCWD, path.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW),
        }
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// The shared body of both public functions.
fn apply(
    source: &Path,
    destination: Handle<'_>,
    current: &Metadata,
    metadata: &Metadata,
    remove_extras: bool,
    losses: &mut AttributeLosses,
) -> Result<(), AttributeFailure> {
    if let Ok(from) = c_path("llistxattr", source) {
        copy_extended_attributes(from.as_c_str(), destination, remove_extras, losses);
    }
    set_owner(destination, current, metadata, losses);
    if let Handle::Descriptor(file) = destination {
        // SAFETY: the descriptor is open for the call.
        let result: libc::c_int = unsafe { libc::fchmod(file.as_raw_fd(), metadata.mode() & 0o7777) };
        let outcome: std::io::Result<()> = if result == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
        settle(outcome, "set the permission bits of", &mut losses.mode)?;
    }
    settle(set_times(destination, metadata), "set the times of", &mut losses.times)?;
    return Ok(());
}

/// What:
///  Make the open copy carry the attributes of `source`:
/// extended attributes,
/// owner,
/// permission bits and times,
/// in this
///  order.
///  `remove_extras` also removes the carried attributes the source lacks.
/// Why:
///   Setting the owner clears the set-user-ID bits,
/// so the permission bits come after it;
/// the times come last,
///  because the earlier calls change the change time but not the modification time,
/// and writing the file's content
///  must already be finished.
/// The descriptor may be a regular file,
/// a directory or a named pipe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await preserveAttributes(source, copyHandle, metadata);
/// ```
pub fn preserve_on_descriptor(
    source: &Path,
    destination: &File,
    metadata: &Metadata,
    remove_extras: bool,
    losses: &mut AttributeLosses,
) -> Result<(), AttributeFailure> {
    let current: Metadata = match destination.metadata() {
        Ok(found) => found,
        Err(error) => return Err(AttributeFailure { action: "examine", error }),
    };
    return apply(source, Handle::Descriptor(destination), &current, metadata, remove_extras, losses);
}

/// What:
///  Make the symbolic link at `destination` carry the owner and times of the original link.
/// Why:
///   A link has no permission bits of its own and cannot be opened,
/// so only the `l` calls on its path apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lchown(link, st.uid, st.gid); await lutimes(link, st.atime, st.mtime);
/// ```
pub fn preserve_on_link(source: &Path, destination: &Path, metadata: &Metadata, losses: &mut AttributeLosses) -> Result<(), AttributeFailure> {
    let name: CString = match c_path("lchown", destination) {
        Ok(text) => text,
        Err(error) => return Err(AttributeFailure { action: "address", error: std::io::Error::from_raw_os_error(error.errno) }),
    };
    let current: Metadata = match std::fs::symlink_metadata(destination) {
        Ok(found) => found,
        Err(error) => return Err(AttributeFailure { action: "examine", error }),
    };
    return apply(source, Handle::Link(name.as_c_str()), &current, metadata, false, losses);
}

/// What:
///  Open a named pipe without blocking,
/// so attribute calls can use its descriptor.
/// Why:
///   Opening a pipe read-only with `O_NONBLOCK` succeeds at once whether or not a writer exists,
/// and `O_NOFOLLOW`
///  refuses a link put in its place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await open(path, constants.O_RDONLY | constants.O_NONBLOCK | constants.O_NOFOLLOW);
/// ```
pub fn open_fifo(path: &Path) -> std::io::Result<File> {
    return std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path);
}

/// What:
///  Create a named pipe (FIFO) with the permission bits of the original.
///  `mkfifo(3)` is the call;
/// the umask applies,
/// and `preserve_on_descriptor` sets the exact bits afterwards.
/// Why:
///   A FIFO is part of the tree;
/// sockets and device nodes are not recreated (the report lists them).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkfifo(path, mode);
/// ```
pub fn make_fifo(path: &Path, mode: u32) -> std::io::Result<()> {
    let name = match c_path("mkfifo", path) {
        Ok(text) => text,
        Err(error) => return Err(std::io::Error::from_raw_os_error(error.errno)),
    };
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), mode & 0o7777) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// Attribute controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_sync_attrs_tests.rs"]
mod tests;
