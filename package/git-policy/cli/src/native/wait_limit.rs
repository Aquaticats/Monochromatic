//! What:
//!  How long a command waits for the commit service's own hold on the index, and the variable that sets it.
//! Why:
//!  A hold that is alive but stuck would otherwise stall every later command with no output at all. The wait ends at a
//!  generous backstop (`CLI_GIT_LOCK_WAIT_SECONDS`, thirty minutes by default), read by the wrapper that waits and by the
//!  service that waits for a foreign `index.lock` before it lands a commit; it belongs in the environment of the unit that
//!  runs the service.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const limitMs = positiveSeconds(process.env.CLI_GIT_LOCK_WAIT_SECONDS) ?? DEFAULT_LIVE_WAIT_MS;
//! ```

/// Positive-seconds parsing shared with the push deadline.
use super::bounded_push::positive_seconds;
/// Borrowed OS text of the environment value.
use std::ffi::OsStr;
/// Time spans.
use std::time::Duration;

/// Environment variable that sets, in whole seconds, how long a command waits for a lock held by a live process.
pub const LIVE_WAIT_VARIABLE: &str = "CLI_GIT_LOCK_WAIT_SECONDS";

/// How long a command waits for a live holder when the variable is absent or unusable:
///  longer than any ordinary hook run, short enough that a stuck holder is noticed within the hour.
pub const DEFAULT_LIVE_WAIT: Duration = Duration::from_secs(30 * 60);

/// What:
///  The wait limit for a value of the variable.
/// Why:
///   A missing, zero or malformed value falls back to the default,
///  never to "no limit".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveSeconds(value) ?? DEFAULT_LIVE_WAIT_MS
/// ```
pub fn live_wait_limit_from(value: Option<&OsStr>) -> Duration {
    let Some(text) = value else {
        return DEFAULT_LIVE_WAIT;
    };
    return positive_seconds(LIVE_WAIT_VARIABLE, text).unwrap_or(DEFAULT_LIVE_WAIT);
}

/// The wait limit this process runs under: the variable in its environment, or the default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// liveWaitLimitMs(process.env.CLI_GIT_LOCK_WAIT_SECONDS)
/// ```
pub fn live_wait_limit() -> Duration {
    return live_wait_limit_from(std::env::var_os(LIVE_WAIT_VARIABLE).as_deref());
}

/// Wait limit controls stay out of the release executable.
#[cfg(test)]
#[path = "wait_limit_tests.rs"]
mod tests;
