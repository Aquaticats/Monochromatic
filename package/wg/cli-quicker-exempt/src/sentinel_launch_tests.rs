//! Verifies that the sentinel starts outside the watcher's cgroup behind a program a confined system manager may execute,
//! falls back loudly when the manager refuses the unit or accepts it and nothing confirms,
//! and is called running only when it left a marker that names the watcher while its process runs.

/// The launch under test.
use crate::sentinel_launch::{
    confirm_running,
    launch_sentinel,
    path_breaks_env,
    sentinel_arguments,
    systemd_run_arguments,
    unit_name,
    Placement,
    SentinelRequest,
    SentinelSteps,
};
/// The marker a sentinel leaves.
use crate::sentinel_marker::SentinelMarker;
/// Arguments that need not be text.
use std::ffi::OsString;
/// Building a path from bytes.
use std::os::unix::ffi::OsStringExt;
/// Shared recording between the test and the steps.
use std::cell::{Cell, RefCell};
/// Fixture paths.
use std::path::Path;
/// Waiting between checks.
use std::time::Duration;

/// The arguments as text for a recording,
/// where bytes that are not text may be replaced.
fn text(arguments: &[OsString]) -> String {
    return arguments.iter().map(|argument| return argument.to_string_lossy().into_owned()).collect::<Vec<String>>().join(" ");
}

/// Steps that record what they were asked and answer from a script.
struct FakeSteps {
    /// Events in the order they happened.
    events: RefCell<Vec<String>>,
    /// Whether starting as a service fails.
    service_fails: bool,
    /// Whether starting detached fails.
    detached_fails: bool,
    /// Why the unit cannot be shown to be stopped,
    /// or nothing when stopping works.
    stop_error: Option<String>,
    /// The marker successive reads answer while the sentinel started as a service,
    /// the last one repeating.
    service_markers: Vec<Option<SentinelMarker>>,
    /// The marker successive reads answer after the sentinel started detached,
    /// the last one repeating.
    detached_markers: Vec<Option<SentinelMarker>>,
    /// Reads since the last start.
    reads: Cell<usize>,
    /// Reads in all.
    total_reads: Cell<usize>,
    /// Whether the detached start happened.
    detached_started: Cell<bool>,
    /// Whether the sentinel process runs.
    alive: bool,
}

/// The scripted steps.
impl SentinelSteps for FakeSteps {
    /// Records the arguments and fails when told to.
    fn start_as_service(&self, arguments: &[OsString]) -> Result<(), String> {
        self.events.borrow_mut().push(format!("service {}", text(arguments)));
        if self.service_fails {
            return Err("systemd-run exit status: 1".to_string());
        }
        return Ok(());
    }

    /// Records the unit that was stopped and answers as told.
    fn stop_service(&self, unit: &str) -> Result<(), String> {
        self.events.borrow_mut().push(format!("stop {unit}"));
        return match &self.stop_error {
            Some(why) => Err(why.clone()),
            None => Ok(()),
        };
    }

    /// Records the program and arguments and fails when told to.
    fn start_detached(&self, executable: &Path, arguments: &[OsString]) -> Result<(), String> {
        self.events.borrow_mut().push(format!("detached {} {}", executable.display(), text(arguments)));
        self.detached_started.set(true);
        self.reads.set(0);
        if self.detached_fails {
            return Err("spawn failed".to_string());
        }
        return Ok(());
    }

    /// Answers the next scripted marker of the phase.
    fn read_marker(&self, _key: &str) -> Option<SentinelMarker> {
        self.total_reads.set(self.total_reads.get() + 1);
        let script = if self.detached_started.get() { &self.detached_markers } else { &self.service_markers };
        let index = self.reads.get();
        self.reads.set(index + 1);
        let answer = script.get(index.min(script.len().saturating_sub(1)))?;
        return answer.as_ref().map(|marker| return SentinelMarker { watcher_pid: marker.watcher_pid, pid: marker.pid, start_time: marker.start_time });
    }

    /// Records the removal.
    fn remove_marker(&self, _key: &str) {
        self.events.borrow_mut().push("remove marker".to_string());
    }

    /// Answers as told.
    fn is_alive(&self, _pid: i32, _start_time: u64) -> bool {
        return self.alive;
    }

    /// Does not wait.
    fn pause(&self, _duration: Duration) {}
}

/// Builds steps from the scenario.
fn steps(service_fails: bool, detached_fails: bool, service_markers: Vec<Option<SentinelMarker>>, detached_markers: Vec<Option<SentinelMarker>>) -> FakeSteps {
    return FakeSteps {
        events: RefCell::new(Vec::new()),
        service_fails,
        detached_fails,
        stop_error: None,
        service_markers,
        detached_markers,
        reads: Cell::new(0),
        total_reads: Cell::new(0),
        detached_started: Cell::new(false),
        alive: true,
    };
}

/// A marker that names the watcher under test.
fn marker() -> Option<SentinelMarker> {
    return Some(SentinelMarker { watcher_pid: 4242, pid: 777, start_time: 5 });
}

/// The request under test.
fn request(executable: &Path) -> SentinelRequest<'_> {
    return SentinelRequest { key: "wgtest", watcher_pid: 4242, watcher_start_time: 9001, uid: 1000, gid: 1001, executable };
}

/// The program path used by the scenarios.
const EXECUTABLE: &str = "/usr/bin/wg-quicker-exempt";

/// The service starts outside the watcher's cgroup,
/// collected,
/// with `systemd-oomd` asked to omit it,
/// and runs this program behind `/usr/bin/env`,
/// which a confined system manager may execute,
/// with the hidden command and the watcher's identity after a separator.
#[test]
fn the_service_arguments_name_the_unit_the_oom_preference_the_trampoline_and_the_command() {
    let executable = Path::new(EXECUTABLE);
    let arguments = systemd_run_arguments(&request(executable));
    assert!(arguments.contains(&OsString::from("--collect")), "{arguments:?}");
    assert!(arguments.contains(&OsString::from("--no-ask-password")), "{arguments:?}");
    assert!(arguments.contains(&OsString::from("--unit=wg-quicker-exempt-sentinel-4242-9001")), "{arguments:?}");
    assert!(arguments.contains(&OsString::from("--property=ManagedOOMPreference=omit")), "{arguments:?}");
    let separator = arguments.iter().position(|argument| return argument == "--").expect("a separator");
    assert!(arguments[..separator].iter().all(|argument| return argument.to_string_lossy().starts_with("--")), "{arguments:?}");
    assert_eq!(
        arguments[separator + 1..],
        ["/usr/bin/env", "/usr/bin/wg-quicker-exempt", "__sentinel", "wgtest", "4242", "9001", "1000", "1001"].map(OsString::from)
    );
    assert_eq!(unit_name(4242, 9001), "wg-quicker-exempt-sentinel-4242-9001");
    assert_eq!(sentinel_arguments(&request(executable))[0], OsString::from("__sentinel"));
}

/// A confirmed service start removes the old marker first and reports nothing.
#[test]
fn a_service_that_confirms_is_started_after_the_old_marker_is_removed_and_reports_nothing() {
    let executable = Path::new(EXECUTABLE);
    let fake = steps(false, false, vec![marker()], vec![]);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, Some(Placement::Service));
    assert!(outcome.confirmed);
    assert!(outcome.reports.is_empty(), "{:?}", outcome.reports);
    let events = fake.events.borrow();
    assert_eq!(events[0], "remove marker");
    assert!(events[1].starts_with("service --quiet --collect"), "{events:?}");
    assert!(!events.iter().any(|event| return event.starts_with("detached") || event.starts_with("stop")), "{events:?}");
}

/// A refused service falls back to a detached sentinel and says what that costs.
#[test]
fn a_refused_service_falls_back_beside_the_watcher_and_says_both_die_together() {
    let executable = Path::new(EXECUTABLE);
    let fake = steps(true, false, vec![], vec![marker()]);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, Some(Placement::BesideTheWatcher));
    assert!(outcome.confirmed);
    assert_eq!(outcome.reports.len(), 1);
    assert!(outcome.reports[0].contains("POLICY NOT IN FORCE"), "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("could not start as its own service (systemd-run exit status: 1)"), "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("systemd-oomd"), "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("ends both without a report"), "{:?}", outcome.reports);
    assert!(fake.events.borrow().iter().any(|event| return event.starts_with("detached /usr/bin/wg-quicker-exempt __sentinel wgtest 4242")));
}

/// A unit the manager accepted whose program then fails to execute never confirms:
/// the unit is stopped,
/// the old marker is removed again,
/// and the sentinel starts beside the watcher.
/// This is what happened on an SELinux host,
/// where the system manager accepted the unit and then could not execute the program.
#[test]
fn an_accepted_service_that_never_confirms_is_stopped_and_replaced_by_a_detached_sentinel() {
    let executable = Path::new(EXECUTABLE);
    let fake = steps(false, false, vec![None], vec![marker()]);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, Some(Placement::BesideTheWatcher));
    assert!(outcome.confirmed);
    assert_eq!(outcome.reports.len(), 1, "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("accepted as a service but did not confirm that it runs"), "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("journalctl --unit=wg-quicker-exempt-sentinel-4242-9001"), "{:?}", outcome.reports);
    assert!(!outcome.reports[0].contains("systemctl status"), "a collected unit has no status to show: {:?}", outcome.reports);
    assert!(outcome.reports[0].contains("ends both without a report"), "{:?}", outcome.reports);
    let events = fake.events.borrow();
    let order: Vec<&str> = events.iter().map(|event| return event.split(' ').next().unwrap_or("")).collect();
    assert_eq!(order, ["remove", "service", "stop", "remove", "detached"], "{events:?}");
    assert!(events.iter().any(|event| return event == "stop wg-quicker-exempt-sentinel-4242-9001"), "{events:?}");
}

/// When the detached sentinel does not confirm either,
/// that is said as well.
#[test]
fn when_the_detached_sentinel_does_not_confirm_either_that_is_said_too() {
    let executable = Path::new(EXECUTABLE);
    let fake = steps(false, false, vec![None], vec![None]);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, Some(Placement::BesideTheWatcher));
    assert!(!outcome.confirmed);
    assert_eq!(outcome.reports.len(), 2, "{:?}", outcome.reports);
    assert!(outcome.reports[1].contains("started beside the watcher but did not confirm"), "{:?}", outcome.reports);
}

/// Neither way starting is said twice and no marker is waited for.
#[test]
fn when_neither_way_starts_a_sentinel_it_says_so_and_does_not_wait() {
    let executable = Path::new(EXECUTABLE);
    let fake = steps(true, true, vec![], vec![]);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, None);
    assert!(!outcome.confirmed);
    assert_eq!(outcome.reports.len(), 2);
    assert!(outcome.reports[1].contains("could not start at all (spawn failed)"), "{:?}", outcome.reports);
    assert_eq!(fake.total_reads.get(), 0);
}

/// A marker confirms only when it names this watcher and its process runs.
#[test]
fn a_marker_of_another_watcher_or_a_dead_process_does_not_confirm() {
    let other = Some(SentinelMarker { watcher_pid: 1, pid: 777, start_time: 5 });
    assert!(!confirm_running(&steps(false, false, vec![other], vec![]), "wgtest", 4242));
    let mut dead = steps(false, false, vec![marker()], vec![]);
    dead.alive = false;
    assert!(!confirm_running(&dead, "wgtest", 4242));
    assert!(!confirm_running(&steps(false, false, vec![None], vec![]), "wgtest", 4242));
    let late = steps(false, false, vec![None, None, marker()], vec![]);
    assert!(confirm_running(&late, "wgtest", 4242));
    assert_eq!(late.reads.get(), 3);
}

/// The summary says where the sentinel runs and whether it confirmed.
#[test]
fn the_summary_says_where_the_sentinel_runs_and_whether_it_confirmed() {
    let executable = Path::new(EXECUTABLE);
    let confirmed = launch_sentinel(&steps(false, false, vec![marker()], vec![]), &request(executable)).summary();
    assert!(confirmed.contains("as its own service, outside the cgroup of the watcher"), "{confirmed}");
    assert!(confirmed.contains("and confirmed that it runs"), "{confirmed}");
    let beside = launch_sentinel(&steps(true, false, vec![], vec![None]), &request(executable)).summary();
    assert!(beside.contains("beside the watcher, in its cgroup"), "{beside}");
    assert!(beside.contains("has not confirmed"), "{beside}");
    let nowhere = launch_sentinel(&steps(true, true, vec![], vec![]), &request(executable)).summary();
    assert!(nowhere.contains("started nowhere"), "{nowhere}");
}

/// A program path with `=` in it would be read by `/usr/bin/env` as a variable assignment
/// (`env: '__sentinel': No such file or directory`,
/// exit 127),
/// so it is refused as a service with a loud line and the sentinel starts beside the watcher,
/// which does not go through `env`.
#[test]
fn a_program_path_with_an_equals_sign_is_refused_as_a_service_and_starts_beside_the_watcher() {
    let executable = Path::new("/home/user/a=b/wg-quicker-exempt");
    let fake = steps(false, false, vec![], vec![marker()]);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, Some(Placement::BesideTheWatcher));
    assert!(outcome.confirmed);
    assert_eq!(outcome.reports.len(), 1, "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("contains '='"), "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("variable assignment"), "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("/home/user/a=b/wg-quicker-exempt"), "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("ends both without a report"), "{:?}", outcome.reports);
    let events = fake.events.borrow();
    assert!(!events.iter().any(|event| return event.starts_with("service") || event.starts_with("stop")), "{events:?}");
    assert!(events.iter().any(|event| return event.starts_with("detached /home/user/a=b/wg-quicker-exempt __sentinel")), "{events:?}");
}

/// Only an `=` in the program path matters:
/// the arguments after the program are not read by `env`.
#[test]
fn only_an_equals_sign_in_the_program_path_breaks_env() {
    assert!(path_breaks_env(Path::new("/a=b/c")));
    assert!(path_breaks_env(Path::new("/a/b=")));
    assert!(path_breaks_env(Path::new(&OsString::from_vec(b"/opt/\xff=/c".to_vec()))));
    assert!(!path_breaks_env(Path::new("/a/b/c")));
    assert!(!path_breaks_env(Path::new("/a b/c")));
    assert!(!path_breaks_env(Path::new(&OsString::from_vec(b"/opt/\xff/c".to_vec()))));
}

/// A program path whose bytes are not valid UTF-8 reaches `systemd-run` byte for byte,
/// and is not replaced by the replacement character.
#[test]
fn a_program_path_that_is_not_text_reaches_systemd_run_unchanged() {
    let bytes = OsString::from_vec(b"/opt/\xff\xfe/wg-quicker-exempt".to_vec());
    assert!(bytes.to_str().is_none(), "the fixture is not valid UTF-8");
    let arguments = systemd_run_arguments(&request(Path::new(&bytes)));
    let separator = arguments.iter().position(|argument| return argument == "--").expect("a separator");
    assert_eq!(arguments[separator + 1], OsString::from("/usr/bin/env"));
    assert_eq!(arguments[separator + 2], bytes);
}

/// A unit that cannot be shown to be stopped is said to be possibly still running:
/// two sentinels may run and one death may be reported twice,
/// and the sentinel still starts beside the watcher.
#[test]
fn a_unit_that_cannot_be_shown_to_be_stopped_is_said_to_possibly_run_next_to_the_fallback() {
    let executable = Path::new(EXECUTABLE);
    let mut fake = steps(false, false, vec![None], vec![marker()]);
    fake.stop_error = Some("the unit is active (systemctl stop exit status: 1)".to_string());
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, Some(Placement::BesideTheWatcher));
    assert!(outcome.confirmed);
    assert_eq!(outcome.reports.len(), 2, "{:?}", outcome.reports);
    assert!(outcome.reports[1].contains("wg-quicker-exempt-sentinel-4242-9001 could not be shown to be stopped"), "{:?}", outcome.reports);
    assert!(outcome.reports[1].contains("the unit is active (systemctl stop exit status: 1)"), "{:?}", outcome.reports);
    assert!(outcome.reports[1].contains("two sentinels may run"), "{:?}", outcome.reports);
    assert!(outcome.reports[1].contains("reported twice"), "{:?}", outcome.reports);
    assert!(fake.events.borrow().iter().any(|event| return event.starts_with("detached")));
}
