//! Controls for the meter and the clock of a push's progress,
//! on real short-lived processes that write,
//! sleep and start
//! others at known times.
//! Why:
//!  The stop on no progress is only as exact as the bytes it counts and the limits it keeps,
//! so each is pinned on both sides:
//!  bytes of the root and of what it started count,
//! an idle tree counts nothing,
//! a moving tree never stalls,
//! a stalled tree
//!  stops at its window,
//! no total time stops a moving tree,
//! and where nothing is measured the window is a fixed deadline.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(meter.sample(pid)).toBe(true); await sleep(idle); expect(meter.sample(pid)).toBe(false);
//! ```
#![cfg(unix)]

use super::*;
use crate::test_support::{fixture, remove, spawn_owned};
use std::process::{Child, Command, Stdio};
/// The name the control gives the probe test in order to run it as a process of its own.
const PROBE_NAME: &str = "push::push_progress::tests::disk_wait_probe";

/// The variable that holds the probe's hold in milliseconds;
/// without it the probe does nothing.
const PROBE_HOLD_VARIABLE: &str = "CLI_GIT_DISK_WAIT_PROBE_HOLD_MILLISECONDS";

/// The variable that says what the probe does when its hold is over:
/// `exit`,
/// or `idle` to sleep as a process waiting on a server.
const PROBE_THEN_VARIABLE: &str = "CLI_GIT_DISK_WAIT_PROBE_THEN";

/// The variable that names a file the probe creates from inside its hold;
/// without it the probe announces nothing.
const PROBE_ANNOUNCE_VARIABLE: &str = "CLI_GIT_DISK_WAIT_PROBE_ANNOUNCE_PATH";

/// How long the probe of the disk wait is suspended before it says so,
/// so that the announcement comes from inside the hold and never from a process still on its way into it.
const ANNOUNCE_AFTER: Duration = Duration::from_millis(50);

/// Pause between two looks of a control,
/// shorter than every limit it uses.
const LOOK: Duration = Duration::from_millis(50);

/// How often a control asks whether a fact it waits for has happened,
/// so a fact that never comes fails the control instead of holding it.
const POLL: Duration = Duration::from_millis(5);

/// How long a control waits for something that must happen,
/// so a broken meter fails it and never holds it.
const PATIENCE: Duration = Duration::from_secs(20);

/// A shell command running `script` in `directory`,
/// with its output discarded.
fn shell(script: &str, directory: &std::path::Path) -> Child {
    return spawn_owned(
        Command::new("sh")
            .arg("-c")
            .arg(script)
            .current_dir(directory)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null()),
    )
    .expect("start the shell");
}

/// End a child the control started.
fn end(child: Child) {
    crate::child_ended_with_its_whole_group::end_with_its_group(child);
}

/// Look at the tree until a sample shows nothing new,
/// which an idle tree does within a few looks of its start.
fn settle(meter: &mut Meter, root: i32) {
    let began: Instant = Instant::now();
    while meter.sample(root) {
        assert!(began.elapsed() < PATIENCE, "the tree never went idle");
        std::thread::sleep(LOOK);
    }
}

/// What:
///  An idle tree moves nothing between looks,
///  and a write of the root shows at the next look.
/// Why the child says it has started before the tree is settled:
///   A tree is idle only once the child is:
/// settling before the shell was scheduled makes the shell's own start the next
/// look's movement,
/// which read as a write of the root and failed this control once in a run of 1,183 tests while it passed
///  alone in under five seconds (report `kill-sweeps-reach-the-calls-they-name-1`,
/// 2026-11-09).
/// The child writes `started` first,
/// and the
///  control waits for that file,
/// so the order is the control's and never the scheduler's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = shell('echo > started; sleep 0.6; echo a >> log; sleep 60', root);
/// await exists(join(root, 'started')); settle(meter, child.pid); expect(meter.sample(pid)).toBe(false);
/// ```
#[test]
fn a_write_of_the_root_is_progress_and_an_idle_tree_is_not() {
    let root: std::path::PathBuf = fixture("progress-root");
    // The child says it has started and is about to sleep, so the tree is settled after the shell's own start and not before.
    let child: Child = shell(": > started; sleep 0.6; echo a >> log; sleep 60", root.as_path());
    let pid: i32 = i32::try_from(child.id()).expect("pid");
    let began_waiting: Instant = Instant::now();
    while !root.join("started").exists() {
        assert!(began_waiting.elapsed() < PATIENCE, "the child never said it had started");
        std::thread::sleep(POLL);
    }
    let mut meter: Meter = Meter::new();
    settle(&mut meter, pid);
    let idle: u64 = meter.moved();
    assert!(!meter.sample(pid), "nothing moved between two looks of an idle tree");
    assert_eq!(meter.moved(), idle);
    let began: Instant = Instant::now();
    let mut seen: bool = false;
    while !seen && began.elapsed() < PATIENCE {
        std::thread::sleep(LOOK);
        seen = meter.sample(pid);
    }
    assert!(seen, "the write of the root was seen");
    assert!(meter.moved() > idle);
    end(child);
    remove(root.as_path());
}

/// What a process started by the root writes counts too:
/// the root itself only waits.
#[test]
fn a_write_of_a_process_the_root_started_is_progress() {
    let root: std::path::PathBuf = fixture("progress-descendant");
    let child: Child = shell("(sleep 0.6; echo a >> log; sleep 60) & wait", root.as_path());
    let pid: i32 = i32::try_from(child.id()).expect("pid");
    let mut meter: Meter = Meter::new();
    settle(&mut meter, pid);
    let idle: u64 = meter.moved();
    let began: Instant = Instant::now();
    let mut seen: bool = false;
    while !seen && began.elapsed() < PATIENCE {
        std::thread::sleep(LOOK);
        seen = meter.sample(pid);
    }
    assert!(seen, "the write of the descendant was seen");
    assert!(meter.moved() > idle);
    // Any byte moved sets `seen` (a read counts), so the control waits for the write itself before it ends the tree.
    while !root.join("log").exists() && began.elapsed() < PATIENCE {
        std::thread::sleep(LOOK);
    }
    assert!(root.join("log").exists(), "the descendant wrote its line");
    end(child);
    remove(root.as_path());
}

/// Ending a control's child ends what the child started too,
/// before the fixture goes:
/// a background subshell that outlives its leader
///  would write into the fixture while it is removed (`DirectoryNotEmpty`,
/// seen in the gate of `85e6d2ab99`).
/// The subshell here would
///  create `log` 0.3 s after it starts,
/// so a `log` that shows up after the child was ended means it was left running.
#[test]
fn ending_a_child_ends_the_processes_it_started_too() {
    let root: std::path::PathBuf = fixture("progress-end-group");
    let child: Child = shell("(sleep 0.3; echo a >> log; sleep 60) & wait", root.as_path());
    let pid: i32 = i32::try_from(child.id()).expect("pid");
    // The subshell must exist before the child is ended, or there is nothing left running to find.
    let began: Instant = Instant::now();
    while crate::process_tree::descendants_of(pid).is_empty() {
        assert!(began.elapsed() < PATIENCE, "the shell never started its subshell");
        std::thread::sleep(Duration::from_millis(5));
    }
    end(child);
    std::thread::sleep(Duration::from_millis(600));
    let left_running: bool = root.join("log").exists();
    remove(root.as_path());
    assert!(!left_running, "a process the ended child had started was left running and wrote into the fixture");
}

/// A process that ends takes its counters with it,
/// and the bytes it moved stay counted.
#[test]
fn the_bytes_of_a_process_that_ended_stay_counted() {
    let root: std::path::PathBuf = fixture("progress-ended");
    let child: Child = shell("(echo a >> log; echo b >> log) ; sleep 60", root.as_path());
    let pid: i32 = i32::try_from(child.id()).expect("pid");
    let mut meter: Meter = Meter::new();
    settle(&mut meter, pid);
    let counted: u64 = meter.moved();
    assert!(counted > 0, "the shell and its subshell moved bytes");
    assert!(!meter.sample(pid));
    assert_eq!(meter.moved(), counted, "a subshell that has ended does not take its bytes away");
    end(child);
    remove(root.as_path());
}

/// A process that does not exist moves nothing,
/// and the system counts bytes for a process that does.
#[test]
fn a_missing_process_moves_nothing_and_this_system_counts_bytes() {
    assert_eq!(bytes_moved_by(i32::MAX), None);
    assert!(bytes_moved_by(i32::try_from(std::process::id()).expect("pid")).is_some());
    assert!(system_measures());
    assert!(!Meter::new().sample(i32::MAX));
}

/// A fixed patience stops at its limit,
/// and not before.
#[test]
fn a_fixed_patience_stops_at_its_limit() {
    let child: Child = spawn_owned(Command::new("sleep").arg("60")).expect("sleep");
    let limit: Duration = Duration::from_millis(400);
    let mut clock: Clock = Clock::start(child.id(), Patience::Fixed(limit));
    let began: Instant = Instant::now();
    assert!(clock.check().is_none(), "not yet");
    let cause: StopCause = loop {
        if let Some(cause) = clock.check() {
            break cause;
        }
        assert!(began.elapsed() < PATIENCE);
        std::thread::sleep(LOOK);
    };
    assert!(matches!(cause, StopCause::Deadline));
    assert!(began.elapsed() >= limit);
    end(child);
}

/// A tree that moves nothing stalls at its window,
/// with the window,
/// the bytes moved and the time in the report.
#[test]
fn a_tree_that_moves_nothing_stalls_at_its_window() {
    let child: Child = spawn_owned(Command::new("sleep").arg("60")).expect("sleep");
    let stall: Duration = Duration::from_millis(1200);
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall });
    let began: Instant = Instant::now();
    let cause: StopCause = loop {
        if let Some(cause) = clock.check() {
            break cause;
        }
        assert!(began.elapsed() < PATIENCE);
        std::thread::sleep(LOOK);
    };
    let StopCause::Stalled(report) = cause else { panic!("a stall, not another limit") };
    assert_eq!(report.limit, stall);
    assert!(report.elapsed >= stall, "{:?}", report.elapsed);
    // Progress, and not a clock: the report carries the bytes the meter counted, which is what says the stop was measured from
    //  the tree and not from a constant.
    assert_eq!(Some(report.moved), clock.meter.as_ref().map(Meter::moved), "the report carries the bytes the meter counted");
    // The ceiling below is the one place this control still reads the clock, because the detector reads it too: nothing about
    //  the tree's progress bounds how late this host wakes the poll loop. Measured on this host at a load average of 47: the
    //  control reported at 1.7 s for a window of 1.2 s, so the 4.2 s ceiling carries 2.4 times the figure it was measured at.
    assert!(report.elapsed < stall + SAMPLE_INTERVAL * 6, "{:?}", report.elapsed);
    end(child);
}

/// A tree that keeps moving is not stopped for time at all:
/// no total bound ends a push that shows progress.
#[test]
fn a_tree_that_keeps_moving_is_never_stopped_for_time() {
    let root: std::path::PathBuf = fixture("progress-no-ceiling");
    let child: Child = shell("while :; do echo x >> log; sleep 0.1; done", root.as_path());
    let stall: Duration = Duration::from_millis(1500);
    let watched: Duration = stall * 3;
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall });
    let began: Instant = Instant::now();
    let mut stopped: Option<StopCause> = None;
    while stopped.is_none() && began.elapsed() < watched {
        stopped = clock.check();
        std::thread::sleep(LOOK);
    }
    assert!(stopped.is_none(), "data kept moving, so nothing stops the tree");
    end(child);
    remove(root.as_path());
}

/// Whether the process `pid` still runs:
/// a process that is gone,
/// or ended and waiting to be reaped,
/// does not.
fn process_runs(pid: u32) -> bool {
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else { return false };
    return stat.rsplit_once(')').and_then(|(_, rest)| { return rest.trim_start().chars().next(); }) != Some('Z');
}

/// The loop that leaked for 87 minutes,
/// started by a test that then panics,
/// is gone when the test ends:
/// a failing test
///  must not leave its helper behind,
/// with its `sleep` either.
#[test]
fn a_loop_started_by_a_panicking_test_is_gone_when_the_test_ends() {
    let (sender, receiver) = std::sync::mpsc::channel::<u32>();
    let handle = std::thread::spawn(move || {
        let root: std::path::PathBuf = fixture("progress-panic");
        let mut child: Child = shell("while :; do echo x >> log; sleep 0.1; done", root.as_path());
        sender.send(child.id()).expect("send the pid");
        // A look that does not wait: the loop is still running when the test fails, as in the leak this control is for.
        let _still_running = child.try_wait();
        panic!("the test fails before it ends the loop");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let pid: u32 = receiver.recv().expect("the pid");
    let began: Instant = Instant::now();
    while process_runs(pid) && began.elapsed() < PATIENCE {
        std::thread::sleep(LOOK);
    }
    let left: bool = process_runs(pid);
    if left {
        // Tidy up the leak this control exists to catch, so a red run leaves nothing behind either.
        // SAFETY: `kill` takes a pid and a signal and touches no memory.
        unsafe { libc::kill(libc::pid_t::try_from(pid).expect("pid"), libc::SIGKILL) };
    }
    assert!(!left, "the loop of process {pid} outlived the test that started it");
}

/// A process that product code started in the fixture,
/// and that no guard of the test holds,
/// is gone when the test ends:
/// a
///  hook that does `exec sleep` cannot watch its test without starting processes the meter would count as progress,
/// so the
///  fixture ends what works in it.
#[test]
fn a_process_working_in_a_fixture_is_gone_when_the_test_ends() {
    let (sender, receiver) = std::sync::mpsc::channel::<u32>();
    let handle = std::thread::spawn(move || {
        let root: std::path::PathBuf = fixture("progress-cwd-sweep");
        let mut child: Child = Command::new("sleep")
            .arg("600")
            .current_dir(root.as_path())
            .stdin(Stdio::null())
            .spawn()
            .expect("start the sleeper");
        sender.send(child.id()).expect("send the pid");
        // A look that does not wait: the sleeper is still running when the test fails, as in the leak this control is for.
        let _still_running = child.try_wait();
        panic!("the test fails before it ends the sleeper");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let pid: u32 = receiver.recv().expect("the pid");
    let began: Instant = Instant::now();
    while process_runs(pid) && began.elapsed() < PATIENCE {
        std::thread::sleep(LOOK);
    }
    let left: bool = process_runs(pid);
    if left {
        // Tidy up the leak this control exists to catch, so a red run leaves nothing behind either.
        // SAFETY: `kill` takes a pid and a signal and touches no memory.
        unsafe { libc::kill(libc::pid_t::try_from(pid).expect("pid"), libc::SIGKILL) };
    }
    assert!(!left, "the sleeper of process {pid} outlived the test that started it");
}

/// Where nothing is measured the stall window is a fixed deadline,
/// even for a tree that writes all the time.
#[test]
fn without_a_measure_the_stall_window_is_a_fixed_deadline() {
    let root: std::path::PathBuf = fixture("progress-unmeasured");
    let child: Child = shell("while :; do echo x >> log; sleep 0.1; done", root.as_path());
    let stall: Duration = Duration::from_millis(600);
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall }).unmeasured();
    let began: Instant = Instant::now();
    let cause: StopCause = loop {
        if let Some(cause) = clock.check() {
            break cause;
        }
        assert!(began.elapsed() < PATIENCE);
        std::thread::sleep(LOOK);
    };
    assert!(matches!(cause, StopCause::Deadline), "no measure, so the window is the whole deadline");
    assert!(began.elapsed() >= stall);
    end(child);
    remove(root.as_path());
}

/// What the vfork child of a disk wait is told:
/// how long to sleep,
/// when to announce,
/// and what to create as the announcement.
///  It lives in the memory the child shares with the thread that started it,
/// and that thread's frame outlives the child,
/// so the child reads it and writes nothing.
#[repr(C)]
struct DiskWaitHold {
    /// How long the child sleeps before it ends,
    /// in milliseconds.
    milliseconds: usize,
    /// How long the child sleeps before it announces,
    /// in milliseconds,
    /// so the announcement is made from inside the hold.
    announce_after_milliseconds: usize,
    /// A NUL-terminated path the child creates once the thread that holds is suspended,
    /// or null for no announcement.
    announce: *const libc::c_char,
}

/// Sleep for `milliseconds` in a way that only the async-signal-safe calls of a vfork child may use:
/// the child shares the memory of a suspended thread,
/// so it may not allocate,
/// take a lock or return.
extern "C" fn sleep_for_the_hold(milliseconds: usize) {
    let pause: libc::timespec = libc::timespec {
        tv_sec: libc::time_t::try_from(milliseconds / 1000).unwrap_or(0),
        tv_nsec: libc::c_long::try_from((milliseconds % 1000) * 1_000_000).unwrap_or(0),
    };
    // SAFETY: `nanosleep` touches only the `timespec` it is given and the thread that calls it.
    unsafe {
        libc::nanosleep(&pause, std::ptr::null_mut());
    }
}

/// What the probe's child does for its hold:
/// sleep in each of two phases,
/// create the announcement between them,
/// and end.
/// The probe's thread is blocked in `clone` with
///  `CLONE_VFORK`,
/// which the kernel waits out in uninterruptible sleep (state `D`) without any disk,
/// byte or processor time.
///  The announcement is created in the second phase,
/// so it says that the calling thread is suspended in that wait and not that it is on its way into it.
extern "C" fn hold_in_vfork_child(note: *mut libc::c_void) -> libc::c_int {
    // SAFETY: `note` points at the `DiskWaitHold` of the frame that called `clone`, which outlives this child (that frame
    //  returns only after this child ended); the child calls only `nanosleep`, `open`, `close` and `_exit`, all
    //  async-signal-safe and none of them allocating. The child shares the parent's memory until it ends, so it touches
    //  nothing of the parent but the read-only note and the stack this child owns.
    unsafe {
        let hold: &DiskWaitHold = &*note.cast::<DiskWaitHold>();
        sleep_for_the_hold(hold.announce_after_milliseconds);
        if !hold.announce.is_null() {
            let made: libc::c_int =
                libc::open(hold.announce, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC | libc::O_CLOEXEC, 0o644 as libc::mode_t);
            if made >= 0 {
                let _closed: libc::c_int = libc::close(made);
            }
        }
        sleep_for_the_hold(hold.milliseconds);
        libc::_exit(0);
    }
}

/// Hold the calling thread in uninterruptible sleep for `milliseconds`:
/// no I/O,
/// no processor time,
/// no byte moved,
/// which is
///  what a wait for a slow disk looks like to `/proc` and what no file system,
/// device or FUSE mount is needed to make.
///  `announce` is created from inside the hold,
/// so whoever waits for that file knows the thread is suspended.
fn hold_in_disk_wait(milliseconds: usize, announce: Option<&std::path::Path>) {
    let path: Option<std::ffi::CString> = announce.map(|named: &std::path::Path| {
        return std::ffi::CString::new(named.as_os_str().as_encoded_bytes()).expect("a path without a nul byte");
    });
    let note: DiskWaitHold = DiskWaitHold {
        milliseconds,
        announce_after_milliseconds: usize::try_from(ANNOUNCE_AFTER.as_millis()).unwrap_or(0),
        announce: path.as_ref().map_or(std::ptr::null(), |held: &std::ffi::CString| { return held.as_ptr(); }),
    };
    let mut stack: Vec<u8> = vec![0; 64 * 1024];
    // SAFETY: the child gets its own stack (`stack` outlives the call, which returns only after the child ended) and runs
    //  `hold_in_vfork_child`, which only sleeps, creates one file and exits; the parent waits for it with `waitpid` so no
    //  zombie is left, and the note it reads lives in this frame, which is unmoved while the child runs.
    unsafe {
        let top: *mut libc::c_void = stack.as_mut_ptr().add(stack.len()).cast();
        let held: *mut libc::c_void = std::ptr::from_ref(&note).cast_mut().cast();
        let child: libc::pid_t = libc::clone(hold_in_vfork_child, top, libc::CLONE_VM | libc::CLONE_VFORK | libc::SIGCHLD, held);
        assert!(child > 0, "clone failed: {}", std::io::Error::last_os_error());
        libc::waitpid(child, std::ptr::null_mut(), 0);
    }
}

/// The process the controls start when they need a tree in disk wait:
/// not a test of its own,
/// it does nothing unless the
///  variable names a hold,
/// and then holds a thread in uninterruptible sleep for that long and either ends or sleeps on.
///  It creates the file `PROBE_ANNOUNCE_VARIABLE` names from inside the hold,
/// so a control that reads it knows the tree is
/// suspended rather than still starting up.
#[test]
fn disk_wait_probe() {
    let Some(hold) = std::env::var(PROBE_HOLD_VARIABLE).ok().and_then(|text| { return text.parse::<usize>().ok(); }) else {
        return;
    };
    let announce: Option<std::path::PathBuf> = std::env::var_os(PROBE_ANNOUNCE_VARIABLE).map(std::path::PathBuf::from);
    hold_in_disk_wait(hold, announce.as_deref());
    if std::env::var(PROBE_THEN_VARIABLE).as_deref() == Ok("idle") {
        std::thread::sleep(Duration::from_secs(600));
    }
}

/// The command that runs the probe:
/// this test binary,
/// the probe only,
/// holding for `hold`,
/// then `then`,
/// and creating `announce` from inside the hold when it names a path.
pub(crate) fn disk_wait_command(hold: Duration, then: &str, announce: Option<&std::path::Path>) -> Command {
    #[allow(clippy::disallowed_methods, reason = "identifies the running test binary so the control can start the probe test of it as a process of its own")]
    let test_binary: std::path::PathBuf = std::env::current_exe().expect("this test binary");
    let mut command: Command = Command::new(test_binary);
    command
        .args(["--exact", PROBE_NAME, "--nocapture", "--test-threads", "1"])
        .env(PROBE_HOLD_VARIABLE, hold.as_millis().to_string())
        .env(PROBE_THEN_VARIABLE, then)
        .stdin(Stdio::null());
    if let Some(path) = announce {
        command.env(PROBE_ANNOUNCE_VARIABLE, path);
    }
    return command;
}

/// Wait until the probe has created the file it announces its hold with.
///  A control that looks before that point is looking at a process still starting up,
/// which is work for the same meter;
///  so the first look of the control below follows the announcement and never a guess at how fast this host starts a process.
fn wait_for_announcement(announced: &std::path::Path) {
    let began: Instant = Instant::now();
    while !announced.exists() {
        assert!(began.elapsed() < PATIENCE, "the probe never announced its hold: {}", announced.display());
        std::thread::sleep(POLL);
    }
}

/// A thread in uninterruptible sleep is a sign of life at every look,
/// with nothing else moving:
/// the wait of a process for
///  its disk (the fsync of an object,
/// the update of a ref) is not a stall.
/// The probe's hold ends and it sleeps on,
/// and then
///  the tree is idle at last.
///  Every look falls inside the hold:
/// the probe creates its announcement from inside the hold,
/// and the look that  records the counters the others are measured against comes after it,
/// so nothing here is timed from the probe's start.
#[test]
fn a_tree_in_disk_wait_is_progress_at_every_look_until_it_ends() {
    let root: std::path::PathBuf = fixture("progress-disk-wait");
    let announced: std::path::PathBuf = root.join("in-disk-wait");
    let mut probe: Command = disk_wait_command(Duration::from_millis(4000), "idle", Some(announced.as_path()));
    probe.stdout(Stdio::null()).stderr(Stdio::null());
    let child: Child = spawn_owned(&mut probe).expect("start the probe");
    let pid: i32 = i32::try_from(child.id()).expect("pid");
    let mut meter: Meter = Meter::new();
    wait_for_announcement(announced.as_path());
    // The counters and the states of the tree as the hold began: what the looks below find is a change since this look, so a
    //  process seen for the first time cannot pass one of them by being new.
    assert!(meter.sample(pid), "the probe moved bytes on its way into the hold, and this look records them");
    for look in 0..4 {
        std::thread::sleep(Duration::from_millis(400));
        assert!(meter.sample(pid), "look {look}: a thread in uninterruptible sleep is alive");
    }
    let began: Instant = Instant::now();
    let mut idle: bool = false;
    while !idle && began.elapsed() < PATIENCE {
        std::thread::sleep(Duration::from_millis(400));
        idle = !meter.sample(pid);
    }
    assert!(idle, "once the hold is over and the probe sleeps, the tree is idle");
    end(child);
    remove(root.as_path());
}

/// A push-like tree in disk wait for longer than the stall window is not stopped,
/// and one that sleeps on afterwards is.
#[test]
fn a_clock_does_not_stall_a_tree_in_disk_wait_and_stalls_it_after() {
    let mut probe: Command = disk_wait_command(Duration::from_millis(4000), "idle", None);
    probe.stdout(Stdio::null()).stderr(Stdio::null());
    let child: Child = spawn_owned(&mut probe).expect("start the probe");
    let stall: Duration = Duration::from_millis(1500);
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall });
    // The probe's own start-up counts as work; the hold begins after it and lasts well past the window.
    let began: Instant = Instant::now();
    let cause: StopCause = loop {
        if let Some(cause) = clock.check() {
            break cause;
        }
        assert!(began.elapsed() < PATIENCE);
        std::thread::sleep(LOOK);
    };
    let StopCause::Stalled(report) = cause else { panic!("a stall, not another limit") };
    assert!(
        report.elapsed >= Duration::from_millis(4000),
        "the hold of 4 s in disk wait is no stall, and the stall came only after it: {:?}",
        report.elapsed
    );
    end(child);
}

/// A push whose connection to the remote is open and silent is ended when the connected window passes (owner,
/// 2026-11-03:
/// "Bound
/// it,
/// keep retrying"):
/// the clock stops it as a stall,
/// long after the stall window of a push with no connection,
/// and the
/// service's retry plan keeps the push.
#[test]
fn a_silent_open_connection_is_ended_at_the_connected_window() {
    let listener: std::net::TcpListener = std::net::TcpListener::bind("127.0.0.1:0").expect("a listener");
    let port: u16 = listener.local_addr().expect("address").port();
    // The child holds an established connection to the listener and then does nothing: the remote that has not answered.
    let mut holder: Command = Command::new("bash");
    holder.args(["-c", format!("exec 3<>/dev/tcp/127.0.0.1/{port}; sleep 30").as_str()]).stdout(Stdio::null()).stderr(Stdio::null());
    let child: Child = spawn_owned(&mut holder).expect("start the holder");
    let (_accepted, _) = listener.accept().expect("the holder connects");
    let stall: Duration = Duration::from_millis(1500);
    let window: Duration = Duration::from_millis(3500);
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall }).with_connected_window(window);
    let began: Instant = Instant::now();
    let cause: Option<StopCause> = loop {
        if let Some(cause) = clock.check() {
            break Some(cause);
        }
        if began.elapsed() > window * 4 {
            break None;
        }
        std::thread::sleep(LOOK);
    };
    end(child);
    let Some(StopCause::Stalled(report)) = cause else { panic!("the silent connection was not ended within {:?}", began.elapsed()) };
    assert!(report.limit == window && began.elapsed() >= window, "ended at the connected window: {:?}", began.elapsed());
    // The words of the stop name the window that fired, and never offer the stall setting as its remedy: the setting does
    //  not change the connected window, so advice to set it sends its reader to a limit that cannot be moved.
    let words: String = crate::lfs_upload_explanations::stopped_upload_words(&StopCause::Stalled(report));
    assert!(!words.contains("to allow longer"), "a connected stop does not offer a setting that cannot change its window: {words}");
    assert!(
        words.contains(&format!("{} does not change this window", crate::bounded_push::PUSH_DEADLINE_VARIABLE)),
        "the setting is named as one that does not move this window: {words}"
    );
    assert!(words.contains("fixed at"), "the window it was ended by is named: {words}");
    assert!(words.contains("git lfs push --all"), "a remedy that works is named: {words}");
}

/// What:
///  A tree whose only change between two looks is the processor time of the process that holds the connection
/// is waiting,
/// and its wait ends at the connected window.
/// Why:
///   The helper that speaks HTTP for a push polls its socket while it waits,
/// and the kernel charges it one tick every few seconds:
/// measured on 2026-11-11 against a server that takes the connection and never answers,
/// `git-remote-http` used one tick every three seconds and moved no byte.
/// A clock that counts those ticks as work
/// never reaches a connected window,
/// which is what the audit's F8 measured:
/// 290 to 330 seconds of a service push that had a 150 second window.
///  The holder here does what that helper does:
/// it holds its connection and burns a few processor milliseconds between two polls of it,
/// so every look finds either a tick or a task able to run,
/// and the tree is still waiting.
#[test]
fn a_connected_tree_that_only_uses_processor_time_is_waiting() {
    let listener: std::net::TcpListener = std::net::TcpListener::bind("127.0.0.1:0").expect("a listener");
    let port: u16 = listener.local_addr().expect("address").port();
    // No fork and no write: the holder only burns the processor between two timed reads of the socket it holds.
    let mut holder: Command = Command::new("bash");
    holder
        .args([
            "-c",
            format!("exec 3<>/dev/tcp/127.0.0.1/{port}; while :; do n=0; while [ $n -lt 4000 ]; do n=$((n+1)); done; read -t 0.1 -u 3 line; done")
                .as_str(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let child: Child = spawn_owned(&mut holder).expect("start the holder");
    let (_accepted, _) = listener.accept().expect("the holder connects");
    let stall: Duration = Duration::from_millis(1500);
    let window: Duration = Duration::from_secs(4);
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall }).with_connected_window(window);
    let began: Instant = Instant::now();
    let cause: Option<StopCause> = loop {
        if let Some(cause) = clock.check() {
            break Some(cause);
        }
        if began.elapsed() > window * 3 {
            break None;
        }
        std::thread::sleep(LOOK);
    };
    end(child);
    let Some(StopCause::Stalled(report)) = cause else {
        panic!("a tree whose only sign of life is its holder's processor time was not ended within {:?}", began.elapsed())
    };
    assert_eq!(report.window, StopWindow::Connected, "ended by the connected window after {:?}", began.elapsed());
    assert!(began.elapsed() >= window, "ended at the connected window: {:?}", began.elapsed());
}

/// A `/proc/<pid>/stat` line is read after its last parenthesis:
/// the state,
/// and the user and system ticks added.
#[test]
fn a_stat_line_gives_its_state_and_its_processor_ticks() {
    let line: &str = "4242 (a (odd) name) D 1 4242 4242 0 -1 4194560 100 0 0 0 7 5 0 0 20 0 1 0 12345 1000 10 18446744073709551615";
    assert_eq!(parse_activity(line), Some(('D', 12)));
    assert_eq!(parse_activity("1 (x) S 0 1"), None, "a line cut short gives nothing");
    assert_eq!(parse_activity("no parenthesis"), None);
    assert!(is_working('R') && is_working('D'));
    for resting in ['S', 'T', 't', 'Z', 'I'] {
        assert!(!is_working(resting), "{resting}");
    }
}

/// Each sign of work counts alone,
/// at the look that finds it,
/// and a process that shows none is idle:
/// bytes,
/// processor ticks,
///  a task able to run or waiting for the disk.
#[test]
fn each_sign_of_work_counts_alone_and_nothing_counts_for_nothing() {
    let key: (i32, u64) = (100, 1);
    let mut meter: Meter = Meter::new();
    assert!(!meter.observe(key, 0, 0, false), "a process that has done nothing is idle at its first look");
    assert!(!meter.observe(key, 0, 0, false));
    assert!(meter.observe(key, 10, 0, false), "bytes");
    assert_eq!(meter.moved(), 10);
    assert!(!meter.observe(key, 10, 0, false), "the same bytes are not new work");
    assert!(meter.observe(key, 10, 1, false), "one processor tick");
    assert!(!meter.observe(key, 10, 1, false));
    assert!(meter.observe(key, 10, 1, true), "a task able to run or waiting for the disk, at every look it is found");
    assert!(meter.observe(key, 10, 1, true));
    assert!(!meter.observe(key, 10, 1, false), "and idle when it is not");
    assert_eq!(meter.moved(), 10, "processor time and states are not bytes");
    assert!(meter.observe((200, 1), 0, 3, false), "a new process that has already used time");
    assert!(!meter.observe(key, 5, 1, false), "a counter that fell (a number reused) is not work");
}

/// A child that ends before the first sample interval has passed is never looked for among the machine's processes:
/// the
/// listing reads the status file of every process the host runs (841 files,
/// 7 ms on this host at a load of 40,
/// and more where
/// more run),
/// and the wrapper watches every Git it starts,
/// a dozen to two dozen for one commit,
/// most of them ended within
/// ten milliseconds.
/// The first look at a child that has just started needs only the child itself.
#[test]
fn a_child_that_has_just_started_is_looked_at_without_listing_the_machine() {
    let root: std::path::PathBuf = fixture("progress-first-look");
    let child: Child = shell("sleep 60", root.as_path());
    let pid: u32 = child.id();
    let mut clock: Clock = Clock::start(pid, Patience::Progress { stall: PATIENCE });
    assert!(clock.measures(), "this system counts what a child moves, so the clock samples");
    assert!(clock.check().is_none(), "a child that has just started is not stalled");
    assert_eq!(clock.meter.as_ref().map(Meter::scans), Some(0), "the first look listed the machine's processes");
    // Positive control: the clock does list the machine's processes once the sample interval has passed.
    std::thread::sleep(SAMPLE_INTERVAL + LOOK);
    assert!(clock.check().is_none(), "a child at work is not stalled");
    assert_eq!(clock.meter.as_ref().map(Meter::scans), Some(1), "the next look, one interval later, lists them");
    end(child);
    remove(root.as_path());
}
