//! What:
//!  Directory-relative file operations that never follow a symbolic link or cross a mount:
//! open a directory,
//! a file or a
//!  link below a held directory handle one plain name at a time (`openat2` with `RESOLVE_BENEATH`,
//! `RESOLVE_NO_SYMLINKS`,
//!  `RESOLVE_NO_XDEV` and `RESOLVE_NO_MAGICLINKS`),
//! create entries there,
//! set times on them,
//! rename them,
//! publish an open
//!  file under a new name without replacing anything,
//! and remove a lock file whose identity is the one that was recorded.
//! Why:
//!  A path string is resolved again by every call that takes it,
//! so a directory swapped for a link between two calls
//!  redirects the second one.
//! A name that was checked may also name another object when it is unlinked,
//! and nothing in the
//!  kernel makes the two one step,
//! so no tree is deleted through this module:
//! what the ladder made is kept and reported.
//!  The clone rung and the snapshot's stub removal work on trees other processes can rename into;
//! holding the handle of each
//!  directory and naming one component at a time keeps every call on the object that was looked at.
//!
//!  The walk down a chain of names and the hop that may end on a mount are in `fs_beneath_walk_chain`,
//!  the mount ID and the path a held descriptor names now are in `fs_beneath_mount_facts`,
//!  and both use the open `fs_beneath_open` owns,
//!  so one place keeps the resolution bits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const root = await openRoot('/canonical/path'); // throws when an ancestor is a link
//! const dir = await openDirectoryAt(root, 'a'); // throws on a link, a mount, '.' or '..'
//! await removeOwnedAt(dir, 'file', recordedIdentity); // keeps a replacement
//! ```
//!
//! The code of this module is split by role,
//! and this file re-exports every name those files hold,
//! so that a caller writes `fs_beneath::open_root` as it did before:
//! `fs_beneath_names` holds the plain names and the descriptor paths,
//! `fs_beneath_identity` the identity of an object,
//! `fs_beneath_open` the `openat2` opens,
//! `fs_beneath_entries` the entry calls,
//! `fs_beneath_owned_removal` the lock file's removal and the publishing of an open file,
//! and `fs_beneath_permissions` the widening of a directory.

pub use super::fs_beneath_entries::{
    create_file_at,
    entry_metadata_at,
    exchange_at,
    make_directory_at,
    read_link_at,
    remove_at,
    rename_at,
    set_times_at,
    symlink_at,
};
pub use super::fs_beneath_identity::{
    ObjectId,
    device_of,
    identity_at,
    identity_of,
};
pub use super::fs_beneath_names::{
    handle_path,
    plain_components,
};
pub use super::fs_beneath_open::{
    handle_directory_at,
    open_directory_at,
    open_directory_crossing_one_mount,
    open_file_at,
    open_handle_at_a_mount,
    open_root,
};
pub use super::fs_beneath_owned_removal::{
    Owned,
    link_open_file_at,
    remove_if_identical,
};
pub use super::fs_beneath_permissions::{
    open_directory_widening,
    widen_directory,
    widen_through_a_handle,
};
// The `RESOLVE_*` bits and the open they go to are the folder's own: `fs_beneath_walk_chain` builds its last hop from
// them, so they are re-exported at the visibility they had here.
pub(super) use super::fs_beneath_open::{
    RESOLVE_BENEATH,
    RESOLVE_NO_MAGICLINKS,
    RESOLVE_NO_SYMLINKS,
    open_beneath_by,
};

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_beneath_tests.rs"]
mod tests;
