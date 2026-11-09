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
/// The deadline runner shared with the push,
/// and its stop causes.
use crate::bounded_push::{MAX_PUSH_OUTPUT, StopCause, flooded, stop_push, wait_until_done_up_to};
use crate::push_deadline_and_patience_settings::{Patience, push_deadline};
/// The clock a wait is judged by,
/// and its reading of the signs of work.
use crate::push_progress::Clock;
/// How long the writer of a second-pass command's input waits while the command takes none of it.
const FEEDING_PAUSE: Duration = std::time::Duration::from_millis(10);

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
/// The pipe's own descriptor,
/// for a write that must not wait in a full pipe.
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
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
pub fn query_clock(pid: u32, window: Duration) -> Clock {
    return Clock::start(pid, Patience::Progress { stall: window });
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
    return match run_on_runtime(run_bounded_git_awaiting(command, arguments, window, None, grace, environment, None, MAX_PUSH_OUTPUT)) {
        Ok(output) => output,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(refusal.to_string()))),
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
    return match run_on_runtime(run_bounded_git_awaiting(command, arguments, window, ceiling, grace, environment, input, cap)) {
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
    return run_bounded_git_awaiting(command, arguments, window, None, grace, environment, None, MAX_PUSH_OUTPUT).await;
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
    mut command: Command,
    arguments: &str,
    window: Duration,
    ceiling: Option<Duration>,
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
    let writer: Option<InputWriter> = match input {
        None => None,
        Some(bytes) => match feed_standard_input(started.stdin.take(), bytes, window) {
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
        Some(total) => match tokio::time::timeout(total, wait_until_done_up_to(&mut child, &[&stdout, &stderr], clock, cap, &mut quiet, stop.as_ref())).await {
            Ok(outcome) => outcome?,
            Err(_passed) => {
                stop_push(child, grace, locks).await;
                return Err(SingleFlightError::Ceiling {
                    arguments: arguments.to_owned(),
                    seconds: total.as_secs(),
                });
            }
        },
        None => wait_until_done_up_to(&mut child, &[&stdout, &stderr], clock, cap, &mut quiet, stop.as_ref()).await?,
    };
    let step: String = format!("git {}", arguments.split_whitespace().next().unwrap_or_default());
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
    //  an answer made from the request.
    if ended.is_ok()
        && let Some(reason) = writer.as_ref().and_then(InputWriter::cut_short)
    {
        return Err(SingleFlightError::Git {
            arguments: arguments.to_owned(),
            code: None,
            stderr: format!("the request of this query could not be written whole: {reason}"),
        });
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
                StopCause::Flood => flood_of(arguments, cap),
            })
        }
    };
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
fn flood_of(arguments: &str, cap: u64) -> SingleFlightError {
    return SingleFlightError::Git {
        arguments: arguments.to_owned(),
        code: None,
        stderr: format!("the query wrote more than {cap} bytes, so its output is not taken as Git's answer"),
    };
}

/// What:
///  Write `bytes` to the standard input of a started command from a thread of its own.
/// Why:
///  The command may answer before it has read all of its input,
/// so writing here could block on a full pipe while nothing reads the answer;
/// the thread's write fails once the command is stopped and its pipe closes.
///
/// # Errors
///
/// A thread that cannot be started.
fn feed_standard_input(pipe: Option<ChildStdin>, bytes: Vec<u8>, window: Duration) -> Result<InputWriter, String> {
    let progress: InputProgress = InputProgress::default();
    let reported: InputProgress = progress.clone();
    reported.started(bytes.len());
    let written: Result<std::thread::JoinHandle<()>, std::io::Error> = match pipe {
        // No input was asked of the command, so there is nothing to write and its standard input is not piped.
        None => crate::scoped_start::spawn_named("cli-git-second-pass-input", nothing_to_write),
        Some(open) => crate::scoped_start::spawn_named("cli-git-second-pass-input", move || {
            write_within_the_window(open, bytes.as_slice(), window, reported);
        }),
    };
    return match written {
        Ok(handle) => Ok(InputWriter { handle, progress }),
        Err(error) => Err(format!("no thread could be started to give the command its input: {error}")),
    };
}

/// What:
///  The thread that gives a command its input,
/// and what it has to report about it.
/// Why:
///   A request that could not be written whole leaves a command reading a prefix,
/// so the query must learn that from the
///  writer and not pass the answer on as one made from the whole request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InputWriter = { join(): Promise<void>; failure(): string | undefined };
/// ```
pub(crate) struct InputWriter {
    /// The thread that writes the input,
    /// and closes the command's end when it ends.
    /// A query waits for its child and lets this thread end by itself;
    /// a control joins it to see that it did.
    #[cfg_attr(not(test), expect(dead_code, reason = "only a control joins the writer; a query waits for its child"))]
    handle: std::thread::JoinHandle<()>,
    /// What the thread has written of the request,
    /// and why it stopped short when it did.
    progress: InputProgress,
}

/// What a writer has written of its request,
/// and why it stopped short when it did,
/// shared with the thread that writes it.
/// A cell behind a mutex is enough:
/// one thread writes it and one reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Progress = { written: number; total: number; failure?: string };
/// ```
#[derive(Clone, Default)]
pub(crate) struct InputProgress(std::sync::Arc<std::sync::Mutex<Progress>>);

/// How far a writer got.
#[derive(Clone, Default)]
struct Progress {
    /// The bytes handed to the kernel so far.
    written: usize,
    /// The bytes the request holds.
    total: usize,
    /// Why the writer stopped before the end,
    /// once it knows of it.
    failure: Option<String>,
}

impl InputProgress {
    /// The request this writer was given,
    /// so a reader can count from the start.
    fn started(&self, total: usize) {
        if let Ok(mut slot) = self.0.lock() {
            slot.total = total;
        }
    }

    /// Records the bytes handed to the kernel so far.
    fn advanced(&self, written: usize) {
        if let Ok(mut slot) = self.0.lock() {
            slot.written = written;
        }
    }

    /// Records why the request could not be written whole.
    fn record(&self, reason: String) {
        if let Ok(mut slot) = self.0.lock() {
            slot.failure = Some(reason);
        }
    }

    /// What the request could not be written whole for,
    /// or nothing when every byte reached the command.
    /// Why:
    ///   A command that ended while bytes remain cannot have read them,
    /// so the answer was made from a prefix of the
    ///  request even where the writer had not yet noticed the broken pipe.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// progress.failure ?? (progress.written < progress.total ? `after ${progress.written} of ${progress.total} bytes` : undefined)
    /// ```
    fn cut_short(&self) -> Option<String> {
        let Ok(slot) = self.0.lock() else {
            return Some(String::from("how much of its request was written could not be read"));
        };
        if let Some(reason) = slot.failure.as_ref() {
            return Some(reason.clone());
        }
        if slot.written >= slot.total {
            return None;
        }
        return Some(format!(
            "it stopped taking its input after {} of {} bytes, so {} of them were not written",
            slot.written,
            slot.total,
            slot.total.saturating_sub(slot.written)
        ));
    }
}

impl InputWriter {
    /// Waits for the writer to end.
    #[cfg(test)]
    pub(crate) fn join(self) -> std::thread::Result<()> {
        return self.handle.join();
    }

    /// Why the request could not be written whole,
    /// or nothing when every byte reached the command.
    pub(crate) fn cut_short(&self) -> Option<String> {
        return self.progress.cut_short();
    }
}


/// A step for the thread a command with no input gets:
/// it writes nothing and ends,
/// which closes the writing end it was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// () => {}
/// ```
fn nothing_to_write() {}

/// What:
///  Writes `bytes` into `open` while the command keeps taking them,
/// and gives up when it takes none for `window`.
/// Why:
///   The command may answer before it has read all of its input,
/// so a blocking write can wait on a full pipe for as long as
///  the command holds its end open,
/// and a command the kernel does not release would keep that thread for good.
/// The pipe's
///  writing end is made non-blocking,
/// so the thread sees a full pipe instead of waiting in it,
/// and it holds the
///  request only until nothing has been taken for the window the query itself is judged by;
/// a request of ours that a
///  stopped command never read has nothing left to say.
///  The writing end is closed when this returns,
/// so the command reads the end of its input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setNonBlocking(fd); while (written < bytes.length) { const n = tryWrite(fd, bytes.subarray(written)); if (n > 0) { written += n; last = Date.now() } else if (Date.now() - last > windowMs) return; else await sleep(POLL) }
/// ```
fn write_within_the_window(mut open: ChildStdin, bytes: &[u8], window: Duration, progress: InputProgress) {
    // The writing end is the only place that waits: making it non-blocking turns a full pipe into a report of it, and the
    //  command's own end is a different description of the same pipe.
    let descriptor: std::os::fd::RawFd = open.as_fd().as_raw_fd();
    // SAFETY: `fcntl` reads and writes an integer of the descriptor's own flags and touches no memory of this process.
    let flags: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        crate::diagnostic_log::debug("monochromatic-git-policy-cli", "the input of a second-pass command could not be written without blocking");
        progress.record(String::from("its input could not be written without blocking"));
        let _closed = open.flush();
        return;
    }
    let mut written: usize = 0;
    let mut last_taken: std::time::Instant = std::time::Instant::now();
    while written < bytes.len() {
        match open.write(&bytes[written..]) {
            // A write of nothing with bytes left is a request the command will not take; the count says how far it got.
            Ok(0) => break,
            Ok(taken) => {
                written += taken;
                progress.advanced(written);
                last_taken = std::time::Instant::now();
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if last_taken.elapsed() >= window {
                    let reason: String = format!(
                        "it took none of its input for {} seconds, so {} of {} bytes were not written",
                        window.as_secs(),
                        bytes.len().saturating_sub(written),
                        bytes.len()
                    );
                    crate::diagnostic_log::debug("monochromatic-git-policy-cli", format!("a second-pass command {reason}").as_str());
                    progress.record(reason);
                    break;
                }
                std::thread::sleep(FEEDING_PAUSE);
            }
            // A command that closed its end before every byte was written answered from a prefix of the request: the loop runs
            //  only while bytes remain, so nothing here is "what it wanted".
            Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {
                let reason: String = format!(
                    "it closed its input after {} of {} bytes, so {} of them were not written",
                    written,
                    bytes.len(),
                    bytes.len().saturating_sub(written)
                );
                crate::diagnostic_log::debug("monochromatic-git-policy-cli", format!("a second-pass command {reason}").as_str());
                progress.record(reason);
                break;
            }
            Err(error) => {
                let reason: String = format!("its input could not be written: {error}");
                crate::diagnostic_log::debug("monochromatic-git-policy-cli", format!("a second-pass command {reason}").as_str());
                progress.record(reason);
                break;
            }
        }
    }
    // Closing our end tells the command that its input has ended.
    let _closed = open.flush();
}

/// Query windows are controlled in the test module.
#[cfg(test)]
#[path = "bounded_query_tests.rs"]
mod tests;
