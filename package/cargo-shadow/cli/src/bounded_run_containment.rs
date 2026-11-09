//! What:
//! How a run is contained and stopped:
//! the choice between a control group scope and a process group,
//! the start of the child inside it,
//! and the stop that shows nothing of the run is left.
//! Why:
//! Split out of `bounded_run.rs`,
//! which runs a child process within a time bound,
//! to keep that file under the code-line limit.
//! The items are re-exported from `bounded_run`,
//! so their paths there are unchanged.

/// The trace text,
/// the notice gate and the start hook that exist in a test build only.
#[cfg(test)]
use crate::bounded_run::{NOTICE_ENTERED, NOTICE_GATE, START_HOOK, trace_text};
/// The bounds,
/// the scope state,
/// the error and the helpers of the bounded run that the containment uses.
use crate::bounded_run::{Bound, QUIESCE_BOUND, POLL_INTERVAL, READER_GRACE, REAP_BOUND, RunError, ScopeState, WEAKER_NOTICE_GIVEN, append_record, append_record_with, configure_stdio, find_route, trace};

/// Imports `std::io` for this file.
use std::io;


/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `std::os::unix::process::CommandExt` for this file.
use std::os::unix::process::CommandExt;

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports `std::process::{Child, Command, ExitStatus, Output, Stdio}` for this file.
use std::process::{Child, Command, ExitStatus};

/// Imports `std::rc::Rc` for this file.
use std::rc::Rc;

/// Imports `std::sync::Arc` for this file.
use std::sync::Arc;

/// Imports `std::sync::atomic::{AtomicBool, Ordering}` for this file.
use std::sync::atomic::{AtomicBool, Ordering};

/// Imports `std::sync::mpsc::{Receiver, RecvTimeoutError}` for this file.
use std::sync::mpsc::{Receiver, RecvTimeoutError};

/// Imports `std::thread` for this file.
use std::thread;

/// Imports `std::time::{Duration, Instant}` for this file.
use std::time::{Duration, Instant};


/// Imports `rustix::event::{PollFd, PollFlags, Timespec, poll}` for this file.
use rustix::event::{PollFd, PollFlags, Timespec, poll};

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// Imports `rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group}` for this file.
use rustix::process::{Pid, PidfdFlags, Signal, kill_process_group, pidfd_open, test_kill_process_group};


/// Imports the control group pieces for this file.
use crate::control_group::{ControlGroup, GroupUnavailable, Record, Route};

/// What:
/// `fn reap(child: &mut Child) -> Option<ExitStatus>` waits for the direct child,
/// a few milliseconds at a time,
/// up to `REAP_BOUND`.
/// Why:
/// `Child::wait` has no deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reap(child: ChildProcess): ExitStatus | undefined
/// ```
pub(crate) fn reap(child: &mut Child) -> Option<ExitStatus> {
    // A child that cannot be reaped within its bound is "not reaped"; a test makes that so without a process in an uninterruptible wait.
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::ReapStuck) {
        return None;
    }
    let deadline: Instant = Instant::now() + REAP_BOUND;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_) => return None,
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// What:
/// `fn wait_for_exit(watch: Option<&OwnedFd>, remaining: Duration)` waits until the child ends or `remaining` passes.
/// Why:
/// A pidfd wakes at the exit;
/// without one (a kernel before 5.3) the wait is a sleep of `POLL_INTERVAL`.
/// Measured on 2026-11-04 (debug build,
/// 40 runs of `true`):
/// the sleep-polling run took 11.3 ms at best,
/// almost all of it the poll interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitForExit(watch: Descriptor | undefined, remainingMs: number): void
/// ```
pub(crate) fn wait_for_exit(watch: Option<&OwnedFd>, remaining: Duration) {
    let Some(descriptor) = watch else {
        thread::sleep(POLL_INTERVAL.min(remaining));
        return;
    };
    let timeout: Timespec = Timespec { tv_sec: i64::try_from(remaining.as_secs()).unwrap_or(0), tv_nsec: i64::from(remaining.subsec_nanos()) };
    let mut watched = [PollFd::new(descriptor, PollFlags::IN)];
    // An interrupted or failed poll only means the next turn of the loop asks `try_wait` again.
    let _ = poll(&mut watched, Some(&timeout));
}

/// How a run is contained.
pub(crate) enum Containment {
    /// In a transient scope the manager made:
    /// quiescence is shown by the kernel.
    Scope {
        /// The scope of this run.
        group: ControlGroup,
        /// The route it was planned on.
        route: Route,
    },
    /// In a process group of its own (the child's process id is the group):
    /// the weaker path.
    ProcessGroup,
}

/// What:
/// `pub(crate) fn weaker_notice(unavailable: &GroupUnavailable) -> String` is the one-line notice of the weaker path.
/// Why:
/// It names the missing capability and says what cannot be shown,
/// so a person knows the guarantee they have (contract,
/// "The weaker path").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function weakerNotice(unavailable: GroupUnavailable): string
/// ```
pub(crate) fn weaker_notice(unavailable: &GroupUnavailable) -> String {
    return format!("cargo-shadow: no delegated control group ({unavailable}); a process that leaves the process group cannot be stopped or found, so quiescence is not shown and a recovery of this run waits for `cargo-shadow acknowledge`");
}

/// What:
/// `fn contain() -> Containment` plans a scope,
/// or says in one line which capability is missing and falls back to a process group.
/// Why:
/// The notice is printed once per process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contain(): Containment
/// ```
pub(crate) fn contain() -> Containment {
    return match find_route() {
        Ok(route) => match ControlGroup::plan(&route) {
            Ok(group) => Containment::Scope { group, route },
            Err(error) => weaker(GroupUnavailable { reason: format!("a scope name cannot be made ({error})") }),
        },
        Err(unavailable) => weaker(unavailable),
    };
}

/// What:
/// `fn weaker(unavailable: GroupUnavailable) -> Containment` gives the notice that the run is contained by a process group only,
/// and returns that containment.
/// Why:
/// A function replaces the closure that `contain` called at both refusals.
pub(crate) fn weaker(unavailable: GroupUnavailable) -> Containment {
    // The notice this run is given is part of the trace, so a test sees the notice of the run and not a text it formats itself.
    // The text is computed, so this statement exists in a test build only.
    #[cfg(test)]
    trace_text(format!("notice {}", weaker_notice(&unavailable)));
    if !WEAKER_NOTICE_GIVEN.swap(true, std::sync::atomic::Ordering::SeqCst) {
        #[cfg(test)]
        {
            NOTICE_ENTERED.store(true, std::sync::atomic::Ordering::SeqCst);
            drop(NOTICE_GATE.lock());
        }
        eprintln!("{}", weaker_notice(&unavailable));
    }
    return Containment::ProcessGroup;
}

/// What:
/// `fn stop_process_group(child: &mut Child, group: Pid) -> bool` kills the process group,
/// reaps the direct child,
/// and returns `true` once no member of the group is left.
/// Why:
/// This is all the weaker path can show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopProcessGroup(child: ChildProcess, group: number): boolean
/// ```
pub(crate) fn stop_process_group(child: &mut Child, group: Pid) -> bool {
    let reaped: bool = {
        let _ = kill_process_group(group, Signal::KILL);
        reap(child).is_some()
    };
    let deadline: Instant = Instant::now() + QUIESCE_BOUND;
    loop {
        if test_kill_process_group(group) == Err(Errno::SRCH) {
            return reaped;
        }
        if Instant::now() >= deadline {
            return false;
        }
        let _ = kill_process_group(group, Signal::KILL);
        thread::sleep(POLL_INTERVAL);
    }
}

/// What:
/// `fn stop_everything(containment, child) -> bool` ends the run early or at its normal exit:
/// it kills and reaps the direct child,
/// stops what the containment holds,
/// and returns `true` only if all of it is shown gone.
/// Why:
/// This is the one way a run is ended,
/// on timeout,
/// on a normal exit and on every error path.
/// The direct child goes first:
/// it is the process that becomes the program,
/// so once it is reaped nothing of this program's can
/// still start the program in a scope that is not there yet.
/// A scope that cannot be stopped,
/// and any failure to inspect or stop it,
/// is "not shown gone".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopEverything(containment: Containment, child: ChildProcess): boolean
/// ```
pub(crate) fn stop_everything(containment: &Containment, child: &mut Child) -> bool {
    return match containment {
        Containment::Scope { group, .. } => {
            let _ = child.kill();
            let reaped: bool = reap(child).is_some();
            let stopped: bool = match group.stop(QUIESCE_BOUND) {
                Ok(done) => done,
                Err(error) => {
                    eprintln!("cargo-shadow: warning: the scope {} could not be shown empty ({error})", group.directory().display());
                    false
                }
            };
            reaped && stopped
        }
        Containment::ProcessGroup => match Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)) {
            Some(group) => stop_process_group(child, group),
            None => false,
        },
    };
}

/// What:
/// `fn settle(containment: &Containment, scope: &Option<Rc<ScopeState>>)` records that the run's launcher is reaped and its scope
/// stopped,
/// so a recovery may read an absent scope as one that ended.
/// Why:
/// Until then an absent scope may be one the launcher has not made yet (round 9,
/// 8a F1).
/// A record that cannot be written leaves the journal unsettled,
/// which the next recovery treats as uncertainty to acknowledge:
/// the safe side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settle(containment: Containment, scope: ScopeState | undefined): void
/// ```
pub(crate) fn settle(containment: &Containment, scope: &Option<Rc<ScopeState>>) {
    if let (Containment::Scope { group, .. }, Some(state)) = (containment, scope)
        && let Err(error) = append_record(&state.log, &Record::Settled { path: group.directory().to_path_buf() })
    {
        eprintln!("cargo-shadow: warning: the end of the run in {} could not be recorded ({error}); a recovery will ask for an acknowledgement", group.directory().display());
    }
}

/// What:
/// `fn fail_started(containment, child, scope, bound, error) -> RunError` ends a run that failed after its program started.
/// Why:
/// Any failure on the containment path after the start is "not quiescent" unless everything is shown gone (contract A2):
/// it must not become an ordinary failure that lets the caller roll back or accept under a writer that may remain (7a F3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failStarted(containment: Containment, child: ChildProcess, scope: ScopeState | undefined, bound: Bound, error: RunError): RunError
/// ```
pub(crate) fn fail_started(containment: &Containment, child: &mut Child, scope: &Option<Rc<ScopeState>>, bound: &Bound, error: RunError) -> RunError {
    if stop_everything(containment, child) {
        settle(containment, scope);
        return error;
    }
    lose(scope);
    return RunError::NotQuiescent { what: bound.what.clone() };
}

/// What:
/// `fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError>` takes the result of an output worker.
/// Why:
/// A worker that does not report means the output is not complete,
/// and a prefix must never be returned as the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finished(receiver: Promise<Buffer>, what: string): Buffer  // throws RunError
/// ```
pub(crate) fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError> {
    return match receiver.recv_timeout(READER_GRACE) {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(error)) => Err(RunError::Io(error)),
        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => Err(RunError::IncompleteOutput { what: what.to_string() }),
    };
}

/// Sets the cancel flag when the run ends,
/// however it ends,
/// so no worker outlives the call.
pub(crate) struct CancelOnDrop(
    /// The flag the pipe workers look at.
    pub(crate) Arc<AtomicBool>,
);

/// How the guard acts.
impl Drop for CancelOnDrop {
    /// What:
    /// `fn drop(&mut self)` tells the pipe workers to stop.
    /// Why:
    /// A worker must not keep a thread,
    /// a descriptor and a growing buffer after the call returned.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// What:
/// `fn lose(scope: &Option<Rc<ScopeState>>)` marks the open transaction as one whose writers may remain.
/// Why:
/// The caller of the transaction reads the mark and stops the command instead of rolling back or finishing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lose(scope: ScopeState | undefined): void
/// ```
pub(crate) fn lose(scope: &Option<Rc<ScopeState>>) {
    if let Some(state) = scope {
        state.lost.set(true);
    }
}

/// What:
/// `fn program_resolves(command: &Command) -> bool` says whether the program of the command exists as an executable file,
/// searched on the `PATH` the command will have when its name has no slash.
/// Why:
/// `systemd-run` reports a program it cannot start as an exit status like any other,
/// so a program that does not exist is found before the launcher is started and reported as the I/O error it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function programResolves(command: Command): boolean
/// ```
pub(crate) fn program_resolves(command: &Command) -> bool {
    let program: &std::ffi::OsStr = command.get_program();
    let relative_to: PathBuf = command.get_current_dir().map(Path::to_path_buf).unwrap_or_default();
    if program.as_encoded_bytes().contains(&b'/') {
        return crate::control_group::is_executable_file(&relative_to.join(program));
    }
    let mut own_path: Option<std::ffi::OsString> = None;
    for (name, value) in command.get_envs() {
        if name == "PATH" {
            own_path = value.map(std::ffi::OsStr::to_os_string);
            break;
        }
    }
    if own_path.is_none() {
        own_path = std::env::var_os("PATH");
    }
    if let Some(search) = own_path {
        for directory in std::env::split_paths(&search) {
            if crate::control_group::is_executable_file(&relative_to.join(directory).join(program)) {
                return true;
            }
        }
    }
    return false;
}

/// What:
/// `fn run_start_hook()` runs the hook a test put inside the start of a run,
/// if any.
/// Why:
/// It is the boundary at which a test overlaps other work with a start,
/// or kills a wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runStartHook(): void
/// ```
#[cfg(test)]
pub(crate) fn run_start_hook() {
    if let Some(hook) = START_HOOK.with(|slot| { return slot.borrow().clone(); }) {
        hook();
    }
}

/// The production build has no hook.
#[cfg(not(test))]
pub(crate) fn run_start_hook() {}

/// What:
/// `fn note_launcher(log: &Path, scope: &Path, child: &Child)` appends the launcher record:
/// the process id,
/// the start time and the
/// process id namespace of the process that is about to become the program.
/// Why:
/// A recovery that finds this process gone knows it can never join the scope,
/// so it can read the scope as final without an
/// acknowledgement (round 11).
/// A line that cannot be written is only a missing proof:
/// the journal then waits for an acknowledgement like an older journal,
/// so the
/// run goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteLauncher(log: string, scope: string, child: ChildProcess): void
/// ```
pub(crate) fn note_launcher(log: &Path, scope: &Path, child: &Child) {
    let pid: i32 = i32::try_from(child.id()).unwrap_or(0);
    let noted: io::Result<()> = match crate::control_group::process_start(pid) {
        Ok(start) => record_launcher(log, scope, pid, start),
        Err(error) => Err(error),
    };
    if let Err(error) = noted {
        eprintln!("cargo-shadow: warning: the launcher of {} could not be recorded ({error}); if this wrapper is killed, a recovery will ask for an acknowledgement", scope.display());
    }
}

/// What:
/// `fn reason_of(unavailable: &GroupUnavailable) -> String` clones the reason of a refusal.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
pub(crate) fn reason_of(unavailable: &GroupUnavailable) -> String {
    return unavailable.reason.clone();
}

/// What:
/// `fn open_pidfd(pid: Pid) -> Option<OwnedFd>` opens a descriptor that becomes readable when the process ends,
/// or `None` when the kernel refuses.
/// Why:
/// `Option::and_then` takes it by path instead of a closure.
pub(crate) fn open_pidfd(pid: Pid) -> Option<OwnedFd> {
    return pidfd_open(pid, PidfdFlags::empty()).ok();
}

/// What:
/// `fn record_launcher(log: &Path, scope: &Path, pid: i32, start: u64) -> io::Result<()>` appends the launcher record.
/// Why:
/// A function replaces the closure that `note_launcher` passed to `and_then`.
pub(crate) fn record_launcher(log: &Path, scope: &Path, pid: i32, start: u64) -> io::Result<()> {
    let pid_namespace: u64 = crate::control_group::pid_namespace_id()?;
    return append_record_with(log, &Record::Launcher { pid, start, pid_namespace, path: scope.to_path_buf() }, false);
}

/// What:
/// `fn scope_record_failed(error: io::Error) -> RunError` is the refusal for a scope record that could not be written.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
pub(crate) fn scope_record_failed(error: io::Error) -> RunError {
    return RunError::Io(io::Error::new(error.kind(), format!("{error} (a program whose scope cannot be recorded is not started)")));
}

/// What:
/// `fn group_announcement_failed(error: io::Error) -> RunError` is the refusal for a process-group announcement that could not be written.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
pub(crate) fn group_announcement_failed(error: io::Error) -> RunError {
    return RunError::Io(io::Error::new(error.kind(), format!("{error} (a child whose process group cannot be announced is not started)")));
}

/// What:
/// `fn start_contained(command, containment, scope, has_input, clean_environment) -> Result<Child, RunError>` records the
/// containment in the journal and starts the child inside it.
/// Why:
/// For a scope the record is forced to disk before the program starts,
/// and a record that cannot be written means the program is not started.
/// For the weaker path the process group is only known once the child exists,
/// so a `pgroup starting` line is forced to disk first and the group's line follows the start;
/// a child whose group cannot be recorded is stopped at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startContained(command: Command, containment: Containment, scope: ScopeState | undefined, hasInput: boolean, cleanEnvironment: boolean): ChildProcess  // throws RunError
/// ```
pub(crate) fn start_contained(command: &mut Command, containment: &Containment, scope: Option<&Rc<ScopeState>>, has_input: bool, clean_environment: bool) -> Result<Child, RunError> {
    match containment {
        Containment::Scope { group, route } => {
            if !program_resolves(command) {
                return Err(RunError::Io(io::Error::new(io::ErrorKind::NotFound, format!("{:?} was not found (os error 2)", command.get_program()))));
            }
            if let Some(state) = scope {
                append_record(&state.log, &group.record()).map_err(scope_record_failed)?;
            }
            run_start_hook();
            let mut launch: Command = group.launcher(route, command, clean_environment);
            configure_stdio(&mut launch, has_input);
            // The text computes a file name, so this statement exists in a test build only.
            #[cfg(test)]
            trace_text(format!("launch scope {}", group.directory().file_name().map_or_else(String::new, |name| { return name.to_string_lossy().into_owned(); })));
            let child: Child = launch.spawn().map_err(RunError::Io)?;
            // The direct child of this process, as the kernel numbered it: a test compares it with what the program says it is.
            trace(format_args!("spawned {}", child.id()));
            crate::bounded_run::run_after_launch_hook(child.id());
            if let Some(state) = scope {
                note_launcher(&state.log, group.directory(), &child);
            }
            return Ok(child);
        }
        Containment::ProcessGroup => {
            if let Some(state) = scope {
                state.process_group_only.set(true);
            }
            command.process_group(0);
            // The child's group is its process id, known only after the spawn: a "starting" line is durable before it, so a
            // wrapper killed in between is seen by a recovery as one that may have a child running.
            if let Some(state) = scope {
                append_record(&state.log, &Record::ProcessGroupStarting).map_err(group_announcement_failed)?;
            }
            run_start_hook();
            trace(format_args!("launch pgroup"));
            let mut child: Child = command.spawn().map_err(RunError::Io)?;
            trace(format_args!("spawned {}", child.id()));
            let leader: i32 = i32::try_from(child.id()).unwrap_or(0);
            if let Some(state) = scope
                && let Err(error) = append_record(&state.log, &Record::ProcessGroup(leader))
            {
                let stopped: bool = match Pid::from_raw(leader) {
                    Some(group) => stop_process_group(&mut child, group),
                    None => false,
                };
                if !stopped {
                    lose(&Some(Rc::clone(state)));
                }
                return Err(RunError::Io(io::Error::new(error.kind(), format!("{error} (the child was stopped because its process group could not be recorded)"))));
            }
            return Ok(child);
        }
    }
}
