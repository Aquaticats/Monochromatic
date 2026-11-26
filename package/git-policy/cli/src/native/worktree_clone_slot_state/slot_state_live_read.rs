//! What:
//!  The reads that fill one slot's readings from the running machine:
//!  the binding record under the repository's Git common directory,
//!  the removal's plan folder,
//!  the clone's own mount intent through the gated ZFS device,
//!  this process's mount table,
//!  and the Git registration the record names.
//! Why:
//!  The classification and the report take plain values,
//!  and this file is the one place those values come from the machine,
//!  so a control can script every source and the release path reads the same
//!  code.
//!  The intent is a property of the clone dataset,
//!  so reading it is a raw ZFS request and every such request passes the
//!  compatibility gate first
//!  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`,
//!  "Any version,
//! layout decides"),
//!  exactly as the intent's writer does.
//!  The intent is asked only for a clone the record says was mounted:
//!  an add's earlier phases have no clone,
//!  and a fallback's own intent is the add's business and not this report's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reads = { mounts: new LiveMountedDatasets(), intents: new GatedCloneIntentReads(gate) };
//! const readings = readSlot(commonDir, 'topic', slotPath, reads);
//! ```

use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::registration_home::home_folder;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{
    IntentReading,
    read_intent,
};
use crate::worktree_clone_slot_state::slot_state_mounts::{
    read_mount_rows,
    slot_mount_of,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    IntentFacts,
    PlanReading,
    SlotReadings,
    intent_facts_of,
    read_binding_reading,
    read_plan,
};
use crate::zfs_compatibility_gate::gate_verdict::{
    BeforeRequests,
    Verdict,
};
use crate::zfs_compatibility_gate::verdict_line;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    ZfsDevice,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  One clone's desired-state property,
///  read as the protocol's grammar reads it.
/// Why:
///  The live read needs a gate and a device,
///  and a control needs neither,
///  so the read is a trait and the classification never opens `/dev/zfs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CloneIntentReads { intentOf(clone: string): IntentReading }
/// ```
pub trait CloneIntentReads {
    /// The clone's intent,
    /// or why no property could be read.
    ///
    /// # Errors
    /// The gate refused,
    /// the device could not be opened,
    /// or the dataset could not be read.
    fn intent_of(
        &self,
        clone: &str,
    ) -> Result<IntentReading, String>;
}

/// What:
///  The release reader:
///  the gate,
///  then the binding's own `/dev/zfs` handle.
/// Why:
///  Both wrapped ZFS programs run the gate before their first request,
///  and this reader is the account-run wrapper's second reader of a clone's
///  properties,
///  so it takes the same gate as the writer and opens no device before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GatedCloneIntentReads { constructor(gate) {} intentOf(clone) { gate.verdict(); ... } }
/// ```
pub struct GatedCloneIntentReads<'a> {
    /// The gate every raw request passes.
    gate: &'a dyn BeforeRequests,
}

impl<'a> GatedCloneIntentReads<'a> {
    /// The reader that asks this gate before any property read.
    pub fn new(gate: &'a dyn BeforeRequests) -> Self {
        return GatedCloneIntentReads {
            gate,
        };
    }
}

impl CloneIntentReads for GatedCloneIntentReads<'_> {
    fn intent_of(
        &self,
        clone: &str,
    ) -> Result<IntentReading, String> {
        let verdict: Verdict = self.gate.verdict();
        if let Verdict::Refused(_) = verdict {
            return Err(format!(
                "{}\nthe compatibility gate refused, so no property was read",
                verdict_line(&verdict)
            ));
        }
        let device: ZfsDevice = ZfsDevice::open()
            .map_err(|error| return format!("the ZFS device could not be opened ({error})"))?;
        let identity: DatasetIdentity = device
            .objset_stats(clone)
            .map_err(|error| return format!("`{clone}` could not be read ({error})"))?;
        return Ok(read_intent(Some(&identity.properties), clone));
    }
}

/// What:
///  The two readers one slot's state is read through.
/// Why:
///  A caller names the mount table and the intent source once,
///  so a control supplies both and the release path supplies the live pair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LiveSlotReads = { mounts: MountedDatasets; intents: CloneIntentReads };
/// ```
pub struct SlotReadingSources<'a> {
    /// This process's mount table.
    pub mounts: &'a dyn MountedDatasets,
    /// Where one clone's intent comes from.
    pub intents: &'a dyn CloneIntentReads,
}

/// What:
///  Whether the Git registration the record names still stands.
/// Why:
///  The record holds the registration's path,
///  and a registration that is gone changes what a reattach would leave,
///  so the report says which of the two the filesystem shows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stands = registrationStands(record);
/// ```
fn registration_stands(record: &BindingRecord) -> Option<bool> {
    let path: PathBuf = record.registration_path()?;
    return Some(std::fs::symlink_metadata(path.as_path()).is_ok());
}

/// What:
///  The intent facts for a record that says the worktree was mounted.
/// Why:
///  The property lives on the clone dataset,
///  so the question has an answer only from `phase=cloned` on,
///  and the report uses it only for a clone the record says was mounted or
///  filled:
///  an add's earlier phases have no clone,
///  and a fallback's intent belongs to the add's own list of commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intent = askedIntent(record, reads.intents);
/// ```
fn asked_intent(
    record: &BindingRecord,
    intents: &dyn CloneIntentReads,
) -> IntentFacts {
    if !matches!(record.phase, Phase::Mounted | Phase::Filled) {
        return IntentFacts::Unasked;
    }
    return match intents.intent_of(record.clone_dataset.as_str()) {
        Ok(reading) => intent_facts_of(&reading),
        Err(reason) => IntentFacts::Unreadable {
            reason,
        },
    };
}

/// What:
///  Every reading one slot's state is decided from,
///  read from the machine.
/// Why:
///  A caller asks one question and gets the whole report's input,
///  and each source's refusal travels with the reading instead of ending the
///  call,
///  because the design's answer to a source it cannot read is a state it names
///  rather than a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readings = readSlot(commonDir, slot, slotPath, reads);
/// ```
pub fn read_slot(
    common_dir: &Path,
    slot: &str,
    slot_path: &Path,
    reads: &SlotReadingSources<'_>,
) -> SlotReadings {
    let home: PathBuf = home_folder(common_dir);
    let binding: BindingReading = read_binding_reading(home.as_path(), slot);
    let mounts = read_mount_rows(reads.mounts);
    let slot_mount = slot_mount_of(reads.mounts, slot_path);
    // A record is keyed by the slot's last name, so the path it names says which worktree it belongs to, and the
    // classification proves nothing about a different path from it. Asking the plan folder and the clone's own intent
    // first would open the kernel's ZFS device for a removal, a move or a repair of a plain worktree that shares a bound
    // slot's last name, which endangers nothing (`refuse_the_removal_with`; finding Low 5 of the alpha.47 pass), so the
    // record's path is compared before either is asked.
    let (plan, intent, stands): (PlanReading, IntentFacts, Option<bool>) = match &binding {
        BindingReading::Record(record) if names_another_path(record, slot_path) => {
            (PlanReading::NotAsked, IntentFacts::Unasked, None)
        }
        BindingReading::Record(record) => (
            read_plan(common_dir, record.clone_guid, record.clone_dataset.as_str()),
            asked_intent(record, reads.intents),
            registration_stands(record),
        ),
        BindingReading::Absent
        | BindingReading::Unreadable {
            ..
        } => (PlanReading::NotAsked, IntentFacts::Unasked, None),
    };
    // One mount reading serves the clone's question and the slot's, so the two can never disagree about the same table.
    return SlotReadings {
        slot: String::from(slot),
        slot_path: slot_path.to_path_buf(),
        binding,
        plan,
        intent,
        mounts,
        slot_mount,
        registration_stands: stands,
    };
}

/// What:
///  Whether the record filed under `slot` names a path other than the one the
///  command stands at.
/// Why:
///  The read above and the classification must compare the same paths,
///  or one of them would ask the pool about a worktree that is not the one the
///  command names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesAnother = (record, slotPath) => record.slotPath() !== slotPath;
/// ```
fn names_another_path(
    record: &BindingRecord,
    slot_path: &Path,
) -> bool {
    return record
        .slot_path()
        .is_some_and(|recorded| return recorded.as_path() != slot_path);
}
/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_state_live_read_tests.rs"]
mod tests;
