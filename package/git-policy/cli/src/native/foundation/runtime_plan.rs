//! What:
//!  How many worker threads and blocking-pool threads a runtime gets: the wrapper's budget for parallel work, bounded by the
//!  limits the system puts on threads and address space.
//! Why:
//!  The owner's ruling is a multi-thread runtime "sized to budget" (`doc/planning/git-policy-cli-full-tokio.md`, "The owner's
//!  answers at 2026-11-07", choices 2 and 3); the limits keep the build from asking for threads the system will refuse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planRuntime(host, limits); // { kind: 'current' } | { kind: 'multi', workers, blockingThreads }
//! ```

use crate::parallel_budget::{HostResources, cpu_margin};
use crate::runtime_limits::RuntimeLimits;

/// Red stub: the controls fail until the plan is written.
pub const THREAD_VIRTUAL_BYTES: u64 = 1;

/// Red stub.
pub const ADDRESS_SPACE_RESERVE_BYTES: u64 = 1;

/// Which runtime to build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePlan {
    /// One thread runs every task; no worker is started.
    CurrentThread,
    /// A pool of workers that poll tasks, and a pool for blocking jobs.
    MultiThread {
        /// Worker threads.
        workers: usize,
        /// Most blocking-pool threads.
        blocking_threads: usize,
    },
}

/// Red stub.
pub fn plan_runtime(host: &HostResources, limits: &RuntimeLimits) -> RuntimePlan {
    let _ = (host, limits, cpu_margin(1));
    return RuntimePlan::CurrentThread;
}

/// Plan controls stay out of the release executable.
#[cfg(test)]
#[path = "runtime_plan_tests.rs"]
mod tests;
