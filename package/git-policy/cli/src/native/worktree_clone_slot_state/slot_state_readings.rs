//! What:
//!  The readings one slot's state is decided from,
//!  each naming the source it came from:
//!  the binding record filed under the slot's name,
//!  the removal's destruction plan that names the clone,
//!  the clone's own mount intent,
//!  and whether the Git registration the record names still stands.
//! Why:
//!  The design tells a known unmounted clone from a pending add or removal by
//!  reading exactly these sources
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12),
//!  and each source has an arm where it cannot be read at all.
//!  Keeping that arm beside the value it replaces is what lets the
//!  classification name the state it cannot prove instead of guessing.
//!  The record's own reader makes the account,
//!  mode and size checks,
//!  and the add's own read of the plan folder answers whether a removal names
//!  this clone,
//!  so this module decides neither question a second time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const readings = { slot, slotPath, binding, plan, intent, mounts, slotMount, registrationStands };
//! ```

use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::plan_folder_read::{PlanStop, refuse_while_a_plan_names_the_clone};
use crate::worktree_clone_binding_record::record_publication::{RecordRefusalAtTheFile, read_binding};
use crate::worktree_clone_binding_record::registration_home::binding_name;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{IntentReading, MountOperation};
use crate::worktree_clone_slot_state::slot_state_mounts::{MountRows, SlotMount};
use std::path::{Path, PathBuf};

/// What:
///  What stands at one slot's record name.
/// Why:
///  "No record" and "a record no reader may act on" are different states:
///  the first says the route holds nothing here,
///  and the second says nothing may be concluded,
///  because a record that cannot be read may name this very slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BindingReading = { kind: 'absent' } | { kind: 'record'; record: BindingRecord } | { kind: 'unreadable'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingReading {
    /// No record is filed under the slot's name.
    Absent,
    /// The record filed under the slot's name,
    /// read whole.
    Record(Box<BindingRecord>),
    /// A record is filed and cannot be read,
    /// with the sentence the record's own reader wrote.
    Unreadable {
        /// Why the record was not read.
        reason: String,
    },
}

/// What:
///  Read the binding record filed under one slot's name.
/// Why:
///  The record's reader already tells a missing file from a file it refuses,
///  and this call keeps that distinction in the shape the classification
///  reads.
///
/// # Errors
/// None:
///  a record that cannot be read is one of the answers,
///  because a caller must report it rather than fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readBindingReading(home, slot);
/// ```
pub fn read_binding_reading(home: &Path, slot: &str) -> BindingReading {
    let path: PathBuf = home.join(binding_name(slot));
    return match read_binding(home, slot) {
        Ok(record) => BindingReading::Record(Box::new(record)),
        Err(RecordRefusalAtTheFile::Missing) => BindingReading::Absent,
        Err(refusal) => BindingReading::Unreadable { reason: refusal.message(path.as_path()) },
    };
}

/// What:
///  Whether a removal's destruction plan names the clone one record describes.
/// Why:
///  A plan may stand beside a binding of any phase,
///  and a removal that was killed leaves its plan as the only witness of what
///  it already destroyed,
///  so a standing plan makes the slot a pending removal whatever the binding
///  says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanReading = { kind: 'not-asked' } | { kind: 'none' } | { kind: 'names-the-clone'; plan: PathBuf } | { kind: 'unreadable'; plan: PathBuf; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanReading {
    /// The plan folder was not read,
    /// because no record was read to name the clone with.
    NotAsked,
    /// No plan in the folder names this clone.
    NoPlan,
    /// A readable plan names this clone.
    NamesTheClone {
        /// The plan's file.
        plan: PathBuf,
    },
    /// A file in the plan folder could not be read as a plan,
    /// so nothing proves it does not name this clone.
    Unreadable {
        /// The plan's file.
        plan: PathBuf,
        /// Why the file was not read.
        reason: String,
    },
}

/// What:
///  Read the plan folder for a plan naming one clone.
/// Why:
///  The add's own read of this folder already decides which entry is a plan
///  and refuses rather than guessing,
///  so the slot's reader asks it instead of reading the folder a second way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = readPlan(commonDir, cloneGuid, cloneDataset);
/// ```
pub fn read_plan(common_dir: &Path, clone_guid: Option<DatasetGuid>, clone_dataset: &str) -> PlanReading {
    return match refuse_while_a_plan_names_the_clone(common_dir, clone_guid, clone_dataset) {
        Ok(()) => PlanReading::NoPlan,
        Err(PlanStop::NamesTheClone(stop)) => PlanReading::NamesTheClone { plan: stop.plan.clone() },
        Err(PlanStop::Unreadable(stop)) => PlanReading::Unreadable { plan: stop.plan.clone(), reason: stop.sentence.clone() },
    };
}

/// What:
///  What one clone's desired-state property asked for,
///  in the terms a person reads.
/// Why:
///  The property's grammar is the protocol's,
///  and the report needs only which operation stands and under which nonce,
///  or why no request could be read.
///  An inherited value authorizes nothing,
///  so it is its own answer rather than an absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IntentFacts = { kind: 'unasked' } | { kind: 'absent' } | { kind: 'mounting'; nonce } | { kind: 'unmounting'; nonce } | { kind: 'inherited'; from } | { kind: 'unreadable'; reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntentFacts {
    /// No clone dataset exists yet,
    /// so no property was asked.
    Unasked,
    /// The clone carries no intent of its own.
    Absent,
    /// The clone asks to be mounted under this nonce.
    Mounting {
        /// The request's 32 lowercase hexadecimal digits.
        nonce: String,
    },
    /// The clone asks to be unmounted under this nonce.
    Unmounting {
        /// The request's 32 lowercase hexadecimal digits.
        nonce: String,
    },
    /// The value is inherited from another dataset and authorizes nothing.
    Inherited {
        /// The dataset the property is set on.
        from: String,
    },
    /// The property cannot be read as a request,
    /// and this is what the grammar or the property reader said.
    Unreadable {
        /// Why no request was read.
        reason: String,
    },
}

/// What:
///  One protocol reading as the facts a report prints.
/// Why:
///  The protocol keeps the nonce and the operation in one value,
///  and this conversion is the only place the two are taken apart,
///  so the empty and refused arms keep the protocol's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = intentFactsOf(readIntent(reply.properties, clone));
/// ```
pub fn intent_facts_of(reading: &IntentReading) -> IntentFacts {
    return match reading {
        IntentReading::Absent => IntentFacts::Absent,
        IntentReading::Inherited { from } => IntentFacts::Inherited { from: from.clone() },
        IntentReading::Request(intent) => match intent.operation {
            MountOperation::Mount => IntentFacts::Mounting { nonce: String::from(intent.nonce.as_str()) },
            MountOperation::Unmount => IntentFacts::Unmounting { nonce: String::from(intent.nonce.as_str()) },
        },
        IntentReading::Refused(refusal) => IntentFacts::Unreadable { reason: refusal.message() },
    };
}

/// What:
///  Every reading one slot's state is decided from,
///  and the two paths those readings are about.
/// Why:
///  One value carries the sources so the classification and the report read
///  the same facts,
///  and a caller cannot hand one of them a fresher record than the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotReadings = { slot: string; slotPath: Path; binding: BindingReading; plan: PlanReading; intent: IntentFacts; mounts: MountRows; slotMount: SlotMount; registrationStands?: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotReadings {
    /// The slot's name,
    /// which is the record's key.
    pub slot: String,
    /// The slot's whole path,
    /// as the record names it.
    pub slot_path: PathBuf,
    /// What stands at the slot's record name.
    pub binding: BindingReading,
    /// Whether a removal's plan names this clone.
    pub plan: PlanReading,
    /// What the clone's mount intent asked for.
    pub intent: IntentFacts,
    /// This process's mount table,
    /// as far as the clone is concerned.
    pub mounts: MountRows,
    /// The mount standing at the slot's own path.
    pub slot_mount: SlotMount,
    /// Whether the Git registration the record names still stands,
    /// and nothing when the record names none yet.
    pub registration_stands: Option<bool>,
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_state_readings_tests.rs"]
mod tests;
