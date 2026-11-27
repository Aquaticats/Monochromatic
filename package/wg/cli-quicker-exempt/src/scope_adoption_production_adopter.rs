//! The adopter that reaches the user's manager over its own private socket:
//! the calls it builds,
//! and the arguments each one carries.

/// The interface this adopter implements.
use super::Adopter;
/// How long a confirmation may take,
/// and how long the move waits between two reads.
use super::adoption_confirmation::{
    MOVE_CONFIRMATION_TIMEOUT,
    POLL_INTERVAL,
};
/// What a move reports,
/// what the policy asks for,
/// and whether a process is established as gone.
use super::adoption_words::{
    AdoptError,
    AdoptRequest,
    Joined,
    LimitRequest,
    Presence,
};
/// The lock an origin stop and a move share.
use crate::adoption_gate;
/// A call's reply and the arguments of a call.
use crate::dbus_wire::{
    Message,
    MethodCall,
};
/// The seam that runs one call against the user's manager.
use crate::manager_socket::{
    ManagerCaller,
    ManagerError,
    PrivateSocketManager,
};
/// Names a user chose,
/// delimited so a cause ends them where the line does.
use crate::report_cause::quote_name;
/// The calls of the manager.
use crate::scope_calls::{
    attach_call,
    create_call,
    ensure_slice_call,
    scope_description,
    set_properties_call,
};
/// The names of the scope of an origin,
/// and the directory that scope occupies.
use crate::scope_naming::{
    origin_scope_names,
    scope_cgroup_directory,
};
/// Syscall and process failures.
use std::io;
/// Fake procfs and cgroup roots in tests.
use std::path::{
    Path,
    PathBuf,
};
/// Bounded waits.
use std::time::{
    Duration,
    Instant,
};

/// What the manager says,
/// in the client's message,
/// when a pid of the request no longer exists (`ESRCH`),
/// for example `Call failed: No such process` or `Failed to set unit properties: No such process`.
const NO_SUCH_PROCESS: &str = "No such process";

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
    /// Directory the adoption gate file lives in.
    pub gate_root: PathBuf,
    /// How long the moved pid may take to appear in the slice.
    pub confirmation_timeout: Duration,
}

/// Construction and the calls of an adoption.
impl PrivateSocketAdopter {
    /// Builds the production adopter.
    pub fn new() -> PrivateSocketAdopter {
        return PrivateSocketAdopter {
            caller: Box::new(PrivateSocketManager::new()),
            proc_root: PathBuf::from("/proc"),
            cgroup_root: PathBuf::from("/sys/fs/cgroup"),
            gate_root: PathBuf::from(adoption_gate::PRODUCTION_GATE_ROOT),
            confirmation_timeout: MOVE_CONFIRMATION_TIMEOUT,
        };
    }

    /// Runs one call and turns its failure into the adoption's error.
    pub(super) fn run(
        &self,
        uid: u32,
        call: &MethodCall,
    ) -> Result<Message, AdoptError> {
        return match self.caller.call(uid, call) {
            Ok(reply) => Ok(reply),
            Err(ManagerError::TimedOut(text)) => Err(AdoptError::TimedOut(text)),
            Err(ManagerError::Refused(text) | ManagerError::Failed(text)) => {
                Err(AdoptError::Failed(text))
            }
        };
    }

    /// The directory of the scope of `request` in the cgroup filesystem.
    pub(super) fn directory_of(
        &self,
        request: &AdoptRequest<'_>,
    ) -> PathBuf {
        return scope_cgroup_directory(
            &self.cgroup_root,
            request.uid,
            request.slice,
            request.scope_name,
        );
    }

    /// Reports whether any of the processes is established as gone or on its way out.
    fn any_gone(
        &self,
        pids: &[i32],
    ) -> bool {
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
    fn call(
        &self,
        request: &AdoptRequest<'_>,
        first: i32,
        call: &MethodCall,
    ) -> Result<(), AdoptError> {
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
}

/// The production seam to the user's manager.
impl Adopter for PrivateSocketAdopter {
    /// Requests the slice through the manager and waits for its cgroup directory.
    fn ensure_slice(
        &self,
        uid: u32,
        slice: &str,
        slice_directory: &Path,
    ) -> io::Result<()> {
        if slice_directory.is_dir() {
            return Ok(());
        }
        if let Err(error) = self.run(uid, &ensure_slice_call(slice)) {
            return Err(io::Error::other(error.to_string()));
        }
        let started = Instant::now();
        while !slice_directory.is_dir() {
            if started.elapsed() > self.confirmation_timeout {
                return Err(io::Error::other(format!(
                    "{slice} was requested but {} never appeared",
                    quote_name(&slice_directory.display().to_string())
                )));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
        return Ok(());
    }

    /// The first of the origin's scope names whose cgroup directory exists,
    /// so a scope that earlier builds named differently still receives the origin's processes.
    fn existing_scope_name(
        &self,
        uid: u32,
        slice: &str,
        origin: &str,
    ) -> Option<String> {
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
    fn adopt(
        &self,
        request: &AdoptRequest<'_>,
    ) -> Result<Joined, AdoptError> {
        let Some(first) = request.pids.first() else {
            return Err(AdoptError::Failed("no pid to adopt".to_string()));
        };
        // Held until this function returns:
        // a watcher's sweep of empty origin scopes reads and stops a scope under the same gate,
        // because the manager ends every process in a scope it stops.
        // A holder of the gate is one move, so waiting here would stall the policy loop:
        // a move that cannot start at once is left for the next scan, which reads the process again.
        let _held = match adoption_gate::try_acquire(&self.gate_root, request.uid) {
            Ok(Some(held)) => held,
            Ok(None) => return Err(AdoptError::Deferred),
            Err(error) => {
                return Err(AdoptError::Failed(format!(
                    "the adoption gate could not be taken: {error}"
                )));
            }
        };
        let directory = self.directory_of(request);
        if directory.is_dir() {
            match self.call(
                request,
                *first,
                &attach_call(request.scope_name, request.pids),
            ) {
                Ok(()) => return self.settle(request, *first, Joined::Attached),
                Err(error) if directory.is_dir() => return Err(error),
                // The scope ended while it was being attached to: create it afresh.
                Err(_ended) => {}
            }
        }
        let description = scope_description(request.origin, request.reason);
        self.call(
            request,
            *first,
            &create_call(
                request.scope_name,
                request.slice,
                request.pids,
                request.limits,
                &description,
            ),
        )?;
        return self.settle(request, *first, Joined::Created);
    }

    /// Gives the running scope the limits through `SetUnitProperties`.
    fn apply_limits(
        &self,
        request: &LimitRequest<'_>,
    ) -> Result<(), AdoptError> {
        let directory = scope_cgroup_directory(
            &self.cgroup_root,
            request.uid,
            request.slice,
            request.scope_name,
        );
        if !directory.is_dir() {
            return Err(AdoptError::ScopeGone);
        }
        return match self.run(
            request.uid,
            &set_properties_call(request.scope_name, request.limits),
        ) {
            Ok(_reply) => Ok(()),
            Err(AdoptError::TimedOut(text)) => Err(AdoptError::TimedOut(text)),
            Err(error) if directory.is_dir() => Err(error),
            Err(_ended) => Err(AdoptError::ScopeGone),
        };
    }
}
