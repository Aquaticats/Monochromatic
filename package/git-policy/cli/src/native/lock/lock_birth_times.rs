//! What:
//!  When a lock file and the PID file beside it were born,
//!  as one read,
//!  with the arm a control uses to hide those times or to make them coarse.
//! Why:
//!  The gate that decides whether a PID file names this lock's holder compares the two births
//!  (`Judge::pid_file_state`).
//!  A file system may record no birth time at all,
//!  or record it in ticks of a second or two,
//!  and then two files written moments apart carry the same time:
//!  the order of the two births is not proven,
//!  and an equal or missing time must never be read as "the PID file is older" or as "it is newer"
//!  (`AGENTS.md`,
//! rule `NHP`:
//! unusual file systems are normal).
//!  One function reads a birth time,
//!  so a control can drive the gate on a file system it does not have:
//!  the thread-local arm is for the unit controls,
//!  and the variable arms the same arm in a child process the binary controls start.
//!  Both are inert unless a control sets them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! birthTimeOf(path) // statx btime, floored to the control's tick, or undefined
//! ```

/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// Time spans and the clock a birth time is read from.
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// What:
///  How the birth times a control sees are reported,
///  which is what a file system of another granularity would give.
/// Why:
///  Three answers a file system can give,
///  and the gate must treat the last two as "not proven":
///  the times it records,
///  no time at all,
///  and a time in ticks coarse enough that two files written apart share one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BirthTimes = { kind: 'recorded' } | { kind: 'hidden' } | { kind: 'coarse'; tickMs: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BirthTimes {
    /// What the file system records.
    Recorded,
    /// No birth time at all,
    /// as a file system without `statx` birth times gives.
    Hidden,
    /// A time floored to this tick,
    /// as a file system with coarse times gives.
    Coarse(
        /// The tick size.
        Duration,
    ),
}

/// The variable a control arms to report birth times as another file system would,
/// in a child process:
/// `none`,
/// or a tick size in whole milliseconds.
pub const BIRTH_TIME_VARIABLE: &str = "CLI_GIT_TEST_ONLY_BIRTH_TIME";

/// What:
///  The birth times a setting names,
///  or nothing when the setting means nothing.
/// Why:
///   A misspelt or empty setting must leave the real times,
///  so a run that sets nothing behaves as production does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// birthTimesOf('none') // { kind: 'hidden' }; birthTimesOf('2000') // { kind: 'coarse', tickMs: 2000 }
/// ```
pub fn birth_times_of(setting: &str) -> Option<BirthTimes> {
    let trimmed: &str = setting.trim();
    if trimmed == "none" {
        return Some(BirthTimes::Hidden);
    }
    let milliseconds: u64 = match trimmed.parse::<u64>() {
        Ok(found) if found > 0 => found,
        _ => return None,
    };
    return Some(BirthTimes::Coarse(Duration::from_millis(milliseconds)));
}

// The arm a control sets on the running thread,
// which no production path sets: a unit control drives the gate without a file system of another
// granularity, and the arm is per thread because the suite runs many controls in one process.
#[cfg(test)]
thread_local! {
    /// The arm of the running thread.
    static TIMES_FOR_THIS_THREAD: std::cell::Cell<Option<BirthTimes>> = const { std::cell::Cell::new(None) };
}

/// What:
///  Run `body` with birth times reported as another file system would,
///  and put the thread's own arm back however the body ends.
/// Why:
///  A control that left the arm set would change the controls that run after it on the same thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { set(times); body(); } finally { clear(); }
/// ```
#[cfg(test)]
pub fn with_birth_times<T>(times: BirthTimes, body: impl FnOnce() -> T) -> T {
    let started: bool = TIMES_FOR_THIS_THREAD.with(|held| {
        let was_none: bool = held.get().is_none();
        held.set(Some(times));
        return was_none;
    });
    let ended: T = body();
    if started {
        TIMES_FOR_THIS_THREAD.with(|held| { held.set(None); });
    }
    return ended;
}

/// What:
///  The arm that applies now:
///  this thread's,
/// else the process's variable,
/// else the real times.
/// Why:
///  One order decides,
///  so a unit control's arm wins over an inherited variable and a run with neither reads the file system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const times = threadArm ?? birthTimesOf(process.env[BIRTH_TIME_VARIABLE] ?? '') ?? { kind: 'recorded' };
/// ```
fn times_in_force() -> BirthTimes {
    #[cfg(test)]
    if let Some(armed) = TIMES_FOR_THIS_THREAD.with(std::cell::Cell::get) {
        return armed;
    }
    let Ok(setting) = std::env::var(BIRTH_TIME_VARIABLE) else {
        return BirthTimes::Recorded;
    };
    return birth_times_of(setting.as_str()).unwrap_or(BirthTimes::Recorded);
}

/// What:
///  The birth time of the file at `path`,
///  as the file system of this run reports it,
///  or nothing when it reports none.
/// Why:
///  A path that cannot be looked at has no time,
///  and an arm that hides the times answers nothing at all,
///  which the gate reads as "not proven" rather than as either order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// birthTimeOf(path) // statSync(path).birthtimeMs, floored to the arm's tick
/// ```
pub(crate) fn birth_time_of(path: &Path) -> Option<SystemTime> {
    let recorded: Option<SystemTime> = std::fs::symlink_metadata(path).ok().and_then(recorded_birth);
    return match times_in_force() {
        BirthTimes::Recorded => recorded,
        BirthTimes::Hidden => None,
        BirthTimes::Coarse(tick) => match recorded {
            Some(time) => floor_to(time, tick),
            None => None,
        },
    };
}

/// What:
///  The birth time the file system records for one file,
///  or nothing when it records none.
/// Why:
///  One read,
///  so the arm and the gate cannot differ in how a time is taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// metadata.created().ok()
/// ```
fn recorded_birth(metadata: std::fs::Metadata) -> Option<SystemTime> {
    return metadata.created().ok();
}

/// What:
///  A time floored to a whole number of `tick` sizes since the epoch,
///  which is what a file system of that granularity reports.
/// Why:
///  Flooring keeps the order of two times that lie in different ticks,
///  and gives one time to two that lie in the same tick,
///  which is exactly the state the gate must read as unproven.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Date(Math.floor(ms / tickMs) * tickMs)
/// ```
fn floor_to(time: SystemTime, tick: Duration) -> Option<SystemTime> {
    let since: Duration = time.duration_since(UNIX_EPOCH).ok()?;
    let tick_nanos: u128 = tick.as_nanos().max(1);
    let floored: u128 = since.as_nanos() / tick_nanos * tick_nanos;
    let seconds: u64 = u64::try_from(floored / 1_000_000_000).ok()?;
    let nanos: u32 = u32::try_from(floored % 1_000_000_000).ok()?;
    return Some(UNIX_EPOCH + Duration::new(seconds, nanos));
}

/// Controls of the arm stay out of the release executable.
#[cfg(test)]
#[path = "lock_birth_times_tests.rs"]
pub(crate) mod tests;
