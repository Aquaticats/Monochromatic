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
//! `env` reads a first argument that contains `=` as a variable assignment and not as a program,
//! and it has no option that prevents that,
//! so a program path with `=` in it is refused as a service and the sentinel starts beside the watcher instead.
//! A sentinel that was started is called running only after it left a marker that names the watcher
//! while its own process still exists,
//! because a unit the manager accepted can still fail to execute its program.
//! If the manager refuses the unit,
//! or accepts it and nothing confirms,
//! the unit is stopped and the sentinel starts beside the watcher,
//! and the report says what that costs.
//! If the unit cannot be shown to be gone,
//! the report says that two sentinels may run
//! and a death may be reported twice.

/// The marker of a running sentinel.
use crate::sentinel_marker::SentinelMarker;
/// The words of a loud report.
use crate::update_report::policy_failure;
/// The hidden command that runs the sentinel.
use crate::watcher_sentinel::SENTINEL_COMMAND;
/// Arguments that need not be text.
use std::ffi::OsString;
/// The bytes of a path.
use std::os::unix::ffi::OsStrExt;
/// Paths of the program.
use std::path::Path;
/// Waiting between checks.
use std::time::Duration;

/// Checks of the marker before the sentinel is called unconfirmed.
const CONFIRM_ATTEMPTS: u32 = 100;
/// Time between two checks of the marker.
const CONFIRM_DELAY: Duration = Duration::from_millis(20);
/// The first program of the service,
/// which the system manager may execute where it may not execute the sentinel's program itself.
const SERVICE_TRAMPOLINE: &str = "/usr/bin/env";

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
    fn start_as_service(&self, arguments: &[OsString]) -> Result<(), String>;
    /// Stops the transient service with this unit name,
    /// and checks that no process of it is left.
    /// An error says why the unit cannot be shown to be gone.
    fn stop_service(&self, unit: &str) -> Result<(), String>;
    /// Starts the sentinel as a detached process with these arguments after the program.
    fn start_detached(&self, executable: &Path, arguments: &[OsString]) -> Result<(), String>;
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

/// Whether `/usr/bin/env` would read the program path as a variable assignment,
/// which it does for any first argument that contains `=`.
pub fn path_breaks_env(executable: &Path) -> bool {
    return executable.as_os_str().as_bytes().contains(&b'=');
}

/// Arguments of the sentinel command after the program.
pub fn sentinel_arguments(request: &SentinelRequest<'_>) -> Vec<OsString> {
    return [
        SENTINEL_COMMAND.to_string(),
        request.key.to_string(),
        request.watcher_pid.to_string(),
        request.watcher_start_time.to_string(),
        request.uid.to_string(),
        request.gid.to_string(),
    ]
    .iter()
    .map(OsString::from)
    .collect();
}

/// Arguments of `systemd-run` that start the sentinel as its own collected service,
/// outside the cgroup of the watcher,
/// with `systemd-oomd` asked to omit it,
/// and with the program behind `/usr/bin/env` so a host that confines the system manager still runs it.
/// The program path is passed as the bytes it has,
/// not as text that would replace bytes that are not valid UTF-8.
pub fn systemd_run_arguments(request: &SentinelRequest<'_>) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = [
        "--quiet".to_string(),
        "--collect".to_string(),
        "--no-ask-password".to_string(),
        format!("--unit={}", unit_name(request.watcher_pid, request.watcher_start_time)),
        "--description=wg-quicker-exempt application watcher sentinel".to_string(),
        "--property=ManagedOOMPreference=omit".to_string(),
        "--".to_string(),
        SERVICE_TRAMPOLINE.to_string(),
    ]
    .iter()
    .map(OsString::from)
    .collect();
    arguments.push(request.executable.as_os_str().to_owned());
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

/// What a fallback to a sentinel beside the watcher costs,
/// said at the end of every report that leads to one.
const BESIDE_COST: &str = "so it starts beside the watcher: a kill of the watcher's whole cgroup, for example by systemd-oomd, then ends both without a report";

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
/// the program path cannot go behind `env`,
/// or the manager accepts the unit and the sentinel never confirms (its program may fail to execute after the start was accepted),
/// the unit is stopped and the sentinel starts beside the watcher.
/// If the unit cannot be shown to be gone,
/// a second report says that two sentinels may run.
/// The start of the watcher is not failed,
/// because the watcher itself is running.
pub fn launch_sentinel(steps: &dyn SentinelSteps, request: &SentinelRequest<'_>) -> LaunchOutcome {
    steps.remove_marker(request.key);
    let unit = unit_name(request.watcher_pid, request.watcher_start_time);
    let mut reports: Vec<String> = Vec::new();
    if path_breaks_env(request.executable) {
        reports.push(policy_failure(&format!(
            "the sentinel that reports the watcher's own death could not start as its own service \
(the program path {} contains '=', which /usr/bin/env would read as a variable assignment and not as a program), {BESIDE_COST}",
            request.executable.display()
        )));
        let (placement, confirmed) = start_beside_the_watcher(steps, request, &mut reports);
        return LaunchOutcome { placement, confirmed, reports };
    }
    if let Err(service_error) = steps.start_as_service(&systemd_run_arguments(request)) {
        reports.push(policy_failure(&format!(
            "the sentinel that reports the watcher's own death could not start as its own service ({service_error}), {BESIDE_COST}"
        )));
        let (placement, confirmed) = start_beside_the_watcher(steps, request, &mut reports);
        return LaunchOutcome { placement, confirmed, reports };
    }
    if confirm_running(steps, request.key, request.watcher_pid) {
        return LaunchOutcome { placement: Some(Placement::Service), confirmed: true, reports };
    }
    reports.push(policy_failure(&format!(
        "the sentinel that reports the watcher's own death was accepted as a service but did not confirm that it runs within {} ms \
(its program may have failed to execute; look at `journalctl --unit={unit}`), {BESIDE_COST}",
        confirm_wait_ms()
    )));
    if let Err(stop_error) = steps.stop_service(&unit) {
        reports.push(policy_failure(&format!(
            "the unit {unit} could not be shown to be stopped ({stop_error}), so it may still run next to the sentinel that starts beside the watcher: \
two sentinels may run, and one death of the watcher may be reported twice"
        )));
    }
    let (placement, confirmed) = start_beside_the_watcher(steps, request, &mut reports);
    return LaunchOutcome { placement, confirmed, reports };
}
