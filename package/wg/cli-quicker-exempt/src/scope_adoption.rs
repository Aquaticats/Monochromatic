//! Moves processes into the updates slice by asking the user's systemd manager to adopt them.
//! The manager creates a transient scope under the slice and moves the listed pids into it,
//! so its own records stay true.
//! Writing to `cgroup.procs` directly would put processes into a cgroup the manager never created.
//! The manager would not garbage-collect it,
//! and its pid-to-unit map would go stale.
//! That breaks the single-writer rule of systemd's cgroup delegation document.
//! The manager can only move processes inside its own subtree,
//! `user@<uid>.service`,
//! so other processes are reported and left where they are.
//! The new scope carries the resource limits of the cgroup the process left,
//! so the move never loosens a CPU or memory cap.

/// The limits the scope carries.
use crate::inherited_limits::InheritedLimits;
/// The client run as the user.
use crate::user_client::{ClientError, UserClient};
/// Syscall and process failures.
use std::io;
/// Fake procfs roots in tests.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::{Duration, Instant};

/// How long the adopted pid may take to appear in the slice after the job is queued.
const MOVE_CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(1);
/// Pause between confirmation reads.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// One adoption of a process group into a transient scope.
pub struct AdoptRequest<'a> {
    /// Processes to move,
    /// the matched process first.
    pub pids: &'a [i32],
    /// User whose manager adopts them.
    pub uid: u32,
    /// Group id used when dropping privilege for the bus call.
    pub gid: u32,
    /// Slice that receives the scope.
    pub slice: &'a str,
    /// Limits the scope carries,
    /// read from the cgroup the processes leave.
    pub limits: &'a InheritedLimits,
}

/// Why an adoption did not happen.
#[derive(Debug, Eq, PartialEq)]
pub enum AdoptError {
    /// The first process exited before it could be moved,
    /// which needs no report.
    Vanished,
    /// The manager refused or the move did not show.
    Failed(
        /// The reason.
        String,
    ),
    /// The client ran past its timeout and was killed.
    /// The manager may be hung.
    TimedOut(
        /// The reason.
        String,
    ),
}

/// Wording for reports.
impl std::fmt::Display for AdoptError {
    /// Writes the plain text of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            AdoptError::Vanished => write!(formatter, "the process exited before it could be moved"),
            AdoptError::Failed(text) | AdoptError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}

/// Seam between the policy and the user's service manager.
pub trait Adopter {
    /// Creates the slice when it does not exist.
    fn ensure_slice(&self, uid: u32, gid: u32, slice: &str, slice_directory: &Path) -> io::Result<()>;
    /// Moves the requested processes into a new scope below the slice.
    fn adopt(&self, request: &AdoptRequest<'_>) -> Result<(), AdoptError>;
}

/// Builds the `busctl` arguments for `StartTransientUnit` of a slice.
pub fn ensure_slice_arguments(slice: &str) -> Vec<String> {
    return vec![
        "--user".to_string(),
        "call".to_string(),
        "org.freedesktop.systemd1".to_string(),
        "/org/freedesktop/systemd1".to_string(),
        "org.freedesktop.systemd1.Manager".to_string(),
        "StartTransientUnit".to_string(),
        "ssa(sv)a(sa(sv))".to_string(),
        slice.to_string(),
        "fail".to_string(),
        "1".to_string(),
        "Description".to_string(),
        "s".to_string(),
        "wg-quicker update traffic that bypasses the tunnel".to_string(),
        "0".to_string(),
    ];
}

/// Builds the `busctl` arguments for `StartTransientUnit` of a scope that adopts `pids`
/// and carries `limits`.
pub fn adopt_arguments(scope_name: &str, slice: &str, pids: &[i32], limits: &InheritedLimits) -> Vec<String> {
    let limit_properties = limits.properties();
    let mut arguments: Vec<String> = vec![
        "--user".to_string(),
        "call".to_string(),
        "org.freedesktop.systemd1".to_string(),
        "/org/freedesktop/systemd1".to_string(),
        "org.freedesktop.systemd1.Manager".to_string(),
        "StartTransientUnit".to_string(),
        "ssa(sv)a(sa(sv))".to_string(),
        scope_name.to_string(),
        "fail".to_string(),
        (3 + limit_properties.len()).to_string(),
        "Slice".to_string(),
        "s".to_string(),
        slice.to_string(),
        "PIDs".to_string(),
        "au".to_string(),
        pids.len().to_string(),
    ];
    for pid in pids {
        arguments.push(pid.to_string());
    }
    arguments.push("Description".to_string());
    arguments.push("s".to_string());
    arguments.push("wg-quicker update traffic that bypasses the tunnel".to_string());
    for (name, value) in limit_properties {
        arguments.push(name.to_string());
        arguments.push("t".to_string());
        arguments.push(value.to_string());
    }
    arguments.push("0".to_string());
    return arguments;
}

/// Adopts through the `busctl` command line client.
/// The marshalling of D-Bus values is the client's,
/// not hand-written wire code in a root process.
pub struct BusctlAdopter {
    /// The client run as the target user.
    pub client: UserClient,
    /// Procfs root used to confirm the move.
    pub proc_root: PathBuf,
    /// How long the moved pid may take to appear in the slice.
    pub confirmation_timeout: Duration,
}

/// Construction and the confirmation steps.
impl BusctlAdopter {
    /// Builds the production adopter.
    pub fn new() -> BusctlAdopter {
        return BusctlAdopter {
            client: UserClient::new("busctl"),
            proc_root: PathBuf::from("/proc"),
            confirmation_timeout: MOVE_CONFIRMATION_TIMEOUT,
        };
    }

    /// Reports whether the process still exists.
    fn process_exists(&self, pid: i32) -> bool {
        return self.proc_root.join(pid.to_string()).join("cgroup").exists();
    }

    /// Waits until the pid's cgroup path names the slice,
    /// and notices at once when the process exits meanwhile.
    fn confirm_moved(&self, pid: i32, slice: &str) -> Result<(), AdoptError> {
        let path = self.proc_root.join(pid.to_string()).join("cgroup");
        let started = Instant::now();
        loop {
            match std::fs::read_to_string(&path) {
                Ok(text) if text.contains(slice) => return Ok(()),
                Ok(_text) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH) => {
                    return Err(AdoptError::Vanished);
                }
                Err(error) => return Err(AdoptError::Failed(format!("reading the cgroup of pid {pid} failed: {error}"))),
            }
            if started.elapsed() > self.confirmation_timeout {
                return Err(AdoptError::Failed(format!(
                    "the manager accepted the job but pid {pid} is not in {slice} after {:?}",
                    self.confirmation_timeout
                )));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }
}

/// The production seam to the user's manager.
impl Adopter for BusctlAdopter {
    /// Requests the slice through the client and waits for its cgroup directory.
    fn ensure_slice(&self, uid: u32, gid: u32, slice: &str, slice_directory: &Path) -> io::Result<()> {
        if slice_directory.is_dir() {
            return Ok(());
        }
        if let Err(error) = self.client.run(uid, gid, &ensure_slice_arguments(slice)) {
            return Err(io::Error::other(error.to_string()));
        }
        let started = Instant::now();
        while !slice_directory.is_dir() {
            if started.elapsed() > self.confirmation_timeout {
                return Err(io::Error::other(format!("{slice} was requested but {} never appeared", slice_directory.display())));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
        return Ok(());
    }

    /// Requests a scope that adopts the pids and waits until the first pid is in the slice.
    /// A failure is never retried without the limits,
    /// because a process that kept its place keeps its caps and one moved without them would lose them.
    fn adopt(&self, request: &AdoptRequest<'_>) -> Result<(), AdoptError> {
        let Some(first) = request.pids.first() else {
            return Err(AdoptError::Failed("no pid to adopt".to_string()));
        };
        let scope_name = format!("wg-quicker-update-{first}.scope");
        let arguments = adopt_arguments(&scope_name, request.slice, request.pids, request.limits);
        match self.client.run(request.uid, request.gid, &arguments) {
            Ok(()) => {}
            Err(ClientError::TimedOut(text)) => return Err(AdoptError::TimedOut(text)),
            Err(error) => {
                if !self.process_exists(*first) {
                    return Err(AdoptError::Vanished);
                }
                return Err(AdoptError::Failed(error.to_string()));
            }
        }
        return self.confirm_moved(*first, request.slice);
    }
}
