//! What:
//!  The clause a command reads when its wait for the commit service ended and the wait's own facts are the
//!  reason:
//!  which unit the user service manager was asked about,
//!  what it last answered,
//!  and whether it was asked to start that unit.
//! Why:
//!  "The commit service did not start" alone said nothing about the wait that ended.
//!  A start that was never asked for leaves the service's own log empty,
//!  so the remedy that message names shows nothing,
//!  and that is what left the merges of 2026-11-14T20:51:03-05:00 and 2026-11-14T20:58:58-05:00 unexplained:
//!  their service had retired and stopped its own socket,
//!  the manager had unloaded the unit,
//!  and the client read the manager's "it holds no such unit" as "the manager cannot be asked"
//!  (`service_client_reach_service.rs`).
//!  The facts belong in the message because they separate a manager that could not be reached from a start
//!  that was asked for and did not come up,
//!  which have two different next steps.
//!  They live beside the rendering of every failure rather than inside it,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessagesOfAStartThatWasAskedAndDidNotCome.ts
//! export const theStartThatDidNotCome = (unit, answer, startAsked): string => ...;
//! ```

/// What:
///  The clause for a wait that ended without a greeting,
///  built from the unit this command asked about,
///  the service manager's last answer about it,
///  and whether a start was asked for.
/// Why:
///  Each state has one next step,
///  so each has its own words,
///  and none of them claims a start was asked for when none was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theStartThatDidNotCome(unit, answer, startAsked) // 'the service manager could not be asked to start cli-git@3f9a.service …'
/// ```
pub(crate) fn the_start_that_did_not_come(unit: Option<&str>, answer: Option<&str>, start_asked: bool) -> String {
    return match (unit, answer, start_asked) {
        (None, _, _) => String::from("no service manager could be asked about this worktree's commit service, so nothing started it"),
        (Some(named), Some(state), true) => format!("the service manager was asked to start {named} and last said that unit is {state}"),
        (Some(named), Some(state), false) => format!("the service manager says {named} is {state}, and nothing asked it to start that unit"),
        (Some(named), None, true) => format!("the service manager was asked to start {named} and gave no answer about it afterwards"),
        (Some(named), None, false) => format!("the service manager could not be asked to start {named}, so nothing started the commit service"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_messages_of_a_start_that_was_asked_and_did_not_come_tests.rs"]
mod tests;
