//! Moves processes into the updates slice by asking the user's systemd manager to adopt them.
//! Processes that leave the same origin cgroup join one long-lived scope for that origin,
//! created once with the origin's limits,
//! so together they stay within the origin's cap instead of each carrying a copy of it.
//! The manager creates the scope for the first process,
//! later processes are attached to it,
//! and it ends by itself when its last process exits.
//! Writing to `cgroup.procs` directly would put processes into a cgroup the manager never created.
//! The manager would not garbage-collect it,
//! and its pid-to-unit map would go stale.
//! That breaks the single-writer rule of systemd's cgroup delegation document.
//! The manager can only move processes inside its own subtree,
//! `user@<uid>.service`,
//! so other processes are reported and left where they are.

/// The limits the scope carries.
use crate::inherited_limits::InheritedLimits;
/// Whether the process is on its way out.
use crate::process_info::read_exiting;
/// Names a user chose,
/// delimited so a cause ends them where the line does.
use crate::report_cause::quote_name;
/// The arguments of the manager calls.
use crate::scope_calls::{attach_arguments, create_arguments, ensure_slice_arguments, set_properties_arguments, stop_arguments};
/// Where a scope lives in the cgroup filesystem.
use crate::scope_naming::scope_cgroup_directory;
/// The client run as the user.
use crate::user_client::{ClientError, UserClient};
/// Syscall and process failures.
use std::io;
/// Fake procfs and cgroup roots in tests.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::{Duration, Instant};

/// How long the adopted pid may take to appear in the slice after the job is queued.
const MOVE_CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(1);
/// Pause between confirmation reads.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// One adoption of a process group into the scope of its origin.
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
    /// Limits a new scope carries,
    /// read from the origin cgroup the processes leave.
    pub limits: &'a InheritedLimits,
    /// Name of the scope of this origin.
    pub scope_name: &'a str,
}

/// New limits for a scope that is running.
pub struct LimitRequest<'a> {
    /// User whose manager owns the scope.
    pub uid: u32,
    /// Group id used when dropping privilege for the bus call.
    pub gid: u32,
    /// Slice that holds the scope.
    pub slice: &'a str,
    /// Name of the scope.
    pub scope_name: &'a str,
    /// The limits it gets.
    pub limits: &'a InheritedLimits,
}

/// How the processes got into the scope.
#[derive(Debug, Eq, PartialEq)]
pub enum Joined {
    /// The scope did not exist and was created with the origin's limits.
    Created,
    /// The scope existed and the processes were attached to it.
    Attached,
}

/// Why an adoption did not happen.
#[derive(Debug, Eq, PartialEq)]
pub enum AdoptError {
    /// The first process exited before it could be moved,
    /// or was exiting and was left alone,
    /// which needs no report.
    Vanished,
    /// The scope ended before the limits could be applied,
    /// which needs no report.
    ScopeGone,
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
            AdoptError::ScopeGone => write!(formatter, "the scope ended before its limits could be applied"),
            AdoptError::Failed(text) | AdoptError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}

/// Seam between the policy and the user's service manager.
pub trait Adopter {
    /// Creates the slice when it does not exist.
    fn ensure_slice(&self, uid: u32, gid: u32, slice: &str, slice_directory: &Path) -> io::Result<()>;
    /// Moves the requested processes into the scope of their origin,
    /// creating the scope with the request's limits when it does not exist.
    fn adopt(&self, request: &AdoptRequest<'_>) -> Result<Joined, AdoptError>;
    /// Gives a running scope the limits of its origin as they are now.
    fn apply_limits(&self, request: &LimitRequest<'_>) -> Result<(), AdoptError>;
}

/// Adopts through the `busctl` command line client.
/// The marshalling of D-Bus values is the client's,
/// not hand-written wire code in a root process.
pub struct BusctlAdopter {
    /// The client run as the target user.
    pub client: UserClient,
    /// Procfs root used to confirm the move.
    pub proc_root: PathBuf,
    /// Cgroup filesystem root,
    /// where a scope's directory and its populated state are read.
    pub cgroup_root: PathBuf,
    /// How long the moved pid may take to appear in the slice.
    pub confirmation_timeout: Duration,
}

/// Reports a `cgroup.events` line that says the cgroup holds no process.
fn is_unpopulated(line: &str) -> bool {
    return line.trim() == "populated 0";
}

/// Construction and the steps of an adoption.
impl BusctlAdopter {
    /// Builds the production adopter.
    pub fn new() -> BusctlAdopter {
        return BusctlAdopter {
            client: UserClient::new("busctl"),
            proc_root: PathBuf::from("/proc"),
            cgroup_root: PathBuf::from("/sys/fs/cgroup"),
            confirmation_timeout: MOVE_CONFIRMATION_TIMEOUT,
        };
    }

    /// Reports whether the process still exists.
    fn process_exists(&self, pid: i32) -> bool {
        return self.proc_root.join(pid.to_string()).join("cgroup").exists();
    }

    /// Waits until the pid's cgroup path names the slice,
    /// and notices at once when the process exits meanwhile.
    /// A process that is exiting is never moved:
    /// the kernel skips it while the write to `cgroup.procs` still succeeds,
    /// and a zombie keeps its old cgroup path,
    /// so waiting out the timeout would report a failure for a process that is simply gone.
    fn confirm_moved(&self, pid: i32, slice: &str) -> Result<(), AdoptError> {
        let path = self.proc_root.join(pid.to_string()).join("cgroup");
        let started = Instant::now();
        loop {
            match std::fs::read_to_string(&path) {
                Ok(text) if text.contains(slice) => return Ok(()),
                Ok(_text) => {
                    if read_exiting(&self.proc_root, pid) != Some(false) {
                        return Err(AdoptError::Vanished);
                    }
                }
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

    /// The directory of the scope of `request` in the cgroup filesystem.
    fn directory_of(&self, request: &AdoptRequest<'_>) -> PathBuf {
        return scope_cgroup_directory(&self.cgroup_root, request.uid, request.slice, request.scope_name);
    }

    /// Runs one manager call as the user and turns its failure into the adoption's error.
    /// A failure while the first process is gone is a vanish and not a failure.
    fn call(&self, request: &AdoptRequest<'_>, first: i32, arguments: &[String]) -> Result<(), AdoptError> {
        return match self.client.run(request.uid, request.gid, arguments) {
            Ok(_output) => Ok(()),
            Err(ClientError::TimedOut(text)) => Err(AdoptError::TimedOut(text)),
            Err(error) => {
                if self.process_exists(first) {
                    Err(AdoptError::Failed(error.to_string()))
                } else {
                    Err(AdoptError::Vanished)
                }
            }
        };
    }

    /// Stops the scope when its cgroup holds no process,
    /// and reports whether it was empty.
    /// A scope that adopted only exiting processes is running with `populated 0` and never gets an empty event
    /// (`scope_enter_running` in systemd v259.10 `src/core/scope.c`,
    /// line 403 onward),
    /// so it would run forever;
    /// a scope whose directory is gone has ended and holds nothing either.
    /// Stopping is best effort:
    /// the manager may have stopped an emptied scope itself.
    fn stop_if_empty(&self, request: &AdoptRequest<'_>) -> bool {
        let events = self.directory_of(request).join("cgroup.events");
        let Ok(text) = std::fs::read_to_string(events) else {
            return true;
        };
        if !text.lines().any(is_unpopulated) {
            return false;
        }
        let _stopped = self.client.run(request.uid, request.gid, &stop_arguments(request.scope_name));
        return true;
    }

    /// Confirms the move,
    /// and stops a scope the move left empty.
    /// The emptiness check runs after a failed confirmation too,
    /// because a process that was exiting never moves and leaves a new scope empty.
    fn settle(&self, request: &AdoptRequest<'_>, first: i32, joined: Joined) -> Result<Joined, AdoptError> {
        let confirmed = self.confirm_moved(first, request.slice);
        let emptied = self.stop_if_empty(request);
        return match confirmed {
            Ok(()) if emptied => Err(AdoptError::Vanished),
            Ok(()) => Ok(joined),
            Err(error) => Err(error),
        };
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
                return Err(io::Error::other(format!("{slice} was requested but {} never appeared", quote_name(&slice_directory.display().to_string()))));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
        return Ok(());
    }

    /// Attaches the pids to the scope of their origin when it exists,
    /// and creates it with the origin's limits when it does not,
    /// then waits until the first pid is in the slice.
    /// Whether the scope exists is read from its cgroup directory,
    /// which exists exactly while the scope is active,
    /// so no state is kept here and a restarted watcher finds the scopes of the one before.
    /// A failure is never retried without the limits,
    /// because a process that kept its place keeps its caps and one moved without them would lose them.
    fn adopt(&self, request: &AdoptRequest<'_>) -> Result<Joined, AdoptError> {
        let Some(first) = request.pids.first() else {
            return Err(AdoptError::Failed("no pid to adopt".to_string()));
        };
        let directory = self.directory_of(request);
        if directory.is_dir() {
            match self.call(request, *first, &attach_arguments(request.scope_name, request.pids)) {
                Ok(()) => return self.settle(request, *first, Joined::Attached),
                Err(error) if directory.is_dir() => return Err(error),
                // The scope ended while it was being attached to: create it afresh.
                Err(_ended) => {}
            }
        }
        self.call(request, *first, &create_arguments(request.scope_name, request.slice, request.pids, request.limits))?;
        return self.settle(request, *first, Joined::Created);
    }

    /// Gives the running scope the limits through `SetUnitProperties`.
    fn apply_limits(&self, request: &LimitRequest<'_>) -> Result<(), AdoptError> {
        let directory = scope_cgroup_directory(&self.cgroup_root, request.uid, request.slice, request.scope_name);
        if !directory.is_dir() {
            return Err(AdoptError::ScopeGone);
        }
        return match self.client.run(request.uid, request.gid, &set_properties_arguments(request.scope_name, request.limits)) {
            Ok(_output) => Ok(()),
            Err(ClientError::TimedOut(text)) => Err(AdoptError::TimedOut(text)),
            Err(error) if directory.is_dir() => Err(AdoptError::Failed(error.to_string())),
            Err(_ended) => Err(AdoptError::ScopeGone),
        };
    }
}
