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

use crate::worktree_clone_mount::attach_the_clone::{AttachRefusal, attach_with_attributes};
use crate::worktree_clone_mount_handler::handler_install_paths::{CHECKOUT_RECORD_PATH, CONFIGURATION_PATH, root_modes_name};
use crate::worktree_clone_mount_handler::handler_mode_arguments::MountPathDescendantRequest;
use crate::worktree_clone_mount_handler::record_checkout_mode::{RecordCheckoutFailure, RecordedCheckout, read_record};
use crate::worktree_clone_mount_handler::root_mode_caller::NotRootRefusal;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts, PropertyValue, value_of,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ReadFailure, ZfsDatasetReader};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{LiveMountedDatasets, MountedDatasetInstance, MountedDatasets};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use rustix::process::geteuid;
use std::path::{Path, PathBuf};

/// The properties whose values decide whether an explicit mount succeeds,
/// which upstream's `share_mount_one` refuses one by one.
pub const CANMOUNT_PROPERTY: &str = "canmount";
/// The property that says the dataset belongs to a non-global zone.
pub const ZONED_PROPERTY: &str = "zoned";
/// The property holding how much of a received stream is redacted.
pub const REDACTED_PROPERTY: &str = "redacted";
/// The property that says a receive is unfinished.
pub const INCONSISTENT_PROPERTY: &str = "inconsistent";
/// The property holding a partially received stream's resume token.
pub const RECEIVE_RESUME_TOKEN_PROPERTY: &str = "receive_resume_token";
/// The property naming the folder a dataset mounts at.
pub const MOUNTPOINT_PROPERTY: &str = "mountpoint";
/// The property that allows one mount above another.
pub const OVERLAY_PROPERTY: &str = "overlay";
/// The property naming the cipher a dataset's data is encrypted with.
pub const ENCRYPTION_PROPERTY: &str = "encryption";
/// The property that says whether an encrypted dataset's key is loaded.
pub const KEY_STATUS_PROPERTY: &str = "keystatus";
/// The property that says a mount of the dataset carries no access-time writes.
pub const ATIME_PROPERTY: &str = "atime";
/// The property that says access times are updated relative to the modification.
pub const RELATIME_PROPERTY: &str = "relatime";
/// The property that says a mount of the dataset carries device nodes.
pub const DEVICES_PROPERTY: &str = "devices";
/// The property that says a mount of the dataset may run its files.
pub const EXEC_PROPERTY: &str = "exec";
/// The property that says a mount of the dataset is read-only.
pub const READONLY_PROPERTY: &str = "readonly";
/// The property that says a mount of the dataset carries set-user-ID execution.
pub const SETUID_PROPERTY: &str = "setuid";
/// The property that has no Linux mount attribute and is read as the no-op it
/// is.
pub const NBMAND_PROPERTY: &str = "nbmand";
/// The value `canmount` holds when nothing may mount the dataset itself.
pub const CANMOUNT_OFF: &str = "off";
/// The value a property holds when it is switched on.
pub const SWITCHED_ON: &str = "on";
/// The value `keystatus` holds when the key is loaded.
pub const KEY_STATUS_AVAILABLE: &str = "available";
/// The value a property holds when it is switched off.
pub const SWITCHED_OFF: &str = "off";
/// The value `mountpoint` holds when nothing may mount the dataset by name.
pub const MOUNTPOINT_LEGACY: &str = "legacy";
/// The value `mountpoint` holds when nobody mounts the dataset.
pub const MOUNTPOINT_NONE: &str = "none";
/// The value `mountpoint` holds when the dataset has no mountpoint at all.
pub const MOUNTPOINT_DASH: &str = "-";

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
            MountPathDescendantFailure::NotRoot { effective_user } => {
                return NotRootRefusal {
                    mode: "mount-path-descendant",
                    nothing: "read, mounted or recorded",
                    because: "a mount this mode places stands in the one mount namespace of the process that placed it, and an account that could place one inside a served checkout could mount anything there",
                    way: "run the mode as root, from the boot unit or by hand",
                    effective_user: *effective_user,
                }
                .sentence();
            }
            MountPathDescendantFailure::NoRecordedCheckout { record } => format!(
                "{prefix}no-recorded-checkout: {} holds no record, so nothing says which checkout this dataset's mountpoint descends from. Nothing was mounted. Ways forward: run `{} record-checkout --checkout <path>` as root, then run the mode again, or let the record's repair run from the value the boot printed.",
                record.display(),
                root_modes_name()
            ),
            MountPathDescendantFailure::Record(refusal) => refusal.message(),
            MountPathDescendantFailure::RecordNoLongerMatches { field, recorded, kernel } => format!(
                "{prefix}checkout-record-no-longer-matches: the record's {field} is {recorded} and the kernel shows {kernel}. Nothing was mounted. Ways forward: run `{} record-checkout --checkout <path> --replace {kernel}` as root when the record only moved its GUID, or restore the dataset the record names; until then this dataset stays unmounted.",
                root_modes_name()
            ),
            MountPathDescendantFailure::DatasetRead { dataset, failure } => format!(
                "{prefix}`{dataset}` could not be read from the kernel, so nothing was mounted: {}. Ways forward: check that the dataset exists in an imported pool, then run the mode again.",
                failure.message()
            ),
            MountPathDescendantFailure::CheckoutNotMounted { checkout, dataset } => format!(
                "{prefix}checkout-not-mounted: the mount table does not show `{dataset}` mounted at {} now, so the folder this dataset would be placed under is not a mount the route verified. Nothing was mounted. Ways forward: check that the checkout's own mount is in place (the boot's checkout unit, or `zfs mount <the checkout dataset>`), then run the mode again.",
                checkout.display()
            ),
            MountPathDescendantFailure::DatasetOutsideTheCheckout { dataset, mountpoint, checkout } => format!(
                "{prefix}dataset-outside-the-checkout: `{dataset}` mounts at {}, which is not strictly under the recorded checkout {}. Nothing was mounted. Ways forward: give the dataset a `mountpoint` under the checkout, or mount it with the unit that owned it before this mode.",
                mountpoint.display(),
                checkout.display()
            ),
            MountPathDescendantFailure::DatasetInAnotherPool { dataset, pool, recorded } => format!(
                "{prefix}dataset-in-another-pool: `{dataset}` is in pool `{pool}` and the recorded checkout is in pool `{recorded}`, so this dataset would put another pool's subtree inside a served checkout. Nothing was mounted. Ways forward: name a dataset in the recorded checkout's own pool, or serve the checkout whose pool holds it.",
            ),
            MountPathDescendantFailure::DatasetWithoutAMountpoint { dataset, mountpoint } => format!(
                "{prefix}dataset-without-a-mountpoint: `{dataset}`'s `mountpoint` is `{mountpoint}`, so the dataset is mounted by someone else's rule and this mode would be guessing. Nothing was mounted. Ways forward: give the dataset a `mountpoint` under the checkout, then run the mode again."
            ),
            MountPathDescendantFailure::DatasetAlreadyMountedElsewhere { dataset, mounted_at, mountpoint } => format!(
                "{prefix}dataset-already-mounted-elsewhere: the kernel has `{dataset}` mounted at {}, and its own `mountpoint` is {}. Nothing was mounted. Ways forward: unmount the dataset where it stands, or give its property the path it is mounted at, then run the mode again.",
                mounted_at.display(),
                mountpoint.display()
            ),
            MountPathDescendantFailure::DatasetNotMountable { dataset, property, value, because } => format!(
                "{prefix}dataset-not-mountable: `{dataset}`'s `{property}` is `{value}`, and {because}. Nothing was mounted. Ways forward: clear the property the refusal names, or let the unit that owned this dataset before this mode mount it."
            ),
            MountPathDescendantFailure::EncryptionKeyNotLoaded { dataset, keystatus } => format!(
                "{prefix}encryption-key-not-loaded: `{dataset}` is encrypted and its `keystatus` is `{keystatus}`. Nothing was mounted. Ways forward: load the key by hand (`zfs load-key <dataset>`), then run the mode again; a boot unit has nobody to prompt."
            ),
            MountPathDescendantFailure::MountpointNotEmpty { dataset, mountpoint } => format!(
                "{prefix}mountpoint-not-empty: {} holds files and `{dataset}`'s `overlay` is off, so a mount there would hide them. Nothing was mounted. Ways forward: empty the folder, or give the dataset `overlay=on` when hiding what it holds is intended.",
                mountpoint.display()
            ),
            MountPathDescendantFailure::MountpointUnusable { dataset, mountpoint, because } => format!(
                "{prefix}mountpoint-unusable: the walk to {} refused for `{dataset}`: {because}. Nothing was mounted. Ways forward: check the folder's components as the refusal names them, then run the mode again; this mode creates the last component and no tree above it.",
                mountpoint.display()
            ),
            MountPathDescendantFailure::MountFailed { dataset, refusal } => format!(
                "{prefix}the mount of `{dataset}` refused at {}: {}. Nothing was mounted. Ways forward: read the step the refusal names, fix what it reports, then run the mode again.",
                refusal.call.call_name(),
                refusal.message()
            ),
        };
    }
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

/// What:
///  Refuse an explicit mount of a dataset upstream refuses,
///  naming the property and its value.
/// Why:
///  An explicit `zfs mount` fails loudly for each of these and returns 1,
///  and a unit that reported success while mounting nothing is the failure this
///  refusal replaces;
///  the receive-token case also names the resume a person can run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseUnmountable('pool/dataset', facts); // throws dataset-not-mountable
/// ```
///
/// # Errors
/// `DatasetNotMountable` for each property that refuses a mount.
fn refuse_unmountable(dataset: &str, facts: &CloneObjectFacts) -> Result<(), MountPathDescendantFailure> {
    if value_of(&facts.properties, CANMOUNT_PROPERTY).text() == Some(CANMOUNT_OFF) {
        return Err(MountPathDescendantFailure::DatasetNotMountable {
            dataset: String::from(dataset),
            property: String::from(CANMOUNT_PROPERTY),
            value: String::from(CANMOUNT_OFF),
            because: String::from("an explicit mount of a `canmount=off` dataset refuses rather than mounting it"),
        });
    }
    if value_of(&facts.properties, ZONED_PROPERTY).text() == Some(SWITCHED_ON) {
        return Err(MountPathDescendantFailure::DatasetNotMountable {
            dataset: String::from(dataset),
            property: String::from(ZONED_PROPERTY),
            value: String::from(SWITCHED_ON),
            because: String::from("a dataset exported to a local zone is mounted by that zone and an explicit mount in the global zone refuses"),
        });
    }
    let redacted: PropertyValue = value_of(&facts.properties, REDACTED_PROPERTY);
    if redacted.number().is_some_and(|count| return count != 0) {
        return Err(MountPathDescendantFailure::DatasetNotMountable {
            dataset: String::from(dataset),
            property: String::from(REDACTED_PROPERTY),
            value: redacted.describe(REDACTED_PROPERTY),
            because: String::from("a redacted dataset is not mounted by an explicit mount"),
        });
    }
    let inconsistent: bool = value_of(&facts.properties, INCONSISTENT_PROPERTY).number() == Some(1);
    let token: PropertyValue = value_of(&facts.properties, RECEIVE_RESUME_TOKEN_PROPERTY);
    if inconsistent && token.text().is_some_and(|text| return !text.is_empty()) {
        return Err(MountPathDescendantFailure::DatasetNotMountable {
            dataset: String::from(dataset),
            property: String::from(INCONSISTENT_PROPERTY),
            value: String::from("1"),
            because: String::from("a partially received dataset is refused by an explicit mount, and its receive can be resumed with `zfs send -t <the token>`"),
        });
    }
    let encryption: PropertyValue = value_of(&facts.properties, ENCRYPTION_PROPERTY);
    let keystatus: PropertyValue = value_of(&facts.properties, KEY_STATUS_PROPERTY);
    if encryption.text().is_some_and(|cipher| return cipher != SWITCHED_OFF) && keystatus.text() != Some(KEY_STATUS_AVAILABLE) {
        return Err(MountPathDescendantFailure::EncryptionKeyNotLoaded {
            dataset: String::from(dataset),
            keystatus: keystatus.text().map_or_else(|| return keystatus.describe(KEY_STATUS_PROPERTY), String::from),
        });
    }
    return Ok(());
}

/// What:
///  The `fsmount` attributes the dataset's own properties resolve to,
///  in upstream's order.
/// Why:
///  Upstream derives one option from each pair and skips the `relatime` pair
///  when `atime` is off,
///  and a pool imported read-only forces every mount read-only;
///  `nbmand` has no Linux attribute and is read as the no-op it is,
///  which the mode states rather than passing an option nothing reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributes = attributesOf(facts, reader, 'zpcachyos'); // MOUNT_ATTR_NOATIME
/// ```
///
/// # Errors
/// The pool's own `readonly` could not be read.
fn attributes_of(facts: &CloneObjectFacts, datasets: &dyn DatasetReader, pool: &str) -> Result<u64, MountPathDescendantFailure> {
    let mut attributes: u64 = 0;
    let atime_on: bool = value_of(&facts.properties, ATIME_PROPERTY).text() == Some(SWITCHED_ON);
    let relatime_on: bool = value_of(&facts.properties, RELATIME_PROPERTY).text() != Some(SWITCHED_OFF);
    if !atime_on {
        attributes |= libc::MOUNT_ATTR_NOATIME;
    } else if !relatime_on {
        attributes |= libc::MOUNT_ATTR_STRICTATIME;
    }
    if value_of(&facts.properties, DEVICES_PROPERTY).text() == Some(SWITCHED_OFF) {
        attributes |= libc::MOUNT_ATTR_NODEV;
    }
    if value_of(&facts.properties, EXEC_PROPERTY).text() == Some(SWITCHED_OFF) {
        attributes |= libc::MOUNT_ATTR_NOEXEC;
    }
    if value_of(&facts.properties, SETUID_PROPERTY).text() == Some(SWITCHED_OFF) {
        attributes |= libc::MOUNT_ATTR_NOSUID;
    }
    let identity: DatasetIdentity = datasets.identity(pool).map_err(|failure| return MountPathDescendantFailure::DatasetRead { dataset: String::from(pool), failure })?;
    let pool_readonly: bool = value_of(&identity.properties, READONLY_PROPERTY).text() == Some(SWITCHED_ON);
    if pool_readonly || value_of(&facts.properties, READONLY_PROPERTY).text() == Some(SWITCHED_ON) {
        attributes |= libc::MOUNT_ATTR_RDONLY;
    }
    return Ok(attributes);
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
