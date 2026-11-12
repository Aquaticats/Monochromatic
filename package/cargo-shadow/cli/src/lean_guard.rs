//! What:
//! The proof that a Cargo profile directory is not in use,
//! made of the build locks taken without waiting and a scan of every process.
//! Why:
//! Nothing a running build,
//! a checkout's lock or a running container uses may be removed.
//! The proof is taken under the locks and kept as text so each removal's log line carries it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function guardProfile(profile: string, env: GuardEnvironment): { ready: Guarded } | { skip: string }
//! ```

/// Paths.
use std::path::{Path, PathBuf};

/// The proof and the deadline it must answer inside.
use crate::lean_identity::Deadline;
/// The locks of the profile directory.
use crate::lean_lock_probe::{HeldLocks, LockOutcome, describe_held, lock_profile_directory};
/// The scan of the processes.
use crate::lean_proc_scan::{ScanReport, identities_of_existing, scan_processes};

/// Where the guard reads the machine's lock table and process table,
/// replaceable so tests read fixtures.
#[derive(Clone, Debug)]
pub struct GuardEnvironment {
    /// The kernel lock table (`/proc/locks`).
    pub lock_table: PathBuf,
    /// The procfs root (`/proc`).
    pub proc_root: PathBuf,
    /// The process that must not count against the directory:
    /// this one.
    pub own_pid: i32,
}

/// The environment of the real machine.
impl GuardEnvironment {
    /// The real machine.
    pub fn real() -> GuardEnvironment {
        return GuardEnvironment {
            lock_table: PathBuf::from(crate::lean_lock_probe::KERNEL_LOCK_TABLE),
            proc_root: PathBuf::from("/proc"),
            own_pid: i32::try_from(std::process::id()).unwrap_or(-1),
        };
    }
}

/// A profile directory that is locked and scanned.
#[derive(Debug)]
pub struct Guarded {
    /// The locks,
    /// held until this value drops.
    pub locks: HeldLocks,
    /// The processes using something under the profile directory.
    pub scan: ScanReport,
    /// The proof as text for a log line.
    pub proof: String,
}

/// The result of asking for the proof.
#[derive(Debug)]
pub enum Guard {
    /// The directory is quiescent;
    /// the locks are held.
    Ready(
        /// The profile directory with its locks held.
        Guarded,
    ),
    /// The directory is skipped for this reason.
    Skip(
        /// Why the directory is skipped.
        String,
    ),
}

/// What:
/// `pub fn guard_profile(profile: &Path, environment: &GuardEnvironment, deadline: &Deadline) -> Guard` takes the build locks of the directory
/// without waiting and scans every process for uses under it.
/// Why:
/// A busy lock,
/// a queued waiter,
/// a file system that cannot lock,
/// and a directory with no lock file all skip the directory;
/// a process whose working directory,
/// program,
/// open file or mapped file lies under it blocks only the entries it touches (`ScanReport::blocks`).
/// A scan the deadline stopped is a skip and never a proof:
/// it read some of the machine's processes and cannot say what the rest hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function guardProfile(profile: string, env: GuardEnvironment, deadline: Deadline): Guard
/// ```
pub fn guard_profile(profile: &Path, environment: &GuardEnvironment, deadline: &Deadline) -> Guard {
    let locks: HeldLocks = match lock_profile_directory(profile, &environment.lock_table) {
        LockOutcome::Held(held) => held,
        LockOutcome::Busy(file) => return Guard::Skip(format!("busy: a build holds {}", file.display())),
        LockOutcome::Waiting(file) => return Guard::Skip(format!("a build is queued behind {}", file.display())),
        LockOutcome::Unlockable { file, reason } => return Guard::Skip(format!("{} cannot be locked here: {reason}", file.display())),
        LockOutcome::NoLockFile => return Guard::Skip(String::from("no Cargo lock file, so nothing shows the directory quiescent")),
    };
    let identities = identities_of_existing(locks.names());
    let Some(scan) = scan_processes(&environment.proc_root, &[profile], &identities, environment.own_pid, deadline) else {
        return Guard::Skip(String::from("the deadline passed while scanning the processes, so nothing was removed"));
    };
    let proof: String = format!("{}; {}", describe_held(&locks), scan.describe());
    return Guard::Ready(Guarded { locks, scan, proof });
}
