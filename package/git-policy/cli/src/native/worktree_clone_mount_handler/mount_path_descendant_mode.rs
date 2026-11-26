//! What:
//!  The `mount-path-descendant` mode:
//!  the recorded checkout,
//!  one nested dataset's own mountpoint beneath it,
//!  the kernel's facts about both,
//!  the walk to the mountpoint through descriptors,
//!  and the mount itself,
//!  placed with the primitive the route already uses for a clone.
//! Why:
//!  The step 19 record replaces `/usr/bin/zfs mount <dataset>` with one mode
//!  line, so a boot places the nested dataset only when every proof holds
//!  (`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "The two modes these answers add";
//!  `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`).
//!  The record is what makes "under the checkout" a proof rather than a guess:
//!  the dataset's mountpoint is read from the kernel, compared with the
//!  recorded checkout, and mounted onto a folder the walk opened beneath the
//!  checkout's own mount,
//!  so a mount never lands on a path an account renamed.
//!  An `ExecStart=` re-run answers without writing anything,
//!  which is why the already-mounted answer stands before the walk.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = mountPathDescendant({ dataset: 'zpcachyos/nosnap/vm-builder-output' }); // the mounted line, or a refusal
//! ```

use crate::worktree_clone_mount::attach_the_clone::attach_with_attributes;
use crate::worktree_clone_mount_handler::handler_install_paths::{CHECKOUT_RECORD_PATH, CONFIGURATION_PATH};
use crate::worktree_clone_mount_handler::handler_mode_arguments::MountPathDescendantRequest;
use crate::worktree_clone_mount_handler::record_checkout_mode::{RecordedCheckout, read_record};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{CloneObjectFacts, value_of};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ReadFailure, ZfsDatasetReader};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{LiveMountedDatasets, MountedDatasetInstance, MountedDatasets};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use rustix::process::geteuid;
use std::path::{Path, PathBuf};

/// Why the mode mounted nothing,
/// one variant per cause,
/// and the words each answers.
pub use super::mount_path_descendant_mode_refusals::MountPathDescendantFailure;

/// The properties and values an explicit mount is refused by,
/// and the values they hold.
pub use super::mount_path_descendant_mode_properties::{
    ATIME_PROPERTY, CANMOUNT_OFF, CANMOUNT_PROPERTY, DEVICES_PROPERTY, ENCRYPTION_PROPERTY, EXEC_PROPERTY, INCONSISTENT_PROPERTY, KEY_STATUS_AVAILABLE, KEY_STATUS_PROPERTY, MOUNTPOINT_DASH,
    MOUNTPOINT_LEGACY, MOUNTPOINT_NONE, MOUNTPOINT_PROPERTY, NBMAND_PROPERTY, OVERLAY_PROPERTY, READONLY_PROPERTY, RECEIVE_RESUME_TOKEN_PROPERTY, REDACTED_PROPERTY, RELATIME_PROPERTY,
    SETUID_PROPERTY, SWITCHED_OFF, SWITCHED_ON, ZONED_PROPERTY,
};

use super::mount_path_descendant_mode_properties::{attributes_of, refuse_unmountable};

/// What:
///  The one mount this mode may place,
///  with the attributes the dataset's own properties resolve to.
/// Why:
///   Every proof has answered by the time a plan exists,
///  so the walk and the mount act on an object that was proved rather than on
///  a path that was read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountPlan = { dataset, checkout, mountpoint, attributes };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountPlan {
    /// The nested dataset's full name.
    pub dataset: String,
    /// The recorded checkout the mountpoint descends from.
    pub checkout: PathBuf,
    /// The dataset's own mountpoint,
    /// as the kernel's property gives it.
    pub mountpoint: PathBuf,
    /// The attributes `fsmount` is asked for.
    pub attributes: u64,
}

/// What:
///  What one run answers before it acts.
/// Why:
///  A re-run of a boot unit must write nothing,
///  so the already-mounted answer is a decision of its own rather than a step
///  the walk reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { kind: 'already-mounted' } | { kind: 'plan'; plan };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Answer {
    /// The kernel already has the dataset at its own mountpoint.
    AlreadyMountedAtItsOwnMountpoint {
        /// The dataset.
        dataset: String,
        /// The mountpoint it stands at.
        mountpoint: PathBuf,
    },
    /// Every proof answered and the mount may be placed.
    Plan(MountPlan),
}

/// What:
///  Where one `mount-path-descendant` run reads and mounts.
/// Why:
///   The release path reads the installed record, the kernel's tables and the
///  live device,
///  and a control hands in fixtures and scripted reads,
///  so no control reads this machine's table,
///  pool or record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountPlaces = { record, configuration, datasets, mounted };
/// ```
pub struct MountPlaces<'a> {
    /// The record the checkout is named in.
    pub record: PathBuf,
    /// The configuration the container is named in.
    pub configuration: PathBuf,
    /// The pool reads the dataset's own reply comes from.
    pub datasets: &'a dyn DatasetReader,
    /// The mount table rows the checkout and the dataset are read from.
    pub mounted: &'a dyn MountedDatasets,
}

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
    let datasets: ZfsDatasetReader = ZfsDatasetReader::open().map_err(|error| return MountPathDescendantFailure::DatasetRead { dataset: String::from("<the device>"), failure: ReadFailure::at(error.to_string()) })?;
    let places: MountPlaces<'_> = MountPlaces {
        record: PathBuf::from(CHECKOUT_RECORD_PATH),
        configuration: PathBuf::from(CONFIGURATION_PATH),
        datasets: &datasets,
        mounted: &LiveMountedDatasets,
    };
    return run_with(&places, request, effective_user.is_root(), effective_user.as_raw());
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
pub fn run_with(places: &MountPlaces<'_>, request: &MountPathDescendantRequest, running_as_root: bool, effective_user: u32) -> Result<String, MountPathDescendantFailure> {
    return match plan_with(places, request, running_as_root, effective_user)? {
        Answer::AlreadyMountedAtItsOwnMountpoint { dataset, mountpoint } => Ok(already_mounted_line(dataset.as_str(), mountpoint.as_path())),
        Answer::Plan(plan) => place_the_mount(&plan),
    };
}

/// What:
///  Every proof before any act,
///  and the one of two answers a proved state gives.
/// Why:
///  Step 2 reads dataset facts by name and no mount table,
///  which is what leaves `checkout-not-mounted` reachable when the checkout's
///  own mount is delayed,
///  and step 5 stands before the walk so a dataset the kernel already has
///  mounted is answered rather than mounted twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = planWith(places, request, true, 0); // { kind: 'already-mounted' } | { kind: 'plan'; plan }
/// ```
///
/// # Errors
/// Every refusal in `MountPathDescendantFailure`.
pub fn plan_with(places: &MountPlaces<'_>, request: &MountPathDescendantRequest, running_as_root: bool, effective_user: u32) -> Result<Answer, MountPathDescendantFailure> {
    if !running_as_root {
        return Err(MountPathDescendantFailure::NotRoot { effective_user });
    }
    let recorded: Option<RecordedCheckout> = read_record(places.record.as_path()).map_err(MountPathDescendantFailure::Record)?;
    let Some(record) = recorded else {
        return Err(MountPathDescendantFailure::NoRecordedCheckout { record: places.record.clone() });
    };
    let checkout_facts: CloneObjectFacts = read_facts(places.datasets, record.dataset.as_str())?;
    if record.guid != checkout_facts.guid.to_string() {
        return Err(MountPathDescendantFailure::RecordNoLongerMatches {
            field: String::from("GUID"),
            recorded: record.guid.clone(),
            kernel: checkout_facts.guid.to_string(),
        });
    }
    match checkout_facts.mountpoint.text() {
        Some(text) if Path::new(text) == record.checkout.as_path() => {}
        other => {
            return Err(MountPathDescendantFailure::RecordNoLongerMatches {
                field: String::from("mountpoint"),
                recorded: record.checkout.display().to_string(),
                kernel: other.map_or_else(|| return checkout_facts.mountpoint.describe(MOUNTPOINT_PROPERTY), String::from),
            });
        }
    }
    let container_facts: CloneObjectFacts = read_facts(places.datasets, record.container.as_str())?;
    let container_mountpoint: String = container_facts.mountpoint.text().map_or_else(|| return container_facts.mountpoint.describe(MOUNTPOINT_PROPERTY), String::from);
    if !the_container_is_beside(record.dataset.as_str(), record.container.as_str(), container_mountpoint.as_str(), record.checkout.as_path()) {
        return Err(MountPathDescendantFailure::RecordNoLongerMatches {
            field: String::from("container"),
            recorded: record.container.clone(),
            kernel: container_mountpoint,
        });
    }
    let rows: Vec<MountedDatasetInstance> = places.mounted.mounted_datasets().map_err(|said| return MountPathDescendantFailure::DatasetRead { dataset: String::from("<the mount table>"), failure: ReadFailure::at(said) })?;
    if !mounted_at(rows.as_slice(), record.dataset.as_str(), record.checkout.as_path()) {
        return Err(MountPathDescendantFailure::CheckoutNotMounted { checkout: record.checkout.clone(), dataset: record.dataset.clone() });
    }
    let dataset: String = request.dataset.clone();
    if dataset == record.dataset {
        return Err(MountPathDescendantFailure::DatasetOutsideTheCheckout { dataset, mountpoint: record.checkout.clone(), checkout: record.checkout.clone() });
    }
    let pool: String = pool_of(dataset.as_str());
    if pool != record.pool {
        return Err(MountPathDescendantFailure::DatasetInAnotherPool { dataset, pool, recorded: record.pool.clone() });
    }
    let facts: CloneObjectFacts = read_facts(places.datasets, dataset.as_str())?;
    let Some(mountpoint_text) = facts.mountpoint.text() else {
        return Err(MountPathDescendantFailure::DatasetWithoutAMountpoint { dataset, mountpoint: facts.mountpoint.describe(MOUNTPOINT_PROPERTY) });
    };
    if matches!(mountpoint_text, MOUNTPOINT_NONE | MOUNTPOINT_DASH | MOUNTPOINT_LEGACY) {
        return Err(MountPathDescendantFailure::DatasetWithoutAMountpoint { dataset, mountpoint: String::from(mountpoint_text) });
    }
    let mountpoint: PathBuf = PathBuf::from(mountpoint_text);
    if !is_strictly_under(mountpoint.as_path(), record.checkout.as_path()) {
        return Err(MountPathDescendantFailure::DatasetOutsideTheCheckout { dataset, mountpoint, checkout: record.checkout.clone() });
    }
    if let Some(row) = rows.iter().find(|row| return row.dataset == dataset) {
        if row.mount_point == mountpoint {
            return Ok(Answer::AlreadyMountedAtItsOwnMountpoint { dataset, mountpoint });
        }
        return Err(MountPathDescendantFailure::DatasetAlreadyMountedElsewhere { dataset, mounted_at: row.mount_point.clone(), mountpoint });
    }
    refuse_unmountable(&dataset, &facts)?;
    if value_of(&facts.properties, OVERLAY_PROPERTY).text() != Some(SWITCHED_ON) && holds_anything(mountpoint.as_path()) {
        return Err(MountPathDescendantFailure::MountpointNotEmpty { dataset, mountpoint });
    }
    let attributes: u64 = attributes_of(&facts, places.datasets, &pool)?;
    return Ok(Answer::Plan(MountPlan { dataset, checkout: record.checkout, mountpoint, attributes }));
}

/// What:
///  The walk from the checkout's opened mount to the mountpoint,
///  and the mount placed on the folder the walk opened.
/// Why:
///   The mount lands on the descriptor the walk held,
///  never on a path,
///  so a folder an account renamed onto the path receives nothing,
///  and a missing last component is created while a missing intermediate one
///  refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = placeTheMount({ dataset, checkout, mountpoint, attributes });
/// ```
///
/// # Errors
/// `MountpointUnusable` when the walk refuses,
/// and `MountFailed` when the primitive does.
fn place_the_mount(plan: &MountPlan) -> Result<String, MountPathDescendantFailure> {
    let root: std::fs::File = crate::subvolume::fs_beneath::open_root(plan.checkout.as_path()).map_err(|error| return MountPathDescendantFailure::MountpointUnusable { dataset: plan.dataset.clone(), mountpoint: plan.mountpoint.clone(), because: error.to_string() })?;
    let names = relative_names(plan.checkout.as_path(), plan.mountpoint.as_path());
    let Some((last, between)) = names.split_last() else {
        return Err(MountPathDescendantFailure::MountpointUnusable { dataset: plan.dataset.clone(), mountpoint: plan.mountpoint.clone(), because: String::from("the mountpoint is the checkout itself") });
    };
    let held: Vec<std::fs::File> = crate::subvolume::fs_beneath_walk_chain::open_chain_beneath(&root, between).map_err(|refusal| return MountPathDescendantFailure::MountpointUnusable { dataset: plan.dataset.clone(), mountpoint: plan.mountpoint.clone(), because: describe_chain_refusal(&refusal) })?;
    let Some(parent) = held.last() else {
        return Err(MountPathDescendantFailure::MountpointUnusable { dataset: plan.dataset.clone(), mountpoint: plan.mountpoint.clone(), because: String::from("the walk held no descriptor") });
    };
    let folder: std::fs::File = match crate::subvolume::fs_beneath_walk_chain::open_directory_at_a_mount(parent, last.as_os_str()) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => create_and_open(parent, last.as_os_str()).map_err(|why| return MountPathDescendantFailure::MountpointUnusable { dataset: plan.dataset.clone(), mountpoint: plan.mountpoint.clone(), because: why })?,
        Err(error) => return Err(MountPathDescendantFailure::MountpointUnusable { dataset: plan.dataset.clone(), mountpoint: plan.mountpoint.clone(), because: error.to_string() }),
    };
    attach_with_attributes(plan.dataset.as_str(), &folder, plan.attributes).map_err(|refusal| return MountPathDescendantFailure::MountFailed { dataset: plan.dataset.clone(), refusal })?;
    return Ok(mounted_line(plan.dataset.as_str(), plan.mountpoint.as_path()));
}

/// What:
///  One walk refusal in words a person can act on,
///  naming the component that stopped the walk.
/// Why:
///  The walk's own type carries the component and the system's error,
///  and a mountpoint refusal that said only "the walk refused" would send a
///  person to the whole path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = describeChainRefusal({ kind: 'link', step: 1, component: 'worktrees' });
/// ```
fn describe_chain_refusal(refusal: &crate::subvolume::fs_beneath_walk_chain::ChainRefusal) -> String {
    use crate::subvolume::fs_beneath_walk_chain::ChainRefusal;
    return match refusal {
        ChainRefusal::Link { step, component } => format!("component {} (`{}`) is a link, and the walk follows none", step + 1, component.to_string_lossy()),
        ChainRefusal::Boundary { step, component } => format!("component {} (`{}`) is a mount point or a folder on another device", step + 1, component.to_string_lossy()),
        ChainRefusal::Unreadable { step, error } => format!("component {} could not be opened: {error}", step + 1),
    };
}

/// What:
///  Create a missing last component as a folder only root may write and reopen
///  it without following a link.
/// Why:
///   The mode creates the mountpoint and never a tree above it,
///  so only the last component is ever made,
///  and the descriptor acts on the folder it opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = mkdirAndOpen(parent, 'output');
/// ```
fn create_and_open(parent: &std::fs::File, name: &std::ffi::OsStr) -> Result<std::fs::File, String> {
    rustix::fs::mkdirat(parent, name, rustix::fs::Mode::from_bits_truncate(0o755)).map_err(|error| return format!("the last component could not be created: {error}"))?;
    return rustix::fs::openat(parent, name, rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
        .map(std::fs::File::from)
        .map_err(|error| return format!("the folder could not be reopened: {error}"));
}

/// The components of a mountpoint below a checkout,
/// as names,
/// which is what the walk takes.
fn relative_names(checkout: &Path, mountpoint: &Path) -> Vec<std::ffi::OsString> {
    let relative: &Path = mountpoint.strip_prefix(checkout).unwrap_or(mountpoint);
    return relative.components().map(|component| return component.as_os_str().to_os_string()).collect();
}

/// What:
///  Whether one path is strictly below another.
/// Why:
///   The dataset's mountpoint must descend from the checkout and must not be the
///  checkout itself,
///  which is what makes a mount land inside a mount the route can place it
///  under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inside = isStrictlyUnder('/home/user/Monochromatic/package/output', '/home/user/Monochromatic');
/// ```
fn is_strictly_under(inner: &Path, outer: &Path) -> bool {
    let folded_inner: PathBuf = fold(inner);
    let folded_outer: PathBuf = fold(outer);
    return folded_inner != folded_outer && folded_inner.starts_with(folded_outer);
}

/// A path's text with its `.` and `..` components resolved and its repeated
/// slashes folded,
/// without asking the system.
fn fold(path: &Path) -> PathBuf {
    let mut folded: PathBuf = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                let _dropped = folded.pop();
            }
            other => folded.push(other.as_os_str()),
        }
    }
    return folded;
}

/// Whether a folder holds anything at all,
/// which a mount hides when the dataset does not overlay.
fn holds_anything(folder: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return false;
    };
    return entries.take(1).next().is_some();
}

/// Whether the mount table shows the dataset mounted at the given path now.
fn mounted_at(rows: &[MountedDatasetInstance], dataset: &str, path: &Path) -> bool {
    return rows.iter().any(|row| return row.dataset == dataset && fold(row.mount_point.as_path()) == fold(path));
}

/// What:
///  Whether the container the record names is still a dataset beside the
///  checkout.
/// Why:
///   The record's container is what the sweep places clones under,
///  and a container that moved inside the checkout or away from it makes the
///  record stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const beside = theContainerIsBeside(dataset, container, containerMountpoint, checkout);
/// ```
fn the_container_is_beside(dataset: &str, container: &str, container_mountpoint: &str, checkout: &Path) -> bool {
    if container == dataset || container.starts_with(format!("{dataset}/").as_str()) || dataset.starts_with(format!("{container}/").as_str()) {
        return false;
    }
    return !is_strictly_under(Path::new(container_mountpoint), checkout);
}


/// One dataset's facts,
/// or the refusal naming the dataset and what the read answered.
fn read_facts(datasets: &dyn DatasetReader, dataset: &str) -> Result<CloneObjectFacts, MountPathDescendantFailure> {
    let identity: DatasetIdentity = datasets.identity(dataset).map_err(|failure| return MountPathDescendantFailure::DatasetRead { dataset: String::from(dataset), failure })?;
    return Ok(crate::worktree_clone_mount_handler::sweep_dataset_facts::facts_of(dataset, &identity));
}

/// The pool a dataset's name stands in.
fn pool_of(dataset: &str) -> String {
    return dataset.split('/').next().unwrap_or_default().to_string();
}

/// The line a run that placed a mount answers.
fn mounted_line(dataset: &str, mountpoint: &Path) -> String {
    return format!("mounted: {dataset} at {}", mountpoint.display());
}

/// The line a run that found the dataset already mounted answers.
fn already_mounted_line(dataset: &str, mountpoint: &Path) -> String {
    return format!("already mounted: {dataset} at {}", mountpoint.display());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_path_descendant_mode_tests.rs"]
mod tests;
