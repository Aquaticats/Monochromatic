//! What:
//!  A runtime as this process has it:
//!  the runtime itself,
//!  how its build came out,
//!  how long that build took,
//!  and why a future could not be run on it.
//! Why:
//!  `lazy_runtime.rs` holds the one runtime of a process and answers what can be done with it,
//! and `the_build_of_the_process_runtime.rs` produces these values;
//!  the shapes live in a file of their own,
//! so neither of those files has to reach into the other for them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type BuiltFlavor = 'currentThread' | { multiThread: number } | 'currentThreadAfterDeadline' | 'currentThreadAfterFailure';
//! type BuiltRuntime = { runtime: Runtime; flavor: BuiltFlavor };
//! type RuntimeError = { nested: true } | { couldNotStart: string };
//! type RuntimeBuild = { flavor: BuiltFlavor; spent: number };
//! ```

/// The clock a runtime's build is timed with and measured in.
use std::time::Duration;
/// The runtime itself,
/// which every value here carries or names.
use tokio::runtime::Runtime;

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
    /// The multi-thread build did not finish by the deadline;
    /// the current-thread runtime stands in.
    CurrentThreadAfterDeadline,
    /// The multi-thread build failed,
    /// panicked or could not get its helper thread;
    /// the current-thread runtime stands in.
    CurrentThreadAfterFailure,
}

/// A runtime and how it came to be.
pub struct BuiltRuntime {
    /// The runtime itself;
    /// it is never dropped.
    pub runtime: Runtime,
    /// How it was built.
    pub flavor: BuiltFlavor,
}

/// Why a future could not be run on the runtime.
#[derive(Debug)]
pub enum RuntimeError {
    /// Called from inside the runtime,
    /// where `block_on` would panic.
    Nested,
    /// No runtime could be built,
    /// with the system's reason.
    CouldNotStart(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            RuntimeError::Nested => write!(
                formatter,
                "the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; \
                 nothing was run. This is a defect of monochromatic-git-policy-cli: report it with the command that was run"
            ),
            RuntimeError::CouldNotStart(reason) => write!(
                formatter,
                "cannot start the runtime that runs this command's waits: {reason}. Hint: the system refused a thread or memory, so \
                 check the process limit (`ulimit -u`), the address-space limit (`ulimit -v`) and the control group's `pids.max` and \
                 `memory.max`, free some, and run the command again; no Git command ran"
            ),
        };
    }
}

/// How the process's runtime was built,
/// with how long its build took.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeBuild {
    /// How it came to be what it is.
    pub flavor: BuiltFlavor,
    /// How long the build took,
    /// `BUILD_DEADLINE` included when it was replaced at the deadline.
    pub spent: Duration,
}
