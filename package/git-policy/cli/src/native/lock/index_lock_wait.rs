//! What:
//!  Git's own lock-retry schedule:
//! the backoff position,
//! its next step,
//! and the jittered wait it prescribes.
//! Why:
//!  A caller that retries a lock the way Git retries its own waits on the same
//!  schedule,
//! so it neither hammers the lock nor starves behind it.
//! Waiting for a foreign `index.lock` is gone:
//! a command
//!  that cannot take Git's `index.lock` ends as native Git ends it,
//! and the commit service's landing retries it a bounded
//!  number of times itself (`src/index-lock/index-lock-wait.ts` is the incumbent of the schedule).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const wait = jitteredWait(state, Math.random()); state = nextBackoff(state);
//! ```

/// Git's `BACKOFF_MAX_MULTIPLIER`:
///  the backoff stops growing here.
pub const BACKOFF_MAX_MULTIPLIER: u64 = 1_000;

/// Git's jitter floor in thousandths of the backoff.
const JITTER_FLOOR_PERMILLE: u64 = 750;

/// Git's jitter span in thousandths of the backoff.
const JITTER_SPAN_PERMILLE: u64 = 500;

/// Thousandths per whole.
const PERMILLE: u64 = 1_000;

/// What:
///  Git's backoff position:
///  `step` is Git's `n`,
///  `multiplier` the wait base in milliseconds.
/// Why:
///   `BackoffState` in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BackoffState = { step: number; multiplier: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackoffState {
    /// Git's `n`.
    pub step: u64,
    /// Git's `multiplier`,
    ///  the current backoff in milliseconds.
    pub multiplier: u64,
}

/// Backoff before the first wait.
pub const INITIAL_BACKOFF: BackoffState = BackoffState {
    step: 1,
    multiplier: 1,
};

/// What:
///  One jittered wait from Git's formula `(750 + rand() % 500) * backoff / 1000`,
///  rounded down.
/// Why:
///   Jitter spreads retrying processes apart;
///  `random` is a value in `[0, 1)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.floor(((750 + Math.floor(random * 500)) * state.multiplier) / 1000);
/// ```
pub fn jittered_wait(state: BackoffState, random: f64) -> u64 {
    let span: u64 = (random * (JITTER_SPAN_PERMILLE as f64)).floor() as u64;
    return (JITTER_FLOOR_PERMILLE + span) * state.multiplier / PERMILLE;
}

/// What:
///  The next backoff:
///  `multiplier += 2n + 1`,
///  capped at `BACKOFF_MAX_MULTIPLIER`.
/// Why:
///   Adding `2n + 1` makes the multiplier the square of the step,
///  which is Git's schedule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nextBackoff({ step: 1, multiplier: 1 }); // { step: 2, multiplier: 4 }
/// ```
pub fn next_backoff(state: BackoffState) -> BackoffState {
    let multiplier: u64 = state.multiplier + 2 * state.step + 1;
    if multiplier > BACKOFF_MAX_MULTIPLIER {
        return BackoffState {
            step: state.step,
            multiplier: BACKOFF_MAX_MULTIPLIER,
        };
    }
    return BackoffState {
        step: state.step + 1,
        multiplier,
    };
}

/// Backoff controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_wait_tests.rs"]
mod tests;
