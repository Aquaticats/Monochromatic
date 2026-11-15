//! What:
//!  The one instant of the service's start a control may take hold of:
//!  the step it is about to take on the way to reading its queue,
//!  named by the caller.
//! Why:
//!  A control of what the first commit after a start waits for needs a step that does not return,
//! and the state that makes one on this machine is a disk of the account's own taken by another program's work.
//! The rules that bind every run forbid a probe that stalls a real drive
//! (`package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md`),
//! so a control holds this step instead of poisoning a disk.
//!  The point is compiled into the release executable and only the explicit variable arms it,
//! as `service_fault.rs` does with its own,
//! and a service that carries no variable reads one environment variable and goes on at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! whereAControlHoldsAStartStep(step); takeTheStep();
//! ```

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Environment variable naming where a control holds a start step:
/// `<step>:<path>`,
/// and the service leaves `<path>.reached` there and goes on once `<path>.release` is there,
/// or at `CONTROL_PARK_CEILING`.
/// Why:
///   The value carries the step's own name, so one control can hold one step of a start and leave the others alone,
///  and a file is the control's own hand:
/// it says when the service reached the step,
/// so the control makes the commit it measures in that instant,
/// and nothing is decided by a sleep.
pub const HOLD_VARIABLE: &str = "CLI_GIT_TEST_ONLY_START_STEP_HOLD";

/// The longest a control's hold keeps a start step from ending.
/// Why:
///   A control that ended without releasing its hold must leave no thread of the service parked for good;
/// a minute is longer than any control of one start takes,
/// and a control that needs longer leaves a `.release` file and the service goes on.
const CONTROL_PARK_CEILING: Duration = Duration::from_secs(60);

/// The pause between two looks for the release,
/// so a held step costs no processor time.
const RELEASE_POLL: Duration = Duration::from_millis(2);

/// The file a control leaves or waits for, beside the path the variable named.
fn marker(where_the_control_holds_it: &Path, what: &str) -> PathBuf {
    return where_the_control_holds_it.with_file_name(format!("{}.{what}", where_the_control_holds_it.file_name().unwrap_or_default().to_string_lossy()));
}

/// What:
///  Waits where a control says,
///  when the variable names this step,
///  and goes on at once otherwise.
/// Why:
///   The hold is the control's,
///  and the step taken after it is the step the service would have taken anyway:
/// what a control changes while the service waits is when the step happens,
/// never what it does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whereAControlHoldsAStartStep('it reads the records of old finished batches');
/// ```
pub(crate) fn the_start_step_waits_where_a_control_says(step: &str) {
    let named: Option<std::ffi::OsString> = std::env::var_os(HOLD_VARIABLE);
    return the_start_step_waits_where_a_control_says_within(named.as_deref(), step, CONTROL_PARK_CEILING);
}

/// `the_start_step_waits_where_a_control_says` with the value and the ceiling a control states,
///  so its own case holds nothing of the process's environment and waits out no minute.
pub(crate) fn the_start_step_waits_where_a_control_says_within(place: Option<&OsStr>, step: &str, ceiling: Duration) {
    let Some(named) = place else { return };
    let Some(text) = named.to_str() else { return };
    let Some((held_step, path)) = text.split_once(':') else { return };
    if held_step != step {
        return;
    }
    let where_the_control_holds_it: PathBuf = PathBuf::from(path);
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
#[path = "service_start_step_hold_control_window_tests.rs"]
mod tests;
