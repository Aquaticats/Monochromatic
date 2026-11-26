//! What:
//!  The add's third step:
//!  the frozen capture taken before the snapshot,
//!  the checkout snapshot,
//!  and the origin inventory published beside them.
//! Why:
//!  `add_route_steps.rs` runs every step and would go over the file-size limit
//!  carrying these too,
//!  and the capture and the snapshot are the one pair a later comparison of the
//!  clone rests on,
//!  so a reader meets the order the two are written in one place.
//!  The capture must predate the snapshot,
//!  which is why it is written first and why a snapshot without one refuses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await stepSnapshot(steps); // the capture, then the snapshot, then the inventory
//! ```

use super::facts_and_refusal::{StepRefusal, Steps, write_the_record};
use crate::worktree_clone_add_route::add_capture::{Capture, CaptureInput, capture_the_source};
use crate::worktree_clone_add_route::add_capture_store::{publish_capture, read_capture};
use crate::worktree_clone_add_route::add_zfs::ObjectIdentity;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_inventory::InventoryKind;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::record_publication::{RecordRefusalAtTheFile, publish_inventory, read_inventory};

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
    let standing: Option<Capture> = read_capture(steps.home_path, steps.route.slot.as_str()).map_err(StepRefusal::refused)?;
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
    let input: CaptureInput<'_> = CaptureInput {
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
    let capture: Capture = capture_the_source(&input).map_err(StepRefusal::refused)?;
    publish_capture(steps.home, steps.route.slot.as_str(), &capture).map_err(|error| {
        return StepRefusal::refused(format!("the frozen capture cannot be published ({error}); nothing was mutated beyond the registration, and running the add again resumes it"));
    })?;
    steps.stops.hold("capture").map_err(StepRefusal::refused)?;
    return Ok(());
}
