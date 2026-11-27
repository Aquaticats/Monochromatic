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
//!
//! The code of this module is split by role,
//! and this file re-exports every name those files hold,
//! so that a caller writes `subvolume_sync_attrs::preserve_on_descriptor` as it did before:
//! `subvolume_sync_attrs_losses` holds what a copy could not carry,
//! `subvolume_sync_attrs_extended` the extended attributes,
//! `subvolume_sync_attrs_owner_and_times` the owner,
//! the permission bits and the times,
//! and `subvolume_sync_attrs_fifo` the named pipes.

pub use crate::subvolume_sync_attrs_extended::carried_attributes_equal;
pub use crate::subvolume_sync_attrs_fifo::{
    make_fifo,
    open_fifo,
};
pub use crate::subvolume_sync_attrs_losses::{
    AttributeFailure,
    AttributeLosses,
};
pub use crate::subvolume_sync_attrs_owner_and_times::{
    preserve_on_descriptor,
    preserve_on_link,
};

// The pieces the module's own controls reach, at the visibility this module gave them.
#[cfg(test)]
pub(super) use crate::subvolume_sync_attrs_extended::{
    Handle,
    already_carries,
};
#[cfg(test)]
pub(super) use crate::subvolume_sync_attrs_owner_and_times::{
    same_owner,
    set_owner,
    settle,
};

/// Attribute controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_sync_attrs_tests.rs"]
mod tests;
