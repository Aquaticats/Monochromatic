//! What:
//!  The waits a forced removal would need and their values,
//!  which are none:
//!  the wait between `SIGTERM` and `SIGKILL`,
//!  the handler's heartbeat period and the wrapper's give-up window,
//!  and the wait for a locked unit's stop.
//! Why:
//!  The owner's answer fixes the order and leaves the length to a decision or a
//!  measurement, and the design repeats it:
//!  "the wait's length is a constant with no value;
//!  step 17's build sets it only after the owner decides one or M12 sizes it,
//!  and the forced arm does not ship before then"
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The signal",
//!  "The act's liveness",
//!  L9 and L21;
//!  `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
//!  "The signal").
//!  A number here would be a number no measurement supports, so this build
//!  writes the constants as [Option]s that answer nothing,
//!  and the one function that would arm a clock refuses while any of them is
//!  unset.
//!  The probe rounds ran their stand-in with provisional values of the harness's
//!  own (10 s, 1 s and 4 s) and those are not these:
//!  a harness value that a test reads is never a shipped one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const TERMINATION_WAIT: number | undefined = undefined;
//! const window = theTimeoutWindow(); // undefined until every constant is set
//! ```

use std::time::Duration;

/// How long a blocker has to leave after `SIGTERM` before it is sent `SIGKILL`.
/// Why:
///  The owner chose "ask, wait, then kill" and left the wait unvalued
///  (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
///  "The signal").
///  Step 22's measurement sizes it; until then a forced removal that would need
///  it does not run, which is what makes the arm unshippable rather than
///  untested.
pub const TERMINATION_WAIT: Option<Duration> = None;

/// How often the handler advances its progress file's counter.
/// Why:
///  The wrapper reads the counter to tell a dead handler from a slow one,
///  so the period is what makes "no movement" mean something;
///  the design leaves both it and the give-up window to the owner or M12 (L21).
pub const HEARTBEAT_PERIOD: Option<Duration> = None;

/// How many whole heartbeat periods the wrapper waits before it reads the
/// handler as gone.
/// Why:
///  The design fixes the multiple and not the period: "a stated multiple of
///  `T_h` (four in this design, since a whole number of periods is what a reader
///  can compare)" (L21).
///  A multiple is a relation rather than a measured duration, so it is written
///  while the period itself is not.
pub const GIVE_UP_HEARTBEATS: u32 = 4;

/// How long a locked unit is given to leave its cgroup after its manager is asked
/// to stop it.
/// Why:
///  The design gives this wait no provisional value at all ("`T_stop` has no
///  provisional value, because a manager's stop is not the act's own wait and
///  nothing has timed one", "The constants").
pub const STOP_WAIT: Option<Duration> = None;

/// What:
///  A constant a forced removal needs and this build does not have.
/// Why:
///  A refusal that named only "no value" would leave a person guessing which
///  number is missing,
///  and each one is sized by a different later step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RefusedClock extends Error { constructor(readonly constant: string) {} }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusedClock {
    /// [`TERMINATION_WAIT`] has no value.
    TerminationWait,
    /// [`HEARTBEAT_PERIOD`] has no value.
    Heartbeat,
    /// [`STOP_WAIT`] has no value.
    StopWait,
}

impl RefusedClock {
    /// The constant's own name,
    /// as the design and step 22 spell it.
    pub fn constant(self) -> &'static str {
        return match self {
            RefusedClock::TerminationWait => "T_wait",
            RefusedClock::Heartbeat => "T_h",
            RefusedClock::StopWait => "T_stop",
        };
    }
}

impl std::fmt::Display for RefusedClock {
    /// One sentence naming the constant and who sizes it.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return write!(
            formatter,
            "the forced removal cannot run: its {} has no value yet, and step 17's build sets one only after the owner decides it or step 22's measurement sizes it",
            self.constant()
        );
    }
}

/// What:
///  The three periods one version of the act runs with,
///  which exist only together.
/// Why:
///  The wrapper's give-up window is counted in the handler's heartbeat periods,
///  so a window whose period is unset compares nothing,
///  and a structure that could hold one of the two without the other would let a
///  caller arm half a liveness check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TimeoutWindow = { termination: number; heartbeat: number; giveUp: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeoutWindow {
    /// The wait between `SIGTERM` and `SIGKILL`.
    termination: Duration,
    /// The handler's heartbeat period.
    heartbeat: Duration,
}

impl TimeoutWindow {
    /// What:
    ///  The window these constants give,
    ///  or the first unset constant that stops it.
    /// Why:
    ///  One place turns the constants into a window,
    ///  so no caller can assemble its own from half of them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const window = TimeoutWindow.ofTheConstants();
    /// ```
    pub fn of_the_constants() -> Result<TimeoutWindow, RefusedClock> {
        let Some(heartbeat) = HEARTBEAT_PERIOD else {
            return Err(RefusedClock::Heartbeat);
        };
        let Some(termination) = TERMINATION_WAIT else {
            return Err(RefusedClock::TerminationWait);
        };
        return Ok(TimeoutWindow {
            termination,
            heartbeat,
        });
    }

    /// A window of these two periods,
    /// for a control that needs values the release does not have.
    pub fn at(
        termination: Duration,
        heartbeat: Duration,
    ) -> TimeoutWindow {
        return TimeoutWindow {
            termination,
            heartbeat,
        };
    }

    /// The wait between `SIGTERM` and `SIGKILL`.
    pub fn termination(self) -> Duration {
        return self.termination;
    }

    /// The handler's heartbeat period.
    pub fn heartbeat(self) -> Duration {
        return self.heartbeat;
    }

    /// The wrapper's give-up window:
    /// [`GIVE_UP_HEARTBEATS`] whole periods,
    /// in which a counter that has not moved reads as a dead handler.
    pub fn give_up(self) -> Duration {
        return self.heartbeat.saturating_mul(GIVE_UP_HEARTBEATS);
    }
}

/// What:
///  The wait for a locked unit's stop,
///  or the constant that stops it.
/// Why:
///  A stop of a unit is the one act a forced removal asks another program for,
///  and the design gives its wait no provisional value,
///  so the act refuses on the same terms as the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopWait = theStopWait(); // throws while T_stop is unset
/// ```
pub fn the_stop_wait() -> Result<Duration, RefusedClock> {
    return STOP_WAIT.ok_or(RefusedClock::StopWait);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "forced_arm_numbers_tests.rs"]
mod tests;
