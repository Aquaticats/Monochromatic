//! What:
//!  The lines the merge device writes,
//!  one per answer a taker can give.
//! Why:
//!  Every answer says whether the pass ran,
//!  what the device did with the mark,
//!  and which trigger covers a request this invocation left standing,
//!  because a trigger that asked for a pass and got none is the fact a person
//!  reads after a clone waited
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What a taker does",
//!  "What the marks guarantee,
//! and what they do not",
//!  and V7's first option for a record that could not be read).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.log(extraPassLine()); // 'another-pass-is-due was there … one more pass, and one more is the most one invocation runs'
//! ```

use crate::single_owner_claim::BusyOwner;

/// What:
///  The line a pass that found the run claim held is written as.
/// Why:
///  The trigger did no work,
///  so the line names the file the other pass holds and the request this
///  trigger left behind,
///  and no sentence suggests anything is wrong:
///  a held trigger is the answer the device exists to give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = heldLine(owner);
/// ```
pub(crate) fn held_line(owner: &BusyOwner) -> String {
    return format!(
        "another event-driven pass holds the run claim (process {} holds {}), so this trigger ran no pass; it wrote {} and that pass, or the next trigger, covers the request",
        owner.pid,
        owner.path.display(),
        crate::worktree_clone_mount_sweep_merge::sweeps_folder::PASS_MARK_NAME
    );
}

/// What:
///  The line an invocation writes when another pass held the claim on its
///  second attempt too.
/// Why:
///  Nothing ran,
///  and the mark stands,
///  so the sentence names the trigger that will cover the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = markLeftWithNoPassLine(owner);
/// ```
pub(crate) fn mark_left_with_no_pass_line(owner: &BusyOwner) -> String {
    return format!(
        "another event-driven pass holds the run claim (process {} holds {}), so this trigger ran no pass; another-pass-is-due stands for the next trigger, whose pass covers the request",
        owner.pid,
        owner.path.display()
    );
}

/// What:
///  The line an invocation writes when it ran a second pass because a trigger
///  marked.
/// Why:
///  The extra pass is a cost the project adopted with its veto open (V12),
///  so the line says why it ran and that one extra pass is the most one
///  invocation runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = extraPassLine();
/// ```
pub(crate) fn extra_pass_line() -> String {
    return String::from(
        "another-pass-is-due was there when this pass released the run claim, so this invocation ran two more passes; one more pass is the most one invocation runs",
    );
}

/// What:
///  The line an invocation writes when a trigger marked after its last look.
/// Why:
///  The mark stands for the next trigger,
///  which is limitation L3 of the design and not a lost request,
///  and the line has to say so rather than leave a person reading a count of
///  passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = markLeftStandingLine();
/// ```
pub(crate) fn mark_left_standing_line() -> String {
    return String::from(
        "another-pass-is-due stands after this invocation's last look, so a trigger asked for a pass after it ran; that request waits for the next trigger, and the mark never loses it",
    );
}

/// What:
///  The line a pass that ran without the device is written as.
/// Why:
///  V7's first option keeps an unreadable cost record from refusing a pass,
///  so the sentence says which record could not be read,
///  that the pass ran without it,
///  and that the per-slot claim is what keeps the work safe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = device_unreadable_line('the run claim … could not be read');
/// ```
pub(crate) fn device_unreadable_line(reason: &str) -> String {
    return format!(
        "the sweep merge device could not be read ({reason}); this pass ran without the run claim or the mark, because the device is cost and the per-slot claim is safety"
    );
}

/// What:
///  The line a pass writes when the mark could not be looked at.
/// Why:
///  A mark that cannot be read still says a trigger asked,
///  so the device answers a request rather than dropping it,
///  and the person reads what could not be read beside that answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = mark_unreadable_line(reason);
/// ```
pub(crate) fn mark_unreadable_line(reason: &str) -> String {
    return format!("{reason}, so this pass treated the mark as a request it still has to answer");
}

/// What:
///  The line a pass writes when the mark it meant to drop is still there.
/// Why:
///  A mark that could not be removed makes the next trigger's pass run for
///  nothing,
///  which is a cost a person has to be able to see,
///  and the pass itself did its work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = mark_not_removed_line(reason);
/// ```
pub(crate) fn mark_not_removed_line(reason: &str) -> String {
    return format!(
        "{reason}, so the next trigger's pass runs for a request this pass already covered"
    );
}

/// What:
///  The line a pass writes when the claim it gave back could not be released.
/// Why:
///  A claim that names this process stops counting as alive when the process
///  ends,
///  so the next trigger takes the claim without help,
///  and the sentence says what happened without offering a repair nobody needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = claim_not_released_line(reason);
/// ```
pub(crate) fn claim_not_released_line(reason: &str) -> String {
    return format!(
        "{reason}; this process names the claim until it ends, and the next trigger takes the claim then"
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "merge_lines_tests.rs"]
mod tests;
