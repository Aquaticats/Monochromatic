//! What:
//!  The words for a wait that ended without a greeting,
//!  built from two facts the wait kept:
//!  what the service manager last said about the unit,
//!  and what this command asked the manager to do about it.
//! Why:
//!  "The commit service did not start" alone said nothing about the wait that ended,
//!  and the first attempt at naming it kept one fact too few:
//!  a state word of the manager,
//!  which is `None` for a manager that answered it holds no such unit,
//!  and a flag of whether this command had asked for a start,
//!  which is false for a unit the manager is already starting.
//!  Both sentences that came out of those two facts were false in their own cell:
//!  a unit the manager reported `activating` was worded "nothing asked it to start",
//!  though a connection to a socket-activated unit is what asks it to start,
//!  and a manager that kept answering that it holds no such unit was worded as one that gave no answer.
//!  So the words are composed from the two facts themselves,
//!  one clause each,
//!  and every clause follows from what was remembered
//!  (`service_client_reach_service.rs`,
//!  the table in its module doc).
//!  They live beside the rendering of every failure rather than inside it,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessagesOfAStartThatWasAskedAndDidNotCome.ts
//! export const didNotStart = (unit, answer, asked): string => `${whatTheManagerSaid(unit, answer)}, and ${whatThisCommandAsked(unit, asked)}.`;
//! ```

/// The two facts these words read are declared beside the failures that hold them
/// (`service_messages_the_failures_a_person_can_meet.rs`),
/// because a failure's own record holds them and would otherwise use this file.
pub use crate::service_messages_the_failures_a_person_can_meet::{
    ManagerLastWords,
    RequestedStart,
};

/// What:
///  The clause for a wait that ended without a greeting.
/// Why:
///  Two clauses,
///  one per remembered fact,
///  so the sentence can be checked against the table cell it came from,
///  and no cell says something the wait did not learn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// didNotStart(unit, answer, asked) // 'the service manager says cli-git@3f9a.service is activating, so a start of it is under way, and this command asked it for nothing itself'
/// ```
pub(crate) fn did_not_start(
    unit: Option<&str>,
    answer: &ManagerLastWords,
    asked: RequestedStart,
) -> String {
    let Some(named) = unit else {
        return String::from(
            "no service manager could be asked about this worktree's commit service, so nothing started it",
        );
    };
    return format!(
        "{}, and {}",
        what_the_manager_said(named, answer),
        what_this_command_asked(named, asked)
    );
}

/// What:
///  What the manager last said,
///  in a clause.
/// Why:
///  A unit the manager reports as `activating` is one whose start is under way,
///  and saying so is the fact that keeps a stuck start from reading as a start nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whatTheManagerSaid(unit, answer) // 'the service manager says cli-git@3f9a.service is activating, so a start of it is under way'
/// ```
fn what_the_manager_said(
    unit: &str,
    answer: &ManagerLastWords,
) -> String {
    return match answer {
        ManagerLastWords::State(state) if state == "activating" => {
            format!("the service manager says {unit} is activating, so a start of it is under way")
        }
        ManagerLastWords::State(state) => format!("the service manager says {unit} is {state}"),
        ManagerLastWords::NotHeld => {
            format!("the service manager answered that it holds no such unit, {unit}")
        }
        ManagerLastWords::Silent => format!("the service manager gave no answer about {unit}"),
    };
}

/// What:
///  What this command asked the manager for,
///  in a clause.
/// Why:
///  "itself" carries the one thing the client knows and the reader needs:
///  what this command asked,
///  never what any other connection or person asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whatThisCommandAsked(unit, asked) // 'this command asked it to start cli-git@3f9a.service'
/// ```
fn what_this_command_asked(
    unit: &str,
    asked: RequestedStart,
) -> String {
    return match asked {
        RequestedStart::Nothing => String::from("this command asked it for nothing itself"),
        RequestedStart::Start => format!("this command asked it to start {unit}"),
        RequestedStart::ResetAndStart => {
            format!("this command asked it to forget that {unit} failed and then to start it")
        }
        RequestedStart::Restart => format!("this command asked it to restart {unit}"),
    };
}

/// What:
///  The failure a hand-off reports for a wait that ended without a greeting,
///  or nothing when the failure is another one.
/// Why:
///  The reason is the two facts the wait remembered,
///  so it is worded here where they are,
///  and the hand-off keeps the one call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// workNotHandedOver(failure) // { kind: 'work-not-handed-over', reason } | undefined
/// ```
pub(crate) fn work_not_handed_over(
    failure: &crate::service_messages_the_failures_a_person_can_meet::ClientFailure
) -> Option<crate::service_messages_the_failures_a_person_can_meet::ClientFailure> {
    let crate::service_messages_the_failures_a_person_can_meet::ClientFailure::DidNotStart {
        unit,
        answer,
        asked,
    } = failure
    else {
        return None;
    };
    return Some(crate::service_messages_the_failures_a_person_can_meet::ClientFailure::WorkNotHandedOver(
        format!(
            "the commit service did not start: {}",
            did_not_start(unit.as_deref(), answer, *asked)
        ),
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_messages_of_a_start_that_was_asked_and_did_not_come_tests.rs"]
mod tests;
