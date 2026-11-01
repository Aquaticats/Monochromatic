//! The real steps of starting a sentinel:
//! `systemd-run` and `systemctl` through the bounded runner,
//! a detached process in a new session,
//! and the marker and process facts of the host.

/// Where the state of one watcher lives.
use crate::application_watch_state::watch_state_directory;
/// Commands with a time limit and a fixed search path.
use crate::bounded_command::run_bounded;
/// Process start times.
use crate::process_info::read_start_time;
/// The marker of a running sentinel.
use crate::sentinel_marker::{read_marker_in, SentinelMarker, MARKER_NAME};
/// The steps being implemented.
use crate::sentinel_launch::SentinelSteps;
/// Starting a detached process.
use std::os::unix::process::CommandExt;
/// Arguments that need not be text.
use std::ffi::OsString;
/// Paths of the marker.
use std::path::{Path, PathBuf};
/// Starting commands.
use std::process::{Command, Stdio};
/// Waiting between checks.
use std::time::Duration;

/// Time `systemd-run` and `systemctl` may take before they are ended.
const SYSTEMD_COMMAND_LIMIT: Duration = Duration::from_secs(10);

/// Whether the active state `systemctl` printed for a unit means no process of it runs:
/// a unit that ended,
/// failed,
/// or was collected and is no longer loaded (which prints `inactive`).
/// Any other state,
/// and any text that is not a state,
/// means it may still run.
pub fn unit_state_means_gone(printed: &str) -> bool {
    return matches!(printed.trim(), "inactive" | "failed");
}

/// The words of an input or output error.
fn io_message(error: std::io::Error) -> String {
    return error.to_string();
}

/// Starts a new session in the forked child so the sentinel is isolated from the invoking terminal.
fn start_new_session() -> std::io::Result<()> {
    // SAFETY: `setsid` is async-signal-safe and touches only the child's session state.
    if unsafe { libc::setsid() } < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// The real steps.
pub struct SystemSteps;

/// Starting real processes.
impl SentinelSteps for SystemSteps {
    /// Runs `systemd-run` with a cleared environment,
    /// a fixed search path,
    /// and a time limit.
    fn start_as_service(&self, arguments: &[OsString]) -> Result<(), String> {
        let output = run_bounded("systemd-run", arguments, SYSTEMD_COMMAND_LIMIT)?;
        if output.success {
            return Ok(());
        }
        return Err(format!("systemd-run {}: {} {}", output.status, output.stderr.trim(), output.stdout.trim()).trim_end().to_string());
    }

    /// Runs `systemctl stop`,
    /// then asks for the unit's state whatever `stop` said:
    /// a unit that already ended and was collected is no longer loaded,
    /// which `stop` reports as a failure although nothing runs.
    fn stop_service(&self, unit: &str) -> Result<(), String> {
        let stop = run_bounded("systemctl", &[OsString::from("stop"), OsString::from(unit)], SYSTEMD_COMMAND_LIMIT);
        let stop_said = match &stop {
            Ok(output) if output.success => "systemctl stop succeeded".to_string(),
            Ok(output) => format!("systemctl stop {}: {}", output.status, output.stderr.trim()),
            Err(error) => error.clone(),
        };
        let show = run_bounded(
            "systemctl",
            &[OsString::from("show"), OsString::from("--property=ActiveState"), OsString::from("--value"), OsString::from(unit)],
            SYSTEMD_COMMAND_LIMIT,
        );
        return match show {
            Ok(output) if output.success && unit_state_means_gone(&output.stdout) => Ok(()),
            Ok(output) if output.success => Err(format!("the unit is {} ({stop_said})", output.stdout.trim())),
            Ok(output) => Err(format!("its state could not be read: systemctl show {}: {} ({stop_said})", output.status, output.stderr.trim())),
            Err(error) => Err(format!("its state could not be read: {error} ({stop_said})")),
        };
    }

    /// Starts the program in a new session with no standard streams.
    fn start_detached(&self, executable: &Path, arguments: &[OsString]) -> Result<(), String> {
        let mut command = Command::new(executable);
        command.args(arguments).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        // SAFETY: setsid is async-signal-safe and isolates the sentinel from the invoking terminal.
        unsafe {
            command.pre_exec(start_new_session);
        }
        return match command.spawn() {
            Ok(_child) => Ok(()),
            Err(error) => Err(io_message(error)),
        };
    }

    /// Reads the marker file in the state directory.
    fn read_marker(&self, key: &str) -> Option<SentinelMarker> {
        let directory: PathBuf = watch_state_directory(key).ok()?;
        return read_marker_in(&directory);
    }

    /// Removes the marker file.
    fn remove_marker(&self, key: &str) {
        if let Ok(directory) = watch_state_directory(key) {
            let _removed = std::fs::remove_file(directory.join(MARKER_NAME));
        }
    }

    /// Compares the start time of the process now.
    fn is_alive(&self, pid: i32, start_time: u64) -> bool {
        return read_start_time(Path::new("/proc"), pid) == Some(start_time);
    }

    /// Sleeps.
    fn pause(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}
