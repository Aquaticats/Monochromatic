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
use std::process::{Command, Output};
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
/// the wait is a thread whose result is
///  taken with a deadline,
/// and a query past the deadline is ended and reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await withDeadline(waitFor(child), ceiling) ?? (kill(child), throw ceilingReached());
/// ```
pub fn output_of_within_bound(command: &mut Command, ceiling: Duration) -> std::io::Result<Output> {
    let program: String = command.get_program().to_string_lossy().into_owned();
    let capture: Capture = Capture::new()?;
    let (standard_output, standard_error) = capture.stdio()?;
    command.stdout(standard_output).stderr(standard_error);
    let child: std::process::Child = crate::start_retry::RetryStart::spawn_retrying(command)?;
    let pid: u32 = child.id();
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::ExitStatus>>();
    // The child moves to a thread that waits for it and reaps it, so this caller can stop waiting at its deadline.
    let waiting = std::thread::Builder::new().name(String::from("cli-git-query-wait")).spawn(move || {
        let mut waiting_child: std::process::Child = child;
        // A caller that stopped waiting took the failure; there is nobody left to tell.
        let _abandoned = sender.send(waiting_child.wait());
    });
    if let Err(error) = waiting {
        // Nothing waits for the child, and the caller is told rather than the child being left to run unwatched.
        end_by_number(pid, libc::SIGKILL);
        return Err(std::io::Error::other(format!(
            "{program} was started but no thread could be started to wait for it: {error}. Hint: check the process limit (`ulimit -u`) \
             and the control group's `pids.max`, free some, and run the command again"
        )));
    }
    match receiver.recv_timeout(ceiling) {
        Ok(Ok(status)) => {
            let captured = capture.snapshot(STEP_OUTPUT_LIMIT);
            return Ok(Output { status, stdout: captured.stdout, stderr: captured.stderr });
        }
        Ok(Err(error)) => return Err(error),
        Err(_timed_out) => {
            // The thread owns the child and has not reaped it (this caller stopped waiting first), so the number still names
            //  the child and cannot have been given to another process.
            end_by_number(pid, libc::SIGKILL);
            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, query_ceiling_text(program.as_str(), ceiling)));
        }
    }
}

/// What:
///  The sentence for a query that did not answer within its ceiling and was ended.
/// Why:
///  The person has to know the query is what was stopped,
/// the limit it met,
/// and that nothing of the repository was changed by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${program} did not answer within ${seconds} seconds ... and was ended ...`
/// ```
fn query_ceiling_text(program: &str, ceiling: Duration) -> String {
    return format!(
        "{program} did not answer within {} seconds, the longest a read-only query of this wrapper waits, and was ended. Such a \
         query changes nothing of the repository. Hint: check with `ps` what that program waits for, look at the disk it reads, and \
         run the command again.",
        ceiling.as_secs()
    );
}

/// Send `signal` to the process with this number,
/// whose child this process has not reaped,
/// so the number cannot name another process yet.
fn end_by_number(pid: u32, signal: libc::c_int) {
    let Ok(number) = libc::pid_t::try_from(pid) else {
        return;
    };
    // SAFETY: `kill` takes two integers and touches no memory; the child was not reaped when this was called.
    let _ended: libc::c_int = unsafe { libc::kill(number, signal) };
}

/// Captured-run controls stay out of the release executable.
#[cfg(test)]
#[path = "captured_child_run_to_its_end_tests.rs"]
mod tests;
