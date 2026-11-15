//! What:
//!  The one instant of a request's decision a control may take hold of:
//!  the moment the landing board is about to publish a request's decision as a file in the journal's
//!  `busy` directory.
//! Why:
//!  A control of the wait a commit bears before its group begins needs a write that does not return,
//! and the state that makes one on this machine is a disk of the account's own taken by another program's work.
//! The rules that bind every run forbid a probe that stalls a real drive
//! (`package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md`),
//! so a control holds this write instead of poisoning a disk.
//!  The point is compiled into the release executable and only the explicit variable arms it,
//! as `service_fault.rs` does with its own,
//! and a request that carries no variable reads one environment variable and goes on at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! whereAControlHoldsTheRecord(); publishTheDecision(); // the publish, after the control's own pause
//! ```

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Environment variable naming where a control holds the write of a request's decision:
/// the board leaves `<path>.reached` there and goes on once `<path>.release` is there,
/// or at `CONTROL_PARK_CEILING`.
/// Why:
///   A file is the control's own hand:
/// it says when the board has reached the write,
/// so the control makes the commit it measures in the instant the record is being written,
/// and nothing is decided by a sleep.
pub const HOLD_VARIABLE: &str = "CLI_GIT_TEST_ONLY_DECISION_RECORD_HOLD";

/// The longest a control's hold keeps the decision unwritten.
/// Why:
///   A control that ended without releasing its hold must leave no thread of the service parked for good;
/// a minute is longer than any control of one commit takes,
/// and a control that needs longer leaves a `.release` file and the board goes on.
const CONTROL_PARK_CEILING: Duration = Duration::from_secs(60);

/// The pause between two looks for the release,
/// so a held record costs no processor time.
const RELEASE_POLL: Duration = Duration::from_millis(2);

/// The file a control leaves or waits for, beside the path it named.
fn marker(where_the_control_holds_it: &Path, what: &str) -> PathBuf {
    return where_the_control_holds_it.with_file_name(format!("{}.{what}", where_the_control_holds_it.file_name().unwrap_or_default().to_string_lossy()));
}

/// What:
///  Waits where a control says, when the variable arms one,
///  before the decision of a request is published.
/// Why:
///   The hold is the control's,
///  and the record written after it is the record the board would have written anyway:
/// what a control changes while the board waits is when the record reaches the journal,
/// never what it says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whereAControlHoldsTheRecord(); publishTheDecision();
/// ```
pub(crate) fn the_decision_record_waits_where_a_control_says() {
    let named: Option<std::ffi::OsString> = std::env::var_os(HOLD_VARIABLE);
    return the_decision_record_waits_where_a_control_says_within(named.as_deref(), CONTROL_PARK_CEILING);
}

/// `the_decision_record_waits_where_a_control_says` with the place and the ceiling a control states,
///  so its own case holds nothing of the process's environment and waits out no minute.
pub(crate) fn the_decision_record_waits_where_a_control_says_within(named: Option<&OsStr>, ceiling: Duration) {
    let Some(named) = named else { return };
    let where_the_control_holds_it: PathBuf = PathBuf::from(named);
    let reached: PathBuf = marker(where_the_control_holds_it.as_path(), "reached");
    if let Err(error) = std::fs::write(reached.as_path(), b"") {
        crate::service_warn!("cannot leave the control's reached file {}: {error}", reached.display());
        return;
    }
    let release: PathBuf = marker(where_the_control_holds_it.as_path(), "release");
    let until: Instant = Instant::now() + ceiling;
    while !release.exists() && Instant::now() < until {
        std::thread::sleep(RELEASE_POLL);
    }
}

/// Controls of this point stay out of the release executable's own tests.
#[cfg(test)]
#[path = "service_decision_record_hold_control_window_tests.rs"]
mod tests;
