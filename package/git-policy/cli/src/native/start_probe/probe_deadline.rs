//! What:
//!  The one bound the mode puts on its worker and holder:
//!  the deadline's value,
//!  the margin over the measured run,
//!  and the test a starting time has passed it.
//! Why:
//!  The mode enforces the deadline rather than the worker,
//!  because a worker wedged in a kernel wait never reaches its own check
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "Who runs it,
//! and where").
//!  No value ships before its measurement,
//!  so the value here is cited beside the run that sized it,
//!  and the margin covers the run-to-run range the harness saw
//!  (rule `LQM`,
//!  `doc/decision/cli-git-zfs-start-probe.md`,
//!  "Amendments made with this record").
//!  The deadline bounds the mode's wait only:
//!  it starts no kernel wait and kills the worker's process group and the
//!  holder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const deadline = start + PROBE_DEADLINE; if (now > deadline) killTheGroup();
//! ```

use std::time::{Duration, Instant};

/// What:
///  The whole run's bound,
///  the slowest measured whole-probe run plus the margin between the
///  interleaved rounds.
/// Why:
///   A run inside the range the harness already saw does not fail closed on
///  noise,
///  and a wedged probe is killed rather than holding a boot forever.
///  The value and the margin beside it are measured,
/// not chosen:
///  the measurement is M4's whole-probe timing in this step's guest arms
///  (`doc/planning/cli-git-zfs-start-probe-design.md`,
///  "M4:
/// what the probe costs"),
///  and the value here is replaced by the measurement's own number.
///
/// Pending:
/// M4's run in this step's guest arms replaces this value and the margin
/// below with the numbers it measured,
/// and no shipped build may carry this placeholder.
pub const PROBE_DEADLINE: Duration = Duration::from_secs(60);

/// What:
///  The margin this bound carries over the slowest measured run.
/// Why:
///   `LQM` asks for the measurement beside the value,
///  and the margin is the part of the value that protects a run from
///  noise,
///  so it is stated as its own number rather than hidden in the total.
pub const PROBE_DEADLINE_MARGIN: Duration = Duration::from_millis(0);

/// What:
///  Whether the run that started at `started` has passed its deadline at
///  `now`.
/// Why:
///   The comparison is one place,
///  so the mode's wait,
///  its kill and the timed-out record agree on where the boundary is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const expired = now.durationSince(started) >= PROBE_DEADLINE;
/// ```
pub fn deadline_passed(started: Instant, now: Instant) -> bool {
    return now.duration_since(started) >= PROBE_DEADLINE;
}

/// What:
///  How long the mode may still wait,
///  or nothing once the deadline passed.
/// Why:
///   The wait is one call with this bound rather than a poll,
///  so a run that ends early returns at once and a run that hangs ends at the
///  deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = remaining(started, Instant.now()); // 0 when expired
/// ```
pub fn remaining(started: Instant, now: Instant) -> Option<Duration> {
    let elapsed: Duration = now.duration_since(started);
    if elapsed >= PROBE_DEADLINE {
        return None;
    }
    return Some(PROBE_DEADLINE - elapsed);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_deadline_tests.rs"]
mod tests;
