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
use crate::process_info::parse_stat_fields;
/// Names a user chose,
/// delimited so a cause ends them where the line does.
use crate::report_cause::quote_name;
/// The calls of the manager.
use crate::scope_calls::{attach_call, create_call, ensure_slice_call, scope_description, set_properties_call, stop_call};
/// The names of the scope of an origin.
use crate::scope_naming::{origin_scope_names, scope_cgroup_directory};
/// Why a process is kept outside the tunnel,
/// which the scope's description names.
use crate::update_entry_points::MatchReason;
/// The seam that runs one call against the user's manager.
use crate::manager_socket::{ManagerCaller, ManagerError, PrivateSocketManager};
/// A call's reply and the arguments of a call.
use crate::dbus_wire::{Message, MethodCall};
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
/// What the manager says,
/// in the client's message,
/// when a pid of the request no longer exists (`ESRCH`),
/// for example `Call failed: No such process` or `Failed to set unit properties: No such process`.
const NO_SUCH_PROCESS: &str = "No such process";

/// One adoption of a process group into the scope of its origin.
pub struct AdoptRequest<'a> {
    /// Processes to move,
    /// the matched process first.
    pub pids: &'a [i32],
    /// User whose manager adopts them.
    pub uid: u32,
    /// Slice that receives the scope.
    pub slice: &'a str,
    /// Limits a new scope carries,
    /// read from the origin cgroup the processes leave.
    pub limits: &'a InheritedLimits,
    /// Name of the scope of this origin,
    /// as `origin_scope_names` resolves a scope that is already running under an earlier spelling.
    pub scope_name: &'a str,
    /// The cgroup the processes leave,
    /// which the scope's description names.
    pub origin: &'a str,
    /// The method that matched them,
    /// which the scope's description names.
    pub reason: MatchReason,
    /// Start time of the first process as it was read when it was matched,
    /// so a pid that another process took over later counts as gone.
    /// `None` skips that check.
    pub first_start_time: Option<u64>,
}

/// Whether a process is established as gone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Presence {
    /// The entry is absent,
    /// the process is a zombie or exiting,
    /// or the pid belongs to another process.
    Gone,
    /// Its state was read and it is running.
    Alive,
    /// Its state could not be read or understood.
    Unknown,
}

/// New limits for a scope that is running.
pub struct LimitRequest<'a> {
    /// User whose manager owns the scope.
    pub uid: u32,
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
    /// The manager answered "No such process" while the first process was not established as gone
    /// and another process of the group was.
    /// The group is collected again on the next scan,
    /// so a few in a row need no report,
    /// and the policy reports a run of them.
    MemberVanished,
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
            AdoptError::MemberVanished => write!(formatter, "a process of the group exited before the group could be moved"),
            AdoptError::ScopeGone => write!(formatter, "the scope ended before its limits could be applied"),
            AdoptError::Failed(text) | AdoptError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}

/// Seam between the policy and the user's service manager.
pub trait Adopter {
    /// Creates the slice when it does not exist.
    fn ensure_slice(&self, uid: u32, slice: &str, slice_directory: &Path) -> io::Result<()>;
    /// The name of the scope of `origin` that is running right now,
    /// current spelling or an earlier one,
    /// so its later processes join it instead of one more scope with the same cap.
    fn existing_scope_name(&self, uid: u32, slice: &str, origin: &str) -> Option<String>;
    /// Moves the requested processes into the scope of their origin,
    /// creating the scope with the request's limits when it does not exist.
    fn adopt(&self, request: &AdoptRequest<'_>) -> Result<Joined, AdoptError>;
    /// Gives a running scope the limits of its origin as they are now.
    fn apply_limits(&self, request: &LimitRequest<'_>) -> Result<(), AdoptError>;
}

/// Adopts over the user manager's own private socket.
/// The manager is the only process that can answer there,
/// and the client checks the pid behind the socket before it sends anything,
/// so no bus name another process could take stands between the watcher and the manager.
pub struct PrivateSocketAdopter {
    /// The seam that runs one call against the manager.
    pub caller: Box<dyn ManagerCaller>,
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
impl PrivateSocketAdopter {
    /// Builds the production adopter.
    pub fn new() -> PrivateSocketAdopter {
        return PrivateSocketAdopter {
            caller: Box::new(PrivateSocketManager::new()),
            proc_root: PathBuf::from("/proc"),
            cgroup_root: PathBuf::from("/sys/fs/cgroup"),
            confirmation_timeout: MOVE_CONFIRMATION_TIMEOUT,
        };
    }

    /// Runs one call and turns its failure into the adoption's error.
    fn run(&self, uid: u32, call: &MethodCall) -> Result<Message, AdoptError> {
        return match self.caller.call(uid, call) {
            Ok(reply) => Ok(reply),
            Err(ManagerError::TimedOut(text)) => Err(AdoptError::TimedOut(text)),
            Err(ManagerError::Refused(text) | ManagerError::Failed(text)) => Err(AdoptError::Failed(text)),
        };
    }

    /// Establishes whether a process is gone,
    /// from its `stat` file:
    /// the entry is absent,
    /// it is a zombie or flagged as exiting,
    /// or its start time is not `start_time`,
    /// so the pid belongs to another process now.
    /// A file that cannot be read for another reason,
    /// or cannot be parsed,
    /// leaves it unknown,
    /// and an unknown process is never counted as gone.
    fn presence(&self, pid: i32, start_time: Option<u64>) -> Presence {
        let text = match std::fs::read(self.proc_root.join(pid.to_string()).join("stat")) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH) => return Presence::Gone,
            Err(_unreadable) => return Presence::Unknown,
        };
        let Some(fields) = parse_stat_fields(&text) else {
            return Presence::Unknown;
        };
        if fields.exiting() {
            return Presence::Gone;
        }
        if let Some(expected) = start_time
            && fields.start_time != expected
        {
            return Presence::Gone;
        }
        return Presence::Alive;
    }

    /// Waits until the pid's cgroup path names the slice,
    /// and notices at once when the process exits meanwhile.
    /// A process that is exiting is never moved:
    /// the kernel skips it while the write to `cgroup.procs` still succeeds,
    /// and a zombie keeps its old cgroup path,
    /// so waiting out the timeout would report a failure for a process that is simply gone.
    /// A process whose state cannot be established is waited for like a living one.
    fn confirm_moved(&self, pid: i32, slice: &str, start_time: Option<u64>) -> Result<(), AdoptError> {
        let path = self.proc_root.join(pid.to_string()).join("cgroup");
        let started = Instant::now();
        loop {
            match std::fs::read_to_string(&path) {
                Ok(text) if text.contains(slice) => return Ok(()),
                Ok(_text) => {
                    if self.presence(pid, start_time) == Presence::Gone {
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

    /// Reports whether any of the processes is established as gone or on its way out.
    fn any_gone(&self, pids: &[i32]) -> bool {
        for pid in pids {
            if self.presence(*pid, None) == Presence::Gone {
                return true;
            }
        }
        return false;
    }

    /// Runs one manager call as the user and turns its failure into the adoption's error.
    /// A failure while the first process is established as gone or exiting is a vanish and not a failure.
    /// The manager's "No such process" answer,
    /// while the first process is not established as gone
    /// and another process of the request is,
    /// is a vanish of a member:
    /// the answer names no pid,
    /// and the group is collected again on the next scan.
    /// Anything else stays a failure,
    /// including a process whose state could not be read,
    /// because a failed move of a living process must not be hidden.
    fn call(&self, request: &AdoptRequest<'_>, first: i32, call: &MethodCall) -> Result<(), AdoptError> {
        return match self.run(request.uid, call) {
            Ok(_reply) => Ok(()),
            Err(AdoptError::TimedOut(text)) => Err(AdoptError::TimedOut(text)),
            Err(error) => {
                let answers_no_such_process = matches!(&error, AdoptError::Failed(text) if text.contains(NO_SUCH_PROCESS) || text.contains("NoSuchProcess"));
                if self.presence(first, request.first_start_time) == Presence::Gone {
                    Err(AdoptError::Vanished)
                } else if answers_no_such_process && self.any_gone(request.pids) {
                    Err(AdoptError::MemberVanished)
                } else {
                    Err(AdoptError::Failed(error.to_string()))
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
        let _stopped = self.run(request.uid, &stop_call(request.scope_name));
        return true;
    }

    /// Confirms the move,
    /// and stops a scope the move left empty.
    /// The emptiness check runs after a failed confirmation too,
    /// because a process that was exiting never moves and leaves a new scope empty.
    fn settle(&self, request: &AdoptRequest<'_>, first: i32, joined: Joined) -> Result<Joined, AdoptError> {
        let confirmed = self.confirm_moved(first, request.slice, request.first_start_time);
        let emptied = self.stop_if_empty(request);
        return match confirmed {
            Ok(()) if emptied => Err(AdoptError::Vanished),
            Ok(()) => Ok(joined),
            Err(error) => Err(error),
        };
    }
}

/// The production seam to the user's manager.
impl Adopter for PrivateSocketAdopter {
    /// Requests the slice through the manager and waits for its cgroup directory.
    fn ensure_slice(&self, uid: u32, slice: &str, slice_directory: &Path) -> io::Result<()> {
        if slice_directory.is_dir() {
            return Ok(());
        }
        if let Err(error) = self.run(uid, &ensure_slice_call(slice)) {
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

    /// The first of the origin's scope names whose cgroup directory exists,
    /// so a scope that earlier builds named differently still receives the origin's processes.
    fn existing_scope_name(&self, uid: u32, slice: &str, origin: &str) -> Option<String> {
        for name in origin_scope_names(origin, uid) {
            if scope_cgroup_directory(&self.cgroup_root, uid, slice, &name).is_dir() {
                return Some(name);
            }
        }
        return None;
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
            match self.call(request, *first, &attach_call(request.scope_name, request.pids)) {
                Ok(()) => return self.settle(request, *first, Joined::Attached),
                Err(error) if directory.is_dir() => return Err(error),
                // The scope ended while it was being attached to: create it afresh.
                Err(_ended) => {}
            }
        }
        let description = scope_description(request.origin, request.reason);
        self.call(request, *first, &create_call(request.scope_name, request.slice, request.pids, request.limits, &description))?;
        return self.settle(request, *first, Joined::Created);
    }

    /// Gives the running scope the limits through `SetUnitProperties`.
    fn apply_limits(&self, request: &LimitRequest<'_>) -> Result<(), AdoptError> {
        let directory = scope_cgroup_directory(&self.cgroup_root, request.uid, request.slice, request.scope_name);
        if !directory.is_dir() {
            return Err(AdoptError::ScopeGone);
        }
        return match self.run(request.uid, &set_properties_call(request.scope_name, request.limits)) {
            Ok(_reply) => Ok(()),
            Err(AdoptError::TimedOut(text)) => Err(AdoptError::TimedOut(text)),
            Err(error) if directory.is_dir() => Err(error),
            Err(_ended) => Err(AdoptError::ScopeGone),
        };
    }
}
