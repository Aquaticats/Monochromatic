//! What:
//!  The two answers one clone's facts take from this machine without a pool:
//!  what stands at a path,
//!  and which tasks the caller's own session keeps.
//! Why:
//!  A registration's look and a removal's spared session are facts a report
//!  every person reads depends on,
//!  and neither needs `/dev/zfs`,
//!  so they are read here where a host control can prove them
//!  (plan step 18,
//!  "damaged or pruned Git registrations";
//!  `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
//!  "Which processes a forced removal never ends").
//!  A look this account is refused is its own answer and never absence,
//!  and a task whose number names no process any more belongs to no live
//!  session,
//!  so a holder in the caller's own session is the kernel's answer and never a
//!  guess from a list of tasks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const state = theStateOfAPath(registration);
//! const mine = theTasksInTheCallersSession(holders.map((holder) => holder.task));
//! ```

/// What stands at one path.
use super::handler_checks::PathState;
use std::path::Path;

/// What:
///  The state of one path,
///  as every handler check reads it.
/// Why:
///  Owner and mode are two answers a person acts on differently,
///  and a look this account was refused proves nothing about the path,
///  so a refused look is its own answer and never absence.
pub fn the_state_of_a_path(path: &Path) -> PathState {
    use std::os::unix::fs::MetadataExt as _;
    return match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => PathState::Absent,
        Err(error) => PathState::Unreadable {
            why: error.to_string(),
        },
        Ok(metadata) if metadata.is_dir() => PathState::Directory {
            owner: metadata.uid(),
            mode: metadata.mode() & 0o7777,
        },
        Ok(metadata) if metadata.is_file() => PathState::File {
            owner: metadata.uid(),
            mode: metadata.mode() & 0o7777,
        },
        Ok(_other) => PathState::WrongKind {
            found: String::from("a symbolic link or a device node"),
        },
    };
}

/// What:
///  Which of these tasks the caller's own session holds.
/// Why:
///  A session is what a removal spares besides the caller itself,
///  so a holder in it is told to leave the folder rather than ended,
///  and the kernel's own session number is the only proof of membership.
///  A task whose number names no process any more belongs to no live session,
///  and the holder is still named and still blocks the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mine = theTasksInTheCallersSession(holders.map((holder) => holder.task));
/// ```
pub fn the_tasks_in_the_callers_session(tasks: &[u32]) -> Vec<u32> {
    let ours: libc::pid_t = the_session_of(0);
    if ours == 0 {
        return Vec::new();
    }
    return tasks
        .iter()
        .filter(|task| return the_session_of(**task as libc::pid_t) == ours)
        .copied()
        .collect();
}

/// What:
///  The session one task belongs to,
///  or zero when the kernel could not answer.
/// Why:
///  A session is what a removal spares besides the caller itself,
///  and the kernel's own number is the only proof of membership,
///  so the answer is the kernel's and no list of tasks is read.
fn the_session_of(task: libc::pid_t) -> libc::pid_t {
    // SAFETY: `getsid` reads one process's session identifier and touches no
    // memory of ours; a process that has gone answers -1 with `ESRCH`, which
    // is no session and so belongs to none the caller holds.
    let answered: libc::pid_t = unsafe { libc::getsid(task) };
    if answered < 0 {
        return 0;
    }
    return answered;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_facts_live_reads_tests.rs"]
mod tests;
