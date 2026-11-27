//! What:
//!  The one state one slot is in,
//!  decided from the binding record,
//!  the removal's plan,
//!  the clone's mount intent and the mount table:
//!  a known unmounted clone,
//!  a pending add,
//!  a pending removal,
//!  a mounted managed clone,
//!  a slot the route holds nothing in,
//!  or a state no reading proves.
//! Why:
//!  The design asks the wrapper to keep a known unmounted clone apart from a
//!  pending add or removal,
//!  and to name a state it cannot prove rather than treat absent files as
//!  deleted tracked files or offer an ordinary add over them
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  The order is the design's:
//!  a record no reader may act on proves nothing,
//!  a plan that names the clone is a removal in flight,
//!  the record's phase says how far the add got,
//!  and only the mount table says whether a clone the record calls mounted is
//!  mounted now.
//!  Nothing here reads the pool,
//!  writes a record or changes a mount:
//!  every arm is a statement about the readings handed in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const state = classify_slot({ slot, slotPath, binding, plan, intent, mounts, slotMount, registrationStands });
//! ```

use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_slot_state::slot_state_mounts::{
    CloneMount,
    SlotMount,
    clone_mount_of,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    PlanReading,
    SlotReadings,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Which reading left a slot's state unproved,
///  as the removal's refusal reads it.
/// Why:
///  The owner's answer of 2026-11-26 refuses the removal in three cases a slot
///  proves nothing about and leaves every other unproved state to Git,
///  so the classification keeps which reading stopped the proof rather than
///  leaving a caller to read a sentence's words
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProofStoppedBy = 'a-mount-no-record-names' | 'a-record-of-another-path' | 'a-mount-that-cannot-be-read' | 'another-reading';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofStoppedBy {
    /// A mount stands at the slot's path and no record is filed under its name.
    AMountNoRecordNames,
    /// The record filed under the slot's name names another path.
    ARecordOfAnotherPath,
    /// The mount table,
    /// or the mount at the slot,
    /// could not be read.
    AMountThatCannotBeRead,
    /// Any other reading that proved nothing.
    AnotherReading,
}

impl ProofStoppedBy {
    /// What:
    ///  Whether a removal of this slot must be refused for this reading.
    /// Why:
    ///  The owner's answer of 2026-11-26 refuses the removal where nothing proves
    ///  what a slot holds,
    ///  because Git's own removal would delete the registration and the underlay
    ///  folder while a clone's files may stand behind the mount;
    ///  a plan nobody could read and a phase that disagrees with its mount are
    ///  readings the answer does not name,
    ///  so they stay Git's until the owner says otherwise.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const refuses = (stopped) => stopped !== 'another-reading';
    /// ```
    pub fn refuses_a_removal(self) -> bool {
        return match self {
            ProofStoppedBy::AMountNoRecordNames
            | ProofStoppedBy::ARecordOfAnotherPath
            | ProofStoppedBy::AMountThatCannotBeRead => true,
            ProofStoppedBy::AnotherReading => false,
        };
    }
}

/// What:
///  Which of the route's states one slot is in.
/// Why:
///  Each answer has one next safe action,
///  and the answer a reader cannot prove is its own value rather than a guess
///  between two others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotRouteState = { kind: 'not-a-managed-clone' } | { kind: 'pending-add'; phase } | { kind: 'pending-removal'; plan } | { kind: 'known-unmounted-clone'; cloneDataset } | { kind: 'mounted-managed-clone'; cloneDataset } | { kind: 'cannot-prove'; reason; stoppedBy };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotRouteState {
    /// No record is filed under the slot and no mount stands there.
    NotAManagedClone,
    /// The add that made the slot has not finished,
    /// or its own fallback has not.
    PendingAdd {
        /// How far the record says the add got.
        phase: Phase,
        /// The clone dataset the record names.
        clone_dataset: String,
    },
    /// A removal's destruction plan names this clone.
    PendingRemoval {
        /// The plan's file.
        plan: PathBuf,
        /// The clone dataset the record names.
        clone_dataset: String,
    },
    /// The record says the worktree was verified,
    /// and no row of this mount table mounts the clone at its slot.
    KnownUnmountedClone {
        /// The clone dataset the record names.
        clone_dataset: String,
    },
    /// The record says the worktree was verified,
    /// and the clone's own root is mounted at its slot.
    MountedManagedClone {
        /// The clone dataset the record names.
        clone_dataset: String,
    },
    /// Nothing the readings hold proves which of these the slot is.
    CannotProve {
        /// Why no state could be proved,
        /// in the words of the source that refused.
        reason: String,
        /// Which reading stopped the proof.
        stopped_by: ProofStoppedBy,
    },
}

impl SlotRouteState {
    /// The state in the words a report prints.
    pub fn word(&self) -> &'static str {
        return match self {
            SlotRouteState::NotAManagedClone => "not a managed clone",
            SlotRouteState::PendingAdd {
                ..
            } => "pending add",
            SlotRouteState::PendingRemoval {
                ..
            } => "pending removal",
            SlotRouteState::KnownUnmountedClone {
                ..
            } => "known unmounted clone",
            SlotRouteState::MountedManagedClone {
                ..
            } => "mounted managed clone",
            SlotRouteState::CannotProve {
                ..
            } => "cannot be proved",
        };
    }
}

/// What:
///  Why a slot's record and its mounts disagree.
/// Why:
///  Three arms produce the same answer,
///  and one sentence keeps the report from wording them three ways.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const disagree = (phase: Phase, mount: string) => `the binding record says phase=${phase} and ${mount} is mounted`;
/// ```
fn disagree(
    phase: Phase,
    mounted: &str,
) -> String {
    return format!(
        "the binding record says `phase={}`, and {mounted}",
        phase.word()
    );
}

/// What:
///  The state one slot is in.
/// Why:
///  The order of the arms is the design's order of proof:
///  an unreadable record or plan proves nothing,
///  a plan that names the clone is a removal in flight,
///  the phase says how far the add got,
///  and the mount table decides whether a clone the record calls mounted is
///  mounted here,
///  somewhere else,
///  or nowhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classify_slot(readings: SlotReadings): SlotRouteState;
/// ```
pub fn classify_slot(readings: &SlotReadings) -> SlotRouteState {
    let record = match &readings.binding {
        BindingReading::Unreadable {
            reason,
        } => {
            return SlotRouteState::CannotProve {
                reason: format!(
                    "the record filed under `{}` cannot be read: {reason}",
                    readings.slot
                ),
                // A record nobody may act on is the removal's own early refusal, so this answer is no other reading's.
                stopped_by: ProofStoppedBy::AnotherReading,
            };
        }
        BindingReading::Absent => {
            return match &readings.slot_mount {
                SlotMount::Unreadable {
                    reason,
                } => SlotRouteState::CannotProve {
                    reason: format!(
                        "no record is filed under `{}`, and the mount at its path could not be read: {reason}",
                        readings.slot
                    ),
                    stopped_by: ProofStoppedBy::AMountThatCannotBeRead,
                },
                SlotMount::Something {
                    described,
                } => SlotRouteState::CannotProve {
                    reason: format!(
                        "no record is filed under `{}`, and {described}, so nothing proves whether the route or a person put that mount there",
                        readings.slot
                    ),
                    stopped_by: ProofStoppedBy::AMountNoRecordNames,
                },
                SlotMount::Nothing => SlotRouteState::NotAManagedClone,
            };
        }
        BindingReading::Record(record) => record,
    };
    match record.slot_path() {
        Some(recorded) if recorded == readings.slot_path => {}
        Some(recorded) => {
            return SlotRouteState::CannotProve {
                reason: format!(
                    "the record filed under `{}` names the slot {}, so it is not this slot's record",
                    readings.slot,
                    recorded.display()
                ),
                stopped_by: ProofStoppedBy::ARecordOfAnotherPath,
            };
        }
        None => {
            return SlotRouteState::CannotProve {
                reason: format!(
                    "the record filed under `{}` holds a `slot=` that is not a path, so the slot it belongs to is unproved",
                    readings.slot
                ),
                stopped_by: ProofStoppedBy::AnotherReading,
            };
        }
    }
    match &readings.plan {
        PlanReading::NamesTheClone {
            plan,
        } => {
            return SlotRouteState::PendingRemoval {
                plan: plan.clone(),
                clone_dataset: record.clone_dataset.clone(),
            };
        }
        PlanReading::Unreadable {
            plan,
            reason,
        } => {
            return SlotRouteState::CannotProve {
                reason: format!(
                    "the destruction plan at {} could not be read ({reason}), so nothing proves no removal is in flight for this slot",
                    plan.display()
                ),
                stopped_by: ProofStoppedBy::AnotherReading,
            };
        }
        PlanReading::NotAsked => {
            return SlotRouteState::CannotProve {
                reason: format!(
                    "the record filed under `{}` stands, and the plan folder was not read, so nothing proves no removal is in flight for this slot",
                    readings.slot
                ),
                stopped_by: ProofStoppedBy::AnotherReading,
            };
        }
        PlanReading::NoPlan => {}
    }
    let mount: CloneMount = clone_mount_of(
        &readings.mounts,
        &readings.slot_mount,
        record.clone_dataset.as_str(),
        readings.slot_path.as_path(),
    );
    return match record.phase {
        Phase::Claimed
        | Phase::Slot
        | Phase::Registered
        | Phase::Snapshotted
        | Phase::Cloned
        | Phase::Settling => match mount {
            CloneMount::Nowhere => SlotRouteState::PendingAdd {
                phase: record.phase,
                clone_dataset: record.clone_dataset.clone(),
            },
            CloneMount::Unreadable {
                reason,
            } => SlotRouteState::CannotProve {
                reason: format!(
                    "the binding record says `phase={}`, and the mount table could not be read: {reason}",
                    record.phase.word()
                ),
                // A clone exists from `cloned` on, so where it is mounted is a whole worktree that table would show;
                //  an earlier phase has no clone for a removal to delete inside, and its removal is the add's way forward.
                stopped_by: match record.phase {
                    Phase::Cloned | Phase::Settling => ProofStoppedBy::AMountThatCannotBeRead,
                    _ => ProofStoppedBy::AnotherReading,
                },
            },
            CloneMount::AtItsSlot => SlotRouteState::CannotProve {
                reason: disagree(record.phase, "the clone's own root is mounted at the slot"),
                stopped_by: ProofStoppedBy::AnotherReading,
            },
            CloneMount::Elsewhere {
                described,
            } => SlotRouteState::CannotProve {
                reason: disagree(record.phase, &format!("{described} is mounted")),
                stopped_by: ProofStoppedBy::AnotherReading,
            },
            CloneMount::SomethingElseAtTheSlot {
                described,
            } => SlotRouteState::CannotProve {
                reason: disagree(record.phase, &format!("{described} stands at the slot")),
                stopped_by: ProofStoppedBy::AnotherReading,
            },
        },
        Phase::Mounted | Phase::Filled => match mount {
            CloneMount::AtItsSlot => SlotRouteState::MountedManagedClone {
                clone_dataset: record.clone_dataset.clone(),
            },
            CloneMount::Nowhere => SlotRouteState::KnownUnmountedClone {
                clone_dataset: record.clone_dataset.clone(),
            },
            CloneMount::Elsewhere {
                described,
            } => SlotRouteState::CannotProve {
                reason: format!(
                    "the binding record says `phase={}`, and the clone is mounted somewhere else ({described}), so nothing proves the mount at its slot is the clone's",
                    record.phase.word()
                ),
                stopped_by: ProofStoppedBy::AnotherReading,
            },
            CloneMount::SomethingElseAtTheSlot {
                described,
            } => SlotRouteState::CannotProve {
                reason: format!(
                    "the binding record says `phase={}`, and {described} stands at the slot while the clone itself is not mounted there",
                    record.phase.word()
                ),
                stopped_by: ProofStoppedBy::AnotherReading,
            },
            CloneMount::Unreadable {
                reason,
            } => SlotRouteState::CannotProve {
                reason: format!(
                    "the binding record says `phase={}`, and the mount table could not be read: {reason}",
                    record.phase.word()
                ),
                // A record this far along names a clone, so the table would say whether it is mounted at the slot.
                stopped_by: ProofStoppedBy::AMountThatCannotBeRead,
            },
        },
    };
}

/// What:
///  The slot a path names,
///  or why it names none.
/// Why:
///  A record is keyed by a plain slot name,
///  so a path whose last component is no plain name cannot hold one,
///  and the refusal is the record name's own rule rather than a second copy of
///  it.
///
/// # Errors
/// The path has no last component or it is no plain slot name,
/// with the rule's own sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = slotNameOf(path); // throws the rule's sentence
/// ```
pub fn slot_name_of(path: &Path) -> Result<String, String> {
    return crate::worktree_clone_binding_record::registration_home::slot_of(path);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_state_classification_tests.rs"]
mod tests;
