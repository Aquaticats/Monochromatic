//! What:
//!  What the checkout record holds and why the mode recorded nothing:
//!  the record's version and the modes its file and its folder are made with,
//!  the six facts one run reads out of the checkout,
//!  the field a later run finds different,
//!  and the refusal each cause answers.
//! Why:
//!  `record_checkout_mode.rs` decides whether to write and would go over the
//!  file-size limit carrying the record's shape and every refusal too,
//!  and these are what the mode's other files need first:
//!  the file's own read and write take both types,
//!  and the mode's proofs answer with them.
//!  Nothing here reads a file or the kernel:
//!  every value is the record's or the refusal's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RecordCheckoutFailure = { kind: 'checkout-record-differs'; recorded: string; kernel: string; field: DifferingField };
//! ```

use crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name;
use crate::worktree_clone_mount_handler::root_mode_caller::NotRootRefusal;
use crate::worktree_clone_mount_handler::sweep_checkout_mount::CheckoutMountRefusal;
use crate::worktree_clone_mount_handler::sweep_configuration_refusal::ConfigurationRefusal;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use std::path::PathBuf;

/// The version of the record's own format,
/// which the step that adds a field is the step that moves.
pub const RECORD_VERSION: u64 = 1;

/// The mode the record's folder is made with:
/// root writes it,
/// and an account may read and enter it.
pub const RECORD_FOLDER_MODE: u32 = 0o755;

/// The mode the record file is made with:
/// readable by every account,
/// writable by root alone.
pub const RECORD_MODE: u32 = 0o644;

/// The permission bits that let an account other than the owner write.
pub const WRITABLE_BY_OTHERS: u32 = 0o022;

/// The name the record is written under while it is being replaced,
/// so a reader sees the old record or the new one and never half of either.
pub const WRITING_NAME: &str = ".git-worktree-clone-checkout.writing";

/// What:
///  The five facts the route reads out of one checkout.
/// Why:
///  They are written together and compared together,
///  so a record that agrees about the path and disagrees about the GUID cannot
///  be answered as if it agreed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedCheckout = { version, checkout, dataset, guid, container, pool };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedCheckout {
    /// The record format's version.
    pub version: u64,
    /// The checkout's path.
    pub checkout: PathBuf,
    /// The dataset mounted there.
    pub dataset: String,
    /// That dataset's own GUID,
    /// as decimal digits,
    /// because it is wider than a JSON number carries exactly.
    pub guid: String,
    /// The dataset holding the managed clones.
    pub container: String,
    /// The pool both datasets are in.
    pub pool: String,
}

/// What:
///  One field of the record that the kernel no longer agrees with.
/// Why:
///  Each field's repair is a different command,
///  and a refusal that named only "the record differs" would send a person to
///  the runbook for a flag that fixes another field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DifferingField = { field: 'path' | 'dataset' | 'guid' | 'container', recorded, kernel };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DifferingField {
    /// The recorded path is not the checkout.
    Path {
        /// The path the record holds.
        recorded: String,
    },
    /// The recorded dataset is not the one mounted at the checkout.
    Dataset {
        /// The dataset the record holds.
        recorded: String,
    },
    /// The recorded GUID is not the one the kernel shows.
    Guid {
        /// The GUID the record holds.
        recorded: String,
    },
    /// The recorded container is not the one the configuration names.
    Container {
        /// The container the record holds.
        recorded: String,
    },
}

/// What:
///  Why the mode recorded nothing.
/// Why:
///  Every refusal names what was not done and a way forward a person can run,
///  and the two GUID refusals print the number the repair takes,
///  which is what lets a boot refuse and still leave one command to type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordCheckoutFailure = { kind: 'not-root' | 'unreadable' | 'configuration' | 'not-a-mount-point' | ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordCheckoutFailure {
    /// The mode is not running as root.
    NotRoot {
        /// The effective user the mode found.
        effective_user: u32,
    },
    /// The configuration could not be read.
    Configuration(ConfigurationRefusal),
    /// The checkout's mount could not be read.
    Mount(CheckoutMountRefusal),
    /// The checkout's own dataset could not be read.
    DatasetRead {
        /// The dataset whose reply was asked for.
        dataset: String,
        /// What the read answered.
        failure: ReadFailure,
    },
    /// The checkout is not a mount point of a dataset.
    NotAMountPoint {
        /// The checkout as given.
        checkout: PathBuf,
        /// What the read found instead.
        found: String,
    },
    /// The checkout's mount is not a ZFS dataset.
    NotOnZfs {
        /// The checkout as given.
        checkout: PathBuf,
        /// The filesystem type the mount lists.
        filesystem_type: String,
    },
    /// The container is not a dataset beside the checkout.
    ContainerNotBesideTheCheckout {
        /// The dataset the checkout is on.
        dataset: String,
        /// The container dataset.
        container: String,
        /// Which direction the proof found.
        because: String,
    },
    /// The record exists and a field differs from the kernel.
    RecordDiffers {
        /// What the record holds under the field that differs.
        recorded: String,
        /// What the kernel shows.
        kernel: String,
        /// The field that differs.
        field: DifferingField,
    },
    /// `--replace` was given a value that is not the kernel's GUID.
    ReplaceTokenDiffers {
        /// The GUID the kernel shows.
        kernel: String,
        /// The value the caller typed.
        typed: String,
    },
    /// `--replace` was given and no record stands.
    NothingToReplace {
        /// The record's path.
        record: PathBuf,
    },
    /// The record file or its folder is not one root alone may write,
    /// or could not be read.
    RecordUnreadable {
        /// The path the read named.
        path: PathBuf,
        /// Why it refused.
        reason: String,
    },
    /// The record could not be written.
    RecordUnwritable {
        /// The path the write named.
        path: PathBuf,
        /// Why it refused.
        reason: String,
    },
}

impl RecordCheckoutFailure {
    /// The refusal in words,
    /// opening with the words every root mode is reached through.
    pub fn message(&self) -> String {
        let prefix: String = format!("{}: ", root_modes_name());
        return match self {
            RecordCheckoutFailure::NotRoot { effective_user } => {
                return NotRootRefusal {
                    mode: "record-checkout",
                    nothing: "read or recorded",
                    because: "the record decides which checkout the route places a mount under, and an account that could write it could point the route at a checkout it never proved",
                    way: "run the mode as root, from the setup or by hand",
                    effective_user: *effective_user,
                }
                .sentence();
            }
            RecordCheckoutFailure::Configuration(refusal) => format!(
                "{prefix}the configuration was not read, so nothing was recorded: {}. Ways forward: fix what the refusal names and run the mode again; the record is written only after the container is proved.",
                refusal.message()
            ),
            RecordCheckoutFailure::Mount(refusal) => format!(
                "{prefix}the checkout's own mount was not read, so nothing was recorded: {}. Ways forward: check that the checkout dataset is mounted at that path, then run the mode again.",
                refusal.message()
            ),
            RecordCheckoutFailure::DatasetRead { dataset, failure } => format!(
                "{prefix}`{dataset}` could not be read from the kernel, so nothing was recorded: {}. Ways forward: check that the dataset exists in an imported pool, then run the mode again.",
                failure.message()
            ),
            RecordCheckoutFailure::NotAMountPoint { checkout, found } => format!(
                "{prefix}checkout-not-a-mount-point: {} {found}. Nothing was recorded. Ways forward: mount the checkout dataset at that path as its own `mountpoint`, then run the mode again.",
                checkout.display()
            ),
            RecordCheckoutFailure::NotOnZfs { checkout, filesystem_type } => format!(
                "{prefix}checkout-not-on-zfs: {} is on a `{filesystem_type}` mount, and this route serves ZFS. Nothing was recorded. Ways forward: run the mode on a checkout whose path is a ZFS dataset's own mountpoint, or leave the route off this checkout.",
                checkout.display()
            ),
            RecordCheckoutFailure::ContainerNotBesideTheCheckout { dataset, container, because } => format!(
                "{prefix}container-not-beside-the-checkout: `{container}` is not a dataset beside `{dataset}`: {because}. Nothing was recorded. Ways forward: give `{CLONE_MOUNTS}.{PARENT}` a dataset in the same pool that is neither inside the checkout nor holds it, then run the mode again.",
                CLONE_MOUNTS = crate::worktree_clone_mount_handler::sweep_configuration::CLONE_MOUNTS_KEY,
                PARENT = crate::worktree_clone_mount_handler::sweep_configuration::PARENT_KEY,
            ),
            RecordCheckoutFailure::RecordDiffers { recorded, kernel, field } => {
                return match field {
                    DifferingField::Guid { .. } => format!(
                        "{prefix}checkout-record-differs: the record's GUID is {recorded} and the kernel shows {kernel}. Nothing was written. Ways forward: run this command again with `--replace {kernel}` to record this dataset, or restore the dataset the record names, which answers the recorded GUID again."
                    ),
                    DifferingField::Path { .. } => format!(
                        "{prefix}checkout-record-differs: the record's checkout is {recorded} and this run names {kernel}. Nothing was written. Ways forward: run the mode for the checkout the record names, or remove the record after the checkout has moved on purpose and record the new path."
                    ),
                    DifferingField::Dataset { .. } => format!(
                        "{prefix}checkout-record-differs: the record's dataset is `{recorded}` and the kernel shows `{kernel}` mounted at the checkout. Nothing was written. Ways forward: restore the dataset the record names, or remove the record and record the checkout again once you know which dataset the route should serve."
                    ),
                    DifferingField::Container { .. } => format!(
                        "{prefix}checkout-record-differs: the record's container is `{recorded}` and the configuration names `{kernel}`. Nothing was written. Ways forward: give the configuration the container the record names, or remove the record and record the checkout again once the configuration holds the container you mean."
                    ),
                };
            }
            RecordCheckoutFailure::ReplaceTokenDiffers { kernel, typed } => format!(
                "{prefix}replace-token-differs: the kernel shows {kernel}, and `--replace` was given {typed}. Nothing was written. Ways forward: read the kernel's GUID from this refusal and pass it whole, or run the command without `--replace` to read the difference again."
            ),
            RecordCheckoutFailure::NothingToReplace { record } => format!(
                "{prefix}nothing-to-replace: `--replace` was given and {} holds no record. Nothing was written. Ways forward: run the command without `--replace`, which records the checkout.",
                record.display()
            ),
            RecordCheckoutFailure::RecordUnreadable { path, reason } => format!(
                "{prefix}checkout-record-unreadable: {} could not be read as the record this route writes ({reason}). Nothing was written. Ways forward: check that the file is root's regular file with no group or other write bit, or remove it after checking what it holds and record the checkout again.",
                path.display()
            ),
            RecordCheckoutFailure::RecordUnwritable { path, reason } => format!(
                "{prefix}checkout-record-unwritable: {} could not be written ({reason}). Nothing was written. Ways forward: check that {} is root's folder with no group or other write bit, then run the mode again.",
                path.display(),
                path.parent().map_or(String::from("the record's folder"), |folder| return folder.display().to_string())
            ),
        };
    }
}
