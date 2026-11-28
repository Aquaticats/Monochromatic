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
    /// or nothing when this machine's reading failed.
    ///  Nothing lifts the memory bound rather than tightening it:
    /// `worker_budget` then applies no bound of its own,
    /// while a failed thread count answers one thread.
    ///  No record decides which side the memory failure belongs on,
    /// and a bound there would hold every batch of every caller to one item at a time;
    /// the question and what each answer costs wait in `doc/handover/questions-for-the-owner.md`.
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
    ///  The thread count fails closed and the memory figure fails open:
    /// a count this process cannot read answers one thread,
    /// and a memory reading that failed answers nothing,
    /// which the budget reads as no memory bound at all.
    ///  That difference is no decision of this file's
    /// (`doc/handover/questions-for-the-owner.md`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const host = HostResources.read();
    /// ```
    pub fn read() -> HostResources {
        let parallelism: usize =
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        return HostResources {
            parallelism,
            memory_available: memory_available_now(),
        };
    }
}

/// What:
///  Where the CPUs and memory one batch of independent work is sized by come from:
///  a host the caller states,
///  or the machine itself,
///  read as that batch is sized.
/// Why:
///  A control states a host,
///  so its answer is the same on every machine and inside every capped scope
///  (issue 6567:
///  a build's page cache charged to a capped group read no headroom,
///  and every batch then ran one item at a
///  time).
///  A shipped run states the machine instead:
///  a reading taken before earlier batches ran can be minutes old,
///  and a batch of four items that each hold `SLOW_ITEM_MEMORY` bytes starts four of them on a host that has since
///  lost that room,
///  which asks the kernel to kill one of them.
///  The batch's own reading is taken where the width is decided,
///  so the figure is the room the host had when the batch started.
///  The width is decided once for a batch,
///  and every wave of it runs on that figure:
///  the reading belongs to the start of a batch and to no wave after it,
///  which is the staleness a batch of trashes that takes minutes still carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheHostOfABatch = { kind: 'stated'; host: HostResources } | { kind: 'theMachinesOwn' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TheHostOfABatch {
    /// A host the caller states,
    /// so a control's answer does not depend on this machine.
    Stated(HostResources),
    /// This machine,
    /// read when the batch is sized.
    TheMachinesOwn,
}

/// What:
///  The host the batch being sized is measured against:
///  the stated host itself,
///  or the machine read now.
/// Why:
///  One call is what every width decision makes,
///  so the reading cannot be taken somewhere else and carried to the decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theHost.now() // the stated host, or readHost()
/// ```
impl TheHostOfABatch {
    /// The host to size this batch by,
    /// as the source of this value says.
    pub fn now(&self) -> HostResources {
        return match self {
            TheHostOfABatch::Stated(host) => *host,
            TheHostOfABatch::TheMachinesOwn => {
                // What: a control's own count of the readings of the machine this seam took,
                //  so a control can prove a batch reads the machine where it is sized.
                //  It is the whole process's, so the control that reads it runs its body in a child of its own.
                // Why:  the count is how a control tells one reading for a round from one reading for each batch.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (kind === 'theMachinesOwn') readsForAControl += 1;
                // ```
                #[cfg(test)]
                {
                    let _counted: usize = MACHINE_READS_FOR_A_CONTROL
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
                HostResources::read()
            }
        };
    }
}

/// How many times a batch of this process took the machine's own reading,
/// for the controls that prove each batch reads
/// the machine where it is sized.
///  It is the whole process's,
///  so a control that reads it runs its body in a child of its own
/// (`testing/process_wide_state_child_test_support.rs`).
#[cfg(test)]
static MACHINE_READS_FOR_A_CONTROL: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// What:
///  How many batches of this process have taken the machine's own reading.
/// Why:
///  A control proves the machines is read at each batch,
///  and not once for a round,
///  by counting these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// machineReadsForAControl() // the count so far
/// ```
#[cfg(test)]
pub(crate) fn machine_reads_for_a_control() -> usize {
    return MACHINE_READS_FOR_A_CONTROL.load(std::sync::atomic::Ordering::SeqCst);
}

/// What:
///  The bytes that can still be given out:
/// the smaller of the system's available memory and the room left in the control group
///  this process runs in and the groups above it,
/// from the published `sysinfo` crate;
/// nothing when the crate cannot start.
///  Nothing is what a failed reading answers,
///  and `worker_budget` reads it as no memory bound:
///  the failure lifts the bound instead of tightening it,
///  which the field's own documentation says and no record decides.
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
    let by_cpu: usize = request
        .host
        .parallelism
        .saturating_sub(cpu_margin(request.host.parallelism))
        .max(1);
    let by_memory: usize = match request.host.memory_available {
        Some(bytes) => {
            usize::try_from((bytes / MEMORY_SHARE_DIVISOR) / request.per_item_memory.max(1))
                .unwrap_or(usize::MAX)
        }
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
pub fn sliced_workers(
    items: usize,
    ceiling: usize,
    host: &HostResources,
) -> usize {
    return worker_budget(&BudgetRequest {
        items,
        per_item_memory: SLICE_MEMORY,
        ceiling: Some(ceiling),
        cap: None,
        host,
    });
}

/// Budget controls stay out of the release executable.
#[cfg(test)]
#[path = "parallel_budget_tests.rs"]
mod tests;
