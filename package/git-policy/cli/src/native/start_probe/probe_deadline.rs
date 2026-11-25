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
/// Measured 2026-11-21 in one throwaway guest with every round kept:
/// warm whole-probe rounds ran at 94,973 to 103,230 microseconds,
/// the same binary with the probe disabled at 75,889 to 93,432,
/// so the worker phase those two bracket was 8,861 to 27,341;
/// cache-dropped rounds 104,147 to 108,328,
/// the phase 26,203 to 30,414;
/// the first run after a guest boot 145,525;
/// with 109 mount namespaces the whole probe ran at 120,499 to 153,667,
/// the phase 38,875 to 68,068;
/// and with 309 of them 197,714 to 211,176,
/// the phase 106,349 to 118,503.
/// The clock starts just before the worker's fork,
/// so the bound covers the worker and holder,
/// not the gate,
/// the fingerprint or the started record the control also runs.
/// The slowest phase estimate plus the margin below is 349.5 milliseconds.
/// The guest's pool is a file-backed image,
/// so the number is the guest's until the owner measures this host's storage
/// (`package/git-policy/cli/doc/start-probe-deadline-measurement-2026-11-21.md`).
pub const PROBE_DEADLINE: Duration = Duration::from_millis(350);

/// The test-only variable that raises the bound for M3's raised-deadline
/// control,
/// which must record a held step as passed rather than timed out.
pub const DEADLINE_SWITCH_VARIABLE: &str = "CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS";

/// What:
///  The margin this bound carries over the slowest measured worker phase.
/// Why:
///   `LQM` asks for the measurement beside the value,
///  and the margin is the part of the value that protects a run from
///  noise,
///  so it is stated as its own number rather than hidden in the total.
pub const PROBE_DEADLINE_MARGIN: Duration = Duration::from_millis(231);

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

/// What:
///  The bound this build uses: the shipped one,
///  or a control's, when this build was made for a control.
/// Why:
///   A variable that can raise the bound turns a missed deadline into a pass,
///  and a unit's `Environment=` line can set one;
///  a build made for a control says so in its own manifest and cannot be the
///  build a release ships
///  (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
///  choice 11).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = theBound(); // shipped, unless this build was made for a control
/// ```
pub fn the_bound() -> Duration {
    #[cfg(feature = "start-probe-raised-deadline")]
    if let Some(raised) = deadline_from_words(std::env::var(DEADLINE_SWITCH_VARIABLE).ok().as_deref()) {
        return raised;
    }
    return PROBE_DEADLINE;
}

/// The bound the deadline variable names,
/// which only a build made for a control reads.
pub fn deadline_from_environment() -> Duration {
    return the_bound();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_deadline_tests.rs"]
mod tests;
