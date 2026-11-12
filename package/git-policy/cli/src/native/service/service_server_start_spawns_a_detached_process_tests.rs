//! Controls of the detached start:
//! the process leaves the command's session and terminal,
//! its output goes to the file it was given,
//! and its number is reported.

use super::*;

fn folder(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture_root().join(format!("detached-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// The field of `/proc/<pid>/stat` that names a process's session.
fn session_of(pid: i32) -> Option<u32> {
    let stat: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after: &str = stat.get(stat.rfind(')')? + 2..)?;
    return after.split(' ').nth(4)?.parse().ok();
}

/// The field of `/proc/<pid>/stat` that names a process's process group.
fn group_of(pid: i32) -> Option<u32> {
    let stat: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after: &str = stat.get(stat.rfind(')')? + 2..)?;
    return after.split(' ').nth(2)?.parse().ok();
}

/// What: The started process is in a session and a group of its own, and its output is in the file it was given.
/// Why:  A command's terminal, its shell and the signal that ends them must not end the service.
#[test]
fn the_started_process_leaves_the_commands_session_and_writes_to_its_file() {
    let root = folder("detach");
    let log = root.join("out.log");
    let started = spawn(&Detached {
        program: PathBuf::from("/usr/bin/env"),
        arguments: vec![OsStr::new("sh").to_os_string(), OsStr::new("-c").to_os_string(), OsStr::new("echo from-the-service; sleep 1").to_os_string()],
        log: log.clone(),
    })
    .expect("the start happens");
    let pid: i32 = started.expect("the middle process names the server");
    assert_eq!(session_of(pid), Some(u32::try_from(pid).expect("a process number")), "the server leads its own session");
    assert_eq!(group_of(pid), Some(u32::try_from(pid).expect("a process number")), "and its own process group");
    std::thread::sleep(std::time::Duration::from_millis(200));
    let written: String = std::fs::read_to_string(log.as_path()).expect("the output file is there");
    assert!(written.contains("from-the-service"), "the server's output is in its file: {written}");
}

/// What: A program that is not there costs a start that ends, never a hang, and leaves the file it was given.
/// Why:  The one failure left is a service that does not come up, and its record is that file.
#[test]
fn a_program_that_is_not_there_ends_the_start_and_leaves_its_file() {
    let root = folder("missing");
    let log = root.join("out.log");
    let started = spawn(&Detached { program: root.join("no-such-program"), arguments: Vec::new(), log: log.clone() }).expect("the start happens");
    assert!(started.is_some(), "the middle process names the child it started");
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(log.exists(), "the file the output was to go to is there");
}
