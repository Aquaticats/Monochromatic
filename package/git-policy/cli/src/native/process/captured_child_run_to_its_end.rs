//! What:
//!  Runs a command to its end and returns its status and output,
//! the way `Command::output` does,
//! with the child started by the
//!  standard library,
//! awaited through its pidfd on the runtime,
//! and its output read from anonymous files.
//! Why:
//!  The one-shot child waits of the wrapper (a question put to Git,
//! a scan of lock holders) moved off `Command::output`,
//! which
//!  blocks a thread until the child ends,
//! onto the runtime (`doc/planning/git-policy-cli-full-tokio.md`,
//! step T4).
//! The files
//!  mean no reader is needed and a child that writes a lot never waits for one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { status, stdout, stderr } = await execFile(program, args); // not throwing on a non-zero exit
//! ```

use crate::child_awaited_through_a_pidfd::adopt;
use crate::lazy_runtime::run_on_runtime;
use crate::output_capture::{Capture, STEP_OUTPUT_LIMIT};
use crate::start_retry::RetryStart as _;
use std::process::{Command, Output};

/// What:
///  Runs `command` to its end.
/// The start is the standard library's (a refused start leaves no child,
/// and `EAGAIN` is retried
///  within its budget);
/// everything after it is owned by the awaiting value.
/// Why:
///  The output is the last `STEP_OUTPUT_LIMIT` bytes of each stream,
/// as every other captured step of the wrapper is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await outputOf(command);
/// ```
pub async fn output_of(command: &mut Command) -> std::io::Result<Output> {
    let capture: Capture = Capture::new()?;
    let (standard_output, standard_error) = capture.stdio()?;
    command.stdout(standard_output).stderr(standard_error);
    let child: std::process::Child = command.spawn_retrying()?;
    let mut owned = adopt(child);
    let status: std::process::ExitStatus = owned.wait().await?;
    let captured = capture.snapshot(STEP_OUTPUT_LIMIT);
    return Ok(Output { status, stdout: captured.stdout, stderr: captured.stderr });
}

/// What:
///  `output_of` for synchronous code:
/// enters the process's runtime,
/// building it on the first need.
/// From inside the runtime it
///  returns an error instead of panicking,
/// because waiting for the runtime from within it would never end.
/// Why:
///  The callers are synchronous until their own callers become asynchronous (step T11);
/// this bridge goes with the last of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = outputOfBlocking(command); // only until every caller is async
/// ```
pub fn output_of_blocking(command: &mut Command) -> std::io::Result<Output> {
    return match run_on_runtime(output_of(command)) {
        Ok(finished) => finished,
        Err(refusal) => Err(std::io::Error::other(refusal.to_string())),
    };
}

/// Captured-run controls stay out of the release executable.
#[cfg(test)]
#[path = "captured_child_run_to_its_end_tests.rs"]
mod tests;
