//! Collects the group of a matched process,
//! the processes that move with it,
//! and checks the group again right before the move.
//! The candidates are the members of the process's cgroup,
//! read from its `cgroup.procs`,
//! and the group is those that descend from the matched process and run as the watched user.

/// The processes of the cgroup,
/// and the facts about one process.
use crate::process_info::{list_pids, read_cgroup_pids, read_process, ProcessInfo};
/// The setup the engine was built with.
use crate::update_routing::RoutingSetup;

/// Reports whether a collected group already holds `pid`.
fn contains_pid(group: &[(i32, u64)], pid: i32) -> bool {
    for (member, _start_time) in group {
        if *member == pid {
            return true;
        }
    }
    return false;
}

/// Collects the live descendants of `root` that run as the watched user in the same cgroup,
/// with their start times,
/// and the number of processes of this user that could not be inspected.
/// The candidates are the members of the cgroup,
/// read from its `cgroup.procs`,
/// because a pass over every process of the host cost about 26 ms of CPU for each move.
/// A cgroup whose member list cannot be read falls back to the pass over every process.
pub fn collect_group(setup: &RoutingSetup, root: &ProcessInfo) -> Result<(Vec<(i32, u64)>, usize), String> {
    let pids = match read_cgroup_pids(&setup.cgroup_root, &root.cgroup) {
        Some(members) => members,
        None => match list_pids(&setup.proc_root) {
            Ok(pids) => pids,
            Err(error) => return Err(format!("listing the processes failed: {error}")),
        },
    };
    let mut snapshot: Vec<ProcessInfo> = Vec::new();
    let mut unreadable = 0;
    for pid in pids {
        match read_process(&setup.proc_root, pid, setup.uid) {
            Ok(Some(info)) => snapshot.push(info),
            Ok(None) => {}
            Err(_error) => unreadable += 1,
        }
    }
    let mut found: Vec<(i32, u64)> = vec![(root.pid, root.start_time)];
    let mut index = 0;
    while index < found.len() {
        let parent = found[index].0;
        index += 1;
        for candidate in &snapshot {
            if candidate.parent == parent && candidate.cgroup == root.cgroup && !contains_pid(&found, candidate.pid) {
                found.push((candidate.pid, candidate.start_time));
            }
        }
    }
    return Ok((found, unreadable));
}

/// Keeps the processes that are still the same ones,
/// and counts the rest.
/// A pid that exited and was reused has another start time and is dropped.
pub fn recheck_group(setup: &RoutingSetup, group: &[(i32, u64)], cgroup: &str) -> (Vec<i32>, usize) {
    let mut kept: Vec<i32> = Vec::new();
    let mut dropped = 0;
    for (pid, start_time) in group {
        match read_process(&setup.proc_root, *pid, setup.uid) {
            Ok(Some(again)) if again.start_time == *start_time && again.cgroup == cgroup => kept.push(*pid),
            _ => dropped += 1,
        }
    }
    return (kept, dropped);
}
