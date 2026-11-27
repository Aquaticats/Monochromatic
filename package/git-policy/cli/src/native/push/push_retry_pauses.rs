//! What:
//!  The pause before each new try of a push the service stopped or that failed:
//!  the variable that sets the first pause,
//!  the directory the plan is kept in,
//!  and the pause that doubles with every try.
//! Why:
//!  A push stopped by the stall window,
//!  by the network,
//!  by an LFS server error or by a reference lock passes by itself,
//!  so the service tries again with pauses that grow,
//!  and a host that is offline for an hour is not asked every few seconds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pause = Math.min(first * 2 ** (attempt - 1), LONGEST);
//! ```

/// Import the positive-seconds reading the variable is read with.
use crate::push_deadline_and_patience_settings::positive_seconds;
/// Import the measured duration the pauses are counted in.
use std::time::Duration;

/// Environment variable that sets,
/// in whole seconds,
/// the pause before the first new try of a stopped push.
pub const RETRY_VARIABLE: &str = "CLI_GIT_PUSH_RETRY_SECONDS";

/// Directory,
/// under the service instance's journal,
/// that holds the plan of the pushes waiting for a new try,
/// so that a restarted
/// service keeps trying them.
pub const PLAN_DIRECTORY: &str = "push-retries";

/// The pause before the first new try when nothing configures it.
pub const FIRST_PAUSE: Duration = Duration::from_secs(10);

/// The longest pause between two tries:
/// a push that keeps failing is tried about this often for as long as it fails.
pub const LONGEST_PAUSE: Duration = Duration::from_secs(10 * 60);

/// What:
///  The first pause for a value of the variable.
/// Why:
///   A missing,
/// zero or malformed value falls back to the default,
/// never to a tight loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveSeconds(value) ?? FIRST_PAUSE_MS
/// ```
pub fn first_pause_from(value: Option<&std::ffi::OsStr>) -> Duration {
    let Some(text) = value else {
        return FIRST_PAUSE;
    };
    return positive_seconds(RETRY_VARIABLE, text)
        .unwrap_or(FIRST_PAUSE)
        .min(LONGEST_PAUSE);
}

/// What:
///  The pause before try number `attempt` (the first new try is number one):
/// the first pause,
/// doubled each time,
/// never past
///  the longest.
/// Why:
///   Short pauses mend a stall quickly;
/// longer ones keep a host that is offline for an hour from being asked every few seconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.min(first * 2 ** (attempt - 1), LONGEST)
/// ```
pub fn pause_before(
    attempt: u32,
    first: Duration,
) -> Duration {
    let doublings: u32 = attempt.saturating_sub(1).min(30);
    return first.saturating_mul(1_u32 << doublings).min(LONGEST_PAUSE);
}
