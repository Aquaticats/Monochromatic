//! Starts the sentinel of the application watcher outside the watcher's cgroup,
//! and confirms that it runs.
//! The watcher and a sentinel in the same cgroup die together when the whole cgroup is killed,
//! for example by `systemd-oomd`,
//! and then nothing reports it.
//! The sentinel therefore starts as its own transient service of the system manager through `systemd-run`,
//! with `ManagedOOMPreference=omit` so `systemd-oomd` does not choose that service itself.
//! The program starts behind `/usr/bin/env`:
//! on an SELinux host the system manager (`init_t`) may not execute files under a home directory
//! or files labelled for containers,
//! and the toolchain and this program's build sit in such places,
//! but it may execute `/usr/bin/env`,
//! after which the service runs in `unconfined_service_t`,
//! which may.
//! See `doc/troubleshooting/selinux-denies-system-service-exec-of-home-files.md`.
//! A sentinel that was started is called running only after it left a marker that names the watcher
//! while its own process still exists,
//! because a unit the manager accepted can still fail to execute its program.
//! If the manager refuses the unit,
//! or accepts it and nothing confirms,
//! the unit is stopped and the sentinel starts beside the watcher,
//! and the report says what that costs.

/// Where the state of one watcher lives.
use crate::application_watch_state::watch_state_directory;
/// Commands with a time limit and a fixed search path.
use crate::bounded_command::run_bounded;
/// Process start times.
use crate::process_info::read_start_time;
/// The words of a loud report.
use crate::update_report::policy_failure;
/// The hidden command that runs the sentinel.
use crate::watcher_sentinel::SENTINEL_COMMAND;
/// Writing a file without following a link.
use std::io::Write;
/// Creating a file with a mode.
use std::os::unix::fs::OpenOptionsExt;
/// Starting a detached process.
use std::os::unix::process::CommandExt;
/// Paths of the marker.
use std::path::{Path, PathBuf};
/// Starting commands.
use std::process::{Command, Stdio};
/// Waiting between checks.
use std::time::Duration;

/// Checks of the marker before the sentinel is called unconfirmed.
const CONFIRM_ATTEMPTS: u32 = 100;
/// Time between two checks of the marker.
const CONFIRM_DELAY: Duration = Duration::from_millis(20);
/// File name of the marker inside the state directory of the watcher.
const MARKER_NAME: &str = "sentinel";
/// Time `systemd-run` and `systemctl` may take before they are ended.
const SYSTEMD_COMMAND_LIMIT: Duration = Duration::from_secs(10);
/// The first program of the service,
/// which the system manager may execute where it may not execute the sentinel's program itself.
const SERVICE_TRAMPOLINE: &str = "/usr/bin/env";

/// What a running sentinel leaves for the process that started it.
#[derive(Debug, Eq, PartialEq)]
pub struct SentinelMarker {
    /// Process identifier of the watcher the sentinel waits for.
    pub watcher_pid: i32,
    /// Process identifier of the sentinel.
    pub pid: i32,
    /// Start time of the sentinel process,
    /// so a reused pid is not mistaken for it.
    pub start_time: u64,
}

/// Parses the text of a marker file.
pub fn parse_marker(text: &str) -> Option<SentinelMarker> {
    let mut words = text.split_whitespace();
    let watcher_pid = words.next()?.parse().ok()?;
    let pid = words.next()?.parse().ok()?;
    let start_time = words.next()?.parse().ok()?;
    if words.next().is_some() {
        return None;
    }
    return Some(SentinelMarker { watcher_pid, pid, start_time });
}

/// Writes the marker of the running sentinel into `directory`.
/// The text goes to a new file that is renamed over the marker,
/// so a link planted at the marker's name is replaced and never followed.
pub fn write_marker_in(directory: &Path, watcher_pid: i32) -> std::io::Result<()> {
    let own = i32::try_from(std::process::id()).map_err(std::io::Error::other)?;
    let Some(start_time) = read_start_time(Path::new("/proc"), own) else {
        return Err(std::io::Error::other("the sentinel process is not in procfs"));
    };
    let temporary = directory.join(format!("{MARKER_NAME}.{own}.tmp"));
    let _removed = std::fs::remove_file(&temporary);
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
    file.write_all(format!("{watcher_pid} {own} {start_time}\n").as_bytes())?;
    file.sync_all()?;
    return std::fs::rename(&temporary, directory.join(MARKER_NAME));
}

/// Reads the marker in `directory`,
/// or nothing when there is none or it is malformed.
pub fn read_marker_in(directory: &Path) -> Option<SentinelMarker> {
    let text = std::fs::read_to_string(directory.join(MARKER_NAME)).ok()?;
    return parse_marker(&text);
}

/// Writes the marker of the running sentinel for the watcher stored under `key`.
pub fn write_marker(key: &str, watcher_pid: i32) -> std::io::Result<()> {
    return write_marker_in(&watch_state_directory(key)?, watcher_pid);
}

/// The watcher the sentinel is started for.
pub struct SentinelRequest<'a> {
    /// Key of the watcher.
    pub key: &'a str,
    /// Process identifier of the watcher.
    pub watcher_pid: i32,
    /// Start time of the watcher process.
    pub watcher_start_time: u64,
    /// User the watcher serves.
    pub uid: u32,
    /// Primary group of that user.
    pub gid: u32,
    /// The program that runs the sentinel,
    /// which is this program.
    pub executable: &'a Path,
}

/// Where the sentinel was started.
#[derive(Debug, Eq, PartialEq)]
pub enum Placement {
    /// As its own transient service of the system manager.
    Service,
    /// Beside the watcher,
    /// in the cgroup of the process that started it.
    BesideTheWatcher,
}

/// What starting the sentinel led to.
#[derive(Debug)]
pub struct LaunchOutcome {
    /// Where it was started,
    /// or nothing when it could not be started.
    pub placement: Option<Placement>,
    /// Whether it confirmed that it runs.
    pub confirmed: bool,
    /// Loud reports to print.
    pub reports: Vec<String>,
}

/// What the start of a sentinel led to,
/// in words for the terminal that ran `watch-start`.
impl LaunchOutcome {
    /// One line saying where the sentinel runs and whether it confirmed.
    pub fn summary(&self) -> String {
        let place = match self.placement {
            Some(Placement::Service) => "as its own service, outside the cgroup of the watcher",
            Some(Placement::BesideTheWatcher) => "beside the watcher, in its cgroup",
            None => "nowhere",
        };
        let state = if self.confirmed { "and confirmed that it runs" } else { "and has not confirmed that it runs" };
        return format!("wg-quicker-exempt: the sentinel that reports the watcher's own death was started {place} {state}");
    }
}

/// The steps of starting a sentinel,
/// replaceable in tests.
pub trait SentinelSteps {
    /// Starts the sentinel as a transient service with these `systemd-run` arguments.
    fn start_as_service(&self, arguments: &[String]) -> Result<(), String>;
    /// Stops the transient service with this unit name,
    /// when it still exists.
    fn stop_service(&self, unit: &str);
    /// Starts the sentinel as a detached process with these arguments after the program.
    fn start_detached(&self, executable: &Path, arguments: &[String]) -> Result<(), String>;
    /// Reads the marker of the sentinel of the watcher under `key`.
    fn read_marker(&self, key: &str) -> Option<SentinelMarker>;
    /// Removes a marker an earlier sentinel left.
    fn remove_marker(&self, key: &str);
    /// Whether the process with this pid and start time runs.
    fn is_alive(&self, pid: i32, start_time: u64) -> bool;
    /// Waits between two checks.
    fn pause(&self, duration: Duration);
}

/// Name of the transient service that holds the sentinel of one watcher.
pub fn unit_name(watcher_pid: i32, watcher_start_time: u64) -> String {
    return format!("wg-quicker-exempt-sentinel-{watcher_pid}-{watcher_start_time}");
}

/// Arguments of the sentinel command after the program.
pub fn sentinel_arguments(request: &SentinelRequest<'_>) -> Vec<String> {
    return vec![
        SENTINEL_COMMAND.to_string(),
        request.key.to_string(),
        request.watcher_pid.to_string(),
        request.watcher_start_time.to_string(),
        request.uid.to_string(),
        request.gid.to_string(),
    ];
}

/// Arguments of `systemd-run` that start the sentinel as its own collected service,
/// outside the cgroup of the watcher,
/// with `systemd-oomd` asked to omit it,
/// and with the program behind `/usr/bin/env` so a host that confines the system manager still runs it.
pub fn systemd_run_arguments(request: &SentinelRequest<'_>) -> Vec<String> {
    let mut arguments: Vec<String> = vec![
        "--quiet".to_string(),
        "--collect".to_string(),
        "--no-ask-password".to_string(),
        format!("--unit={}", unit_name(request.watcher_pid, request.watcher_start_time)),
        "--description=wg-quicker-exempt application watcher sentinel".to_string(),
        "--property=ManagedOOMPreference=omit".to_string(),
        "--".to_string(),
        SERVICE_TRAMPOLINE.to_string(),
        request.executable.display().to_string(),
    ];
    arguments.extend(sentinel_arguments(request));
    return arguments;
}

/// Waits until the sentinel left a marker that names this watcher and its process runs.
pub fn confirm_running(steps: &dyn SentinelSteps, key: &str, watcher_pid: i32) -> bool {
    for _attempt in 0..CONFIRM_ATTEMPTS {
        if let Some(marker) = steps.read_marker(key)
            && marker.watcher_pid == watcher_pid
            && steps.is_alive(marker.pid, marker.start_time)
        {
            return true;
        }
        steps.pause(CONFIRM_DELAY);
    }
    return false;
}

/// The wait for a marker,
/// in milliseconds.
fn confirm_wait_ms() -> u128 {
    return u128::from(CONFIRM_ATTEMPTS) * CONFIRM_DELAY.as_millis();
}

/// Starts the sentinel beside the watcher and confirms it,
/// adding the loud reports for what did not go as intended.
fn start_beside_the_watcher(steps: &dyn SentinelSteps, request: &SentinelRequest<'_>, reports: &mut Vec<String>) -> (Option<Placement>, bool) {
    steps.remove_marker(request.key);
    if let Err(detached_error) = steps.start_detached(request.executable, &sentinel_arguments(request)) {
        reports.push(policy_failure(&format!(
            "the sentinel that reports the watcher's own death could not start at all ({detached_error}), \
so if the watcher is killed or runs out of memory nothing will say so"
        )));
        return (None, false);
    }
    let confirmed = confirm_running(steps, request.key, request.watcher_pid);
    if !confirmed {
        reports.push(policy_failure(&format!(
            "the sentinel that reports the watcher's own death was started beside the watcher but did not confirm that it runs within {} ms, \
so if the watcher is killed or runs out of memory nothing may say so",
            confirm_wait_ms()
        )));
    }
    return (Some(Placement::BesideTheWatcher), confirmed);
}

/// Starts the sentinel outside the cgroup of the watcher,
/// confirms that it runs,
/// and returns the loud reports for what did not go as intended.
/// If the manager refuses the unit,
/// or accepts it and the sentinel never confirms (its program may fail to execute after the start was accepted),
/// the unit is stopped and the sentinel starts beside the watcher.
/// The start of the watcher is not failed,
/// because the watcher itself is running.
pub fn launch_sentinel(steps: &dyn SentinelSteps, request: &SentinelRequest<'_>) -> LaunchOutcome {
    steps.remove_marker(request.key);
    let unit = unit_name(request.watcher_pid, request.watcher_start_time);
    let mut reports: Vec<String> = Vec::new();
    if let Err(service_error) = steps.start_as_service(&systemd_run_arguments(request)) {
        reports.push(policy_failure(&format!(
            "the sentinel that reports the watcher's own death could not start as its own service ({service_error}), \
so it starts beside the watcher: a kill of the watcher's whole cgroup, for example by systemd-oomd, then ends both without a report"
        )));
        let (placement, confirmed) = start_beside_the_watcher(steps, request, &mut reports);
        return LaunchOutcome { placement, confirmed, reports };
    }
    if confirm_running(steps, request.key, request.watcher_pid) {
        return LaunchOutcome { placement: Some(Placement::Service), confirmed: true, reports };
    }
    steps.stop_service(&unit);
    reports.push(policy_failure(&format!(
        "the sentinel that reports the watcher's own death was accepted as a service but did not confirm that it runs within {} ms \
(its program may have failed to execute; look at `journalctl --unit={unit}`), \
so it starts beside the watcher: a kill of the watcher's whole cgroup, for example by systemd-oomd, then ends both without a report",
        confirm_wait_ms()
    )));
    let (placement, confirmed) = start_beside_the_watcher(steps, request, &mut reports);
    return LaunchOutcome { placement, confirmed, reports };
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
    fn start_as_service(&self, arguments: &[String]) -> Result<(), String> {
        let output = run_bounded("systemd-run", arguments, SYSTEMD_COMMAND_LIMIT)?;
        if output.success {
            return Ok(());
        }
        return Err(format!("systemd-run {}: {} {}", output.status, output.stderr.trim(), output.stdout.trim()).trim_end().to_string());
    }

    /// Runs `systemctl stop` with the same limits and ignores a unit that is already gone.
    fn stop_service(&self, unit: &str) {
        let _stopped = run_bounded("systemctl", &["stop".to_string(), unit.to_string()], SYSTEMD_COMMAND_LIMIT);
    }

    /// Starts the program in a new session with no standard streams.
    fn start_detached(&self, executable: &Path, arguments: &[String]) -> Result<(), String> {
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
