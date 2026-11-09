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
use crate::output_capture::{Capture, STEP_OUTPUT_LIMIT};
use crate::start_retry::spawn_retrying_without_blocking;
use std::process::{Child, Command, Output};
use std::time::Duration;

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
    let child: std::process::Child = spawn_retrying_without_blocking(command).await?;
    let mut owned = adopt(child);
    let status: std::process::ExitStatus = owned.wait().await?;
    let captured = capture.snapshot(STEP_OUTPUT_LIMIT);
    return Ok(Output { status, stdout: captured.stdout, stderr: captured.stderr });
}

/// What:
///  `output_of` for a synchronous caller whose wait carries a ceiling of its own,
/// without entering the runtime.
/// Why:
///   A read-only query is asked from code that may already be a task of the runtime,
/// where entering it would never
///  end,
/// and a query that never answers held the command for good
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 5).
///  The output goes to anonymous files,
/// so no pipe can fill and no reader is needed;
/// the wait is made by this thread,
///  which owns the child,
/// so the ceiling ends the child here and no signal is ever sent to a number
///  another thread could have reaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = waitWithin(child, ceiling) ?? (child.kill(), throw ceilingReached());
/// ```
pub fn output_of_within_bound(command: &mut Command, ceiling: Duration) -> std::io::Result<Output> {
    let program: String = command.get_program().to_string_lossy().into_owned();
    let capture: Capture = Capture::new()?;
    let (standard_output, standard_error) = capture.stdio()?;
    command.stdout(standard_output).stderr(standard_error);
    let child: Child = crate::start_retry::RetryStart::spawn_retrying(command)?;
    // This thread has not reaped the child, so the ceiling's kill reaches it and nothing else, and the reap here leaves no
    //  zombie behind.
    let status: std::process::ExitStatus = match crate::child_waited_for_within_a_ceiling::wait_within(child, ceiling) {
        Ok(finished) => finished,
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
            let left_running: bool = crate::child_waited_for_within_a_ceiling::left_running_when_the_ceiling_was_reached(&error);
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                crate::child_waited_for_within_a_ceiling::ceiling_reached_text(program.as_str(), ceiling, left_running),
            ));
        }
        Err(error) => return Err(error),
    };
    let captured = capture.snapshot(STEP_OUTPUT_LIMIT);
    return Ok(Output { status, stdout: captured.stdout, stderr: captured.stderr });
}

/// Captured-run controls stay out of the release executable.
#[cfg(test)]
#[path = "captured_child_run_to_its_end_tests.rs"]
mod tests;
