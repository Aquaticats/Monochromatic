//! What:
//!  The bound of a drain request: the caller of an upgrade or restart waits for the service to drain at most `bound`, then returns the
//!  in-progress answer ("draining, upgrade not applied") while the drain goes on.
//! Why:
//!  The unit must make the drain possible (an indefinitely running hook delays it, which protects the hook), and the owner's rule is that
//!  nothing waits for ever (`doc/planning/cli-git-commits-never-fail.md`, R15-7 and R16a-3). The caller only polls; it never cancels the
//!  drain. The design's `WAIT_DRAIN` is a constant that must be measured, and no measurement exists, so the bound is a parameter and no
//!  constant is shipped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // waitForDrain(bound, pause, drained): 'drained' | 'in progress'
//! ```

use std::time::{Duration, Instant};

/// What the caller tells the person when the bound is reached.
pub const DRAIN_IN_PROGRESS: &str = "draining, upgrade not applied";

/// How a drain wait ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrainAnswer {
    /// The service drained within the bound.
    Drained,
    /// The bound was reached; the drain continues and the upgrade was not applied.
    InProgress,
}

/// What:
///  Polls `drained` every `pause` until it is true or `bound` has passed. `drained` must return promptly (it is a nonblocking probe of the
///  drain, not a wait of its own); the bound holds only as far as that contract does.
/// Why:
///  The poll is the only thing the caller does to the drain, so returning at the bound cancels nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitForDrain(bound: number, pause: number, drained: () => boolean): DrainAnswer;
/// ```
pub fn wait_for_drain(bound: Duration, pause: Duration, drained: &mut dyn FnMut() -> bool) -> DrainAnswer {
    let deadline: Instant = Instant::now() + bound;
    loop {
        if drained() {
            return DrainAnswer::Drained;
        }
        if Instant::now() >= deadline {
            return DrainAnswer::InProgress;
        }
        // The pause never runs past the bound: a long pause cannot hold the caller after its deadline.
        std::thread::sleep(pause.min(deadline.saturating_duration_since(Instant::now())));
    }
}
