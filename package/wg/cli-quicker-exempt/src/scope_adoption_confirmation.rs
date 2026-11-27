//! The checks that prove a move happened,
//! and the stop of a scope the move left empty.

/// What a move reports,
/// and whether a process is established as gone.
use super::adoption_words::{
    AdoptError,
    AdoptRequest,
    Joined,
    Presence,
};
/// The adopter whose steps these are.
use super::production_adopter::PrivateSocketAdopter;
/// Whether the process is on its way out.
use crate::process_info::parse_stat_fields;
/// The calls of the manager.
use crate::scope_calls::stop_call;
/// The path of a scope as procfs spells it.
use crate::scope_naming::scope_cgroup_path;
/// Syscall and process failures.
use std::io;
/// Bounded waits.
use std::time::{
    Duration,
    Instant,
};

/// How long the adopted pid may take to appear in the slice after the job is queued.
pub(super) const MOVE_CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(1);
/// Pause between confirmation reads.
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Reports a `cgroup.events` line that says the cgroup holds no process.
fn is_unpopulated(line: &str) -> bool {
    return line.trim() == "populated 0";
}

/// The unified cgroup path one `/proc/<pid>/cgroup` text names.
fn unified_cgroup_path(text: &str) -> Option<&str> {
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("0::") {
            return Some(path);
        }
    }
    return None;
}

/// Reports whether one `/proc/<pid>/cgroup` text names `destination` or a cgroup below it.
fn names_destination(
    text: &str,
    destination: &str,
) -> bool {
    let Some(cgroup) = unified_cgroup_path(text) else {
        return false;
    };
    return is_within(cgroup, destination);
}

/// Reports whether a process's cgroup path is `destination` or lies below it.
///
/// Whole path components are compared,
/// because a unit whose name contains the slice's name sits beside the slice and not in it:
/// `.../app.slice/app-monochromatic\x2dupdates.slice-task.scope` holds every character of
/// `app-monochromatic\x2dupdates.slice`,
/// and a check on the text alone takes that lookalike for a move.
fn is_within(
    cgroup: &str,
    destination: &str,
) -> bool {
    if cgroup == destination {
        return true;
    }
    return cgroup.starts_with(&format!("{destination}/"));
}

/// The steps that prove a move,
/// and the stop of a scope it left empty.
impl PrivateSocketAdopter {
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
    pub(super) fn presence(
        &self,
        pid: i32,
        start_time: Option<u64>,
    ) -> Presence {
        let text = match std::fs::read(self.proc_root.join(pid.to_string()).join("stat")) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(error)
                if error.kind() == io::ErrorKind::NotFound
                    || error.raw_os_error() == Some(libc::ESRCH) =>
            {
                return Presence::Gone;
            }
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

    /// Waits until the pid's cgroup path is `destination` or lies below it,
    /// and notices at once when the process exits meanwhile.
    /// A process that is exiting is never moved:
    /// the kernel skips it while the write to `cgroup.procs` still succeeds,
    /// and a zombie keeps its old cgroup path,
    /// so waiting out the timeout would report a failure for a process that is simply gone.
    /// A process whose state cannot be established is waited for like a living one.
    fn confirm_moved(
        &self,
        pid: i32,
        destination: &str,
        start_time: Option<u64>,
    ) -> Result<(), AdoptError> {
        let path = self.proc_root.join(pid.to_string()).join("cgroup");
        let started = Instant::now();
        loop {
            match std::fs::read_to_string(&path) {
                Ok(text) if names_destination(&text, destination) => return Ok(()),
                Ok(_text) => {
                    if self.presence(pid, start_time) == Presence::Gone {
                        return Err(AdoptError::Vanished);
                    }
                }
                Err(error)
                    if error.kind() == io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(libc::ESRCH) =>
                {
                    return Err(AdoptError::Vanished);
                }
                Err(error) => {
                    return Err(AdoptError::Failed(format!(
                        "reading the cgroup of pid {pid} failed: {error}"
                    )));
                }
            }
            if started.elapsed() > self.confirmation_timeout {
                return Err(AdoptError::Failed(format!(
                    "the manager accepted the job but pid {pid} is not in {destination} after {:?}",
                    self.confirmation_timeout
                )));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
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
    fn stop_if_empty(
        &self,
        request: &AdoptRequest<'_>,
    ) -> bool {
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
    pub(super) fn settle(
        &self,
        request: &AdoptRequest<'_>,
        first: i32,
        joined: Joined,
    ) -> Result<Joined, AdoptError> {
        let destination = scope_cgroup_path(request.uid, request.slice, request.scope_name);
        let confirmed = self.confirm_moved(first, &destination, request.first_start_time);
        let emptied = self.stop_if_empty(request);
        return match confirmed {
            Ok(()) if emptied => Err(AdoptError::Vanished),
            Ok(()) => Ok(joined),
            Err(error) => Err(error),
        };
    }
}
