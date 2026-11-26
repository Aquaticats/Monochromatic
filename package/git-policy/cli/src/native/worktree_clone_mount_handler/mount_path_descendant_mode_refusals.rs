//! What:
//!  Why the `mount-path-descendant` mode mounted nothing:
//!  one variant per cause,
//!  each naming the property,
//!  the dataset or the path a person must look at,
//!  and the words every refusal answers with.
//! Why:
//!  `mount_path_descendant_mode.rs` decides whether a mount may be placed and
//!  would go over the file-size limit carrying these words too,
//!  and every proof in it answers with one of these variants,
//!  so a reader holds the causes apart from the proofs that find them.
//!  No refusal here acts:
//!  each is answered before a walk starts or after one refused,
//!  and none is answered after a mount was placed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MountPathDescendantFailure = { kind: 'not-root'; effectiveUser } | { kind: 'dataset-not-mountable'; dataset; property; value; because };
//! ```

use crate::worktree_clone_mount::attach_the_clone::AttachRefusal;
use crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name;
use crate::worktree_clone_mount_handler::record_checkout_mode::RecordCheckoutFailure;
use crate::worktree_clone_mount_handler::root_mode_caller::NotRootRefusal;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use std::path::PathBuf;

/// What:
///  Why the mode mounted nothing.
/// Why:
///  Each refusal names the property, name or path a person must look at,
///  and no refusal is answered after a mount has been placed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountPathDescendantFailure = { kind: 'not-root' | 'no-recorded-checkout' | ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MountPathDescendantFailure {
    /// The mode is not running as root.
    NotRoot {
        /// The effective user the mode found.
        effective_user: u32,
    },
    /// The record is not there.
    NoRecordedCheckout {
        /// The record's path.
        record: PathBuf,
    },
    /// The record could not be read.
    Record(RecordCheckoutFailure),
    /// The record no longer agrees with the kernel.
    RecordNoLongerMatches {
        /// The field that differs.
        field: String,
        /// What the record holds.
        recorded: String,
        /// What the kernel shows.
        kernel: String,
    },
    /// The dataset's own reply could not be read.
    DatasetRead {
        /// The dataset whose reply was asked for.
        dataset: String,
        /// What the read answered.
        failure: ReadFailure,
    },
    /// The mount table does not show the recorded checkout mounted now.
    CheckoutNotMounted {
        /// The recorded checkout.
        checkout: PathBuf,
        /// The dataset the record names.
        dataset: String,
    },
    /// The dataset's mountpoint is not strictly under the recorded checkout.
    DatasetOutsideTheCheckout {
        /// The dataset.
        dataset: String,
        /// The mountpoint the property gives.
        mountpoint: PathBuf,
        /// The recorded checkout.
        checkout: PathBuf,
    },
    /// The dataset is in another pool than the recorded one.
    DatasetInAnotherPool {
        /// The dataset.
        dataset: String,
        /// The pool the dataset is in.
        pool: String,
        /// The pool the record holds.
        recorded: String,
    },
    /// The dataset has no mountpoint this mode may use.
    DatasetWithoutAMountpoint {
        /// The dataset.
        dataset: String,
        /// The value the property holds.
        mountpoint: String,
    },
    /// The kernel has the dataset mounted somewhere else.
    DatasetAlreadyMountedElsewhere {
        /// The dataset.
        dataset: String,
        /// Where the kernel has it.
        mounted_at: PathBuf,
        /// Where its property says it belongs.
        mountpoint: PathBuf,
    },
    /// An explicit mount would refuse this dataset.
    DatasetNotMountable {
        /// The dataset.
        dataset: String,
        /// The property that refuses it.
        property: String,
        /// The value the property holds.
        value: String,
        /// What the value means for a mount.
        because: String,
    },
    /// An encrypted dataset's key is not loaded.
    EncryptionKeyNotLoaded {
        /// The dataset.
        dataset: String,
        /// The value `keystatus` holds.
        keystatus: String,
    },
    /// The mountpoint holds something and the dataset does not overlay.
    MountpointNotEmpty {
        /// The dataset.
        dataset: String,
        /// The mountpoint.
        mountpoint: PathBuf,
    },
    /// The walk to the mountpoint refused.
    MountpointUnusable {
        /// The dataset.
        dataset: String,
        /// The mountpoint.
        mountpoint: PathBuf,
        /// What the walk found.
        because: String,
    },
    /// The mount itself refused.
    MountFailed {
        /// The dataset.
        dataset: String,
        /// The refusal the primitive answered.
        refusal: AttachRefusal,
    },
}

impl MountPathDescendantFailure {
    /// The refusal in words,
    /// opening with the words every root mode is reached through.
    pub fn message(&self) -> String {
        let prefix: String = format!("{}: ", root_modes_name());
        return match self {
            MountPathDescendantFailure::NotRoot {
                effective_user,
            } => {
                return NotRootRefusal {
                    mode: "mount-path-descendant",
                    nothing: "read, mounted or recorded",
                    because: "a mount this mode places stands in the one mount namespace of the process that placed it, and an account that could place one inside a served checkout could mount anything there",
                    way: "run the mode as root, from the boot unit or by hand",
                    effective_user: *effective_user,
                }
                .sentence();
            }
            MountPathDescendantFailure::NoRecordedCheckout {
                record,
            } => format!(
                "{prefix}no-recorded-checkout: {} holds no record, so nothing says which checkout this dataset's mountpoint descends from. Nothing was mounted. Ways forward: run `{} record-checkout --checkout <path>` as root, then run the mode again, or let the record's repair run from the value the boot printed.",
                record.display(),
                root_modes_name()
            ),
            MountPathDescendantFailure::Record(refusal) => refusal.message(),
            MountPathDescendantFailure::RecordNoLongerMatches {
                field,
                recorded,
                kernel,
            } => format!(
                "{prefix}checkout-record-no-longer-matches: the record's {field} is {recorded} and the kernel shows {kernel}. Nothing was mounted. Ways forward: run `{} record-checkout --checkout <path> --replace {kernel}` as root when the record only moved its GUID, or restore the dataset the record names; until then this dataset stays unmounted.",
                root_modes_name()
            ),
            MountPathDescendantFailure::DatasetRead {
                dataset,
                failure,
            } => format!(
                "{prefix}`{dataset}` could not be read from the kernel, so nothing was mounted: {}. Ways forward: check that the dataset exists in an imported pool, then run the mode again.",
                failure.message()
            ),
            MountPathDescendantFailure::CheckoutNotMounted {
                checkout,
                dataset,
            } => format!(
                "{prefix}checkout-not-mounted: the mount table does not show `{dataset}` mounted at {} now, so the folder this dataset would be placed under is not a mount the route verified. Nothing was mounted. Ways forward: check that the checkout's own mount is in place (the boot's checkout unit, or `zfs mount <the checkout dataset>`), then run the mode again.",
                checkout.display()
            ),
            MountPathDescendantFailure::DatasetOutsideTheCheckout {
                dataset,
                mountpoint,
                checkout,
            } => format!(
                "{prefix}dataset-outside-the-checkout: `{dataset}` mounts at {}, which is not strictly under the recorded checkout {}. Nothing was mounted. Ways forward: give the dataset a `mountpoint` under the checkout, or mount it with the unit that owned it before this mode.",
                mountpoint.display(),
                checkout.display()
            ),
            MountPathDescendantFailure::DatasetInAnotherPool {
                dataset,
                pool,
                recorded,
            } => format!(
                "{prefix}dataset-in-another-pool: `{dataset}` is in pool `{pool}` and the recorded checkout is in pool `{recorded}`, so this dataset would put another pool's subtree inside a served checkout. Nothing was mounted. Ways forward: name a dataset in the recorded checkout's own pool, or serve the checkout whose pool holds it.",
            ),
            MountPathDescendantFailure::DatasetWithoutAMountpoint {
                dataset,
                mountpoint,
            } => format!(
                "{prefix}dataset-without-a-mountpoint: `{dataset}`'s `mountpoint` is `{mountpoint}`, so the dataset is mounted by someone else's rule and this mode would be guessing. Nothing was mounted. Ways forward: give the dataset a `mountpoint` under the checkout, then run the mode again."
            ),
            MountPathDescendantFailure::DatasetAlreadyMountedElsewhere {
                dataset,
                mounted_at,
                mountpoint,
            } => format!(
                "{prefix}dataset-already-mounted-elsewhere: the kernel has `{dataset}` mounted at {}, and its own `mountpoint` is {}. Nothing was mounted. Ways forward: unmount the dataset where it stands, or give its property the path it is mounted at, then run the mode again.",
                mounted_at.display(),
                mountpoint.display()
            ),
            MountPathDescendantFailure::DatasetNotMountable {
                dataset,
                property,
                value,
                because,
            } => format!(
                "{prefix}dataset-not-mountable: `{dataset}`'s `{property}` is `{value}`, and {because}. Nothing was mounted. Ways forward: clear the property the refusal names, or let the unit that owned this dataset before this mode mount it."
            ),
            MountPathDescendantFailure::EncryptionKeyNotLoaded {
                dataset,
                keystatus,
            } => format!(
                "{prefix}encryption-key-not-loaded: `{dataset}` is encrypted and its `keystatus` is `{keystatus}`. Nothing was mounted. Ways forward: load the key by hand (`zfs load-key <dataset>`), then run the mode again; a boot unit has nobody to prompt."
            ),
            MountPathDescendantFailure::MountpointNotEmpty {
                dataset,
                mountpoint,
            } => format!(
                "{prefix}mountpoint-not-empty: {} holds files and `{dataset}`'s `overlay` is off, so a mount there would hide them. Nothing was mounted. Ways forward: empty the folder, or give the dataset `overlay=on` when hiding what it holds is intended.",
                mountpoint.display()
            ),
            MountPathDescendantFailure::MountpointUnusable {
                dataset,
                mountpoint,
                because,
            } => format!(
                "{prefix}mountpoint-unusable: the walk to {} refused for `{dataset}`: {because}. Nothing was mounted. Ways forward: check the folder's components as the refusal names them, then run the mode again; this mode creates the last component and no tree above it.",
                mountpoint.display()
            ),
            MountPathDescendantFailure::MountFailed {
                dataset,
                refusal,
            } => format!(
                "{prefix}the mount of `{dataset}` refused at {}: {}. Nothing was mounted. Ways forward: read the step the refusal names, fix what it reports, then run the mode again.",
                refusal.call.call_name(),
                refusal.message()
            ),
        };
    }
}
