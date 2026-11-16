//! What:
//!  The one instant of the service's last sweep a control may take hold of:
//!  the moment the shutdown asks for the markers made ahead to be removed and has not removed them yet.
//! Why:
//!  A control of the end of a stopped service needs a sweep that does not answer,
//!  and the rules that bind every run forbid a probe that holds a real file system here
//!  (`package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md`,
//!  the FUSE incident),
//!  so a control holds the sweep at this point instead of poisoning a mount.
//!  The point is compiled into the release executable and only the explicit variable arms it,
//!  as `service_fault.rs` and `service_warm_request_control_window.rs` do with their own,
//!  and a sweep that carries no variable reads one environment variable and goes on at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! theSweepWaitsWhereAControlSays(); // returns at once unless the control's variable names a path
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Environment variable naming where a control holds the sweep:
/// the sweep leaves `<path>.asked` there and removes nothing until `<path>.release` is there,
/// or at `CONTROL_PARK_CEILING`.
/// Why:
///   A file is the control's own hand:
/// it says when the shutdown has asked for the sweep,
/// so the control reads the end of the service at that instant without a sleep deciding anything,
/// and it can leave the release out to hold the sweep for the whole life of the process.
pub const PARK_VARIABLE: &str = "CLI_GIT_TEST_ONLY_SWEEP_PARK";

/// The longest a control's park holds the sweep.
/// Why:
///   A control that ended without releasing its sweep must leave no thread of this process parked for good;
/// a minute is longer than any control of the end of a service takes,
/// and a control that needs longer leaves a `.release` file and the sweep goes on.
const CONTROL_PARK_CEILING: Duration = Duration::from_secs(60);

/// The pause between two looks for the release,
/// so a parked sweep costs no processor time.
const RELEASE_POLL: Duration = Duration::from_millis(2);

/// The file a control leaves or waits for, beside the path it named.
fn marker(where_the_control_holds_it: &Path, what: &str) -> PathBuf {
    let mut path: PathBuf = where_the_control_holds_it.to_path_buf();
    let name: String = format!("{}.{what}", where_the_control_holds_it.file_name().unwrap_or_default().to_string_lossy());
    path.set_file_name(name);
    return path;
}

/// What:
///  Leaves the control's `asked` file and waits for its `release`,
///  when a control named where to hold the sweep.
/// Why:
///   The hold is the control's,
/// and the sweep is the same before and after it:
/// a control that holds the sweep proves what the end of the service does while the sweep does not answer,
/// and a service that carries no variable pays one environment read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theSweepWaits(); // the asked file, then the release file, then on
/// ```
pub fn the_sweep_waits_where_a_control_says() {
    let Some(named) = std::env::var_os(PARK_VARIABLE) else { return };
    let where_the_control_holds_it: PathBuf = PathBuf::from(named);
    let asked: PathBuf = marker(where_the_control_holds_it.as_path(), "asked");
    if let Err(error) = std::fs::write(asked.as_path(), b"") {
        crate::service_warn!("cannot leave the control's asked file {}: {error}", asked.display());
        return;
    }
    let release: PathBuf = marker(where_the_control_holds_it.as_path(), "release");
    let until: Instant = Instant::now() + CONTROL_PARK_CEILING;
    while !release.exists() && Instant::now() < until {
        std::thread::sleep(RELEASE_POLL);
    }
}

/// Controls of this point stay out of the release executable's own tests.
#[cfg(test)]
#[path = "service_sweep_control_window_tests.rs"]
mod tests;
