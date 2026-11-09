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
use crate::pinned_member_signalled_through_a_pidfd::{Delivered, PinnedProcess, pin_of_our_own_child};
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
/// the wait is a thread whose result is
///  taken with a deadline,
/// and a query past the deadline is ended and reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await withDeadline(waitFor(child), ceiling) ?? (kill(child), throw ceilingReached());
/// ```
pub fn output_of_within_bound(command: &mut Command, ceiling: Duration) -> std::io::Result<Output> {
    return Ok(captured_run_within_bound_starting_with(command, ceiling, &start_a_named_thread)?.0);
}

/// What:
///  `output_of_within_bound` for a caller whose answer must be whole:
/// the same run,
/// but a snapshot that left anything
///  out is an error rather than an answer.
/// Why:
///   A metadata query's bytes are parsed or compared,
/// and the tail of a cut answer is a wrong answer
/// (`repo_read/git_metadata.rs`).
/// The reader this replaced returned the query's exact output,
/// so a query whose
///  output passes `STEP_OUTPUT_LIMIT` fails and says what was left out instead of answering with the last part
///  of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { status, stdout } = await withDeadline(waitFor(child), ceiling);
/// if (notes.length > 0) throw new Error(`the answer was cut: ${notes.join('; ')}`);
/// ```
pub fn output_of_whole_within_bound(command: &mut Command, ceiling: Duration) -> std::io::Result<Output> {
    let program: String = command.get_program().to_string_lossy().into_owned();
    let (output, notes): (Output, Vec<String>) = captured_run_within_bound_starting_with(command, ceiling, &start_a_named_thread)?;
    if !notes.is_empty() {
        return Err(std::io::Error::other(format!(
            "{program} answered, but what it printed does not fit what this command keeps of one stream: {}. Hint: run the same \
             command by hand with the same arguments to see the whole of its answer.",
            notes.join("; ")
        )));
    }
    return Ok(output);
}

/// What:
///  The body of `output_of_within_bound`,
/// with the start of its waiting thread supplied by the caller,
/// and the notes of the capture
///  returned beside the output.
/// Why:
///  A control has to make that start fail,
/// which no real process limit lets it do without starving the whole test process;
///  and a caller that must have the whole answer judges the notes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function capturedRunWithinBound(command, ceiling, startThread): { output: Output; notes: string[] };
/// ```
fn captured_run_within_bound_starting_with(
    command: &mut Command,
    ceiling: Duration,
    start_thread: &dyn Fn(&str, Box<dyn FnOnce() + Send>) -> std::io::Result<std::thread::JoinHandle<()>>,
) -> std::io::Result<(Output, Vec<String>)> {
    let program: String = command.get_program().to_string_lossy().into_owned();
    let capture: Capture = Capture::new()?;
    let (standard_output, standard_error) = capture.stdio()?;
    command.stdout(standard_output).stderr(standard_error);
    // The child moves to a thread that waits for it and reaps it, so this caller can stop waiting at its deadline. The thread
    //  is started before the child, so a refused start costs the command its start and never leaves a killed child unreaped.
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::ExitStatus>>();
    let (hand_over, taken) = std::sync::mpsc::channel::<Child>();
    if let Err(error) = start_thread("cli-git-query-wait", Box::new(move || {
        // Nothing is handed over when the start below fails, and then there is no child to wait for.
        let Ok(mut child) = taken.recv() else {
            return;
        };
        // A caller that stopped waiting took the failure; there is nobody left to tell.
        let _abandoned = sender.send(child.wait());
    })) {
        return Err(std::io::Error::other(format!(
            "{program} was not started, because no thread could be started to wait for it: {error}. Hint: check the process limit \
             (`ulimit -u`) and the control group's `pids.max`, free some, and run the command again"
        )));
    }
    // Nothing is started until a thread exists to reap it.
    let child: Child = crate::start_retry::RetryStart::spawn_retrying(command)?;
    // The descriptor is taken while the number certainly names this process's own unreaped child, so the signal a ceiling
    //  sends later reaches that child and never a number that was reused after the waiting thread reaped it.
    let reach: Option<PinnedProcess> = pin_of_our_own_child(child.id());
    let _handed_over = hand_over.send(child);
    match receiver.recv_timeout(ceiling) {
        Ok(Ok(status)) => {
            let captured = capture.snapshot(STEP_OUTPUT_LIMIT);
            return Ok((Output { status, stdout: captured.stdout, stderr: captured.stderr }, captured.notes));
        }
        Ok(Err(error)) => return Err(error),
        Err(_timed_out) => {
            let ended: bool = end_the_child(&reach);
            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, query_ceiling_text(program.as_str(), ceiling, ended)));
        }
    }
}

/// Start a thread of this wrapper by name.
fn start_a_named_thread(name: &str, body: Box<dyn FnOnce() + Send>) -> std::io::Result<std::thread::JoinHandle<()>> {
    return std::thread::Builder::new().name(name.to_owned()).spawn(body);
}

/// What:
///  End a query past its ceiling with `SIGKILL`,
/// through the descriptor taken while the number certainly named it.
/// Why:
///  A number is a safe address only while its process is this process's own unreaped child,
/// and the thread that waits for
///  the query reaps it as soon as it ends.
/// The descriptor reaches that one process or fails,
/// and it never reaches a successor.
///  Says whether the signal was delivered: a host that gave no descriptor proves nothing about what the number names by now,
///  so nothing is signalled and the sentence says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endTheQuery(reach: Pin | undefined): boolean;
/// ```
fn end_the_child(reach: &Option<PinnedProcess>) -> bool {
    return match reach {
        Some(pinned) => pinned.signal(libc::SIGKILL) == Delivered::Sent,
        None => false,
    };
}

/// What:
///  The sentence for a query that did not answer within its ceiling and was ended.
/// Why:
///  The person has to know the query is what was stopped,
/// the limit it met,
/// and that nothing of the repository was changed by it;
///  a host that gave no descriptor for the query is told that it was not ended,
/// because a number proves nothing about the process it names once the query was reaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${program} did not answer within ${seconds} seconds ... and was ended ...`
/// ```
fn query_ceiling_text(program: &str, ceiling: Duration, ended: bool) -> String {
    let ending: &str = if ended {
        "and was ended"
    } else {
        "and nothing was signalled, because no descriptor naming it could be taken before it was reaped, so check with `ps` \
         whether it is still running"
    };
    return format!(
        "{program} did not answer within {} seconds, the longest a read-only query of this wrapper waits, {ending}. Such a query \
         changes nothing of the repository. Hint: check with `ps` what that program waits for, look at the disk it reads, and run \
         the command again.",
        ceiling.as_secs()
    );
}

/// Captured-run controls stay out of the release executable.
#[cfg(test)]
#[path = "captured_child_run_to_its_end_tests.rs"]
mod tests;
