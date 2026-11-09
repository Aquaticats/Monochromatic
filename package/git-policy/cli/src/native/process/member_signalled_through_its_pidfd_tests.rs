//! Controls for signalling one member of a tree through a descriptor:
//! the signal reaches the process that was found,
//! and a member the kernel refuses a descriptor for is left alone.

use super::*;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long a control waits before it calls a process it did not signal alive:
///  longer than a `SIGKILL` on its way takes.
const STILL_RUNNING_BOUND: Duration = Duration::from_millis(500);

/// A child of this test process that is nobody's step.
fn a_sleeper() -> Child {
    return crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a long-running child");
}

/// The member a look would have read for `child`.
#[cfg(target_os = "linux")]
fn member_of_the_sleeper(child: &Child) -> TreeMember {
    let pid: i32 = i32::try_from(child.id()).expect("a process number");
    let (started, _zombie): (u64, bool) = crate::process_tree::identity_of(pid).expect("the sleeper is read");
    return TreeMember { pid, started };
}

/// Whether `child` ended within the bound, and how.
fn ended_within(child: &mut Child, bound: Duration) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    let deadline: Instant = Instant::now() + bound;
    loop {
        if let Some(status) = child.try_wait().expect("status") {
            return Some(status.signal().unwrap_or(0));
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// What:
///  A member the kernel refuses a descriptor for is not signalled at all.
/// Why:
///   A signal by number after a successful check is still a signal to a number:
/// the process the check proved can end and its number pass to another command before the call,
/// so a stop must never send one it cannot pin (P1 of the 2026-11-09 second review).
/// A descriptor limit is normal here (`NHR`), so this is a path a stop really takes.
#[cfg(target_os = "linux")]
#[test]
fn a_member_the_kernel_refuses_a_descriptor_for_is_not_signalled() {
    let mut sleeper: Child = a_sleeper();
    let member: TreeMember = member_of_the_sleeper(&sleeper);
    refuse_the_next_descriptors(1);
    let delivered: Vec<Delivered> = kill_members(std::slice::from_ref(&member));
    let ended: Option<i32> = ended_within(&mut sleeper, STILL_RUNNING_BOUND);
    if ended.is_none() {
        sleeper.kill().expect("cleanup");
        sleeper.wait().expect("reaped");
    }
    assert!(ended.is_none(), "the member was signalled by its number after the kernel refused a descriptor for it: {ended:?}");
}

/// The positive control: with a descriptor to be had, the same member is signalled and ends,
/// so the control above cannot pass by signalling nothing at all.
#[cfg(target_os = "linux")]
#[test]
fn a_member_with_a_descriptor_is_signalled() {
    let mut sleeper: Child = a_sleeper();
    let member: TreeMember = member_of_the_sleeper(&sleeper);
    let delivered: Vec<Delivered> = kill_members(std::slice::from_ref(&member));
    let ended: Option<i32> = ended_within(&mut sleeper, STILL_RUNNING_BOUND);
    if ended.is_none() {
        sleeper.kill().expect("cleanup");
        sleeper.wait().expect("reaped");
    }
    assert_eq!(delivered, vec![Delivered::Sent], "a member whose number was proven is signalled through its descriptor");
    assert!(ended.is_some(), "the signalled member ended");
}
