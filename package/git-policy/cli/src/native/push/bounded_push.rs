//! What:
//!  Run the auto-push's real `git push` (and its fetch and Git LFS upload) until it ends,
//! or stops showing any sign of work
//!  with no connection to a remote open.
//! Why:
//!  A push that shows nothing and holds no connection would otherwise hold the service's push of that branch forever,
//!  and every later push of the repository would wait behind it.
//!  Such a push is stopped when its stall window passes,
//!  the commit stays saved locally,
//!  and the failure is reported like any other failed push.
//!  A stopped push is reported as what is known and no more:
//! that it was stopped,
//! that the remote may or may not have
//!  taken it,
//! and whether a retry is planned.
//! Nothing here asks the remote what it holds:
//! the wrapper cannot know
//!  which object Git really sent,
//! which of a remote's push URLs went first or what configuration shaped the attempt,
//! and a claim of delivery a person acts on has to hold for all of them.
//! The retry settles the question instead,
//! because
//!  a push of something the remote already has succeeds and clears the record (`SPEC.md`,
//! "A local remote's hooks run in the
//!  push's tree").
//!  A push whose connection to the remote is open is never stopped for time (owner,
//! 2026-11-03:
//! "Wait while connected
//!  (Recommended)";
//! pick 5 of `doc/planning/cli-git-shipped-limits-audit.md`,
//! the coordinator's under the owner's delegation and
//!  open to the owner's veto):
//! it has no ceiling,
//! and the wrapper sets no low-speed abort for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { exitCode, output } = await nanoSpawn(gitPath, pushArgs, { cwd, signal: AbortSignal.timeout(deadlineMs) });
//! ```

/// The attempt record a finished or stopped push produces.
use crate::auto_push_single_flight::{PushAttempt, SingleFlightError};
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::{debug, warn};
/// How to allow a push longer,
/// in the words of the program that runs it.
use crate::push_deadline_advice::extend_clause;
/// The clock that stops a push on a stall,
/// and the wait it lets pass instead.
use crate::push_progress::{Clock, WaitReport};
/// The lock files a stopped push leaves,
/// and the ledger that finds them.
use crate::stop_of_the_service_noticed_by_its_children::{StopArm, Unfinished, begin_unfinished_work};
use crate::stopped_git_locks::LockLedger;
/// Starts that ride out a short shortage of processes.
/// The anonymous output file of the push,
/// made in the per-user scratch directory.
use crate::scratch_location::scratch_file;
use crate::child_awaited_through_a_pidfd::{OwnedChild, adopt};
use crate::lazy_runtime::run_on_runtime;
/// The stop of a push and the sweep of the locks it leaves,
/// and the pause between two looks at a running push.
use crate::stop_of_a_push_and_sweep_of_its_locks::POLL_PAUSE;
pub use crate::stop_of_a_push_and_sweep_of_its_locks::{settle_locks, stop_push, sweep_after_exit};
use crate::start_retry::spawn_retrying_without_blocking;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// `File` is the shared anonymous output file;
/// `Read`,
/// `Seek` and `SeekFrom` read it back from its start.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
/// `Child` is a push handed to the sweeper;
/// `Command` describes it;
/// `Stdio` connects its streams.
use std::process::{Child, Command, ExitStatus, Stdio};
/// Time spans and the monotonic clock.
use std::time::Duration;
pub use crate::push_deadline_and_patience_settings::{DEFAULT_PUSH_DEADLINE, PUSH_DEADLINE_VARIABLE, Patience, positive_seconds, push_deadline, push_patience, strip_one_line_break};
/// The window a run whose connection is open may stay silent for.
use crate::push_deadline_and_patience_settings::connected_window;
/// The server a wait and a stop name,
/// and the word a command gave as its destination.
pub use crate::push_server_naming::{ServerNaming, first_word};

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// The causes and reports of a stop,
/// and the line a stalled one prints,
/// which `stop_cause_and_words.rs` owns.
pub use crate::stop_cause_and_words::{PUSH_QUEUED_LINE, STOPPED_DELIVERY_IS_NOT_KNOWN, STOPPED_EXIT_CODE, StopCause, StopReport, StopWindow};
use crate::stop_cause_and_words::stalled_reason;

/// How long a stopped push gets to end after the polite stop signal before it is killed.
///  Citation:
/// partial.
/// The helpers of a stopped push were measured on 2026-11-03 to outlive it for 60 seconds,
/// and a `git-lfs` that
///  never answers for good,
/// and empty tracking-ref locks appeared about 37 seconds after a stop;
/// those justify the kill that
///  follows the grace and the sweep,
/// not five seconds,
/// and no measurement of how long Git takes to remove its own locks on a
///  polite stop exists.
/// An open question for the owner.
pub const STOP_GRACE: Duration = Duration::from_secs(5);

/// The most output bytes kept from one push;
///  a push that writes more is stopped.
///  Citation:
/// none yet.
/// The owner decided "Keep all lines" for push output,
/// which this cap limits,
/// and no measurement of the
///  size of a push's output is logged;
/// the audit lists it as a memory bound to be measured.
pub const MAX_PUSH_OUTPUT: u64 = 4 * 1024 * 1024;

/// What the line a push's output gains has after the name of the program,
/// when it waited with its connection open.
pub const WAIT_NOTE_PHRASE: &str = "is waiting for the server";

/// Whether an output of this size is a flood:
/// only a size past the cap is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// size > cap
/// ```
fn is_flood(size: u64, cap: u64) -> bool {
    return size > cap;
}

/// What:
///  Whether one of a run's output files has grown past the cap.
/// Why:
///   The wait looks at these between its polls,
/// and a run that ends in the pause between two looks must be judged by the same bound:
/// what it printed past the cap is not all of what it printed.
///  A size that cannot be read counts as past the cap,
/// so nothing unreadable passes for a complete answer either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outputs.some((output) => isFlood(statSize(output)))
/// ```
pub(crate) fn flooded(outputs: &[&File], cap: u64) -> bool {
    for output in outputs {
        let size: u64 = match output.metadata() {
            Ok(metadata) => metadata.len(),
            Err(_) => u64::MAX,
        };
        if is_flood(size, cap) {
            return true;
        }
    }
    return false;
}

/// What a clock keeps as the longest quiet gap:
/// its figure when it reads anything,
/// nothing when it reads nothing.
fn quiet_gap_kept(watched: &Clock) -> Option<Duration> {
    if watched.measures() {
        return Some(watched.longest_quiet());
    }
    return None;
}

/// Read the shared output file back from its start,
///  keeping at most the output cap so a flood is never held in memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readFile(outputPath, 'utf8')
/// ```
fn read_output(file: &mut File) -> Result<String, SingleFlightError> {
    file.seek(SeekFrom::Start(0))
        .map_err(SingleFlightError::Start)?;
    let mut bytes: Vec<u8> = Vec::new();
    file.take(MAX_PUSH_OUTPUT)
        .read_to_end(&mut bytes)
        .map_err(SingleFlightError::Start)?;
    return Ok(String::from_utf8_lossy(&bytes).into_owned());
}

/// What:
///  Look at a running read-only query until it ends,
/// stalls or floods its output,
/// and say how quiet it got.
/// Why:
///   `std` has no timed wait,
///  so the parent looks with `try_wait`,
///  which never blocks;
/// the clock reads the signs of work a few times a second.
/// A query that works is waited for however
///  long it takes:
/// it has no wall-clock bound (owner,
/// 2026-11-03:
/// "Stop only on no progress";
/// the coordinator's scope addition
///  to picks 4 and 9 of `doc/planning/cli-git-shipped-limits-audit.md`,
/// open to the owner's veto),
/// and on a system that counts
///  nothing it is never judged stalled.
/// `quiet` receives the longest quiet gap seen,
/// for the ledger,
/// and nothing where the system counts nothing.
/// The clock is the caller's,
/// because what a run may do where nothing can be measured is the caller's decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([exited, stalled, flood]); // never by elapsed time
/// ```
pub async fn wait_until_done(
    child: &mut OwnedChild,
    outputs: &[&File],
    clock: Clock,
    quiet: &mut Option<Duration>,
    stop: Option<&StopArm>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    return wait_until_done_up_to(child, outputs, clock, MAX_PUSH_OUTPUT, quiet, stop).await;
}

/// What:
///  The same wait as `wait_until_done`,
/// with the output ceiling it stops past given by the caller.
/// Why:
///   A query's answer is a report,
/// and the push report's cap bounds it;
/// the second pass of a merge reads the three stages of the files it may merge,
/// up to the
///  pass's own blob limit for each name it asks for,
/// so its ceiling is that sum and not the report's.
/// Nothing else may pass a ceiling above `MAX_PUSH_OUTPUT`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([exited, stalled, flood]); // flood: any watched file past `cap`
/// ```
pub async fn wait_until_done_up_to(
    child: &mut OwnedChild,
    outputs: &[&File],
    clock: Clock,
    cap: u64,
    quiet: &mut Option<Duration>,
    stop: Option<&StopArm>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let mut waits: Vec<WaitReport> = Vec::new();
    let mut naming: ServerNaming<'_> = ServerNaming::none(&[]);
    return wait_with_clock(child, outputs, clock, cap, "git", &mut waits, quiet, stop, &mut naming).await;
}

/// What:
///  The same wait as `wait_until_done` for a push,
/// handing over the wait of a push that is waiting for its remote with the
///  connection open.
/// Why:
///   The push is not stopped while it waits;
/// the caller says so once,
/// in a line,
/// and the wait ends the way any other does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([exited, stalled, flood]); // waits.push(report) when the push begins to wait for the server
/// ```
pub async fn wait_until_done_noting(
    child: &mut OwnedChild,
    outputs: &[&File],
    patience: Patience,
    label: &str,
    waits: &mut Vec<WaitReport>,
    stop: Option<&StopArm>,
    naming: &mut ServerNaming<'_>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let clock: Clock = Clock::start(child.id(), patience).with_connected_window(connected_window(naming.environment));
    let mut unused: Option<Duration> = None;
    return wait_with_clock(child, outputs, clock, MAX_PUSH_OUTPUT, label, waits, &mut unused, stop, naming).await;
}

/// The loop of both waits,
/// over a clock the caller made.
async fn wait_with_clock(
    child: &mut OwnedChild,
    outputs: &[&File],
    mut clock: Clock,
    cap: u64,
    label: &str,
    waits: &mut Vec<WaitReport>,
    quiet: &mut Option<Duration>,
    stop: Option<&StopArm>,
    naming: &mut ServerNaming<'_>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    // Nothing is kept where the clock reads nothing: its figure would be the elapsed time, not a quiet gap.
    loop {
        // The exit is awaited on the child's pidfd; the pause is the look at the output sizes and the clock, a few times a second.
        if let Ok(waited) = tokio::time::timeout(POLL_PAUSE, child.wait()).await {
            let status: ExitStatus = waited.map_err(SingleFlightError::Start)?;
            *quiet = quiet_gap_kept(&clock);
            return Ok(Ok(status));
        }
        // The service's shutdown told the stop: the wait ends here, and the caller ends the child with its own grace.
        if stop.is_some_and(StopArm::is_told) {
            *quiet = quiet_gap_kept(&clock);
            return Ok(Err(StopCause::ServiceStopped));
        }
        if flooded(outputs, cap) {
            *quiet = quiet_gap_kept(&clock);
            return Ok(Err(StopCause::Flood));
        }
        if let Some(cause) = clock.check() {
            *quiet = quiet_gap_kept(&clock);
            return Ok(Err(cause));
        }
        if let Some(report) = clock.take_wait_note() {
            // Said as the wait begins, so a long wait is read while it lasts (the service's log, or the person's terminal).
            let server: Option<&str> = naming.server();
            warn(TAG, &wait_line(label, &report, server));
            waits.push(report);
        }
    }
}

/// What:
///  The name a run goes by in the lines about it:
/// the program's file name and its first word that is not an option,
/// as in
///  `git push`,
/// `git fetch` or `git-lfs push`.
/// Why:
///   The wait line must say which of the wrapper's network runs is waiting,
/// and the command is the only thing that knows.
/// The
///  values of Git's `-c` and `-C` options are skipped with them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [program, ...args] = command; return [basename(program), args.find((word) => !word.startsWith('-'))].join(' ');
/// ```
pub fn command_label(command: &Command) -> String {
    let program: String = command.get_program().to_string_lossy().rsplit('/').next().unwrap_or_default().to_owned();
    let mut arguments = command.get_args();
    return match first_word(&mut arguments) {
        Some(word) => format!("{program} {word}"),
        None => program,
    };
}

/// What:
///  The line a push's output gains when more of it was printed than is kept.
/// Why:
///   The cap bounds what is held in memory,
/// and a person reading an attempt's output must know that what they see is the
///  head of it.
/// The push itself was not stopped,
/// so the line says what was kept and points at the program that printed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git push wrote more than ${MAX} bytes of output, so only the first ${MAX} bytes are kept here; the push itself was not stopped. Hint: if a hook or the remote prints this much, read the hook's own output by running it directly.`
/// ```
fn shortened_output_line() -> String {
    return format!(
        "monochromatic-git-policy-cli: git push wrote more than {MAX_PUSH_OUTPUT} bytes of output, so only the first {MAX_PUSH_OUTPUT} bytes \
         are kept here; the push itself was not stopped. Hint: if a hook or the remote prints this much, read the hook's own output \
         by running it directly."
    );
}

/// What:
///  The line that names a run's wait for its remote.
/// Why:
///   The person or the journal reads that the run is not stuck:
/// the connection to the remote is open,
/// and what ends the wait.
///  A wait that shows no sign of work ends at the connected window,
/// which the report carries,
/// so the line names the same limit the run will be stopped by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${label} ${WAIT_NOTE_PHRASE}: the connection to the remote is open (silent ${silent} seconds)`
/// ```
fn wait_line(label: &str, report: &WaitReport, named: Option<&str>) -> String {
    // What the line calls the other end: the server it named, or the remote a destination names nothing about.
    let other_end: String = match named {
        Some(host) => format!("the server `{host}`"),
        None => String::from("the remote"),
    };
    return format!(
        "monochromatic-git-policy-cli: {label} {WAIT_NOTE_PHRASE}: the connection to {other_end} is open, and nothing has moved for {} seconds \
         (elapsed_seconds={}); it may be checking the push or preparing its answer. It is waited for until it answers or \
         the connection closes, and a connection that stays open with nothing moving for {} seconds is stopped as a stall.",
        report.silent.as_secs(),
        report.elapsed.as_secs(),
        report.window.as_secs(),
    );
}

/// What:
///  Run a push command to its end or its deadline and capture its interleaved output,
/// after the reference locks that a
///  killed Git left in its repository have been cleared when their holder is proven gone.
/// Why:
///   Every start of the wrapper's own fetch and push,
/// the repetitions of a push included,
/// goes through here
///  (`push_lock_clearing.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBoundedPush({ command, patience, grace, environment }): Promise<PushAttempt>;
/// ```
pub fn run_bounded_push(
    command: Command,
    patience: Patience,
    grace: Duration,
    environment: &[(OsString, OsString)],
) -> Result<PushAttempt, SingleFlightError> {
    return crate::push_lock_clearing::run_clearing_locks(command, patience, grace, environment, &crate::stale_ref_lock_judge::Judge::here());
}

/// What:
///  Run a push command to its end or its deadline and capture its interleaved output.
/// Why:
///   Both streams share one anonymous file so lines keep the order Git wrote them,
///  and a file needs no reader thread that a stuck helper could keep alive.
///  A stopped push becomes an attempt that says so,
///  so the commit stays saved and the report is recorded for later invocations;
/// an attempt whose references the remote
///  already holds says that instead,
/// so a published branch is never reported as a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBoundedPush({ command, deadlineMs, graceMs }): Promise<PushAttempt>;
/// ```
pub fn run_bounded_push_once(
    command: Command,
    patience: Patience,
    grace: Duration,
    environment: &[(OsString, OsString)],
) -> Result<PushAttempt, SingleFlightError> {
    return match run_on_runtime(run_bounded_push_awaiting(command, patience, grace, environment)) {
        Ok(attempt) => attempt,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(refusal.to_string()))),
    };
}

/// What:
///  `run_bounded_push_once` as a task of the runtime:
/// every wait in it is awaited,
/// and no worker is held while the push runs.
/// Why:
///   The synchronous face enters the runtime until the callers are asynchronous themselves;
/// from inside the runtime that face
///  refuses,
/// and this one is called.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = await runBoundedPushAwaiting({ command, patience, graceMs, environment });
/// ```
pub async fn run_bounded_push_awaiting(
    mut command: Command,
    patience: Patience,
    grace: Duration,
    environment: &[(OsString, OsString)],
) -> Result<PushAttempt, SingleFlightError> {
    // The service's shutdown waits for this push and ends it when the service is stopped; a push that would begin after the stop is not
    //  started, and is queued for the next service as every stopped push is.
    let mut unfinished: Unfinished = match begin_unfinished_work(false, "git push") {
        Ok(counted) => counted,
        Err(refusal) => {
            return Ok(PushAttempt { exit_code: STOPPED_EXIT_CODE, output: format!("monochromatic-git-policy-cli: {refusal}. {PUSH_QUEUED_LINE}") });
        }
    };
    let stop: Option<StopArm> = unfinished.take_arm();
    let mut output: File = scratch_file(environment).map_err(SingleFlightError::Start)?;
    let for_stdout: File = output.try_clone().map_err(SingleFlightError::Start)?;
    let for_stderr: File = output.try_clone().map_err(SingleFlightError::Start)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(for_stdout))
        .stderr(Stdio::from(for_stderr));
    let label: String = command_label(&command);
    // How the lines a wait and a stop print name the server, read from the command before it is given to the child, and asked
    //  before the locks are noted so that nothing this query does is taken for the push's own.
    let mut naming: ServerNaming<'_> = ServerNaming::of(&command, environment);
    naming.ask().await;
    // Taken before the push starts, so a lock born earlier is never taken for the push's.
    let locks: LockLedger = LockLedger::before_start(&command);
    let started: Child = spawn_retrying_without_blocking(&mut command).await.map_err(SingleFlightError::Start)?;
    drop(command);
    let mut child: OwnedChild = adopt(started);
    let mut waits: Vec<WaitReport> = Vec::new();
    let ended: Result<ExitStatus, StopCause> =
        wait_until_done_noting(&mut child, &[&output], patience, label.as_str(), &mut waits, stop.as_ref(), &mut naming).await?;
    if ended.is_err() {
        stop_push(child, grace, locks).await;
    }
    // A push that ends in the pause between two looks can still have printed past the cap. Its output is the log a person
    //  reads, not Git's answer, so the attempt keeps the push's own status; but the text kept is the head of it, and passing
    //  that for all of it would be the same silence a shortened answer is.
    let shortened: bool = ended.is_ok() && flooded(&[&output], MAX_PUSH_OUTPUT);
    let captured: String = read_output(&mut output)?;
    let mut shown_text: String = strip_one_line_break(captured.as_str()).to_owned();
    if shortened {
        if !shown_text.is_empty() {
            shown_text.push('\n');
        }
        shown_text.push_str(shortened_output_line().as_str());
    }
    // A wait is said once more, in the push's own output (the journal's `pushed` entry and the person's terminal carry it).
    let server: Option<String> = naming.server().map(str::to_owned);
    for report in &waits {
        let line: String = wait_line(label.as_str(), report, server.as_deref());
        if !shown_text.is_empty() {
            shown_text.push('\n');
        }
        shown_text.push_str(line.as_str());
    }
    let shown: &str = shown_text.as_str();
    return Ok(match ended {
        Ok(status) => PushAttempt {
            exit_code: i64::from(status.code().unwrap_or(1)),
            output: shown.to_owned(),
        },
        Err(cause) => {
            let reason: String = match cause {
                StopCause::Deadline => format!(
                    "monochromatic-git-policy-cli: git push did not finish within {} seconds and was stopped (limit=deadline); \
                     {}. {PUSH_QUEUED_LINE} Hint: if you were pushing over a slow or \
                     offline connection, your commit is already saved; push again when the connection works.",
                    match patience {
                        Patience::Fixed(limit) => limit.as_secs(),
                        Patience::Progress { stall } => stall.as_secs(),
                    },
                    extend_clause()
                ),
                StopCause::Stalled(report) => stalled_reason(&report, server.as_deref()),
                StopCause::ServiceStopped => format!(
                    "monochromatic-git-policy-cli: git push was ended because the service was stopped (limit=service-stop); \
                     {PUSH_QUEUED_LINE}"
                ),
                StopCause::Flood => format!(
                    "monochromatic-git-policy-cli: git push wrote more than {MAX_PUSH_OUTPUT} bytes of output and was stopped. Hint: if a hook or the \
                     remote prints this much, run `git ls-remote` to see whether your commits arrived, and read the \
                     hook's own output by running it directly before pushing again."
                ),
            };
            debug(TAG, &reason);
            let joined: String = if shown.is_empty() {
                reason
            } else {
                format!("{shown}\n{reason}")
            };
            PushAttempt {
                exit_code: STOPPED_EXIT_CODE,
                output: joined,
            }
        }
    });
}

/// Bounded push behavior is controlled in the test module.
#[cfg(test)]
#[path = "bounded_push_tests.rs"]
mod tests;

/// Controls that the waits are awaited on the runtime stay out of the release executable.
#[cfg(test)]
#[path = "bounded_push_awaiting_tests.rs"]
mod awaiting_tests;
