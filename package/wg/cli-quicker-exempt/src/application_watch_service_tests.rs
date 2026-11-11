//! Verifies which watcher log lines are relayed to the terminal that ran the start.

/// Relay selection under test.
use crate::startup_relay::{failure_lines_since, relayed_text};
/// The marker of a notice about the log itself.
use crate::log_line::LOG_NOTICE_MARKER;
/// The marker the producer of loud reports uses.
use crate::update_report::policy_failure;

/// Only loud lines written after the start began are relayed.
#[test]
fn only_new_loud_lines_are_relayed() {
    let old = format!("{}\n", policy_failure("an earlier run failed"));
    let notice = "wg-quicker-exempt: POLICY NOTICE. By the repository's policy this traffic leaves outside the tunnel\n";
    let fresh = format!("{notice}{}\nmoved pid 5\n", policy_failure("the updates slice cannot be created"));
    let log = format!("{old}{fresh}");
    let offset = u64::try_from(old.len()).expect("fits");
    let lines = failure_lines_since(&log, offset);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("the updates slice cannot be created"));
    assert_eq!(failure_lines_since(&log, 0).len(), 2);
    assert!(failure_lines_since(&log, u64::try_from(log.len()).expect("fits")).is_empty());
    assert!(failure_lines_since(&log, u64::MAX).is_empty());
    assert!(failure_lines_since("", 0).is_empty());
}

/// A notice that the log dropped lines is relayed next to the loud reports,
/// because a dropped line may have been a report.
#[test]
fn log_limit_notices_are_relayed_with_the_loud_reports() {
    let log = format!(
        "{}\nwg-quicker-exempt: {LOG_NOTICE_MARKER}: 12 log line(s) were dropped because the log budget was reached\nmoved pid 5\n",
        policy_failure("a process was skipped")
    );
    let lines = failure_lines_since(&log, 0);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[1].contains("were dropped"), "{lines:?}");
}

/// A start with more findings than the terminal shows prints the first ones,
/// escaped,
/// and says how many are left in the log;
/// a start with fewer prints them all and no count.
#[test]
fn a_relay_of_many_lines_is_bounded_and_says_how_many_are_in_the_log() {
    let lines: Vec<String> = (0..250).map(|index| return policy_failure(&format!("finding number {index}"))).collect();
    let printed = relayed_text(&lines, "/run/wg-quicker-exempt/watchers/x/log");
    assert_eq!(printed.len(), 201);
    assert!(printed[0].contains("finding number 0"));
    assert!(printed[199].contains("finding number 199"));
    assert!(printed[200].contains("50 more loud line(s)"), "{}", printed[200]);
    assert!(printed[200].contains("/run/wg-quicker-exempt/watchers/x/log"));
    let few = relayed_text(&lines[..3], "/log");
    assert_eq!(few.len(), 3);
    let hostile = vec!["wg-quicker-exempt: POLICY NOT IN FORCE: name \u{1b}[2J".to_string()];
    assert!(!relayed_text(&hostile, "/log")[0].contains('\u{1b}'));
    assert!(relayed_text(&[], "/log").is_empty());
}

/// A watcher leads its own session,
/// and a child it started stays in the process group it left.
fn a_watcher_that_left_a_child(name: &str) -> (std::process::Child, i32) {
    use std::io::BufRead;
    use std::os::unix::process::CommandExt;
    let mut command = std::process::Command::new("/bin/sh");
    command
        .arg("-c")
        .arg("/usr/bin/sleep 300 & printf '%s\\n' \"$!\"; exec /usr/bin/sleep 300")
        .stdout(std::process::Stdio::piped());
    // SAFETY: `setsid` is async-signal-safe and touches only the child's session state.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    let mut child = command.spawn().expect("start a watcher that leads its own session");
    let mut line = String::new();
    std::io::BufReader::new(child.stdout.take().expect("the watcher prints")).read_line(&mut line).expect("read the child pid");
    let pid = line.trim().parse::<i32>().expect("the child pid is a number");
    let _named = name;
    return (child, pid);
}

/// The process group a process belongs to,
/// read from the `pgrp` field of its stat file,
/// or `None` when the process is gone.
fn process_group(pid: i32) -> Option<i32> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let end = text.rfind(')')?;
    let tail = text.get(end + 1..)?;
    return tail.split_whitespace().nth(2)?.parse::<i32>().ok();
}

/// A watcher that was killed on its own leaves the children it started in the group it led,
/// and a stop ends that group before it forgets the watcher.
///
/// The watcher's child is an `ip` or manager client in production,
/// and one left running can finish a manager change after the stop returned.
#[test]
fn a_stop_ends_the_group_a_killed_watcher_left() {
    use crate::application_watch_state::{read_watch_state, watch_state_path, write_watch_state, WatchProcess};
    use crate::application_watch_service::stop_application_watch;
    use crate::keeper_process::process_start_time;
    let root = crate::test_scratch::scratch_root().join(format!("wg-quicker-watch-stop-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("make the scratch state root");
    // SAFETY: this test process sets one variable that nothing else in it reads at the same time.
    unsafe { std::env::set_var("WG_QUICKER_EXEMPT_TEST_WATCH_STATE_ROOT", &root) };
    let key = "watch-stop-group";
    let (mut leader, child) = a_watcher_that_left_a_child(key);
    let leader_pid = i32::try_from(leader.id()).expect("pid fits");
    let start_time = process_start_time(leader_pid).expect("read the watcher start time");
    assert_eq!(process_group(child), Some(leader_pid), "the child is in the group the watcher leads");
    write_watch_state(key, WatchProcess { pid: leader_pid, start_time, mark: 1, uid: 0 }).expect("write the watcher state");
    // The watcher leader is killed on its own, as an out-of-memory kill does:
    // the child it started keeps running in the recorded group.
    assert_eq!(unsafe { libc::kill(leader_pid, libc::SIGKILL) }, 0, "kill the watcher leader");
    let _reaped = leader.wait();
    assert_eq!(process_group(child), Some(leader_pid), "the child outlived the leader in the recorded group");
    stop_application_watch(key).expect("the stop answers");
    let left = process_group(child);
    let _killed = unsafe { libc::kill(child, libc::SIGKILL) };
    assert_eq!(left, None, "the child the killed watcher left was still running after the stop");
    assert!(read_watch_state(key).expect("the state reads").is_none(), "the record of the watcher was kept");
    assert!(!watch_state_path(key).expect("the path builds").exists());
    // SAFETY: the variable is removed by the test that set it, and nothing else reads it.
    unsafe { std::env::remove_var("WG_QUICKER_EXEMPT_TEST_WATCH_STATE_ROOT") };
    let _cleaned = std::fs::remove_dir_all(&root);
}
