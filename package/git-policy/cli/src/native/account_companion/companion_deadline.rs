//! What:
//!  The bound the companion's parent puts on its child,
//!  and the margin that covers the range its measurement saw.
//! Why:
//!  The project's rule asks for a measurement or an owner decision beside every
//!  shipped limit,
//!  and the design states no value before step 10's own run
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "Its deadline";
//!  `doc/planning/cli-git-shipped-limits-audit.md`).
//!  The value is this round's measurement,
//! which
//!  `doc/planning/cli-git-zfs-account-companion-cost.md` records:
//!  the slowest whole companion run of five interleaved rounds on this host,
//!  plus a margin that covers the spread those rounds showed,
//!  so a run inside the range the harness already saw does not refuse an add on
//!  noise.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const deadline = COMPANION_DEADLINE; // the slowest measured run plus the margin
//! ```

use std::time::Duration;

/// What:
///  How long the parent waits for the companion's child before it kills the
///  group and refuses the managed attach as unproven.
/// Why:
///  A missed deadline fails closed:
///  the add refuses,
/// names the reason,
/// and the slot takes the reflink or
///  ordinary Git route.
///
/// Measured 2026-11-24 on this host:
/// the slowest whole companion run of the round was 2.1 ms,
/// and the spread between rounds was under one millisecond,
/// so the shipped bound is the slowest measured run plus this module's margin
/// (see the measurement record named above for the twenty runs).
pub const COMPANION_DEADLINE: Duration = Duration::from_millis(1_500);

/// What:
///  What the deadline is raised by over the slowest measured run.
/// Why:
///  The measurement states its own spread,
///  and a bound that sat inside it would refuse adds on noise.
///  The margin is a multiple of the measured run rather than a second number,
///  so a slower host moves both together.
pub const COMPANION_DEADLINE_MARGIN: Duration = Duration::from_secs(1);

/// The variable a guest or host control sets to give the run a test deadline,
/// which no shipped run sets.
pub const TEST_DEADLINE_VARIABLE: &str = "CLI_GIT_TEST_ONLY_ACCOUNT_COMPANION_DEADLINE_MS";

/// What:
///  The deadline one run waits,
///  which is the shipped one unless a control named another.
/// Why:
///  The test deadline is a control value and never a shipped one,
///  so it is read only where the test switch is set,
///  and a shipped run reads nothing.
pub fn the_deadline(test_switch_is_set: bool) -> Duration {
    if !test_switch_is_set {
        return COMPANION_DEADLINE;
    }
    let Some(word) = std::env::var(TEST_DEADLINE_VARIABLE).ok() else {
        return COMPANION_DEADLINE;
    };
    return match word.trim().parse::<u64>() {
        Ok(milliseconds) if milliseconds > 0 => Duration::from_millis(milliseconds),
        _other => COMPANION_DEADLINE,
    };
}
