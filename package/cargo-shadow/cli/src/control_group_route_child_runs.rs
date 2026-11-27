//! What:
//! The short programs route discovery runs:
//! one program started with no standard input,
//! read under one deadline that also covers the killing and the reaping,
//! with the text a failure adds about the killed program's reap,
//! and the check of whether a group directory can be killed through `cgroup.kill`.
//! Why:
//! The question "can this host give a delegated scope" is put to the manager by running it,
//! and a question must not wait without end:
//! a reaping with a budget of its own would end after the promised time,
//! so one deadline covers reading,
//! killing and reaping,
//! and only a killed child is given the grace that lets its own reap answer.
//! Split out of `control_group_route_discovery.rs`;
//! `kill_file_present` is re-exported from there,
//! so its path is unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control_group_route_child_runs.ts: runPlain(program, args, env, bound), runAsking(...), endChild, reapWithin, reapNote, killFilePresent
//! ```

/// What:
/// `use crate::control_group_route_discovery::DISCOVERY_POLL;` imports the interval at which a killed program is asked whether it ended.
/// Why:
/// Both waits of this file poll at that interval,
/// so the discovery has one pacing value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DISCOVERY_POLL } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::DISCOVERY_POLL;
/// What:
/// `use crate::lazy_error_text::ExplainedError;` imports the trait that turns a failure into a message with a stated cause.
/// Why:
/// A program the discovery cannot start is reported with the reason it was started for,
/// not as a bare I/O code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a throw new Error(`${why}: ${cause.message}`) at the call site
/// ```
use crate::lazy_error_text::ExplainedError;
/// What:
/// `use std::fs;` imports the standard library's file-system module so this file can name it unqualified.
/// Why:
/// Whether a group directory can be killed is read from the presence of one file in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { existsSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// A failing wait on the started child is an `io::Error`,
/// and a test injects one through the failpoints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;
/// What:
/// `use std::os::fd::OwnedFd;` imports the owning descriptor type (its siblings are `BorrowedFd`,
/// a view,
/// and `RawFd`,
/// the bare number).
/// Why:
/// The two output pipes are handed to the readers that own them until the deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node closes a stream descriptor when the stream is closed
/// ```
use std::os::fd::OwnedFd;
/// What:
/// `use std::path::Path;` imports the borrowed path type from the standard library so this file can name it unqualified.
/// Why:
/// The group directory of the `cgroup.kill` check is only read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;
/// What:
/// `use std::process::{Command, Stdio};` imports the command builder and the choice of what a child's standard streams are.
/// Why:
/// The question is one short command,
/// and its standard input is the null device so it can never wait for input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';   // stdio: ['ignore', 'pipe', 'pipe']
/// ```
use std::process::{
    Command,
    Stdio,
};
/// What:
/// `use std::sync::Arc;` imports the atomically counted pointer type (its sibling `Rc` counts without atomics,
/// for one thread only).
/// Why:
/// The cancel flag is shared with the two reader threads,
/// so the count must be atomic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // shared object references in TS need no count
/// ```
use std::sync::Arc;
/// What:
/// `use std::sync::atomic::{AtomicBool, Ordering};` imports a boolean that threads may set and read.
/// Why:
/// The readers stop when this flag is set,
/// and the flag is set on every exit of the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let cancelled = false; // single-threaded in Node
/// ```
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the monotonic clock (sibling of `Instant`:
/// `SystemTime`,
/// the wall clock,
/// which can jump).
/// Why:
/// One deadline,
/// read from the monotonic clock,
/// covers the reading,
/// the killing and the reaping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() and a number of milliseconds
/// ```
use std::time::{
    Duration,
    Instant,
};
/// How long a killed short program is given to become reapable before its reap attempt is read as "still running".
/// Why:
/// `kill` sends a signal and returns;
/// the child becomes reapable a moment later,
/// so an attempt made immediately after the kill races with the scheduler and mostly answers that the child is still
/// running.
/// What the value rests on:
/// the wait is `reap_within`,
/// which asks `try_wait` once every `DISCOVERY_POLL` (5 ms),
/// so a killed program is noticed at most one poll after it ends,
/// and this grace is twenty of those polls.
/// Measured 2026-11-28 on this host through that wait,
/// five kills of `sleep 30` in a scope capped like the machine's rules require:
/// the median wait was 5.069852 ms and the longest 5.079702 ms;
/// with the same scope throttled to a `CPUQuota` of 2% the median was 5.079712 ms and the longest 5.145821 ms.
/// The grace is the most a reap attempt adds past the deadline it serves.
/// A program that outlasts it is not waited for again:
/// `std::process::Child` has no destructor that waits,
/// so the killed child stays a zombie of the shadow until the shadow exits,
/// and a second wait would be another bounded grace on a path a hung manager already delays.
pub(crate) const KILL_REAP_GRACE: Duration = Duration::from_millis(100);

/// What:
/// `pub(crate) struct Answered` is what a short program answered:
/// how it ended and the two streams it wrote.
/// Why:
/// A caller that has to read the exit status itself,
/// as the step that ends a run's scope unit does,
/// cannot be given only the standard output of a program that succeeded.
pub(crate) struct Answered {
    /// How the program ended.
    pub(crate) status: std::process::ExitStatus,
    /// What it wrote on standard output.
    pub(crate) out: Vec<u8>,
    /// What it wrote on standard error.
    pub(crate) err: Vec<u8>,
}

/// What:
/// `fn run_plain(program, arguments, environment, bound) -> Result<String, String>` runs a short program under a deadline and
/// returns its standard output,
/// or the reason it failed.
/// Why:
/// The question to the manager must not wait without end (contract V6);
/// the program is not a run of the user's,
/// so it needs no containment.
/// A caller that has to see the exit status,
/// rather than only whether the program succeeded,
/// asks `run_asking` instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runPlain(program: string, args: string[], env: [string, string | undefined][]): string  // throws the reason
/// ```
pub(crate) fn run_plain(
    program: &str,
    arguments: &[&str],
    environment: &[(String, Option<String>)],
    bound: Duration,
) -> Result<String, String> {
    let answered: Answered = run_asking(program, arguments, environment, bound)?;
    if !answered.status.success() {
        let message: String = String::from_utf8_lossy(&answered.err).to_string();
        return Err(format!(
            "{program} failed ({})",
            message.lines().next().unwrap_or("no message").trim()
        ));
    }
    return Ok(String::from_utf8_lossy(&answered.out).to_string());
}

/// What:
/// `fn run_asking(program, arguments, environment, bound) -> Result<Answered, String>` runs a short program under a deadline and
/// returns how it ended and what it wrote,
/// or the reason it could not be read.
/// Why:
/// An answer the caller has to judge itself (a stop request the manager refuses as "not loaded") is a status and two streams,
/// and it is the same one-deadline run as `run_plain`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runAsking(program: string, args: string[], env: [string, string | undefined][]): {status, out, err}
/// ```
pub(crate) fn run_asking(
    program: &str,
    arguments: &[&str],
    environment: &[(String, Option<String>)],
    bound: Duration,
) -> Result<Answered, String> {
    let mut command: Command = Command::new(program);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in environment {
        match value {
            Some(set) => command.env(name, set),
            None => command.env_remove(name),
        };
    }
    let mut child = command
        .spawn()
        .explained(format_args!("{program} could not be started"))?;
    // One deadline for the reading, the killing and the reaping (contract V6; round 10, 9a D3).
    let deadline: Instant = Instant::now() + bound;
    let cancel: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let (Some(out_pipe), Some(err_pipe)) = (child.stdout.take(), child.stderr.take()) else {
        let _ = child.kill();
        let note: String = reap_note(reap_within(
            &mut child,
            deadline.max(Instant::now() + KILL_REAP_GRACE),
        ));
        return Err(format!("{program} has no output pipes{note}"));
    };
    let out_reader = crate::bounded_io::collect(OwnedFd::from(out_pipe), Arc::clone(&cancel));
    let err_reader = crate::bounded_io::collect(OwnedFd::from(err_pipe), Arc::clone(&cancel));
    // Every exit from here ends the readers, kills a child that is still running, and reaps it within what is left of the deadline.
    // It returns the text that says so when the child was not reaped in time.
    let status = loop {
        match child.try_wait() {
            Ok(Some(found)) => break found,
            Ok(None) => {}
            Err(error) => {
                let note: String = end_child(&mut child, &cancel, deadline);
                return Err(format!("{program} could not be waited for ({error}){note}"));
            }
        };
        if Instant::now() >= deadline {
            let note: String = end_child(&mut child, &cancel, deadline);
            return Err(format!("{program} did not answer within {bound:?}{note}"));
        }
        std::thread::sleep(DISCOVERY_POLL);
    };
    // The direct child ended; a descendant may still hold the pipes, so the draining has the same deadline.
    let out_wait: Duration = deadline.saturating_duration_since(Instant::now());
    let out: Vec<u8> = match out_reader.recv_timeout(out_wait) {
        Ok(Ok(bytes)) => bytes,
        _ => {
            let note: String = end_child(&mut child, &cancel, deadline);
            return Err(format!(
                "{program} did not answer within {bound:?} (its output was not closed){note}"
            ));
        }
    };
    let err_wait: Duration = deadline.saturating_duration_since(Instant::now());
    let err: Vec<u8> = match err_reader.recv_timeout(err_wait) {
        Ok(Ok(bytes)) => bytes,
        _ => {
            let note: String = end_child(&mut child, &cancel, deadline);
            return Err(format!(
                "{program} did not answer within {bound:?} (its error output was not closed){note}"
            ));
        }
    };
    cancel.store(true, Ordering::Relaxed);
    return Ok(Answered {
        status,
        out,
        err,
    });
}

/// What:
/// `fn end_child(running: &mut std::process::Child, readers: &Arc<AtomicBool>, deadline: Instant) -> String` ends the readers,
/// kills a child that is still running,
/// reaps it within what is left of the deadline,
/// and answers the text that says what became of it.
/// Why:
/// Every exit from the wait ends the child this way,
/// and a function replaces the closure that captured the deadline.
/// The text names what the kill and the reap answered (issue:
/// the note said a dead line had passed when it had not),
/// and a killed child is reaped before the note is built,
/// so the note speaks of one that was still running rather than one that was never waited for.
pub(crate) fn end_child(
    running: &mut std::process::Child,
    readers: &Arc<AtomicBool>,
    deadline: Instant,
) -> String {
    readers.store(true, Ordering::Relaxed);
    let _ = running.kill();
    return reap_note(reap_within(
        running,
        deadline.max(Instant::now() + KILL_REAP_GRACE),
    ));
}

/// What:
/// `pub(crate) enum Reap` is what the attempts to reap a killed program answered.
/// Why:
/// The note a failure carries has to say which of three things happened:
/// the program was reaped,
/// it was still running when the grace after its kill ran out,
/// or the wait itself failed.
pub(crate) enum Reap {
    /// The program was reaped.
    Reaped,
    /// The program had not ended when the grace after its kill ran out.
    StillRunning,
    /// The wait itself failed,
    /// so the program was not reaped.
    Failed {
        /// The failure the wait answered.
        error: io::Error,
    },
}

/// What:
/// `fn reap_within(child: &mut std::process::Child, deadline: Instant) -> Reap` waits for a killed child until `deadline` and
/// says what the waiting answered.
/// Why:
/// `Child::wait` has no deadline (contract V6),
/// and the discovery's one deadline covers its reading,
/// its killing and its reaping:
/// a reaping with a budget of its own would end after the promised time (round 10,
/// 9a D3).
/// A wait that fails is not a reap and ends the waiting,
/// because a broken child handle does not mend by asking again;
/// an interrupted wait is asked again,
/// because the kernel's signal is not the child's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reapWithin(child: ChildProcess, deadline: number): 'reaped' | 'still running' | Error
/// ```
pub(crate) fn reap_within(
    child: &mut std::process::Child,
    deadline: Instant,
) -> Reap {
    // A test makes the killed child look unreapable for as long as the budget lasts.
    #[cfg(test)]
    let stuck: bool = crate::failpoints::fires(crate::failpoints::Point::ReapStuck);
    #[cfg(not(test))]
    let stuck: bool = false;
    #[cfg(test)]
    let injected_error: bool = crate::failpoints::fires(crate::failpoints::Point::ReapError);
    #[cfg(not(test))]
    let injected_error: bool = false;
    loop {
        let waited: io::Result<Option<std::process::ExitStatus>> = if injected_error {
            Err(io::Error::other("injected wait failure"))
        } else {
            child.try_wait()
        };
        match waited {
            Ok(Some(_)) if !stuck => return Reap::Reaped,
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => {
                return Reap::Failed {
                    error,
                };
            }
        }
        let now: Instant = Instant::now();
        if now >= deadline {
            return Reap::StillRunning;
        }
        std::thread::sleep(DISCOVERY_POLL.min(deadline - now));
    }
}

/// What:
/// `fn reap_note(answered: Reap) -> String` is the text a failure adds about a killed program,
/// or nothing when it was reaped.
/// Why:
/// The failure is conservative:
/// a program that may still exist is reported as one that may,
/// and a person who reads it is told what was tried and what came back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reapNote(answered: 'reaped' | 'still running' | Error): string
/// ```
fn reap_note(answered: Reap) -> String {
    return match answered {
        Reap::Reaped => String::new(),
        Reap::StillRunning => format!(
            "; the program was killed and could not be reaped (it had not ended within {KILL_REAP_GRACE:?} of the kill, and nothing waits for it again)"
        ),
        Reap::Failed {
            error,
        } => format!(
            "; the program was killed and could not be reaped (the wait after it failed: {error})"
        ),
    };
}

/// What:
/// `pub(crate) fn kill_file_present(directory: &Path) -> bool` says whether a group directory has `cgroup.kill`.
/// Why:
/// The file exists since Linux 5.14;
/// a kernel without it takes the weaker path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function killFilePresent(directory: string): boolean
/// ```
pub(crate) fn kill_file_present(directory: &Path) -> bool {
    return fs::metadata(directory.join("cgroup.kill")).is_ok();
}

/// The controls of the grace a killed short program is given.
#[cfg(test)]
#[path = "control_group_route_child_runs_tests.rs"]
mod tests;
