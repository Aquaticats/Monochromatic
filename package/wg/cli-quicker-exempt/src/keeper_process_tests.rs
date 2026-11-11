//! Verifies that stopping a watcher or keeper ends the whole process group it leads:
//! its `ip` and manager clients are children in that group,
//! and a signal to the leader alone leaves them running
//! and able to change the user's manager after the stop returned.

/// The stop under test.
use crate::keeper_process::{process_start_time, stop_command_process};
/// Starting a fixture in its own session, as the watcher and the keeper are.
use std::os::unix::process::CommandExt;
/// The fixture's processes.
use std::process::{Child, Command, Stdio};

/// The program line the fixture runs.
const FIXTURE_SCRIPT: &str = "/usr/bin/sleep 60 & wait";

/// How many processes are in a process group,
/// read from the `pgrp` field of every `/proc/<pid>/stat`.
/// Counting and not asking whether the group exists matters:
/// a group of one member is the leader alone, which is what a stop that missed the children leaves.
fn group_member_count(pid: i32) -> usize {
    let mut members = 0;
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return members;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_text) = name.to_str() else {
            continue;
        };
        if !name_text.bytes().all(|byte| return byte.is_ascii_digit()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        // The command name is in parentheses and may hold spaces, so the fields after it are read from its end.
        let Some(after_name) = text.rfind(')') else {
            continue;
        };
        let fields: Vec<&str> = text[after_name + 1..].split_whitespace().collect();
        // `state` is the first field after the name, so `pgrp` is the third.
        let Some(group) = fields.get(2).and_then(|field| return field.parse::<i32>().ok()) else {
            continue;
        };
        if group == pid {
            members += 1;
        }
    }
    return members;
}

/// Starts the fixture as the leader of its own session and process group.
fn session_leader() -> Child {
    let mut command = Command::new("/bin/sh");
    command.arg("-c").arg(FIXTURE_SCRIPT).stdout(Stdio::null()).stderr(Stdio::null());
    // SAFETY: setsid is async-signal-safe and touches only the child's session state.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    return command.spawn().expect("the fixture starts");
}

/// The exact command line the fixture shows in procfs.
fn expected_command() -> Vec<Vec<u8>> {
    return vec![
        b"/bin/sh".to_vec(),
        b"-c".to_vec(),
        FIXTURE_SCRIPT.as_bytes().to_vec(),
    ];
}

/// The child of a watcher is in the watcher's group,
/// so a stop that signalled only the leader left it running.
/// The stop ends the group, and the wait it already had stays bounded.
#[test]
fn a_stop_ends_the_process_group_of_the_watcher_it_stops() {
    let mut child = session_leader();
    let pid = child.id() as i32;
    let start_time = process_start_time(pid).expect("the fixture has a start time");
    // The shell starts its child a moment after it starts, and a busy machine makes that moment longer:
    // the fixture is asked for a bounded while, and the case fails only when the child never appears.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while group_member_count(pid) <= 1 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(group_member_count(pid) > 1, "the fixture's child is in the group the fixture leads");
    stop_command_process(pid, start_time, &expected_command(), "watcher").expect("the stop succeeds");
    assert_eq!(group_member_count(pid), 0, "no process of the group is left");
    let _reaped = child.wait();
}

/// A pid the kernel gave to another process is never signalled as a group,
/// because the group of the recorded watcher is gone by then.
#[test]
fn a_stop_of_a_pid_that_names_another_command_is_refused() {
    let mut child = Command::new("/usr/bin/sleep").arg("60").stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("the fixture starts");
    let pid = child.id() as i32;
    let start_time = process_start_time(pid).expect("the fixture has a start time");
    let refusal = stop_command_process(pid, start_time, &expected_command(), "watcher").expect_err("the stop is refused");
    assert_eq!(refusal.kind(), std::io::ErrorKind::InvalidData);
    let _killed = child.kill();
    let _reaped = child.wait();
}

/// A watcher killed on its own leaves its children in the group it led,
/// and the group is ended from the recorded identity,
/// so a stop that finds only the group still ends it.
#[test]
fn the_group_of_a_killed_watcher_is_ended_and_an_absent_group_is_no_work() {
    use crate::keeper_process::{end_leftover_group, LeftoverGroup};
    let mut child = session_leader();
    let pid = child.id() as i32;
    let start_time = process_start_time(pid).expect("the fixture has a start time");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while group_member_count(pid) <= 1 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(group_member_count(pid) > 1, "the fixture's child is in the group the fixture leads");
    assert_eq!(unsafe { libc::kill(pid, libc::SIGKILL) }, 0, "the leader is killed on its own");
    let _reaped = child.wait();
    assert_eq!(group_member_count(pid), 1, "the child outlived the leader in the recorded group");
    assert_eq!(end_leftover_group(pid, start_time, "watcher").expect("the group is ended"), LeftoverGroup::Ended);
    assert_eq!(group_member_count(pid), 0, "no process of the group is left");
    assert_eq!(end_leftover_group(pid, start_time, "watcher").expect("an absent group answers"), LeftoverGroup::Absent);
}

/// A group whose number the kernel gave to another process is never signalled,
/// because the recorded watcher and its group are gone by then.
#[test]
fn a_group_recorded_under_another_start_time_is_refused() {
    use crate::keeper_process::end_leftover_group;
    let mut child = session_leader();
    let pid = child.id() as i32;
    let recorded = process_start_time(pid).expect("the fixture has a start time");
    let refusal = end_leftover_group(pid, recorded + 1, "watcher").expect_err("the group is refused");
    assert_eq!(refusal.kind(), std::io::ErrorKind::InvalidData, "{refusal}");
    assert!(group_member_count(pid) > 0, "the refused group was signalled anyway");
    let _killed = unsafe { libc::kill(-pid, libc::SIGKILL) };
    let _reaped = child.wait();
}
