//! What:
//!  How many workers a set of independent items may use together: the CPUs the process is allowed minus a margin, the memory
//!  that is free, an optional ceiling the shared storage was measured to need, and the person's own cap.
//! Why:
//!  Real jobs may use almost the whole host, so a bare constant is wrong in both directions: too low on a large host, too high
//!  inside a container with a quota or on a host whose memory other sessions already hold. The budget is derived from the
//!  facts of the moment, and the facts are a value so a control can state them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const workers = workerBudget({ items: 48, perItemMemory: 256 * MiB, ceiling: undefined, cap: undefined, host: readHost() });
//! ```

use std::path::{Path, PathBuf};

/// The share of the CPUs the process is allowed that stays free: one in this many, at least one when there are more than two.
const CPU_MARGIN_DIVISOR: usize = 8;

/// The fraction of the memory that is free which the items may use together: one over this many. The rest stays for the
///  operating system's caches and for the other sessions of the host.
const MEMORY_SHARE_DIVISOR: u64 = 2;

/// Where the kernel lists the memory it can still give out.
const MEMORY_INFORMATION: &str = "/proc/meminfo";

/// Where the kernel names the control group of the process.
const CONTROL_GROUP_NAME: &str = "/proc/self/cgroup";

/// The root of the unified control-group hierarchy.
const CONTROL_GROUP_ROOT: &str = "/sys/fs/cgroup";

/// What:
///  The facts a budget is derived from.
/// Why:
///   Reading the host is one place and the arithmetic another, so a control states a host of any size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HostResources = { parallelism: number; memoryAvailable?: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostResources {
    /// The threads the process may run at once (a container's CPU quota included).
    pub parallelism: usize,
    /// The bytes that can still be given out without pushing others out, or nothing when the host does not say.
    pub memory_available: Option<u64>,
}

/// What:
///  What one budget asks for.
/// Why:
///   One value keeps the call sites short and names every input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BudgetRequest = { items: number; perItemMemory: number; ceiling?: number; cap?: number; host: HostResources };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct BudgetRequest<'host> {
    /// How many independent items there are; more workers than items would idle.
    pub items: usize,
    /// The bytes one item holds at its peak, a measured figure of the caller.
    pub per_item_memory: u64,
    /// The most workers the shared storage was measured to use usefully, or nothing when it was not measured to flatten.
    pub ceiling: Option<usize>,
    /// The most workers the person allows (`--jobs`), or nothing.
    pub cap: Option<usize>,
    /// The facts of this host now.
    pub host: &'host HostResources,
}

impl HostResources {
    /// What:
    ///  The resources of this process now: the threads it may run, and the least of the system's available memory and the
    ///  room left in its control group and the groups above it.
    /// Why:
    ///   The memory figure that matters is the one the kernel would enforce, which a quota can make smaller than the host's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const host = HostResources.read();
    /// ```
    pub fn read() -> HostResources {
        let parallelism: usize = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        let system: Option<u64> = std::fs::read_to_string(MEMORY_INFORMATION).ok().and_then(|text: String| { return memory_available_from(text.as_str()); });
        let group: Option<u64> = std::fs::read_to_string(CONTROL_GROUP_NAME)
            .ok()
            .and_then(|text: String| { return control_group_path(text.as_str()); })
            .and_then(|path: String| { return group_headroom(Path::new(CONTROL_GROUP_ROOT), path.as_str()); });
        let memory_available: Option<u64> = match (system, group) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (left, right) => left.or(right),
        };
        return HostResources { parallelism, memory_available };
    }
}

/// The `MemAvailable` figure of `/proc/meminfo` text, in bytes.
pub fn memory_available_from(text: &str) -> Option<u64> {
    let line: &str = text.lines().find(|candidate: &&str| { return candidate.starts_with("MemAvailable:"); })?;
    let kibibytes: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    return kibibytes.checked_mul(1024);
}

/// The path of the unified (version 2) control group in `/proc/self/cgroup` text: the line that starts with `0::`.
pub fn control_group_path(text: &str) -> Option<String> {
    return text.lines().find_map(|line: &str| { return line.strip_prefix("0::").map(String::from); });
}

/// The bytes a control group can still take under its limit: `max` minus `current`, nothing when there is no numeric limit.
pub fn headroom_from(max: &str, current: &str) -> Option<u64> {
    let limit: u64 = max.trim().parse().ok()?;
    let used: u64 = current.trim().parse().ok()?;
    return Some(limit.saturating_sub(used));
}

/// The least headroom of the group at `path` below `root` and of every group above it, nothing when none has a limit.
fn group_headroom(root: &Path, path: &str) -> Option<u64> {
    let mut least: Option<u64> = None;
    let mut directory: PathBuf = root.join(path.trim_start_matches('/'));
    loop {
        let limit: Option<String> = std::fs::read_to_string(directory.join("memory.max")).ok();
        let current: Option<String> = std::fs::read_to_string(directory.join("memory.current")).ok();
        if let (Some(max_text), Some(current_text)) = (limit, current)
            && let Some(room) = headroom_from(max_text.as_str(), current_text.as_str())
        {
            least = Some(least.map_or(room, |known: u64| { return known.min(room); }));
        }
        if directory == root || !directory.pop() || !directory.starts_with(root) {
            break;
        }
    }
    return least;
}

/// The threads left free for everything else: an eighth of them, at least one, and none when the process may run only two.
pub fn cpu_margin(parallelism: usize) -> usize {
    if parallelism <= 2 {
        return 0;
    }
    return (parallelism / CPU_MARGIN_DIVISOR).max(1);
}

/// What:
///  The number of workers for independent items.
/// Why:
///   The least of the items, the CPUs minus the margin, the memory that is free over what one item holds, the ceiling the
///  storage was measured to use, and the person's cap; never fewer than one, so a starved host still makes progress.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const workers = Math.max(1, Math.min(items, cpus - margin, memoryShare / perItem, ceiling, cap));
/// ```
pub fn worker_budget(request: &BudgetRequest<'_>) -> usize {
    let by_cpu: usize = request.host.parallelism.saturating_sub(cpu_margin(request.host.parallelism)).max(1);
    let by_memory: usize = match request.host.memory_available {
        Some(bytes) => usize::try_from((bytes / MEMORY_SHARE_DIVISOR) / request.per_item_memory.max(1)).unwrap_or(usize::MAX),
        None => usize::MAX,
    };
    return request
        .items
        .min(by_cpu)
        .min(by_memory)
        .min(request.ceiling.unwrap_or(usize::MAX))
        .min(request.cap.unwrap_or(usize::MAX))
        .max(1);
}

/// Budget controls stay out of the release executable.
#[cfg(test)]
#[path = "parallel_budget_tests.rs"]
mod tests;
