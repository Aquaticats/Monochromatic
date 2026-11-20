//! What:
//!  The one command line of the root-run clone-mount handler:
//!  read the arguments as a mode,
//!  run that mode,
//!  write what it said,
//!  and answer with the process's exit status.
//! Why:
//!   Every mode ends in the same two answers to an operator or a unit:
//!  a line saying what happened,
//!  or a refusal saying why nothing happened,
//!  and the same exit status for each.
//!  A refusal is never a success:
//!  a boot unit's `ExecStart` that could not reconcile must fail,
//!  so a status of zero is written only after a mode did its work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.exit(run(process.argv.slice(2))); // 0: done, 2: refused
//! ```

use super::adopt_worktree_parent_mode;
use super::attaching_sweep_mode;
use super::boot_reconcile_mode;
use super::handler_mode_arguments::{self, HandlerMode};
use super::shutdown_unmount_mode;
use crate::diagnostic_lines_off_the_calling_thread::write;
use std::ffi::OsString;

/// The status of an invocation that did what it was asked.
pub const EXIT_DONE: i32 = 0;

/// The status of an invocation that refused:
/// an unreadable argument,
/// or a mode that would not act.
pub const EXIT_REFUSED: i32 = 2;

/// What:
///  Say one line to the process's standard error,
///  through the door every diagnostic line goes through.
/// Why:
///   `eprintln!` takes the stream handle every writer of this process shares,
///  and a reader that stopped draining it would hold this program where it stands;
///  the door hands the line to the thread that never holds its caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const say = (line) => write(line); // the diagnostic door
/// ```
fn say(line: &str) {
    write(line);
}

/// What:
///  The status one mode's answer becomes,
///  writing its line first.
/// Why:
///   The success line and the refusal line are both a person's answer,
///  and only the success reaches zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = (outcome) => { if (outcome.ok) { say(outcome.line); return 0 } say(outcome.refusal); return 2 };
/// ```
fn report(outcome: Result<String, String>) -> i32 {
    return match outcome {
        Ok(line) => {
            say(line.as_str());
            EXIT_DONE
        }
        Err(refusal) => {
            say(refusal.as_str());
            EXIT_REFUSED
        }
    };
}

/// What:
///  Read one invocation and run it.
/// Why:
///   The argument grammar is the only way in:
///  an unreadable invocation prints its cause and the usage and refuses,
///  and each mode that has landed runs,
///  while each mode whose step has not prints that step and refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = run(argv); // 0 when the mode did its work, 2 otherwise
/// ```
pub fn run(arguments: &[OsString]) -> i32 {
    let mode: HandlerMode = match handler_mode_arguments::parse(arguments) {
        Ok(mode) => mode,
        Err(refusal) => {
            say(refusal.message().as_str());
            say(handler_mode_arguments::usage().as_str());
            return EXIT_REFUSED;
        }
    };
    return match mode {
        HandlerMode::Help => {
            say(handler_mode_arguments::usage().as_str());
            EXIT_DONE
        }
        HandlerMode::Version => {
            say(handler_mode_arguments::version_line().as_str());
            EXIT_DONE
        }
        HandlerMode::AttachingSweep => report(attaching_sweep_mode::run().map_err(|awaiting| return awaiting.message())),
        HandlerMode::BootReconcile => report(boot_reconcile_mode::run().map_err(|awaiting| return awaiting.message())),
        HandlerMode::ShutdownUnmount => report(shutdown_unmount_mode::run().map_err(|awaiting| return awaiting.message())),
        HandlerMode::AdoptWorktreeParent(request) => report(adopt_worktree_parent_mode::run(&request).map_err(|failure| return failure.message())),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_mode_dispatch_tests.rs"]
mod tests;
