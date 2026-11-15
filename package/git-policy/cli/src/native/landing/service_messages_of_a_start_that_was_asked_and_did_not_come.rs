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
//!  (`service_client_reach_service.rs`, the table in its module doc).
//!  They live beside the rendering of every failure rather than inside it,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessagesOfAStartThatWasAskedAndDidNotCome.ts
//! export const didNotStart = (unit, answer, asked): string => `${whatTheManagerSaid(unit, answer)}, and ${whatThisCommandAsked(unit, asked)}.`;
//! ```

/// What:
///  What the service manager last said about the unit this command asked after.
/// Why:
///  A state word and the two answers that are not states are three different next steps,
///  and folding them into one made a manager that answered every poll read as one that answered none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagerLastWords = { kind: 'state', text: string } | { kind: 'not-held' } | { kind: 'silent' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagerLastWords {
    /// The manager named the unit's state, in its own word (`active`,
    /// `activating`,
    /// `inactive`,
    /// `failed`,
    /// ...).
    State(String),
    /// The manager answered that it holds no such unit,
    /// which is what a unit that stopped and was unloaded leaves.
    NotHeld,
    /// The manager gave no answer: the exchange with it failed,
    /// or it was never asked.
    Silent,
}

/// What:
///  What this command asked the service manager to do about the unit during its wait.
/// Why:
///  A person reads whether anything of this command asked for a start,
///  which is not the same as whether a start is under way:
///  a connection to the socket of a socket-activated unit asks for one,
///  and the client cannot tell which connection asked,
///  so the words say what this command asked and leave who started the unit to the manager's own state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestedStart = 'nothing' | 'start' | 'reset-then-start' | 'restart';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestedStart {
    /// No call of this command asked the manager for anything.
    Nothing,
    /// This command asked the manager to start the unit,
    /// which a unit it holds no such unit for needs to be loaded again.
    Start,
    /// This command asked the manager to forget that the unit failed and then to start it.
    ResetAndStart,
    /// This command asked the manager to stop the unit and start it again.
    Restart,
}

/// What:
///  The clause for a wait that ended without a greeting.
/// Why:
///  Two clauses, one per remembered fact,
///  so the sentence can be checked against the table cell it came from,
///  and no cell says something the wait did not learn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// didNotStart(unit, answer, asked) // 'the service manager says cli-git@3f9a.service is activating, so a start of it is under way, and this command asked it for nothing itself'
/// ```
pub(crate) fn did_not_start(unit: Option<&str>, answer: &ManagerLastWords, asked: RequestedStart) -> String {
    let Some(named) = unit else {
        return String::from("no service manager could be asked about this worktree's commit service, so nothing started it");
    };
    return format!("{}, and {}", what_the_manager_said(named, answer), what_this_command_asked(named, asked));
}

/// What:
///  What the manager last said, in a clause.
/// Why:
///  A unit the manager reports as `activating` is one whose start is under way,
///  and saying so is the fact that keeps a stuck start from reading as a start nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whatTheManagerSaid(unit, answer) // 'the service manager says cli-git@3f9a.service is activating, so a start of it is under way'
/// ```
fn what_the_manager_said(unit: &str, answer: &ManagerLastWords) -> String {
    return match answer {
        ManagerLastWords::State(state) if state == "activating" => {
            format!("the service manager says {unit} is activating, so a start of it is under way")
        }
        ManagerLastWords::State(state) => format!("the service manager says {unit} is {state}"),
        ManagerLastWords::NotHeld => format!("the service manager answered that it holds no such unit, {unit}"),
        ManagerLastWords::Silent => format!("the service manager gave no answer about {unit}"),
    };
}

/// What:
///  What this command asked the manager for, in a clause.
/// Why:
///  "itself" carries the one thing the client knows and the reader needs:
///  what this command asked,
///  never what any other connection or person asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whatThisCommandAsked(unit, asked) // 'this command asked it to start cli-git@3f9a.service'
/// ```
fn what_this_command_asked(unit: &str, asked: RequestedStart) -> String {
    return match asked {
        RequestedStart::Nothing => String::from("this command asked it for nothing itself"),
        RequestedStart::Start => format!("this command asked it to start {unit}"),
        RequestedStart::ResetAndStart => format!("this command asked it to forget that {unit} failed and then to start it"),
        RequestedStart::Restart => format!("this command asked it to restart {unit}"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_messages_of_a_start_that_was_asked_and_did_not_come_tests.rs"]
mod tests;
