//! What:
//!  Run one read-only Git query that is ended only when it shows no sign of work,
//! with separate output streams.
//! Why:
//!  The auto-push asks Git a handful of questions before it pushes,
//!  and a question that never returns would hold the service's push for good.
//!  A query that shows no sign of work for the quiet window is stopped and reported,
//! so the commit stays saved.
//! A query that
//!  works is never stopped for being slow:
//! no total time bounds it (owner,
//! 2026-11-03:
//! "Stop only on no progress";
//! the
//!  coordinator's scope addition to picks 4 and 9 of `doc/planning/cli-git-shipped-limits-audit.md`,
//! open to the owner's
//!  veto).
//! Before this,
//! a query shared the push's wall-clock stop and the journal showed a `git status` stopped at
//!  2026-11-03T08:28 with `outcome=retrying push_ms=97398`.
//! The longest quiet gap of every query is kept in the ledger.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { exitCode, stdout, stderr } = await nanoSpawn(gitPath, args, { cwd, signal: quietFor(windowMs) }); // quiet, not slow
//! ```

/// The failure type of the auto-push coordination.
use crate::auto_push_single_flight::SingleFlightError;
/// The diagnostic log, for a start whose input thread could not be made.
use crate::diagnostic_log::debug;
/// The deadline runner shared with the push,
/// and its stop causes.
use crate::bounded_push::{MAX_PUSH_OUTPUT, StopCause, stop_push, wait_until_done_up_to};
use crate::push_deadline_and_patience_settings::{Patience, push_deadline};
/// The longest quiet gap of each step,
/// kept for the timing records.
use crate::quiet_ledger;
/// The lock files a stopped query leaves,
/// and the ledger that finds them.
use crate::child_watch::{ended_by_the_stop, refused_while_stopping};
use crate::stop_of_the_service_noticed_by_its_children::{StopArm, Unfinished, begin_unfinished_work};
use crate::stopped_git_locks::LockLedger;
/// Starts that ride out a short shortage of processes.
/// The anonymous output files of the query,
/// made in the per-user scratch directory.
use crate::scratch_location::scratch_file;
use crate::child_awaited_through_a_pidfd::{OwnedChild, adopt};
use crate::lazy_runtime::run_on_runtime;
use crate::start_retry::spawn_retrying_without_blocking;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// The anonymous output files and their reading back.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
/// `Command` describes the query;
/// `Stdio` connects its streams.
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
/// Time spans.
use std::time::Duration;

/// What:
///  What a finished query printed and how it ended.
/// Why:
///   Callers read an exit code as an answer ("not an ancestor") or a failure,
/// never both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type QueryOutput = { code: number | undefined; stdout: Uint8Array; stderr: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOutput {
    /// Exit code,
    /// or nothing when a signal ended the process.
    pub code: Option<i32>,
    /// Complete standard output,
    /// up to the output cap.
    pub stdout: Vec<u8>,
    /// Complete standard error,
    /// up to the output cap.
    pub stderr: Vec<u8>,
}

/// What:
///  The quiet window of a query:
/// the push's stall window.
/// Why:
///   One setting bounds how long every Git child of the auto-push may show no sign of work.
/// The cap of 120 seconds a query
///  used to have is gone with the wall-clock stop it belonged to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushStallMs(env)
/// ```
pub fn query_window(environment: &[(OsString, OsString)]) -> Duration {
    return push_deadline(environment);
}

/// Read one output file back from its start,
/// up to the output cap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readFile(path)
/// ```
fn read_back(file: &mut File, cap: u64) -> Result<Vec<u8>, SingleFlightError> {
    file.seek(SeekFrom::Start(0))
        .map_err(SingleFlightError::Start)?;
    let mut bytes: Vec<u8> = Vec::new();
    file.take(cap)
        .read_to_end(&mut bytes)
        .map_err(SingleFlightError::Start)?;
    return Ok(bytes);
}

/// What:
///  Run a query command to its end,
/// or until it has shown no sign of work for the quiet window.
/// Why:
///   Both streams go to anonymous files,
/// so no reader thread can outlive a stopped query.
///  A query that was stopped for being quiet is a coordination failure,
/// not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBoundedQuery({ command, arguments, windowMs, graceMs }): Promise<QueryOutput>;
/// ```
pub fn run_bounded_query(
    command: Command,
    arguments: &str,
    window: Duration,
    grace: Duration,
    environment: &[(OsString, OsString)],
) -> Result<QueryOutput, SingleFlightError> {
    return run_bounded_git(command, arguments, window, grace, environment, None, MAX_PUSH_OUTPUT);
}

/// What:
///  `run_bounded_git_awaiting` for a caller that is not inside the runtime.
/// Why:
///   The reconcile's second pass is a synchronous value its callers use,
/// so it enters the runtime the way every other synchronous query does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(() => runBoundedGitAwaiting({ command, arguments, windowMs, graceMs, environment, input, cap }))
/// ```
pub fn run_bounded_git(
    command: Command,
    arguments: &str,
    window: Duration,
    grace: Duration,
    environment: &[(OsString, OsString)],
    input: Option<Vec<u8>>,
    cap: u64,
) -> Result<QueryOutput, SingleFlightError> {
    return match run_on_runtime(run_bounded_git_awaiting(command, arguments, window, grace, environment, input, cap)) {
        Ok(output) => output,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(refusal.to_string()))),
    };
}

/// What:
///  `run_bounded_query` as a task of the runtime:
/// the wait for the query is awaited,
/// and no worker is held while it runs.
/// Why:
///   The synchronous face enters the runtime until the callers are asynchronous themselves;
/// from inside the runtime that face
///  refuses,
/// and this one is called.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runBoundedQueryAwaiting({ command, arguments, windowMs, graceMs, environment });
/// ```
pub async fn run_bounded_query_awaiting(
    command: Command,
    arguments: &str,
    window: Duration,
    grace: Duration,
    environment: &[(OsString, OsString)],
) -> Result<QueryOutput, SingleFlightError> {
    return run_bounded_git_awaiting(command, arguments, window, grace, environment, None, MAX_PUSH_OUTPUT).await;
}

/// What:
///  Run one read-only Git command to its end,
/// or until it shows no sign of work for the quiet window or prints past `cap`,
/// with `input` given on its standard input.
/// Why:
///   The commands of the word-level second pass read their request from the standard input,
/// and the reconcile starts them in an object store another program may have
///  wedged:
/// the same meter,
/// the same stop of the service and the same grace that bound the queries must bound them,
/// or a
///  merge driver that never returns holds the push worker and the drain of its repository for good.
/// The input goes in from a thread of its own,
/// so a command that answers while it still reads cannot block on a full pipe;
/// a stopped
///  command's pipe closes and the thread's write fails instead of waiting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runBoundedGitAwaiting({ command, arguments, windowMs, graceMs, environment, input, cap });
/// ```
pub async fn run_bounded_git_awaiting(
    mut command: Command,
    arguments: &str,
    window: Duration,
    grace: Duration,
    environment: &[(OsString, OsString)],
    input: Option<Vec<u8>>,
    cap: u64,
) -> Result<QueryOutput, SingleFlightError> {
    // The service's shutdown waits for this query and ends it when the service is stopped; a query that would begin after the stop is not started.
    let mut unfinished: Unfinished = match begin_unfinished_work(false, "git query") {
        Ok(counted) => counted,
        Err(refusal) => return Err(SingleFlightError::Start(refused_while_stopping(&refusal))),
    };
    let stop: Option<StopArm> = unfinished.take_arm();
    let mut stdout: File = scratch_file(environment).map_err(SingleFlightError::Start)?;
    let mut stderr: File = scratch_file(environment).map_err(SingleFlightError::Start)?;
    let for_stdout: File = stdout.try_clone().map_err(SingleFlightError::Start)?;
    let for_stderr: File = stderr.try_clone().map_err(SingleFlightError::Start)?;
    command
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::from(for_stdout))
        .stderr(Stdio::from(for_stderr));
    // Taken before the query starts, so a lock born earlier is never taken for the query's.
    let locks = LockLedger::before_start(&command);
    let mut started: Child = spawn_retrying_without_blocking(&mut command).await.map_err(SingleFlightError::Start)?;
    drop(command);
    if let Some(bytes) = input
        && let Err(reason) = feed_standard_input(started.stdin.take(), bytes)
    {
        // The command never got its request, so it is stopped here and the failure names why.
        stop_push(adopt(started), grace, locks).await;
        return Err(SingleFlightError::Start(std::io::Error::other(reason)));
    }
    let mut child: OwnedChild = adopt(started);
    let mut quiet: Option<Duration> = None;
    let ended: Result<ExitStatus, StopCause> =
        wait_until_done_up_to(&mut child, &[&stdout, &stderr], Patience::Progress { stall: window }, cap, &mut quiet, stop.as_ref()).await?;
    let step: String = format!("git {}", arguments.split_whitespace().next().unwrap_or_default());
    if let Some(gap) = quiet {
        quiet_ledger::note(step.as_str(), gap, ended.is_err());
    }
    return match ended {
        Ok(status) => Ok(QueryOutput {
            code: status.code(),
            stdout: read_back(&mut stdout, cap)?,
            stderr: read_back(&mut stderr, cap)?,
        }),
        Err(cause) => {
            stop_push(child, grace, locks).await;
            Err(match cause {
                // A query waits on its signs of work, so only a stall reaches here; a deadline is the same wait.
                StopCause::Deadline | StopCause::Stalled(_) => SingleFlightError::TimedOut {
                    arguments: arguments.to_owned(),
                    seconds: window.as_secs(),
                },
                StopCause::ServiceStopped => SingleFlightError::Start(ended_by_the_stop(step.as_str())),
                StopCause::Flood => SingleFlightError::Git {
                    arguments: arguments.to_owned(),
                    code: None,
                    stderr: format!(
                        "the query wrote more than {MAX_PUSH_OUTPUT} bytes and was stopped"
                    ),
                },
            })
        }
    };
}

/// What:
///  Write `bytes` to the standard input of a started command from a thread of its own.
/// Why:
///   The command may answer before it has read all of its input,
/// so writing here could block on a full pipe while nothing reads the answer;
/// the thread's write fails once the command is stopped and its pipe closes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn(() => { try { pipe.write(bytes) } catch { /* the command was stopped */ } });
/// ```
fn feed_standard_input(pipe: Option<ChildStdin>, bytes: Vec<u8>) -> Result<(), String> {
    let Some(mut open) = pipe else {
        return Ok(());
    };
    return match crate::scoped_start::spawn_named("cli-git-second-pass-input", move || {
        if let Err(error) = open.write_all(bytes.as_slice()) {
            debug(TAG, format!("the input of a second-pass command was not taken: {error}").as_str());
        }
    }) {
        Ok(_writer) => Ok(()),
        Err(error) => Err(format!("no thread could be started to give the command its input: {error}")),
    };
}

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// Query windows are controlled in the test module.
#[cfg(test)]
#[path = "bounded_query_tests.rs"]
mod tests;
