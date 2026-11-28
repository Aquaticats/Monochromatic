//! What:
//!  The two sentences a slot's report is made of:
//!  what the readings saw,
//!  and the one thing that is safe to do next in each state.
//! Why:
//!  The design asks for the slot,
//!  the dataset and the next safe action instead of treating absent files as
//!  deleted tracked files or offering an ordinary add over them
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees"),
//!  and the add's own list of commands stays the authority for what resumes an
//!  unfinished add
//!  (`worktree_clone_binding_record/add_state_commands.rs`).
//!  A person reads these sentences,
//!  so each names the state it found,
//!  what it means for their files,
//!  and every action that is safe from it.
//!  The actions name the commands and the property the design defines,
//!  and none of them guesses a pool answer:
//!  the state that needs the pool says so and sends the reader to the command
//!  that reads it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const action = nextSafeAction(state, record, readings);
//! const seen = evidence(readings);
//! ```

use crate::worktree_clone_binding_record::add_state_commands::{
    SETTLING_COMMAND_WORDS,
    THE_REMOVALS_WORDS,
};
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::SettleReason;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use crate::worktree_clone_slot_state::slot_state_classification::SlotRouteState;
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    IntentFacts,
    SlotReadings,
};

/// The placeholder a report leaves where a person must copy a value the record holds.
const GUIDE_DIGITS_PLACEHOLDER: &str = "<the last four decimal digits of cloneGuid=>";

/// The placeholder a mount request's nonce is written as.
const NONCE_PLACEHOLDER: &str = "<32 lowercase hex digits>";

/// The unit whose journal explains a sweep that has not attached a requested mount.
const SWEEP_UNIT: &str = "reconcile-git-worktree-clone-mounts.service";

/// What:
///  The line that says whether the Git registration the record names stands.
/// Why:
///  A registration is what makes a worktree Git's,
///  and a clone whose registration is gone needs a reader's attention before a
///  reattach,
///  so the report says which of the two it found rather than leaving it out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = registration_sentence(readings.registrationStands);
/// ```
pub fn registration_sentence(registration_stands: Option<bool>) -> String {
    return match registration_stands {
        Some(true) => String::from(
            "The Git registration the record names still stands, so `git worktree` lists this worktree.",
        ),
        Some(false) => String::from(
            "The Git registration the record names is gone, so `git worktree` no longer lists this worktree.",
        ),
        None => String::from("The record names no Git registration yet."),
    };
}

/// What:
///  What one clone's intent says about the request standing on it.
/// Why:
///  An unmounted clone is a different situation when a mount is already asked
///  for than when nothing is,
///  and an intent no reader can parse is neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked = intent_sentence(dataset, intent);
/// ```
fn intent_sentence(
    clone_dataset: &str,
    intent: &IntentFacts,
) -> String {
    return match intent {
        IntentFacts::Unasked => String::from(
            " No mount request was asked of the clone, because the record names no clone dataset yet.",
        ),
        IntentFacts::Absent => format!(
            " No mount request stands on the clone. To use the worktree again, write one on it (`zfs set {DESIRED_STATE_PROPERTY}=v1:{NONCE_PLACEHOLDER}:mount {clone_dataset}`) and let the handler's next sweep attach it; to be rid of it: {}.",
            THE_REMOVALS_WORDS
        ),
        IntentFacts::Mounting {
            nonce,
        } => format!(
            " A mount request under nonce `{nonce}` stands on the clone, and no sweep has attached it yet: the handler's next sweep does, and `journalctl -u {SWEEP_UNIT}` says when one ran. Do not add here and do not move the slot while it waits."
        ),
        IntentFacts::Unmounting {
            nonce,
        } => format!(
            " The clone's own intent keeps it unmounted under nonce `{nonce}`, so this is the state a person asked for. To use the worktree again, write a mount request back on the clone (`zfs set {DESIRED_STATE_PROPERTY}=v1:{NONCE_PLACEHOLDER}:mount {clone_dataset}`) and let the handler's next sweep attach it; to be rid of it: {}.",
            THE_REMOVALS_WORDS
        ),
        IntentFacts::Inherited {
            from,
        } => format!(
            " The clone's property is inherited from `{from}` and authorizes nothing, so no mount request stands. To use the worktree again, write a request on the clone itself (`zfs set {DESIRED_STATE_PROPERTY}=v1:{NONCE_PLACEHOLDER}:mount {clone_dataset}`) and let the handler's next sweep attach it; to be rid of it: {}.",
            THE_REMOVALS_WORDS
        ),
        IntentFacts::Unreadable {
            reason,
        } => format!(
            " The clone's mount intent cannot be read, so no request stands that a person can rely on: {reason} Write the intent again in the designed form on `{clone_dataset}`; to be rid of it: {}.",
            THE_REMOVALS_WORDS
        ),
    };
}

/// What:
///  The clause for an add the sending phase left unfinished.
/// Why:
///  The add's own list of commands is what resumes it,
///  and the record's `settleReason=` says which step of the fallback stopped,
///  so the report names the one command for the shape rather than its own
///  invention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resume = unfinished_add(dataset, slot, phase, settleReason);
/// ```
fn unfinished_add(
    clone_dataset: &str,
    slot: &str,
    settle_reason: Option<SettleReason>,
) -> String {
    let stopping: String = match settle_reason {
        Some(reason) => format!(" (settle reason `{}`)", reason.word()),
        None => String::new(),
    };
    return format!(
        "do not run Git in this folder and do not add here: the add that made this slot did not finish at `phase=settling`{stopping}. Read the record and take the command its state names: run the add again at `{slot}` while its clone is proved gone and the registration stands, run `git worktree remove {slot}` while the clone still stands, or, when the clone's absence cannot be proved or the registration is gone, the settling command (`{SETTLING_COMMAND_WORDS} {slot} --confirm-removal-of {slot} --confirm-clone-guid {GUIDE_DIGITS_PLACEHOLDER}`) removes the records once you have confirmed the slot and the record's own clone GUID. The clone dataset is `{clone_dataset}`."
    );
}

/// What:
///  The one next safe action for one state,
///  said in terms of what a person may do to their files.
/// Why:
///  Each arm is the design's own answer for that state,
///  and an action that needs the pool says so rather than presenting a pool
///  answer the reader does not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const action = next_safe_action(state, record, readings);
/// ```
pub fn next_safe_action(
    state: &SlotRouteState,
    readings: &SlotReadings,
) -> String {
    let record: Option<&BindingRecord> = match &readings.binding {
        BindingReading::Record(record) => Some(record),
        BindingReading::Absent
        | BindingReading::Unreadable {
            ..
        } => None,
    };
    return match state {
        SlotRouteState::NotAManagedClone => String::from(
            "nothing: no binding record is filed under this slot and nothing is mounted at it, so the wrapper's clone route holds nothing here; add, move or remove it as an ordinary worktree.",
        ),
        SlotRouteState::PendingAdd {
            phase,
            clone_dataset,
        } => {
            let settling: bool =
                phase == &crate::worktree_clone_binding_record::binding_words::Phase::Settling;
            let reason: Option<SettleReason> = record.and_then(|found| return found.settle_reason);
            if settling {
                let sentence: String =
                    unfinished_add(clone_dataset, readings.slot.as_str(), reason);
                return format!(
                    "{sentence} {}",
                    registration_sentence(readings.registration_stands)
                );
            }
            return format!(
                "do not run Git in this folder and do not add here: the add that made this slot is unfinished at `phase={}`. Run the add again at `{}`; it reads the record and carries on from the step the phase names, and the clone dataset is `{clone_dataset}`. {}",
                phase.word(),
                readings.slot,
                registration_sentence(readings.registration_stands)
            );
        }
        SlotRouteState::PendingRemoval {
            plan,
            ..
        } => format!(
            "do not run Git in this folder and do not add here: a removal's destruction plan at {} names this clone, so a removal is in flight or was killed. A `git worktree remove` of this slot runs the wrapper's own removal, which reads the plan, proves every name the plan records again, sends no request for a name the plan records as destroyed, carries the loss forward and ends the plan with the removal's own records; every spelling takes that path, `-f -f` among them, and an add here is refused while the plan stands. Read the plan's own file for the names it records as destroyed. {}",
            plan.display(),
            registration_sentence(readings.registration_stands)
        ),
        SlotRouteState::KnownUnmountedClone {
            clone_dataset,
        } => {
            let asked: String = intent_sentence(clone_dataset, &readings.intent);
            return format!(
                "the clone is unmounted, and nothing under the slot is a person's file: the checkout lives on `{clone_dataset}`, and no tracked file is deleted. Do not run Git in this folder until the clone is mounted again. {asked} {}",
                registration_sentence(readings.registration_stands)
            );
        }
        SlotRouteState::MountedManagedClone {
            clone_dataset,
        } => format!(
            "nothing: `{clone_dataset}` is mounted at its slot, so run Git in the worktree as usual. The slot is fixed and `git worktree move` is refused for it; to be rid of it: {}.",
            THE_REMOVALS_WORDS
        ),
        SlotRouteState::CannotProve {
            reason,
            ..
        } => format!(
            "nothing until the reason is resolved: {reason} Do not add here, do not move or remove the slot, and do not write a mount request; read the record, the mount table or the folder the reason names, then run the command again."
        ),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_state_next_actions_tests.rs"]
mod tests;
