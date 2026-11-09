//! Controls for holding one process of a tree by a pidfd and signalling it through that hold.

use super::*;
use crate::process_tree::TreeMember;
use crate::test_support::spawn_owned;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long a control waits for a process to end before it reports that the process was not stopped.
const ENDING_BOUND: Duration = Duration::from_secs(10);

/// A sleeper of this test,
/// ended and reaped by the control that made it.
fn sleeper() -> Child {
    return spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a sleeper starts");
}

/// The member a look at a running process gives,
/// with the start time that names it.
fn member_of(process: &Child) -> TreeMember {
    let pid: i32 = i32::try_from(process.id()).expect("a process number");
    return match look_of(pid) {
        ProcessLook::Alive(found) => found,
        other => panic!("the control's process is alive: {other:?}"),
    };
}

/// Whether the process still runs,
/// which reads it as its own end rather than as a zombie a signal 0 would still reach.
fn runs(pid: i32) -> bool {
    return !matches!(crate::process_identity::process_birth_identity(i64::from(pid)), Ok(None));
}

/// Waits,
/// bounded,
/// for `pid` to end and says whether it did.
fn ended_within_bound(pid: i32) -> bool {
    let began: Instant = Instant::now();
    while runs(pid) {
        if began.elapsed() >= ENDING_BOUND {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    return true;
}

/// What:
///  A member is held by a pidfd and the signal goes through it,
/// so only that process can be reached.
/// Why:
///   Signalling by number after reading the start time leaves the window between the read and the call,
///  in which the number can name another process of this account
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 7).
/// A descriptor names one process until it is closed.
/// The control also pins the identity check:
/// a member whose start time is not the one the process has is not held at all.
#[cfg(target_os = "linux")]
#[test]
fn a_member_is_held_by_its_pidfd_and_only_that_process_is_signalled() {
    let mut process: Child = sleeper();
    let member: TreeMember = member_of(&process);
    let held: PinnedProcess = match pin_of(&member) {
        PinOutcome::Held(pinned) => pinned,
        other => panic!("the running process is held: {other:?}"),
    };
    assert!(held.held_by_a_pidfd(), "the hold is a descriptor, which no reused number can reach");
    // Another start time is another process: nothing is held and nothing is signalled.
    let stale: TreeMember = TreeMember { pid: member.pid, started: member.started + 1 };
    assert!(matches!(pin_of(&stale), PinOutcome::Gone), "a number whose start time changed is not held");
    // The signal goes through the hold and reaches this process.
    assert_eq!(held.signal(libc::SIGKILL), Delivered::Sent, "the signal is delivered through the descriptor");
    let ended: bool = ended_within_bound(member.pid);
    process.wait().expect("the sleeper is collected");
    assert!(ended, "the pinned process ended");
    // A hold of a process that has ended sends nothing.
    assert_eq!(held.signal(libc::SIGKILL), Delivered::Gone, "a hold of an ended process reaches nothing");
    assert!(matches!(pin_of(&member), PinOutcome::Gone), "the number names no process that started then");
}

/// A member whose record cannot be read is signalled by its number,
/// because nothing about the read proves the process ended.
///  The seam is the record's own directory,
/// as the look's control uses it:
/// a directory where the record should be is an `EISDIR` no wait makes go away.
#[cfg(target_os = "linux")]
#[test]
fn a_member_whose_record_could_not_be_read_is_signalled_by_its_number() {
    let mut process: Child = sleeper();
    let member: TreeMember = member_of(&process);
    let root: PathBuf = crate::test_support::fixture("pinned-member-look");
    let refusing: PathBuf = root.join("refusing");
    let record: PathBuf = refusing.join(member.pid.to_string());
    std::fs::create_dir_all(record.join("stat").as_path()).expect("a directory where the record should be");
    let outcome: PinOutcome = pin_of_through(refusing.as_path(), &member);
    crate::test_support::remove(root.as_path());
    assert!(matches!(outcome, PinOutcome::Unreadable), "a record that could not be read is not a process that ended");
    // The number is what is left to signal, and the control's own process is what it reaches.
    assert_eq!(signal_member(&member, libc::SIGKILL), Delivered::Sent, "the number is signalled");
    let ended: bool = ended_within_bound(member.pid);
    process.wait().expect("the sleeper is collected");
    assert!(ended, "the process the refused record named ended");
}
