//! Controls for the judgment of the run:
//! that a run which keeps moving is left alone,
//! that one which stops moving is ended,
//! that
//!  a task waiting for the disk counts as work,
//! that a system which counts nothing is bounded by the wall clock,
//! that the ending
//!  names what it read,
//! that the look sees the processes the run started,
//! and that a stuck run ends its process.

use super::*;
use crate::test_support::{arm_watchdog, fixture, forbid_core_dumps, remove, spawn_owned, this_test_binary};
use std::os::unix::process::ExitStatusExt as _;

/// A look at a run in which nothing is able to run and nothing waits for the disk.
fn moved(bytes: u64, ticks: u64, members: u64) -> Look {
    return Look { bytes, ticks, working: false, members };
}

/// A run that keeps moving is left alone,
/// however many windows it passes.
#[test]
fn a_run_that_keeps_moving_is_left_alone() {
    let window: Duration = Duration::from_secs(2);
    let mut judge: Judge = Judge::new(window);
    let mut elapsed: Duration = Duration::ZERO;
    // Ten windows of a run that writes a record at every look.
    for step in 0..40_u64 {
        assert_eq!(judge.observe(elapsed, Some(moved(step * 4096, step, 3))), None, "step {step} moved");
        elapsed += SAMPLE_INTERVAL;
    }
    assert!(elapsed > window * 9, "{elapsed:?} passes nine windows");
}

/// A run that stops moving is ended at the first look the window covers,
/// and the ending counts the quiet stretch from the last look that moved.
#[test]
fn a_run_that_stops_moving_is_ended_after_the_window() {
    let window: Duration = Duration::from_secs(2);
    let mut judge: Judge = Judge::new(window);
    let mut elapsed: Duration = Duration::ZERO;
    // Three looks that move, at 0, half a second and one second.
    for step in 0..3_u64 {
        assert_eq!(judge.observe(elapsed, Some(moved(step * 4096, 1, 3))), None);
        elapsed += SAMPLE_INTERVAL;
    }
    let stopped: Look = moved(2 * 4096, 1, 3);
    let mut ended: Option<Ending> = None;
    for _ in 0..10 {
        ended = judge.observe(elapsed, Some(stopped));
        if ended.is_some() {
            break;
        }
        elapsed += SAMPLE_INTERVAL;
    }
    assert_eq!(ended, Some(Ending::Quiet(window)), "the first look the window covers ends the run, and it is the window old");
}

/// A run whose counters stop being readable is still ended:
/// a look that cannot be read is no sign of work,
/// so the clock keeps running through it,
/// and the ending says which reading was missing.
#[test]
fn a_run_whose_counters_stop_being_readable_is_ended() {
    let window: Duration = Duration::from_secs(2);
    let mut judge: Judge = Judge::new(window);
    // One look that reads the run and finds it moving,
    // as the first look of a test that then exhausts its descriptors does.
    assert_eq!(judge.observe(Duration::ZERO, Some(moved(4096, 1, 3))), None, "the run moved");
    let mut elapsed: Duration = Duration::ZERO;
    let mut ended: Option<Ending> = None;
    while ended.is_none() && elapsed < window * 4 {
        elapsed += SAMPLE_INTERVAL;
        ended = judge.observe(elapsed, None);
    }
    let reported: Ending = ended.expect("the run ends rather than wait for a reading that never comes");
    assert_eq!(reported, Ending::Unreadable(window), "the window of looks that read nothing ends the run");
    let said: String = reported.to_string();
    assert!(said.contains("counters"), "the ending names what could not be read: {said}");
}

/// A first look that cannot be read does not decide the run for ever:
/// the looks that read it judge it,
/// so a working run is not ended for one missing reading.
#[test]
fn a_first_look_that_cannot_be_read_leaves_the_later_ones_to_judge() {
    let window: Duration = Duration::from_secs(2);
    let mut judge: Judge = Judge::new(window);
    assert_eq!(judge.observe(Duration::ZERO, None), None, "the window has not passed");
    let mut elapsed: Duration = Duration::ZERO;
    // Descriptors are back, and the run is readable and moving window after window.
    for step in 0..20_u64 {
        assert_eq!(judge.observe(elapsed, Some(moved(step * 4096, step, 3))), None, "step {step} moved");
        elapsed += SAMPLE_INTERVAL;
    }
    assert!(elapsed > window * 4, "{elapsed:?} passes four windows");
}

/// A task waiting for the disk is at work,
/// so a run whose only sign is a task in `D` is left alone.
#[test]
fn a_run_waiting_for_the_disk_is_left_alone() {
    let window: Duration = Duration::from_secs(1);
    let mut judge: Judge = Judge::new(window);
    let waiting: Look = Look { bytes: 7, ticks: 7, working: true, members: 3 };
    let mut elapsed: Duration = Duration::ZERO;
    for _ in 0..8 {
        assert_eq!(judge.observe(elapsed, Some(waiting)), None, "a task in disk wait is at work");
        elapsed += SAMPLE_INTERVAL;
    }
}

/// A run whose processes and tasks come and go is left alone,
/// even when no counter moves.
#[test]
fn a_run_whose_tasks_come_and_go_is_left_alone() {
    let window: Duration = Duration::from_secs(1);
    let mut judge: Judge = Judge::new(window);
    let mut elapsed: Duration = Duration::ZERO;
    for step in 0..8_u64 {
        // The same figures, one task more or fewer at each look: a run that starts and ends threads.
        let members: u64 = 3 + step % 2;
        assert_eq!(judge.observe(elapsed, Some(moved(0, 0, members))), None, "step {step} changed its tasks");
        elapsed += SAMPLE_INTERVAL;
    }
}

/// Where this system counts no task's work the window is a fixed deadline,
/// and the ending says so rather than claiming the run showed nothing.
#[test]
fn a_system_that_counts_nothing_is_bounded_by_the_wall_clock() {
    let window: Duration = Duration::from_secs(2);
    let mut judge: Judge = Judge::new(window);
    assert_eq!(judge.observe(Duration::from_secs(1), None), None, "the window has not passed");
    let ended: Option<Ending> = judge.observe(Duration::from_millis(2001), None);
    assert_eq!(ended, Some(Ending::Unmeasurable(Duration::from_millis(2001))));
}

/// The first look is a change,
/// so a run whose first look comes late is not ended at once.
#[test]
fn the_first_look_is_work() {
    let mut judge: Judge = Judge::new(Duration::from_secs(1));
    assert_eq!(judge.observe(Duration::from_secs(10), Some(moved(0, 0, 1))), None);
}

/// The ending names the figure that decided it and the signs that were read,
/// so a person can tell a stuck test from a slow one.
#[test]
fn the_ending_names_the_quiet_stretch_and_the_signs_it_read() {
    let quiet: String = Ending::Quiet(Duration::from_millis(120_500)).to_string();
    assert!(quiet.contains("120.5 s"), "{quiet}");
    assert!(quiet.contains("no task able to run or waiting for the disk") && quiet.contains("no bytes moved"), "{quiet}");
    let unmeasured: String = Ending::Unmeasurable(Duration::from_secs(120)).to_string();
    assert!(unmeasured.contains("120.0 s") && unmeasured.contains("counts no work"), "{unmeasured}");
    let unread: String = Ending::Unreadable(Duration::from_millis(3_500)).to_string();
    assert!(unread.contains("3.5 s") && unread.contains("counters") && unread.contains("out of file descriptors"), "{unread}");
}

/// The look counts this process and the processes it started,
/// and this system can be read.
#[test]
fn the_look_sees_this_process_and_the_processes_it_started() {
    let Some(reader): Option<i32> = current_thread_id() else {
        eprintln!("skipped: this system numbers no thread, so nothing is measured");
        return;
    };
    let Some(before): Option<Look> = look(reader) else {
        eprintln!("skipped: this system counts no task's work, so nothing is measured");
        return;
    };
    let mut child: std::process::Child = spawn_owned(std::process::Command::new("sleep").arg("60")).expect("sleep starts");
    let after: Look = look(reader).expect("still readable");
    child.kill().expect("the child is killed");
    child.wait().expect("the child is reaped");
    assert!(after.members > before.members, "the child's task is counted too: {} members became {}", before.members, after.members);
}

/// What:
///  The path this file's child test `name` has when the test binary is run directly.
/// Why:
///   A test binary names its tests from the module path without the crate name,
/// and a name that matches no test runs
///  nothing and succeeds,
/// which the controls below rule out by reading what the child said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = `${module_path.split('::').slice(1).join('::')}::${name}`;
/// ```
fn child_path(name: &str) -> String {
    let module: &str = module_path!();
    let without_crate: &str = module.split_once("::").map_or(module, |(_, rest)| { return rest; });
    return format!("{without_crate}::{name}");
}

/// What:
///  Run the child test `name` in a process of its own and give back what it said and how it ended.
/// Why:
///   A watchdog ends its whole process,
/// so what it does can only be seen from outside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = spawnSync(process.execPath, ['--exact', path, '--ignored', '--nocapture', '--test-threads=1']);
/// ```
fn run_child(name: &str) -> std::process::Output {
    return std::process::Command::new(this_test_binary().expect("the test binary"))
        .args(["--exact", child_path(name).as_str(), "--ignored", "--nocapture", "--test-threads=1"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the child test runs");
}

/// How long a child a watcher must end is given before the control counts it as a run that never ended.
/// The child starts this test binary (measured at up to 11 s on this host under load),
/// waits its one-second window
/// and aborts,
/// so this is four times the slowest start measured (`doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`).
const CHILD_DEADLINE: Duration = Duration::from_secs(45);

/// What:
///  Run the child test `name` with its standard error on `stderr`,
/// and give back what it said and how it ended,
/// or nothing where it outlived
///  `CHILD_DEADLINE`,
/// in which case it is killed first.
/// Why:
///   A watcher that cannot end its process leaves a blocked child alive for ever,
/// so a control that waits for such a child without a
///  deadline would hang the run it is checking;
/// the deadline turns that into a failed control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = spawn(exe, args, { stdio: ['ignore', 'pipe', stderr] });
/// const status = await withDeadline(child, CHILD_DEADLINE); // kill and return undefined when it passes
/// return status === undefined ? undefined : { status, stdout: await read(child.stdout) };
/// ```
fn run_child_with_stderr_until(name: &str, stderr: std::process::Stdio) -> Option<std::process::Output> {
    let mut child: std::process::Child = std::process::Command::new(this_test_binary().expect("the test binary"))
        .args(["--exact", child_path(name).as_str(), "--ignored", "--nocapture", "--test-threads=1"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(stderr)
        .spawn()
        .expect("the child test starts");
    let started: Instant = Instant::now();
    loop {
        match child.try_wait().expect("the child is waited for") {
            Some(_ended) => return Some(child.wait_with_output().expect("the child's output")),
            None if started.elapsed() >= CHILD_DEADLINE => {
                child.kill().expect("the child that outlived the deadline is killed");
                let _reaped = child.wait();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

/// Child of the control below:
/// a run whose only task waits for a message nobody sends.
#[test]
#[ignore = "started by a_stuck_run_ends_its_process in a process of its own"]
fn child_that_stops_moving() {
    forbid_core_dumps();
    let _armed: Option<std::thread::JoinHandle<()>> = watch(Duration::from_secs(1));
    let (_sender, receiver) = std::sync::mpsc::channel::<()>();
    receiver.recv().expect("unreachable: nothing is ever sent");
}

/// Child of the control below:
/// a run that keeps writing for three windows and then ends on its own.
#[test]
#[ignore = "started by a_run_that_keeps_moving_past_its_window_is_not_ended in a process of its own"]
fn child_that_keeps_moving() {
    forbid_core_dumps();
    let root: std::path::PathBuf = fixture("progress-child");
    let _armed: Option<std::thread::JoinHandle<()>> = watch(Duration::from_secs(1));
    for _ in 0..30_u32 {
        std::fs::write(root.join("record"), vec![b'x'; 4096]).expect("write");
        std::thread::sleep(Duration::from_millis(100));
    }
    remove(root.as_path());
}

/// A run that stops moving ends its process,
/// and the line it leaves names the window and the signs.
#[test]
fn a_stuck_run_ends_its_process() {
    let output: std::process::Output = run_child("child_that_stops_moving");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    let ran: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    assert!(ran.contains(child_path("child_that_stops_moving").as_str()), "the child ran the test named, not nothing:\n{ran}");
    assert_eq!(output.status.signal(), Some(libc::SIGABRT), "the watchdog ended the process:\n{said}");
    assert!(said.contains("a test is stuck"), "{said}");
    assert!(said.contains("showed no sign of work for"), "{said}");
}

/// A run that stops moving ends its process even where the report cannot be written,
/// so a full disk cannot leave a blocked test alive without its watchdog.
#[test]
fn a_stuck_run_ends_its_process_though_its_report_cannot_be_written() {
    let file: std::fs::File = match std::fs::OpenOptions::new().write(true).open("/dev/full") {
        Ok(opened) => opened,
        Err(error) => {
            eprintln!("skipped: this system has no /dev/full to refuse a report ({error})");
            return;
        }
    };
    let ended: Option<std::process::Output> = run_child_with_stderr_until("child_that_stops_moving", std::process::Stdio::from(file));
    let output: std::process::Output = ended.expect("the watchdog ended the process though its report could not be written");
    let ran: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    assert!(ran.contains(child_path("child_that_stops_moving").as_str()), "the child ran the test named, not nothing:\n{ran}");
    assert_eq!(output.status.signal(), Some(libc::SIGABRT), "the watchdog ended the process, though its report could not be written");
}

/// How many watchers `child_that_sleeps_under_two_watchers` arms:
/// the fixture's one and the one it starts itself.
/// The child prints the count it saw and the parent reads it,
/// because the child's own watcher ends it either way,
/// so a control that only looked at how the child ended would pass with the fixture's watcher disconnected.
const WATCHERS_UNDER_TEST: usize = 2;

/// What:
///  How many threads are registered as readers of this run,
/// waiting up to `within` for `expected` of them to start.
/// Why:
///   A watcher registers itself at the start of its own thread,
/// so the count is the control's to read only once the
///  threads have run;
/// a fixed sleep would make the control's meaning depend on the host's load.
/// Every look at the count writes a newline
///  (a line,
/// because this process's standard output is held until one arrives),
/// and a write is work,
/// so the watcher that must end this child does not judge it quiet while it is still finding out,
/// and a child
///  that saw fewer readers than it expected can still say so.
/// The count is read again every 200 ms,
/// which is well inside the watcher's own 500 ms look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadline = now() + within; let running = readingThreads().length;
/// while (running !== expected && now() < deadline) { write('\n'); await sleep(200); running = readingThreads().length }
/// return running;
/// ```
fn readers_running(expected: usize, within: Duration) -> usize {
    let started: Instant = Instant::now();
    let mut running: usize = reading_threads().len();
    while running != expected && started.elapsed() < within {
        let _written = std::io::stdout().write_all(b"\n");
        std::thread::sleep(Duration::from_millis(200));
        running = reading_threads().len();
    }
    return running;
}

/// Child of the control below:
/// a run under the fixture's watcher and one of its own that shows nothing at all,
/// so neither watcher may count the other's readings as the run's work.
#[test]
#[ignore = "started by two_watchers_do_not_keep_each_other_alive in a process of its own"]
fn child_that_sleeps_under_two_watchers() {
    forbid_core_dumps();
    arm_watchdog();
    let _armed: Option<std::thread::JoinHandle<()>> = watch(Duration::from_secs(1));
    // The parent reads this count: one watcher ends this child either way, so only the count tells two apart from one.
    println!("watchers: {}", readers_running(WATCHERS_UNDER_TEST, Duration::from_secs(5)));
    std::thread::sleep(Duration::from_secs(5));
}

/// Two watchers of a run that shows nothing end it,
/// so a reading is never taken for the work of the run it reads.
#[test]
fn two_watchers_do_not_keep_each_other_alive() {
    let output: std::process::Output = run_child("child_that_sleeps_under_two_watchers");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    let ran: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    assert!(ran.contains(child_path("child_that_sleeps_under_two_watchers").as_str()), "the child ran the test named, not nothing:\n{ran}");
    // One watcher ends this child either way, so the count is what tells the two watchers apart from one.
    assert!(ran.contains(format!("watchers: {WATCHERS_UNDER_TEST}").as_str()), "both watchers ran, so neither could count the other's readings as the run's work:\n{ran}");
    assert_eq!(output.status.signal(), Some(libc::SIGABRT), "a watcher ended the process:\n{said}");
    assert!(said.contains("a test is stuck"), "{said}");
}

/// A run that keeps moving past its window is not ended,
/// which is the failure this watchdog had with a wall clock.
#[test]
fn a_run_that_keeps_moving_past_its_window_is_not_ended() {
    let output: std::process::Output = run_child("child_that_keeps_moving");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    let ran: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    assert!(output.status.success(), "the moving child ended on its own:\n{ran}\n{said}");
    assert!(ran.contains("1 passed"), "the child ran the test named, not nothing:\n{ran}");
    assert!(!said.contains("a test is stuck"), "{said}");
}

/// Child of the control below:
/// a run whose only thread waits for an event nothing delivers,
/// with a window far past the watcher's,
///  so only the watcher can end it.
#[test]
#[ignore = "started by a_wait_does_not_keep_the_watchdog_alive in a process of its own"]
fn child_that_waits_under_the_watchdog() {
    forbid_core_dumps();
    let _armed: Option<std::thread::JoinHandle<()>> = watch(Duration::from_secs(1));
    // The wait reads the run itself at every look, and those reads are not the run's work.
    let _held: bool = crate::test_support::wait_until_for(crate::test_support::WaitingFor { window: Duration::from_secs(600), done: &|| { return false; } });
}

/// A thread inside a control's wait does not keep the watchdog alive:
/// the watchdog ends the process although the waiting thread reads the run.
#[test]
fn a_wait_does_not_keep_the_watchdog_alive() {
    let ended: Option<std::process::Output> = run_child_with_stderr_until("child_that_waits_under_the_watchdog", std::process::Stdio::piped());
    let output: std::process::Output = ended.expect("the watchdog ended the process although a thread of it was waiting for an event nothing delivers");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    let ran: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    assert!(ran.contains(child_path("child_that_waits_under_the_watchdog").as_str()), "the child ran the test named, not nothing:\n{ran}");
    assert_eq!(output.status.signal(), Some(libc::SIGABRT), "the watchdog ended the process:\n{said}");
    assert!(said.contains("a test is stuck"), "{said}");
}
