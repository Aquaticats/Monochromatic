//! The lock that keeps a scope's population check and the manager's stop of that scope
//! in one exclusion.
//! The manager ends every process in a scope it stops,
//! and a watcher cannot make "this origin scope holds nothing" and "stop it" one step:
//! the population is read from the cgroup filesystem,
//! the stop is a call to the user's manager,
//! and a process can join the scope between the two.
//! Two watchers of one user share the scopes of each origin,
//! so the process that joins can be another watcher's adoption.
//! Both sides therefore hold one lock file per watched user:
//! the sweep that stops origin scopes across its recheck and its stop,
//! and every adoption across its manager call and its confirmation.
//! A holder covers one scope check or one move,
//! never a scan or a whole sweep,
//! and a wait is bounded:
//! an adoption that cannot take the lock at once leaves the process for the next scan,
//! and the sweep leaves the scopes for the next watcher start,
//! because a watcher that blocked on the other side's slow manager call would stall the policy loop.
//! The kernel drops a lock whose holder died,
//! so a killed watcher never holds one.

/// The lock file and its descriptor.
use std::fs::File;
/// Whether another holder has the lock.
use std::io;
/// The descriptor's kernel lock.
use std::os::fd::AsRawFd;
/// The mode the gate file is created with.
use std::os::unix::fs::OpenOptionsExt;
/// Where the gate files live.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::{Duration, Instant};

/// Directory the production gate files live in.
pub const PRODUCTION_GATE_ROOT: &str = "/run";

/// Mode of a gate file:
/// only the watcher,
/// which runs as root,
/// takes it.
const GATE_FILE_MODE: u32 = 0o600;

/// Pause between two attempts to take the lock.
const GATE_RETRY_PAUSE: Duration = Duration::from_millis(10);

/// The lock file of one watched user.
pub fn adoption_gate_path(root: &Path, uid: u32) -> PathBuf {
    return root.join(format!("wg-quicker-exempt-adoption-{uid}.lock"));
}

/// The gate held for as long as this value lives.
pub struct AdoptionGate {
    /// The open lock file,
    /// whose descriptor carries the kernel lock.
    _file: File,
}

/// Takes the kernel lock on the gate file,
/// and answers absence when another holder has it.
fn try_lock(root: &Path, uid: u32) -> io::Result<Option<AdoptionGate>> {
    let path = adoption_gate_path(root, uid);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(GATE_FILE_MODE)
        .open(&path)?;
    // SAFETY: `file` owns a valid descriptor and `flock` changes only its kernel lock state.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result == 0 {
        return Ok(Some(AdoptionGate { _file: file }));
    }
    let error = io::Error::last_os_error();
    return match error.raw_os_error() {
        Some(libc::EWOULDBLOCK) => Ok(None),
        _ => Err(error),
    };
}

/// Takes the gate when it is free,
/// and answers absence at once when another holder has it,
/// so an adoption is deferred to the next scan instead of holding the watcher's loop.
pub fn try_acquire(root: &Path, uid: u32) -> io::Result<Option<AdoptionGate>> {
    return try_lock(root, uid);
}

/// Waits up to `limit` for the gate,
/// and answers absence when the limit passes with another holder still in it.
/// The caller says what it waited for and leaves that work for the next pass,
/// because a holder may have a slow manager call and the waiting work here is a cleanup.
pub fn acquire_within(root: &Path, uid: u32, limit: Duration) -> io::Result<Option<AdoptionGate>> {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(gate) = try_lock(root, uid)? {
            return Ok(Some(gate));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(GATE_RETRY_PAUSE);
    }
}
