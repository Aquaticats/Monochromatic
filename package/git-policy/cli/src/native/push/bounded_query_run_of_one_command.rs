//! What:
//!  One read-only Git command run to its end,
//!  or until it has shown no sign of work for its window,
//!  with the answer it printed and how it ended.
//! Why:
//!   A question that never returns would hold the service's push for good,
//!  so the run stops a command that shows no sign of work and reports it;
//!  both streams go to anonymous files,
//!  so no reader thread outlives a stopped command,
//!  and a command stopped for being quiet is a coordination failure,
//!  never an answer.
//!  The run sits in a file of its own,
//!  because the faces that name it and the window they pass it are more than the package's
//!  length limit allows one file (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = await runBoundedGitAwaiting({ command, arguments, windowMs, ceilingMs, graceMs, environment, input, cap });
//! ```

/// The writer of a command's request,
/// and the bound its own result is waited for under.
use crate::bounded_query_input::{
    InputWriter,
    WRITER_SETTLE_CEILING,
    feed_standard_input,
};
/// A child this process owns through its pidfd.
use crate::child_awaited_through_a_pidfd::{
    OwnedChild,
    adopt,
};
/// The word a stopped child is named by,
/// and the refusal a stop gives.
use crate::child_watch::{
    ended_by_the_stop,
    refused_while_stopping,
};
/// The failure type of the auto-push coordination.
use crate::one_push_attempt_and_its_failure::SingleFlightError;
/// The patience a wait runs under,
/// as the progress window or one fixed limit.
use crate::push_deadline_and_patience_settings::Patience;
/// The clock a wait is judged by,
/// and its reading of the signs of work.
use crate::push_progress::Clock;
/// The look that sees a flood,
/// and the wait that ends a run.
use crate::push_wait_until_done::{
    flooded,
    wait_until_done_up_to,
};
/// The longest quiet gap of each step,
/// kept for the timing records.
use crate::quiet_ledger;
/// The anonymous output file of the run,
/// made in the per-user scratch directory.
use crate::scratch_location::scratch_file;
/// Starts that ride out a short shortage of processes.
use crate::start_retry::spawn_retrying_without_blocking;
/// The stop a finished wait names,
/// which the run turns into its own failure.
use crate::stop_cause_and_words::StopCause;
/// The stop of a running push,
/// which ends a run past its window the same way.
use crate::stop_of_a_push_and_sweep_of_its_locks::stop_push;
use crate::stop_of_the_service_noticed_by_its_children::{
    StopArm,
    Unfinished,
    begin_unfinished_work,
};
use crate::stopped_git_locks::LockLedger;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// The anonymous output file and its reading back.
use std::fs::File;
use std::io::{
    Read,
    Seek,
    SeekFrom,
};
/// `Command` describes the run;
/// `Child` is the command this process owns;
/// `Stdio` connects its streams;
/// `ExitStatus` is how it ended.
use std::process::{
    Child,
    Command,
    ExitStatus,
    Stdio,
};
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
///  The clock a read-only query's wait runs under.
/// Why:
///   A query is read-only,
/// so ending one is safe,
/// and the window it is given is also the whole time it may run where the kernel does not count its bytes
/// (a hardened container,
/// `hidepid`,
/// a kernel without `CONFIG_TASK_IO_ACCOUNTING`).
/// Without that fallback nothing at all ended a hung `config --list`,
/// `rev-parse`,
/// `merge-base` or `for-each-ref` there,
/// and the push worker and the drain of its repository waited for it for good.
/// The window and its default are the owner's (`CLI_GIT_PUSH_DEADLINE_SECONDS`,
/// 30 seconds;
/// the rule "Stop only on no progress" and the journal's 794 landed pushes,
/// p50 3.5 s,
/// p99 26.4 s,
/// maximum 72.4 s,
/// are cited beside the value in `push_deadline_and_patience_settings.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Clock.start(pid, { stall: windowMs }) // the wall fallback kept
/// ```
pub fn query_clock(
    pid: u32,
    window: Duration,
) -> Clock {
    return Clock::start(
        pid,
        Patience::Progress {
            stall: window,
        },
    );
}

/// Read one output file back from its start,
/// up to the output cap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readFile(path)
/// ```
fn read_back(
    file: &mut File,
    cap: u64,
) -> Result<Vec<u8>, SingleFlightError> {
    file.seek(SeekFrom::Start(0))
        .map_err(SingleFlightError::Start)?;
    let mut bytes: Vec<u8> = Vec::new();
    file.take(cap)
        .read_to_end(&mut bytes)
        .map_err(SingleFlightError::Start)?;
    return Ok(bytes);
}

/// What:
///  The failure of a query that printed more than the cap.
/// Why:
///   The bytes kept are part of Git's answer,
/// so a status that came with them is not read as an answer at all;
/// the
///  words say what was kept and what was not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new GitFailure(arguments, undefined, `the query wrote more than ${MAX} bytes, so its output is not taken as Git's answer`)
/// ```
fn flood_of(
    arguments: &str,
    cap: u64,
) -> SingleFlightError {
    return SingleFlightError::Git {
        arguments: arguments.to_owned(),
        code: None,
        stderr: format!(
            "the query wrote more than {cap} bytes, so its output is not taken as Git's answer"
        ),
    };
}

/// What:
///  One read-only Git command run to its end,
///  or to its stall,
///  its ceiling or the stop of the service,
///  with its request on the standard input when it is given one.
/// Why:
///   The faces of `bounded_query.rs` differ only in which of those a caller names,
///  and all of them run the command here;
///  a control's step may hold the writer between its write and the count that write stands for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runBoundedGitHeld({ command, arguments, windowMs, ceilingMs, graceMs, environment, input, cap, hold });
/// ```
pub(crate) async fn run_bounded_git_held(
    mut command: Command,
    arguments: &str,
    window: Duration,
    ceiling: Option<Duration>,
    grace: Duration,
    environment: &[(OsString, OsString)],
    input: Option<Vec<u8>>,
    cap: u64,
    hold: Option<Box<dyn Fn() + Send + 'static>>,
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
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::from(for_stdout))
        .stderr(Stdio::from(for_stderr));
    // Taken before the query starts, so a lock born earlier is never taken for the query's.
    let locks = LockLedger::before_start(&command);
    let mut started: Child = spawn_retrying_without_blocking(&mut command)
        .await
        .map_err(SingleFlightError::Start)?;
    drop(command);
    let writer: Option<InputWriter> = match input {
        None => None,
        Some(bytes) => match feed_standard_input(started.stdin.take(), bytes, window, hold) {
            Ok(writer) => Some(writer),
            Err(error) => {
                // The command never got its request, so it is stopped here and the failure names why. The stop is the one every
                //  stopped query gets: asked to end, forced past the grace, and bounded, so a child the kernel does not release is
                //  handed to the sweep instead of being waited for.
                stop_push(adopt(started), grace, locks).await;
                return Err(SingleFlightError::Start(std::io::Error::other(error)));
            }
        },
    };
    let mut child: OwnedChild = adopt(started);
    let mut quiet: Option<Duration> = None;
    // The clock is made from the child's number before the wait borrows it.
    let clock: Clock = query_clock(child.id(), window);
    // The progress window and the total ceiling are two bounds: a silent command is stopped by the first, and one that keeps working without returning only by the second.
    let ended: Result<ExitStatus, StopCause> = match ceiling {
        Some(total) => match tokio::time::timeout(
            total,
            wait_until_done_up_to(
                &mut child,
                &[&stdout, &stderr],
                clock,
                cap,
                &mut quiet,
                stop.as_ref(),
            ),
        )
        .await
        {
            Ok(outcome) => outcome?,
            Err(_passed) => {
                stop_push(child, grace, locks).await;
                return Err(SingleFlightError::Ceiling {
                    arguments: arguments.to_owned(),
                    seconds: total.as_secs(),
                });
            }
        },
        None => {
            wait_until_done_up_to(
                &mut child,
                &[&stdout, &stderr],
                clock,
                cap,
                &mut quiet,
                stop.as_ref(),
            )
            .await?
        }
    };
    let step: String = format!(
        "git {}",
        arguments.split_whitespace().next().unwrap_or_default()
    );
    if let Some(gap) = quiet {
        quiet_ledger::note(step.as_str(), gap, ended.is_err());
    }
    // A child that ends in the pause between two looks can still have printed more than the cap, and the bytes kept are then
    //  part of its answer with the status of a whole one. The caller reads a status as an answer, so this is a flood, never a
    //  success with a shortened stream. Nothing is stopped: a child that ended is not there to end.
    if ended.is_ok() && flooded(&[&stdout, &stderr], cap) {
        return Err(flood_of(arguments, cap));
    }
    // A request that could not be written whole leaves the command answering from a prefix of it, so a status of its own is not
    //  an answer made from the request. The count is the writer's own finished result: a writer descheduled between its last
    //  write and its count has the request out already, so the answer waits for it up to the ceiling, and a writer still held
    //  after that is the blocked case the count names.
    if ended.is_ok()
        && let Some(mut writing) = writer
    {
        let _settled: bool = writing.settled(WRITER_SETTLE_CEILING).await;
        if let Some(reason) = writing.cut_short() {
            return Err(SingleFlightError::Git {
                arguments: arguments.to_owned(),
                code: None,
                stderr: format!("the request of this query could not be written whole: {reason}"),
            });
        }
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
                StopCause::ServiceStopped => {
                    SingleFlightError::Start(ended_by_the_stop(step.as_str()))
                }
                StopCause::Flood => flood_of(arguments, cap),
            })
        }
    };
}
