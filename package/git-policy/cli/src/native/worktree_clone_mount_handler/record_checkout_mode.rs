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

use crate::file_only_root_may_write::{FileControlRefusal, read_file_only_root_may_write};
use crate::worktree_clone_mount_handler::handler_install_paths::{CHECKOUT_RECORD_PATH, root_modes_name};
use crate::worktree_clone_mount_handler::handler_mode_arguments::RecordCheckoutRequest;
use crate::worktree_clone_mount_handler::root_mode_caller::NotRootRefusal;
use crate::worktree_clone_mount_handler::sweep_checkout_mount::{CheckoutMount, CheckoutMountRefusal, CheckoutMounts};
use crate::worktree_clone_mount_handler::sweep_configuration::{SweepConfiguration, read_configuration};
use crate::worktree_clone_mount_handler::sweep_configuration_refusal::ConfigurationRefusal;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{facts_of, value_of, MOUNTPOINT_PROPERTY};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ReadFailure, ZfsDatasetReader};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{LiveMountedDatasets, MountedDatasets};
use rustix::process::geteuid;
use std::io::Write;
use std::path::{Path, PathBuf};

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

/// What:
///  The record as the file holds it,
///  or nothing when the file is not there.
/// Why:
///   The read refuses a file another account could write,
///  and a file that is not the record this mode writes is a refusal rather than
///  a difference,
///  because a record nobody could read decides nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = readRecord('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json');
/// ```
///
/// # Errors
/// The file is not one root alone may write,
/// it could not be read,
/// or it is not the record's shape.
pub(super) fn read_record(path: &Path) -> Result<Option<RecordedCheckout>, RecordCheckoutFailure> {
    // The folder a record stands in decides whether the record is the one this
    // route wrote, so it answers before the file is read:
    // a record another account could replace in its own folder is no record.
    match path.parent() {
        Some(folder) if folder.exists() => check_the_record_folder(folder)?,
        _ => {}
    }
    let text: Option<String> = match read_file_only_root_may_write(path) {
        Ok(text) => text,
        Err(refusal @ FileControlRefusal::Open { .. }) => return Err(RecordCheckoutFailure::RecordUnreadable { path: path.to_path_buf(), reason: refusal.reason() }),
        Err(refusal) => return Err(RecordCheckoutFailure::RecordUnreadable { path: path.to_path_buf(), reason: refusal.reason() }),
    };
    let Some(record_text) = text else {
        return Ok(None);
    };
    let document: serde_json::Value = serde_json::from_str(record_text.as_str()).map_err(|error| return RecordCheckoutFailure::RecordUnreadable { path: path.to_path_buf(), reason: format!("it is not JSON ({error})") })?;
    let Some(object) = document.as_object() else {
        return Err(RecordCheckoutFailure::RecordUnreadable { path: path.to_path_buf(), reason: String::from("it is not a JSON object") });
    };
    let version: u64 = object.get("version").and_then(serde_json::Value::as_u64).ok_or_else(|| return RecordCheckoutFailure::RecordUnreadable { path: path.to_path_buf(), reason: String::from("it holds no `version` this mode wrote") })?;
    let text_field = |key: &str| -> Result<String, RecordCheckoutFailure> {
        return object.get(key).and_then(serde_json::Value::as_str).map(String::from).ok_or_else(|| return RecordCheckoutFailure::RecordUnreadable { path: path.to_path_buf(), reason: format!("it holds no text `{key}`") });
    };
    return Ok(Some(RecordedCheckout {
        version,
        checkout: PathBuf::from(text_field("checkout")?),
        dataset: text_field("dataset")?,
        guid: text_field("guid")?,
        container: text_field("container")?,
        pool: text_field("pool")?,
    }));
}

/// What:
///  Write the record,
///  making its folder when that is absent and refusing one another account
///  could write.
/// Why:
///   The record is root's,
///  so a folder or a file another account could rewrite would make every proof
///  that reads it advisory;
///  the write stands under a name of its own and is renamed into place,
///  so a reader sees the old record or the new one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeRecord('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json', proved);
/// ```
///
/// # Errors
/// The folder could not be made or is not root's,
/// and the write itself.
fn write_record(path: &Path, proved: &RecordedCheckout) -> Result<(), RecordCheckoutFailure> {
    let folder: PathBuf = path.parent().map_or(PathBuf::from("/"), |parent| return parent.to_path_buf());
    make_the_record_folder(folder.as_path())?;
    #[allow(clippy::disallowed_methods, reason = "the JSON macro builds one object of every field; no fallible read is made here")]
    let object: serde_json::Value = serde_json::json!({
        "version": proved.version,
        "checkout": proved.checkout.display().to_string(),
        "dataset": proved.dataset,
        "guid": proved.guid,
        "container": proved.container,
        "pool": proved.pool,
    });
    let document: String = format!("{object:#}\n");
    let held: std::fs::File = crate::foundation::entries_of_one_open_folder::open_folder(folder.as_path()).map_err(|error| return RecordCheckoutFailure::RecordUnwritable { path: path.to_path_buf(), reason: error.to_string() })?;
    let writing: std::fs::File = match rustix::fs::openat(&held, WRITING_NAME, rustix::fs::OFlags::WRONLY | rustix::fs::OFlags::CREATE | rustix::fs::OFlags::EXCL | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::from_bits_truncate(RECORD_MODE)) {
        Ok(opened) => std::fs::File::from(opened),
        Err(error) => {
            let _removed = rustix::fs::unlinkat(&held, WRITING_NAME, rustix::fs::AtFlags::empty());
            return Err(RecordCheckoutFailure::RecordUnwritable { path: path.to_path_buf(), reason: error.to_string() });
        }
    };
    let mut writer: std::fs::File = writing;
    let written: Result<(), std::io::Error> = writer.write_all(document.as_bytes()).and_then(|()| return writer.write_all(b"\n")).and_then(|()| return writer.sync_all());
    drop(writer);
    if let Err(error) = written {
        let _removed = rustix::fs::unlinkat(&held, WRITING_NAME, rustix::fs::AtFlags::empty());
        return Err(RecordCheckoutFailure::RecordUnwritable { path: path.to_path_buf(), reason: error.to_string() });
    }
    let Some(name) = path.file_name() else {
        let _removed = rustix::fs::unlinkat(&held, WRITING_NAME, rustix::fs::AtFlags::empty());
        return Err(RecordCheckoutFailure::RecordUnwritable { path: path.to_path_buf(), reason: String::from("the record's path has no file name") });
    };
    if let Err(error) = rustix::fs::renameat(&held, WRITING_NAME, &held, name) {
        let _removed = rustix::fs::unlinkat(&held, WRITING_NAME, rustix::fs::AtFlags::empty());
        return Err(RecordCheckoutFailure::RecordUnwritable { path: path.to_path_buf(), reason: error.to_string() });
    }
    let _synced = held.sync_all();
    return Ok(());
}

/// What:
///  Make the record's folder when it is absent,
///  and refuse one that is not root's or that another account could write.
/// Why:
///  Step 19's setup runs the same `install -d` line,
///  so a host that never ran the mode still has the folder,
///  and a host whose folder was replaced answers which term failed rather
///  than writing into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// makeTheRecordFolder('/var/lib/monochromatic-git-policy-cli');
/// ```
///
/// # Errors
/// The folder could not be made,
/// or it is not root's or another account could write it.
fn make_the_record_folder(folder: &Path) -> Result<(), RecordCheckoutFailure> {
    if !folder.exists() {
        std::fs::create_dir_all(folder).map_err(|error| return RecordCheckoutFailure::RecordUnwritable { path: folder.to_path_buf(), reason: error.to_string() })?;
        let _moded = rustix::fs::chmod(folder, rustix::fs::Mode::from_bits_truncate(RECORD_FOLDER_MODE));
    }
    return check_the_record_folder(folder);
}

/// What:
///  Refuse the record's folder when it is not this program's account's or
///  when another account could write it.
/// Why:
///  The record decides which checkout the route places a mount under,
///  so the folder it stands in is part of the proof,
///  and a folder another account could add a file to would let that account
///  answer for the route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkTheRecordFolder('/var/lib/monochromatic-git-policy-cli');
/// ```
///
/// # Errors
/// The folder is not this account's,
/// or a group or other write bit is set.
fn check_the_record_folder(folder: &Path) -> Result<(), RecordCheckoutFailure> {
    let held: std::fs::File = crate::foundation::entries_of_one_open_folder::open_folder(folder).map_err(|error| return RecordCheckoutFailure::RecordUnwritable { path: folder.to_path_buf(), reason: error.to_string() })?;
    let metadata: std::fs::Metadata = held.metadata().map_err(|error| return RecordCheckoutFailure::RecordUnwritable { path: folder.to_path_buf(), reason: error.to_string() })?;
    use std::os::unix::fs::MetadataExt;
    let this_account: u32 = geteuid().as_raw();
    if metadata.uid() != this_account {
        return Err(RecordCheckoutFailure::RecordUnwritable {
            path: folder.to_path_buf(),
            reason: format!("its owner is user {} and this program runs as user {this_account}, so the account that owns it could rewrite the record this route acts on", metadata.uid()),
        });
    }
    if metadata.mode() & WRITABLE_BY_OTHERS != 0 {
        return Err(RecordCheckoutFailure::RecordUnwritable {
            path: folder.to_path_buf(),
            reason: format!("its permission bits are {:04o}, and {WRITABLE_BY_OTHERS:04o} would let another account write the record", metadata.mode() & 0o7777),
        });
    }
    return Ok(());
}

/// The line a first record answers.
fn recorded_line(proved: &RecordedCheckout, record: &Path) -> String {
    return format!(
        "recorded: {} is {} (GUID {}), in pool {}, beside container {}; the record is {}",
        proved.checkout.display(),
        proved.dataset,
        proved.guid,
        proved.pool,
        proved.container,
        record.display()
    );
}

/// The line a run that changed nothing answers.
fn already_recorded_line(record: &Path) -> String {
    return format!("already recorded: {}, and every field agrees with the kernel", record.display());
}

/// The line a replace answers.
fn replaced_line(proved: &RecordedCheckout, was: &str, record: &Path) -> String {
    return format!(
        "replaced: {} keeps its path, dataset, container and pool; its GUID was {was} and is now {}. The record is {}.",
        proved.checkout.display(),
        proved.guid,
        record.display()
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "record_checkout_mode_tests.rs"]
mod tests;
