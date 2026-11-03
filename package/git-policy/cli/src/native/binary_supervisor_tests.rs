//! Controls for the hang guard of the binary tests: the judge on built samples, the readers of `/proc`, and real commands
//!  that loop, deadlock, wait for a disk and crawl, run under the monitor with limits short enough to see the verdict.
//!  A loop and a deadlock must be ended and reported with the command, the reason and the tasks of the tree; a command
//!  that is slow but moving, or waits for the disk, must not be.

use super::support::{Fixture, bounded, bounded_for, executable, fixture, remove, wrapper_first_path};
use super::supervisor::{
    CPU_BUDGET, Judge, Limits, NO_PROGRESS_WINDOW, Reason, SHARED, SHARED_BACKSTOP_SECONDS, Sample, UNCLAIMED_GRACE, children, describe,
    disk_wait_looks, end_tree, is_shared, is_shared_command_line, parse_io, parse_stat, plant_kill, sample_tree, take_kill, tick, tree_of,
    unclaimed_kill, watch,
};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The length of a clock tick the built samples assume.
const TICK: Duration = Duration::from_millis(10);

/// The limits of the shared bound with the tick of the built samples.
fn judge() -> Judge {
    return Judge::new(SHARED, TICK);
}

/// A look at a tree that shows the same thing every time: `processes` processes, nothing runnable.
fn still() -> Sample {
    return Sample { cpu_ticks: 7, activity: 100, processes: vec![10, 11], runnable: false, disk_wait: false };
}

/// A tree that shows nothing for the window is hung, and not one look earlier.
#[test]
fn a_tree_that_stands_still_is_hung_when_the_window_has_passed() {
    let mut judge: Judge = judge();
    let step: Duration = Duration::from_millis(500);
    let mut elapsed: Duration = Duration::ZERO;
    assert_eq!(judge.observe(elapsed, &still()), None, "the first look is a first look");
    let verdict: Reason = loop {
        elapsed += step;
        if let Some(reason) = judge.observe(elapsed, &still()) {
            break reason;
        }
        assert!(elapsed < NO_PROGRESS_WINDOW, "the window passed and the tree was not judged hung");
    };
    assert_eq!(verdict, Reason::NoProgress(NO_PROGRESS_WINDOW), "judged exactly when the window passed");
}

/// Each sign of life restarts the window: processor time, a read or write, a new process, a runnable task, a task waiting
/// for the disk.
#[test]
fn every_sign_of_life_restarts_the_window() {
    let moved: [(&str, Sample); 3] = [
        ("processor time", Sample { cpu_ticks: 8, ..still() }),
        ("read and write counters", Sample { activity: 101, ..still() }),
        ("a new process", Sample { processes: vec![10, 11, 12], ..still() }),
    ];
    let late: Duration = NO_PROGRESS_WINDOW - Duration::from_secs(1);
    for (what, sign) in moved {
        let mut judge: Judge = judge();
        assert_eq!(judge.observe(Duration::ZERO, &still()), None);
        assert_eq!(judge.observe(late, &sign), None, "{what}");
        // The window counts from the sign: one window after the start is not yet one window after it.
        assert_eq!(judge.observe(NO_PROGRESS_WINDOW, &sign), None, "{what}: the window restarted");
        assert!(matches!(judge.observe(late + NO_PROGRESS_WINDOW, &sign), Some(Reason::NoProgress(_))), "{what}: and ends a window after the sign");
    }
    for (what, state) in [("a runnable task", Sample { runnable: true, ..still() }), ("a task waiting for the disk", Sample { disk_wait: true, ..still() })] {
        let mut judge: Judge = judge();
        assert_eq!(judge.observe(Duration::ZERO, &still()), None);
        // Far longer than the window, with nothing else moving: it is a slow volume or a starved process, not a hang.
        for second in 1..=30 {
            assert_eq!(judge.observe(Duration::from_secs(second), &state), None, "{what} at {second} s");
        }
        assert!(matches!(judge.observe(Duration::from_secs(30) + NO_PROGRESS_WINDOW, &still()), Some(Reason::NoProgress(_))), "{what}: sleeping again, the window runs out");
    }
}

/// The processor budget ends a tree that is busy, however much else moves; waited-for children moving their time to the
/// parent do not let it drop below what was seen.
#[test]
fn the_processor_budget_ends_a_busy_tree_and_never_goes_back() {
    let ticks_per_budget: u64 = CPU_BUDGET.as_millis() as u64 / TICK.as_millis() as u64;
    let mut judge: Judge = judge();
    let busy = |ticks: u64| { return Sample { cpu_ticks: ticks, activity: ticks, runnable: true, ..still() }; };
    assert_eq!(judge.observe(Duration::from_secs(1), &busy(ticks_per_budget - 1)), None);
    assert_eq!(judge.observe(Duration::from_secs(2), &busy(ticks_per_budget / 2)), None, "a lower figure is not a reset");
    assert!(matches!(judge.observe(Duration::from_secs(3), &busy(ticks_per_budget)), Some(Reason::CpuBudget(used)) if used >= CPU_BUDGET));
    assert!(
        matches!(judge.observe(Duration::from_secs(4), &still()), Some(Reason::CpuBudget(_))),
        "a verdict stays after a lower look: the figure kept is the peak",
    );
    let mut again: Judge = self::judge();
    assert_eq!(again.observe(Duration::from_secs(1), &busy(ticks_per_budget - 1)), None);
    assert!(again.observe(Duration::from_secs(2), &still()).is_none(), "the peak is kept, the budget is not reached by a lower look");
    assert!(matches!(again.observe(Duration::from_secs(3), &busy(ticks_per_budget)), Some(Reason::CpuBudget(_))));
}

/// The wall-clock backstop ends anything, a moving tree included, and the shared limits agree with the text `timeout` gets.
#[test]
fn the_backstop_ends_a_tree_that_keeps_moving_and_agrees_with_the_timeout_argument() {
    let mut judge: Judge = judge();
    let moving = |ticks: u64| { return Sample { cpu_ticks: ticks, activity: ticks, ..still() }; };
    assert_eq!(judge.observe(SHARED.backstop - Duration::from_secs(1), &moving(1)), None);
    assert!(matches!(judge.observe(SHARED.backstop, &moving(2)), Some(Reason::Backstop(elapsed)) if elapsed == SHARED.backstop));
    assert_eq!(SHARED.backstop.as_secs().to_string(), SHARED_BACKSTOP_SECONDS);
}

/// The stat line is read from its last closing parenthesis, so a command name with spaces and parentheses does not move
/// the fields; the counters of `io` are added and the others ignored.
#[test]
fn the_readers_take_the_fields_whatever_the_command_is_called() {
    let line: &str = "123 (a b) (c) D 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 20 0 0 987654 0 0";
    let stat = parse_stat(line).expect("a stat line");
    assert_eq!((stat.state, stat.cpu_ticks, stat.started), ('D', 11 + 12 + 13 + 14, 987_654));
    assert_eq!(parse_stat("not a stat line"), None);
    let io: &str = "rchar: 1\nwchar: 2\nsyscr: 3\nsyscw: 4\nread_bytes: 5\nwrite_bytes: 6\ncancelled_write_bytes: 700\n";
    assert_eq!(parse_io(io), 1 + 2 + 3 + 4 + 5 + 6);
    assert_eq!(parse_io(""), 0);
}

/// Only `timeout --signal=KILL <the shared backstop> <program>` is the shared bound; a commit's longer bound and every other
/// program are not, and a command built by the harness agrees with the reader.
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

/// A shell script at `name` in the fixture, run as a child in its own group the way `timeout` leads one.
fn start(made: &Fixture, script: &str) -> Child {
    let path = made.root.join("script.sh");
    executable(path.as_path(), format!("#!/bin/sh\n{script}\n").as_bytes());
    let mut command: Command = Command::new(path);
    command.current_dir(&made.root).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    return command.spawn().expect("the script starts");
}

/// Wait for a child to end by itself or by the monitor, for at most `patience`; the figure only keeps a broken monitor
/// from holding a control, and never decides a verdict.
fn finish(child: &mut Child, patience: Duration) -> std::process::ExitStatus {
    let began: Instant = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("the child's state") {
            return status;
        }
        if began.elapsed() >= patience {
            // SAFETY: `kill` takes two integers and touches no memory of this process.
            unsafe { libc::kill(-i32::try_from(child.id()).expect("a process ID fits"), libc::SIGKILL) };
            panic!("the monitor never ended the command within {patience:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// How long a control lets the monitor take, which only a monitor that never acts reaches.
const MONITOR_PATIENCE: Duration = Duration::from_secs(120);

/// The limits the controls use: the window short, the budget and the backstop as the verdict needs.
fn short(budget: Duration) -> Limits {
    return Limits { no_progress: Duration::from_millis(600), cpu_budget: budget, backstop: Duration::from_secs(60) };
}

/// Two processes that each wait to open the other's pipe for writing never get past it: the tree is asleep and nothing
/// moves, so the monitor ends it and says so, naming the command and the tasks.
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

/// A loop that never sleeps and does no input or output is ended by the processor budget, with the budget in the message.
/// The loop runs inside the test's own capped scope or container, one processor, and the monitor ends it at the budget.
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

/// A command that keeps moving (it writes and starts a process again and again, for several windows) is never ended, and its
/// kill record stays empty.
/// The window is the monitor's own wall-clock rule and cannot be taken out of this control, so the control keeps the command
/// from ever going quiet for a window instead: it makes a sign of life every few milliseconds, and the number of rounds is
/// derived from the window and the pause, each round lasting at least its pause, so that several windows have passed by the
/// time the command ends without any clock in the control. Only a command that shows nothing at all for a whole window, which
/// is what the monitor exists to end, can fail it.
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

/// A process that waits for the disk longer than the window is not ended: a big write followed by its `fsync`. Where the
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

/// The tree is everything below the command, grandchildren included; its state is read from the tasks, and the description
/// lists them.
#[test]
fn the_tree_holds_the_grandchildren_and_describes_each_task() {
    let made: Fixture = fixture("supervisor-tree");
    let mut child: Child = start(&made, "sleep 600 & wait");
    let root: i32 = i32::try_from(child.id()).expect("a process ID fits");
    let began: Instant = Instant::now();
    let tree: Vec<i32> = loop {
        let tree: Vec<i32> = tree_of(root);
        if tree.len() >= 2 || began.elapsed() >= MONITOR_PATIENCE {
            break tree;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(tree[0], root);
    assert!(tree.len() >= 2 && children(root).contains(&tree[1]), "{tree:?}");
    let sleeping = loop {
        let sample = sample_tree(root).expect("the tree is alive");
        if !sample.runnable || began.elapsed() >= MONITOR_PATIENCE {
            break sample;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(!sleeping.runnable && !sleeping.disk_wait && sleeping.processes.len() >= 2, "{sleeping:?}");
    let text: String = describe(root);
    assert!(text.lines().count() >= 2 && text.contains(&format!("pid {root} ")) && text.contains("sleep 600"), "{text}");
    assert!(tick() > Duration::ZERO);
    // SAFETY: `kill` takes two integers and touches no memory of this process.
    unsafe { libc::kill(-root, libc::SIGKILL) };
    child.wait().expect("reaped");
    remove(&made);
}

/// The group of the command is ended along with the tasks found in its tree, so a process forked after the look dies too.
#[test]
fn ending_a_tree_ends_its_group_even_where_the_look_missed_a_member() {
    let made: Fixture = fixture("supervisor-group");
    let mut child: Child = start(&made, "sleep 600 & wait");
    let root: i32 = i32::try_from(child.id()).expect("a process ID fits");
    let began: Instant = Instant::now();
    let grandchild: i32 = loop {
        if let Some(found) = children(root).first().copied() {
            break found;
        }
        assert!(began.elapsed() < MONITOR_PATIENCE, "the script never started its sleep");
        std::thread::sleep(Duration::from_millis(20));
    };
    // The look names the root only: the grandchild can die by the group alone.
    end_tree(root, &[root]);
    child.wait().expect("reaped");
    let gone = loop {
        let state: Option<char> = std::fs::read_to_string(format!("/proc/{grandchild}/stat")).ok().and_then(|text| { return parse_stat(text.as_str()); }).map(|stat| { return stat.state; });
        if matches!(state, None | Some('Z')) || began.elapsed() >= MONITOR_PATIENCE {
            break state;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(matches!(gone, None | Some('Z')), "the grandchild outlived its group: {gone:?}");
    remove(&made);
}

/// A kill nobody has claimed is found once the grace has passed, and not before; a claimed one never is.
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

/// Through `observe`, a command of the shared bound whose tree is hung fails the test with the supervisor's message, not
/// with a bare exit status; it takes the shared window of five seconds, so there is one such control.
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
