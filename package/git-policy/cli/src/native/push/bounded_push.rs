//! What:
//!  Run the auto-push's real `git push` (and its fetch and Git LFS upload) until it ends, or stops showing any sign of work
//!  with no connection to a remote open.
//! Why:
//!  A push that shows nothing and holds no connection would otherwise hold the service's push of that branch forever,
//!  and every later push of the repository would wait behind it.
//!  Such a push is stopped when its stall window passes,
//!  the commit stays saved locally,
//!  and the failure is reported like any other failed push.
//!  A push whose connection to the remote is open is never stopped for time (owner, 2026-11-03: "Wait while connected
//!  (Recommended)"; pick 5 of `doc/planning/cli-git-shipped-limits-audit.md`, the coordinator's under the owner's delegation and
//!  open to the owner's veto): it has no ceiling, and the wrapper sets no low-speed abort for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { exitCode, output } = await nanoSpawn(gitPath, pushArgs, { cwd, signal: AbortSignal.timeout(deadlineMs) });
//! ```

/// The attempt record a finished or stopped push produces.
use crate::auto_push_single_flight::{PushAttempt, SingleFlightError};
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::{debug, warn};
/// How to allow a push longer, in the words of the program that runs it.
use crate::push_deadline_advice::extend_clause;
/// The clock that stops a push on a stall, and the wait it lets pass instead.
use crate::push_progress::{Clock, WaitReport};
/// The helpers and hooks a stopped push leaves running.
use crate::stop_waits_while_a_member_cannot_be_interrupted::escalation_waits;
use crate::process_tree::{TreeMember, descendants_of, kill_members, member_of, still_running, terminate_members};
/// The lock files a stopped push leaves, and the ledger that finds them.
use crate::stopped_git_locks::LockLedger;
/// Starts that ride out a short shortage of processes.
/// The anonymous output file of the push, made in the per-user scratch directory.
use crate::scratch_location::scratch_file;
use crate::blocking_job::run_blocking_job;
use crate::child_awaited_through_a_pidfd::{OwnedChild, adopt};
use crate::lazy_runtime::{run_on_runtime, spawn_on_runtime};
use crate::start_retry::spawn_retrying_without_blocking;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// `File` is the shared anonymous output file; `Read`, `Seek` and `SeekFrom` read it back from its start.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
/// `Child` is a push handed to the sweeper; `Command` describes it; `Stdio` connects its streams.
use std::process::{Child, Command, ExitStatus, Stdio};
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};
pub use crate::push_deadline_and_patience_settings::{DEFAULT_PUSH_DEADLINE, PUSH_DEADLINE_VARIABLE, Patience, positive_seconds, push_deadline, push_patience, strip_one_line_break};

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// What a person is told when a push is ended for showing no progress: the owner's answer of 2026-11-03, "Bound it, keep retrying
///  (Recommended)", is that the service's retry plan keeps the push and tries it again, and the line says so.
pub const PUSH_QUEUED_LINE: &str = "The push is queued: the commit service keeps it in its retry plan and tries it again.";

/// How long a stopped push gets to end after the polite stop signal before it is killed.
///  Citation: partial. The helpers of a stopped push were measured on 2026-11-03 to outlive it for 60 seconds, and a `git-lfs` that
///  never answers for good, and empty tracking-ref locks appeared about 37 seconds after a stop; those justify the kill that
///  follows the grace and the sweep, not five seconds, and no measurement of how long Git takes to remove its own locks on a
///  polite stop exists. An open question for the owner.
pub const STOP_GRACE: Duration = Duration::from_secs(5);

/// The most output bytes kept from one push;
///  a push that writes more is stopped.
///  Citation: none yet. The owner decided "Keep all lines" for push output, which this cap limits, and no measurement of the
///  size of a push's output is logged; the audit lists it as a memory bound to be measured.
pub const MAX_PUSH_OUTPUT: u64 = 4 * 1024 * 1024;

/// Exit code recorded for a push that this wrapper stopped,
///  the convention of `timeout(1)`.
pub const STOPPED_EXIT_CODE: i64 = 124;

/// What the line a push's output gains has after the name of the program, when it waited with its connection open.
pub const WAIT_NOTE_PHRASE: &str = "is waiting for the server";

/// Pause between two looks at the running push.
const POLL_PAUSE: Duration = Duration::from_millis(10);

/// Ask a running push to end,
///  so Git can stop the helper processes it started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.kill('SIGTERM');
/// ```
#[cfg(unix)]
fn ask_to_stop(id: u32) {
    let Ok(pid) = libc::pid_t::try_from(id) else {
        return;
    };
    // SAFETY: `kill` sends one signal to the PID of a child this module started and has not reaped;
    // a failure is ignored because the child may have ended.
    unsafe { libc::kill(pid, libc::SIGTERM) };
}

/// Platforms without a polite stop signal go straight to the kill.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn ask_to_stop(_id: u32) {}

/// Whether part of the grace is still left after the stop request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// asked < grace
/// ```
fn grace_remains(asked: Duration, grace: Duration) -> bool {
    return asked < grace;
}

/// Whether an output of this size is a flood: only a size past the cap is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// size > MAX_PUSH_OUTPUT
/// ```
fn is_flood(size: u64) -> bool {
    return size > MAX_PUSH_OUTPUT;
}

/// Wait, bounded by `grace`, until every member of a stopped tree has ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([allEnded(tree), delay(graceMs)]);
/// ```
async fn wait_for_tree(tree: &[TreeMember], grace: Duration) {
    let waited: Instant = Instant::now();
    while grace_remains(waited.elapsed(), grace) && tree.iter().any(still_running) {
        tokio::time::sleep(POLL_PAUSE).await;
    }
    return;
}

/// What a clock keeps as the longest quiet gap: its figure when it reads anything, nothing when it reads nothing.
fn quiet_gap_kept(watched: &Clock) -> Option<Duration> {
    if watched.measures() {
        return Some(watched.longest_quiet());
    }
    return None;
}

/// What:
///  Collect a killed child that the kernel has not released, and sweep the locks it leaves, as a task of the runtime.
/// Why:
///   A killed process in an uninterruptible call (a stalled disk or file system) ends only when the kernel lets it go,
///  and on its way out it can create a lock file that nothing will remove (measured on 2026-11-03: empty tracking-ref
///  locks born about 37 seconds after the stop). The push is reported at its deadline and does not wait for that, so
///  the child is collected here, and its locks are swept when it is gone. The task holds no lock and ends with the child.
///  The child is handed over as the standard library's value and adopted inside the task, so a task that never runs
///  leaves it as it was and never waits for it. Returns nothing where the runtime has no worker to run the task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// void child.exited.then(() => locks.sweep(tree));
/// ```
pub fn sweep_after_exit(child: Child, tree: Vec<TreeMember>, locks: LockLedger) -> Option<tokio::task::JoinHandle<()>> {
    return sweep_task(Some(child), tree, locks);
}

/// `sweep_after_exit` for a push that may already have been collected: with no child there is only the tree to wait for.
fn sweep_task(child: Option<Child>, tree: Vec<TreeMember>, locks: LockLedger) -> Option<tokio::task::JoinHandle<()>> {
    let started = spawn_on_runtime(async move {
        if let Some(uncollected) = child {
            let mut owned: OwnedChild = adopt(uncollected);
            // The exit status is not reported: the push was reported at its deadline.
            let _status = owned.wait().await;
        }
        // Nothing waits for this task, so it waits for the rest of the tree as long as that runs.
        while tree.iter().any(still_running) {
            tokio::time::sleep(POLL_PAUSE).await;
        }
        // The sweep reads directories, so it runs on the blocking pool and not on the worker that polls tasks.
        if let Err(error) = run_blocking_job(move || { locks.sweep(tree.as_slice()); }).await {
            debug(TAG, &format!("the sweep of a stopped push's locks did not run: {error}"));
        }
    });
    if started.is_none() {
        debug(TAG, "the runtime has no worker to sweep a stopped push's locks");
    }
    return started;
}

/// What:
///  Stop a running push: ask politely,
///  wait the grace,
///  then kill and collect it,
///  and judge the reference locks it may have left.
/// Why:
///   Git's own signal handling ends the helpers it started and removes its lock files,
///  and a push that ignores the request cannot outlive the kill.
///  A killed push leaves its lock files, which fail every later Git command on the same refs, so once the push and
///  what it started are gone the locks they hold are swept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.kill('SIGTERM'); await Promise.race([exited, delay(graceMs)]) || child.kill('SIGKILL'); locks.sweep(tree.as_slice());
/// ```
pub async fn stop_push(mut child: OwnedChild, grace: Duration, locks: LockLedger) {
    // Read before the push is signalled: once it has exited, its helpers and hooks no longer point to it.
    let below: Vec<TreeMember> = i32::try_from(child.id()).map(descendants_of).unwrap_or_default();
    let mut tree: Vec<TreeMember> = i32::try_from(child.id()).ok().and_then(member_of).into_iter().collect();
    tree.extend(below.iter().cloned());
    ask_to_stop(child.id());
    terminate_members(below.as_slice());
    let asked: Instant = Instant::now();
    let mut ended: bool = false;
    // Past the grace the wait goes on while a member of the tree is in uninterruptible sleep, up to the stop's ceiling, because a kill
    //  then only cuts the cleanup that follows the call (Git removes a lock's pid file, then the lock).
    //  The exit is awaited on the pidfd; the pause is the look at the tree's state, made at the interval the loop always used.
    while !ended && escalation_waits(asked.elapsed(), grace, tree.as_slice()) {
        ended = tokio::time::timeout(POLL_PAUSE, child.wait()).await.is_ok();
    }
    if !ended {
        // The child is being abandoned; its exit is not reported, only the cause.
        let _killed = child.kill();
        // A killed process that sits in uninterruptible I/O (a stalled disk or network file system) cannot be
        //  collected until the kernel releases it, so the wait is bounded by the grace again and the child is then
        //  left to be collected by a task of its own, instead of holding the service's push for as long as the stall.
        if grace_remains(Duration::ZERO, grace) {
            // Whether it ended in time shows below: a collected child has nothing left to hand over.
            let _ended_in_time = tokio::time::timeout(grace, child.wait()).await;
        }
    }
    // What the push started outlives it: its transport helper holds the connection and its hook may never exit
    // (measured: 60 seconds for a helper, for good for a `git-lfs` that never answers). They were asked to end with
    // the push, so they get what is left of the grace and are then killed.
    while escalation_waits(asked.elapsed(), grace, below.as_slice()) && below.iter().any(still_running) {
        tokio::time::sleep(POLL_PAUSE).await;
    }
    kill_members(below.as_slice());
    // A child that was collected is gone; one that was not goes on to the task that waits for it.
    let remaining: Option<Child> = child.into_std_when_unreaped();
    let _sweeper = settle_locks(remaining, tree, locks, grace).await;
}

/// What:
///  Remove the locks of a stopped tree now when it has ended, or as a task of the runtime when it has not.
/// Why:
///   A killed process ends only when the kernel has run it once more, which a loaded or stalled host may not do within the
///  grace; a sweep before that finds the tree running and removes nothing, and nothing would try again. The task waits for
///  the whole tree and then sweeps. `child` is the stopped push when it was not collected, and nothing when it was. Returns the
///  task, if one was started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (child === undefined && allEnded(tree)) { locks.sweep(tree); return undefined; } return sweepAfterExit(child, tree, locks);
/// ```
pub async fn settle_locks(child: Option<Child>, tree: Vec<TreeMember>, locks: LockLedger, grace: Duration) -> Option<tokio::task::JoinHandle<()>> {
    let Some(uncollected) = child else {
        wait_for_tree(tree.as_slice(), grace).await;
        if !tree.iter().any(still_running) {
            // The sweep reads directories, so it runs on the blocking pool and not on the worker that polls tasks.
            let sweeping: Vec<TreeMember> = tree.clone();
            if let Err(error) = run_blocking_job(move || { locks.sweep(sweeping.as_slice()); }).await {
                debug(TAG, &format!("the sweep of a stopped push's locks did not run: {error}"));
            }
            return None;
        }
        // The push was collected and the tree is still running: the task waits for the tree alone.
        return sweep_task(None, tree, locks);
    };
    return sweep_task(Some(uncollected), tree, locks);
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

/// What a stop reports: the limit that was hit, what the push had moved and how long it had run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StopReport {
    /// The limit that was hit: the stall window.
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
}

/// What:
///  Look at a running read-only query until it ends, stalls or floods its output, and say how quiet it got.
/// Why:
///   `std` has no timed wait,
///  so the parent looks with `try_wait`,
///  which never blocks; the clock reads the signs of work a few times a second. A query that works is waited for however
///  long it takes: it has no wall-clock bound (owner, 2026-11-03: "Stop only on no progress"; the coordinator's scope addition
///  to picks 4 and 9 of `doc/planning/cli-git-shipped-limits-audit.md`, open to the owner's veto), and on a system that counts
///  nothing it is never judged stalled. `quiet` receives the longest quiet gap seen, for the ledger, and nothing where the system counts nothing.
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
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let mut waits: Vec<WaitReport> = Vec::new();
    let clock: Clock = Clock::start(child.id(), patience).without_wall_fallback();
    return wait_with_clock(child, outputs, clock, "git", &mut waits, quiet).await;
}

/// What:
///  The same wait as `wait_until_done` for a push, handing over the wait of a push that is waiting for its remote with the
///  connection open.
/// Why:
///   The push is not stopped while it waits; the caller says so once, in a line, and the wait ends the way any other does.
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
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let clock: Clock = Clock::start(child.id(), patience);
    let mut unused: Option<Duration> = None;
    return wait_with_clock(child, outputs, clock, label, waits, &mut unused).await;
}

/// The loop of both waits, over a clock the caller made.
async fn wait_with_clock(
    child: &mut OwnedChild,
    outputs: &[&File],
    mut clock: Clock,
    label: &str,
    waits: &mut Vec<WaitReport>,
    quiet: &mut Option<Duration>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    // Nothing is kept where the clock reads nothing: its figure would be the elapsed time, not a quiet gap.
    loop {
        // The exit is awaited on the child's pidfd; the pause is the look at the output sizes and the clock, a few times a second.
        if let Ok(waited) = tokio::time::timeout(POLL_PAUSE, child.wait()).await {
            let status: ExitStatus = waited.map_err(SingleFlightError::Start)?;
            *quiet = quiet_gap_kept(&clock);
            return Ok(Ok(status));
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
///  The name a run goes by in the lines about it: the program's file name and its first word that is not an option, as in
///  `git push`, `git fetch` or `git-lfs push`.
/// Why:
///   The wait line must say which of the wrapper's network runs is waiting, and the command is the only thing that knows. The
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
///   The person or the journal reads that the run is not stuck: the connection to the remote is open, and what ends the wait.
///  No time ends it: only the remote's answer, the connection closing or TCP giving up.
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
///  Run a push command to its end or its deadline and capture its interleaved output, after the reference locks that a
///  killed Git left in its repository have been cleared when their holder is proven gone.
/// Why:
///   Every start of the wrapper's own fetch and push, the repetitions of a push included, goes through here
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
///  A stopped push becomes a failed attempt that says so,
///  so the commit stays saved and the failure is recorded for later invocations.
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
///  `run_bounded_push_once` as a task of the runtime: every wait in it is awaited, and no worker is held while the push runs.
/// Why:
///   The synchronous face enters the runtime until the callers are asynchronous themselves; from inside the runtime that face
///  refuses, and this one is called.
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
    let started: Child = spawn_retrying_without_blocking(&mut command).await.map_err(SingleFlightError::Start)?;
    drop(command);
    let mut child: OwnedChild = adopt(started);
    let mut waits: Vec<WaitReport> = Vec::new();
    let ended: Result<ExitStatus, StopCause> = wait_until_done_noting(&mut child, &[&output], patience, label.as_str(), &mut waits).await?;
    if ended.is_err() {
        stop_push(child, grace, locks).await;
    }
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
                StopCause::Stalled(report) => format!(
                    "monochromatic-git-policy-cli: git push made no progress for {} seconds and was stopped (limit=stall, bytes_moved={}, \
                     elapsed_seconds={}); {}. {PUSH_QUEUED_LINE} Hint: if you were pushing over a slow or \
                     offline connection, your commit is already saved; push again when the connection works.",
                    report.limit.as_secs(),
                    report.moved,
                    report.elapsed.as_secs(),
                    extend_clause()
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
