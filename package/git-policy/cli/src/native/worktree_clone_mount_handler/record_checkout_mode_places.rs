//! What:
//!  Where one `record-checkout` run reads and writes:
//!  the configuration it takes the container from,
//!  the record it compares and writes,
//!  and the three reads every proof is made over.
//! Why:
//!  The release path takes the installed places and the live reads,
//!  and a control hands in fixture paths and scripted reads,
//!  so the places one run is given are named once and every step of the run
//!  reads them rather than choosing a table of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RecordCheckoutPlaces = { configuration: string; record: string; mounts: CheckoutMounts; mounted: MountedDatasets; datasets: DatasetReader };
//! ```

use crate::worktree_clone_mount_handler::sweep_checkout_mount::CheckoutMounts;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use std::path::PathBuf;

/// What:
///  Where one `record-checkout` run reads and writes,
///  with the installed places by default.
/// Why:
///   The release path reads the installed configuration and writes the installed
///  record,
///  and a control hands in fixture paths and scripted reads,
///  so no control reads this machine's table,
///  pool or record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordCheckoutPlaces = { configuration, record, mounts, mounted, datasets };
/// ```
pub struct RecordCheckoutPlaces<'a> {
    /// The configuration the container is named in.
    pub configuration: PathBuf,
    /// The record the mode writes and compares against.
    pub record: PathBuf,
    /// The mount table the checkout's own mount is read from.
    pub mounts: &'a dyn CheckoutMounts,
    /// The mount table rows the container is proved against.
    pub mounted: &'a dyn MountedDatasets,
    /// The pool reads the dataset's own reply comes from.
    pub datasets: &'a dyn DatasetReader,
}
