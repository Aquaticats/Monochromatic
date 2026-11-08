//! What:
//!  How many worker threads and blocking-pool threads a runtime gets:
//! the wrapper's budget for parallel work,
//! bounded by the
//!  limits the system puts on threads and address space.
//! Why:
//!  The owner's ruling is a multi-thread runtime "sized to budget" (`doc/planning/git-policy-cli-full-tokio.md`,
//! "The owner's
//!  answers at 2026-11-07",
//! choices 2 and 3);
//! the limits keep the build from asking for threads the system will refuse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planRuntime(host, limits); // { kind: 'current' } | { kind: 'multi', workers, blockingThreads }
//! ```

use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::runtime_limits::RuntimeLimits;

/// The stack of every worker and blocking-pool thread,
/// set explicitly on the builder so the figure below does not depend on
///  tokio's default.
/// It is tokio's own default (2 MiB,
/// `Builder::thread_stack_size`).
pub const THREAD_STACK_BYTES: usize = 2 << 20;

/// What a thread adds to the address space beyond its stack:
/// guard page and thread-local storage.
/// Measured on this host's glibc
///  with the malloc arenas capped to one:
/// 28 workers added 57,936 KiB,
/// 2,069 KiB each,
/// and one blocking thread 2,064 KiB,
/// so
///  16 KiB over the stack;
/// rounded up to 64 KiB (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Why the address space runs out").
const THREAD_VIRTUAL_SLACK_BYTES: u64 = 64 << 10;

/// The address space one worker or blocking-pool thread takes.
pub const THREAD_VIRTUAL_BYTES: u64 = (THREAD_STACK_BYTES as u64) + THREAD_VIRTUAL_SLACK_BYTES;

/// The address space kept back for everything the command maps after the runtime is built:
/// its heap,
/// the repository files it
///  reads and the stacks of its other threads.
/// The whole process mapped about 20 MiB in the measurement of today's binary
///  (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Why the address space runs out");
/// the reserve is three times that,
/// the
///  coordinator's pick under the owner's choice 3 ("Cap pools + read limits"),
/// open to the owner's veto.
pub const ADDRESS_SPACE_RESERVE_BYTES: u64 = 64 << 20;

/// The resident memory the budget charges one worker:
/// its stack pages and its share of the allocator,
/// estimated at one
///  mebibyte.
/// 28 idle workers measured 3,836 KiB resident in all (about 137 KiB each),
/// so this is a conservative upper bound.
const WORKER_RESIDENT_BYTES: u64 = 1 << 20;

/// The threads a runtime of `workers` workers can start:
/// the workers,
/// the same number of blocking-pool threads,
/// and the helper
///  thread that builds the runtime under a deadline.
pub fn threads_needed(workers: usize) -> u64 {
    return (workers as u64).saturating_mul(2).saturating_add(1);
}

/// Which runtime to build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePlan {
    /// One thread runs every task;
    /// no worker is started.
    CurrentThread,
    /// A pool of workers that poll tasks,
    /// and a pool for blocking jobs.
    MultiThread {
        /// Worker threads.
        workers: usize,
        /// Most blocking-pool threads.
        blocking_threads: usize,
    },
}

/// The workers a count of threads left allows:
/// each comes with a blocking-pool thread,
/// and one more thread builds the runtime.
fn workers_by_threads(left: u64) -> usize {
    return usize::try_from(left.saturating_sub(1) / 2).unwrap_or(usize::MAX);
}

/// The workers an address space left allows,
/// after the reserve,
/// each as a pair of threads.
fn workers_by_address_space(left: u64) -> usize {
    let pairs: u64 = left.saturating_sub(ADDRESS_SPACE_RESERVE_BYTES) / (2 * THREAD_VIRTUAL_BYTES);
    return usize::try_from(pairs).unwrap_or(usize::MAX);
}

/// What:
///  The plan for this host:
/// the budget's worker count,
/// bounded by the threads and the address space the limits leave,
/// and the
///  current-thread runtime when not even one worker fits.
/// Why:
///  The watchers of waits are tasks that run beside a thread blocked in a read or a wait,
/// so one worker is the least that gives
///  them a thread;
/// a plan that asks for more than the limits leave is the failure the plan exists to avoid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const workers = Math.min(budget, (threadsLeft - 1) / 2, (addressLeft - reserve) / (2 * threadBytes));
/// return workers < 1 ? { kind: 'current' } : { kind: 'multi', workers, blockingThreads: workers };
/// ```
pub fn plan_runtime(host: &HostResources, limits: &RuntimeLimits) -> RuntimePlan {
    let by_budget: usize = worker_budget(&BudgetRequest {
        items: usize::MAX,
        per_item_memory: WORKER_RESIDENT_BYTES,
        ceiling: None,
        cap: None,
        host,
    });
    // Each worker comes with a blocking-pool thread, and one more thread builds the runtime.
    let by_threads: usize = limits.threads_left.map_or(usize::MAX, workers_by_threads);
    let by_address_space: usize = limits.address_space_left.map_or(usize::MAX, workers_by_address_space);
    let workers: usize = by_budget.min(by_threads).min(by_address_space);
    if workers < 1 {
        return RuntimePlan::CurrentThread;
    }
    return RuntimePlan::MultiThread { workers, blocking_threads: workers };
}

/// Plan controls stay out of the release executable.
/// What:
///  `plan` with its workers and its blocking pool held to at most `cap` threads each;
/// a plan that is already smaller,
/// and the current-thread plan,
/// stand.
/// Why:
///  A long-lived process that many of its kind run at once (one commit service per worktree) keeps its idle workers for hours,
/// so the owner's "sized to budget" is bounded for it by a cap measured for its work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capped = plan.kind === 'multi' ? { ...plan, workers: Math.min(plan.workers, cap), blockingThreads: Math.min(plan.blockingThreads, cap) } : plan;
/// ```
pub fn capped_at(plan: RuntimePlan, cap: usize) -> RuntimePlan {
    return match plan {
        RuntimePlan::MultiThread { workers, blocking_threads } => RuntimePlan::MultiThread { workers: workers.min(cap), blocking_threads: blocking_threads.min(cap) },
        RuntimePlan::CurrentThread => RuntimePlan::CurrentThread,
    };
}

#[cfg(test)]
#[path = "runtime_plan_tests.rs"]
mod tests;
