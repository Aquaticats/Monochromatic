//! What:
//!  The manager-started launch route of the receipt flusher (owner decision 22, "Systemd unit + fallback"): a transient systemd user unit,
//!  started with `systemd-run --user --collect` and waited for until the start is confirmed, that runs the service program in its internal
//!  flusher mode (`--flush-lane <lane>`). `launch_routed` tries it first and falls back to the double-forked helper where the manager
//!  refuses, is absent or does not answer within its wait.
//! Why:
//!  A helper started by a double fork from a session scope can end with the session, leaving its receipt to the next owner's startup sweep;
//!  a unit the manager owns survives a logout and a service upgrade by containment. The start is confirmed before the client returns
//!  (no `--no-block`): a start nobody confirmed would leave the receipt unserved with no record. Each launch has its own unit name, so a
//!  refusal never means a flusher already runs (an unloaded-but-not-collected unit of the same name would look the same). The unit's process
//!  takes the flusher lock itself, as the forked helper does, so the argument that a receipt is covered by whichever flusher holds the lock
//!  is the same for both routes. The unit carries no `CPUQuota` or `MemoryMax`: it forces a few receipt files, and no measurement of a
//!  flusher's footprint exists to cite beside a number.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // launchRouted(lane, manager): try manager.start(lane) -> unit; on refusal launchFlusher(lane) (the fork), which refuses a threaded process
//! ```

use super::commit_flusher::{FlusherLaunchError, launch_flusher};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// The internal flag of the service program that runs the flusher on a lane and exits with its code.
pub const FLUSH_LANE_FLAG: &str = "--flush-lane";

/// The prefix of a flusher unit's name.
const UNIT_PREFIX: &str = "cli-git-flusher-";

/// Why the manager did not start the unit; the caller falls back to the fork.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagerRefused(pub String);

/// A way to have the user manager run the flusher for a lane.
pub trait ManagerRoute {
    /// Starts a unit that runs the flusher on `lane` and returns once the manager confirmed the start, with the unit's name.
    fn start(&mut self, lane: &Path) -> Result<String, ManagerRefused>;
}

/// A hash of the lane's path and a per-launch number, as 16 hexadecimal digits.
fn lane_digest(lane: &Path, nonce: u64) -> String {
    let mut state: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in lane.as_os_str().as_encoded_bytes().iter().copied().chain(nonce.to_le_bytes()) {
        state ^= u64::from(byte);
        state = state.wrapping_mul(0x0100_0000_01b3);
    }
    return format!("{:016x}", state);
}

/// A nonce for one launch: eight bytes from the operating system's random source, so two launches in one process, or in two processes on
/// the same clock tick, do not share a unit name. If the source cannot be read the clock and the process identity are mixed in (a collision
/// then makes the manager refuse the unit, and the caller falls back to the fork).
pub fn fresh_nonce() -> u64 {
    use std::io::Read;
    let mut bytes: [u8; 8] = [0; 8];
    if std::fs::File::open("/dev/urandom").and_then(|mut source| { return source.read_exact(&mut bytes); }).is_ok() {
        return u64::from_le_bytes(bytes);
    }
    #[cfg(feature = "test-seams")]
    let frozen: Option<u64> = { let value: u64 = FROZEN_CLOCK.load(std::sync::atomic::Ordering::SeqCst); (value != u64::MAX).then_some(value) };
    #[cfg(not(feature = "test-seams"))]
    let frozen: Option<u64> = None;
    let clock: u64 = frozen.unwrap_or_else(|| { return std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| { return elapsed.as_nanos() as u64; }); });
    return clock ^ (u64::from(std::process::id()) << 32);
}

/// The clock reading the nonce fallback uses while a test freezes it; `u64::MAX` is off.
#[cfg(feature = "test-seams")]
static FROZEN_CLOCK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);

/// Freezes the clock the nonce fallback reads (`None` thaws it): two launches in one clock tick, to show that the random source is what makes
/// their unit names differ.
#[cfg(feature = "test-seams")]
pub fn inject_frozen_clock(value: Option<u64>) {
    FROZEN_CLOCK.store(value.unwrap_or(u64::MAX), std::sync::atomic::Ordering::SeqCst);
}

/// What:
///  Opens `lane` as a lane directory and checks the directory that was opened: an absolute path whose last component is a directory and not
///  a symbolic link, owned by the account that runs the program and writable by no one else, holding a `receipt` directory that is a real
///  directory with the same ownership, and no `state`, `saved`, `inbox` or `decision` that is a link or a file. Nothing is created. The
///  returned descriptor is the lane: everything done afterwards through `pinned_path` is done to this directory, however the name is changed.
/// Why:
///  `cli-git-service --flush-lane <dir>` and `--lane <dir>` are reachable by anyone who can run the program. A wrong caller can name a
///  directory it can write and have `flusher.v2.lock` and `receipt-errors` created there and the regular files under its `receipt/` forced
///  (a read-only operation on file data); a link named like one of the lane's directories is refused, so nothing is read or removed
///  outside the lane through it. The ancestors of the lane are not checked: a directory above it that other accounts can rename into can
///  still change what a name leads to before this call, and cannot change the directory this call returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openLane(lane: string): Directory; // throws a reason
/// ```
pub fn open_lane(lane: &Path) -> Result<std::fs::File, String> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    if !lane.is_absolute() {
        return Err("the lane is not an absolute path".to_string());
    }
    let directory: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(lane)
        .map_err(|error| { return format!("the lane is not a directory (a symbolic link does not count): {}", error); })?;
    let pinned: PathBuf = pinned_path(&directory);
    // SAFETY: `geteuid` takes no argument and touches no memory.
    let me: u32 = unsafe { libc::geteuid() };
    // A lane belongs to the account that runs the program and can be written by no one else: the doorbell and the error record in it are
    // trusted only because of that. (Everything the program does to the lane is done with this account's rights.)
    let own = |label: &str, metadata: &std::fs::Metadata| -> Result<(), String> {
        if metadata.uid() != me {
            return Err(format!("{} belongs to another account", label));
        }
        if metadata.mode() & 0o022 != 0 {
            return Err(format!("{} can be written by its group or by others", label));
        }
        return Ok(());
    };
    own("the lane", &directory.metadata().map_err(|error| { return format!("the lane cannot be read: {}", error); })?)?;
    let receipt: std::fs::Metadata = std::fs::symlink_metadata(pinned.join(super::commit_flusher::RECEIPT_DIR))
        .map_err(|_error| { return "the directory has no receipt directory, so it is not a lane".to_string(); })?;
    if !receipt.file_type().is_dir() {
        return Err("the directory has no receipt directory, so it is not a lane".to_string());
    }
    own("the receipt directory", &receipt)?;
    for name in ["state", "saved", "inbox", "decision"] {
        match std::fs::symlink_metadata(pinned.join(name)) {
            Ok(metadata) if !metadata.file_type().is_dir() => return Err(format!("{} of the lane is not a directory of the lane (a link does not count)", name)),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{} of the lane cannot be read: {}", name, error)),
        }
    }
    return Ok(directory);
}

/// Checks `lane` as `open_lane` does and lets the descriptor go.
pub fn validate_lane(lane: &Path) -> Result<(), String> {
    return open_lane(lane).map(|_directory| {});
}

/// The path through which the opened lane directory is reached for as long as the descriptor is open, whatever its name becomes.
pub fn pinned_path(directory: &std::fs::File) -> PathBuf {
    use std::os::fd::AsRawFd;
    return PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()));
}

/// The unit name for one launch: the lane's digest and `nonce`, which differs per launch.
pub fn unit_name(lane: &Path, nonce: u64) -> String {
    return format!("{}{}", UNIT_PREFIX, lane_digest(lane, nonce));
}

/// The arguments of `systemd-run` that start the flusher unit: a user unit named `unit`, unloaded after it ran (`--collect`), the start
/// waited for, then the service program in flusher mode on `lane`.
pub fn transient_unit_arguments(unit: &str, service_program: &Path, lane: &Path) -> Vec<OsString> {
    // `Type=exec`: the start job completes when the service program has been executed, so a missing or unexecutable program is a refusal
    // and the caller falls back to the fork. What happens after the execution (a death before the sweep) is not confirmed by the manager:
    // the receipt stays in the directory for the next sweep, and the flusher records its own failures.
    let mut arguments: Vec<OsString> = vec!["--user".into(), "--collect".into(), "--quiet".into(), "--property=Type=exec".into(), format!("--unit={}", unit).into(), "--".into()];
    arguments.push(service_program.as_os_str().to_os_string());
    arguments.push(FLUSH_LANE_FLAG.into());
    arguments.push(lane.as_os_str().to_os_string());
    return arguments;
}

/// What:
///  Runs `command` with its streams closed and waits at most `wait` for it: a zero exit is `Ok`; a non-zero exit, a program that cannot be
///  started, and a command still running at the bound (which is killed and reaped) are refusals.
/// Why:
///  The manager's answer is a bounded wait of the client; it ends at `wait` whatever the manager does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runWithin(command: Command, wait: number): ExitStatus; // throws ManagerRefused
/// ```
pub fn run_within(mut command: Command, wait: Duration) -> Result<ExitStatus, ManagerRefused> {
    let mut child = command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|error| { return ManagerRefused(format!("the manager's command could not be started: {}", error)); })?;
    let deadline: Instant = Instant::now() + wait;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(status),
            Ok(Some(status)) => return Err(ManagerRefused(format!("the manager's command ended with {}", status))),
            Ok(None) => {}
            Err(error) => return Err(ManagerRefused(format!("the manager's command could not be waited for: {}", error))),
        }
        if Instant::now() >= deadline {
            // The kill is attempted and never waited for without a bound. A child that has not died within the window goes to the pending list,
            // which later launches poll without waiting, so it is reaped once it does end.
            let killed: bool = child.kill().is_ok();
            let reap_until: Instant = Instant::now() + Duration::from_millis(100);
            while killed && Instant::now() < reap_until && matches!(child.try_wait(), Ok(None)) {
                std::thread::sleep(Duration::from_millis(1));
            }
            #[cfg(feature = "test-seams")]
            let unreaped_by_force: bool = UNREAPABLE.load(std::sync::atomic::Ordering::SeqCst);
            #[cfg(not(feature = "test-seams"))]
            let unreaped_by_force: bool = false;
            if unreaped_by_force || !matches!(child.try_wait(), Ok(Some(_))) {
                // No thread waits for it: the caller may have to fork next, and a fork needs a process with one thread.
                PENDING_REAP.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); }).push(child);
            }
            return Err(ManagerRefused(format!("the manager did not confirm the start within {:?}", wait)));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Children that were killed after a timeout and had not been reaped within the window; `reap_pending` polls them.
static PENDING_REAP: std::sync::Mutex<Vec<std::process::Child>> = std::sync::Mutex::new(Vec::new());

/// Makes the cleanup window of `run_within` treat the killed child as not yet reaped (a test cannot make a process survive SIGKILL).
#[cfg(feature = "test-seams")]
static UNREAPABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Test seam: `true` makes `run_within` leave its killed child to the pending list whatever the window saw.
#[cfg(feature = "test-seams")]
pub fn inject_unreapable_child(on: bool) {
    UNREAPABLE.store(on, std::sync::atomic::Ordering::SeqCst);
}

/// Test seam: puts a live child in the pending list as a command that was killed and did not end would be.
#[cfg(feature = "test-seams")]
pub fn inject_pending_child(child: std::process::Child) {
    PENDING_REAP.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); }).push(child);
}

/// How many killed children wait to be reaped.
pub fn pending_reaps() -> usize {
    return PENDING_REAP.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); }).len();
}

/// Reaps the pending children that have ended, without waiting for any, and keeps the rest in the list. No thread is started: the caller
/// may have to fork next (the fork needs a process with one thread), and a child that never ends would otherwise keep a waiter thread alive
/// across every later launch. A child that has not ended stays until a later call finds it ended.
pub fn reap_pending() {
    let mut pending = PENDING_REAP.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); });
    pending.retain_mut(|child| {
        return match child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(error) => {
                crate::service_error!("a killed manager command could not be polled and is dropped from the list: {error}");
                false
            }
        };
    });
}

/// The real route: `systemd-run --user` with the service program.
#[derive(Clone, Debug)]
pub struct TransientUnitRoute {
    systemd_run: PathBuf,
    service_program: PathBuf,
    /// The longest the client waits for the manager's confirmation of the start.
    pub wait: Duration,
    launches: u64,
}

impl TransientUnitRoute {
    /// A route that runs `systemd_run` and starts `service_program`; the wait is the service start ceiling (measured: the slowest of 52
    /// cold starts took 1,041 ms, `src/native/service/wait_ceilings.rs`).
    pub fn new(systemd_run: PathBuf, service_program: PathBuf) -> TransientUnitRoute {
        return TransientUnitRoute { systemd_run, service_program, wait: crate::wait_ceilings::START_CEILING, launches: 0 };
    }
}

impl ManagerRoute for TransientUnitRoute {
    fn start(&mut self, lane: &Path) -> Result<String, ManagerRefused> {
        self.launches += 1;
        let unit: String = unit_name(lane, fresh_nonce());
        let mut command: Command = Command::new(&self.systemd_run);
        command.args(transient_unit_arguments(&unit, &self.service_program, lane));
        run_within(command, self.wait)?;
        return Ok(unit);
    }
}

/// How a flusher was started.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Routed {
    /// The manager started this unit.
    Manager {
        /// The unit's name.
        unit: String,
    },
    /// The manager refused and a double-forked helper was started.
    Fork,
}

/// What:
///  Starts the flusher for `lane`: through `manager` first, and by the double fork where the manager refuses.
/// Why:
///  Decision 22. The fork keeps its own requirement (a process with one thread, `LaunchError::Threaded` otherwise, with every receipt
///  recorded); the manager route needs none, since it forks nothing in this process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function launchRouted(lane: string, manager: ManagerRoute): Routed; // throws FlusherLaunchError
/// ```
pub fn launch_routed(lane: &Path, manager: &mut dyn ManagerRoute) -> Result<Routed, FlusherLaunchError> {
    // Children an earlier launch left that have ended are reaped before this one, and polled again after it.
    reap_pending();
    let routed: Result<Routed, FlusherLaunchError> = match manager.start(lane) {
        Ok(unit) => Ok(Routed::Manager { unit }),
        Err(_refused) => launch_flusher(lane).map(|_started| { return Routed::Fork; }),
    };
    // The command of this launch may have timed out: its child is polled without waiting.
    reap_pending();
    return routed;
}
