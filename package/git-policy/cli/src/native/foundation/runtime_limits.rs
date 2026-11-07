//! What:
//!  The limits the system puts on a process that starts threads: how much address space is left under `RLIMIT_AS`, and how
//!  many more threads fit under `RLIMIT_NPROC` and under the `pids.max` of its control group and every group above it.
//! Why:
//!  A multi-thread runtime panics, and can then hang, when the system refuses it a thread or address space
//!  (`doc/planning/git-policy-cli-full-tokio.md`, "A multi-thread runtime panics, then hangs"). Reading these limits before the
//!  build lowers the chance of that failure; the deadline-bounded build in `lazy_runtime.rs` is what bounds the wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const limits = RuntimeLimits.read(); // { addressSpaceLeft?: number; threadsLeft?: number }
//! ```

use crate::parallel_budget::control_group_path;
use std::path::{Path, PathBuf};

/// Where the kernel lists the size of this process's mappings and its thread count.
const SELF_STATUS: &str = "/proc/self/status";

/// Where the kernel lists, as the fourth field, how many threads and processes exist on the whole system.
const SYSTEM_LOAD: &str = "/proc/loadavg";

/// What:
///  What is left of each limit, nothing for a limit that is not set.
/// Why:
///  The runtime plan takes the least of what the budget allows and what these allow, so an unlimited resource must not
///  bound it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuntimeLimits = { addressSpaceLeft?: number; threadsLeft?: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeLimits {
    /// Bytes of address space the process may still map, or nothing when `RLIMIT_AS` is unlimited.
    pub address_space_left: Option<u64>,
    /// Threads the process may still start, or nothing when neither `RLIMIT_NPROC` nor a `pids.max` is set.
    pub threads_left: Option<u64>,
}

/// The size of the mappings, in bytes, from the `VmSize` line of `/proc/<pid>/status` text.
pub fn mapped_bytes_from(status: &str) -> Option<u64> {
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmSize:") {
            let kibibytes: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return kibibytes.checked_mul(1024);
        }
    }
    return None;
}

/// The threads and processes that exist on the system, from the fourth field of `/proc/loadavg` text (`running/total`).
///  It counts every user's threads, so it over-counts this user's; an over-count only lowers the number of workers.
pub fn system_threads_from(load: &str) -> Option<u64> {
    let field: &str = load.split_whitespace().nth(3)?;
    let (_running, total) = field.split_once('/')?;
    return total.parse().ok();
}

/// What is left of a limit that is `limit` (nothing when unlimited) with `used` already taken.
pub fn left_under(limit: Option<u64>, used: u64) -> Option<u64> {
    return limit.map(|bound| { return bound.saturating_sub(used); });
}

/// The threads a control group can still hold: its `pids.max` minus its `pids.current`, nothing when `pids.max` is `max`.
pub fn group_threads_left_from(max: &str, current: &str) -> Option<u64> {
    let limit: u64 = max.trim().parse().ok()?;
    let used: u64 = current.trim().parse().ok()?;
    return Some(limit.saturating_sub(used));
}

/// The least of two optional figures where nothing means unlimited.
pub fn least_of(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    return match (left, right) {
        (Some(first), Some(second)) => Some(first.min(second)),
        (first, second) => first.or(second),
    };
}

/// The threads left under the group at `path` below `root` and every group above it, nothing when none has a limit.
pub fn group_threads_left(root: &Path, path: &str) -> Option<u64> {
    let mut least: Option<u64> = None;
    let mut directory: PathBuf = root.join(path.trim_start_matches('/'));
    loop {
        let limit: Option<String> = std::fs::read_to_string(directory.join("pids.max")).ok();
        let current: Option<String> = std::fs::read_to_string(directory.join("pids.current")).ok();
        if let (Some(max_text), Some(current_text)) = (limit, current) {
            least = least_of(least, group_threads_left_from(max_text.as_str(), current_text.as_str()));
        }
        if directory == root || !directory.pop() || !directory.starts_with(root) {
            break;
        }
    }
    return least;
}

/// The root of the unified control-group hierarchy.
const CONTROL_GROUP_ROOT: &str = "/sys/fs/cgroup";

impl RuntimeLimits {
    /// What:
    ///  The limits of this process now.
    /// Why:
    ///  The build of a runtime needs them as one value; each source that cannot be read gives no bound, as the budget's own
    ///  reader does.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const limits = RuntimeLimits.read();
    /// ```
    #[cfg(unix)]
    pub fn read() -> RuntimeLimits {
        let address_space: rustix::process::Rlimit = rustix::process::getrlimit(rustix::process::Resource::As);
        let mapped: u64 = std::fs::read_to_string(SELF_STATUS)
            .ok()
            .and_then(|text| { return mapped_bytes_from(text.as_str()); })
            .unwrap_or(0);
        let address_space_left: Option<u64> = left_under(address_space.current, mapped);
        // Counting the user's threads is a scan of every process, so one read of the system total stands in for it.
        let processes: rustix::process::Rlimit = rustix::process::getrlimit(rustix::process::Resource::Nproc);
        let system_threads: u64 = std::fs::read_to_string(SYSTEM_LOAD)
            .ok()
            .and_then(|text| { return system_threads_from(text.as_str()); })
            .unwrap_or(0);
        let by_process_limit: Option<u64> = left_under(processes.current, system_threads);
        let by_group: Option<u64> = std::fs::read_to_string("/proc/self/cgroup")
            .ok()
            .and_then(|text| { return control_group_path(text.as_str()); })
            .and_then(|path| { return group_threads_left(Path::new(CONTROL_GROUP_ROOT), path.as_str()); });
        return RuntimeLimits { address_space_left, threads_left: least_of(by_process_limit, by_group) };
    }
}

/// Limit controls stay out of the release executable.
#[cfg(test)]
#[path = "runtime_limits_tests.rs"]
mod tests;
