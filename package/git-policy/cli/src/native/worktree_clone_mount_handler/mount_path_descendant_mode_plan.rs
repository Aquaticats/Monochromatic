//! What:
//!  Every proof the `mount-path-descendant` mode makes before it acts,
//!  and the one of two answers a proved state gives:
//!  the record no longer matching the kernel,
//!  the container that moved,
//!  the checkout that is not mounted,
//!  the dataset outside the checkout or in another pool,
//!  the mountpoint that is none or holds something,
//!  and the attributes the dataset's own properties resolve to.
//! Why:
//!  The record is what makes "under the checkout" a proof rather than a guess:
//!  the dataset's mountpoint is read from the kernel and compared with the
//!  recorded checkout,
//!  and an `ExecStart=` re-run that finds the dataset already mounted answers
//!  without writing anything
//!  (`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "The two modes these answers add";
//!  `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`).
//!  The plan is the whole decision,
//!  so the walk and the mount are reached only with a mountpoint and attributes
//!  these proofs read,
//!  and it is their own file so the mount's placement stays short enough to
//!  read whole
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = planWith(places, { dataset: 'zpcachyos/nosnap/vm-builder-output' }, true, 0);
//! // { kind: 'already-mounted', ... } | { kind: 'plan', plan }
//! ```

use super::mount_path_descendant_mode_properties::{
    MOUNTPOINT_DASH,
    MOUNTPOINT_LEGACY,
    MOUNTPOINT_NONE,
    MOUNTPOINT_PROPERTY,
    OVERLAY_PROPERTY,
    SWITCHED_ON,
    attributes_of,
    refuse_unmountable,
};
use super::mount_path_descendant_mode_refusals::MountPathDescendantFailure;
use crate::worktree_clone_mount_handler::handler_mode_arguments::MountPathDescendantRequest;
use crate::worktree_clone_mount_handler::record_checkout_mode::{
    RecordedCheckout,
    read_record,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    value_of,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ReadFailure,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountedDatasetInstance,
    MountedDatasets,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::path::{
    Path,
    PathBuf,
};

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
///   The release path reads the installed record,
///  the kernel's tables and the
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
pub fn plan_with(
    places: &MountPlaces<'_>,
    request: &MountPathDescendantRequest,
    running_as_root: bool,
    effective_user: u32,
) -> Result<Answer, MountPathDescendantFailure> {
    if !running_as_root {
        return Err(MountPathDescendantFailure::NotRoot {
            effective_user,
        });
    }
    let recorded: Option<RecordedCheckout> =
        read_record(places.record.as_path()).map_err(MountPathDescendantFailure::Record)?;
    let Some(record) = recorded else {
        return Err(MountPathDescendantFailure::NoRecordedCheckout {
            record: places.record.clone(),
        });
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
                kernel: other.map_or_else(
                    || return checkout_facts.mountpoint.describe(MOUNTPOINT_PROPERTY),
                    String::from,
                ),
            });
        }
    }
    let container_facts: CloneObjectFacts = read_facts(places.datasets, record.container.as_str())?;
    let container_mountpoint: String = container_facts.mountpoint.text().map_or_else(
        || return container_facts.mountpoint.describe(MOUNTPOINT_PROPERTY),
        String::from,
    );
    if !the_container_is_beside(
        record.dataset.as_str(),
        record.container.as_str(),
        container_mountpoint.as_str(),
        record.checkout.as_path(),
    ) {
        return Err(MountPathDescendantFailure::RecordNoLongerMatches {
            field: String::from("container"),
            recorded: record.container.clone(),
            kernel: container_mountpoint,
        });
    }
    let rows: Vec<MountedDatasetInstance> = places.mounted.mounted_datasets().map_err(|said| {
        return MountPathDescendantFailure::DatasetRead {
            dataset: String::from("<the mount table>"),
            failure: ReadFailure::at(said),
        };
    })?;
    if !mounted_at(
        rows.as_slice(),
        record.dataset.as_str(),
        record.checkout.as_path(),
    ) {
        return Err(MountPathDescendantFailure::CheckoutNotMounted {
            checkout: record.checkout.clone(),
            dataset: record.dataset.clone(),
        });
    }
    let dataset: String = request.dataset.clone();
    if dataset == record.dataset {
        return Err(MountPathDescendantFailure::DatasetOutsideTheCheckout {
            dataset,
            mountpoint: record.checkout.clone(),
            checkout: record.checkout.clone(),
        });
    }
    let pool: String = pool_of(dataset.as_str());
    if pool != record.pool {
        return Err(MountPathDescendantFailure::DatasetInAnotherPool {
            dataset,
            pool,
            recorded: record.pool.clone(),
        });
    }
    let facts: CloneObjectFacts = read_facts(places.datasets, dataset.as_str())?;
    let Some(mountpoint_text) = facts.mountpoint.text() else {
        return Err(MountPathDescendantFailure::DatasetWithoutAMountpoint {
            dataset,
            mountpoint: facts.mountpoint.describe(MOUNTPOINT_PROPERTY),
        });
    };
    if matches!(
        mountpoint_text,
        MOUNTPOINT_NONE | MOUNTPOINT_DASH | MOUNTPOINT_LEGACY
    ) {
        return Err(MountPathDescendantFailure::DatasetWithoutAMountpoint {
            dataset,
            mountpoint: String::from(mountpoint_text),
        });
    }
    let mountpoint: PathBuf = PathBuf::from(mountpoint_text);
    if !is_strictly_under(mountpoint.as_path(), record.checkout.as_path()) {
        return Err(MountPathDescendantFailure::DatasetOutsideTheCheckout {
            dataset,
            mountpoint,
            checkout: record.checkout.clone(),
        });
    }
    if let Some(row) = rows.iter().find(|row| return row.dataset == dataset) {
        if row.mount_point == mountpoint {
            return Ok(Answer::AlreadyMountedAtItsOwnMountpoint {
                dataset,
                mountpoint,
            });
        }
        return Err(MountPathDescendantFailure::DatasetAlreadyMountedElsewhere {
            dataset,
            mounted_at: row.mount_point.clone(),
            mountpoint,
        });
    }
    refuse_unmountable(&dataset, &facts)?;
    if value_of(&facts.properties, OVERLAY_PROPERTY).text() != Some(SWITCHED_ON)
        && holds_anything(mountpoint.as_path())
    {
        return Err(MountPathDescendantFailure::MountpointNotEmpty {
            dataset,
            mountpoint,
        });
    }
    let attributes: u64 = attributes_of(&facts, places.datasets, &pool)?;
    return Ok(Answer::Plan(MountPlan {
        dataset,
        checkout: record.checkout,
        mountpoint,
        attributes,
    }));
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
fn is_strictly_under(
    inner: &Path,
    outer: &Path,
) -> bool {
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
fn mounted_at(
    rows: &[MountedDatasetInstance],
    dataset: &str,
    path: &Path,
) -> bool {
    return rows
        .iter()
        .any(|row| return row.dataset == dataset && fold(row.mount_point.as_path()) == fold(path));
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
fn the_container_is_beside(
    dataset: &str,
    container: &str,
    container_mountpoint: &str,
    checkout: &Path,
) -> bool {
    if container == dataset
        || container.starts_with(format!("{dataset}/").as_str())
        || dataset.starts_with(format!("{container}/").as_str())
    {
        return false;
    }
    return !is_strictly_under(Path::new(container_mountpoint), checkout);
}

/// One dataset's facts,
/// or the refusal naming the dataset and what the read answered.
fn read_facts(
    datasets: &dyn DatasetReader,
    dataset: &str,
) -> Result<CloneObjectFacts, MountPathDescendantFailure> {
    let identity: DatasetIdentity = datasets.identity(dataset).map_err(|failure| {
        return MountPathDescendantFailure::DatasetRead {
            dataset: String::from(dataset),
            failure,
        };
    })?;
    return Ok(
        crate::worktree_clone_mount_handler::sweep_dataset_facts::facts_of(dataset, &identity),
    );
}

/// The pool a dataset's name stands in.
fn pool_of(dataset: &str) -> String {
    return dataset.split('/').next().unwrap_or_default().to_string();
}
