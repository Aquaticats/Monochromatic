//! Verifies that the sentinel starts outside the watcher's cgroup behind a program a confined system manager may execute,
//! falls back loudly when the manager refuses the unit or accepts it and nothing confirms,
//! and is called running only when it left a marker that names the watcher while its process runs.

/// The launch under test.
use crate::sentinel_launch::{
    confirm_running,
    launch_sentinel,
    parse_marker,
    read_marker_in,
    sentinel_arguments,
    systemd_run_arguments,
    unit_name,
    write_marker_in,
    Placement,
    SentinelMarker,
    SentinelRequest,
    SentinelSteps,
};
/// Shared recording between the test and the steps.
use std::cell::{Cell, RefCell};
/// Fixture paths.
use std::path::Path;
/// Waiting between checks.
use std::time::Duration;

/// Steps that record what they were asked and answer from a script.
struct FakeSteps {
    /// Events in the order they happened.
    events: RefCell<Vec<String>>,
    /// Whether starting as a service fails.
    service_fails: bool,
    /// Whether starting detached fails.
    detached_fails: bool,
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
    fn start_as_service(&self, arguments: &[String]) -> Result<(), String> {
        self.events.borrow_mut().push(format!("service {}", arguments.join(" ")));
        if self.service_fails {
            return Err("systemd-run exit status: 1".to_string());
        }
        return Ok(());
    }

    /// Records the unit that was stopped.
    fn stop_service(&self, unit: &str) {
        self.events.borrow_mut().push(format!("stop {unit}"));
    }

    /// Records the program and arguments and fails when told to.
    fn start_detached(&self, executable: &Path, arguments: &[String]) -> Result<(), String> {
        self.events.borrow_mut().push(format!("detached {} {}", executable.display(), arguments.join(" ")));
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
    assert!(arguments.contains(&"--collect".to_string()), "{arguments:?}");
    assert!(arguments.contains(&"--no-ask-password".to_string()), "{arguments:?}");
    assert!(arguments.contains(&"--unit=wg-quicker-exempt-sentinel-4242-9001".to_string()), "{arguments:?}");
    assert!(arguments.contains(&"--property=ManagedOOMPreference=omit".to_string()), "{arguments:?}");
    let separator = arguments.iter().position(|argument| return argument == "--").expect("a separator");
    assert!(arguments[..separator].iter().all(|argument| return argument.starts_with("--")), "{arguments:?}");
    assert_eq!(
        arguments[separator + 1..],
        ["/usr/bin/env", "/usr/bin/wg-quicker-exempt", "__sentinel", "wgtest", "4242", "9001", "1000", "1001"]
    );
    assert_eq!(unit_name(4242, 9001), "wg-quicker-exempt-sentinel-4242-9001");
    assert_eq!(sentinel_arguments(&request(executable))[0], "__sentinel");
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

/// The marker is written with the pid and start time of the running process,
/// read back,
/// and absent when missing or malformed.
#[test]
fn the_marker_round_trips_and_malformed_text_is_absent() {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-marker-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("make the directory");
    assert_eq!(read_marker_in(&directory), None);
    write_marker_in(&directory, 4242).expect("write the marker");
    let read = read_marker_in(&directory).expect("read the marker back");
    assert_eq!(read.watcher_pid, 4242);
    assert_eq!(u32::try_from(read.pid).expect("positive"), std::process::id());
    assert!(read.start_time > 0);
    assert_eq!(parse_marker("1 2 3\n"), Some(SentinelMarker { watcher_pid: 1, pid: 2, start_time: 3 }));
    assert_eq!(parse_marker("1 2"), None);
    assert_eq!(parse_marker("1 2 3 4"), None);
    assert_eq!(parse_marker("a b c"), None);
    assert_eq!(parse_marker(""), None);
    std::fs::write(directory.join("sentinel"), "garbage").expect("write garbage");
    assert_eq!(read_marker_in(&directory), None);
}

/// A link planted at the marker's name is replaced by the marker and never followed,
/// and no temporary file is left behind.
#[test]
fn a_link_at_the_marker_name_is_replaced_and_not_followed() {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-marker-link-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("make the directory");
    let victim = directory.join("victim");
    std::fs::write(&victim, "precious").expect("write the victim");
    std::os::unix::fs::symlink(&victim, directory.join("sentinel")).expect("plant the link");
    write_marker_in(&directory, 4242).expect("write the marker");
    assert_eq!(std::fs::read_to_string(&victim).expect("read the victim"), "precious");
    assert!(!std::fs::symlink_metadata(directory.join("sentinel")).expect("the marker exists").file_type().is_symlink());
    assert_eq!(read_marker_in(&directory).expect("read the marker back").watcher_pid, 4242);
    let leftovers: Vec<String> = std::fs::read_dir(&directory)
        .expect("list the directory")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().into_owned())
        .filter(|name| return name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
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
