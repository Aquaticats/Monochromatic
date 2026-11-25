//! What:
//!  The pool read the mounted arm of the two-fact proof compares a record's `cloneGuid=` with.
//! Why:
//!  A dataset name alone passes a dataset made again under the same name after the recorded one was destroyed,
//!  and the GUID does not,
//!  so while a mount covers the slot the proof reads the GUID of the dataset the mount's source names
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "What proves ownership",
//!  and measurement `M2`).
//!  The read is the add route's own,
//!  `worktree_clone_add_route::add_zfs::OpenedDevice::identity_of`,
//!  so this seam names it rather than writing a second reader,
//!  and a control hands in its own answers so no host control asks a pool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface DatasetGuidReads { guidOf(dataset: string): DatasetGuid } // throws the pool's own reason
//! ```

use crate::worktree_clone_mount_protocol::DatasetGuid;

/// What:
///  One dataset's GUID,
///  as the pool answers it.
/// Why:
///  The mounted arm compares an identity the name cannot carry,
///  and the caller owns how the pool is reached:
///  the add's resume passes the device it already opened behind the gate,
///  and a control passes its own scripted answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guid = reads.guidOf(record.clone.dataset); // the route's identity_of, and nothing else
/// ```
pub trait DatasetGuidReads {
    /// Read one dataset's GUID from the pool.
    ///
    /// # Errors
    /// The dataset does not answer,
    /// or the pool reported no GUID this route can name.
    fn guid_of(&self, dataset: &str) -> Result<DatasetGuid, String>;
}
