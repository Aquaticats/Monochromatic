//! What:
//!  The lines the settling command prints:
//!  the slot and the record it confirmed,
//!  what each read answered,
//!  which read could not prove the clone's absence and why,
//!  the exact removals in order,
//!  and the one sentence about what the wrapper can no longer prove.
//! Why:
//!  A command that removes evidence owes the person everything it read before
//!  the removals are made
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command",
//!  "What it asks the person to confirm").
//!  The confirmation is the two flags' values and never a prompt,
//!  so these lines are the report of what those values were checked against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = evidenceLines(evidence).concat(removalLines(steps), [lastLine(slot)]);
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_record_settling::settling_absence::TheClonesPresence;
use crate::worktree_clone_record_settling::settling_record::TheRecord;
use crate::worktree_clone_record_settling::settling_slot::{ThePointer, TheRegistration, TheSlot};
use crate::worktree_clone_record_settling::settling_steps::SettlingStep;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;

/// What:
///  Everything one run read,
///  which the lines print.
/// Why:
///  The lines are one list so a refusal and a completed run print the same
///  evidence,
///  and a control asserts a sentence rather than three folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Evidence = { record; presence; slot; cleanup; plan };
/// ```
pub struct TheEvidence<'a> {
    /// The record the two flags confirmed.
    pub record: &'a TheRecord,
    /// The pool's answer about the recorded clone.
    pub presence: &'a TheClonesPresence,
    /// What stands at the slot.
    pub slot: &'a TheSlot,
    /// The origin cleanup record that stands,
    /// when one does.
    pub cleanup: Option<&'a OriginCleanupRecord>,
    /// The destruction plan that stands,
    /// when one does.
    pub plan: Option<&'a DestructionPlan>,
}

/// The line naming the slot and the record the two flags confirmed.
fn the_record_line(record: &TheRecord, slot: &TheSlot) -> String {
    let identity: String = match record.slot_identity {
        Some(identity) => identity.encode(),
        None => String::from("none recorded"),
    };
    return format!(
        "the binding record at `phase={}` names the slot `{}` (underlay identity `{identity}`), the clone `{}` (GUID `{}`, creation transaction group {}) and the origin snapshot `{}` (GUID `{}`, creation transaction group {}); both flags confirm this record, and no other slot's.",
        record.phase.word(),
        slot.path.display(),
        record.recorded.clone_dataset,
        record.recorded.clone_guid.file_name(),
        record.recorded.clone_createtxg,
        record.recorded.origin_snapshot,
        record.recorded.origin_snapshot_guid.file_name(),
        record.recorded.origin_snapshot_createtxg
    );
}

/// The line naming what stands at the slot,
/// and whether a checkout has begun.
fn the_slot_line(slot: &TheSlot) -> String {
    let pointer: String = match slot.pointer {
        ThePointer::Absent => String::from("no `.git` stands at the slot"),
        ThePointer::NamesThis => String::from("the slot's `.git` names this add's registration"),
        ThePointer::NamesAnother { .. } => String::from("the slot's `.git` names another registration"),
    };
    let registration: String = match slot.registration {
        TheRegistration::Gone => String::from("and the registration the record names is gone, as a kill inside this command leaves it"),
        TheRegistration::Stands { .. } => format!("and the registration the record names stands with the identity the record holds and {} entries of its own", slot.registration_entries.len()),
        TheRegistration::Differs { .. } => String::from("and the registration the record names stands with another identity"),
    };
    let entries: String = match slot.entries_beside_the_pointer.len() {
        0 => String::from("the slot holds nothing beside `.git`, so no checkout has begun and the abandoned registration is this command's to remove"),
        count => format!("the slot holds {count} entr(ies) beside `.git`, so a checkout has begun and this command removes only the records and leaves the worktree"),
    };
    return format!("no mount stands at the slot; {pointer}, {registration}; {entries}.");
}

/// The line naming what the pool answered,
/// and the evidence when it could not prove the absence.
fn the_pool_line(record: &TheRecord, presence: &TheClonesPresence) -> String {
    return match presence {
        TheClonesPresence::ProvedGone => format!(
            "the pool answers neither the recorded name `{}` nor its GUID `{}` under the pinned parent, so the clone's absence is proved.",
            record.recorded.clone_dataset,
            record.recorded.clone_guid.file_name()
        ),
        TheClonesPresence::Unprovable { evidence } => format!(
            "the pool could not prove the recorded clone's absence, and that refusal is the reason this command exists: {evidence}"
        ),
        TheClonesPresence::Stands { name, guid } => format!("the clone `{name}` answers the pool with its recorded GUID `{}`, so it stands and nothing is settled.", guid.file_name()),
        TheClonesPresence::TheNameHoldsAnother { name, guid } => {
            format!("the recorded name `{name}` answers the pool with GUID `{}`, which is not the recorded clone's, so the name was reused.", guid.file_name())
        }
        TheClonesPresence::AnswersElsewhere { found } => format!("the recorded clone's GUID answers the pool at `{found}`, so the clone stands under another name."),
    };
}

/// The line naming a standing origin cleanup record,
/// which this command compares and leaves.
fn the_cleanup_line(cleanup: Option<&OriginCleanupRecord>) -> String {
    let Some(record) = cleanup else {
        return String::from("no origin cleanup record stands, so no origin snapshot is owed a cleanup this command would print.");
    };
    return format!(
        "an origin cleanup record stands for `{}` (state `{}`, `{}`) and names the same objects as the binding; it is left standing, because the origin's cleanup is the origin look's business and no settling command removes or rewrites it.",
        record.origin_snapshot,
        record.state.word(),
        record.block.map_or_else(|| return String::from("no block recorded"), |block| return block.detail())
    );
}

/// The line naming a standing plan and the names it recorded as destroyed.
fn the_plan_line(standing: Option<&DestructionPlan>) -> String {
    let Some(plan) = standing else {
        return String::from("no destruction plan stands for this clone, so no cancelled removal left a loss for this command to print.");
    };
    let names: Vec<String> = crate::worktree_clone_record_settling::settling_standing_records::the_names_the_plan_lost(plan);
    return format!(
        "a destruction plan stands for this clone with its head `{}` and records these names as destroyed, which are printed before the plan goes: {}.",
        plan.head.word(),
        names.join(", ")
    );
}

/// What:
///  The evidence lines,
///  in the order the reads were made.
/// Why:
///  The reads have one order and the lines follow it,
///  so a person reads what the command read before it removed anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = evidenceLines(evidence);
/// ```
pub fn evidence_lines(evidence: &TheEvidence<'_>) -> Vec<String> {
    return vec![
        the_record_line(evidence.record, evidence.slot),
        the_slot_line(evidence.slot),
        the_pool_line(evidence.record, evidence.presence),
        the_cleanup_line(evidence.cleanup),
        the_plan_line(evidence.plan),
    ];
}

/// What:
///  One line for each removal,
///  numbered in the order the run makes them.
/// Why:
///  The exact removals in order are what the person confirms,
///  and a numbered line per step is the order itself rather than a summary of
///  it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = removalLines(steps);
/// ```
pub fn removal_lines(steps: &[SettlingStep]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::with_capacity(steps.len() + 1);
    lines.push(String::from("this command removes, in this order:"));
    for (number, step) in steps.iter().enumerate() {
        lines.push(format!("  {}. {}", number + 1, step.words()));
    }
    return lines;
}

/// What:
///  What the slot becomes once the removals are made,
///  said plainly.
/// Why:
///  The project chose this command knowing its cost,
///  and the sentence names that cost to the person who confirms it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = lastLine('topic');
/// ```
pub fn last_line(slot: &str) -> String {
    return format!("after these removals the wrapper can no longer prove which clone the slot `{slot}` held: the records that named it are gone, and only the pool's own history remembers it.");
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_lines_tests.rs"]
mod tests;
