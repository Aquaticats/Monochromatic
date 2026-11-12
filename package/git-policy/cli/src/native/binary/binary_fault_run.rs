//! What:
//!  Runs the wrapper itself,
//! with no `timeout` in front of it,
//! in a process group of its own that a watchdog ends.
//! Why:
//!  `timeout` forwards an `INT`,
//! `QUIT`,
//! `HUP` or `TERM` it receives to its child and to its whole process group,
//! so a
//!  control that signals the `timeout` it started delivers the signal to Git and the editor directly and proves nothing about
//!  the wrapper's own relay (`signal_relay.rs`).
//! Here the process ID that is signalled is the wrapper's,
//! nothing else
//!  receives the signal unless the wrapper sends it,
//! and a limit that must hold for the wrapper's children (a process limit
//!  would stop `timeout` from starting the wrapper at all) is set on the wrapper itself.
//! The hang guard is a thread that
//!  ends the group when the command's process tree shows no sign of work,
//! so a stuck wrapper fails the control instead of holding the run,
//! and a slow one is left alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const wrapper = spawn(gitLink, args, { detached: true }); const guard = new Judge(QUIET_WINDOW).observe(() => process.kill(-wrapper.pid, 'SIGKILL'));
//! process.kill(wrapper.pid, 'SIGTERM'); // the wrapper alone
//! ```

use super::supervisor::{Judge, Limits, Reason};
use super::supervisor_proc::{Look, look_at, stat_of, tick};
use super::support::{Fixture, ServiceLaunch, hermetic, read_from_start, restart_ended_services, service_runtime, wrapper_first_path};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// How long the processes a killed wrapper left behind may take to end by themselves,
/// in seconds.
pub(super) const STRAGGLER_SECONDS: u64 = 30;

/// How long a watched wrapper's process tree may show no sign of work before its group is ended:
/// a hang guard of the control,
/// never part of a verdict.
/// Why not a wall clock:
///  a traced commit runs under a `ptrace` of every process it starts,
/// and on a loaded host one such run took 121 to 300 s of wall time while it was moving all the time,
/// so a clock of 120 s ended eighteen `fault_trace::` controls that had nothing wrong with them
/// (the suite of 2026-11-10,
/// `${HOME}/temp/agent/plk/suite2.log`).
/// A run that is stuck shows nothing instead,
/// which is the evidence the whole-run watchdog reads
/// (`testing/test_support.rs`,
/// `QUIET_WINDOW`,
/// whose 120 s is the longest quiet stretch a working run ever showed:
/// 8.0 s),
/// and a run that moves for ever is ended by the test runner's own per-test limit
/// (`.config/nextest.toml`).
const QUIET_WINDOW: Duration = Duration::from_secs(120);

/// The limits a watched wrapper's tree is judged by:
/// only the quiet window,
/// because a traced run moves bytes and processor time in the `strace` that carries it,
/// and its whole duration is the disk's,
/// never a loop's.
const LIMITS: Limits = Limits { no_progress: QUIET_WINDOW, cpu_budget: Duration::MAX, backstop: Duration::MAX };

/// The wrapper (the fixture's `bin/git`) with the fixture's environment,
/// run in `directory`,
/// and no `timeout`.
pub(super) fn unbounded_wrapper(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Command {
    let mut command: Command = Command::new(fixture.root.join("bin/git"));
    hermetic(&mut command, fixture, wrapper_first_path(fixture).as_os_str());
    if let Some(runtime) = service_runtime(fixture) {
        command.env("XDG_RUNTIME_DIR", runtime);
    }
    command.current_dir(directory).args(arguments);
    return command;
}

/// A started command,
/// its process group and the watchdog that ends the group.
pub(super) struct Watched {
    /// The started wrapper,
    /// the leader of its group.
    child: Child,
    /// What the command printed on standard output,
    /// in a file:
    /// a program the wrapper leaves behind (an editor,
    /// a pager) that
    ///  keeps a pipe open would otherwise hold the wait for the wrapper's output long after the wrapper ended.
    stdout: std::fs::File,
    /// What the command printed on standard error,
    /// in a file,
    /// for the same reason.
    stderr: std::fs::File,
    /// Set when the command was waited for,
    /// so the watchdog stops.
    finished: Arc<AtomicBool>,
    /// The reason the watchdog ended the group,
    /// and nothing while it has not.
    expired: Arc<Mutex<Option<Reason>>>,
    /// When the wrapper started,
    /// in clock ticks after boot,
    /// which with the process ID names the one process a look reads.
    started: u64,
    /// The process group of the wrapper,
    /// which is its own process ID.
    group: i32,
    /// How long the command's tree may show nothing before the group is ended.
    window: Duration,
}

impl Watched {
    /// The process ID of the wrapper,
    /// which is also the ID of its group.
    pub(super) fn id(&self) -> i32 {
        return i32::try_from(self.child.id()).expect("a process ID fits");
    }

    /// Waits until `marker` exists,
    /// which a stage of the command writes when it is reached.
    ///  # Panics
    ///  When the wrapper ends first,
    ///  naming what it printed,
    ///  or when its tree shows no sign of work before the marker appears.
    pub(super) fn wait_for(&mut self, marker: &Path) {
        let began: Instant = Instant::now();
        let mut judge: Judge = Judge::new(Limits { no_progress: self.window, ..LIMITS }, tick());
        while !marker.exists() {
            if let Some(status) = self.child.try_wait().expect("the wrapper's state") {
                let said: String = String::from_utf8_lossy(read_from_start(&mut self.stderr).as_slice()).into_owned();
                panic!("the wrapper ended with {status:?} before {} appeared: {said}", marker.display());
            }
            if let Look::Seen(sample) = look_at(self.group, self.started)
                && let Some(reason) = judge.observe(began.elapsed(), &sample, false)
            {
                panic!("{} never appeared: the wrapper was ended because it {reason}", marker.display());
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Waits for the command and returns what it printed and how it ended,
    /// after whatever the wrapper left running in its group
    ///  (the Git it started when it was killed) has ended by itself,
    /// as it does in production;
    /// a straggler is ended after
    ///  `STRAGGLER_SECONDS`.
    ///  # Panics
    ///  When the watchdog had to end the group,
    ///  which names a hang.
    pub(super) fn finish_after_stragglers(self) -> Output {
        let (output, ended): (Output, Option<Reason>) = self.wait_and_end(true);
        if let Some(reason) = ended {
            panic!("the wrapper's group was ended because it {reason}");
        }
        return output;
    }

    /// Waits for the command and returns what it printed and how it ended;
    /// whatever it left running in its group ends with it.
    ///  # Panics
    ///  When the watchdog had to end the group,
    ///  which names a hang.
    pub(super) fn finish(self) -> Output {
        let (output, ended): (Output, Option<Reason>) = self.wait_and_end(false);
        if let Some(reason) = ended {
            panic!("the wrapper's group was ended because it {reason}");
        }
        return output;
    }

    /// What the command printed,
    /// how it ended,
    /// and the reason the guard ended its group when it did,
    /// so a control of the guard itself reads the judgement instead of catching a panic.
    pub(super) fn finish_reporting(self) -> (Output, Option<Reason>) {
        return self.wait_and_end(false);
    }

    /// The end of a run:
    /// the wrapper is waited for (which reaps it,
    /// so that a zombie leader does not keep the group alive),
    /// the
    ///  group is given time to end by itself when `let_stragglers_end`,
    /// then ended.
    fn wait_and_end(self, let_stragglers_end: bool) -> (Output, Option<Reason>) {
        let Watched { mut child, mut stdout, mut stderr, finished, expired, .. } = self;
        let status: std::process::ExitStatus = child.wait().expect("the wrapper ends");
        finished.store(true, Ordering::SeqCst);
        let group: i32 = i32::try_from(child.id()).expect("a process ID fits");
        if let_stragglers_end {
            let began: Instant = Instant::now();
            // SAFETY: `kill` with signal 0 only checks that the group has a member.
            while unsafe { libc::kill(-group, 0) } == 0 && began.elapsed() < Duration::from_secs(STRAGGLER_SECONDS) {
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        // Whatever the wrapper left running in its group (an editor that was never told to stop) ends with it.
        // SAFETY: `kill` takes two integers and touches no memory of this process.
        unsafe { libc::kill(-group, libc::SIGKILL) };
        let output: Output = Output { status, stdout: read_from_start(&mut stdout), stderr: read_from_start(&mut stderr) };
        let ended: Option<Reason> = *expired.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        return (output, ended);
    }
}

/// Starts `command` as the leader of a new process group,
/// with a watchdog,
/// and standard output and error in files under the
///  fixture.
pub(super) fn spawn_watched(fixture: &Fixture, command: &mut Command) -> Watched {
    return spawn_watched_with(fixture, command, QUIET_WINDOW);
}

/// `spawn_watched` with the quiet window given,
/// so a control of the guard itself states a window it reaches in a moment.
pub(super) fn spawn_watched_with(fixture: &Fixture, command: &mut Command, window: Duration) -> Watched {
    let stdout: std::fs::File = tempfile::tempfile_in(&fixture.root).expect("a file for standard output");
    let stderr: std::fs::File = tempfile::tempfile_in(&fixture.root).expect("a file for standard error");
    let child: Child = command
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone().expect("standard output file")))
        .stderr(Stdio::from(stderr.try_clone().expect("standard error file")))
        .spawn()
        .expect("the wrapper starts");
    let group: i32 = i32::try_from(child.id()).expect("a process ID fits");
    let started: u64 = stat_of(group).expect("the wrapper that just started to be running").started;
    let finished: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let expired: Arc<Mutex<Option<Reason>>> = Arc::new(Mutex::new(None));
    let watching: (Arc<AtomicBool>, Arc<Mutex<Option<Reason>>>) = (Arc::clone(&finished), Arc::clone(&expired));
    std::thread::spawn(move || {
        let (done, reason): (Arc<AtomicBool>, Arc<Mutex<Option<Reason>>>) = watching;
        let began: Instant = Instant::now();
        let mut judge: Judge = Judge::new(Limits { no_progress: window, ..LIMITS }, tick());
        while !done.load(Ordering::SeqCst) {
            std::thread::sleep(crate::supervisor::POLL);
            // A wrapper that has exited is waited for by the control, and its work is over whatever its group still holds.
            let Look::Seen(sample) = look_at(group, started) else {
                continue;
            };
            if let Some(ending) = judge.observe(began.elapsed(), &sample, false) {
                *reason.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(ending);
                // SAFETY: `kill` takes two integers and touches no memory of this process.
                unsafe { libc::kill(-group, libc::SIGKILL) };
                return;
            }
        }
    });
    return Watched { child, stdout, stderr, finished, expired, started, group, window };
}

/// Runs `command` to its end under the watchdog.
pub(super) fn run_watched(fixture: &Fixture, command: &mut Command) -> Output {
    return spawn_watched(fixture, command).finish();
}

/// Sends `signal` to one process (a positive target) or to a process group (a negative one).
///  # Panics
///  When the signal cannot be sent.
pub(super) fn send(target: i32, signal: i32) {
    // SAFETY: `kill` takes two integers and touches no memory of this process.
    let status: libc::c_int = unsafe { libc::kill(target, signal) };
    assert_eq!(status, 0, "kill({target}, {signal}) failed: {}", std::io::Error::last_os_error());
}

/// Whether a process with this ID still exists (a zombie that nothing reaped counts as gone:
/// it runs nothing).
pub(super) fn is_running(pid: i32) -> bool {
    let Ok(status) = std::fs::read_to_string(format!("/proc/{pid}/status")) else {
        return false;
    };
    return !status.lines().any(|line| { return line.starts_with("State:") && line.contains('Z'); });
}

/// Runs one wrapped command to its end and reports what it showed its caller,
/// without waiting for the service's after-commit work
///  (see `verify_recovery_with`).
pub(super) fn run_unsettled(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> super::support::Observed {
    let output: Output = run_watched(fixture, &mut unbounded_wrapper(fixture, directory, arguments));
    return super::support::Observed { code: output.status.code(), stdout: output.stdout, stderr: output.stderr };
}

/// Sets a flag when dropped,
/// so a thread that polls it ends however its owner ends.
struct EndsOnDrop<'flag>(&'flag AtomicBool);

impl Drop for EndsOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

/// How often the stand-in for the user manager looks for an ended service.
const MANAGER_POLL: Duration = Duration::from_millis(5);

/// Runs `body` while a stand-in for the user manager starts again any service of the fixture that ends,
/// as production does at the
///  next connection to the manager's socket.
pub(super) fn with_manager<T>(fixture: &Fixture, body: impl FnOnce() -> T) -> T {
    let finished: AtomicBool = AtomicBool::new(false);
    return std::thread::scope(|scope| {
        scope.spawn(|| {
            while !finished.load(Ordering::SeqCst) {
                restart_ended_services(fixture);
                std::thread::sleep(MANAGER_POLL);
            }
        });
        let _ends = EndsOnDrop(&finished);
        return body();
    });
}

/// How many cases of a sweep run at once:
/// enough to finish a sweep in seconds,
/// few enough to leave the host its other work.
pub(super) const SWEEP_WIDTH: usize = 6;

/// Runs `check` over every item,
/// `SWEEP_WIDTH` at a time,
/// and returns the messages of the items that failed.
pub(super) fn in_batches<T: Sync>(items: &[T], check: &(dyn Fn(&T) -> Result<(), String> + Sync)) -> Vec<String> {
    let failures: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    for batch in items.chunks(SWEEP_WIDTH) {
        std::thread::scope(|scope| {
            for item in batch {
                let collected: &std::sync::Mutex<Vec<String>> = &failures;
                scope.spawn(move || {
                    if let Err(message) = check(item) {
                        collected.lock().expect("failure list").push(message);
                    }
                });
            }
        });
    }
    return failures.into_inner().expect("failure list");
}

/// What a failure of a fault control says:
/// the message it was given and what the fixture's services logged.
/// Why:
///   A service that refuses a step or ends says so only on its own standard error,
/// and a sweep runs several cases at once,
/// so the line that settles one failure is mixed with the other cases';
/// the log the fixture keeps says which service said what
/// (the recorded failure of 2026-11-07 could be settled only by such a line).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = `${message}\n${serviceLog(fixture)}`;
/// ```
pub(super) fn with_service_log(fixture: &Fixture, message: String) -> String {
    return format!("{message}\n{}", super::support::service_log(fixture.root.as_path()));
}

/// The verdict of a fault case,
/// with the fixture's service log carried on a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = raw.ok ? raw : { error: withServiceLog(fixture, raw.error) };
/// ```
pub(super) fn keeping_the_service_log(fixture: &Fixture, verdict: Result<(), String>) -> Result<(), String> {
    return match verdict {
        Ok(()) => Ok(()),
        Err(message) => Err(with_service_log(fixture, message)),
    };
}

/// The pressure ends:
/// the running services are stopped,
/// and the next one starts as in production,
/// with no extra limit and
///  no tracer.
pub(super) fn end_the_pressure(fixture: &Fixture) {
    if let Some(services) = fixture.service.lock().expect("the fixture's service lock").as_ref() {
        services.stop_all();
    }
    *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch::default();
}
