//! Controls for holding one process of a tree by a pidfd and signalling it through that hold.

use super::*;
use crate::process_tree::{TreeMember, look_of};
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

/// What:
///  A member whose record cannot be read is not signalled at all,
/// and the pin says the record could not be read.
/// Why:
///  Nothing about a refused record proves which process the number names now:
/// the member was found a moment
///  ago,
/// and it may have ended and been reaped since,
/// so its number may belong to an unrelated process.
/// A signal
///  sent to it would end that process.
/// The stop waits for such a member instead,
/// and one line says it could
///  not be stopped.
///  The seam is the record's own directory,
/// as the look's control uses it:
/// a directory where the record should be is an `EISDIR` no wait makes go away.
#[cfg(target_os = "linux")]
#[test]
fn a_member_whose_record_could_not_be_read_is_not_signalled() {
    let mut process: Child = sleeper();
    let member: TreeMember = member_of(&process);
    let root: PathBuf = crate::test_support::fixture("pinned-member-look");
    let refusing: PathBuf = root.join("refusing");
    let record: PathBuf = refusing.join(member.pid.to_string());
    std::fs::create_dir_all(record.join("stat").as_path()).expect("a directory where the record should be");
    let outcome: PinOutcome = pin_of_through(refusing.as_path(), &member);
    // Nothing proves which process the number names, so nothing is sent to it.
    let delivered: Delivered = delivered_by(outcome, libc::SIGKILL);
    crate::test_support::remove(root.as_path());
    assert_eq!(delivered, Delivered::Gone, "a number whose record could not be read is not signalled");
    std::thread::sleep(Duration::from_millis(300));
    let survived: bool = runs(member.pid);
    process.kill().expect("the control ends its sleeper");
    process.wait().expect("the sleeper is collected");
    assert!(survived, "the process the refused record named was left alone");
}

/// What:
///  A look at the record that fails after the pin's descriptor was opened does not hold the process at all,
/// and nothing is signalled.
/// Why:
///  The descriptor was opened after the first look,
/// and a second look that fails does not establish that the
///  number still named the process the first look found:
/// between the two,
/// that process can have ended and been
///  reaped,
/// and the number can name an unrelated process,
/// which the descriptor then names.
/// The pin this branch kept for that case was a
///  descriptor of the wrong process,
/// and the signal ended it.
#[cfg(target_os = "linux")]
#[test]
fn a_second_look_that_fails_holds_nothing_and_signals_nothing() {
    let mut process: Child = sleeper();
    let member: TreeMember = member_of(&process);
    // The first look reads the real record and finds the process; the second, made after the descriptor is open, reads a root
    //  where that record refuses, which is what a descriptor limit does to it.
    let root: PathBuf = crate::test_support::fixture("pinned-member-second-look");
    let refusing: PathBuf = root.join("refusing");
    std::fs::create_dir_all(refusing.join(member.pid.to_string()).join("stat").as_path()).expect("a directory where the record should be");
    let outcome: PinOutcome = pin_after_two_looks_of(std::path::Path::new("/proc"), refusing.as_path(), &member);
    crate::test_support::remove(root.as_path());
    let delivered: Delivered = delivered_by(outcome, libc::SIGKILL);
    std::thread::sleep(Duration::from_millis(300));
    let survived: bool = runs(member.pid);
    process.kill().expect("the control ends its sleeper");
    process.wait().expect("the sleeper is collected");
    assert_eq!(delivered, Delivered::Gone, "a look that failed after the descriptor was opened signals nothing");
    assert!(survived, "the process the first look found was left alone");
}

/// What:
///  Nothing in the pin module signals a process by its number,
/// and no pin exists without a descriptor.
/// Why:
///  A number is a safe address only while its process is this process's own unreaped child,
/// and every other caller
///  of this module names a process it found through `/proc`,
/// which can be reaped between the look and the signal.
///  A host that gives no descriptor therefore signals nothing at all:
/// the fallback that read the start time again and
///  then called `kill` was a window a reuse of the number could pass through.
#[test]
fn no_pin_signals_a_process_by_its_number() {
    let code: String = include_str!("pinned_member_signalled_through_a_pidfd.rs")
        .lines()
        .filter(|line| { return !line.trim_start().starts_with("//"); })
        .collect::<Vec<&str>>()
        .join("\n");
    // The needle is built at run time, so the control's own text does not contain it.
    let kill: String = format!("libc::{}", "kill");
    assert!(!code.contains(kill.as_str()), "the pin module still signals a process by its number");
    let optional: String = format!("descriptor: {}<", "Option");
    assert!(!code.contains(optional.as_str()), "a pin can still exist without a descriptor");
}
