//! What:
//!  The processes below one process:
//! found before it is stopped,
//! and signalled afterwards,
//! each only while it is still
//!  the process that was found.
//! Why:
//!  Stopping a process does not stop what it started.
//! A `git push` that is sent `SIGTERM` or `SIGKILL` leaves its
//!  transport helper (`git-remote-http`,
//! `ssh`) and the hooks it ran (`git-lfs`) alive,
//! orphaned to the init process,
//!  holding their connection until their own timeout ends them (measured:
//! a helper stayed 60 seconds,
//! a `git-lfs`
//!  that never exits stayed for good).
//! The tree has to be read before the root dies,
//! because a child's link to its
//!  parent is gone once the parent has exited.
//! A process number can be reused,
//! so each process is identified by the
//!  start time the kernel recorded for it,
//! and a number whose start time changed is not signalled.
//!  The list comes from `/proc`,
//! so only Linux finds any;
//! elsewhere the tree is empty and only the root is stopped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = await descendantsOf(child.pid); child.kill('SIGTERM'); for (const process of tree) process.signalIfSame('SIGTERM');
//! ```
//!
//! A look at a record answers three things,
//! not two:
//! the member is there and is the one that was found,
//! it is gone,
//! or its record could not be read.
//! The third is signalled and waited for like the first,
//! because a stop that reads a refusal as an exit ends nothing and sweeps the locks of a live tree.

/// What a signal sent through a pin came to.
use crate::pinned_member_signalled_through_a_pidfd::Delivered;

/// One process of a tree and the start time that proves it is still that process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeMember {
    /// The process number.
    pub pid: i32,
    /// The start time the kernel recorded,
    /// in clock ticks since boot (`/proc/<pid>/stat`,
    /// field 22).
    pub started: u64,
}

/// What:
///  The pid,
/// parent and start time of the process a `stat` file describes,
/// and whether it is a zombie.
/// Why:
///   The command name sits in parentheses and may hold spaces and parentheses,
/// so the fields are read after the last
///  closing parenthesis.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [state, ppid, ...rest] = text.slice(text.lastIndexOf(')') + 2).split(' ');
/// ```
#[cfg(target_os = "linux")]
fn parse_stat(text: &str) -> Option<(i32, u64, bool)> {
    let after_name: &str = text.get(text.rfind(')')? + 2..)?;
    let fields: Vec<&str> = after_name.split(' ').collect();
    // After the name: state (index 0), parent (1), ..., start time (19).
    let state: &str = fields.first()?;
    let parent: i32 = fields.get(1)?.parse().ok()?;
    let started: u64 = fields.get(19)?.parse().ok()?;
    return Some((parent, started, state == "Z"));
}

/// What:
///  Every living process below `root`,
/// children before grandchildren,
/// with their start times.
/// Why:
///   Read once,
/// before the root is signalled,
/// from one pass over `/proc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function descendantsOf(root: number): Promise<TreeMember[]>;
/// ```
#[cfg(target_os = "linux")]
pub fn descendants_of(root: i32) -> Vec<TreeMember> {
    use std::collections::HashMap;
    let Ok(listing) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    // Parent number to the children found under it.
    let mut children: HashMap<i32, Vec<TreeMember>> = HashMap::new();
    for entry in listing.flatten() {
        let file_name: std::ffi::OsString = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let Ok(pid) = name.parse::<i32>() else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        if let Some((parent, started, zombie)) = parse_stat(text.as_str())
            && !zombie
        {
            children.entry(parent).or_default().push(TreeMember { pid, started });
        }
    }
    let mut found: Vec<TreeMember> = Vec::new();
    let mut frontier: Vec<i32> = vec![root];
    while let Some(parent) = frontier.pop() {
        for member in children.remove(&parent).unwrap_or_default() {
            frontier.push(member.pid);
            found.push(member);
        }
    }
    return found;
}

/// Only Linux lists processes;
/// elsewhere no descendant is found and only the root is stopped.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn descendants_of(_root: i32) -> Vec<TreeMember> {
    return Vec::new();
}

/// What:
///  The root of a tree as a member:
/// its pid and the start time that proves later that it is still that process.
/// Why:
///   `descendants_of` lists only what is below a process,
/// and a stop must wait for the process itself too.
///  A process that has ended,
/// or is a zombie,
/// is no member.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function memberOf(pid: number): Promise<TreeMember | undefined>;
/// ```
#[cfg(target_os = "linux")]
pub fn member_of(pid: i32) -> Option<TreeMember> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let (_parent, started, zombie) = parse_stat(text.as_str())?;
    if zombie {
        return None;
    }
    return Some(TreeMember { pid, started });
}

/// Only Linux reads a process's start time;
/// elsewhere no member is found.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn member_of(_pid: i32) -> Option<TreeMember> {
    return None;
}

/// Whether the process is still the one that was found:
/// it exists,
/// is not a zombie,
/// and has the recorded start time.
///  A record that cannot be read says nothing about the process,
/// so it is waited for and signalled as if it were alive:
/// reading a refusal as an exit ends nothing and lets a stop's lock sweep run against a live tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = lookOf(member.pid); return look.kind !== 'gone' && (look.kind !== 'alive' || look.started === member.started);
/// ```
#[cfg(target_os = "linux")]
pub fn still_running(member: &TreeMember) -> bool {
    return still_running_when(look_of(member.pid), member);
}

/// What:
///  What a look at a record means for the member that was found:
/// it is still that process or nothing proves it ended.
/// Why:
///   The rule both the wait and the signal share,
/// so an unreadable record is never taken for an exit in one of them and not in
///  the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillRunningWhen = (look, member) => look.kind === 'unknown' || (look.kind === 'alive' && look.member.started === member.started);
/// ```
pub fn still_running_when(look: ProcessLook, member: &TreeMember) -> bool {
    return match look {
        ProcessLook::Alive(found) => found.started == member.started,
        ProcessLook::Gone => false,
        ProcessLook::Unknown => true,
    };
}

/// What:
///  What a look at one process's record came to.
/// Why:
///   A process number can name a live process,
/// nothing at all,
/// or a record that could not be read (a
///  descriptor limit,
/// a permission,
/// an I/O error,
/// a record that could not be parsed).
/// The third is not the second:
///  the step is signalled and waited for on an unknown record,
/// because a stop that treats a refusal as an exit ends nothing
///  and sweeps the locks of a live tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProcessLook = { kind: 'alive'; member: Member } | { kind: 'gone' } | { kind: 'unknown' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessLook {
    /// The record was read and names a process that has not exited.
    Alive(TreeMember),
    /// The record is not there,
    /// or names a zombie:
    /// the process has ended.
    Gone,
    /// The record could not be read or understood for a reason that is not the process's end,
    /// so nothing is known about it.
    Unknown,
}

/// The error kinds that mean the process ended while it was being looked at,
/// never a refusal.
#[cfg(target_os = "linux")]
fn exited(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH);
}

/// What:
///  What a look at the record of process `pid` below `proc_root` came to.
/// Why:
///   The record's own directory is the seam a control uses to read a record that refuses,
/// so a refusal is
///  proven without exhausting this process's descriptors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lookOfThrough(procRoot: string, pid: number): ProcessLook;
/// ```
#[cfg(target_os = "linux")]
pub fn look_of_through(proc_root: &std::path::Path, pid: i32) -> ProcessLook {
    let text: String = match std::fs::read_to_string(proc_root.join(pid.to_string()).join("stat")) {
        Ok(text) => text,
        Err(error) if exited(&error) => return ProcessLook::Gone,
        // A refusal, an I/O error or a record that was not there for the process's end: nothing is known.
        Err(_unreadable) => return ProcessLook::Unknown,
    };
    let Some((_parent, started, zombie)) = parse_stat(text.as_str()) else {
        // A record that cannot be understood is not proof that the process ended.
        return ProcessLook::Unknown;
    };
    if zombie {
        return ProcessLook::Gone;
    }
    return ProcessLook::Alive(TreeMember { pid, started });
}

/// The root of the process table on Linux,
/// where a record is read from.
#[cfg(target_os = "linux")]
pub(crate) const PROC: &str = "/proc";

/// What a look at the record of process `pid` came to.
#[cfg(target_os = "linux")]
pub fn look_of(pid: i32) -> ProcessLook {
    return look_of_through(std::path::Path::new(PROC), pid);
}

/// Only Linux reads a process's record;
/// elsewhere nothing is known about a process,
/// so a stop signals its root without an identity check,
/// as it always did.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn look_of(_pid: i32) -> ProcessLook {
    return ProcessLook::Unknown;
}

/// What:
///  Send `signal` to each member that is still the process that was found;
/// the others are left alone.
/// Why:
///   A number that was reused by an unrelated process must never be signalled.
///  A member whose record could not be read is signalled:
/// the number was a member of the tree a moment
///  ago,
/// and a stop that sends nothing because a look was refused is a stop that ends nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const member of tree) signalMember(member, signal); // through its pidfd
/// ```
#[cfg(unix)]
pub fn signal_members(members: &[TreeMember], signal: i32) {
    for member in members {
        // The member is held by a pidfd where the kernel gives one, so the signal reaches that process and never a number
        //  that was reused between the look and the call
        //  (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`, finding 7).
        let _delivered: Delivered = crate::pinned_member_signalled_through_a_pidfd::signal_member(member, signal);
    }
}

/// What:
///  Whether the process is still the one that was found and is in uninterruptible sleep (state `D` in `/proc/<pid>/stat`).
/// Why:
///   A process in that state,
/// held by a stalled disk call or a processor it is not given,
/// cannot act on a signal until the call
///  returns,
/// and a `SIGKILL` then only cuts what follows it:
/// Git's handler removes a lock's pid file and then the lock,
/// and a kill
///  between the two leaves a lock nothing proves stale.
/// A stop therefore waits while a member is in this state,
/// up to its ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// member.stillRunning() && readState(member.pid) === 'D'
/// ```
#[cfg(target_os = "linux")]
pub fn in_uninterruptible_sleep(member: &TreeMember) -> bool {
    return state_letter_of(member) == Some('D');
}

/// Only Linux reads a process's state;
/// elsewhere nothing is known to be in uninterruptible sleep.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn in_uninterruptible_sleep(_member: &TreeMember) -> bool {
    return false;
}

/// The state letter of the process (`R`,
/// `S`,
/// `D`,
/// `T`,
/// `t`,
/// ...) while it is still the one that was found,
/// and nothing otherwise.
#[cfg(target_os = "linux")]
pub fn state_letter_of(member: &TreeMember) -> Option<char> {
    let text: String = std::fs::read_to_string(format!("/proc/{}/stat", member.pid)).ok()?;
    let (_parent, started, zombie) = parse_stat(text.as_str())?;
    if started != member.started || zombie {
        return None;
    }
    return text.get(text.rfind(')')? + 2..)?.chars().next();
}

/// Nothing is found elsewhere,
/// so nothing is still running.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn still_running(_member: &TreeMember) -> bool {
    return false;
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn signal_members(_members: &[TreeMember], _signal: i32) {}

/// Ask each member that is still the process that was found to end,
/// with `SIGTERM`.
#[cfg(unix)]
pub fn terminate_members(members: &[TreeMember]) {
    signal_members(members, libc::SIGTERM);
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn terminate_members(_members: &[TreeMember]) {}

/// End each member that is still the process that was found,
/// with `SIGKILL`.
#[cfg(unix)]
pub fn kill_members(members: &[TreeMember]) {
    signal_members(members, libc::SIGKILL);
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn kill_members(_members: &[TreeMember]) {}

/// Process-tree controls stay out of the release executable.
#[cfg(test)]
#[path = "process_tree_tests.rs"]
mod tests;

/// Controls for platforms without `/proc`,
/// whose branches the Linux build does not compile.
#[cfg(all(test, not(target_os = "linux")))]
#[path = "process_tree_other_tests.rs"]
mod other_tests;
