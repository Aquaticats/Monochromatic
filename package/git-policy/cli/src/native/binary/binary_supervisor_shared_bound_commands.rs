//! What:
//!  Which of the commands the tests start are the shared bound,
//!  and how one is recognised from its command line or from a `Command` about to be started.
//! Why:
//!   Most wrapped commands run under a per-test bound the control chose,
//!   and one shape runs under a bound every test shares:
//!   `timeout --signal=KILL 600 <program> ...`,
//!   whose backstop stands in for the wall-clock bound the guard replaced.
//!  The monitor watches only that shape by itself,
//!  because the per-test bounds already end their own commands,
//!  and a control that starts a command of its own shape registers it with `watch` instead.
//!  The question is asked twice:
//!  of a command line read from `/proc` (a child between `fork` and `exec` still shows its
//!  parent's line),
//!  and of a `Command` before it is started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const isShared = (command: Command) => isSharedCommandLine([command.program, ...command.args]);
//! ```
use super::supervisor_limits::SHARED_BACKSTOP_SECONDS;
use std::process::Command;

/// Whether the command line of a process is the shared bound:
/// `timeout --signal=KILL <backstop> <program> ...`.
pub fn is_shared_command_line(arguments: &[&str]) -> bool {
    return arguments.len() > 3
        && arguments[0].rsplit('/').next() == Some("timeout")
        && arguments[1] == "--signal=KILL"
        && arguments[2] == SHARED_BACKSTOP_SECONDS;
}

/// Whether a command about to be started is the shared bound.
pub fn is_shared(command: &Command) -> bool {
    let program: String = command.get_program().to_string_lossy().into_owned();
    let arguments: Vec<String> = command
        .get_args()
        .map(|argument| {
            return argument.to_string_lossy().into_owned();
        })
        .collect();
    let mut line: Vec<&str> = vec![program.as_str()];
    line.extend(arguments.iter().map(String::as_str));
    return is_shared_command_line(line.as_slice());
}
