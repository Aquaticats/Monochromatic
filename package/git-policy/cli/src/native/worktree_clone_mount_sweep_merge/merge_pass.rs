//! What:
//!  One event-driven pass with the route-wide run claim,
//!  the mark and the one-extra-pass cap around it.
//! Why:
//!  Two triggers about one clone need one pass that reads the clone after both
//!  writes,
//!  so a pass runs under one claim and a contender marks instead of running:
//!  `run.<generation>`,
//!  then `another-pass-is-due`,
//!  then a second claim attempt that never waits
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What a taker does",
//!  and "What the marks guarantee,
//! and what they do not";
//!  veto entries V4 and V12,
//!  and the seventeen choices as the project was shown them,
//!  choices 8 and 17,
//!  in
//!  `doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`).
//!  The pass itself is the caller's:
//!  this device decides how many passes run and who runs them,
//!  never what a pass reads or writes,
//!  which is why one closure is the whole of its dependency on the sweep.
//!  The steps the design gives a taker that this file does not run are the
//!  ones whose records are plan step 22's:
//!  the budget read and the charge of a pass's own duration,
//!  because the design puts `pass-budget` and its number in that step
//!  (V12's first option,
//!  "17.
//! The merge device is built in step 19,
//! the budget's number set in step
//!  22").
//!  The budget decision belongs between the claim and the mark's removal,
//!  and a wiring round that lands step 22 inserts it there.
//!  Everything the device could not read is a line its answer carries and never
//!  a refusal (V7's first option),
//! because the per-slot claim is what keeps two
//!  passes safe and the device only decides how many run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = runAMergePass(folder, () => sweep()); // { line, refused }
//! ```

use crate::worktree_clone_mount_sweep_merge::merge_lines::{
    claim_not_released_line,
    device_unreadable_line,
    extra_pass_line,
    held_line,
    mark_left_standing_line,
    mark_left_with_no_pass_line,
    mark_not_removed_line,
    mark_unreadable_line,
};
use crate::worktree_clone_mount_sweep_merge::pass_mark::{
    MarkReading,
    remove_the_mark,
    the_mark_is_there,
    write_the_mark,
};
use crate::worktree_clone_mount_sweep_merge::run_claim::{
    ClaimAnswer,
    HeldRunClaim,
    take_the_run_claim,
};
use crate::worktree_clone_mount_sweep_merge::sweeps_folder::SweepsFolder;

/// The most passes one invocation runs:
/// the pass it was started for and one more for a mark.
/// Value:
///  the design's own count of one extra pass per admitted run,
///  adopted with the project's veto open (V12's first option),
///  and not a measurement:
///  measurement M4 reports whether the device earns that cost.
pub const MOST_PASSES: usize = 2;

/// What:
///  Run one event-driven pass under the merge device.
/// Why:
///   This is the taker the design states for a handler that may pass,
///  with the caller's own entry order already done:
///  the prerequisite checks and the filter run before this call,
///  because a prerequisite that refused must release no claim it never took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = runAMergePass(folder, () => sweep());
/// ```
pub fn run_a_merge_pass<F>(
    folder: &SweepsFolder,
    pass: F,
) -> Result<String, String>
where
    F: FnMut() -> Result<String, String>,
{
    return run_a_merge_pass_seamed(folder, pass, &mut no_call, &mut no_call);
}

/// The call a taker makes where no seam was asked for:
/// none.
fn no_call() {}

/// What:
///  Run one event-driven pass under the merge device,
///  with two calls a control may answer.
/// Why:
///   The two windows this device has are the ones a control must not have to
///  win by timing:
///  one between the mark a held trigger writes and the claim's second attempt,
///  where a control lets the hold go,
///  and one between the first pass's release and the look for the mark,
///  where a control writes the mark itself.
///  The design asks for the second one by name (its test 4),
///  and the first is the same race read from the other side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = runAMergePassSeamed(folder, () => sweep(), () => releaseTheHold(), () => writeTheMark());
/// ```
pub fn run_a_merge_pass_seamed<F>(
    folder: &SweepsFolder,
    mut pass: F,
    after_the_first_attempt: &mut dyn FnMut(),
    after_the_first_pass_release: &mut dyn FnMut(),
) -> Result<String, String>
where
    F: FnMut() -> Result<String, String>,
{
    let mut lines: Vec<String> = Vec::new();
    let claim: HeldRunClaim = match take_the_run_claim(folder) {
        ClaimAnswer::Unreadable {
            reason,
        } => {
            // V7's first option: a cost record that cannot be read is reported and the pass runs.
            lines.push(device_unreadable_line(reason.as_str()));
            return answered(lines, pass());
        }
        ClaimAnswer::Held(claim) => claim,
        ClaimAnswer::HeldByAnother(_owner) => {
            match write_the_mark(folder) {
                Ok(()) => {}
                Err(reason) => lines.push(reason),
            }
            after_the_first_attempt();
            match take_the_run_claim(folder) {
                ClaimAnswer::Held(claim) => claim,
                ClaimAnswer::HeldByAnother(again) => {
                    lines.push(mark_left_with_no_pass_line(&again));
                    return Err(every_line(lines, String::new()));
                }
                ClaimAnswer::Unreadable {
                    reason,
                } => {
                    lines.push(device_unreadable_line(reason.as_str()));
                    return answered(lines, pass());
                }
            }
        }
    };
    let first: Result<String, String> = run_the_pass(folder, claim, &mut pass, &mut lines);
    after_the_first_pass_release();
    let reading: MarkReading = the_mark_is_there(folder);
    if let MarkReading::Unreadable {
        reason,
    } = &reading
    {
        lines.push(mark_unreadable_line(reason.as_str()));
    }
    if !reading.asks_for_a_pass() {
        return answered(lines, first);
    }
    return one_more_pass(folder, &mut pass, lines, first);
}

/// What:
///  The one pass a mark buys,
///  and nothing after it.
/// Why:
///   The cap is the shape of this function:
///  a mark that survives the second pass is left standing for the next
///  trigger and never read again here,
///  so one invocation runs at most `MOST_PASSES` passes whatever the marks it
///  meets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = oneMorePass(folder, pass, lines, first);
/// ```
fn one_more_pass<F>(
    folder: &SweepsFolder,
    pass: &mut F,
    mut lines: Vec<String>,
    first: Result<String, String>,
) -> Result<String, String>
where
    F: FnMut() -> Result<String, String>,
{
    return match take_the_run_claim(folder) {
        ClaimAnswer::Held(claim) => {
            lines.push(extra_pass_line());
            let second: Result<String, String> = run_the_pass(folder, claim, pass, &mut lines);
            if the_mark_is_there(folder).asks_for_a_pass() {
                lines.push(mark_left_standing_line());
            }
            answered(lines, second)
        }
        ClaimAnswer::HeldByAnother(owner) => {
            // The pass that holds the claim answers this trigger; this invocation's pass already ran.
            lines.push(held_line(&owner));
            lines.push(mark_left_standing_line());
            answered(lines, first)
        }
        ClaimAnswer::Unreadable {
            reason,
        } => {
            lines.push(device_unreadable_line(reason.as_str()));
            lines.push(extra_pass_line());
            answered(lines, pass())
        }
    };
}

/// What:
///  One pass under the claim this invocation holds:
///  the mark goes because this pass enumerates,
///  the pass runs,
///  and the claim is given back.
/// Why:
///   The order is the taker's steps 4 to 6,
///  and the release is reported rather than logged,
///  because a claim that could not be given back is a line the answer carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = runThePass(folder, claim, pass, lines);
/// ```
fn run_the_pass<F>(
    folder: &SweepsFolder,
    claim: HeldRunClaim,
    pass: &mut F,
    lines: &mut Vec<String>,
) -> Result<String, String>
where
    F: FnMut() -> Result<String, String>,
{
    match remove_the_mark(folder) {
        Ok(()) => {}
        Err(reason) => lines.push(mark_not_removed_line(reason.as_str())),
    }
    let outcome: Result<String, String> = pass();
    if let Err(reason) = claim.release() {
        lines.push(claim_not_released_line(reason.as_str()));
    }
    return outcome;
}

/// What:
///  One invocation's answer:
///  the device's own lines,
///  then the pass's own answer,
///  with the pass's status deciding the status.
/// Why:
///   The device adds lines and never a status of its own,
///  except when it ran no pass at all:
///  a refusal is what a pass's own answer earns,
///  and the device's reports are lines beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = answered(lines, passOutcome);
/// ```
fn answered(
    lines: Vec<String>,
    outcome: Result<String, String>,
) -> Result<String, String> {
    return match outcome {
        Ok(report) => Ok(every_line(lines, report)),
        Err(refusal) => Err(every_line(lines, refusal)),
    };
}

/// What:
///  Every line,
/// each on its own,
///  with an empty last one left out.
/// Why:
///   A report ends the answer and a refusal ends it too,
///  so a pass that answered nothing must not add a blank line a journal would
///  keep as part of the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = everyLine(['a'], ''); // 'a'
/// ```
fn every_line(
    lines: Vec<String>,
    last: String,
) -> String {
    let mut all: Vec<String> = lines;
    if !last.is_empty() {
        all.push(last);
    }
    return all.join("\n");
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "merge_pass_tests.rs"]
mod tests;

/// The contended paths of the taker stay out of the release executable.
#[cfg(test)]
#[path = "merge_pass_contention_tests.rs"]
mod contention_tests;
