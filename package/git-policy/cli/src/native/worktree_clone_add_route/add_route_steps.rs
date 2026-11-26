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

use super::add_mount::{MountedHere, the_mount_at, wait_for_the_mount};
use super::add_zfs::ObjectIdentity;
use crate::worktree_clone_binding_record::binding_inventory::InventoryKind;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::record_publication::{RecordRefusalAtTheFile, publish_inventory, read_inventory};

/// The fixed facts one add's steps share,
/// the refusal a step stops with and the one write that persists a phase.
pub use super::add_route_steps_facts_and_refusal::{StepRefusal, Steps, write_the_record};

/// Make or verify the empty slot and record its identity.
pub use super::add_route_steps_slot_and_registration::step_slot;

/// Scan for the registration and run Git's own add,
/// or adopt what stands.
pub use super::add_route_steps_slot_and_registration::step_git;

/// Take the checkout snapshot and publish its inventory.
pub use super::add_route_steps_snapshot_and_capture::step_snapshot;

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
