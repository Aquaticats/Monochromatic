//! What:
//!  The one instant of the warm request a control may take hold of:
//!  the moment the look at the processes has its answer and has not given it yet.
//! Why:
//!  A control of the look's ceiling needs a look that does not answer,
//! and a control of what the connection can still meet needs an instant between that answer and the connection.
//! Both states are made on this machine by another session's process or by the kernel,
//! and the rules that bind every run forbid a probe that holds such a state here
//! (`package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md`,
//! the FUSE incident),
//! so a control holds the request at this point instead of poisoning a process.
//!  The point is compiled into the release executable and only the explicit variable arms it,
//! as `service_fault.rs` does with its own,
//! and a request that carries no variable reads one environment variable and goes on at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = theAnswerWaitsWhereAControlSays(found); // found, after the control's own pause
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Environment variable naming where a control holds the answer:
/// the request leaves `<path>.answered` there and goes on once `<path>.release` is there,
/// or at `CONTROL_PARK_CEILING`.
/// Why:
///   A file is the control's own hand:
/// it says when the answer has been computed,
/// so the control does its own work (a kill, a start) in the instant between the look and the connection,
/// and nothing is decided by a sleep.
pub const PARK_VARIABLE: &str = "CLI_GIT_TEST_ONLY_WARM_SCAN_PARK";

/// The longest a control's park holds the answer.
/// Why:
///   A control that ended without releasing its request must leave no thread of this process parked for good;
/// a minute is longer than any control of this request takes,
/// and a control that needs longer leaves a `.release` file and the request goes on.
const CONTROL_PARK_CEILING: Duration = Duration::from_secs(60);

/// The pause between two looks for the release,
/// so a parked request costs no processor time.
const RELEASE_POLL: Duration = Duration::from_millis(2);

/// The file a control leaves or waits for, beside the path it named.
fn marker(where_the_control_holds_it: &Path, what: &str) -> PathBuf {
    let mut path: PathBuf = where_the_control_holds_it.to_path_buf();
    let name: String = format!("{}.{what}", where_the_control_holds_it.file_name().unwrap_or_default().to_string_lossy());
    path.set_file_name(name);
    return path;
}

/// What:
///  `found`,
///  after the control's own pause when one is armed.
/// Why:
///   The pause is the control's,
///  and the answer is the same before and after it:
/// what a control changes while the request waits is the machine the request then meets,
/// not the answer the look already gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theAnswerWaits(found) // found
/// ```
pub(crate) fn the_answer_waits_where_a_control_says(found: bool) -> bool {
    let Some(named) = std::env::var_os(PARK_VARIABLE) else { return found };
    let where_the_control_holds_it: PathBuf = PathBuf::from(named);
    let answered: PathBuf = marker(where_the_control_holds_it.as_path(), "answered");
    if let Err(error) = std::fs::write(answered.as_path(), b"") {
        crate::service_warn!("cannot leave the control's answered file {}: {error}", answered.display());
        return found;
    }
    let release: PathBuf = marker(where_the_control_holds_it.as_path(), "release");
    let until: Instant = Instant::now() + CONTROL_PARK_CEILING;
    while !release.exists() && Instant::now() < until {
        std::thread::sleep(RELEASE_POLL);
    }
    return found;
}

/// Controls of this point stay out of the release executable's own tests.
#[cfg(test)]
#[path = "service_warm_request_control_window_tests.rs"]
mod tests;
