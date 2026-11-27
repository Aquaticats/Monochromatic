//! What:
//!  The `mount-path-descendant` mode's own entry:
//!  where one run reads,
//!  the two answers a proved state gives,
//!  and the line a re-run that finds the dataset already mounted answers.
//! Why:
//!  Step 19's record replaces `/usr/bin/zfs mount <dataset>` with one mode line,
//!  so a boot places the nested dataset only when every proof holds
//!  (`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "The two modes these answers add";
//!  `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`).
//!  Every control drives `plan_with` or `run_with`,
//!  so this file is the only place the live reads are chosen,
//!  and an `ExecStart=` re-run answers without writing anything.
//!  The proofs are `mount_path_descendant_mode_plan.rs`'s and the walk and the
//!  mount are `mount_path_descendant_mount_placement.rs`'s,
//!  so each of the three can be read whole on its own
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = mountPathDescendant({ dataset: 'zpcachyos/nosnap/vm-builder-output' }); // the mounted line, or a refusal
//! ```

use crate::worktree_clone_mount_handler::handler_install_paths::{
    CHECKOUT_RECORD_PATH,
    CONFIGURATION_PATH,
};
use crate::worktree_clone_mount_handler::handler_mode_arguments::MountPathDescendantRequest;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    ReadFailure,
    ZfsDatasetReader,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use rustix::process::geteuid;
use std::path::{
    Path,
    PathBuf,
};

/// Why the mode mounted nothing,
/// one variant per cause,
/// and the words each answers.
pub use super::mount_path_descendant_mode_refusals::MountPathDescendantFailure;

/// The properties and values an explicit mount is refused by,
/// and the values they hold.
pub use super::mount_path_descendant_mode_properties::{
    ATIME_PROPERTY,
    CANMOUNT_OFF,
    CANMOUNT_PROPERTY,
    DEVICES_PROPERTY,
    ENCRYPTION_PROPERTY,
    EXEC_PROPERTY,
    INCONSISTENT_PROPERTY,
    KEY_STATUS_AVAILABLE,
    KEY_STATUS_PROPERTY,
    MOUNTPOINT_DASH,
    MOUNTPOINT_LEGACY,
    MOUNTPOINT_NONE,
    MOUNTPOINT_PROPERTY,
    NBMAND_PROPERTY,
    OVERLAY_PROPERTY,
    READONLY_PROPERTY,
    RECEIVE_RESUME_TOKEN_PROPERTY,
    REDACTED_PROPERTY,
    RELATIME_PROPERTY,
    SETUID_PROPERTY,
    SWITCHED_OFF,
    SWITCHED_ON,
    ZONED_PROPERTY,
};

/// The one mount this mode may place,
/// and the two answers a proved state gives,
/// which a control names by this file.
pub use super::mount_path_descendant_mode_plan::{
    Answer,
    MountPlaces,
    MountPlan,
    plan_with,
};

use super::mount_path_descendant_mount_placement::place_the_mount;

/// What:
///  Run the mode as the release runs it,
///  with the installed record and the kernel's own reads.
/// Why:
///  Every control drives `plan_with` or `run_with`,
///  so this function is the only place the live reads are chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = run({ dataset: 'zpcachyos/nosnap/vm-builder-output' });
/// ```
///
/// # Errors
/// The device could not be opened,
/// or everything `run_with` refuses.
pub fn run(request: &MountPathDescendantRequest) -> Result<String, MountPathDescendantFailure> {
    let effective_user = geteuid();
    let datasets: ZfsDatasetReader = ZfsDatasetReader::open().map_err(|error| {
        return MountPathDescendantFailure::DatasetRead {
            dataset: String::from("<the device>"),
            failure: ReadFailure::at(error.to_string()),
        };
    })?;
    let places: MountPlaces<'_> = MountPlaces {
        record: PathBuf::from(CHECKOUT_RECORD_PATH),
        configuration: PathBuf::from(CONFIGURATION_PATH),
        datasets: &datasets,
        mounted: &LiveMountedDatasets,
    };
    return run_with(
        &places,
        request,
        effective_user.is_root(),
        effective_user.as_raw(),
    );
}

/// What:
///  Decide what this run answers,
///  and place the mount only when every proof answered.
/// Why:
///  The plan is the whole decision,
///  so the walk and the mount are reached only with a mountpoint and attributes
///  that were proved,
///  and a re-run answers the already-mounted line and writes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = runWith(places, request, true, 0);
/// ```
///
/// # Errors
/// Every refusal in `MountPathDescendantFailure`.
pub fn run_with(
    places: &MountPlaces<'_>,
    request: &MountPathDescendantRequest,
    running_as_root: bool,
    effective_user: u32,
) -> Result<String, MountPathDescendantFailure> {
    return match plan_with(places, request, running_as_root, effective_user)? {
        Answer::AlreadyMountedAtItsOwnMountpoint {
            dataset,
            mountpoint,
        } => Ok(already_mounted_line(dataset.as_str(), mountpoint.as_path())),
        Answer::Plan(plan) => place_the_mount(&plan),
    };
}

/// The line a run that found the dataset already mounted answers.
fn already_mounted_line(
    dataset: &str,
    mountpoint: &Path,
) -> String {
    return format!("already mounted: {dataset} at {}", mountpoint.display());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_path_descendant_mode_tests.rs"]
mod tests;
