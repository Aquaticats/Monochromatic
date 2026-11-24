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
/// Measured 2026-11-24 on this host,
/// over five interleaved rounds of three samples each
/// (`doc/planning/cli-git-zfs-account-companion-cost.md`):
/// a companion run's median was 1.62 to 1.79 ms in every round,
/// its samples ran 1.342 to 2.759 ms,
/// and a real `git worktree add` through the same build's wrapper took a median
/// of 11.35 to 14.04 ms,
/// so the companion is 1.123 to 1.144 of an add.
/// The bound is the slowest measured run plus the margin below.
pub const COMPANION_DEADLINE: Duration = Duration::from_millis(100);

/// What:
///  What the deadline is raised by over the slowest measured run.
/// Why:
///  A bound that sat inside the spread would refuse adds on noise,
///  and this host loads heavily and a first sample runs cold:
///  the same round's first add took 91.4 ms against a 11.4 ms median,
///  an excursion of about eight times.
///  97 ms is 68 times the companion's measured spread of 1.42 ms,
///  so a cold start on a loaded host still answers,
///  and a wedged mount call is still refused well inside a second.
pub const COMPANION_DEADLINE_MARGIN: Duration = Duration::from_millis(97);

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
