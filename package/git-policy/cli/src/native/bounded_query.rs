//! What:
//!  Run one read-only Git query with a deadline and separate output streams.
//! Why:
//!  The auto-push asks Git a handful of questions before it pushes,
//!  and a question that never returns would hold the commit and the branch's push lock for good.
//!  A query that outlives its deadline is stopped and reported, so the commit stays saved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { exitCode, stdout, stderr } = await nanoSpawn(gitPath, args, { cwd, signal: AbortSignal.timeout(deadlineMs) });
//! ```

/// The failure type of the auto-push coordination.
use super::auto_push_single_flight::SingleFlightError;
/// The deadline runner shared with the push, and its stop causes.
use super::bounded_push::{MAX_PUSH_OUTPUT, StopCause, push_deadline, stop_push, wait_until_done};
/// Starts that ride out a short shortage of processes.
/// The anonymous output files of the query, made in the per-user scratch directory.
use super::scratch_location::scratch_file;
use super::start_retry::RetryStart;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// The anonymous output files and their reading back.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
/// `Command` describes the query; `Stdio` connects its streams.
use std::process::{Command, ExitStatus, Stdio};
/// Time spans.
use std::time::Duration;

/// The longest a query may run whatever the push deadline is:
///  queries read local repository state and finish in milliseconds.
pub const QUERY_CEILING: Duration = Duration::from_secs(120);

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
///  The deadline of a query: the push deadline, but never more than the ceiling.
/// Why:
///   One setting bounds every Git child of the auto-push, and a long push deadline must not stretch queries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.min(pushDeadlineMs(env), QUERY_CEILING_MS)
/// ```
pub fn query_deadline(environment: &[(OsString, OsString)]) -> Duration {
    return push_deadline(environment).min(QUERY_CEILING);
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
///  Run a query command to its end or its deadline.
/// Why:
///   Both streams go to anonymous files, so no reader thread can outlive a stopped query.
///  A query past its deadline is a coordination failure, not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBoundedQuery({ command, arguments, deadlineMs, graceMs }): Promise<QueryOutput>;
/// ```
pub fn run_bounded_query(
    mut command: Command,
    arguments: &str,
    deadline: Duration,
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
    let mut child = command.spawn_retrying().map_err(SingleFlightError::Start)?;
    drop(command);
    let ended: Result<ExitStatus, StopCause> =
        wait_until_done(&mut child, &[&stdout, &stderr], deadline)?;
    return match ended {
        Ok(status) => Ok(QueryOutput {
            code: status.code(),
            stdout: read_back(&mut stdout)?,
            stderr: read_back(&mut stderr)?,
        }),
        Err(cause) => {
            stop_push(&mut child, grace);
            Err(match cause {
                StopCause::Deadline => SingleFlightError::TimedOut {
                    arguments: arguments.to_owned(),
                    seconds: deadline.as_secs(),
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

/// Query deadlines are controlled in the test module.
#[cfg(test)]
#[path = "bounded_query_tests.rs"]
mod tests;
