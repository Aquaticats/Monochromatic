//! What:
//!  The add's steps after the claim, one per phase:
//!  the empty slot and its identity,
//!  Git's registration and the scan that adopts or clears it,
//!  the checkout snapshot and its inventory,
//!  the clone and its inventory,
//!  the mount request and its verification,
//!  and the fill.
//! Why:
//!  Each step reads the state a kill at the step before it leaves and writes
//!  only the facts it confirmed, so a resume continues from the phase the
//!  record names
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The add's writes in one order" and "What the writer confirms before each write").
//!  The steps live apart from the runner so each is one readable unit and the
//!  runner only chooses which one runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await stepSlot(steps); await stepGit(steps); await stepSnapshot(steps); ...
//! ```

use super::add_git_words::the_locked_add_words;
use super::add_mount::{MountedHere, the_mount_at, wait_for_the_mount};
use super::add_registration::{Registration, ScanInput, ScanOutcome, scan_and_settle};
use super::add_route_plan::{AddNames, Stops};
use super::add_zfs::{ObjectIdentity, OpenedDevice};
use crate::forwarding::{ChildOutcome, run_real_git};
use crate::hooks_off::StartOwner;
use crate::private_storage::create_private_directory_all;
use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_inventory::InventoryKind;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::record_publication::{RecordRefusalAtTheFile, publish_inventory, read_inventory, write_binding};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _};
use std::path::Path;

/// What:
///  The fixed facts one add's steps share.
/// Why:
///  Every step names the same slot, records and device, so one value carries
///  them and no step recomputes one.
pub struct Steps<'a> {
    /// The gate's facts.
    pub route: &'a super::add_route_gate::RouteFacts,
    /// The names chosen before the first write.
    pub names: &'a AddNames,
    /// The proven clean source.
    pub source: &'a SnapshotSource,
    /// The add's plan.
    pub plan: &'a AddPlan,
    /// The repository's Git context.
    pub context: &'a GitContext,
    /// The wrapper's record home, open.
    pub home: &'a std::fs::File,
    /// The record home's path.
    pub home_path: &'a Path,
    /// The pool's device, opened behind the gate.
    pub device: &'a OpenedDevice,
    /// The checkout dataset's GUID.
    pub checkout_guid: DatasetGuid,
    /// The pool's GUID.
    pub pool_guid: DatasetGuid,
    /// The person's arguments.
    pub arguments: &'a [OsString],
    /// The real Git.
    pub real_git: &'a Path,
    /// The variables every Git child receives.
    pub overlay: &'a [(OsString, OsString)],
    /// Whether the person asked for no lines.
    pub quiet: bool,
    /// The test-only switch that ends a control run at one step.
    pub stops: &'a Stops,
    /// The lines the add prints for the person.
    pub out: std::cell::RefCell<Vec<String>>,
}

/// What:
///  Why one step stopped without finishing.
/// Why:
///  A step's refusal and Git's own status are different answers:
///  a refusal changes nothing and keeps the records,
///  and a Git failure keeps its status so a script reads the same one.
#[derive(Debug)]
pub struct StepRefusal {
    /// The words the person reads.
    pub message: String,
    /// The exit status this add ends with.
    pub code: i32,
}

impl StepRefusal {
    /// A refusal of the add's own.
    pub fn refused(message: String) -> StepRefusal {
        return StepRefusal { message, code: REFUSED_EXIT_CODE };
    }

    /// A Git run that ended unsuccessfully,
    /// keeping Git's own status.
    pub fn git(code: i32) -> StepRefusal {
        return StepRefusal { message: String::new(), code };
    }
}

/// What:
///  Persist one phase of the record and flush it.
/// Why:
///  Every write of this record is flushed, because a later command must not have
///  to rediscover a fact the pool already committed.
pub fn write_the_record(steps: &Steps<'_>, record: &BindingRecord) -> Result<(), StepRefusal> {
    write_binding(steps.home, &steps.route.slot, record).map_err(|error| return StepRefusal::refused(format!("the add's record cannot be written ({error}); the claim, the slot and the registration stand, and running the add again resumes them")))?;
    return Ok(());
}

/// What:
///  Make or verify the empty slot and record its identity.
/// Why:
///  Git accepts an existing empty slot and nothing else,
///  and the identity is recorded before any mount covers the path.
pub fn step_slot(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    let slot: &Path = steps.route.destination.as_path();
    let existing: Option<FolderIdentity> = match std::fs::symlink_metadata(slot) {
        Ok(metadata) => Some(verify_the_slot(&metadata, slot)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(StepRefusal::refused(format!("the slot {} cannot be examined ({error})", slot.display()))),
    };
    let identity: FolderIdentity = match existing {
        Some(found) => found,
        None => make_the_slot(slot)?,
    };
    if record.slot_identity == Some(identity) {
        return Ok(());
    }
    record.slot_identity = Some(identity);
    record.phase = Phase::Slot;
    write_the_record(steps, record)?;
    steps.stops.after("slot-identity");
    return Ok(());
}

/// Verify one existing slot: a directory of this account, empty, on the
/// checkout's own dataset.
fn verify_the_slot(metadata: &std::fs::Metadata, slot: &Path) -> Result<FolderIdentity, StepRefusal> {
    if !metadata.is_dir() {
        return Err(StepRefusal::refused(format!("the slot {} holds something that is not a folder; Ways forward: remove it, then run the add again", slot.display())));
    }
    // SAFETY: `geteuid` reads this process's own identity and has no preconditions.
    let ours: u32 = unsafe { libc::geteuid() };
    if metadata.uid() != ours {
        return Err(StepRefusal::refused(format!("the slot {} belongs to account {}, and this is account {ours}; Ways forward: remove it as its owner, then run the add again", slot.display(), metadata.uid())));
    }
    let mut entries = std::fs::read_dir(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be listed ({error})", slot.display())))?;
    if entries.next().is_some() {
        return Err(StepRefusal::refused(format!(
            "the slot {} holds entries, and the clone route registers a worktree only in an empty slot; Ways forward: remove the entries or name another worktree",
            slot.display()
        )));
    }
    return FolderIdentity::of_folder(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be identified ({error})", slot.display())));
}

/// Make the empty slot with this account's ownership.
fn make_the_slot(slot: &Path) -> Result<FolderIdentity, StepRefusal> {
    let parent: &Path = slot.parent().ok_or_else(|| return StepRefusal::refused(format!("the slot {} names no parent", slot.display())))?;
    create_private_directory_all(parent).map_err(|error| return StepRefusal::refused(format!("the slot's folder {} cannot be made ({error})", parent.display())))?;
    let mut builder: std::fs::DirBuilder = std::fs::DirBuilder::new();
    builder.mode(0o755);
    builder.create(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be made ({error})", slot.display())))?;
    steps_note_after_making(slot)?;
    return FolderIdentity::of_folder(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be identified ({error})", slot.display())));
}

/// Make sure the slot Git receives is a plain empty folder on the checkout.
fn steps_note_after_making(slot: &Path) -> Result<(), StepRefusal> {
    let metadata: std::fs::Metadata = std::fs::metadata(slot).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be examined ({error})", slot.display())))?;
    if !metadata.is_dir() {
        return Err(StepRefusal::refused(format!("the slot {} is no folder after it was made", slot.display())));
    }
    return Ok(());
}

/// What:
///  Scan for the registration,
///  adopt the complete one,
///  clear the incomplete one and run Git,
///  or run Git when nothing stands.
/// Why:
///  Git's own add is the only writer of the lock and the registration,
///  so the scan's answer decides whether it runs again at all,
///  and the line a lock put back owes a person is printed here,
///  once,
///  where the command's other lines go.
pub fn step_git(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    for _round in 0..3 {
        let outcome: ScanOutcome = scan_and_settle(&ScanInput { common_dir: steps.route.common_dir.as_path(), home: steps.home_path, slot: steps.route.slot.as_str(), slot_path: steps.route.destination.as_path(), nonce: &record.nonce, guids: steps.device })
            .map_err(StepRefusal::refused)?;
        match outcome {
            ScanOutcome::Adopt { registration, wrote_the_lock } => {
                if let Some(line) = wrote_the_lock {
                    steps.out.borrow_mut().push(format!("monochromatic-git-policy-cli: {line}\n"));
                }
                return adopt(steps, record, &registration);
            }
            ScanOutcome::RunGit { cleared } => {
                if !cleared.is_empty() {
                    steps.out.borrow_mut().push(format!("monochromatic-git-policy-cli: cleared an incomplete registration at {} ({}).\n", steps.route.slot, cleared.join(", ")));
                }
                // Git is about to run, so the slot must be the empty folder its
                // own add requires, with its identity recorded before the mount
                // can cover it. A slot Git already wrote to is the scan's
                // business above and never this step's.
                step_slot(steps, record)?;
                run_the_git_add(steps, &record.nonce)?;
                steps.stops.after("git-add");
            }
        }
    }
    return Err(StepRefusal::refused(format!(
        "the registration for {} kept changing while the add ran; the claim, the slot and whatever Git left stand, and running the add again resumes them",
        steps.route.slot
    )));
}

/// Record the adopted registration's facts and move to `registered`.
fn adopt(steps: &Steps<'_>, record: &mut BindingRecord, registration: &Registration) -> Result<(), StepRefusal> {
    // A registration adopted at phase `claimed` was made without A4's write, and
    // every later reader needs the identity of the folder it names.
    if record.slot_identity.is_none() {
        record.slot_identity = Some(FolderIdentity::of_folder(steps.route.destination.as_path()).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be identified ({error})", steps.route.destination.display())))?);
    }
    record.registration = Some(registration.path.as_os_str().as_encoded_bytes().to_vec());
    record.registration_identity = Some(registration.identity);
    record.pointer = Some(registration.pointer.clone());
    record.phase = Phase::Registered;
    write_the_record(steps, record)?;
    steps.stops.after("registered");
    return Ok(());
}

/// Run Git's own add with the managed lock, keeping Git's status.
fn run_the_git_add(steps: &Steps<'_>, nonce: &crate::worktree_clone_mount_protocol::RequestNonce) -> Result<(), StepRefusal> {
    let words: Vec<OsString> = the_locked_add_words(steps.arguments, steps.plan.insert_at, nonce).map_err(StepRefusal::refused)?;
    let ran: ChildOutcome = run_real_git(steps.real_git, words.as_slice(), steps.overlay, StartOwner::Person).map_err(|error| {
        return StepRefusal::refused(format!("Git's own add could not be started ({error}); the claim, the slot and the records stand, and running the add again resumes them"));
    })?;
    return match ran {
        ChildOutcome::Exited(0) => Ok(()),
        ChildOutcome::Exited(code) => Err(StepRefusal::git(code)),
        ChildOutcome::Signaled(signal) => Err(StepRefusal::git(128 + signal)),
    };
}

/// Take the checkout snapshot, publish its inventory, and move to
/// `snapshotted`.
pub fn step_snapshot(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    write_the_capture(steps, record)?;
    let origin: ObjectIdentity = match record.origin_snapshot_guid {
        Some(guid) => {
            let live: ObjectIdentity = steps.device.identity_of(record.origin_snapshot.as_str()).map_err(StepRefusal::refused)?;
            if live.guid != guid || Some(live.createtxg) != record.origin_snapshot_createtxg {
                return Err(StepRefusal::refused(format!(
                    "the snapshot {} answers GUID {} at transaction group {}, and the record names {} at {}; Ways forward: inspect the snapshot, then run the add again or remove the records by hand",
                    record.origin_snapshot,
                    live.guid.file_name(),
                    live.createtxg,
                    guid.file_name(),
                    record.origin_snapshot_createtxg.map_or_else(|| return String::from("none"), |txg| return txg.to_string())
                )));
            }
            live
        }
        None => {
            let live: ObjectIdentity = steps.device.snapshot_the_checkout(steps.route.pool.as_str(), record.origin_snapshot.as_str()).map_err(StepRefusal::refused)?;
            record.origin_snapshot_guid = Some(live.guid);
            record.origin_snapshot_createtxg = Some(live.createtxg);
            live
        }
    };
    let inventory = steps
        .device
        .origin_inventory(steps.route.checkout_dataset.as_str(), steps.pool_guid, steps.checkout_guid, record.origin_snapshot.as_str(), origin)
        .map_err(StepRefusal::refused)?;
    match read_inventory(steps.home_path, steps.route.slot.as_str(), InventoryKind::Origin) {
        Ok(_recorded) => {}
        Err(RecordRefusalAtTheFile::Missing) => {
            publish_inventory(steps.home, steps.route.slot.as_str(), InventoryKind::Origin, &inventory).map_err(|error| {
                return StepRefusal::refused(format!("the origin inventory cannot be published ({error}); the snapshot stands and its name is recorded, and running the add again resumes it"));
            })?;
        }
        Err(refusal) => return Err(StepRefusal::refused(refusal.message(&steps.home_path.join(crate::worktree_clone_binding_record::registration_home::origin_inventory_name(steps.route.slot.as_str()))))),
    }
    record.origin_dataset = Some(steps.route.checkout_dataset.clone());
    record.origin_dataset_guid = Some(steps.checkout_guid);
    record.phase = Phase::Snapshotted;
    write_the_record(steps, record)?;
    steps.stops.after("origin-inventory");
    return Ok(());
}

/// What:
///  Take and publish the frozen capture before the snapshot,
///  or use the one an earlier run published.
/// Why:
///  The capture must predate the snapshot, so a run that finds a snapshot but no
///  capture has lost the only proof its clone can be compared with.
fn write_the_capture(steps: &Steps<'_>, record: &BindingRecord) -> Result<(), StepRefusal> {
    let standing: Option<super::add_capture::Capture> = super::add_capture_store::read_capture(steps.home_path, steps.route.slot.as_str()).map_err(StepRefusal::refused)?;
    if standing.is_some() {
        steps.stops.hold("capture").map_err(StepRefusal::refused)?;
        return Ok(());
    }
    if record.origin_snapshot_guid.is_some() {
        return Err(StepRefusal::refused(format!(
            "the snapshot {} stands and the frozen capture that proves it is gone, so nothing can compare the clone with the source; Ways forward: inspect the snapshot, remove it and the records, then run the add again",
            record.origin_snapshot
        )));
    }
    let admin: std::path::PathBuf = match record.registration_path() {
        Some(found) => found,
        None => return Err(StepRefusal::refused(String::from("the record names no registration, so the capture has no worktree to read its HEAD from"))),
    };
    let index: std::path::PathBuf = steps.source.git_dir.join("index");
    let bytes: Vec<u8> = std::fs::read(index.as_path()).map_err(|error| return StepRefusal::refused(format!("the source's index {} cannot be read for the capture ({error})", index.display())))?;
    let metadata: std::fs::Metadata = std::fs::metadata(index.as_path()).map_err(|error| return StepRefusal::refused(format!("the source's index {} cannot be examined ({error})", index.display())))?;
    let format: crate::repo_read_layout::ObjectFormat = if steps.source.clean.hash_length == 20 { crate::repo_read_layout::ObjectFormat::Sha1 } else { crate::repo_read_layout::ObjectFormat::Sha256 };
    let head: String = crate::worktree_clean::head_commit(admin.as_path(), steps.route.common_dir.as_path(), format).ok_or_else(|| return StepRefusal::refused(String::from("the new worktree's HEAD names no commit, so the capture cannot record what the add checks out")))?;
    let source_head: String = steps.source.clean.head_commit.clone();
    let input: super::add_capture::CaptureInput<'_> = super::add_capture::CaptureInput {
        source_root: steps.route.source_root.as_path(),
        repository: steps.route.common_dir.as_path(),
        index_bytes: bytes.as_slice(),
        index_mtime: (std::os::unix::fs::MetadataExt::mtime(&metadata), std::os::unix::fs::MetadataExt::mtime_nsec(&metadata) as u32),
        hash_length: steps.source.clean.hash_length,
        nonce: &record.nonce,
        head,
        source_head,
        checkout_guid: steps.checkout_guid,
    };
    let capture: super::add_capture::Capture = super::add_capture::capture_the_source(&input).map_err(StepRefusal::refused)?;
    super::add_capture_store::publish_capture(steps.home, steps.route.slot.as_str(), &capture).map_err(|error| {
        return StepRefusal::refused(format!("the frozen capture cannot be published ({error}); nothing was mutated beyond the registration, and running the add again resumes it"));
    })?;
    steps.stops.hold("capture").map_err(StepRefusal::refused)?;
    return Ok(());
}

/// What:
///  Make the clone when it is not there, adopt it when only its record is
///  missing, publish its inventory, and ask for its mount.
/// Why:
///  A successful clone does not imply a mount, and the request is what the
///  handler sweeps for.
pub fn step_clone(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    let identity: ObjectIdentity = match record.clone_guid {
        Some(guid) => {
            let live: ObjectIdentity = steps.device.identity_of(record.clone_dataset.as_str()).map_err(StepRefusal::refused)?;
            if live.guid != guid {
                return Err(StepRefusal::refused(format!("the clone {} answers GUID {}, and the record names {}; Ways forward: inspect the dataset, then run the add again", record.clone_dataset, live.guid.file_name(), guid.file_name())));
            }
            live
        }
        None => match steps.device.identity_of(record.clone_dataset.as_str()) {
            Ok(found) => {
                if !the_clone_came_from(found, steps, record)? {
                    return Err(StepRefusal::refused(format!("the dataset {} already stands and does not come from {}, so nothing proves it is this add's; Ways forward: remove or rename it, then run the add again", record.clone_dataset, record.origin_snapshot)));
                }
                found
            }
            Err(_absent) => {
                let made: ObjectIdentity = steps.device.clone_the_origin(record.clone_dataset.as_str(), record.origin_snapshot.as_str()).map_err(StepRefusal::refused)?;
                record.clone_guid = Some(made.guid);
                record.clone_createtxg = Some(made.createtxg);
                steps.stops.after("clone");
                made
            }
        },
    };
    record.clone_guid = Some(identity.guid);
    record.clone_createtxg = Some(identity.createtxg);
    let inventory = steps.device.clone_inventory(record.clone_dataset.as_str(), steps.pool_guid, identity).map_err(StepRefusal::refused)?;
    match read_inventory(steps.home_path, steps.route.slot.as_str(), InventoryKind::Clone) {
        Ok(_recorded) => {}
        Err(RecordRefusalAtTheFile::Missing) => {
            // The clone's own history is the transferred set only while it holds
            // no snapshot of its own, so a rebuild after one refuses.
            if !inventory.entries.is_empty() {
                return Err(StepRefusal::refused(format!(
                    "the clone {} holds snapshots of its own and its initial inventory is missing, so what it started from can no longer be proven; Ways forward: inspect the clone and remove it, then run the add again",
                    record.clone_dataset
                )));
            }
            publish_inventory(steps.home, steps.route.slot.as_str(), InventoryKind::Clone, &inventory).map_err(|error| {
                return StepRefusal::refused(format!("the clone's inventory cannot be published ({error}); the clone stands and its identity is recorded, and running the add again resumes it"));
            })?;
        }
        Err(refusal) => return Err(StepRefusal::refused(refusal.message(&steps.home_path.join(crate::worktree_clone_binding_record::registration_home::clone_inventory_name(steps.route.slot.as_str()))))),
    }
    record.phase = Phase::Cloned;
    write_the_record(steps, record)?;
    steps.stops.after("clone-inventory");
    return Ok(());
}

/// Whether a dataset that answers the clone's name came from this add's origin.
fn the_clone_came_from(found: ObjectIdentity, steps: &Steps<'_>, record: &BindingRecord) -> Result<bool, StepRefusal> {
    let _identity: ObjectIdentity = found;
    let origin: Option<String> = steps.device.origin_of(record.clone_dataset.as_str()).map_err(StepRefusal::refused)?;
    return Ok(origin.as_deref() == Some(record.origin_snapshot.as_str()));
}

/// What:
///  Ask for the mount and verify it at the slot, then move to `mounted`.
/// Why:
///  The slot shows the clone only in this command's own namespace, so the wait
///  reads the mount table through the slot and not the handler's word.
pub fn step_mount(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    if let Some(mount) = the_mount_at(steps.route.destination.as_path(), record.clone_dataset.as_str()).map_err(StepRefusal::refused)? {
        let _verified: crate::worktree_clone_mount::attached_mount_identity::MountIdentity = mount;
        return record_mounted(steps, record);
    }
    if record.phase == Phase::Mounted {
        return Err(StepRefusal::refused(format!(
            "the record says the worktree is mounted, and the slot {} shows no mount of {}; Ways forward: unmount and mount the clone again, then run the add again",
            steps.route.destination.display(),
            record.clone_dataset
        )));
    }
    steps.device.request_the_mount(record.clone_dataset.as_str(), &record.nonce).map_err(StepRefusal::refused)?;
    steps.stops.after("mount-intent");
    let mounted: MountedHere = wait_for_the_mount(steps.route.destination.as_path(), record.clone_dataset.as_str()).map_err(StepRefusal::refused)?;
    let _mount = mounted;
    steps.stops.after("mount");
    return record_mounted(steps, record);
}

/// Record the verified mount.
fn record_mounted(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    record.phase = Phase::Mounted;
    return write_the_record(steps, record);
}

/// What:
///  Fill the mounted clone and mark the record usable.
/// Why:
///  The phase follows the exchange and its verification, so a reader never sees
///  a usable record before the worktree it names is whole.
pub fn step_fill(steps: &Steps<'_>, record: &mut BindingRecord) -> Result<Option<crate::worktree_trash::PendingTrash>, StepRefusal> {
    let admin_path: std::path::PathBuf = match record.registration_path() {
        Some(found) => found,
        None => {
            return Err(StepRefusal::refused(String::from(
                "the record names no registration, so the fill has no administrative directory; the claim, the slot and the registration stand, and running the add again resumes them",
            )));
        }
    };
    let capture = super::add_capture_store::read_capture(steps.home_path, steps.route.slot.as_str())
        .map_err(StepRefusal::refused)?
        .ok_or_else(|| return StepRefusal::refused(format!("the frozen capture for {} is not among the records, so the clone cannot be proven; Ways forward: remove the records and run the add again", steps.route.slot)))?;
    let pointer: Vec<u8> = record.pointer.clone().ok_or_else(|| return StepRefusal::refused(String::from("the record holds no pointer, so the fill cannot exchange it")))?;
    let filled: super::add_fill::Filled = super::add_fill::fill_the_clone(&super::add_fill::FillInput {
        source: steps.source,
        plan: steps.plan,
        slot: steps.route.slot.as_str(),
        clone_root: steps.route.destination.as_path(),
        common_dir: steps.route.common_dir.as_path(),
        capture: &capture,
        hash_length: steps.source.clean.hash_length,
        pointer: pointer.as_slice(),
        nonce: &record.nonce,
        home: steps.home_path,
        admin_path: admin_path.as_path(),
        context: steps.context,
        real_git: steps.real_git,
        overlay: steps.overlay,
        quiet: steps.quiet,
        stops: steps.stops,
    })
    .map_err(StepRefusal::refused)?;
    record.phase = Phase::Filled;
    write_the_record(steps, record)?;
    steps.stops.after("filled");
    return Ok(filled.trash);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_route_steps_tests.rs"]
mod tests;
