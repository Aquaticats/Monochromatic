//! Verifies the death watch of the application watcher:
//! the verdict from the state,
//! the wait on a real throwaway process through its process file descriptor and through the poll fallback,
//! and the report of an unasked end through the failure reporter.

/// Persisted watcher identity.
use crate::application_watch_state::WatchProcess;
/// Failure delivery.
use crate::runtime_report::{ReportChannel, RuntimeReporter};
/// Process facts.
use crate::process_info::read_start_time;
/// Death watch under test.
use crate::watcher_sentinel::{death_report, judge, poll_for_exit, run_sentinel, wait_for_exit, StateSource, Verdict, WATCHER_DIED_KIND};
/// Shared recording between the test and the channel.
use std::sync::{Arc, Mutex};
/// Bounded waits.
use std::time::{Duration, Instant};

/// Records what it was asked to deliver.
struct Recording {
    /// Kinds and lines delivered in order.
    lines: Arc<Mutex<Vec<(String, String)>>>,
}

/// The recording channel.
impl ReportChannel for Recording {
    /// The label.
    fn name(&self) -> &'static str {
        return "recording";
    }

    /// Records the delivery.
    fn deliver(&self, kind: &str, line: &str) -> Result<(), String> {
        self.lines.lock().expect("the recording is not poisoned").push((kind.to_string(), line.to_string()));
        return Ok(());
    }
}

/// A state source that answers the same way every time.
enum Fixed {
    /// The state names this watcher.
    Names(WatchProcess),
    /// There is no state.
    Absent,
    /// The state cannot be read.
    Broken,
}

/// The fixed state source.
impl StateSource for Fixed {
    /// Answers as configured.
    fn read(&self) -> std::io::Result<Option<WatchProcess>> {
        return match self {
            Fixed::Names(process) => Ok(Some(*process)),
            Fixed::Absent => Ok(None),
            Fixed::Broken => Err(std::io::Error::other("the state is garbled")),
        };
    }
}

/// A throwaway child that sleeps,
/// with its pid and start time.
fn throwaway() -> (std::process::Child, i32, u64) {
    let child = std::process::Command::new("/usr/bin/sleep").arg("60").spawn().expect("start the throwaway process");
    let pid = i32::try_from(child.id()).expect("pid fits");
    let start_time = read_start_time(std::path::Path::new("/proc"), pid).expect("the child has a start time");
    return (child, pid, start_time);
}

/// Kills the child from another thread after a short delay and reaps it.
fn kill_later(mut child: std::process::Child) -> std::thread::JoinHandle<()> {
    return std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        let _killed = child.kill();
        let _reaped = child.wait();
    });
}

/// The state naming the dead watcher means the death was not asked for;
/// no state,
/// or a state naming another watcher,
/// means it was;
/// an unreadable state is said to be unknown.
#[test]
fn the_state_decides_whether_the_end_was_asked_for() {
    let named = Fixed::Names(WatchProcess { pid: 10, start_time: 99, mark: 1, uid: 1000 });
    assert_eq!(judge(&named, 10, 99), Verdict::Unexpected);
    assert_eq!(judge(&Fixed::Absent, 10, 99), Verdict::Expected);
    assert_eq!(judge(&named, 11, 99), Verdict::Expected, "another pid");
    assert_eq!(judge(&named, 10, 100), Verdict::Expected, "another start time: a newer start replaced it");
    assert_eq!(judge(&Fixed::Broken, 10, 99), Verdict::Unknown("the state is garbled".to_string()));
}

/// A wait on a real process returns soon after it is killed,
/// through its process file descriptor.
#[test]
fn a_wait_on_a_throwaway_process_ends_when_it_is_killed() {
    let (child, pid, start_time) = throwaway();
    let killer = kill_later(child);
    let started = Instant::now();
    wait_for_exit(std::path::Path::new("/proc"), pid, start_time).expect("the wait succeeds");
    killer.join().expect("the killer finished");
    assert!(started.elapsed() < Duration::from_secs(10), "the wait took {:?}", started.elapsed());
}

/// A wait on a process that was replaced by another with the same pid ends at once.
#[test]
fn a_wait_on_a_replaced_pid_ends_at_once() {
    let (mut child, pid, start_time) = throwaway();
    let started = Instant::now();
    wait_for_exit(std::path::Path::new("/proc"), pid, start_time + 1).expect("the wait succeeds");
    assert!(started.elapsed() < Duration::from_secs(5));
    let _killed = child.kill();
    let _reaped = child.wait();
}

/// Where the kernel offers no process file descriptor the start time is polled,
/// which a fake procfs shows:
/// the entry disappears and the wait ends.
#[test]
fn the_poll_fallback_ends_when_the_process_entry_goes_away() {
    let root = crate::test_scratch::scratch_root().join(format!("wg-quicker-sentinel-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&root);
    let entry = root.join("4242");
    std::fs::create_dir_all(&entry).expect("make the fake process");
    let zeros = vec!["0"; 17].join(" ");
    std::fs::write(entry.join("stat"), format!("4242 (x) S 1 {zeros} 555 0\n")).expect("write stat");
    let removing = entry.clone();
    let remover = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        std::fs::remove_dir_all(removing).expect("the process exits");
    });
    let started = Instant::now();
    poll_for_exit(&root, 4242, 555, Duration::from_millis(20)).expect("the poll succeeds");
    remover.join().expect("the remover finished");
    assert!(started.elapsed() < Duration::from_secs(5));
}

/// A watcher killed without a `watch-stop` is reported:
/// the report names both policies and the pid,
/// is of the kind the reporter limits on,
/// and the channels are drained before the sentinel returns.
#[test]
fn an_unasked_death_is_reported() {
    let (child, pid, start_time) = throwaway();
    let delivered = Arc::new(Mutex::new(Vec::new()));
    let mut reporter = RuntimeReporter::with_channels(vec![(Box::new(Recording { lines: Arc::clone(&delivered) }), Duration::from_secs(1))]);
    let state = Fixed::Names(WatchProcess { pid, start_time, mark: 1, uid: 1000 });
    let killer = kill_later(child);
    let verdict = run_sentinel(std::path::Path::new("/proc"), pid, start_time, Duration::from_millis(50), &state, &mut reporter).expect("the sentinel runs");
    killer.join().expect("the killer finished");
    assert_eq!(verdict, Verdict::Unexpected);
    let lines = delivered.lock().expect("the recording is not poisoned");
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].0, WATCHER_DIED_KIND);
    assert_eq!(lines[0].1, death_report(pid));
    assert!(lines[0].1.contains("POLICY NOT IN FORCE"));
    assert!(lines[0].1.contains("socket marking of application and update cgroups"));
    assert!(lines[0].1.contains("process watch"));
    assert!(lines[0].1.contains(&format!("pid {pid}")));
}

/// A watcher stopped by `watch-stop`,
/// whose state is gone,
/// is not reported.
#[test]
fn a_stop_that_removed_the_state_is_not_reported() {
    let (child, pid, start_time) = throwaway();
    let delivered = Arc::new(Mutex::new(Vec::new()));
    let mut reporter = RuntimeReporter::with_channels(vec![(Box::new(Recording { lines: Arc::clone(&delivered) }), Duration::from_secs(1))]);
    let state = Fixed::Absent;
    let killer = kill_later(child);
    let verdict = run_sentinel(std::path::Path::new("/proc"), pid, start_time, Duration::from_millis(50), &state, &mut reporter).expect("the sentinel runs");
    killer.join().expect("the killer finished");
    assert_eq!(verdict, Verdict::Expected);
    assert!(delivered.lock().expect("the recording is not poisoned").is_empty());
}
