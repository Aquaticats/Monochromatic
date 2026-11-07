//! What:
//!  How many workers a set of independent items may use together:
//! the CPUs the process is allowed minus a margin,
//! the memory
//!  that is free,
//! an optional ceiling the shared storage was measured to need,
//! and the person's own cap.
//! Why:
//!  Real jobs may use almost the whole host,
//! so a bare constant is wrong in both directions:
//! too low on a large host,
//! too high
//!  inside a container with a quota or on a host whose memory other sessions already hold.
//! The budget is derived from the
//!  facts of the moment,
//! and the facts are a value so a control can state them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const workers = workerBudget({ items: 48, perItemMemory: 256 * MiB, ceiling: undefined, cap: undefined, host: readHost() });
//! ```

/// The smallest number of entries at which creating them on several threads beat creating them on one,
/// for the three places that
///  make files by cloning:
/// 16 entries.
/// Measured with the copy stage (`copy_manifest_entries`),
/// the install (`install_snapshot`
///  with a recorder that syncs every batch) and `clone_tree`,
/// each forced to 1,
/// 2,
/// 4 and 8 workers on reflinked trees of this
///  repository (2026-11-03,
/// medians of five runs in alternating orders,
/// other sessions running,
/// a debug build):
/// `clone_tree` took
///  0.6 ms on one thread and 0.5 to 0.6 ms on 2,
/// 4 and 8 for 10 entries,
/// and 1.8 ms against 1.2 ms for 30;
/// the copy stage 1.1 ms
///  against 0.7 ms for 30,
/// 3.6 ms against 1.9 ms for 90,
/// 397.6 ms against 104.4 ms for 9,301;
/// the install 1.6 ms against 1.3 ms for
///  30,
/// 14.8 ms against 8.9 ms for 296,
/// 500.2 ms against 244.9 ms for 9,301.
/// The parallel path was never slower from 16 entries up.
///  The coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
///  "Picked for these follow-ups",
/// thresholds of the copy stage and the install,
/// option A).
pub const PARALLEL_BREAK_EVEN_ENTRIES: usize = 16;

/// The share of the CPUs the process is allowed that stays free:
/// one in this many,
/// at least one when there are more than two.
///  The owner's words are "make sure we're running everything that can be run in parallel in parallel" (handover
///  `doc/handover/cli-git-rust-implementation.md`,
/// lines 864 and 865),
/// which leaves a margin to a person's other work.
/// On this
///  28-thread host the margin leaves 25 workers,
/// and the measured ceilings are 8 for file clones and 16 for conversions,
/// so the
///  divisor does not bind here;
/// it binds on hosts of 8 threads or fewer.
/// A share cannot be measured as a quantity,
/// so the value
///  is the coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Parallelism caps and memory estimates").
const CPU_MARGIN_DIVISOR: usize = 8;

/// The fraction of the memory that is free which the items may use together:
/// one over this many.
/// The rest stays for the
///  operating system's caches and for the other sessions of the host.
/// With 49 GB available on this host and the estimates below
///  (128 MiB at most per item) it admits more than 180 items at once,
/// so it binds only on hosts of a few gigabytes.
/// A share
///  cannot be measured as a quantity:
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
const MEMORY_SHARE_DIVISOR: u64 = 2;

/// What:
///  The facts a budget is derived from.
/// Why:
///   Reading the host is one place and the arithmetic another,
/// so a control states a host of any size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HostResources = { parallelism: number; memoryAvailable?: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostResources {
    /// The threads the process may run at once (a container's CPU quota included).
    pub parallelism: usize,
    /// The bytes that can still be given out without pushing others out,
    /// or nothing when the host does not say.
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
    /// How many independent items there are;
    /// more workers than items would idle.
    pub items: usize,
    /// The bytes one item holds at its peak,
    /// a measured figure of the caller.
    pub per_item_memory: u64,
    /// The most workers the shared storage was measured to use usefully,
    /// or nothing when it was not measured to flatten.
    pub ceiling: Option<usize>,
    /// The most workers the person allows (`--jobs`),
    /// or nothing.
    pub cap: Option<usize>,
    /// The facts of this host now.
    pub host: &'host HostResources,
}

impl HostResources {
    /// What:
    ///  The resources of this process now:
    /// the threads it may run,
    /// and the least of the system's available memory and the
    ///  room left in its control group and the groups above it.
    /// Why:
    ///   The memory figure that matters is the one the kernel would enforce,
    /// which a quota can make smaller than the host's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const host = HostResources.read();
    /// ```
    pub fn read() -> HostResources {
        let parallelism: usize = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        return HostResources { parallelism, memory_available: memory_available_now() };
    }
}

/// What:
///  The bytes that can still be given out:
/// the smaller of the system's available memory and the room left in the control group
///  this process runs in and the groups above it,
/// from the published `sysinfo` crate;
/// nothing when the crate cannot start.
/// Why:
///  The owner's ruling of 2026-11-07 ("Memory headroom:
/// sysinfo") replaces the wrapper's own readers of `/proc/meminfo` and of the
///  control groups' `memory.max` and `memory.current`.
/// The crate's cgroup reader takes the least headroom up the hierarchy,
/// as the
///  hand-written walk did;
/// the two agreed within a page under two `MemoryMax` caps
///  (`doc/research/wrapper-git-free-jobs-and-published-crates.md`,
/// "Parallel work within a budget").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const available = Math.min(os.freemem(), cgroupHeadroom());
/// ```
fn memory_available_now() -> Option<u64> {
    let mut system: sysinfo::System = sysinfo::System::new().ok()?;
    system.refresh_memory_specifics(sysinfo::MemoryRefreshKind::nothing().with_ram());
    let by_system: u64 = system.available_memory();
    return Some(match system.cgroup_limits() {
        Some(group) => by_system.min(group.free_memory),
        None => by_system,
    });
}

/// The threads left free for everything else:
/// an eighth of them,
/// at least one,
/// and none when the process may run only two.
pub fn cpu_margin(parallelism: usize) -> usize {
    if parallelism <= 2 {
        return 0;
    }
    return (parallelism / CPU_MARGIN_DIVISOR).max(1);
}

/// What:
///  The number of workers for independent items.
/// Why:
///   The least of the items,
/// the CPUs minus the margin,
/// the memory that is free over what one item holds,
/// the ceiling the
///  storage was measured to use,
/// and the person's cap;
/// never fewer than one,
/// so a starved host still makes progress.
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

/// The bytes one slice of a file-copy or file-removal job holds at its peak:
/// a batch of entries and the handles it opens,
/// estimated
///  at sixteen mebibytes.
///  The estimate was checked against `clone_tree`:
/// its resident size grew by 1.9 MiB over a pre-clone level of 21.6 MiB (peak
///  24,080 KiB) for 22,718 entries at 16 workers,
/// so a slice holds far less than sixteen mebibytes.
/// The estimate is kept as a
///  conservative upper bound because the copy stage itself was not measured;
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const SLICE_MEMORY: u64 = 16 << 20;

/// What:
///  The workers for file-tree jobs that split their entries into slices:
/// the host's budget up to a ceiling the filesystem was
///  measured or chosen to use.
/// Why:
///   The copy,
/// install and removal of a worktree's files share the filesystem's metadata locks,
/// so a ceiling stays,
/// but the CPUs
///  left free for other sessions and the memory of the control group bound them too,
/// which a bare `min(available, 8)` ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const workers = slicedWorkers({ items, ceiling: 8, host });
/// ```
pub fn sliced_workers(items: usize, ceiling: usize, host: &HostResources) -> usize {
    return worker_budget(&BudgetRequest { items, per_item_memory: SLICE_MEMORY, ceiling: Some(ceiling), cap: None, host });
}

/// Budget controls stay out of the release executable.
#[cfg(test)]
#[path = "parallel_budget_tests.rs"]
mod tests;
