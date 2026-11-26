//! What:
//!  The numbers the hang guard of the binary tests judges a command by:
//! how long its process tree may show nothing,
//! how much processor time it may use,
//! how long the wall-clock backstop waits,
//! how often the monitor looks,
//! and how long a kill may stay unclaimed,
//! with the shared limits every wrapped command runs under.
//! Why:
//!  A reader asking "what is too slow" finds one file of figures,
//!  each with the sentence that says which command it ends and why it is that large,
//!  instead of meeting them one by one between the judge,
//!  the monitor and the kills.
//!  The values are the same as they were beside the code that uses them,
//!  so nothing a test measured moves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const NO_PROGRESS_WINDOW = 5000; const CPU_BUDGET = 10000; const POLL = 100; const SHARED = { noProgress: NO_PROGRESS_WINDOW, cpuBudget: CPU_BUDGET, backstop: 600 };
//! ```

use std::time::Duration;

/// The wall-clock backstop of a shared-bound command as the text `timeout` takes:
/// ten minutes,
/// which only a command
///  that is both not idle and not burning the processor ever reaches.
pub const SHARED_BACKSTOP_SECONDS: &str = "600";

/// How long a process tree may show nothing at all before it counts as hung.
pub const NO_PROGRESS_WINDOW: Duration = Duration::from_secs(5);

/// How much processor time a tree may use in all.
/// Real commands of these tests use well under a second of it.
pub const CPU_BUDGET: Duration = Duration::from_secs(10);

/// How often the monitor looks at the commands.
pub const POLL: Duration = Duration::from_millis(100);

/// How long a kill may stay unclaimed by the test that ran the command,
/// with no thread of that test occupied with the command,
/// before the process aborts with its message.
pub const UNCLAIMED_GRACE: Duration = Duration::from_secs(3);

/// The status a process exits with when a kill nobody claimed is found at its exit.
pub const UNCLAIMED_EXIT_STATUS: i32 = 101;

/// The limits of one watched command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// How long nothing may move.
    pub no_progress: Duration,
    /// The processor time of the tree that ends it.
    pub cpu_budget: Duration,
    /// The wall time that ends it whatever else is true.
    pub backstop: Duration,
}

/// The limits of the shared bound.
pub const SHARED: Limits = Limits {
    no_progress: NO_PROGRESS_WINDOW,
    cpu_budget: CPU_BUDGET,
    backstop: Duration::from_secs(600),
};

/// Why a command was ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    /// Nothing moved for the window.
    NoProgress(Duration),
    /// The tree used its processor budget.
    CpuBudget(Duration),
    /// The wall-clock backstop passed.
    Backstop(Duration),
}

impl std::fmt::Display for Reason {
    /// The reason in words,
    /// with the figure that decided it.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            Reason::NoProgress(idle) => write!(
                formatter,
                "no progress for {:.1} s: no task was runnable, waiting for the disk or held from outside, and processor time, bytes moved, the work of waited-for children and the set of processes stood still (a deadlock or a wait on something that never happens)",
                idle.as_secs_f64()
            ),
            Reason::CpuBudget(used) => write!(
                formatter,
                "it used {:.1} s of processor time (a loop)",
                used.as_secs_f64()
            ),
            Reason::Backstop(elapsed) => write!(
                formatter,
                "it was still running after the {:.0} s wall-clock backstop",
                elapsed.as_secs_f64()
            ),
        };
    }
}
