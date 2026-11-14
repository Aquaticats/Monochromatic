//! What:
//!  The number of blocking places a process holds itself to,
//!  stated by a control in place of the figure the CPUs the process may use give it.
//! Why:
//!  A process's room comes from the CPUs it may use
//! (`budgeted_workers`),
//! and a control's process may be a container's, whose CPU quota is the gate's own figure:
//! the release gate runs the package's tests in a container of four CPUs
//! (`bin/native-gate-container-limits.ts`),
//! and `std::thread::available_parallelism` follows that quota,
//! so a control that needs a larger room than four CPUs give it fails there and nowhere else,
//! and one that takes its room from a cap cannot raise what the machine gave.
//! This seam states the room instead:
//! a control names it in `CLI_GIT_TEST_ONLY_BLOCKING_PLACES`,
//!  and every process it starts,
//! a fixture's commit service included,
//! is built with that many places.
//! The number sizes the runtime's workers and its blocking pool as well as the bound on blocking jobs,
//! because that is what `budgeted_workers` answers
//!  (`runtime_plan.rs`,
//!  `blocking_jobs_within_the_budget.rs`),
//! so a place stays a thread at whatever room a control states.
//! The cap a process states for itself (`hold_blocking_jobs_to`,
//! `cap_workers`) still lowers the room,
//! so a control states one that the process it starts would take.
//! The variable is compiled into the release executable and is inert unless it names a positive number,
//! as `CLI_GIT_TEST_ONLY_HOLD` and `CLI_GIT_TEST_ONLY_SERVICE_FAULT` are.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the fixture's service is started with CLI_GIT_TEST_ONLY_BLOCKING_PLACES=4
//! const room = statedRoom() ?? budgetedWorkers(host, limits);
//! ```
use std::sync::OnceLock;

/// The variable that states the room,
/// in whole blocking places.
pub const ROOM_VARIABLE: &str = "CLI_GIT_TEST_ONLY_BLOCKING_PLACES";

/// The room a control stated,
/// read once for the life of the process.
static STATED: OnceLock<Option<usize>> = OnceLock::new();

/// What:
///  The room a control stated,
///  or nothing when the variable names no positive number.
/// Why:
///   Read once,
/// because a runtime and a bound that are already made keep their sizes,
///  so a value read later would disagree with both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const statedRoom = () => Number(process.env.CLI_GIT_TEST_ONLY_BLOCKING_PLACES) || undefined;
/// ```
pub fn the_room() -> Option<usize> {
    return *STATED.get_or_init(|| {
        return std::env::var(ROOM_VARIABLE).ok().and_then(|text| { return text.trim().parse::<usize>().ok(); }).filter(|places| { return *places > 0; });
    });
}
