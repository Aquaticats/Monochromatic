//! What:
//!  One shutdown pass's whole input:
//!  the places it reads and writes,
//!  and the calls it makes,
//!  each a seam a control fills.
//! Why:
//!  The pass is driven from no mode in this build
//!  (`crate::worktree_clone_mount_handler::shutdown_unmount_mode`),
//!  so the one value a caller builds is what a guest arm builds from this
//!  machine and what a value test builds from answers it states.
//!  Keeping the places and the calls in one value means the pass is handed
//!  nothing it could read from a place the caller did not choose
//! (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
//!  "Nothing a test changes may be process-wide").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const request = new ShutdownUnmountRequest({ places, mounts, calls, walk, lookups, worktrees, guids, holders });
//! ```

use super::shutdown_unmount_places::ShutdownUnmountPlaces;
use super::shutdown_unmount_sources::{
    HolderScanning,
    SlotLookups,
    WorktreesFolder,
};
use crate::worktree_clone_unmount::slot_walk_calls::{
    DatasetGuids,
    SlotWalk,
};
use crate::worktree_clone_unmount::staging_calls::StagingCalls;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;

/// What:
///  Everything one shutdown pass reads and calls.
/// Why:
///   The pass makes the removal's own calls on a mount that stands where it
///  already is,
///  so it takes the same two seams the removal takes
///  (`crate::worktree_clone_unmount::staging_calls::StagingCalls`,
///  `crate::worktree_clone_unmount::slot_walk_calls::SlotWalk`)
///  beside the four it adds,
///  and one value carries them so no caller can hand the pass a mount table
///  from one machine and a walk from another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShutdownUnmountRequest = { places; mounts; calls; walk; lookups; worktrees; guids; holders };
/// ```
pub struct ShutdownUnmountRequest<'a> {
    /// The places the pass reads and writes.
    pub places: &'a ShutdownUnmountPlaces,
    /// This process's mount table.
    pub mounts: &'a dyn MountedDatasets,
    /// The staging route's calls,
    /// of which this pass makes one:
    /// the unmount from the held descriptor.
    pub calls: &'a dyn StagingCalls,
    /// The descriptor walk to a slot.
    pub walk: &'a dyn SlotWalk,
    /// The fresh capture of a slot.
    pub lookups: &'a dyn SlotLookups,
    /// The `.claude/worktrees` identity the route recorded.
    pub worktrees: &'a dyn WorktreesFolder,
    /// One clone's live GUID.
    pub guids: &'a dyn DatasetGuids,
    /// Who holds a clone the kernel would not let go.
    pub holders: &'a dyn HolderScanning,
}
