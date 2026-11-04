//! What:
//!  Run one read-only Git query that is ended only when it shows no sign of work, with separate output streams.
//! Why:
//!  The auto-push asks Git a handful of questions before it pushes,
//!  and a question that never returns would hold the service's push for good.
//!  A query that shows no sign of work for the quiet window is stopped and reported, so the commit stays saved. A query that
//!  works is never stopped for being slow: no total time bounds it (owner, 2026-11-03: "Stop only on no progress"; the
//!  coordinator's scope addition to picks 4 and 9 of `doc/planning/cli-git-shipped-limits-audit.md`, open to the owner's
//!  veto). Before this, a query shared the push's wall-clock stop and the journal showed a `git status` stopped at
//!  2026-11-03T08:28 with `outcome=retrying push_ms=97398`. The longest quiet gap of every query is kept in the ledger.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { exitCode, stdout, stderr } = await nanoSpawn(gitPath, args, { cwd, signal: quietFor(windowMs) }); // quiet, not slow
//! ```

/// The failure type of the auto-push coordination.
use crate::auto_push_single_flight::SingleFlightError;
/// The deadline runner shared with the push, and its stop causes.
use crate::bounded_push::{MAX_PUSH_OUTPUT, Patience, StopCause, push_deadline, stop_push, wait_until_done};
/// The longest quiet gap of each step, kept for the timing records.
use crate::quiet_ledger;
/// The lock files a stopped query leaves, and the ledger that finds them.
use crate::stopped_git_locks::LockLedger;
/// Starts that ride out a short shortage of processes.
/// The anonymous output files of the query, made in the per-user scratch directory.
use crate::scratch_location::scratch_file;
use crate::start_retry::RetryStart;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// The anonymous output files and their reading back.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
/// `Command` describes the query; `Stdio` connects its streams.
use std::process::{Command, ExitStatus, Stdio};
/// Time spans.
use std::time::Duration;

/// What:
///  What a finished query printed and how it ended.
/// Why:
///   Callers read an exit code as an answer ("not an ancestor") or a failure, never both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type QueryOutput = { code: number | undefined; stdout: Uint8Array; stderr: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOutput {
    /// Exit code, or nothing when a signal ended the process.
    pub code: Option<i32>,
    /// Complete standard output, up to the output cap.
    pub stdout: Vec<u8>,
    /// Complete standard error, up to the output cap.
    pub stderr: Vec<u8>,
}

/// What:
///  The quiet window of a query: the push's stall window.
/// Why:
///   One setting bounds how long every Git child of the auto-push may show no sign of work. The cap of 120 seconds a query
///  used to have is gone with the wall-clock stop it belonged to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushStallMs(env)
/// ```
pub fn query_window(environment: &[(OsString, OsString)]) -> Duration {
    return push_deadline(environment);
}

/// Read one output file back from its start, up to the output cap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readFile(path)
/// ```
fn read_back(file: &mut File) -> Result<Vec<u8>, SingleFlightError> {
    file.seek(SeekFrom::Start(0))
        .map_err(SingleFlightError::Start)?;
    let mut bytes: Vec<u8> = Vec::new();
    file.take(MAX_PUSH_OUTPUT)
        .read_to_end(&mut bytes)
        .map_err(SingleFlightError::Start)?;
    return Ok(bytes);
}

/// What:
///  Run a query command to its end, or until it has shown no sign of work for the quiet window.
/// Why:
///   Both streams go to anonymous files, so no reader thread can outlive a stopped query.
///  A query that was stopped for being quiet is a coordination failure, not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBoundedQuery({ command, arguments, windowMs, graceMs }): Promise<QueryOutput>;
/// ```
pub fn run_bounded_query(
    mut command: Command,
    arguments: &str,
    window: Duration,
    grace: Duration,
    environment: &[(OsString, OsString)],
) -> Result<QueryOutput, SingleFlightError> {
    let mut stdout: File = scratch_file(environment).map_err(SingleFlightError::Start)?;
    let mut stderr: File = scratch_file(environment).map_err(SingleFlightError::Start)?;
    let for_stdout: File = stdout.try_clone().map_err(SingleFlightError::Start)?;
    let for_stderr: File = stderr.try_clone().map_err(SingleFlightError::Start)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(for_stdout))
        .stderr(Stdio::from(for_stderr));
    // Taken before the query starts, so a lock born earlier is never taken for the query's.
    let locks = LockLedger::before_start(&command);
    let mut child = command.spawn_retrying().map_err(SingleFlightError::Start)?;
    drop(command);
    let mut quiet: Option<Duration> = None;
    let ended: Result<ExitStatus, StopCause> =
        wait_until_done(&mut child, &[&stdout, &stderr], Patience::Progress { stall: window }, &mut quiet)?;
    let step: String = format!("git {}", arguments.split_whitespace().next().unwrap_or_default());
    if let Some(gap) = quiet {
        quiet_ledger::note(step.as_str(), gap, ended.is_err());
    }
    return match ended {
        Ok(status) => Ok(QueryOutput {
            code: status.code(),
            stdout: read_back(&mut stdout)?,
            stderr: read_back(&mut stderr)?,
        }),
        Err(cause) => {
            stop_push(child, grace, locks);
            Err(match cause {
                // A query waits on its signs of work, so only a stall reaches here; a deadline is the same wait.
                StopCause::Deadline | StopCause::Stalled(_) => SingleFlightError::TimedOut {
                    arguments: arguments.to_owned(),
                    seconds: window.as_secs(),
                },
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

/// Query windows are controlled in the test module.
#[cfg(test)]
#[path = "bounded_query_tests.rs"]
mod tests;
