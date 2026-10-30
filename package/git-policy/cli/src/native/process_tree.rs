//! What:
//!  The processes below one process: found before it is stopped, and signalled afterwards, each only while it is still
//!  the process that was found.
//! Why:
//!  Stopping a process does not stop what it started. A `git push` that is sent `SIGTERM` or `SIGKILL` leaves its
//!  transport helper (`git-remote-http`, `ssh`) and the hooks it ran (`git-lfs`) alive, orphaned to the init process,
//!  holding their connection until their own timeout ends them (measured: a helper stayed 60 seconds, a `git-lfs`
//!  that never exits stayed for good). The tree has to be read before the root dies, because a child's link to its
//!  parent is gone once the parent has exited. A process number can be reused, so each process is identified by the
//!  start time the kernel recorded for it, and a number whose start time changed is not signalled.
//!  The list comes from `/proc`, so only Linux finds any; elsewhere the tree is empty and only the root is stopped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = await descendantsOf(child.pid); child.kill('SIGTERM'); for (const process of tree) process.signalIfSame('SIGTERM');
//! ```

/// One process of a tree and the start time that proves it is still that process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeMember {
    /// The process number.
    pub pid: i32,
    /// The start time the kernel recorded, in clock ticks since boot (`/proc/<pid>/stat`, field 22).
    pub started: u64,
}

/// What:
///  The pid, parent and start time of the process a `stat` file describes, and whether it is a zombie.
/// Why:
///   The command name sits in parentheses and may hold spaces and parentheses, so the fields are read after the last
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
///  Every living process below `root`, children before grandchildren, with their start times.
/// Why:
///   Read once, before the root is signalled, from one pass over `/proc`.
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
        let Some(pid) = entry.file_name().to_str().and_then(|name| { return name.parse::<i32>().ok(); }) else {
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

/// Only Linux lists processes; elsewhere no descendant is found and only the root is stopped.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn descendants_of(_root: i32) -> Vec<TreeMember> {
    return Vec::new();
}

/// Whether the process is still the one that was found: it exists, is not a zombie, and has the recorded start time.
#[cfg(target_os = "linux")]
pub fn still_running(member: &TreeMember) -> bool {
    let Ok(text) = std::fs::read_to_string(format!("/proc/{}/stat", member.pid)) else {
        return false;
    };
    return parse_stat(text.as_str()).is_some_and(|(_parent, started, zombie)| { return started == member.started && !zombie; });
}

/// Nothing is found elsewhere, so nothing is still running.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn still_running(_member: &TreeMember) -> bool {
    return false;
}

/// What:
///  Send `signal` to each member that is still the process that was found; the others are left alone.
/// Why:
///   A number that was reused by an unrelated process must never be signalled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const member of tree) if (member.stillRunning()) process.kill(member.pid, signal);
/// ```
#[cfg(unix)]
pub fn signal_members(members: &[TreeMember], signal: i32) {
    for member in members {
        if still_running(member) {
            // SAFETY: `kill` takes no pointers; a failure is ignored because the process may have ended since the check.
            unsafe { libc::kill(member.pid, signal) };
        }
    }
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn signal_members(_members: &[TreeMember], _signal: i32) {}

/// Ask each member that is still the process that was found to end, with `SIGTERM`.
#[cfg(unix)]
pub fn terminate_members(members: &[TreeMember]) {
    signal_members(members, libc::SIGTERM);
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn terminate_members(_members: &[TreeMember]) {}

/// End each member that is still the process that was found, with `SIGKILL`.
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
