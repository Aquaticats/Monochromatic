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

use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::SettleReason;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use crate::worktree_clone_slot_state::slot_state_classification::SlotRouteState;
use crate::worktree_clone_slot_state::slot_state_mounts::{CloneMount, MountRows, SlotMount, clone_mount_of};
use crate::worktree_clone_slot_state::slot_state_readings::{BindingReading, IntentFacts, PlanReading, SlotReadings};

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
        Some(true) => String::from("The Git registration the record names still stands, so `git worktree` lists this worktree."),
        Some(false) => String::from("The Git registration the record names is gone, so `git worktree` no longer lists this worktree."),
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
fn intent_sentence(clone_dataset: &str, intent: &IntentFacts) -> String {
    return match intent {
        IntentFacts::Unasked => String::from(" No mount request was asked of the clone, because the record names no clone dataset yet."),
        IntentFacts::Absent => format!(
            " No mount request stands on the clone. To use the worktree again, write one on it (`zfs set {DESIRED_STATE_PROPERTY}=v1:{NONCE_PLACEHOLDER}:mount {clone_dataset}`) and let the handler's next sweep attach it; to be rid of it, run the removal, which destroys the clone and its registration."
        ),
        IntentFacts::Mounting { nonce } => format!(
            " A mount request under nonce `{nonce}` stands on the clone, and no sweep has attached it yet: the handler's next sweep does, and `journalctl -u {SWEEP_UNIT}` says when one ran. Do not add here and do not move the slot while it waits."
        ),
        IntentFacts::Unmounting { nonce } => format!(
            " The clone's own intent keeps it unmounted under nonce `{nonce}`, so this is the state a person asked for. To use the worktree again, write a mount request back on the clone (`zfs set {DESIRED_STATE_PROPERTY}=v1:{NONCE_PLACEHOLDER}:mount {clone_dataset}`) and let the handler's next sweep attach it; to be rid of it, run the removal, which resumes with this intent in effect."
        ),
        IntentFacts::Inherited { from } => format!(
            " The clone's property is inherited from `{from}` and authorizes nothing, so no mount request stands. To use the worktree again, write a request on the clone itself (`zfs set {DESIRED_STATE_PROPERTY}=v1:{NONCE_PLACEHOLDER}:mount {clone_dataset}`) and let the handler's next sweep attach it; to be rid of it, run the removal."
        ),
        IntentFacts::Unreadable { reason } => format!(
            " The clone's mount intent cannot be read, so no request stands that a person can rely on: {reason} Write the intent again in the designed form on `{clone_dataset}`, or run the removal."
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
fn unfinished_add(clone_dataset: &str, slot: &str, settle_reason: Option<SettleReason>) -> String {
    let stopping: String = match settle_reason {
        Some(reason) => format!(" (settle reason `{}`)", reason.word()),
        None => String::new(),
    };
    return format!(
        "do not run Git in this folder and do not add here: the add that made this slot did not finish at `phase=settling`{stopping}. Read the record and take the command its state names: run the add again at `{slot}` while its clone is proved gone and the registration stands, run `git worktree remove {slot}` while the clone still stands, or run the settling command (`git monochromatic-git-policy-cli settle-worktree-clone-record {slot} --confirm-removal-of {slot} --confirm-clone-guid {GUIDE_DIGITS_PLACEHOLDER}`) when the clone's absence cannot be proved or the registration is gone. The clone dataset is `{clone_dataset}`."
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
pub fn next_safe_action(state: &SlotRouteState, readings: &SlotReadings) -> String {
    let record: Option<&BindingRecord> = match &readings.binding {
        BindingReading::Record(record) => Some(record),
        BindingReading::Absent | BindingReading::Unreadable { .. } => None,
    };
    return match state {
        SlotRouteState::NotAManagedClone => String::from(
            "nothing: no binding record is filed under this slot and nothing is mounted at it, so the wrapper's clone route holds nothing here; add, move or remove it as an ordinary worktree.",
        ),
        SlotRouteState::PendingAdd { phase, clone_dataset } => {
            let settling: bool = phase == &crate::worktree_clone_binding_record::binding_words::Phase::Settling;
            let reason: Option<SettleReason> = record.and_then(|found| return found.settle_reason);
            if settling {
                let sentence: String = unfinished_add(clone_dataset, readings.slot.as_str(), reason);
                return format!("{sentence} {}", registration_sentence(readings.registration_stands));
            }
            return format!(
                "do not run Git in this folder and do not add here: the add that made this slot is unfinished at `phase={}`. Run the add again at `{}`; it reads the record and carries on from the step the phase names, and the clone dataset is `{clone_dataset}`. {}",
                phase.word(),
                readings.slot,
                registration_sentence(readings.registration_stands)
            );
        }
        SlotRouteState::PendingRemoval { plan, .. } => format!(
            "do not run Git in this folder and do not add here: a removal's destruction plan at {} names this clone, so a removal is in flight or was killed. Run the removal this plan belongs to; it resumes from the plan and ends it. {}",
            plan.display(),
            registration_sentence(readings.registration_stands)
        ),
        SlotRouteState::KnownUnmountedClone { clone_dataset } => {
            let asked: String = intent_sentence(clone_dataset, &readings.intent);
            return format!(
                "the clone is unmounted, and nothing under the slot is a person's file: the checkout lives on `{clone_dataset}`, and no tracked file is deleted. Do not run Git in this folder until the clone is mounted again. {asked} {}",
                registration_sentence(readings.registration_stands)
            );
        }
        SlotRouteState::MountedManagedClone { clone_dataset } => format!(
            "nothing: `{clone_dataset}` is mounted at its slot, so run Git in the worktree as usual. The slot is fixed and `git worktree move` is refused for it, and the removal is the way to be rid of it."
        ),
        SlotRouteState::CannotProve { reason } => format!(
            "nothing until the reason is resolved: {reason} Do not add here, do not move or remove the slot, and do not write a mount request; read the record, the mount table or the folder the reason names, then run the command again."
        ),
    };
}

/// What:
///  What the readings saw,
///  one line per source,
///  so a person can check the state against the evidence it came from.
/// Why:
///  A state without its evidence cannot be checked,
///  and the design's report exists to stop a reader treating an unmounted clone
///  as deleted files,
///  which only the named sources can rule out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seen = evidence(readings);
/// ```
pub fn evidence(readings: &SlotReadings) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    match &readings.binding {
        BindingReading::Absent => lines.push(format!("binding: no record is filed under `{}`", readings.slot)),
        BindingReading::Unreadable { reason } => lines.push(format!("binding: cannot be read: {reason}")),
        BindingReading::Record(record) => {
            let settling: String = record.settle_reason.map_or_else(String::new, |reason| return format!(" settleReason={}", reason.word()));
            lines.push(format!(
                "binding: phase={} cloneDataset=`{}` pool=`{}`{settling}",
                record.phase.word(),
                record.clone_dataset,
                record.pool
            ));
        }
    }
    match (&readings.binding, readings.registration_stands) {
        (BindingReading::Record(record), Some(stands)) => {
            let registration: String = record.registration_path().map_or_else(String::new, |path| return path.display().to_string());
            lines.push(format!(
                "registration: {}{registration}",
                if stands { "stands at " } else { "gone, and the record names " }
            ));
        }
        (BindingReading::Record(record), None) => match record.registration_path() {
            Some(path) => lines.push(format!("registration: not read, and the record names {}", path.display())),
            None => lines.push(String::from("registration: the record names none yet")),
        },
        (BindingReading::Absent | BindingReading::Unreadable { .. }, _) => {
            lines.push(String::from("registration: no record was read, so no registration was looked for"));
        }
    }
    match &readings.plan {
        PlanReading::NoPlan => lines.push(String::from("plan: no destruction plan names this clone")),
        PlanReading::NamesTheClone { plan } => lines.push(format!("plan: {} names this clone", plan.display())),
        PlanReading::Unreadable { plan, reason } => lines.push(format!("plan: {} cannot be read: {reason}", plan.display())),
    }
    match &readings.intent {
        IntentFacts::Unasked => lines.push(String::from("intent: not asked, because the record names no clone dataset yet")),
        IntentFacts::Absent => lines.push(String::from("intent: no request of the clone's own")),
        IntentFacts::Mounting { nonce } => lines.push(format!("intent: mount requested under nonce `{nonce}`")),
        IntentFacts::Unmounting { nonce } => lines.push(format!("intent: unmount requested under nonce `{nonce}`")),
        IntentFacts::Inherited { from } => lines.push(format!("intent: inherited from `{from}`, which authorizes nothing")),
        IntentFacts::Unreadable { reason } => lines.push(format!("intent: cannot be read: {reason}")),
    }
    let clone_dataset: Option<&str> = match &readings.binding {
        BindingReading::Record(record) => Some(record.clone_dataset.as_str()),
        BindingReading::Absent | BindingReading::Unreadable { .. } => None,
    };
    match clone_dataset {
        Some(dataset) => lines.extend(mount_evidence(readings, dataset)),
        None => lines.push(slot_mount_evidence(&readings.slot_mount, readings.slot_path.as_path())),
    }
    return lines;
}

/// What:
///  What the mount table and the slot's own mount say,
///  read against the clone dataset.
/// Why:
///  The class turns on this reading,
///  so the report prints the clone's rows and the slot's own mount instead of a
///  summary,
///  and a person can see whether the slot is the clone,
///  another mount,
///  or empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = mount_evidence(readings, cloneDataset);
/// ```
fn mount_evidence(readings: &SlotReadings, clone_dataset: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mount: CloneMount = clone_mount_of(&readings.mounts, &readings.slot_mount, clone_dataset, readings.slot_path.as_path());
    match &readings.mounts {
        MountRows::Unreadable { reason } => lines.push(format!("mount table: cannot be read: {reason}")),
        MountRows::Rows(rows) => {
            let clone_rows: Vec<&crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance> = rows.iter().filter(|row| return row.dataset == clone_dataset).collect();
            if clone_rows.is_empty() {
                lines.push(format!("mount table: no row mounts `{clone_dataset}`"));
            }
            lines.extend(clone_rows.iter().map(|row| return format!("mount table: clone mounted as {}", row.described())));
        }
    }
    lines.push(slot_mount_evidence(&readings.slot_mount, readings.slot_path.as_path()));
    match mount {
        CloneMount::AtItsSlot => lines.push(String::from("mount: the clone's own root stands at its slot")),
        CloneMount::Nowhere => lines.push(String::from("mount: the clone is not mounted anywhere this table shows")),
        CloneMount::Elsewhere { described } => lines.push(format!("mount: the clone is elsewhere ({described})")),
        CloneMount::SomethingElseAtTheSlot { described } => lines.push(format!("mount: {described} stands at the slot")),
        CloneMount::Unreadable { reason } => lines.push(format!("mount: cannot be read: {reason}")),
    }
    return lines;
}

/// What:
///  One line about the mount at the slot's own path.
/// Why:
///  An empty underlay and a mount a person put there are different states,
///  and only this reading tells them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = slot_mount_evidence(slotMount, slotPath);
/// ```
fn slot_mount_evidence(slot_mount: &SlotMount, slot_path: &std::path::Path) -> String {
    return match slot_mount {
        SlotMount::Nothing => format!("slot mount: nothing is mounted at {}", slot_path.display()),
        SlotMount::Something { described } => format!("slot mount: {described}"),
        SlotMount::Unreadable { reason } => format!("slot mount: cannot be read: {reason}"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_state_next_actions_tests.rs"]
mod tests;
