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
//!  the mount table says which dataset the checkout is on, the dataset's own
//!  reply says its GUID and mountpoint, and the record is compared field by
//!  field with both before anything is written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = recordCheckout({ checkout: '/home/user/Monochromatic' }); // the recorded line, or a refusal
//! ```

use crate::worktree_clone_mount_handler::handler_install_paths::CHECKOUT_RECORD_PATH;
use crate::worktree_clone_mount_handler::handler_mode_arguments::RecordCheckoutRequest;
use crate::worktree_clone_mount_handler::sweep_checkout_mount::{CheckoutMount, CheckoutMounts};
use crate::worktree_clone_mount_handler::sweep_configuration::{SweepConfiguration, read_configuration};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{facts_of, value_of, MOUNTPOINT_PROPERTY};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ReadFailure, ZfsDatasetReader};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{LiveMountedDatasets, MountedDatasets};
use rustix::process::geteuid;
use std::path::{Path, PathBuf};

/// The record's own facts,
/// the field a later run finds different,
/// and why the mode recorded nothing.
#[path = "record_checkout_mode_record.rs"]
mod record;

/// The record as the file it is:
/// the read,
/// the write,
/// its folder's proofs and the three lines an answer prints.
#[path = "record_checkout_mode_record_file.rs"]
mod record_file;

/// The record's version,
/// the modes its folder and its file are made with and the name it is written
/// under while it is replaced.
pub use self::record::{RECORD_FOLDER_MODE, RECORD_MODE, RECORD_VERSION, WRITABLE_BY_OTHERS, WRITING_NAME, DifferingField, RecordCheckoutFailure, RecordedCheckout};

/// The record's own read,
/// which every proof that names a checkout asks.
pub(super) use self::record_file::read_record;

use self::record_file::{already_recorded_line, recorded_line, replaced_line, write_record};

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
    let datasets: ZfsDatasetReader = ZfsDatasetReader::open().map_err(|error| return RecordCheckoutFailure::DatasetRead { dataset: String::from("<the device>"), failure: ReadFailure::at(error.to_string()) })?;
    let places: RecordCheckoutPlaces<'_> = RecordCheckoutPlaces {
        configuration: PathBuf::from(crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH),
        record: PathBuf::from(CHECKOUT_RECORD_PATH),
        mounts: &crate::worktree_clone_mount_handler::sweep_checkout_mount::LiveCheckoutMounts,
        mounted: &LiveMountedDatasets,
        datasets: &datasets,
    };
    return run_with(&places, request, effective_user.is_root(), effective_user.as_raw());
}

/// What:
///  Run the mode from the places, the gate's answer and the caller it is given.
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
pub fn run_with(places: &RecordCheckoutPlaces<'_>, request: &RecordCheckoutRequest, running_as_root: bool, effective_user: u32) -> Result<String, RecordCheckoutFailure> {
    if !running_as_root {
        return Err(RecordCheckoutFailure::NotRoot { effective_user });
    }
    let checkout: PathBuf = request.checkout.clone();
    let configuration: SweepConfiguration = read_configuration(places.configuration.as_path(), places.mounted).map_err(RecordCheckoutFailure::Configuration)?;
    let mount: CheckoutMount = places.mounts.mount_holding(checkout.as_path()).map_err(RecordCheckoutFailure::Mount)?;
    let Some(mounted_dataset) = mount.dataset() else {
        return Err(RecordCheckoutFailure::NotOnZfs { checkout, filesystem_type: mount.filesystem_type });
    };
    let dataset: String = String::from(mounted_dataset);
    if !the_checkout_is_the_mounts_root(checkout.as_path(), &mount) {
        return Err(RecordCheckoutFailure::NotAMountPoint {
            checkout,
            found: format!("is on the mount of `{}` at {}", mount.source, mount.mount_point.display()),
        });
    }
    let identity = places.datasets.identity(dataset.as_str()).map_err(|failure| return RecordCheckoutFailure::DatasetRead { dataset: dataset.clone(), failure })?;
    let facts = facts_of(dataset.as_str(), &identity);
    let mountpoint: String = match value_of(&identity.properties, MOUNTPOINT_PROPERTY) {
        value if value.text() == Some(checkout.to_string_lossy().as_ref()) => String::from(checkout.to_string_lossy().as_ref()),
        value => {
            return Err(RecordCheckoutFailure::NotAMountPoint { checkout, found: value.describe(MOUNTPOINT_PROPERTY) });
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

/// What:
///  Compare the proved facts with the record,
///  admit the GUID a `--replace` names,
///  and write only when the GUID is the whole of what changed.
/// Why:
///  A runbook that runs twice must be harmless,
///  and a differing field is a refusal rather than a write,
///  so the only admitted change is the GUID a restore moved and the caller
///  typed from the kernel's own answer.
///  The record the command writes is the dataset this command named:
///  a container that also differs is refused,
///  because no token names it and the line says the container was kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = writeOrAgree(places, proved, request);
/// ```
///
/// # Errors
/// Everything a differing field,
/// a wrong token,
/// a record that is not one root alone may write,
/// and a write that fails.
fn write_or_agree(places: &RecordCheckoutPlaces<'_>, proved: &RecordedCheckout, request: &RecordCheckoutRequest) -> Result<String, RecordCheckoutFailure> {
    let existing: Option<RecordedCheckout> = read_record(places.record.as_path())?;
    let Some(recorded) = existing else {
        if request.replace.is_some() {
            return Err(RecordCheckoutFailure::NothingToReplace { record: places.record.clone() });
        }
        write_record(places.record.as_path(), proved)?;
        return Ok(recorded_line(proved, places.record.as_path()));
    };
    let difference: Option<DifferingField> = differing_field(&recorded, proved);
    let Some(field) = difference else {
        return Ok(already_recorded_line(places.record.as_path()));
    };
    let Some(typed) = request.replace.as_deref() else {
        return Err(RecordCheckoutFailure::RecordDiffers { recorded: recorded_value(&field), kernel: proved_value(proved, &field), field });
    };
    if typed != proved.guid {
        return Err(RecordCheckoutFailure::ReplaceTokenDiffers { kernel: proved.guid.clone(), typed: String::from(typed) });
    }
    // The token admits the GUID, so the record with that GUID is what the write would leave: any field that still
    // differs is a change no token names, and it refuses rather than being written under a line that says it was kept.
    let was: String = recorded.guid.clone();
    let admitted: RecordedCheckout = RecordedCheckout { guid: proved.guid.clone(), ..recorded };
    if let Some(also) = differing_field(&admitted, proved) {
        return Err(RecordCheckoutFailure::RecordDiffers { recorded: recorded_value(&also), kernel: proved_value(proved, &also), field: also });
    }
    write_record(places.record.as_path(), proved)?;
    return Ok(replaced_line(proved, was.as_str(), places.record.as_path()));
}

/// What:
///  The fields of the record that differ from the proved facts,
///  one at a time.
/// Why:
///  Each field's repair is another command,
///  so the first difference is the one a person is sent to fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const field = differingField(recorded, proved); // undefined when every field agrees
/// ```
fn differing_field(recorded: &RecordedCheckout, proved: &RecordedCheckout) -> Option<DifferingField> {
    if recorded.checkout != proved.checkout {
        return Some(DifferingField::Path { recorded: recorded.checkout.display().to_string() });
    }
    if recorded.dataset != proved.dataset {
        return Some(DifferingField::Dataset { recorded: recorded.dataset.clone() });
    }
    if recorded.guid != proved.guid {
        return Some(DifferingField::Guid { recorded: recorded.guid.clone() });
    }
    if recorded.container != proved.container {
        return Some(DifferingField::Container { recorded: recorded.container.clone() });
    }
    return None;
}

/// The record's value of one field,
/// which a refusal prints beside the kernel's.
fn recorded_value(field: &DifferingField) -> String {
    return match field {
        DifferingField::Path { recorded } | DifferingField::Dataset { recorded } | DifferingField::Guid { recorded } | DifferingField::Container { recorded } => recorded.clone(),
    };
}

/// The kernel's value of one field,
/// which a refusal prints beside the record's.
fn proved_value(proved: &RecordedCheckout, field: &DifferingField) -> String {
    return match field {
        DifferingField::Path { .. } => proved.checkout.display().to_string(),
        DifferingField::Dataset { .. } => proved.dataset.clone(),
        DifferingField::Guid { .. } => proved.guid.clone(),
        DifferingField::Container { .. } => proved.container.clone(),
    };
}

/// What:
///  Whether the checkout is the root of the mount it is on.
/// Why:
///  A folder inside a dataset's mount is not the dataset's own mountpoint,
///  and a record written for it would make every later proof compare against a
///  folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRoot = (checkout, mount) => canonical(checkout) === canonical(mount.mountPoint);
/// ```
fn the_checkout_is_the_mounts_root(checkout: &Path, mount: &CheckoutMount) -> bool {
    return canonical(checkout) == canonical(mount.mount_point.as_path());
}

/// A path with its `.` and `..` components resolved and its repeated slashes
/// folded,
/// as text,
/// and the path as written when it cannot be read.
fn canonical(path: &Path) -> PathBuf {
    return std::fs::canonicalize(path).unwrap_or_else(|_error| return lexical(path));
}

/// What:
///  A path's text with its `.` and `..` components resolved and its repeated
///  slashes folded,
///  without asking the system.
/// Why:
///  A mountpoint whose own directory does not exist yet is compared as the text
///  it is,
///  so the proof never creates a directory to compare one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folded = lexical('/home/user/./Monochromatic//package/../');
/// ```
fn lexical(path: &Path) -> PathBuf {
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

/// What:
///  The pool a dataset's name stands in.
/// Why:
///  A pool is the name's first component,
///  and the container and the checkout must be in one pool or a foreign pool's
///  subtree would sit inside a served checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = (dataset) => dataset.split('/')[0];
/// ```
fn pool_of(dataset: &str) -> String {
    return dataset.split('/').next().unwrap_or_default().to_string();
}

/// What:
///  Refuse a container that is not beside the checkout.
/// Why:
///  The route places the container beside the checkout,
///  and a container inside the checkout would put managed clones inside a served
///  tree,
///  while a container holding the checkout would put a clone store above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheContainerBeside('zpcachyos/nosnap/monochromatic-git-checkout', 'zpcachyos/nosnap/clone-datasets');
/// ```
///
/// # Errors
/// `ContainerNotBesideTheCheckout` naming the direction the proof found.
fn prove_the_container_beside(dataset: &str, container: &str) -> Result<(), RecordCheckoutFailure> {
    if container == dataset {
        return Err(RecordCheckoutFailure::ContainerNotBesideTheCheckout { dataset: String::from(dataset), container: String::from(container), because: String::from("they are the same dataset") });
    }
    if is_inside(container, dataset) {
        return Err(RecordCheckoutFailure::ContainerNotBesideTheCheckout { dataset: String::from(dataset), container: String::from(container), because: String::from("the container is inside the checkout") });
    }
    if is_inside(dataset, container) {
        return Err(RecordCheckoutFailure::ContainerNotBesideTheCheckout { dataset: String::from(dataset), container: String::from(container), because: String::from("the container holds the checkout") });
    }
    return Ok(());
}

/// Whether one dataset's name stands under another's,
/// compared on `/` boundaries so `a/bc` is not inside `a/b`.
fn is_inside(inner: &str, outer: &str) -> bool {
    return inner.len() > outer.len() && inner.starts_with(outer) && inner.as_bytes().get(outer.len()) == Some(&b'/');
}


/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "record_checkout_mode_tests.rs"]
mod tests;
