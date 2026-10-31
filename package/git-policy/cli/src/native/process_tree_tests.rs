//! Controls for reading the processes below one process and signalling them safely.

use super::*;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

/// A shell that starts a sleeping child, prints its number, and waits for it.
fn shell_with_a_sleeper() -> (Child, i32) {
    let mut shell: Child = Command::new("sh")
        .args(["-c", "sleep 600 & echo $!; wait"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the shell starts");
    let mut line: String = String::new();
    BufReader::new(shell.stdout.take().expect("piped stdout"))
        .read_line(&mut line)
        .expect("the sleeper's number");
    let sleeper: i32 = line.trim().parse().expect("a process number");
    return (shell, sleeper);
}

/// The command name sits in parentheses and may hold spaces and parentheses; the fields are read after the last one.
#[cfg(target_os = "linux")]
#[test]
fn a_stat_line_is_read_after_the_last_parenthesis() {
    let line: &str = "1234 (we ) ird) S 99 1 1 0 -1 4194304 0 0 0 0 0 0 0 0 20 0 1 0 555 0 0";
    assert_eq!(parse_stat(line), Some((99, 555, false)));
    let zombie: &str = "1234 (cat) Z 7 1 1 0 -1 4194304 0 0 0 0 0 0 0 0 20 0 1 0 9 0 0";
    assert_eq!(parse_stat(zombie), Some((7, 9, true)));
    assert_eq!(parse_stat("1234 (cat) S 7"), None);
    assert_eq!(parse_stat("no parenthesis"), None);
}

/// A grandchild is found below its grandparent, and signalling the found members ends it.
#[cfg(target_os = "linux")]
#[test]
fn a_descendant_is_found_and_signalled() {
    let (mut shell, sleeper): (Child, i32) = shell_with_a_sleeper();
    let root: i32 = i32::try_from(shell.id()).expect("a process number");
    let found: Vec<TreeMember> = descendants_of(root);
    let member: &TreeMember = found.iter().find(|candidate| { return candidate.pid == sleeper; }).expect("the sleeper is below the shell");
    assert!(still_running(member));
    signal_members(&found, libc::SIGKILL);
    // The shell's `wait` returns when the sleeper has ended, so this is the event the check below needs.
    let status = shell.wait().expect("the shell ends");
    assert!(status.success() || status.code().is_some(), "{status:?}");
    assert!(!still_running(member), "the sleeper ended");
}

/// A number whose start time differs is another process: it is not signalled.
#[cfg(target_os = "linux")]
#[test]
fn a_reused_number_is_not_signalled() {
    let (mut shell, sleeper): (Child, i32) = shell_with_a_sleeper();
    let root: i32 = i32::try_from(shell.id()).expect("a process number");
    let found: Vec<TreeMember> = descendants_of(root);
    let real: &TreeMember = found.iter().find(|candidate| { return candidate.pid == sleeper; }).expect("the sleeper is below the shell");
    let stale: TreeMember = TreeMember { pid: real.pid, started: real.started + 1 };
    assert!(!still_running(&stale));
    signal_members(&[stale], libc::SIGKILL);
    assert!(still_running(real), "the process with the other start time was left alone");
    signal_members(&found, libc::SIGKILL);
    shell.wait().expect("the shell ends");
}

/// A process with no children has no descendants.
#[cfg(target_os = "linux")]
#[test]
fn a_leaf_has_no_descendants() {
    let mut sleeper: Child = Command::new("sleep").arg("600").stdin(Stdio::null()).spawn().expect("sleep starts");
    assert_eq!(descendants_of(i32::try_from(sleeper.id()).expect("a process number")), Vec::<TreeMember>::new());
    sleeper.kill().expect("sleep is killed");
    sleeper.wait().expect("sleep is collected");
}
