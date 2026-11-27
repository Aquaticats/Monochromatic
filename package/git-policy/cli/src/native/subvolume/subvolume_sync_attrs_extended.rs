//! What:
//!  The extended attributes of one entry:
//!  read through the descriptor of a copy or the path of a symbolic link,
//!  set on the copy,
//!  removed when the source no longer has them,
//!  and compared between two paths.
//! Why:
//!  User attributes and POSIX ACLs belong to the file,
//!  so a copy carries them and keeps none the source lost
//!  (a removed ACL entry would keep granting access).
//!  A refused attribute is counted rather than raised,
//!  because the caller is not the privileged user that may set every namespace,
//!  and a mount that forces one security context gives the copy that value by itself,
//!  so a refusal there lost nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const name of await listxattr(source)) await setxattr(copy, name, await getxattr(source, name));
//! ```

use crate::fs_primitive_error::c_path;
use crate::subvolume_sync_attrs_losses::AttributeLosses;
use std::collections::BTreeMap;
use std::ffi::{
    CStr,
    CString,
};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;

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
pub(super) enum Handle<'a> {
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
    return name.starts_with(b"user.")
        || name == b"system.posix_acl_access"
        || name == b"system.posix_acl_default";
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
            Handle::Descriptor(file) => {
                libc::flistxattr(file.as_raw_fd(), names.as_mut_ptr().cast(), names.len())
            }
            Handle::Link(path) => {
                libc::llistxattr(path.as_ptr(), names.as_mut_ptr().cast(), names.len())
            }
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
fn attribute_value(
    handle: Handle<'_>,
    name: &CStr,
) -> Option<Vec<u8>> {
    // SAFETY: a null buffer with size 0 asks for the length of the value.
    let size: isize = unsafe {
        match handle {
            Handle::Descriptor(file) => {
                libc::fgetxattr(file.as_raw_fd(), name.as_ptr(), std::ptr::null_mut(), 0)
            }
            Handle::Link(path) => {
                libc::lgetxattr(path.as_ptr(), name.as_ptr(), std::ptr::null_mut(), 0)
            }
        }
    };
    if size < 0 {
        return None;
    }
    let mut value: Vec<u8> = vec![0; size as usize];
    // SAFETY: `value` holds `size` writable bytes.
    let read: isize = unsafe {
        match handle {
            Handle::Descriptor(file) => libc::fgetxattr(
                file.as_raw_fd(),
                name.as_ptr(),
                value.as_mut_ptr().cast(),
                value.len(),
            ),
            Handle::Link(path) => libc::lgetxattr(
                path.as_ptr(),
                name.as_ptr(),
                value.as_mut_ptr().cast(),
                value.len(),
            ),
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
fn set_attribute(
    handle: Handle<'_>,
    name: &CStr,
    value: &[u8],
) -> bool {
    // SAFETY: `value` holds `value.len()` initialised bytes; the names are NUL-terminated strings.
    let result: libc::c_int = unsafe {
        match handle {
            Handle::Descriptor(file) => libc::fsetxattr(
                file.as_raw_fd(),
                name.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
                0,
            ),
            Handle::Link(path) => libc::lsetxattr(
                path.as_ptr(),
                name.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
                0,
            ),
        }
    };
    return result == 0;
}

/// Remove one extended attribute;
/// true when it is gone.
fn remove_attribute(
    handle: Handle<'_>,
    name: &CStr,
) -> bool {
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
pub(super) fn already_carries(
    destination: Handle<'_>,
    name: &CStr,
    value: &[u8],
) -> bool {
    let Some(held) = attribute_value(destination, name) else {
        return false;
    };
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
pub(super) fn copy_extended_attributes(
    source: &CStr,
    destination: Handle<'_>,
    remove_extras: bool,
    losses: &mut AttributeLosses,
) {
    let wanted: Vec<Vec<u8>> = attribute_names(Handle::Link(source));
    for name in &wanted {
        let Ok(attribute) = CString::new(name.as_slice()) else {
            continue;
        };
        let Some(value) = attribute_value(Handle::Link(source), attribute.as_c_str()) else {
            losses.extended_attributes += 1;
            continue;
        };
        // A refusal is a loss only when the destination does not already hold that value: a mount that forces the label
        //  gives it to the copy by itself, and nothing was lost.
        if !set_attribute(destination, attribute.as_c_str(), value.as_slice())
            && !already_carries(destination, attribute.as_c_str(), value.as_slice())
        {
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
        let Ok(attribute) = CString::new(name.as_slice()) else {
            continue;
        };
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
        let Ok(attribute) = CString::new(name.as_slice()) else {
            continue;
        };
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
pub fn carried_attributes_equal(
    source: &Path,
    destination: &Path,
) -> bool {
    let (Ok(from), Ok(to)) = (
        c_path("llistxattr", source),
        c_path("llistxattr", destination),
    ) else {
        return false;
    };
    return carried_attributes(from.as_c_str()) == carried_attributes(to.as_c_str());
}
