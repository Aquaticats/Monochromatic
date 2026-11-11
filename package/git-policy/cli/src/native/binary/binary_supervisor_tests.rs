//! Controls for the hang guard of the binary tests:
//! the judge on built samples,
//! the readers of `/proc`,
//! the pinned identities,
//!  and real commands that loop,
//! deadlock,
//! wait for a disk,
//! wait on the test and crawl,
//! run under the monitor with limits short
//!  enough to see the verdict.
//! A loop and a deadlock must be ended and reported with the command,
//! the reason and the tasks of the
//!  tree;
//! a command that is slow but moving,
//! waits for the disk,
//! is held by the test or by a signal,
//! or has finished must not be.

use super::support::{Fixture, bounded, bounded_for, executable, fixture, own_group, remove, spawn_owned, supervised_wait, wrapper_first_path};
use super::supervisor::{
    CPU_BUDGET, Judge, Limits, NO_PROGRESS_WINDOW, POLL, Reason, SHARED, SHARED_BACKSTOP_SECONDS, UNCLAIMED_EXIT_STATUS, UNCLAIMED_GRACE,
    disk_wait_looks, end_command, hold_idle_judgment, is_shared, is_shared_command_line, is_watching, kill_recorded, looks_of, plant_kill, take_kill,
    unclaimed_kill, watch,
};
use super::supervisor_pin::{Delivered, Pin, end_tree};
use super::supervisor_proc::{Look, Sample, children, describe, look_at, note_states, parse_io, parse_stat, stat_of, task_states, tick, tree_of, waits_for_an_answer};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The length of a clock tick the built samples assume.
const TICK: Duration = Duration::from_millis(10);

/// The limits of the shared bound with the tick of the built samples.
fn judge() -> Judge {
    return Judge::new(SHARED, TICK);
}

/// A look at a tree that shows the same thing every time:
/// two processes,
/// nothing runnable.
fn still() -> Sample {
    return Sample { cpu_ticks: 7, activity: 100, child_faults: 40, processes: vec![(10, 1), (11, 1)], runnable: false, disk_wait: false, answer_wait: false, held: false };
}

/// A tree that shows nothing for the window is hung,
/// and not one look earlier.
#[test]
fn a_tree_that_stands_still_is_hung_when_the_window_has_passed() {
    let mut judge: Judge = judge();
    let step: Duration = Duration::from_millis(500);
    let mut elapsed: Duration = Duration::ZERO;
    assert_eq!(judge.observe(elapsed, &still(), false), None, "the first look is a first look");
    let verdict: Reason = loop {
        elapsed += step;
        if let Some(reason) = judge.observe(elapsed, &still(), false) {
            break reason;
        }
        assert!(elapsed < NO_PROGRESS_WINDOW, "the window passed and the tree was not judged hung");
    };
    assert_eq!(verdict, Reason::NoProgress(NO_PROGRESS_WINDOW), "judged exactly when the window passed");
}

/// Each sign of life restarts the window:
/// processor time,
/// bytes moved,
/// the work of waited-for children,
/// a new process,
/// a
/// process replaced by another of the same number,
/// a runnable task,
/// a task waiting for the disk.
#[test]
fn every_sign_of_life_restarts_the_window() {
    let moved: [(&str, Sample); 5] = [
        ("processor time", Sample { cpu_ticks: 8, ..still() }),
        ("bytes moved", Sample { activity: 101, ..still() }),
        ("the work of waited-for children", Sample { child_faults: 41, ..still() }),
        ("a new process", Sample { processes: vec![(10, 1), (11, 1), (12, 1)], ..still() }),
        ("a process replaced by another with its number", Sample { processes: vec![(10, 1), (11, 2)], ..still() }),
    ];
    let late: Duration = NO_PROGRESS_WINDOW - Duration::from_secs(1);
    for (what, sign) in moved {
        let mut judge: Judge = judge();
        assert_eq!(judge.observe(Duration::ZERO, &still(), false), None);
        assert_eq!(judge.observe(late, &sign, false), None, "{what}");
        // The window counts from the sign: one window after the start is not yet one window after it.
        assert_eq!(judge.observe(NO_PROGRESS_WINDOW, &sign, false), None, "{what}: the window restarted");
        assert!(matches!(judge.observe(late + NO_PROGRESS_WINDOW, &sign, false), Some(Reason::NoProgress(_))), "{what}: and ends a window after the sign");
    }
    for (what, state) in [
        ("a runnable task", Sample { runnable: true, ..still() }),
        ("a task waiting for the disk", Sample { disk_wait: true, ..still() }),
        ("a task held from outside", Sample { held: true, ..still() }),
    ] {
        let mut judge: Judge = judge();
        assert_eq!(judge.observe(Duration::ZERO, &still(), false), None);
        // Far longer than the window,
        // with nothing else moving:
        // it is a slow volume,
        // a starved process or a held one,
        // not a hang.
        for second in 1..=30 {
            assert_eq!(judge.observe(Duration::from_secs(second), &state, false), None, "{what} at {second} s");
        }
        assert!(matches!(judge.observe(Duration::from_secs(30) + NO_PROGRESS_WINDOW, &still(), false), Some(Reason::NoProgress(_))), "{what}: sleeping again, the window runs out");
    }
}

/// A test that holds a command on purpose suspends the idle rule and only that:
/// the processor budget and the backstop go on.
#[test]
fn a_hold_by_the_test_suspends_the_idle_rule_and_nothing_else() {
    let mut judge: Judge = judge();
    assert_eq!(judge.observe(Duration::ZERO, &still(), true), None);
    for second in 1..=30 {
        assert_eq!(judge.observe(Duration::from_secs(second), &still(), true), None, "held at {second} s");
    }
    assert!(matches!(judge.observe(Duration::from_secs(30) + NO_PROGRESS_WINDOW, &still(), false), Some(Reason::NoProgress(_))), "released, the window counts from the last held look");
    let ticks_per_budget: u64 = CPU_BUDGET.as_millis() as u64 / TICK.as_millis() as u64;
    let mut busy: Judge = self::judge();
    assert!(matches!(busy.observe(Duration::from_secs(1), &Sample { cpu_ticks: ticks_per_budget, ..still() }, true), Some(Reason::CpuBudget(_))), "held, the budget still ends a loop");
    let mut late: Judge = self::judge();
    assert!(matches!(late.observe(SHARED.backstop, &still(), true), Some(Reason::Backstop(_))), "held, the backstop still ends anything");
}

/// The processor budget ends a tree that is busy,
/// however much else moves;
/// waited-for children moving their time to the
/// parent do not let it drop below what was seen.
#[test]
fn the_processor_budget_ends_a_busy_tree_and_never_goes_back() {
    let ticks_per_budget: u64 = CPU_BUDGET.as_millis() as u64 / TICK.as_millis() as u64;
    let mut judge: Judge = judge();
    let busy = |ticks: u64| { return Sample { cpu_ticks: ticks, activity: ticks, runnable: true, ..still() }; };
    assert_eq!(judge.observe(Duration::from_secs(1), &busy(ticks_per_budget - 1), false), None);
    assert_eq!(judge.observe(Duration::from_secs(2), &busy(ticks_per_budget / 2), false), None, "a lower figure is not a reset");
    assert!(matches!(judge.observe(Duration::from_secs(3), &busy(ticks_per_budget), false), Some(Reason::CpuBudget(used)) if used >= CPU_BUDGET));
    assert!(
        matches!(judge.observe(Duration::from_secs(4), &still(), false), Some(Reason::CpuBudget(_))),
        "a verdict stays after a lower look: the figure kept is the peak",
    );
    let mut again: Judge = self::judge();
    assert_eq!(again.observe(Duration::from_secs(1), &busy(ticks_per_budget - 1), false), None);
    assert!(again.observe(Duration::from_secs(2), &still(), false).is_none(), "the peak is kept, the budget is not reached by a lower look");
    assert!(matches!(again.observe(Duration::from_secs(3), &busy(ticks_per_budget), false), Some(Reason::CpuBudget(_))));
}

/// The processor budget is not a function of wall time:
/// a tree that uses the whole budget within one window of wall time is
/// ended,
/// which the old five-second wall bound would have let finish.
#[test]
fn the_processor_budget_ends_a_parallel_tree_inside_the_old_wall_bound() {
    let ticks_per_budget: u64 = CPU_BUDGET.as_millis() as u64 / TICK.as_millis() as u64;
    let mut judge: Judge = judge();
    let wall: Duration = NO_PROGRESS_WINDOW / 2;
    assert!(matches!(judge.observe(wall, &Sample { cpu_ticks: ticks_per_budget, runnable: true, ..still() }, false), Some(Reason::CpuBudget(_))));
}

/// The wall-clock backstop ends anything,
/// a moving tree included,
/// and the shared limits agree with the text `timeout` gets.
#[test]
fn the_backstop_ends_a_tree_that_keeps_moving_and_agrees_with_the_timeout_argument() {
    let mut judge: Judge = judge();
    let moving = |ticks: u64| { return Sample { cpu_ticks: ticks, activity: ticks, ..still() }; };
    assert_eq!(judge.observe(SHARED.backstop - Duration::from_secs(1), &moving(1), false), None);
    assert!(matches!(judge.observe(SHARED.backstop, &moving(2), false), Some(Reason::Backstop(elapsed)) if elapsed == SHARED.backstop));
    assert_eq!(SHARED.backstop.as_secs().to_string(), SHARED_BACKSTOP_SECONDS);
}

/// The stat line is read from its last closing parenthesis,
/// so a command name with spaces and parentheses does not move
/// the fields;
/// only the bytes counters of `io` are added,
/// and the counts of calls are no sign of life.
#[test]
fn the_readers_take_the_fields_whatever_the_command_is_called() {
    let line: &str = "123 (a b) (c) D 77 2 3 4 5 6 7 8 9 10 11 12 13 14 15 20 0 0 987654 0 0";
    let stat = parse_stat(line).expect("a stat line");
    assert_eq!((stat.state, stat.parent, stat.cpu_ticks, stat.child_faults, stat.started), ('D', 77, 11 + 12 + 13 + 14, 8 + 10, 987_654));
    assert_eq!(parse_stat("not a stat line"), None);
    let io: &str = "rchar: 1\nwchar: 2\nsyscr: 3000\nsyscw: 4000\nread_bytes: 5\nwrite_bytes: 6\ncancelled_write_bytes: 700\n";
    assert_eq!(parse_io(io), 1 + 2 + 5 + 6, "calls are not counted");
    assert_eq!(parse_io(""), 0);
}

/// Only `timeout --signal=KILL <the shared backstop> <program>` is the shared bound;
/// a commit's longer bound and every other
/// program are not,
/// and a command built by the harness agrees with the reader.
#[test]
fn only_the_shared_backstop_makes_a_command_shared() {
    let shared: [&str; 4] = ["/usr/bin/timeout", "--signal=KILL", SHARED_BACKSTOP_SECONDS, "/x/git"];
    assert!(is_shared_command_line(&shared));
    assert!(is_shared_command_line(&["timeout", "--signal=KILL", SHARED_BACKSTOP_SECONDS, "/x/git", "status"]));
    assert!(!is_shared_command_line(&["/usr/bin/timeout", "--signal=KILL", "120", "/x/git"]), "a commit's bound");
    assert!(!is_shared_command_line(&["/usr/bin/timeout", "--signal=TERM", SHARED_BACKSTOP_SECONDS, "/x/git"]));
    assert!(!is_shared_command_line(&["/usr/bin/sleep", "--signal=KILL", SHARED_BACKSTOP_SECONDS, "/x/git"]));
    assert!(!is_shared_command_line(&["/usr/bin/timeout", "--signal=KILL", SHARED_BACKSTOP_SECONDS]), "no program");
    let made: Fixture = fixture("supervisor-shared");
    let path = wrapper_first_path(&made);
    assert!(is_shared(&bounded(&made, made.root.join("bin/git").as_path(), path.as_os_str())));
    assert!(!is_shared(&bounded_for(&made, made.root.join("bin/git").as_path(), path.as_os_str(), "120")));
    assert!(!is_shared(&Command::new("/bin/true")));
    remove(&made);
}

/// A shell script at `name` in the fixture,
/// run as a child in its own group the way `timeout` leads one.
fn start(made: &Fixture, script: &str) -> Child {
    let path = made.root.join("script.sh");
    executable(path.as_path(), format!("#!/bin/sh\n{script}\n").as_bytes());
    let mut command: Command = Command::new(path);
    command.current_dir(&made.root).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    let child: Child = command.spawn().expect("the script starts");
    // Owned, so a script that loops or sleeps does not outlive a control that failed before it ended it.
    own_group(&child);
    return child;
}

/// The process ID of a child as the readers take it.
fn id_of(child: &Child) -> i32 {
    return i32::try_from(child.id()).expect("a process ID fits");
}

/// Pin a child of this test and end its tree through the pin;
/// the way every control cleans up what it started.
fn end_by_pin(child: &mut Child) {
    let pid: i32 = id_of(child);
    if let Some(stat) = stat_of(pid)
        && let Some(pin) = Pin::open(pid, stat.started)
    {
        end_tree(&pin);
    }
    child.wait().expect("reaped");
}

/// Wait for a child to end by itself or by the monitor,
/// for at most `patience`;
/// the figure only keeps a broken monitor
/// from holding a control,
/// and never decides a verdict.
fn finish(child: &mut Child, patience: Duration) -> std::process::ExitStatus {
    let began: Instant = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("the child's state") {
            return status;
        }
        if began.elapsed() >= patience {
            end_by_pin(child);
            panic!("the monitor never ended the command within {patience:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Wait until `ready` holds,
/// for at most `patience`,
/// and say whether it did;
/// the figure keeps a broken monitor from holding a
/// control and never decides a verdict.
fn until(patience: Duration, ready: impl Fn() -> bool) -> bool {
    let began: Instant = Instant::now();
    while !ready() {
        if began.elapsed() >= patience {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    return true;
}

/// How long a control lets the monitor take,
/// which only a monitor that never acts reaches.
const MONITOR_PATIENCE: Duration = Duration::from_secs(120);

/// The limits the controls use:
/// the window short,
/// the budget and the backstop as the verdict needs.
fn short(budget: Duration) -> Limits {
    return Limits { no_progress: Duration::from_millis(600), cpu_budget: budget, backstop: Duration::from_secs(60) };
}

/// How many looks of the monitor it takes to be sure the short window has passed several times over.
fn looks_for_windows(windows: u32) -> u32 {
    return u32::try_from((short(CPU_BUDGET).no_progress * windows).as_millis() / POLL.as_millis()).expect("fits") + 1;
}

/// Two processes that each wait to open the other's pipe for writing never get past it:
/// the tree is asleep and nothing
/// moves,
/// so the monitor ends it and says so,
/// naming the command and the tasks.
#[test]
fn a_deadlock_is_ended_and_reported_with_its_tasks() {
    let made: Fixture = fixture("supervisor-deadlock");
    let script: &str = "d=\"$PWD\"; mkfifo \"$d/a\" \"$d/b\"; (cat \"$d/a\" > \"$d/b\") & cat \"$d/b\" > \"$d/a\"; wait";
    let mut child: Child = start(&made, script);
    watch(child.id(), short(Duration::from_secs(60)));
    let status = finish(&mut child, MONITOR_PATIENCE);
    assert_eq!(std::os::unix::process::ExitStatusExt::signal(&status), Some(9), "the monitor ended it");
    let message: String = take_kill(child.id()).expect("the kill is reported");
    assert!(message.contains("no progress") && message.contains("script.sh"), "{message}");
    assert!(message.contains("state S") && message.contains("pid "), "the tasks of the tree are listed: {message}");
    remove(&made);
}

/// A loop that never sleeps and does no input or output is ended by the processor budget,
/// with the budget in the message.
/// The loop runs inside the test's own capped scope or container,
/// one processor,
/// and the monitor ends it at the budget.
#[test]
fn a_loop_is_ended_by_the_processor_budget_and_reported() {
    let made: Fixture = fixture("supervisor-loop");
    let mut child: Child = start(&made, "while :; do :; done");
    watch(child.id(), short(Duration::from_secs(1)));
    let status = finish(&mut child, MONITOR_PATIENCE);
    assert_eq!(std::os::unix::process::ExitStatusExt::signal(&status), Some(9), "the monitor ended it");
    let message: String = take_kill(child.id()).expect("the kill is reported");
    assert!(message.contains("processor time") && message.contains("a loop"), "{message}");
    remove(&made);
}

/// A command that keeps moving (it writes and starts a process again and again,
/// for several windows) is never ended,
/// and its
/// kill record stays empty.
/// The window is the monitor's own wall-clock rule and cannot be taken out of this control,
/// so the control keeps the command
/// from ever going quiet for a window instead:
/// it makes a sign of life every few milliseconds,
/// and the number of rounds is
/// derived from the window and the pause,
/// each round lasting at least its pause,
/// so that several windows have passed by the
/// time the command ends without any clock in the control.
/// Only a command that shows nothing at all for a whole window,
/// which
/// is what the monitor exists to end,
/// can fail it.
#[test]
fn a_slow_command_that_keeps_moving_is_not_ended() {
    let made: Fixture = fixture("supervisor-crawl");
    let window: Duration = short(Duration::from_secs(60)).no_progress;
    let pause: Duration = Duration::from_millis(10);
    let windows_crossed: u128 = 3;
    let rounds: u64 = u64::try_from(window.as_millis() * windows_crossed / pause.as_millis()).expect("a small count");
    let script: String = format!("i=0; while [ $i -lt {rounds} ]; do echo x >> \"$PWD/log\"; sleep {:.2}; i=$((i+1)); done", pause.as_secs_f64());
    let mut child: Child = start(&made, script.as_str());
    watch(child.id(), short(Duration::from_secs(60)));
    let status = finish(&mut child, MONITOR_PATIENCE);
    assert!(status.success(), "{status:?}");
    assert_eq!(take_kill(child.id()), None);
    assert_eq!(std::fs::read_to_string(made.root.join("log")).expect("log").lines().count(), usize::try_from(rounds).expect("fits"));
    remove(&made);
}

/// A process that waits for the disk longer than the window is not ended:
/// a big write followed by its `fsync`.
/// Where the
/// volume is fast the wait is shorter than the window and the control says that it did not exercise the rule.
#[test]
fn a_wait_for_the_disk_longer_than_the_window_is_not_a_hang() {
    let made: Fixture = fixture("supervisor-disk");
    let limits: Limits = Limits { no_progress: Duration::from_millis(300), cpu_budget: Duration::from_secs(120), backstop: Duration::from_secs(300) };
    let mut child: Child = start(&made, "exec dd if=/dev/urandom of=\"$PWD/big\" bs=1M count=300 conv=fsync");
    watch(child.id(), limits);
    let status = finish(&mut child, MONITOR_PATIENCE);
    assert!(status.success(), "{status:?}");
    assert_eq!(take_kill(child.id()), None, "a command waiting for the disk was ended");
    let looks: u32 = disk_wait_looks(child.id());
    if Duration::from_millis(100) * looks < limits.no_progress * 2 {
        eprintln!("DISK-WAIT CONTROL NOT EXERCISED: the volume under {} took the fsync in {looks} looks, less than the window", made.root.display());
    }
    remove(&made);
}

/// What:
///  A command waiting in a Unix socket's read is not this tree's hang,
///  and its answer ends it.
/// Why:
///   The wrapper waits this way for the commit service's answer,
/// and that answer's latency is the service's own disk work,
///  which no look of this tree sees;
/// on 2026-11-11 six `word_merge` controls failed on the idle rule while their
///  answers were coming
///  (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`).
#[test]
fn a_wait_for_an_answer_over_a_socket_is_not_a_hang() {
    let made: Fixture = fixture("supervisor-answer-wait");
    let (answered, waiting) = std::os::unix::net::UnixStream::pair().expect("a socket pair");
    let child_end: std::os::fd::OwnedFd = waiting.into();
    let raw: std::os::fd::RawFd = std::os::fd::AsRawFd::as_raw_fd(&child_end);
    let mut command: Command = Command::new("/bin/sh");
    command.arg("-c").arg("read line <&3; printf '%s' \"$line\" > \"$PWD/said\"").current_dir(&made.root).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    // SAFETY: `dup2` only copies a descriptor of this process into the child's table, which is all a `pre_exec` closure
    //  may do; the descriptor stays owned here until this control ends.
    unsafe {
        std::os::unix::process::CommandExt::pre_exec(&mut command, move || {
            return if libc::dup2(raw, 3) == -1 { Err(std::io::Error::last_os_error()) } else { Ok(()) };
        });
    }
    let mut child: Child = command.spawn().expect("the script starts");
    // Owned, so a script that waits here does not outlive a control that failed before it ended it.
    own_group(&child);
    // The window is short and the whole wait is several of them, so a tree asleep in the socket read is what is judged.
    watch(child.id(), short(Duration::from_secs(120)));
    let pid: i32 = id_of(&child);
    assert!(until(MONITOR_PATIENCE, || { return waits_for_an_answer(pid); }), "the command reached the socket's read");
    std::thread::sleep(short(CPU_BUDGET).no_progress * 4);
    assert!(!kill_recorded(pid), "a command waiting for an answer over a socket was ended");
    // The answer arrives, and the command finishes by itself, which is the proof the wait was the only thing holding it.
    let mut answer: std::os::unix::net::UnixStream = answered;
    std::io::Write::write_all(&mut answer, b"go\n").expect("the answer is written");
    let status: std::process::ExitStatus = finish(&mut child, MONITOR_PATIENCE);
    assert!(status.success(), "{status:?}");
    assert_eq!(take_kill(child.id()), None, "a command that was answered was ended");
    assert_eq!(std::fs::read(made.root.join("said")).expect("the answer the command read"), b"go");
    remove(&made);
}

/// A fifo at `path`.
fn make_fifo(path: &std::path::Path) {
    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    // SAFETY:
    // `mkfifo` takes a valid C string and a mode and touches no memory of this process.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0, "the fifo is made");
}

/// A command that waits on something the test provides later is not judged idle while the test holds it,
/// and is when it does
/// not:
/// the same wait,
/// once with a hold and once without.
#[test]
fn a_wait_on_the_test_is_judged_idle_only_when_the_test_does_not_hold_it() {
    let made: Fixture = fixture("supervisor-hold");
    make_fifo(made.root.join("gate").as_path());
    let mut held: Child = start(&made, "read line < \"$PWD/gate\"; echo \"$line\" > \"$PWD/said\"");
    watch(held.id(), short(Duration::from_secs(60)));
    let hold = hold_idle_judgment(held.id());
    let pid: u32 = held.id();
    assert!(until(MONITOR_PATIENCE, || { return looks_of(pid) >= looks_for_windows(4); }), "the monitor looked at the held command");
    assert!(held.try_wait().expect("state").is_none() && take_kill(pid).is_none(), "the held command was ended");
    // The test provides what the command waits for,
    // and lets go of it.
    std::fs::write(made.root.join("gate"), b"go\n").expect("the gate opens");
    let status = finish(&mut held, MONITOR_PATIENCE);
    drop(hold);
    assert!(status.success(), "{status:?}");
    assert_eq!(std::fs::read_to_string(made.root.join("said")).expect("said").trim(), "go");
    let _removed = std::fs::remove_file(made.root.join("gate"));
    make_fifo(made.root.join("gate").as_path());
    let mut unheld: Child = start(&made, "read line < \"$PWD/gate\"");
    watch(unheld.id(), short(Duration::from_secs(60)));
    let unheld_status = finish(&mut unheld, MONITOR_PATIENCE);
    assert_eq!(std::os::unix::process::ExitStatusExt::signal(&unheld_status), Some(9), "without a hold the same wait is a hang");
    assert!(take_kill(unheld.id()).expect("reported").contains("no progress"));
    remove(&made);
}

/// A tree stopped from outside (a signal a test sent,
/// or a tracer) is held,
/// not idle:
/// it is not ended however long it stays
/// stopped,
/// and it is ended when it is continued and then hangs.
#[test]
fn a_tree_stopped_from_outside_is_not_judged_idle() {
    let made: Fixture = fixture("supervisor-stopped");
    let mut child: Child = start(&made, "sleep 600 & wait");
    let pid: i32 = id_of(&child);
    watch(child.id(), short(Duration::from_secs(60)));
    assert!(until(MONITOR_PATIENCE, || { return children(pid).len() == 1; }), "the script started its sleep");
    // SAFETY:
    // `kill` takes two integers and touches no memory of this process;
    // the process is this test's own child.
    unsafe { libc::kill(pid, libc::SIGSTOP) };
    assert!(until(MONITOR_PATIENCE, || { return stat_of(pid).is_some_and(|stat| { return stat.state == 'T'; }); }), "the script stopped");
    let before: u32 = looks_of(child.id());
    assert!(until(MONITOR_PATIENCE, || { return looks_of(child.id()) >= before + looks_for_windows(4); }), "the monitor looked at the stopped tree");
    assert!(child.try_wait().expect("state").is_none() && take_kill(child.id()).is_none() && is_watching(child.id()), "the stopped tree was ended");
    end_by_pin(&mut child);
    remove(&made);
}

/// The state of every task is read,
/// not the leader's alone:
/// this test process has more than one task,
/// and a worker that waits
/// for the disk or is runnable counts though the leader sleeps.
#[test]
fn the_states_of_every_task_are_read() {
    let own: i32 = i32::try_from(std::process::id()).expect("a process ID fits");
    assert!(task_states(own).len() >= 2, "{:?}", task_states(own));
    for (what, states, runnable, disk_wait, held) in [
        ("a worker waiting for the disk", ['S', 'D'], false, true, false),
        ("a runnable worker", ['S', 'R'], true, false, false),
        ("a stopped worker", ['S', 'T'], false, false, true),
        ("a traced worker", ['S', 't'], false, false, true),
        ("all asleep", ['S', 'S'], false, false, false),
    ] {
        let mut sample: Sample = Sample::default();
        note_states(&mut sample, &states);
        assert_eq!((sample.runnable, sample.disk_wait, sample.held), (runnable, disk_wait, held), "{what}");
    }
}

/// A command that has exited and waits to be reaped is finished:
/// the monitor drops it and records no kill,
/// however long the
/// test takes before it waits.
#[test]
fn a_command_that_has_exited_is_finished_and_not_judged_idle() {
    let made: Fixture = fixture("supervisor-zombie");
    let mut child: Child = start(&made, "exit 0");
    let pid: i32 = id_of(&child);
    assert!(until(MONITOR_PATIENCE, || { return stat_of(pid).is_some_and(|stat| { return stat.state == 'Z'; }); }), "the script exited");
    assert_eq!(look_at(pid, stat_of(pid).expect("the zombie").started), Look::Exited);
    watch(child.id(), short(Duration::from_secs(60)));
    assert!(until(MONITOR_PATIENCE, || { return !is_watching(child.id()); }), "the monitor dropped the finished command");
    assert_eq!(take_kill(child.id()), None, "a command that exited successfully was ended");
    assert!(child.wait().expect("reaped").success());
    remove(&made);
}

/// The work of children that came and went between two looks is seen in the counters of the process that waited for them.
#[test]
fn the_work_of_children_that_came_and_went_is_seen() {
    let made: Fixture = fixture("supervisor-children");
    let mut child: Child = start(&made, "i=0; while [ $i -lt 20 ]; do /bin/true; i=$((i+1)); done; sleep 600 & wait");
    let pid: i32 = id_of(&child);
    let started: u64 = stat_of(pid).expect("running").started;
    let saw = until(MONITOR_PATIENCE, || { return matches!(look_at(pid, started), Look::Seen(sample) if sample.child_faults > 0 && sample.processes.len() >= 2); });
    assert!(saw, "the look showed no work of the finished children");
    end_by_pin(&mut child);
    remove(&made);
}

/// The tree is everything below the command,
/// grandchildren included,
/// each with its start time;
/// its state is read from the
/// tasks,
/// and the description lists them.
#[test]
fn the_tree_holds_the_grandchildren_and_describes_each_task() {
    let made: Fixture = fixture("supervisor-tree");
    let mut child: Child = start(&made, "sleep 600 & wait");
    let root: i32 = id_of(&child);
    assert!(until(MONITOR_PATIENCE, || { return tree_of(root).len() >= 2; }), "the script started its sleep");
    let tree = tree_of(root);
    assert_eq!(tree[0].pid, root);
    assert_eq!(tree[0].started, stat_of(root).expect("running").started);
    assert!(children(root).contains(&tree[1].pid), "{tree:?}");
    assert!(until(MONITOR_PATIENCE, || { return matches!(look_at(root, tree[0].started), Look::Seen(sample) if !sample.runnable); }));
    let Look::Seen(sleeping) = look_at(root, tree[0].started) else { panic!("the tree is alive") };
    assert!(!sleeping.disk_wait && sleeping.processes.len() >= 2, "{sleeping:?}");
    let text: String = describe(root);
    assert!(text.lines().count() >= 2 && text.contains(&format!("pid {root} ")) && text.contains("sleep 600"), "{text}");
    assert!(tick() > Duration::ZERO);
    end_by_pin(&mut child);
    remove(&made);
}

/// A pin names one process:
/// it is refused for another start time,
/// and after the process is gone a signal through it reaches
/// nothing.
#[test]
fn a_pin_names_one_process_and_signals_nothing_else() {
    let made: Fixture = fixture("supervisor-pin");
    let mut child: Child = start(&made, "sleep 600 & wait");
    let pid: i32 = id_of(&child);
    let started: u64 = stat_of(pid).expect("running").started;
    assert!(Pin::open(pid, started + 1).is_none(), "another start time is another process");
    assert!(Pin::open(i32::MAX - 1, started).is_none(), "no such process");
    let pin: Pin = Pin::open(pid, started).expect("pinned");
    assert!(!pin.is_gone());
    end_tree(&pin);
    child.wait().expect("reaped");
    assert!(pin.is_gone(), "the pinned process is gone");
    assert_eq!(pin.signal(libc::SIGKILL), Delivered::Gone, "a signal through the pin of a gone process reaches nothing");
    assert!(Pin::open(pid, started).is_none(), "the number names no process that started then");
    remove(&made);
}

/// Ending a tree ends every member,
/// grandchildren of grandchildren included,
/// and the members are found by pins:
/// the processes
/// the first look did not hold die too.
#[test]
fn ending_a_tree_ends_every_member_at_every_depth() {
    let made: Fixture = fixture("supervisor-depth");
    let mut child: Child = start(&made, "(sleep 600 & wait) & wait");
    let root: i32 = id_of(&child);
    assert!(until(MONITOR_PATIENCE, || { return tree_of(root).len() >= 3; }), "the tree has three levels");
    let members: Vec<i32> = tree_of(root).into_iter().map(|member| { return member.pid; }).collect();
    end_by_pin(&mut child);
    assert!(
        until(MONITOR_PATIENCE, || { return members.iter().all(|member| { return stat_of(*member).is_none_or(|stat| { return stat.state == 'Z'; }); }); }),
        "a member of the tree outlived it: {members:?}"
    );
    remove(&made);
}

/// The record of a kill is there before the first signal goes out:
/// a test that waits for the command sees it end after the
/// signal,
/// so it finds the record.
#[test]
fn the_record_of_a_kill_is_published_before_the_first_signal() {
    let made: Fixture = fixture("supervisor-record-first");
    let mut child: Child = start(&made, "sleep 600 & wait");
    let pid: i32 = id_of(&child);
    let pin: Pin = Pin::open(pid, stat_of(pid).expect("running").started).expect("pinned");
    end_command(&pin, "the record", &|pinned| {
        assert!(kill_recorded(pid), "no record when the signals went out");
        end_tree(pinned);
    });
    child.wait().expect("reaped");
    assert_eq!(take_kill(child.id()), Some(String::from("the record")));
    remove(&made);
}

/// A kill is claimed right after the wait,
/// before the cleanup that can take longer than the grace in which an unclaimed one
/// aborts the process.
#[test]
fn a_kill_is_claimed_before_the_cleanup_runs() {
    let mut command: Command = Command::new("/bin/cat");
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let child: Child = command.spawn().expect("cat starts");
    plant_kill(i32::try_from(child.id()).expect("fits"), "a planted kill", Instant::now());
    let cleaned: std::cell::Cell<bool> = std::cell::Cell::new(false);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { return supervised_wait(child, b"", || { cleaned.set(true); }); }));
    assert!(outcome.is_err(), "the kill is reported");
    assert!(!cleaned.get(), "the cleanup ran before the kill was claimed");
}

/// A command that writes much while its input is still being sent finishes:
/// the output is drained from the spawn and the input
/// goes in on a thread of its own.
#[test]
fn a_command_that_writes_more_than_a_pipe_holds_while_its_input_is_sent_finishes() {
    let mut command: Command = Command::new("/bin/cat");
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let child: Child = command.spawn().expect("cat starts");
    let input: Vec<u8> = (0..4_000_000_u32).map(|index| { return u8::try_from(index % 251).expect("below 256"); }).collect();
    let output = supervised_wait(child, input.as_slice(), || {});
    assert_eq!(output.stdout, input);
}

/// The monitor sees a command that is something else at first and a shared bound after its `exec`:
/// nothing is remembered
/// against a process that has not yet become the command.
#[test]
fn a_command_is_found_after_it_execs_into_the_shared_bound() {
    let made: Fixture = fixture("supervisor-exec");
    make_fifo(made.root.join("gate").as_path());
    super::supervisor::ensure_monitor();
    let script: String = format!("read line < \"$PWD/gate\"; exec /usr/bin/timeout --signal=KILL {SHARED_BACKSTOP_SECONDS} /bin/sleep 1000");
    let mut child: Child = start(&made, script.as_str());
    // The monitor looks at the script a few times while it is still only a shell.
    std::thread::sleep(POLL * 4);
    assert!(!is_watching(child.id()), "a shell is not the shared bound");
    std::fs::write(made.root.join("gate"), b"go\n").expect("the gate opens");
    assert!(until(Duration::from_secs(30), || { return is_watching(child.id()); }), "the command was never found after its exec");
    end_by_pin(&mut child);
    remove(&made);
}

/// A command registered with limits is watched from the moment `watch` returns,
/// and stays watched through the looks that follow.
#[test]
fn a_registered_command_stays_watched() {
    let made: Fixture = fixture("supervisor-registered");
    let mut child: Child = start(&made, "sleep 600 & wait");
    watch(child.id(), Limits { no_progress: Duration::from_secs(600), cpu_budget: Duration::from_secs(600), backstop: Duration::from_secs(600) });
    assert!(is_watching(child.id()), "watched at once");
    assert!(until(MONITOR_PATIENCE, || { return looks_of(child.id()) >= 5; }), "the monitor looked");
    assert!(is_watching(child.id()), "and still watched");
    end_by_pin(&mut child);
    assert!(until(MONITOR_PATIENCE, || { return !is_watching(child.id()); }), "dropped once it ended");
    remove(&made);
}

/// A kill nobody has claimed is found once the grace has passed,
/// and not before;
/// a claimed one never is.
#[test]
fn a_kill_nobody_claimed_is_found_after_the_grace_only() {
    let at: Instant = Instant::now();
    plant_kill(i32::MAX - 7, "the planted kill", at);
    assert_eq!(unclaimed_kill(at, UNCLAIMED_GRACE), None, "not yet");
    assert_eq!(unclaimed_kill(at + UNCLAIMED_GRACE, UNCLAIMED_GRACE), Some(String::from("the planted kill")));
    let claimed: u32 = u32::try_from(i32::MAX - 7).expect("positive");
    assert_eq!(take_kill(claimed), Some(String::from("the planted kill")));
    assert_eq!(unclaimed_kill(at + UNCLAIMED_GRACE * 2, UNCLAIMED_GRACE), None, "a claimed kill is not reported again");
}

/// How much longer than the grace the ending of a command takes in the control below,
/// as on a starved host.
const SLOW_ENDING_EXTRA: Duration = Duration::from_millis(500);

/// The time a test has to claim a kill counts from the moment its command is gone,
/// not from the moment the supervisor began
/// to end it:
/// a tree that is slow to end on a starved host must not leave the test less than the whole grace.
#[test]
fn the_grace_to_claim_a_kill_starts_when_the_ending_is_over() {
    let mut command: Command = Command::new("/bin/sleep");
    command.arg("60");
    let mut child: Child = spawn_owned(&mut command).expect("sleep starts");
    let pid: i32 = i32::try_from(child.id()).expect("fits");
    let started: u64 = stat_of(pid).expect("the child runs").started;
    let pin: Pin = Pin::open(pid, started).expect("the child can be pinned");
    end_command(&pin, "a slow ending", &|named| {
        std::thread::sleep(UNCLAIMED_GRACE + SLOW_ENDING_EXTRA);
        end_tree(named);
    });
    assert_eq!(unclaimed_kill(Instant::now(), UNCLAIMED_GRACE), None, "the ending took longer than the grace and the test had none left");
    assert!(take_kill(child.id()).is_some(), "the kill is still there to claim");
    child.wait().expect("reaped");
}

/// The variable that makes the probe test below plant a kill nobody claims.
const PROBE_VARIABLE: &str = "SUPERVISOR_UNCLAIMED_EXIT_PROBE";

/// A test that ends a shared command by the supervisor and passes without claiming the kill still fails:
/// the exit handler ends
/// the process with a failing status before the grace could pass.
/// This is the probe a control below runs in a process of its
/// own;
/// run on its own it does nothing.
#[test]
fn unclaimed_exit_probe() {
    if std::env::var_os(PROBE_VARIABLE).is_none() {
        return;
    }
    super::supervisor::ensure_monitor();
    plant_kill(i32::MAX - 9, "the kill of the probe", Instant::now());
}

/// The process of the probe exits with the failing status and names the kill,
/// though the test it ran returned normally.
#[test]
fn an_unclaimed_kill_fails_the_process_at_its_exit() {
    // Identification only: the first argument names the test executable this process was started from, to run one test of it again.
    let exe = std::env::args_os().next().expect("the test executable");
    let output = Command::new(exe)
        .args(["--exact", "supervisor_guard::unclaimed_exit_probe", "--nocapture", "--test-threads=1"])
        .env(PROBE_VARIABLE, "1")
        .output()
        .expect("the probe starts");
    let text: String = String::from_utf8_lossy(&output.stderr).into_owned() + &String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("test supervisor_guard::unclaimed_exit_probe ... ok"), "the probe test itself passed: {text}");
    assert_eq!(output.status.code(), Some(UNCLAIMED_EXIT_STATUS), "{text}");
    assert!(text.contains("the kill of the probe"), "{text}");
}

/// Through `observe`,
/// a command of the shared bound whose tree is hung fails the test with the supervisor's message,
/// not
/// with a bare exit status;
/// it takes the shared window of five seconds,
/// so there is one such control.
#[test]
#[should_panic(expected = "the harness supervisor ended the command")]
fn a_hung_shared_command_fails_its_test_with_the_reason_and_the_command() {
    let made: Fixture = fixture("supervisor-observe");
    let script = made.root.join("hang.sh");
    executable(script.as_path(), b"#!/bin/sh\nd=\"$PWD\"; mkfifo \"$d/a\" \"$d/b\"; (cat \"$d/a\" > \"$d/b\") & cat \"$d/b\" > \"$d/a\"; wait\n");
    let mut command: Command = bounded(&made, script.as_path(), wrapper_first_path(&made).as_os_str());
    command.current_dir(&made.root);
    let _ = super::support::observe(&mut command, b"");
}
