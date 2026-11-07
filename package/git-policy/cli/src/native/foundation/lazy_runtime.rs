//! What:
//!  The process's one tokio runtime, built on the first need and never before, sized from the budget and the limits, built on a
//!  helper thread under a deadline, and never dropped.
//! Why:
//!  The owner's rulings of 2026-11-07 (`doc/planning/git-policy-cli-full-tokio.md`, "The owner's answers"): "Full on tokio.
//!  Multi-thread." with the worker count "sized to budget"; built "Lazily, on first need" so a forwarded command pays nothing
//!  (measured +0.3 to +0.5 ms and about 125 minor page faults for an eager build); "Helper thread + deadline" so a build the
//!  system refuses cannot hold a commit ("nothing should ever wait forever"); "Fail plainly" where no thread can start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // There is no TS counterpart of choosing a thread pool: Node has one event loop.
//! const result = await runOnRuntime(async () => { ... });
//! ```

use std::time::Duration;
use tokio::runtime::Runtime;

use crate::runtime_plan::RuntimePlan;

/// How the runtime that was built came to be what it is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltFlavor {
    /// The plan was the current-thread runtime.
    CurrentThread,
    /// The multi-thread runtime was built with this many workers.
    MultiThread {
        /// Worker threads.
        workers: usize,
    },
    /// The multi-thread build did not finish by the deadline; the current-thread runtime stands in.
    CurrentThreadAfterDeadline,
    /// The multi-thread build failed, panicked or could not get its helper thread; the current-thread runtime stands in.
    CurrentThreadAfterFailure,
}

/// A runtime and how it came to be.
pub struct BuiltRuntime {
    /// The runtime itself; it is never dropped.
    pub runtime: Runtime,
    /// How it was built.
    pub flavor: BuiltFlavor,
}

/// Why a future could not be run on the runtime.
#[derive(Debug)]
pub enum RuntimeError {
    /// Called from inside the runtime, where `block_on` would panic.
    Nested,
    /// No runtime could be built, with the system's reason.
    CouldNotStart(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            RuntimeError::Nested => write!(formatter, "red stub"),
            RuntimeError::CouldNotStart(reason) => write!(formatter, "red stub {reason}"),
        };
    }
}

/// Red stub.
pub const BUILD_DEADLINE: Duration = Duration::from_secs(1);

/// Red stub.
pub fn build_with_deadline<B>(plan: RuntimePlan, deadline: Duration, multi_thread: B) -> std::io::Result<BuiltRuntime>
where
    B: FnOnce(usize, usize) -> std::io::Result<Runtime> + Send + 'static,
{
    let _ = (plan, deadline, multi_thread);
    return Err(std::io::Error::other("red stub"));
}

/// Red stub.
pub fn run_on_runtime<F: std::future::Future>(future: F) -> Result<F::Output, RuntimeError> {
    let _ = future;
    return Err(RuntimeError::Nested);
}

/// Red stub.
pub fn built_flavor() -> Option<BuiltFlavor> {
    return None;
}

/// Runtime controls stay out of the release executable.
#[cfg(test)]
#[path = "lazy_runtime_tests.rs"]
mod tests;
