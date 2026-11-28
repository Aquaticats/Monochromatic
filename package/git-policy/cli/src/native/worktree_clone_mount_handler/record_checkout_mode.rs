//! What:
//!  The `record-checkout` mode:
//!  the checkout's own mount,
//!  its dataset's name and GUID,
//!  the container the configuration names,
//!  the pool both are in,
//!  the record the mode writes,
//!  and the `--replace` that admits a dataset a legitimate restore moved.
//! Why:
//!  `mount-path-descendant` places a mount only under a checkout this record
//!  names,
//!  and every later step that names a checkout reads it,
//!  so the record is the one thing that makes "under the checkout" a proof
//!  rather than a guess
//!  (`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "The two modes these answers add";
//!  choice 9 of
//!  `doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`).
//!  The kernel is asked for every fact and nothing is taken from a written name:
//!  the mount table says which dataset the checkout is on,
//! the dataset's own
//!  reply says its GUID and mountpoint,
//! and the record is compared field by
//!  field with both before anything is written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = recordCheckout({ checkout: '/home/user/Monochromatic' }); // the recorded line, or a refusal
//! ```

use super::record_checkout_mode_paths_and_datasets::{
    pool_of,
    prove_the_container_beside,
    the_checkout_is_the_mounts_root,
};
use super::record_checkout_mode_record_comparison::write_or_agree;
use crate::worktree_clone_mount_handler::handler_install_paths::CHECKOUT_RECORD_PATH;
use crate::worktree_clone_mount_handler::handler_mode_arguments::RecordCheckoutRequest;
use crate::worktree_clone_mount_handler::sweep_checkout_mount::CheckoutMount;
use crate::worktree_clone_mount_handler::sweep_configuration::{
    SweepConfiguration,
    read_configuration,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    MOUNTPOINT_PROPERTY,
    facts_of,
    value_of,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    ReadFailure,
    ZfsDatasetReader,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use rustix::process::geteuid;
use std::path::PathBuf;

/// The record's own facts,
/// the field a later run finds different,
/// and why the mode recorded nothing.
pub use super::record_checkout_mode_record::{
    DifferingField,
    RECORD_FOLDER_MODE,
    RECORD_MODE,
    RECORD_VERSION,
    RecordCheckoutFailure,
    RecordedCheckout,
    WRITABLE_BY_OTHERS,
    WRITING_NAME,
};

/// The record's own read,
/// which every proof that names a checkout asks.
pub(super) use super::record_checkout_mode_record_file::read_record;

/// What:
///  Run the mode as the release runs it:
///  as root,
///  against the installed configuration and record,
///  and the kernel's own tables.
/// Why:
///  The release path takes the installed places and the live reads,
///  and every control drives `run_with` instead,
///  so this function is the only place the live ones are chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = run({ checkout: '/home/user/Monochromatic', replace: undefined });
/// ```
///
/// # Errors
/// The device could not be opened,
/// or everything `run_with` refuses.
pub fn run(request: &RecordCheckoutRequest) -> Result<String, RecordCheckoutFailure> {
    let effective_user = geteuid();
    let datasets: ZfsDatasetReader = ZfsDatasetReader::open().map_err(|error| {
        return RecordCheckoutFailure::DatasetRead {
            dataset: String::from("<the device>"),
            failure: ReadFailure::at(error.to_string()),
        };
    })?;
    let places: RecordCheckoutPlaces<'_> = RecordCheckoutPlaces {
        configuration: PathBuf::from(
            crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH,
        ),
        record: PathBuf::from(CHECKOUT_RECORD_PATH),
        mounts: &crate::worktree_clone_mount_handler::sweep_checkout_mount::LiveCheckoutMounts,
        mounted: &LiveMountedDatasets,
        datasets: &datasets,
    };
    return run_with(
        &places,
        request,
        effective_user.is_root(),
        effective_user.as_raw(),
    );
}

/// What:
///  Run the mode from the places,
/// the gate's answer and the caller it is given.
/// Why:
///   Every read and write is an argument,
///  so a control proves each refusal without root,
///  without a pool and without this machine's record;
///  the effective user is an argument for the same reason the probe's is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = runWith(places, request, true, 0);
/// ```
///
/// # Errors
/// Every refusal in `RecordCheckoutFailure`.
pub fn run_with(
    places: &RecordCheckoutPlaces<'_>,
    request: &RecordCheckoutRequest,
    running_as_root: bool,
    effective_user: u32,
) -> Result<String, RecordCheckoutFailure> {
    if !running_as_root {
        return Err(RecordCheckoutFailure::NotRoot {
            effective_user,
        });
    }
    let checkout: PathBuf = request.checkout.clone();
    let configuration: SweepConfiguration =
        read_configuration(places.configuration.as_path(), places.mounted)
            .map_err(RecordCheckoutFailure::Configuration)?;
    let mount: CheckoutMount = places
        .mounts
        .mount_holding(checkout.as_path())
        .map_err(RecordCheckoutFailure::Mount)?;
    let Some(mounted_dataset) = mount.dataset() else {
        return Err(RecordCheckoutFailure::NotOnZfs {
            checkout,
            filesystem_type: mount.filesystem_type,
        });
    };
    let dataset: String = String::from(mounted_dataset);
    if !the_checkout_is_the_mounts_root(checkout.as_path(), &mount) {
        return Err(RecordCheckoutFailure::NotAMountPoint {
            checkout,
            found: format!(
                "is on the mount of `{}` at {}",
                mount.source,
                mount.mount_point.display()
            ),
        });
    }
    let identity = places
        .datasets
        .identity(dataset.as_str())
        .map_err(|failure| {
            return RecordCheckoutFailure::DatasetRead {
                dataset: dataset.clone(),
                failure,
            };
        })?;
    let facts = facts_of(dataset.as_str(), &identity);
    let mountpoint: String = match value_of(&identity.properties, MOUNTPOINT_PROPERTY) {
        value if value.text() == Some(checkout.to_string_lossy().as_ref()) => {
            String::from(checkout.to_string_lossy().as_ref())
        }
        value => {
            return Err(RecordCheckoutFailure::NotAMountPoint {
                checkout,
                found: value.describe(MOUNTPOINT_PROPERTY),
            });
        }
    };
    let container: String = configuration.parent_dataset;
    prove_the_container_beside(dataset.as_str(), container.as_str())?;
    let pool: String = pool_of(dataset.as_str());
    if pool_of(container.as_str()) != pool {
        return Err(RecordCheckoutFailure::ContainerNotBesideTheCheckout {
            dataset,
            container,
            because: String::from("the two are in different pools"),
        });
    }
    let proved: RecordedCheckout = RecordedCheckout {
        version: RECORD_VERSION,
        checkout: PathBuf::from(mountpoint),
        dataset,
        guid: facts.guid.to_string(),
        container,
        pool,
    };
    return write_or_agree(places, &proved, request);
}

/// Where one run reads and writes,
/// so a control reaches it through this module as it did before the split.
pub use super::record_checkout_mode_places::RecordCheckoutPlaces;

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "record_checkout_mode_tests.rs"]
mod tests;
