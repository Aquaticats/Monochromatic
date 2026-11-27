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
//! The run of one command and the answer it gives sit in
//! `bounded_query_run_of_one_command.rs`,
//! which this file re-exports so no caller changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { exitCode, stdout, stderr } = await nanoSpawn(gitPath, args, { cwd, signal: quietFor(windowMs) }); // quiet, not slow
//! ```

/// The writer of a command's request,
/// the steps a control holds it by,
/// and the bound its own result is waited for under.
#[cfg(test)]
pub(crate) use crate::bounded_query_input::{
    InputWriter,
    WRITER_SETTLE_CEILING,
    feed_standard_input,
};
/// One read-only command run to its end,
/// which every face below names.
use crate::bounded_query_run_of_one_command::run_bounded_git_held;
/// The answer of one query,
/// and the clock its wait runs under,
/// which the controls of this module name.
pub use crate::bounded_query_run_of_one_command::{
    QueryOutput,
    query_clock,
};
/// The entry into the runtime a synchronous face makes.
use crate::lazy_runtime::run_on_runtime;
/// The failure type of the auto-push coordination.
use crate::one_push_attempt_and_its_failure::SingleFlightError;
/// The deadline a push may show nothing for.
use crate::push_deadline_and_patience_settings::push_deadline;
/// The clock a wait is judged by,
/// which a control of this module names.
#[cfg(test)]
pub(crate) use crate::push_progress::Clock;
/// The cap of the output a query keeps.
use crate::push_wait_until_done::MAX_PUSH_OUTPUT;
/// The stop a finished wait names,
/// which a control of this module names.
#[cfg(test)]
pub(crate) use crate::stop_cause_and_words::StopCause;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// `Command` describes the query.
use std::process::Command;
/// `Child` is a child a control starts;
/// `Stdio` connects its streams.
#[cfg(test)]
pub(crate) use std::process::{
    Child,
    Stdio,
};
/// Time spans.
use std::time::Duration;

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
    return match run_on_runtime(run_bounded_git_awaiting(
        command,
        arguments,
        window,
        None,
        grace,
        environment,
        None,
        MAX_PUSH_OUTPUT,
    )) {
        Ok(output) => output,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(
            refusal.to_string(),
        ))),
    };
}

/// What:
///  `run_bounded_git_awaiting` for a caller that is not inside the runtime.
/// Why:
///   The reconcile's second pass is a synchronous value its callers use,
/// so it enters the runtime the way every other synchronous query does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(() => runBoundedGitAwaiting({ command, arguments, windowMs, graceMs, environment, input }))
/// ```
pub fn run_bounded_git(
    command: Command,
    arguments: &str,
    window: Duration,
    ceiling: Option<Duration>,
    grace: Duration,
    environment: &[(OsString, OsString)],
    input: Option<Vec<u8>>,
    cap: u64,
) -> Result<QueryOutput, SingleFlightError> {
    return match run_on_runtime(run_bounded_git_awaiting(
        command,
        arguments,
        window,
        ceiling,
        grace,
        environment,
        input,
        cap,
    )) {
        Ok(output) => output,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(
            refusal.to_string(),
        ))),
    };
}

/// `run_bounded_git` with a control's step holding the writer between its write and the count that write stands for;
/// nothing outside a control,
/// and every shipped face passes none.
#[cfg(test)]
pub(crate) fn run_bounded_git_with_hold(
    command: Command,
    arguments: &str,
    window: Duration,
    grace: Duration,
    environment: &[(OsString, OsString)],
    input: Option<Vec<u8>>,
    hold: Box<dyn Fn() + Send + 'static>,
) -> Result<QueryOutput, SingleFlightError> {
    return match run_on_runtime(run_bounded_git_held(
        command,
        arguments,
        window,
        None,
        grace,
        environment,
        input,
        MAX_PUSH_OUTPUT,
        Some(hold),
    )) {
        Ok(output) => output,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(
            refusal.to_string(),
        ))),
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
    return run_bounded_git_awaiting(
        command,
        arguments,
        window,
        None,
        grace,
        environment,
        None,
        MAX_PUSH_OUTPUT,
    )
    .await;
}

/// What:
///  Run one read-only Git command to its end,
/// or until it has shown no sign of work for the quiet window,
/// with `input` given on its standard input.
/// Why:
///   `merge-tree` and every command of the word-level second pass read their request from the standard input,
/// and they are started by the reconcile,
/// which runs them in an object store another program may have wedged:
///  the same meter,
/// the same stop of the service and the same grace that bound the queries must bound them.
///  The input goes in from a thread of its own,
/// so a command that answers while it still reads cannot block on a full pipe;
/// a stopped command's pipe closes,
/// and the thread's write fails instead of waiting for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runBoundedGitAwaiting({ command, arguments, windowMs, graceMs, environment, input });
/// ```
pub async fn run_bounded_git_awaiting(
    command: Command,
    arguments: &str,
    window: Duration,
    ceiling: Option<Duration>,
    grace: Duration,
    environment: &[(OsString, OsString)],
    input: Option<Vec<u8>>,
    cap: u64,
) -> Result<QueryOutput, SingleFlightError> {
    return run_bounded_git_held(
        command,
        arguments,
        window,
        ceiling,
        grace,
        environment,
        input,
        cap,
        None,
    )
    .await;
}

/// Query windows are controlled in the test module.
#[cfg(test)]
#[path = "bounded_query_tests.rs"]
mod tests;
