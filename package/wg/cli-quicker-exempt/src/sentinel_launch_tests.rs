//! Verifies that the sentinel starts outside the watcher's cgroup,
//! falls back loudly,
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
    /// The marker successive reads answer,
    /// the last one repeating.
    markers: Vec<Option<SentinelMarker>>,
    /// Reads so far.
    reads: Cell<usize>,
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

    /// Records the program and arguments and fails when told to.
    fn start_detached(&self, executable: &Path, arguments: &[String]) -> Result<(), String> {
        self.events.borrow_mut().push(format!("detached {} {}", executable.display(), arguments.join(" ")));
        if self.detached_fails {
            return Err("spawn failed".to_string());
        }
        return Ok(());
    }

    /// Answers the next scripted marker.
    fn read_marker(&self, _key: &str) -> Option<SentinelMarker> {
        let index = self.reads.get();
        self.reads.set(index + 1);
        let answer = self.markers.get(index.min(self.markers.len().saturating_sub(1)))?;
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
fn steps(service_fails: bool, detached_fails: bool, markers: Vec<Option<SentinelMarker>>, alive: bool) -> FakeSteps {
    return FakeSteps { events: RefCell::new(Vec::new()), service_fails, detached_fails, markers, reads: Cell::new(0), alive };
}

/// A marker that names the watcher under test.
fn marker() -> Option<SentinelMarker> {
    return Some(SentinelMarker { watcher_pid: 4242, pid: 777, start_time: 5 });
}

/// The request under test.
fn request(executable: &Path) -> SentinelRequest<'_> {
    return SentinelRequest { key: "wgtest", watcher_pid: 4242, watcher_start_time: 9001, uid: 1000, gid: 1001, executable };
}

/// The service starts outside the watcher's cgroup,
/// collected,
/// with `systemd-oomd` asked to omit it,
/// and runs this program with the hidden command and the watcher's identity after a separator.
#[test]
fn the_service_arguments_name_the_unit_the_oom_preference_and_the_command() {
    let executable = Path::new("/usr/bin/wg-quicker-exempt");
    let arguments = systemd_run_arguments(&request(executable));
    assert!(arguments.contains(&"--collect".to_string()), "{arguments:?}");
    assert!(arguments.contains(&"--no-ask-password".to_string()), "{arguments:?}");
    assert!(arguments.contains(&"--unit=wg-quicker-exempt-sentinel-4242-9001".to_string()), "{arguments:?}");
    assert!(arguments.contains(&"--property=ManagedOOMPreference=omit".to_string()), "{arguments:?}");
    let separator = arguments.iter().position(|argument| return argument == "--").expect("a separator");
    assert!(arguments[..separator].iter().all(|argument| return argument.starts_with("--")), "{arguments:?}");
    assert_eq!(
        arguments[separator + 1..],
        ["/usr/bin/wg-quicker-exempt", "__sentinel", "wgtest", "4242", "9001", "1000", "1001"]
    );
    assert_eq!(unit_name(4242, 9001), "wg-quicker-exempt-sentinel-4242-9001");
    assert_eq!(sentinel_arguments(&request(executable))[0], "__sentinel");
}

/// A confirmed service start removes the old marker first and reports nothing.
#[test]
fn a_service_that_confirms_is_started_after_the_old_marker_is_removed_and_reports_nothing() {
    let executable = Path::new("/usr/bin/wg-quicker-exempt");
    let fake = steps(false, false, vec![marker()], true);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, Some(Placement::Service));
    assert!(outcome.confirmed);
    assert!(outcome.reports.is_empty(), "{:?}", outcome.reports);
    let events = fake.events.borrow();
    assert_eq!(events[0], "remove marker");
    assert!(events[1].starts_with("service --quiet --collect"), "{events:?}");
    assert!(!events.iter().any(|event| return event.starts_with("detached")), "{events:?}");
}

/// A refused service falls back to a detached sentinel and says what that costs.
#[test]
fn a_refused_service_falls_back_beside_the_watcher_and_says_both_die_together() {
    let executable = Path::new("/usr/bin/wg-quicker-exempt");
    let fake = steps(true, false, vec![marker()], true);
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

/// Neither way starting is said twice and no marker is waited for.
#[test]
fn when_neither_way_starts_a_sentinel_it_says_so_and_does_not_wait() {
    let executable = Path::new("/usr/bin/wg-quicker-exempt");
    let fake = steps(true, true, vec![None], true);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, None);
    assert!(!outcome.confirmed);
    assert_eq!(outcome.reports.len(), 2);
    assert!(outcome.reports[1].contains("could not start at all (spawn failed)"), "{:?}", outcome.reports);
    assert_eq!(fake.reads.get(), 0);
}

/// A sentinel that was started but never left a marker is reported as unconfirmed,
/// with where to look.
#[test]
fn a_started_sentinel_that_never_confirms_is_reported_with_where_to_look() {
    let executable = Path::new("/usr/bin/wg-quicker-exempt");
    let fake = steps(false, false, vec![None], true);
    let outcome = launch_sentinel(&fake, &request(executable));
    assert_eq!(outcome.placement, Some(Placement::Service));
    assert!(!outcome.confirmed);
    assert_eq!(outcome.reports.len(), 1);
    assert!(outcome.reports[0].contains("did not confirm that it runs"), "{:?}", outcome.reports);
    assert!(outcome.reports[0].contains("systemctl status wg-quicker-exempt-sentinel-4242-9001"), "{:?}", outcome.reports);
}

/// A marker confirms only when it names this watcher and its process runs.
#[test]
fn a_marker_of_another_watcher_or_a_dead_process_does_not_confirm() {
    let other = Some(SentinelMarker { watcher_pid: 1, pid: 777, start_time: 5 });
    assert!(!confirm_running(&steps(false, false, vec![other], true), "wgtest", 4242));
    assert!(!confirm_running(&steps(false, false, vec![marker()], false), "wgtest", 4242));
    assert!(!confirm_running(&steps(false, false, vec![None], true), "wgtest", 4242));
    let late = steps(false, false, vec![None, None, marker()], true);
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

/// The summary says where the sentinel runs and whether it confirmed.
#[test]
fn the_summary_says_where_the_sentinel_runs_and_whether_it_confirmed() {
    let executable = Path::new("/usr/bin/wg-quicker-exempt");
    let confirmed = launch_sentinel(&steps(false, false, vec![marker()], true), &request(executable)).summary();
    assert!(confirmed.contains("as its own service, outside the cgroup of the watcher"), "{confirmed}");
    assert!(confirmed.contains("and confirmed that it runs"), "{confirmed}");
    let beside = launch_sentinel(&steps(true, false, vec![None], true), &request(executable)).summary();
    assert!(beside.contains("beside the watcher, in its cgroup"), "{beside}");
    assert!(beside.contains("has not confirmed"), "{beside}");
    let nowhere = launch_sentinel(&steps(true, true, vec![None], true), &request(executable)).summary();
    assert!(nowhere.contains("started nowhere"), "{nowhere}");
}
