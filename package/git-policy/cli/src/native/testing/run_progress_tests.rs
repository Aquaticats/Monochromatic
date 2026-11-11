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

/// What:
///  A look at a run in which nothing is able to run and nothing waits for the disk,
///  made of `members` tasks that had each moved `bytes` and used `ticks`.
/// Why:
///   A look's totals are the sum of the tasks it found,
/// and the judge reads the tasks,
/// so a look whose tasks each moved a figure is a look whose totals moved it as many times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = (bytes, ticks, members) => ({ bytes: bytes * members, ticks: ticks * members, working: false, members, tasks: [...members].map((task) => ({ task, bytes, ticks })) });
/// ```
fn moved(bytes: u64, ticks: u64, members: u64) -> Look {
    let tasks: Vec<TaskFigures> = (0..members)
        .map(|task: u64| { return TaskFigures { task, bytes, ticks }; })
        .collect();
    return Look { bytes: bytes.saturating_mul(members), ticks: ticks.saturating_mul(members), working: false, members, tasks };
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
        ended = judge.observe(elapsed, Some(stopped.clone()));
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
    let mut waiting: Look = moved(7, 7, 3);
    waiting.working = true;
    let mut elapsed: Duration = Duration::ZERO;
    for _ in 0..8 {
        assert_eq!(judge.observe(elapsed, Some(waiting.clone())), None, "a task in disk wait is at work");
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

/// A run whose every figure falls is ended:
/// a task that ended,
/// or one the look could not read,
/// leaves fewer tasks and less moved behind it,
/// and neither is the run working.
/// A count that fell at every look kept a stuck run alive before this.
///
/// The tasks the look found are what says so:
/// every one of them had moved less than the look before,
/// and the tasks that are gone are not the run working.
#[test]
fn a_run_whose_figures_only_fall_is_ended() {
    let window: Duration = Duration::from_secs(2);
    let mut judge: Judge = Judge::new(window);
    let mut elapsed: Duration = Duration::ZERO;
    let mut ended: Option<Ending> = None;
    // Ten looks, each with one task fewer and less moved than the look before it.
    for step in 0..10_u64 {
        let fewer: Look = moved(4_000 - step * 100, 100 - step * 10, 10 - step);
        ended = judge.observe(elapsed, Some(fewer));
        if ended.is_some() {
            break;
        }
        elapsed += SAMPLE_INTERVAL;
    }
    assert_eq!(ended, Some(Ending::Quiet(window)), "a run whose figures only fall is ended at the first look the window covers");
}

/// What:
///  A run whose totals fall because a task ended is judged by the tasks that are still there,
///  so a task that keeps moving keeps the run alive.
/// Why:
///   A look's totals are sums over the tasks it found,
/// and a task that ends takes every byte and tick it moved with it,
/// so a worker that moves less than the task the look before found leaves every total smaller than the one before it.
/// A judge that read the totals alone ended such a worker once its window passed,
/// though it never stopped working:
/// measured on 2026-11-11 through the real look,
/// three looks over a one-second window fell from 3,093,960 to 2,374,539 and 1,655,121 bytes
/// while a task of the run moved 65,536,
/// 65,630 and
///  65,632
/// bytes of its own and no task was able to run or waiting for the disk,
/// and the judge answered `Quiet(1 s)`.
#[test]
fn a_task_that_keeps_moving_keeps_the_run_alive_though_the_totals_fall() {
    let window: Duration = Duration::from_secs(2);
    let mut judge: Judge = Judge::new(window);
    let mut elapsed: Duration = Duration::ZERO;
    let mut last_total: u64 = u64::MAX;
    // One look per step: the worker (task 1) moved more than the look before, and the task that took the previous
    //  look's place had moved less than the one it replaced.
    for step in 1..=10_u64 {
        let worker: TaskFigures = TaskFigures { task: 1, bytes: step * WORKER_MOVES, ticks: step };
        let replaced: TaskFigures = TaskFigures {
            task: 100 + step,
            bytes: THE_TASK_BEFORE_HAD_MOVED.saturating_sub(step * ITS_SUCCESSOR_MOVED_THIS_MUCH_LESS),
            ticks: ITS_SUCCESSOR_USED_THIS_MUCH_LESS.saturating_sub(step * ITS_SUCCESSOR_USED_LESS_EACH_LOOK),
        };
        let look: Look = Look {
            bytes: worker.bytes + replaced.bytes,
            ticks: worker.ticks + replaced.ticks,
            working: false,
            members: 2,
            tasks: vec![worker, replaced],
        };
        assert!(look.bytes < last_total, "step {step}: the totals fall, {} is not below {last_total}", look.bytes);
        last_total = look.bytes;
        assert_eq!(judge.observe(elapsed, Some(look)), None, "step {step}: the worker moved");
        elapsed += SAMPLE_INTERVAL;
    }
    assert!(elapsed > window, "{elapsed:?} passes the window while the worker moved at every look");
}

/// How much the worker of the control above moves at every look.
const WORKER_MOVES: u64 = 100;

/// What the task of the first look above had moved,
/// before each successor of it moved less.
const THE_TASK_BEFORE_HAD_MOVED: u64 = 10_000;

/// How much less than its predecessor each task of the control above moved.
const ITS_SUCCESSOR_MOVED_THIS_MUCH_LESS: u64 = 1_000;

/// How many processor ticks the task of the first look above had used.
const ITS_SUCCESSOR_USED_THIS_MUCH_LESS: u64 = 100;

/// How many fewer ticks each task of the control above used than the one it replaced.
const ITS_SUCCESSOR_USED_LESS_EACH_LOOK: u64 = 10;

/// A thread inside two waits at once is one reader of the run:
/// its readings are taken off once,
/// not left over as the run's work at every look.
/// A second entry for one thread counted its whole counter as work before this,
/// and the run was then never quiet.
#[test]
fn a_thread_inside_two_waits_is_left_out_of_the_run_once() {
    if current_thread_id().is_none() {
        eprintln!("skipped: this system numbers no thread, so nothing is measured");
        return;
    }
    // Two waits of one thread, as a wait whose event another wait reports.
    let _outer: Option<Reading> = Reading::begin();
    let _inner: Option<Reading> = Reading::begin();
    // A fresh registration has read nothing yet, so the first look counts the thread's whole counter; the two looks
    //  compared here come after it.
    let _primed: Look = look_now().expect("this system counts its tasks' work");
    let before: Look = look_now().expect("still readable");
    let after: Look = look_now().expect("still readable");
    assert_eq!(after.bytes, before.bytes, "a thread's own readings are not the run's work, however many waits it is inside");
}

/// What:
///  A look this thread takes without keeping a registration of its own does not count its own readings
///  as the run's work at the look after it,
///  so a run that only reads it is ended.
/// Why:
///   A control's wait may look at the run without an outer `Reading`,
/// which the comment of `look_now` says is the case it covers;
/// a registration that ends with its look would forget the bytes that look read,
///  so every look would count them as the run's work and a stopped run would never be judged quiet.
#[test]
fn a_look_without_a_registration_does_not_count_its_own_readings_as_work() {
    if current_thread_id().is_none() {
        eprintln!("skipped: this system numbers no thread, so nothing is measured");
        return;
    }
    // The first look counts this thread's whole counter, because no look of this thread came before it.
    let _primed: Look = look_now().expect("this system counts its tasks' work");
    let before: Look = look_now().expect("still readable");
    let after: Look = look_now().expect("still readable");
    assert_eq!(
        after.bytes, before.bytes,
        "the readings the look before took are the run's work, so an idle thread looks busy at every look"
    );
    // The judge on the same evidence: an idle run that is only read is quiet after its window.
    let window: Duration = Duration::from_secs(1);
    let mut judge: Judge = Judge::new(window);
    assert_eq!(judge.observe(Duration::ZERO, look_now()), None, "the first look moved");
    assert_eq!(judge.observe(Duration::from_millis(500), look_now()), None, "the window has not passed");
    assert_eq!(
        judge.observe(window, look_now()),
        Some(Ending::Quiet(window)),
        "a run that only reads it showed no sign of work for the window"
    );
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

/// How long a child a watcher must end is given before the control counts it as a run that never ended.
/// The child starts this test binary (measured at up to 11 s on this host under load),
/// waits its one-second window
/// and aborts,
/// so this is four times the slowest start measured (`doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`).
/// Every control waits on its child through this deadline,
/// so a watcher that does not end its child fails the
///  control,
/// where a wait without a bound would hold the whole run.
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
/// return status === undefined ? undefined : { status, stdout: drain(child.stdout) };
/// ```
fn run_child_with_stderr_until(name: &str, stderr: std::process::Stdio) -> Option<std::process::Output> {
    return run_child_with_stderr_within(name, stderr, CHILD_DEADLINE);
}

/// What:
///  `run_child_with_stderr_until` with the deadline the caller states.
/// Why:
///   The control of the output collection needs a deadline it can reach in a moment,
/// and every other control wants the slowest
///  child start measured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await withDeadline(child, deadline); // kill and return undefined when it passes
/// ```
fn run_child_with_stderr_within(name: &str, stderr: std::process::Stdio, deadline: Duration) -> Option<std::process::Output> {
    let mut child: std::process::Child = std::process::Command::new(this_test_binary().expect("the test binary"))
        .args(["--exact", child_path(name).as_str(), "--ignored", "--nocapture", "--test-threads=1"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(stderr)
        .spawn()
        .expect("the child test starts");
    let started: Instant = Instant::now();
    let status: std::process::ExitStatus = loop {
        match child.try_wait().expect("the child is waited for") {
            Some(ended) => break ended,
            None if started.elapsed() >= deadline => {
                child.kill().expect("the child that outlived the deadline is killed");
                let _reaped = child.wait();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    };
    let collected_out: Vec<u8> = read_a_pipe_until_it_ends(child.stdout.take());
    let collected_err: Vec<u8> = read_a_pipe_until_it_ends(child.stderr.take());
    return Some(std::process::Output { status, stdout: collected_out, stderr: collected_err });
}

/// What:
///  Every byte a pipe still holds once the child that wrote it has ended,
/// read without waiting for the pipe to end.
/// Why:
///   A process the child started inherits the child's standard output and error,
/// and the pipe ends only when every process holding it lets go,
///  which one that never ends never does;
/// so a read that waits for the end of the pipe waits for a process the control never started to watch,
///  where everything the child wrote is already in the pipe when the child ends.
/// The read is non-blocking for that reason,
/// and it stops at the first moment the pipe has nothing left to give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function drain(pipe) { const bytes = []; for (;;) { const chunk = read(pipe, { nonBlocking: true }); if (chunk.length === 0) return bytes; bytes.push(...chunk) } }
/// ```
fn read_a_pipe_until_it_ends(pipe: Option<impl std::io::Read + std::os::fd::AsRawFd>) -> Vec<u8> {
    let Some(mut unending) = pipe else { return Vec::new() };
    let descriptor: std::os::fd::RawFd = unending.as_raw_fd();
    // SAFETY: the descriptor belongs to `pipe`, which outlives both calls, and neither takes a pointer.
    let flags: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags < 0 {
        eprintln!("skipped: this system would not say how the child's output pipe is read (errno {flags})");
        return Vec::new();
    }
    // The flag is set on this copy of the descriptor only, and its read end is closed with the child that held it.
    // SAFETY: as above.
    let _set: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    let mut collected: Vec<u8> = Vec::new();
    let mut buffer: [u8; 8_192] = [0; 8_192];
    loop {
        match unending.read(buffer.as_mut_slice()) {
            // The pipe ended, so everything the child wrote has been read.
            Ok(0) => return collected,
            Ok(read) => collected.extend_from_slice(buffer.get(..read).unwrap_or(buffer.as_slice())),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            // Nothing is there now, and the child has ended, so nothing of the child's will come later.
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return collected,
            Err(_) => return collected,
        }
    }
}

/// What:
///  Run the child test `name` in a process of its own and give back what it said and how it ended,
///  or nothing where it outlived
/// `CHILD_DEADLINE`.
/// Why:
///   A watchdog ends its whole process,
/// so what it does can only be seen from outside it;
/// the deadline is what turns a watcher
///  that never ends its child into a failed control instead of a run that never returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runChildUntil(name, 'piped'); // undefined when the deadline passed
/// ```
fn run_child(name: &str) -> Option<std::process::Output> {
    return run_child_with_stderr_until(name, std::process::Stdio::piped());
}

/// Child of the control below:
/// a run whose only task waits for a message nobody sends.
#[test]
#[ignore = "started by a_stuck_run_ends_its_process in a process of its own"]
fn child_that_stops_moving() {
    forbid_core_dumps();
    // A run whose watcher could not start has no bound at all, so that is a failure of this child, not a quiet skip.
    let _armed: std::thread::JoinHandle<()> = watch(Duration::from_secs(1)).expect("the watchdog thread starts");
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
    // The fixture's own watcher is enough here; this child's is the one-second window under test.
    let _armed: std::thread::JoinHandle<()> = watch(Duration::from_secs(1)).expect("the watchdog thread starts");
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
    let ended: Option<std::process::Output> = run_child("child_that_stops_moving");
    let output: std::process::Output = ended.expect("the watchdog ended the process within CHILD_DEADLINE");
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
/// because either watcher ends it,
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
    let _armed: std::thread::JoinHandle<()> = watch(Duration::from_secs(1)).expect("the watchdog thread starts");
    // The parent reads this count: a watcher ends this child either way, so only the count tells two apart from one.
    println!("watchers: {}", readers_running(WATCHERS_UNDER_TEST, Duration::from_secs(5)));
    // Nothing is ever sent, so only a watcher can end this child: a child that ended on a clock of its own would let a
    //  starved scheduler beat the watcher to it, and the control would fail while the watchdog was sound (measured
    //  2026-11-11, `doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`).
    let (_sender, receiver) = std::sync::mpsc::channel::<()>();
    receiver.recv().expect("unreachable: nothing is ever sent");
}

/// Two watchers of a run that shows nothing end it,
/// so a reading is never taken for the work of the run it reads.
#[test]
fn two_watchers_do_not_keep_each_other_alive() {
    let ended: Option<std::process::Output> = run_child("child_that_sleeps_under_two_watchers");
    let output: std::process::Output = ended.expect("a watcher ended the process within CHILD_DEADLINE");
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
    let ended: Option<std::process::Output> = run_child("child_that_keeps_moving");
    let output: std::process::Output = ended.expect("the moving child ended within CHILD_DEADLINE");
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
    let _armed: std::thread::JoinHandle<()> = watch(Duration::from_secs(1)).expect("the watchdog thread starts");
    // The wait reads the run itself at every look, and those reads are not the run's work.
    let _held: bool = crate::test_support::wait_until_for(crate::test_support::WaitingFor { window: Duration::from_secs(600), done: &|| { return false; } });
}

/// How long the child of the control below leaves a process holding its output open.
/// It is far longer than the start of a child test on this host (measured at up to 11 s under load),
/// so a collection that waits for the end of the pipe cannot pass by accident,
/// and the control ends that process itself,
/// so nothing of the run outlives it.
const PIPE_HOLDER_LIFE: Duration = Duration::from_secs(60);

/// How long the control below lets the collection of a child's output take before it counts the wait as unbounded.
/// A child that has ended has written its whole report,
/// so collecting what is in the pipe needs no more than the time a read of it takes.
const OUTPUT_COLLECTION_ALLOWED: Duration = Duration::from_secs(20);

/// Child of the control below:
/// a run whose direct child ends while a process it started holds the child's output pipe open.
#[test]
#[ignore = "started by a_child_that_leaves_a_pipe_holder_does_not_hold_the_collection_of_its_output in a process of its own"]
fn child_that_leaves_a_pipe_holder() {
    forbid_core_dumps();
    // A shell starts the holder in the background and says its number;
    // the shell itself ends at once,
    // so the holder outlives this child and keeps the child's output open.
    // The parent ends the holder once it has read the line,
    // so nothing of the run outlives the control.
    let shell: std::process::ExitStatus = std::process::Command::new("sh")
        .args(["-c", format!("sleep {} & echo \"holder $!\"", PIPE_HOLDER_LIFE.as_secs()).as_str()])
        .status()
        .expect("a shell starts a process that holds this child's output");
    assert!(shell.success(), "the shell that starts the holder ended with {shell}");
}

/// What:
///  A child that ends before its deadline is collected within the time a read of what it wrote takes,
///  though a process it started still holds the pipe its output comes on.
/// Why:
///   The pipe of a child's output ends only when every process holding it lets go,
/// and a process the child started keeps it open after the child ends;
/// a control that read until the end of the pipe waited for that process's whole life,
/// and one whose holder never ends waits for ever.
#[test]
fn a_child_that_leaves_a_pipe_holder_does_not_hold_the_collection_of_its_output() {
    let started: Instant = Instant::now();
    let ended: Option<std::process::Output> = run_child_with_stderr_within(
        "child_that_leaves_a_pipe_holder",
        std::process::Stdio::piped(),
        CHILD_DEADLINE,
    );
    let elapsed: Duration = started.elapsed();
    let output: std::process::Output = ended.expect("the direct child ended before its deadline");
    let ran: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    // The child's line lands where libtest's own line ends, so the figure is taken from the words of the output.
    let held: Option<i32> = ran
        .split_whitespace()
        .skip_while(|word: &&str| { return *word != "holder"; })
        .nth(1)
        .and_then(|figure: &str| { return figure.parse::<i32>().ok(); });
    if let Some(pid) = held {
        // Nothing of the run outlives this control: the holder is this child's own descendant.
        let _ended: libc::c_int = unsafe { libc::kill(pid, libc::SIGKILL) };
    }
    assert!(output.status.success(), "the direct child ended on its own:\n{ran}");
    assert!(
        ran.contains(child_path("child_that_leaves_a_pipe_holder").as_str()),
        "the child ran the test named, not nothing:\n{ran}"
    );
    assert!(held.is_some(), "the child named the process that holds its output:\n{ran}");
    assert!(
        elapsed < OUTPUT_COLLECTION_ALLOWED,
        "the collection of a child's output waited for a process that holds the pipe after the child ended: it returned after {elapsed:?}, its whole life being {PIPE_HOLDER_LIFE:?}"
    );
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
