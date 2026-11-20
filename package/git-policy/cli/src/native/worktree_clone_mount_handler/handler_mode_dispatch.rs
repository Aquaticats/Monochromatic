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
use crate::zfs_compatibility_gate::gate_verdict::Verdict;
use crate::zfs_compatibility_gate::{BeforeRequests, LiveGate, verdict_line};
use std::ffi::OsString;
use std::io::Write;

/// The status of an invocation that did what it was asked.
pub const EXIT_DONE: i32 = 0;

/// The status of an invocation that refused:
/// an unreadable argument,
/// or a mode that would not act.
pub const EXIT_REFUSED: i32 = 2;

/// What:
///  The bytes of one whole line,
///  its newline included.
/// Why:
///   The door writes exactly the bytes it is handed,
///  so a caller that passed its sentence alone ran the next line into it:
///  two `--version` runs printed `mount-git-worktree-clone-datasets 0.6.1-alpha.44mount-git-worktree-clone-datasets 0.6.1-alpha.44`,
///  and a refusal followed the line before it with no break.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lineToWrite = (line) => `${line}\n`;
/// ```
fn line_to_write(line: &str) -> String {
    return format!("{line}\n");
}

/// What:
///  Say one report line on the process's standard output,
///  which is where a command's answer belongs.
/// Why:
///   `--help` and `--version` are the program's answer,
///  and a person who pipes it (`mount-git-worktree-clone-datasets --version | grep`) reads standard output:
///  a report on standard error would print on the terminal and leave the pipe empty.
///  The write is small and line-terminated,
///  exactly what the wrapper's own exit report writes to the same stream (`invocation::write_exit`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reportLine = (line) => process.stdout.write(lineToWrite(line));
/// ```
fn report_line(line: &str) {
    let _written = std::io::stdout().write_all(line_to_write(line).as_bytes());
}

/// What:
///  Say one whole line to the process's standard error,
///  through the door every diagnostic line goes through.
/// Why:
///   `eprintln!` takes the stream handle every writer of this process shares,
///  and a reader that stopped draining it would hold this program where it stands;
///  the door hands the line to the thread that never holds its caller,
///  and the line reaches it with the newline that ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const say = (line) => write(lineToWrite(line)); // the diagnostic door
/// ```
fn say(line: &str) {
    write(line_to_write(line).as_str());
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
///  Run the compatibility gate before a mode that will make requests,
///  and answer
///  with the gate's line when it refuses.
/// Why:
///  No mode may send a raw request before the loaded module's declaration is
///  compared,
///  and a refusal is the whole answer:
///  the mode is not entered,
///  so nothing is opened,
///  mounted,
///  unmounted or recorded,
///  and registrations and durable recovery state stay as they were.
fn behind_the_gate(
    gate: &dyn BeforeRequests,
    outcome: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    let verdict: Verdict = gate.verdict();
    let line: String = verdict_line(&verdict);
    if let Verdict::Refused(_) = verdict {
        return Err(line);
    }
    return match outcome() {
        Ok(done) => Ok(format!("{line}\n{done}")),
        Err(refusal) => Err(format!("{line}\n{refusal}")),
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
    return run_behind(&LiveGate, arguments);
}

/// What:
///  Read one invocation and run it behind the given gate.
/// Why:
///   The release path's gate reads this machine's kernel files and device,
///  so a
///  control hands in a gate that answers without them,
///  and the order the modes run
///  in stays the one the release path takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = runBehind(gate, argv);
/// ```
pub fn run_behind(gate: &dyn BeforeRequests, arguments: &[OsString]) -> i32 {
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
            report_line(handler_mode_arguments::usage().as_str());
            EXIT_DONE
        }
        HandlerMode::Version => {
            report_line(handler_mode_arguments::version_line().as_str());
            EXIT_DONE
        }
        HandlerMode::AttachingSweep => report(behind_the_gate(gate, || {
            return attaching_sweep_mode::run().map_err(|awaiting| return awaiting.message());
        })),
        HandlerMode::BootReconcile => report(behind_the_gate(gate, || {
            return boot_reconcile_mode::run().map_err(|awaiting| return awaiting.message());
        })),
        HandlerMode::ShutdownUnmount => report(behind_the_gate(gate, || {
            return shutdown_unmount_mode::run().map_err(|awaiting| return awaiting.message());
        })),
        HandlerMode::AdoptWorktreeParent(request) => report(adopt_worktree_parent_mode::run(&request).map_err(|failure| return failure.message())),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_mode_dispatch_tests.rs"]
mod tests;
