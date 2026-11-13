//! What:
//!  The words that tell a person how to change the push deadline,
//! for the program that is running the push.
//! Why:
//!  A push in a terminal reads `CLI_GIT_PUSH_DEADLINE_SECONDS` from that terminal's environment,
//! but the resident commit
//!  service pushes after the commit command returned and reads it from the environment the user manager gave the service
//!  when it started,
//! so advice to set the variable is not something a person can act on there.
//! In the service the advice
//!  tells the person to end the service and run the command again, because a service the wrapper starts reads its
//! environment when it starts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const advice = inService ? 'set VARIABLE for the next command, which starts a fresh service' : 'set VARIABLE'
//! ```

use crate::push_deadline_and_patience_settings::PUSH_DEADLINE_VARIABLE;
use crate::service_messages::RESTART_COMMAND;
use std::sync::atomic::{AtomicBool, Ordering};

/// The stall window the service advice gives as its example,
/// in seconds.
pub const EXAMPLE_SECONDS: u32 = 120;

/// Whether this process is the resident commit service,
/// which says so once at its start.
static IN_SERVICE: AtomicBool = AtomicBool::new(false);

/// Says that this process is the resident commit service,
/// so that the advice it prints is the service's.
pub fn mark_service_process() {
    IN_SERVICE.store(true, Ordering::SeqCst);
}

/// Whether this process is the resident commit service.
fn runs_in_service() -> bool {
    return IN_SERVICE.load(Ordering::SeqCst);
}

/// What:
///  What a person does to give the commit service the variable at its next start.
/// Why:
///  The wrapper starts the service itself, with the environment of the command that started it,
///  so the way to change what the service reads is to end it and let the next command start a fresh one:
/// there is no manager to set an environment on, and no unit for it to restart.
fn set_command() -> String {
    return format!("export {PUSH_DEADLINE_VARIABLE}={EXAMPLE_SECONDS}");
}

/// What:
///  The clause that says how to allow a push more time,
/// for the given program.
/// Why:
///   It reads after a semicolon or a comma in the messages of a stopped push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// extendClause(inService)
/// ```
pub fn extend_clause_for(service: bool) -> String {
    if service {
        return format!(
            "run `{}` in the shell this command runs in, then `{RESTART_COMMAND}` so the next command starts a service with it (a service the wrapper already started keeps the environment it was started with)",
            set_command()
        );
    }
    return format!("set {PUSH_DEADLINE_VARIABLE} to allow longer");
}

/// What:
///  The clause that says what sets the push limit,
/// for the given program.
/// Why:
///   The messages of a push that was not started or was stopped name the setting,
/// and the setting is reached differently in
///  the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// limitClause(inService)
/// ```
pub fn limit_clause_for(service: bool) -> String {
    if service {
        return extend_clause_for(true);
    }
    return format!("{PUSH_DEADLINE_VARIABLE} sets the limit");
}

/// The clause for allowing a push longer,
/// in the program that is running.
pub fn extend_clause() -> String {
    return extend_clause_for(runs_in_service());
}

/// The clause for what sets the push limit,
/// in the program that is running.
pub fn limit_clause() -> String {
    return limit_clause_for(runs_in_service());
}

/// Advice controls stay out of the release executable.
#[cfg(test)]
#[path = "push_deadline_advice_tests.rs"]
mod tests;
