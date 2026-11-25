//! What:
//!  Every sentence the finish pass prints about a plan it removed,
//!  and about the records it removed with one.
//! Why:
//!  A pass that removed a plan says which records went and in which order it proved them gone,
//!  so the line is the person's trace of a removal no command of theirs ran
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "When it is removed";
//!  `doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  steps 11 to 14).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(theRemovalWasFinished(plan, slot, files));
//! ```

/// What:
///  The line for a removal this pass finished:
///  the registration the settle answered for,
///  the files it removed in their order,
///  and the plan and its folder.
/// Why:
///  The line is the only trace of the tail the list gives steps 11 to 14,
///  so it names every record the pass removed and whether a registration was settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theRemovalWasFinished(plan, slot, files, settled);
/// ```
pub fn the_removal_was_finished(plan: &std::path::Path, slot: &str, files: &[String], settlement: &str) -> String {
    let removed: String = match files.is_empty() {
        true => String::from("no record of the binding stood"),
        false => format!("then removed {}", files.join(", ")),
    };
    return format!(
        "Finished the interrupted removal of the slot `{slot}`: {settlement}, {removed}, and removed the plan {} last. The claim is released again, and nothing else was touched.",
        plan.display()
    );
}

/// What:
///  The line for a plan removed after a reused name,
///  beside no registration of this plan's and another clone's binding.
/// Why:
///  By GUID this plan's clone is gone,
///  a dataset now answers its name,
///  and the binding at this slot names that other clone,
///  so the plan is the only record left of a removal that already finished
///  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "The finish pass that settles a registration").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlanWasRemovedAfterAReuse(plan, clone, recorded, live);
/// ```
pub fn the_plan_was_removed_after_a_reuse(plan: &std::path::Path, slot: &str, clone: &str, recorded_guid: &str, live_guid: &str) -> String {
    return format!(
        "Removed the last removal plan of the slot `{slot}` ({}): `{clone}` now answers GUID {live_guid} and this plan's clone GUID {recorded_guid} is gone, no registration of this plan stands, and the binding at this slot names that other clone, so the plan was the only record left. Its binding, its shared registration and everything at the slot were left untouched.",
        plan.display()
    );
}

/// What:
///  The half of a finished removal's line that says what the settle did.
/// Why:
///  A settle answers three ways,
///  and a person reading the line needs to know whether a registration and a lock moved,
///  whether another removal's registration was left,
///  or whether none stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settlement = theSettlementOf(answer);
/// ```
pub fn the_settlement_of(answer: &crate::worktree_clone_removal_proof_and_plan::removal_settle::SettleAnswer) -> String {
    if answer.removed_the_registration {
        return match (answer.wrote_the_lost_lock, answer.released_the_lock) {
            (true, _) => String::from("the managed lock was written again and released, and the registration and the slot's pointer were settled"),
            (false, true) => String::from("the managed lock was released, and the registration and the slot's pointer were settled"),
            (false, false) => String::from("the registration and the slot's pointer were settled"),
        };
    }
    return match answer.released_the_lock {
        true => String::from("the managed lock was released and no registration of this plan stood"),
        false => String::from("no registration of this plan stood under its own registration name"),
    };
}

/// What:
///  The line for a plan removed beside a new slot's binding.
/// Why:
///  A binding that names another clone is a new slot's work rather than this removal's leftover,
///  so it is left and the plan alone is removed as the last witness of the old clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlanWasRemovedBesideANewBinding(plan, slot);
/// ```
pub fn the_plan_was_removed_beside_a_new_binding(plan: &std::path::Path, slot: &str) -> String {
    return format!(
        "Removed the last removal plan of the slot `{slot}` ({}): the binding at the slot names another clone and this plan's clone is gone by its GUID, so the binding was left with the new clone it describes and only the plan went.",
        plan.display()
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "pass_done_lines_tests.rs"]
mod tests;
