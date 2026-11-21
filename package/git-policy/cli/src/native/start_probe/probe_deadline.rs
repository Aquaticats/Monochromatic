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
/// Measured 2026-11-21 in the throwaway guest by M4's five interleaved rounds
/// in each of six green runs:
/// the whole probe ran at 116,570 to 230,892 microseconds per round,
/// which is a spread of 114,322,
/// and the same binary with the probe disabled ran at 65,910 to 167,540,
/// so the probe's own work is about half the run.
/// The slowest round is a cold binary's first run,
/// and it is kept in the range because a boot after an upgrade meets that same
/// cold state.
/// This bound is the slowest round plus more than that spread.
/// The guest's pool is a file-backed image,
/// so the number is the guest's until the owner measures this host's storage
/// (`package/git-policy/cli/doc/start-probe-deadline-measurement-2026-11-21.md`).
pub const PROBE_DEADLINE: Duration = Duration::from_millis(350);

/// The test-only variable that raises the bound for M3's raised-deadline
/// control,
/// which must record a held step as passed rather than timed out.
pub const DEADLINE_SWITCH_VARIABLE: &str = "CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS";

/// What:
///  The margin this bound carries over the slowest measured run.
/// Why:
///   `LQM` asks for the measurement beside the value,
///  and the margin is the part of the value that protects a run from
///  noise,
///  so it is stated as its own number rather than hidden in the total.
pub const PROBE_DEADLINE_MARGIN: Duration = Duration::from_millis(119);

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
pub fn deadline_passed(started: Instant, now: Instant, deadline: Duration) -> bool {
    return now.duration_since(started) >= deadline;
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
pub fn remaining(started: Instant, now: Instant, deadline: Duration) -> Option<Duration> {
    let elapsed: Duration = now.duration_since(started);
    if elapsed >= deadline {
        return None;
    }
    return Some(deadline - elapsed);
}

/// The bound one value asks for,
/// which is the shipped deadline unless the test-only variable names a
/// positive number of milliseconds.
pub fn deadline_from_words(value: Option<&str>) -> Duration {
    let Some(word) = value else {
        return PROBE_DEADLINE;
    };
    return match word.parse::<u64>() {
        Ok(milliseconds) if milliseconds > 0 => Duration::from_millis(milliseconds),
        _ => PROBE_DEADLINE,
    };
}

/// The bound this process's environment asks for.
pub fn deadline_from_environment() -> Duration {
    return deadline_from_words(std::env::var(DEADLINE_SWITCH_VARIABLE).ok().as_deref());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_deadline_tests.rs"]
mod tests;
