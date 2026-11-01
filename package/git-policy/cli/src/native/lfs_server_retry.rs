//! What:
//!  When a failed `git-lfs push` is tried again: the server-error classification, the delays and the notice.
//! Why:
//!  The LFS server of a repository can answer HTTP 5xx for a moment, and `git-lfs` treats such an answer as fatal for the
//!  run (`lfshttp/errors.go`: a status above 499 other than 501, 507 and 509 becomes a fatal error), so a push that was
//!  fine a second later fails. Uploads are content addressed, so a second run never sends an object twice. The retry is
//!  bounded twice over: a short fixed list of delays, and a classification from the program's own output that never
//!  retries a refused credential, a missing local object, or any failure that is not a server error.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const delay = isServerError(output) ? delays[retriesDone] : undefined; // undefined: stop and explain
//! ```

/// The invocation's environment values.
use super::child_environment::environment_value;
/// Owned OS text for the environment.
use std::ffi::OsString;
/// Delays.
use std::time::Duration;

/// How long to wait before each retry of a push that failed with a server error: two retries, so three runs at most.
pub const RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(2), Duration::from_secs(5)];

/// The variable whose comma-separated millisecond list replaces [`RETRY_DELAYS`] (an empty value means no retry); it
///  exists so a test does not wait for the real delays.
pub const RETRY_DELAYS_VARIABLE: &str = "CLI_GIT_LFS_RETRY_DELAYS_MILLISECONDS";

/// The most retries a configured list may hold, so a mistyped list cannot make the step unbounded.
pub const MAX_CONFIGURED_RETRIES: usize = 5;

/// The longest single delay a configured list may hold.
pub const MAX_CONFIGURED_DELAY: Duration = Duration::from_secs(60);

/// Lines that name a failure retrying cannot fix: refused credentials, a missing object or repository, a refused request,
///  a corrupt or missing local object, locked files.
const PERMANENT_MARKERS: [&str; 7] = [
    "Authorization error",
    "Repository or object not found",
    "Client error",
    "Unprocessable entity",
    "(missing)",
    "(corrupt)",
    "locked files",
];

/// What the message of a 500 answer starts with (`lfshttp/errors.go`, `defaultError`).
const SERVER_ERROR_500: &str = "Server error: ";

/// What the message of any other retryable 5xx answer holds (`lfshttp/errors.go`, `defaultError`).
const SERVER_ERROR_OTHER: &str = "Server error ";

/// What precedes the status code in that message.
const FROM_HTTP: &str = " from HTTP ";

/// Whether the text after `from HTTP ` starts with a 5xx status.
fn starts_with_server_status(rest: &str) -> bool {
    let digits: &str = rest.get(..3).unwrap_or_default();
    return digits.len() == 3 && digits.starts_with('5') && digits.bytes().all(|byte| {
        return byte.is_ascii_digit();
    });
}

/// Whether one line is a server-error message of `git-lfs`.
fn line_is_server_error(line: &str) -> bool {
    if line.contains(SERVER_ERROR_500) {
        return true;
    }
    return line.contains(SERVER_ERROR_OTHER)
        && line.split(FROM_HTTP).skip(1).any(starts_with_server_status);
}

/// What:
///  Whether the output of a failed `git-lfs push` says the LFS server answered with a retryable server error, and says
///  nothing that retrying cannot fix.
/// Why:
///   A positive match on the program's own words (`Server error: <url>` for 500, `Server error <url> from HTTP 5xx` for the
///  other retryable 5xx) keeps every unknown failure terminal. A line naming a refused credential, a missing object or a
///  locked file anywhere in the output makes the whole failure terminal, because a retry would meet it again.
///  The words are `git-lfs`'s English messages; a translated message is not retried, which fails toward the explanation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lines.some(isServerErrorLine) && !lines.some((line) => PERMANENT_MARKERS.some((marker) => line.includes(marker)))
/// ```
pub fn is_server_error(output: &str) -> bool {
    let permanent: bool = output.lines().any(|line| {
        return PERMANENT_MARKERS.iter().any(|marker| {
            return line.contains(marker);
        });
    });
    return !permanent && output.lines().any(line_is_server_error);
}

/// What:
///  The delays to use: the configured millisecond list when the variable is present and well formed, else
///  [`RETRY_DELAYS`].
/// Why:
///   A test injects short delays through the environment the wrapper was started with. A list is cut to
///  [`MAX_CONFIGURED_RETRIES`] entries of at most [`MAX_CONFIGURED_DELAY`] each, and a value that is not a list of
///  whole numbers is ignored, so the step stays bounded whatever the variable holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retryDelays(env): number[]; // milliseconds
/// ```
pub fn retry_delays(environment: &[(OsString, OsString)]) -> Vec<Duration> {
    let Some(value) = environment_value(environment, RETRY_DELAYS_VARIABLE) else {
        return RETRY_DELAYS.to_vec();
    };
    let text: String = value.to_string_lossy().trim().to_owned();
    if text.is_empty() {
        return Vec::new();
    }
    let parsed: Option<Vec<Duration>> = text
        .split(',')
        .map(|entry| {
            return entry.trim().parse::<u64>().ok().map(Duration::from_millis);
        })
        .collect();
    return parsed.map_or_else(
        || {
            return RETRY_DELAYS.to_vec();
        },
        |delays| {
            return delays
                .into_iter()
                .take(MAX_CONFIGURED_RETRIES)
                .map(|delay| {
                    return delay.min(MAX_CONFIGURED_DELAY);
                })
                .collect();
        },
    );
}

/// What:
///  The delay before the next run of a push that just failed with `output`, or nothing when it is not to be retried.
/// Why:
///   `retries_done` runs have already been repeated; the list is the whole allowance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nextDelay(output: string, retriesDone: number, delays: number[]): number | undefined;
/// ```
pub fn next_delay(output: &str, retries_done: usize, delays: &[Duration]) -> Option<Duration> {
    let delay: Duration = delays.get(retries_done).copied()?;
    return is_server_error(output).then_some(delay);
}

/// What:
///  The line printed before a retry.
/// Why:
///   The caller sees why the run is repeated, how long it waits and how many repeats are left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: git-lfs push failed with a server error; trying again in ${seconds} seconds (retry ${n} of ${total}).`
/// ```
pub fn retry_notice(delay: Duration, retries_done: usize, total: usize) -> String {
    return format!(
        "cli-git: `git-lfs push` failed with a server error, so it is run again in {:.1} seconds (retry {} of {total}); \
         objects the server already has are not sent twice.\n",
        delay.as_secs_f64(),
        retries_done + 1,
    );
}

/// The classification and the delays are controlled in the test module.
#[cfg(test)]
#[path = "lfs_server_retry_tests.rs"]
mod tests;
