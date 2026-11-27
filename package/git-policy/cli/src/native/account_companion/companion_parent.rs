//! What:
//!  The lines the add's preflight calls the companion through:
//!  the account's runtime folder,
//!  the run before the add's first write,
//!  and the one run inside a folder the caller resolved.
//! Why:
//!  The add resolves the folder once and hands it to the check as an input,
//!  so the check reads no environment and every control passes a folder of its
//!  own;
//!  the run itself,
//!  its bounded wait and its verdict are
//!  `crate::account_companion::companion_parent_run`'s and
//!  `crate::account_companion::companion_parent_verdict`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! beforeTheAdd(runtime) { return runUnder(runtime, switchFromEnvironment(), theDeadline()); }
//! ```

/// The bound the parent enforces.
use super::companion_deadline::the_deadline;
/// The verdict a run answers with,
/// and the refusals a person reads.
use super::companion_lines::{
    CompanionVerdict,
    Refusal,
};
/// The run's own steps,
/// from the scratch directory to the child's ending.
use super::companion_parent_run::run_the_check;
/// The verdict one ended run answers with,
/// which the controls here judge on its own.
#[cfg(test)]
use super::companion_parent_verdict::the_verdict;
/// The runtime folder the caller resolves and the run's directory under it.
use super::companion_scratch::{
    RunDirectory,
    RuntimeFolder,
    make_run_directory,
};
/// The three settings a skip line prints.
use super::companion_settings::{
    self,
    Settings,
};
/// The test-only switch and the fixtures it names.
use super::companion_test_switch::{
    Switch,
    from_environment,
};
/// How long one run may take before the add stops waiting for it.
use std::time::Duration;

/// The fork's retry budget and its pause,
/// which `crate::account_companion::companion_parent_run` spends.
pub use super::companion_parent_run::{
    FORK_RETRY_BUDGET,
    FORK_RETRY_PAUSE,
};

/// What:
///  The check as the add calls it,
///  with the runtime folder the add resolved:
///  prints every line the run carries and answers whether the managed attach
///  may go ahead.
/// Why:
///  Every line goes out on this one path,
///  so no call site decides what a person reads,
///  and a refusal leaves the route at once,
///  which is what puts the slot on the reflink or ordinary Git route.
pub fn the_managed_attach_may_go_ahead(runtime: &RuntimeFolder) -> bool {
    let verdict: CompanionVerdict = before_the_add(runtime);
    verdict.report();
    return verdict.the_managed_attach_goes_ahead();
}

/// What:
///  The account's runtime folder as this process's environment names it,
///  or the line a person reads when it names none.
/// Why:
///  The add route resolves the folder once and hands it to the check as an input,
///  so the check reads no environment and every control passes a folder of its
///  own;
///  the words of a missing folder stay beside the other refusals,
///  where a person's line is written.
pub fn the_accounts_runtime_folder() -> Option<RuntimeFolder> {
    return match RuntimeFolder::from_environment() {
        Ok(folder) => Some(folder),
        Err(refusal) => {
            let refused: CompanionVerdict = CompanionVerdict::Refused {
                refusal: Refusal::Scratch {
                    message: refusal.message,
                },
            };
            refused.report();
            None
        }
    };
}

/// What:
///  Run the companion and answer what the add does with it.
/// Why:
///  The add calls this before its first write,
///  so a refusal leaves no managed object,
///  and a skip lets the managed attach go ahead.
pub fn before_the_add(runtime: &RuntimeFolder) -> CompanionVerdict {
    let switch: Switch = from_environment();
    return run_under(runtime, switch, the_deadline(switch.is_set()));
}

/// What:
///  One run inside the runtime folder the caller resolved,
///  with the deadline named rather than read,
///  which is what the controls use so no test changes another's environment.
pub fn run_under(
    runtime: &RuntimeFolder,
    switch: Switch,
    deadline: Duration,
) -> CompanionVerdict {
    if switch == Switch::Disabled {
        return CompanionVerdict::NotRun;
    }
    let settings: Settings = companion_settings::read();
    let run: RunDirectory = match make_run_directory(runtime) {
        Ok(run) => run,
        Err(refusal) => {
            return CompanionVerdict::Refused {
                refusal: Refusal::Scratch {
                    message: refusal.message,
                },
            };
        }
    };
    let verdict: CompanionVerdict = run_the_check(&run, switch, &settings, deadline);
    return verdict;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_parent_tests.rs"]
mod tests;

/// The measurement this round's deadline cites,
/// which is no control and runs only when it is asked for.
#[cfg(test)]
#[path = "companion_cost_measurement_tests.rs"]
mod measurement;
