//! Controls for the children a command stopped waiting for:
//! the `Child` the standard library made,
//! and the number a raw fork leaves behind.

use super::*;
use std::process::{
    Command,
    Stdio,
};

/// A child of a shell command line whose streams go nowhere,
/// started by `test_support::spawn_owned` so the test thread's fixtures end it however the test ends.
fn quiet(line: &str) -> Child {
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg(line)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    return crate::test_support::spawn_owned(&mut command).expect("the stand-in starts");
}

/// The number this host names one child by,
/// read from the handle the standard library made.
fn number_of(child: &Child) -> libc::pid_t {
    return libc::pid_t::try_from(child.id()).expect("a number this host can name");
}

/// How long a collected child is given to leave the process table.
const COLLECTION_PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);

/// The wait between two looks at one number's process table entry.
const LOOK_AGAIN: std::time::Duration = std::time::Duration::from_millis(10);

/// Whether the number `pid` still names a process.
fn still_listed(pid: libc::pid_t) -> bool {
    return std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok();
}

/// What:
///  A child handed to a reaper is collected when it ends,
/// and the command does not wait for it.
/// Why:
///  A child in an uninterruptible call cannot be ended,
/// so the command stops waiting and a thread of its own collects the
///  end whenever the kernel releases it.
#[test]
fn a_child_kept_for_a_reaper_is_collected_when_it_ends() {
    let child: Child = quiet("sleep 0.2");
    let pid: libc::pid_t = number_of(&child);
    let began = std::time::Instant::now();
    reap_soon(child);
    // The call returns at once: the command never waits for the child here.
    assert!(
        began.elapsed() < std::time::Duration::from_millis(100),
        "reap_soon waited for the child: {:?}",
        began.elapsed()
    );
    // The record is gone once the reaper has collected the child, and no zombie of that number is left.
    let deadline = std::time::Instant::now() + COLLECTION_PATIENCE;
    while still_listed(pid) && std::time::Instant::now() < deadline {
        std::thread::sleep(LOOK_AGAIN);
    }
    assert!(
        !still_listed(pid),
        "the child handed to a reaper was never collected"
    );
}

/// What:
///  A child this process forked itself,
///  which its starter holds only as its number,
///  is handed to a reaper and collected when it ends,
///  and the call returns at once.
/// Why:
///  A raw `fork` leaves the number and no `Child`,
/// and that is the handle the account companion's parent has for the child its
///  deadline kills (`src/native/account_companion/companion_parent.rs`),
/// so the number is the handle this half of the reaper must be held to.
#[test]
fn a_number_handed_to_a_reaper_is_collected_when_it_ends() {
    let child: Child = quiet("sleep 0.2");
    let pid: libc::pid_t = number_of(&child);
    // The starter keeps no handle, which is exactly what a raw `fork` leaves behind.
    drop(child);
    let began = std::time::Instant::now();
    reap_soon_pid(pid);
    assert!(
        began.elapsed() < std::time::Duration::from_millis(100),
        "reap_soon_pid waited for the child: {:?}",
        began.elapsed()
    );
    let deadline = std::time::Instant::now() + COLLECTION_PATIENCE;
    while still_listed(pid) && std::time::Instant::now() < deadline {
        std::thread::sleep(LOOK_AGAIN);
    }
    assert!(
        !still_listed(pid),
        "the child whose number was handed to a reaper was never collected"
    );
}

/// What:
///  A child and a number whose reaper could not start are kept,
/// not lost,
/// and the next reaper that runs collects both.
/// Why:
///  The standard library's spawn consumes the closure,
/// so a refused start would drop the child and leave it a zombie for
///  as long as this process lives;
/// the list of children waiting for a reaper is what keeps it,
///  and one list holds both handles so a reaper drains whatever a refused start left.
#[test]
fn a_child_and_a_number_whose_reaper_could_not_start_are_kept_and_collected_later() {
    let child: Child = quiet("exit 0");
    let child_pid: libc::pid_t = number_of(&child);
    let numbered: Child = quiet("exit 0");
    let number_pid: libc::pid_t = number_of(&numbered);
    drop(numbered);
    keep(Nowait::Child(child));
    keep(Nowait::Pid(number_pid));
    assert_eq!(
        waiting_count(),
        2,
        "the child and the number a refused reaper would have lost are not kept"
    );
    drain_the_registry();
    let deadline = std::time::Instant::now() + COLLECTION_PATIENCE;
    while (still_listed(child_pid) || still_listed(number_pid))
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(LOOK_AGAIN);
    }
    assert_eq!(
        waiting_count(),
        0,
        "the kept children were not taken by the reaper"
    );
    assert!(
        !still_listed(child_pid),
        "the kept child was never collected"
    );
    assert!(
        !still_listed(number_pid),
        "the kept number was never collected"
    );
}
