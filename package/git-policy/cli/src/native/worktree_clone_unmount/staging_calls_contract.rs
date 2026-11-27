//! What:
//!  The contract every staging call keeps:
//!  the thirteen calls the unmount route and its later sweep make,
//!  each answering a refusal rather than a bare error.
//! Why:
//!  A control on a person's machine cannot mount,
//!  so the route's identity comparisons are proved against a recording
//!  implementation and the guest proves the kernel's own
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The unmount call from staging").
//!  One trait in a file of its own is what both implementations implement,
//!  and what every check of the route is written against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface StagingCalls { bindTheFolder(...); makePrivate(...); moveClone(...); unmountTop(...); ... }
//! ```

use super::staging_call_words::StagingCallRefusal;
use crate::fs_mount_options::MountFacts;
use std::ffi::OsStr;
use std::path::PathBuf;

/// What:
///  The calls the unmount route and its later sweep make.
/// Why:
///  A control records them,
///  and the release path makes them,
///  so the identity comparisons are provable without a mount and the guest
///  proves the calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface StagingCalls { bindTheFolder(...); makePrivate(...); moveClone(...); unmountTop(...); ... }
/// ```
pub trait StagingCalls {
    /// Bind the folder onto itself and answer the descriptor the bind's name opens.
    fn bind_the_folder(
        &self,
        folder: &std::fs::File,
        cache: &std::fs::File,
        name: &OsStr,
    ) -> Result<std::fs::File, StagingCallRefusal>;
    /// Make the bind private.
    fn make_private(
        &self,
        bind: &std::fs::File,
    ) -> Result<(), StagingCallRefusal>;
    /// Move the clone onto the bind.
    fn move_clone(
        &self,
        clone: &std::fs::File,
        bind: &std::fs::File,
    ) -> Result<(), StagingCallRefusal>;
    /// Unmount the topmost mount at the held descriptor.
    fn unmount_top(
        &self,
        held: &std::fs::File,
    ) -> Result<(), StagingCallRefusal>;
    /// The folder's handle and its type.
    fn name_handle(
        &self,
        folder: &std::fs::File,
    ) -> Result<(i32, Vec<u8>), StagingCallRefusal>;
    /// The folder reopened from its handle through the cache's descriptor.
    fn open_from_handle(
        &self,
        cache: &std::fs::File,
        handle_type: i32,
        handle: &[u8],
    ) -> Result<std::fs::File, StagingCallRefusal>;
    /// The entry at `name` beneath the cache descriptor,
    /// crossing the one mount that may cover it.
    fn open_by_name(
        &self,
        cache: &std::fs::File,
        name: &OsStr,
    ) -> Result<std::fs::File, StagingCallRefusal>;
    /// The mount a descriptor is on.
    fn mount_id(
        &self,
        file: &std::fs::File,
    ) -> Result<u64, StagingCallRefusal>;
    /// The path a descriptor names now.
    fn observed_path(
        &self,
        file: &std::fs::File,
    ) -> Result<PathBuf, StagingCallRefusal>;
    /// One mount's facts from this process's table.
    fn mount_facts(
        &self,
        mount_id: u64,
    ) -> Result<MountFacts, StagingCallRefusal>;
    /// This process's whole mount table.
    fn mount_table(&self) -> Result<Vec<MountFacts>, StagingCallRefusal>;
    /// This process's mount namespace device and inode.
    fn namespace_pair(&self) -> Result<(u64, u64), StagingCallRefusal>;
    /// This boot's identifier.
    fn boot(&self) -> Result<String, StagingCallRefusal>;
}
