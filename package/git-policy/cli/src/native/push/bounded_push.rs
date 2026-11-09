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
//!  A push whose remote already holds every reference it was asked to deliver is the one stop that is not a failure:
//! the remote's own `receive-pack` runs the hooks after the references are written,
//! so a stalled `post-receive` hook
//!  leaves the branch published;
//! the report then says what the remote holds,
//! names the remote's work as what was cut
//!  short,
//! and says that no push can run that hook again (`delivered_before_the_stop.rs`,
//! `SPEC.md`,
//! "A local remote's hooks run in the push's tree").
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
/// The question that proves a stopped push had already delivered its references.
use crate::delivered_before_the_stop::{AskingGit, DeliveryQuestion, globals_before_the_command, objects_the_push_will_send, refs_already_delivered};
pub use crate::stop_of_a_push_and_sweep_of_its_locks::{settle_locks, stop_push, sweep_after_exit};
use crate::start_retry::spawn_retrying_without_blocking;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// `File` is the shared anonymous output file;
/// `Read`,
/// `Seek` and `SeekFrom` read it back from its start.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
/// The program and the directory the push ran with,
/// for the question the stop asks.
use std::path::{Path, PathBuf};
/// `Child` is a push handed to the sweeper;
/// `Command` describes it;
/// `Stdio` connects its streams.
use std::process::{Child, Command, ExitStatus, Stdio};
/// Time spans and the monotonic clock.
use std::time::Duration;
pub use crate::push_deadline_and_patience_settings::{DEFAULT_PUSH_DEADLINE, PUSH_DEADLINE_VARIABLE, Patience, positive_seconds, push_deadline, push_patience, strip_one_line_break};

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// What:
///  The sentence a report of a stopped push carries when the remote already holds every reference the push was
///  asked to deliver.
/// Why:
///   A push to a local path runs the remote's `receive-pack` in the push's own tree,
/// and its `post-receive` hook runs
///  after the references are written,
/// so a hook that shows no sign of work stops a push that has already landed.
/// The report says
///  what the remote holds instead of calling the push a failure,
/// and the notice the service writes for the person
///  recognises this sentence to word itself the same way.
pub const DELIVERED_BEFORE_THE_STOP: &str = "git push delivered every reference it was asked for";

/// What a person is told when a push is ended for showing no progress:
/// the owner's answer of 2026-11-03,
/// "Bound it,
/// keep retrying
///  (Recommended)",
/// is that the service's retry plan keeps the push and tries it again,
/// and the line says so.
pub const PUSH_QUEUED_LINE: &str = "The push is queued: the commit service keeps it in its retry plan and tries it again.";

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

/// Exit code recorded for a push that this wrapper stopped,
///  the convention of `timeout(1)`.
pub const STOPPED_EXIT_CODE: i64 = 124;

/// What the line a push's output gains has after the name of the program,
/// when it waited with its connection open.
pub const WAIT_NOTE_PHRASE: &str = "is waiting for the server";

/// Whether an output of this size is a flood:
/// only a size past the cap is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// size > MAX_PUSH_OUTPUT
/// ```
fn is_flood(size: u64) -> bool {
    return size > MAX_PUSH_OUTPUT;
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

/// What a stop reports:
/// the limit that was hit,
/// what the push had moved and how long it had run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StopReport {
    /// The limit that was hit:
    /// the stall window.
    pub limit: Duration,
    /// The bytes the push and everything it started had read and written.
    pub moved: u64,
    /// How long the push had run.
    pub elapsed: Duration,
}

/// Why a push or query was stopped.
pub enum StopCause {
    /// The fixed deadline passed.
    Deadline,
    /// The push moved no data for the stall window.
    Stalled(StopReport),
    /// The output grew past the cap.
    Flood,
    /// The service was stopped,
    /// and its shutdown told every waiter to end its child.
    ServiceStopped,
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([exited, stalled, flood]); // never by elapsed time
/// ```
pub async fn wait_until_done(
    child: &mut OwnedChild,
    outputs: &[&File],
    patience: Patience,
    quiet: &mut Option<Duration>,
    stop: Option<&StopArm>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let mut waits: Vec<WaitReport> = Vec::new();
    let clock: Clock = Clock::start(child.id(), patience).without_wall_fallback();
    return wait_with_clock(child, outputs, clock, "git", &mut waits, quiet, stop).await;
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
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let clock: Clock = Clock::start(child.id(), patience);
    let mut unused: Option<Duration> = None;
    return wait_with_clock(child, outputs, clock, label, waits, &mut unused, stop).await;
}

/// The loop of both waits,
/// over a clock the caller made.
async fn wait_with_clock(
    child: &mut OwnedChild,
    outputs: &[&File],
    mut clock: Clock,
    label: &str,
    waits: &mut Vec<WaitReport>,
    quiet: &mut Option<Duration>,
    stop: Option<&StopArm>,
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
        for output in outputs {
            let size: u64 = match output.metadata() {
                Ok(metadata) => metadata.len(),
                Err(_) => u64::MAX,
            };
            if is_flood(size) {
                *quiet = quiet_gap_kept(&clock);
                return Ok(Err(StopCause::Flood));
            }
        }
        if let Some(cause) = clock.check() {
            *quiet = quiet_gap_kept(&clock);
            return Ok(Err(cause));
        }
        if let Some(report) = clock.take_wait_note() {
            // Said as the wait begins, so a long wait is read while it lasts (the service's log, or the person's terminal).
            warn(TAG, &wait_line(label, &report));
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
    while let Some(argument) = arguments.next() {
        let word: String = argument.to_string_lossy().into_owned();
        // `-c name=value` and `-C directory` take the next word as their value.
        if word == "-c" || word == "-C" {
            let _value = arguments.next();
            continue;
        }
        if word.starts_with('-') {
            continue;
        }
        return format!("{program} {word}");
    }
    return program;
}

/// What:
///  The line that names a run's wait for its remote.
/// Why:
///   The person or the journal reads that the run is not stuck:
/// the connection to the remote is open,
/// and what ends the wait.
///  No time ends it:
/// only the remote's answer,
/// the connection closing or TCP giving up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${label} ${WAIT_NOTE_PHRASE}: the connection to the remote is open (silent ${silent} seconds)`
/// ```
fn wait_line(label: &str, report: &WaitReport) -> String {
    return format!(
        "monochromatic-git-policy-cli: {label} {WAIT_NOTE_PHRASE}: the connection to the remote is open, and nothing has moved for {} seconds \
         (elapsed_seconds={}); the remote may be checking the push or preparing its answer. It is waited for until it answers or \
         the connection closes; no time limit applies while the connection is open.",
        report.silent.as_secs(),
        report.elapsed.as_secs(),
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
    // Taken before the push starts, so a lock born earlier is never taken for the push's.
    let locks: LockLedger = LockLedger::before_start(&command);
    let label: String = command_label(&command);
    // Read while the command still has its arguments: a push stopped after its remote wrote the references must not be
    //  called a failure, and what the remote holds is the only proof of that.
    let arguments: Vec<OsString> = command.get_args().map(OsString::from).collect();
    let asking: AskingGit = AskingGit {
        program: PathBuf::from(command.get_program()),
        cwd: command.get_current_dir().map(Path::to_path_buf),
        environment: environment.to_vec(),
        globals: globals_before_the_command(arguments.as_slice()).unwrap_or_default().to_vec(),
    };
    let question: Option<DeliveryQuestion> = DeliveryQuestion::of(arguments.as_slice());
    // The objects the push is about to send are recorded before it starts: the reference they come from may name
    //  another object by the time the push is stopped, and the remote must hold what was sent and not what the
    //  reference says afterwards. The recording is bounded like every other query, so it cannot hold the push up.
    let window: Duration = match patience {
        Patience::Fixed(limit) => limit,
        Patience::Progress { stall } => stall,
    };
    let sending: Option<Vec<String>> = match &question {
        Some(asked) => objects_the_push_will_send(&asking, asked, window, grace).await,
        None => None,
    };
    let started: Child = spawn_retrying_without_blocking(&mut command).await.map_err(SingleFlightError::Start)?;
    drop(command);
    let mut child: OwnedChild = adopt(started);
    let mut waits: Vec<WaitReport> = Vec::new();
    let ended: Result<ExitStatus, StopCause> = wait_until_done_noting(&mut child, &[&output], patience, label.as_str(), &mut waits, stop.as_ref()).await?;
    if ended.is_err() {
        stop_push(child, grace, locks).await;
    }
    // A stop that ended a push whose remote had already written the references is not a push that failed: the proof is
    //  taken after the stop, when the remote's own program can no longer be writing,
    // it compares the remote against the objects recorded before the push started,
    // and it is answered in the time a killed
    //  child is given, so a remote that does not answer leaves the report as it was.
    let delivered: Option<Vec<String>> = match (&ended, question, sending) {
        (Err(StopCause::Stalled(_)), Some(asked), Some(sent)) => refs_already_delivered(&asking, &asked, sent.as_slice(), grace, grace).await,
        _ => None,
    };
    let captured: String = read_output(&mut output)?;
    let mut shown_text: String = strip_one_line_break(captured.as_str()).to_owned();
    // A wait is said once more, in the push's own output (the journal's `pushed` entry and the person's terminal carry it).
    for report in &waits {
        let line: String = wait_line(label.as_str(), report);
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
                StopCause::Stalled(report) => match &delivered {
                    Some(refs) => format!(
                        "monochromatic-git-policy-cli: {DELIVERED_BEFORE_THE_STOP} and then showed no sign of work for {} seconds while the \
                         remote ran the work that follows that update, such as a `post-receive` hook, so the push was stopped and that \
                         work was ended with it (limit=stall, bytes_moved={}, elapsed_seconds={}). The remote holds {}: nothing is left to \
                         deliver, and pushing again runs no remote hook, so it does not restart that work. If the remote's work matters (a \
                         deploy, a mirror, a ledger append), it is unfinished and has to be done by hand at the remote; {}.",
                        report.limit.as_secs(),
                        report.moved,
                        report.elapsed.as_secs(),
                        refs.join(", "),
                        extend_clause()
                    ),
                    None => format!(
                        "monochromatic-git-policy-cli: git push made no progress for {} seconds and was stopped (limit=stall, bytes_moved={}, \
                         elapsed_seconds={}); {}. {PUSH_QUEUED_LINE} Hint: if you were pushing over a slow or \
                         offline connection, your commit is already saved; push again when the connection works.",
                        report.limit.as_secs(),
                        report.moved,
                        report.elapsed.as_secs(),
                        extend_clause()
                    ),
                },
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
