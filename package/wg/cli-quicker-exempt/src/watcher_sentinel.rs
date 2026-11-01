//! Reports the death of the application watcher.
//! The watcher cannot report its own death when `SIGKILL`,
//! the out-of-memory killer or `systemd-oomd` ends it,
//! and from then on neither the socket marking nor the process watch is enforced,
//! which is a failure after startup like any other.
//! A second small process,
//! the sentinel,
//! waits on the watcher through a process file descriptor
//! (a poll of the process start time where the kernel has none),
//! and when the watcher is gone it checks that `watch-stop` did not remove the watcher's state.
//! If the state still names the dead watcher,
//! the death was not asked for,
//! and the sentinel reports it to the journal and the desktop through the same reporter as every other failure.
//! Limit:
//! if the sentinel is killed first,
//! or both die together,
//! nothing reports it.

/// Persisted watcher identity.
use crate::application_watch_state::{read_watch_state, WatchProcess};
/// Facts about one process.
use crate::process_info::read_start_time;
/// Non-panicking log.
use crate::log_line::log_line;
/// Failure delivery.
use crate::runtime_report::RuntimeReporter;
/// The marker of a running sentinel.
use crate::sentinel_launch::write_marker;
/// The words of the policy.
use crate::update_report::{policy_failure, POLICY_PROCESS_WATCH};
/// Syscall failures.
use std::io;
/// Raw descriptor ownership.
use std::os::fd::{FromRawFd, OwnedFd};
/// Fake procfs roots in tests.
use std::path::Path;
/// Bounded waits.
use std::time::Duration;

/// The hidden command that runs a sentinel.
pub const SENTINEL_COMMAND: &str = "__sentinel";

/// Time the sentinel waits after the watcher is gone before it reads the state,
/// so a `watch-stop` that kills the watcher and then removes its state is not mistaken for a death.
pub const STOP_GRACE: Duration = Duration::from_millis(1500);

/// Pause between polls where no process file descriptor exists.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Failure kind of a watcher that died.
pub const WATCHER_DIED_KIND: &str = "watcher died";

/// What the state says about the dead watcher.
#[derive(Debug, Eq, PartialEq)]
pub enum Verdict {
    /// `watch-stop`,
    /// or a newer start,
    /// removed or replaced the state,
    /// so the end was asked for.
    Expected,
    /// The state still names the dead watcher.
    Unexpected,
    /// The state could not be read,
    /// so nobody can say whether the end was asked for.
    Unknown(
        /// Why it could not be read.
        String,
    ),
}

/// Where the state of the watcher is read from.
pub trait StateSource {
    /// Reads the state,
    /// `None` when there is none.
    fn read(&self) -> io::Result<Option<WatchProcess>>;
}

/// The state stored under a watcher key.
pub struct KeyedState<'a> {
    /// The watcher key.
    pub key: &'a str,
}

/// The stored state.
impl StateSource for KeyedState<'_> {
    /// Reads the state file of the key.
    fn read(&self) -> io::Result<Option<WatchProcess>> {
        return read_watch_state(self.key);
    }
}

/// Judges the end of the watcher from the state it left.
pub fn judge(state_source: &dyn StateSource, pid: i32, start_time: u64) -> Verdict {
    return match state_source.read() {
        Ok(Some(state)) if state.pid == pid && state.start_time == start_time => Verdict::Unexpected,
        Ok(_other) => Verdict::Expected,
        Err(error) => Verdict::Unknown(error.to_string()),
    };
}

/// Opens a process file descriptor for `pid`,
/// or returns `None` where the kernel has no `pidfd_open`.
fn open_pidfd(pid: i32) -> Option<OwnedFd> {
    // SAFETY: pidfd_open takes a pid and flags and returns a new descriptor or a negative value.
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
    if raw < 0 {
        return None;
    }
    // SAFETY: a successful pidfd_open returned a descriptor owned by nobody else.
    return Some(unsafe { OwnedFd::from_raw_fd(raw as libc::c_int) });
}

/// Waits until the process `pid` with `start_time` no longer exists.
/// A process file descriptor refers to the process itself,
/// so a reused pid is never mistaken for it;
/// where the kernel has none,
/// the start time in `proc_root` is polled instead.
pub fn wait_for_exit(proc_root: &Path, pid: i32, start_time: u64) -> io::Result<()> {
    if let Some(descriptor) = open_pidfd(pid) {
        // The process may have been replaced between the start and the open.
        if read_start_time(proc_root, pid) != Some(start_time) {
            return Ok(());
        }
        let mut poll_descriptor = libc::pollfd {
            fd: std::os::fd::AsRawFd::as_raw_fd(&descriptor),
            events: libc::POLLIN,
            revents: 0,
        };
        loop {
            // SAFETY: the descriptor structure is valid for the synchronous poll.
            let result = unsafe { libc::poll(&mut poll_descriptor, 1, -1) };
            if result > 0 {
                return Ok(());
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }
    return poll_for_exit(proc_root, pid, start_time, POLL_INTERVAL);
}

/// Waits by polling the start time of the process in `proc_root` every `interval`,
/// until the process is gone or replaced by another with the same pid.
pub fn poll_for_exit(proc_root: &Path, pid: i32, start_time: u64, interval: Duration) -> io::Result<()> {
    while read_start_time(proc_root, pid) == Some(start_time) {
        std::thread::sleep(interval);
    }
    return Ok(());
}

/// The report for a watcher that died.
pub fn death_report(pid: i32) -> String {
    return format!(
        "wg-quicker-exempt: POLICY NOT IN FORCE: socket marking of application and update cgroups, and {POLICY_PROCESS_WATCH}: \
the watcher (pid {pid}) is no longer running and was not stopped by watch-stop (it was killed, ran out of memory or crashed), \
so neither is enforced any more; run wg-quicker up again"
    );
}

/// The report for a watcher that ended while its state cannot be read.
pub fn unknown_report(pid: i32, why: &str) -> String {
    return policy_failure(&format!(
        "the watcher (pid {pid}) is no longer running and its state cannot be read ({why}), so it cannot be told whether watch-stop asked for that; if not, neither the socket marking nor this policy is enforced any more"
    ));
}

/// Runs a sentinel for the watcher `pid` with `start_time` until it ends,
/// reports an unasked end through `reporter`,
/// and waits for the notification to be sent before it returns.
pub fn run_sentinel(
    proc_root: &Path,
    pid: i32,
    start_time: u64,
    grace: Duration,
    state_source: &dyn StateSource,
    reporter: &mut RuntimeReporter,
) -> io::Result<Verdict> {
    wait_for_exit(proc_root, pid, start_time)?;
    std::thread::sleep(grace);
    let verdict = judge(state_source, pid, start_time);
    match &verdict {
        Verdict::Unexpected => {
            reporter.report_failure(WATCHER_DIED_KIND, &death_report(pid));
        }
        Verdict::Unknown(why) => {
            reporter.report_failure(WATCHER_DIED_KIND, &unknown_report(pid, why));
        }
        Verdict::Expected => {}
    }
    reporter.finish();
    return Ok(verdict);
}

/// Runs the hidden sentinel command for the watcher stored under `key`.
pub fn run_sentinel_command(key: &str, pid: i32, start_time: u64, uid: u32, gid: u32) -> io::Result<()> {
    // The marker tells the process that started this one that it got as far as running.
    // A failure to write it does not stop the sentinel, which can still report,
    // but the starter will then say it could not confirm.
    if let Err(error) = write_marker(key, pid) {
        log_line(&policy_failure(&format!("the sentinel could not leave its marker ({error}), so the start cannot confirm that it runs")));
    }
    let mut reporter = RuntimeReporter::new(uid, gid);
    run_sentinel(Path::new("/proc"), pid, start_time, STOP_GRACE, &KeyedState { key }, &mut reporter)?;
    return Ok(());
}
