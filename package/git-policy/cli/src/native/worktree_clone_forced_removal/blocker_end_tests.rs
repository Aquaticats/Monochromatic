//! Controls for the signal stage:
//!  a task that leaves on `SIGTERM` is never killed,
//!  a task that ignores it is killed after the window,
//!  a wrapper that ends during the wait sends no kill at all,
//!  a number no task has is gone,
//!  and the endability read tells a task in an uninterruptible sleep from one it may end.

use crate::worktree_clone_forced_removal::blocker_end::*;
use crate::worktree_clone_forced_removal::forced_arm_numbers::TimeoutWindow;
use std::time::Duration;

/// A short window a control runs with,
/// because the shipped constants hold no value.
fn a_short_window() -> TimeoutWindow {
    return TimeoutWindow::at(Duration::from_millis(120), Duration::from_millis(10));
}

/// The bit of a process's ignored-signal mask that `SIGTERM` (15) occupies.
const TERM_IGNORED_BIT: u64 = 1 << (15 - 1);

/// Wait until a child has installed its own dispositions,
/// bounded,
/// because a shell that has not yet run its `trap` is a shell `SIGTERM` still reaches.
/// Why:
///  A stage that signals a child in the instant after `spawn` would test the kernel's default
///  disposition rather than the one the control built,
///  and the control's own reading of `/proc/<pid>/status` is what says the trap is in place.
fn wait_until_ready(
    pid: i32,
    ignores_term: bool,
) {
    for _attempt in 0..400 {
        let text: String =
            std::fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default();
        let ready: bool = text
            .lines()
            .find(|line| return line.starts_with("SigIgn:"))
            .is_some_and(|line| {
                let mask: u64 =
                    u64::from_str_radix(line.trim_start_matches("SigIgn:").trim(), 16).unwrap_or(0);
                return (mask & TERM_IGNORED_BIT) != 0;
            });
        if ready {
            // The trap is in place, and the shell is waiting on its own child.
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!(
        "the child {pid} never installed the disposition this control needs (ignores TERM = {ignores_term})"
    );
}

/// Start one child that runs `body` under `sh -c` and answer its blocker and the child itself,
/// after the child has installed the dispositions the body sets.
///  The child leads a process group of its own,
/// which the test thread's fixtures kill when the thread ends,
///  so the `sleep` a body starts behind its shell goes with the test instead of outliving it (issue 6471).
fn start_child(body: &str) -> (Blocker, std::process::Child) {
    let mut command: std::process::Command = std::process::Command::new("sh");
    command.arg("-c").arg(body);
    // Quiet, because nothing reads this child's output: the sleep it starts gives the run's output pipe back by holding none.
    let child: std::process::Child =
        crate::test_support::spawn_owned_quiet(&mut command).expect("a child starts");
    let pid: i32 = i32::try_from(child.id()).expect("a child's number fits");
    let blocker: Blocker = Blocker::read(pid).expect("the new child has a record");
    if body.contains("trap") {
        wait_until_ready(pid, true);
    }
    return (blocker, child);
}

/// A task that leaves on `SIGTERM` is recorded as having left after the ask,
/// and no `SIGKILL` is sent.
#[test]
fn a_task_that_leaves_on_the_ask_is_not_killed() {
    let (blocker, mut child) = start_child("exec sleep 30");
    let stage: SignalStage =
        end_the_blockers(&[blocker], a_short_window(), std::thread::sleep, || {
            return true;
        });
    assert_eq!(stage.endings.len(), 1);
    assert_eq!(stage.endings[0].1, EndResult::ExitedAfterTheAsk);
    assert_eq!(stage.killed(), 0);
    assert!(!stage.caller_gone);
    let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
}

/// A task that ignores `SIGTERM` is killed after the window,
/// and the stage says so.
/// The trailing `:` keeps the shell from exec'ing the sleep,
/// which would put the ignored signal's handler out of the process the pin names.
#[test]
fn a_task_that_ignores_the_ask_is_killed() {
    let (blocker, mut child) = start_child("trap '' TERM; /bin/sleep 5; :");
    let stage: SignalStage =
        end_the_blockers(&[blocker], a_short_window(), std::thread::sleep, || {
            return true;
        });
    assert_eq!(stage.endings.len(), 1);
    assert_eq!(stage.endings[0].1, EndResult::Killed);
    assert_eq!(stage.killed(), 1);
    let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
}

/// A wrapper that ended during the wait sends no kill,
/// and the stage says the caller is gone.
#[test]
fn a_dead_wrapper_sends_no_kill() {
    let (blocker, mut child) = start_child("trap '' TERM; /bin/sleep 5; :");
    let stage: SignalStage =
        end_the_blockers(&[blocker], a_short_window(), std::thread::sleep, || {
            return false;
        });
    assert!(stage.caller_gone);
    assert_eq!(stage.killed(), 0);
    assert_eq!(stage.endings.len(), 1);
    assert_eq!(stage.unproven(), 1);
    // The child is still there, which is what "no kill was sent" means; the control ends it itself.
    let _killed = child.kill();
    let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
}

/// A number no task has is gone,
/// and nothing is sent to it.
#[test]
fn a_number_no_task_has_is_gone() {
    let blocker: Blocker = Blocker {
        pid: 2_000_000_000,
        started: 1,
    };
    let stage: SignalStage = end_the_blockers(
        &[blocker],
        a_short_window(),
        |_duration: std::time::Duration| {
            return;
        },
        || {
            return true;
        },
    );
    assert_eq!(stage.endings, vec![(blocker, EndResult::Gone)]);
    assert_eq!(stage.unproven(), 0);
}

/// A task in an uninterruptible sleep and a zombie are not endable,
/// and a live task is.
#[test]
fn the_endability_read_tells_the_states_apart() {
    assert!(!endable_from('D'));
    assert!(!endable_from('Z'));
    assert!(!endable_from('X'));
    assert!(endable_from('S'));
    assert!(endable_from('R'));
    let (_blocker, mut child) = start_child("exec sleep 30");
    let pid: i32 = i32::try_from(child.id()).expect("a number");
    // A child settling into its first exec passes through states this act refuses, so the control waits,
    // bounded, for the state the task will hold for its life.
    let mut settled: Endability = endability_of(std::path::Path::new("/proc"), pid);
    for _attempt in 0..400 {
        if matches!(settled, Endability::Endable(_)) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
        settled = endability_of(std::path::Path::new("/proc"), pid);
    }
    match settled {
        Endability::Endable(blocker) => assert!(blocker.started > 0),
        other => panic!("a live task is endable, not {other:?}"),
    }
    let _killed = child.kill();
    let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
    assert!(matches!(
        endability_of(std::path::Path::new("/proc"), 2_000_000_000),
        Endability::Gone
    ));
}

/// A list longer than one scan answers is refused whole,
/// so the stage never signals a list no report made and never leaves part of one unmentioned.
#[test]
fn a_list_past_the_bound_is_cut() {
    assert_eq!(MOST_BLOCKERS_ONE_ACT_HOLDS, 1024);
    let blocker: Blocker = Blocker {
        pid: 2_000_000_000,
        started: 1,
    };
    let many: Vec<Blocker> = vec![blocker; MOST_BLOCKERS_ONE_ACT_HOLDS + 5];
    let stage: SignalStage = end_the_blockers(
        &many,
        a_short_window(),
        |_duration: std::time::Duration| {
            return;
        },
        || {
            return true;
        },
    );
    // A list past the scan's bound is refused whole rather than cut to a prefix the report would not mention.
    assert!(stage.endings.is_empty());
    assert_eq!(ASK, 15);
    assert_eq!(KILL, 9);
}
