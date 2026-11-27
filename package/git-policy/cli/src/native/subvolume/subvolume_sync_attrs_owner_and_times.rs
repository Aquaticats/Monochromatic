//! What:
//!  The owner,
//!  the permission bits and the times of a copy,
//!  set in the order that keeps them true:
//!  the owner first,
//!  because setting it clears the set-user-ID bits,
//!  the permission bits after it,
//!  and the times last.
//! Why:
//!  Git compares the mode and the owner of each entry,
//!  and the index stat rewrite trusts that modification times survive the copy.
//!  A change refused for lack of ownership is counted and tolerated;
//!  a refusal for any other reason is an error,
//!  because a copy is never recorded as faithful when it is not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await preserveOnDescriptor(source, copy, metadata, true, losses);
//! ```

use crate::fs_primitive_error::c_path;
use crate::subvolume_sync_attrs_extended::{
    Handle,
    copy_extended_attributes,
};
use crate::subvolume_sync_attrs_losses::{
    AttributeFailure,
    AttributeLosses,
};
use std::ffi::CString;
use std::fs::{
    File,
    Metadata,
};
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

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
pub(super) fn settle(
    outcome: std::io::Result<()>,
    action: &'static str,
    refused: &mut u64,
) -> Result<(), AttributeFailure> {
    let Err(error) = outcome else { return Ok(()) };
    if !is_missing_privilege(&error) {
        return Err(AttributeFailure {
            action,
            error,
        });
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
pub(super) fn same_owner(
    source: (u32, u32),
    current: (u32, u32),
) -> bool {
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
pub(super) fn set_owner(
    destination: Handle<'_>,
    current: &Metadata,
    source: &Metadata,
    losses: &mut AttributeLosses,
) {
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
fn set_times(
    destination: Handle<'_>,
    source: &Metadata,
) -> std::io::Result<()> {
    let times: [libc::timespec; 2] = [
        libc::timespec {
            tv_sec: source.atime(),
            tv_nsec: source.atime_nsec(),
        },
        libc::timespec {
            tv_sec: source.mtime(),
            tv_nsec: source.mtime_nsec(),
        },
    ];
    // SAFETY: `times` is two `timespec` values alive for the call, and the descriptor or path is valid.
    let result: libc::c_int = unsafe {
        match destination {
            Handle::Descriptor(file) => libc::futimens(file.as_raw_fd(), times.as_ptr()),
            Handle::Link(path) => libc::utimensat(
                libc::AT_FDCWD,
                path.as_ptr(),
                times.as_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            ),
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
        let result: libc::c_int =
            unsafe { libc::fchmod(file.as_raw_fd(), metadata.mode() & 0o7777) };
        let outcome: std::io::Result<()> = if result == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        };
        settle(outcome, "set the permission bits of", &mut losses.mode)?;
    }
    settle(
        set_times(destination, metadata),
        "set the times of",
        &mut losses.times,
    )?;
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
        Err(error) => {
            return Err(AttributeFailure {
                action: "examine",
                error,
            });
        }
    };
    return apply(
        source,
        Handle::Descriptor(destination),
        &current,
        metadata,
        remove_extras,
        losses,
    );
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
pub fn preserve_on_link(
    source: &Path,
    destination: &Path,
    metadata: &Metadata,
    losses: &mut AttributeLosses,
) -> Result<(), AttributeFailure> {
    let name: CString = match c_path("lchown", destination) {
        Ok(text) => text,
        Err(error) => {
            return Err(AttributeFailure {
                action: "address",
                error: std::io::Error::from_raw_os_error(error.errno),
            });
        }
    };
    let current: Metadata = match std::fs::symlink_metadata(destination) {
        Ok(found) => found,
        Err(error) => {
            return Err(AttributeFailure {
                action: "examine",
                error,
            });
        }
    };
    return apply(
        source,
        Handle::Link(name.as_c_str()),
        &current,
        metadata,
        false,
        losses,
    );
}
