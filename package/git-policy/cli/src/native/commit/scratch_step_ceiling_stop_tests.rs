//! Controls for what the ceiling of the scratch `git commit` does when it comes:
//! the step is signalled and the stop recorded,
//! and nothing is concluded from a record that could not be read.

use super::*;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long a control waits before it calls a process it did not signal alive:
///  longer than a `SIGKILL` on its way takes.
const STILL_RUNNING_BOUND: Duration = Duration::from_millis(500);

/// A child of this test process that stands for the `git commit` of the step.
fn a_step() -> Child {
    return crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a long-running child");
}

/// Whether `child` ended within the bound,
/// and how.
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
///  A step whose record could not be read keeps the ceiling armed instead of reading as ended.
/// Why:
///   Finding 3 of the second review:
/// the identity read the pin makes can fail on its own (`EACCES`,
/// `EIO`,
/// `ENOMEM`) while `/proc` answers,
/// and that failure was read as `Delivered::Gone`,
/// which `ceiling_came` turns into "the step ended":
/// a running Git child then loses its ceiling and the command waits with no bound over it.
///  The step here is alive throughout,
/// so only the reading of it fails.
#[cfg(target_os = "linux")]
#[test]
fn a_step_whose_record_could_not_be_read_keeps_the_ceiling_armed() {
    let mut step: Child = a_step();
    let pid: i32 = i32::try_from(step.id()).expect("a process number");
    // A start whose descriptor the kernel refused, and a record that answers until the pin reads it.
    let watched: ChildNumber = ChildNumber::not_started();
    watched.publish(pid);
    crate::process_tree::unreadable_after_the_next_reads(pid, 1);
    let came: CeilingCame = ceiling_came(&watched).came;
    let ended: Option<i32> = ended_within(&mut step, STILL_RUNNING_BOUND);
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_ne!(came, CeilingCame::TheStepEnded, "a record that could not be read was read as the step having ended, so the ceiling was taken away from a step that is still running");
}

/// The positive control:
/// a step that can be read is signalled and the stop is reported,
/// so the control above cannot pass by never signalling at all.
#[cfg(target_os = "linux")]
#[test]
fn a_step_that_can_be_read_is_stopped_at_the_ceiling() {
    let mut step: Child = a_step();
    let pid: i32 = i32::try_from(step.id()).expect("a process number");
    let watched: ChildNumber = ChildNumber::not_started_pinned();
    watched.publish(pid);
    let came: CeilingCame = ceiling_came(&watched).came;
    let ended: Option<i32> = ended_within(&mut step, STILL_RUNNING_BOUND);
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_eq!(came, CeilingCame::Stopped, "a step that was signalled is reported as stopped");
    assert!(ended.is_some(), "the signalled step ended");
}

/// What:
///  The ceiling takes the step's identity from its start,
/// never from a read of its number.
/// Why:
///   Finding 1 of the second review:
/// the number is published when the step starts,
/// but the identity is read at the ceiling,
/// so the read proves the process found then,
/// and a step that ended and had its number reused is signalled through the successor.
///  A descriptor opened the moment the step starts names the step for good,
/// while this process is its parent and the number cannot have been reused.
/// The control makes every read of the step's number fail:
/// a ceiling that still works proves it needs none of them.
#[cfg(target_os = "linux")]
#[test]
fn the_ceiling_takes_the_step_from_its_start_and_never_from_a_read_of_its_number() {
    let mut step: Child = a_step();
    let pid: i32 = i32::try_from(step.id()).expect("a process number");
    let watched: ChildNumber = ChildNumber::not_started_pinned();
    watched.publish(pid);
    crate::process_tree::unreadable_after_the_next_reads(pid, 0);
    let came: CeilingCame = ceiling_came(&watched).came;
    let ended: Option<i32> = ended_within(&mut step, STILL_RUNNING_BOUND);
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_eq!(came, CeilingCame::Stopped, "the step was not signalled, so the ceiling read its number instead of taking it from the start");
    assert!(ended.is_some(), "the step the ceiling took from its start ended");
}

/// What:
///  A step whose start gave no descriptor is never signalled by this ceiling,
///  and the ceiling concludes nothing about it.
/// Why:
///   Finding 5 of the fifth review:
/// a step with no descriptor was found by its number and a descriptor was opened for that number afterwards,
/// so a step that ended and was reaped in the interval left a number another process could hold,
/// and the signal that followed reached that process.
///  The rule for this package is that a process number is safe only while the process is this command's own
/// unreaped child,
/// and proving that and opening a descriptor for it are two operations,
/// so nothing here sends a signal to a number at all.
///  The commit stays bounded without this path:
/// the wait that owns the step ends the child it holds (`child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`).
/// The step here is alive and its record reads perfectly;
/// only the missing descriptor stands between it and a signal.
#[cfg(target_os = "linux")]
#[test]
fn a_step_without_a_descriptor_is_never_signalled() {
    let mut step: Child = a_step();
    let watched: ChildNumber = ChildNumber::not_started();
    watched.publish(i32::try_from(step.id()).expect("a process number"));
    let came: CeilingCame = ceiling_came(&watched).came;
    let ended: Option<i32> = ended_within(&mut step, STILL_RUNNING_BOUND);
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_eq!(came, CeilingCame::Unknown, "the ceiling read a number and opened a descriptor for it afterwards, which a reuse can be behind");
    assert!(ended.is_none(), "the step was signalled through a number the ceiling read: {ended:?}");
}

/// What:
///  A number that names a process this command did not start is never signalled either,
///  and proves nothing.
/// Why:
///   The ceiling reads no number at all now,
/// so a process of another command is never a candidate for its signal,
/// and nothing is concluded about the step from such a number.
#[cfg(target_os = "linux")]
#[test]
fn a_number_that_names_another_process_s_child_is_never_read() {
    // The init process: alive, and nobody's child of this command.
    let watched: ChildNumber = ChildNumber::not_started();
    watched.publish(1);
    let came: CeilingCame = ceiling_came(&watched).came;
    assert_eq!(came, CeilingCame::Unknown, "a number was read for a step whose start gave no descriptor");
}

/// What:
///  A step that started without a descriptor and has been reaped proves nothing about its number.
/// Why:
///   A reaped step's number can belong to another process in the next instant,
/// so nothing may be concluded from it,
/// and the ceiling stays armed until the wait that owns the step has ended it.
#[cfg(target_os = "linux")]
#[test]
fn a_step_without_a_descriptor_that_was_reaped_proves_nothing() {
    let mut step: Child = a_step();
    let watched: ChildNumber = ChildNumber::not_started();
    watched.publish(i32::try_from(step.id()).expect("a process number"));
    step.kill().expect("the step ends");
    step.wait().expect("reaped");
    let came: CeilingCame = ceiling_came(&watched).came;
    assert_eq!(came, CeilingCame::Unknown, "a number was read for a step that was already reaped");
}

/// What:
///  A walk that ran while the step was alive proves nothing once the step has ended.
/// Why:
///   Finding 2 of the third review:
/// a number cannot be reused while its process is alive,
/// so the walk is safe only for the instant it is taken,
/// and stopping at that instant and then signalling the numbers it read reaches whatever took them.
/// The step here is published while it is alive,
/// ends before the ceiling looks,
/// and the walk is made to answer a live process that the step never started.
#[cfg(target_os = "linux")]
#[test]
fn a_walk_whose_step_ended_signals_nothing() {
    let mut step: Child = a_step();
    let watched: ChildNumber = ChildNumber::not_started_pinned();
    watched.publish(i32::try_from(step.id()).expect("a process number"));
    let mut never_the_steps_child: Child = a_step();
    let planted_pid: i32 = i32::try_from(never_the_steps_child.id()).expect("a process number");
    let planted: crate::process_tree::TreeMember = match crate::process_tree::member_read(planted_pid) {
        crate::process_tree::MemberRead::Found { member, .. } => member,
        other => panic!("the planted member must be a live process: {other:?}"),
    };
    step.kill().expect("the step ends");
    step.wait().expect("reaped");
    plant_what_the_walk_finds(vec![planted]);
    let came: CeilingCame = ceiling_came(&watched).came;
    let ended: Option<i32> = ended_within(&mut never_the_steps_child, STILL_RUNNING_BOUND);
    if ended.is_none() {
        never_the_steps_child.kill().expect("cleanup");
        never_the_steps_child.wait().expect("reaped");
    }
    assert_eq!(came, CeilingCame::TheStepEnded, "a step that ended while its children were read has ended");
    assert!(ended.is_none(), "a number the walk read was signalled although the step had ended: {ended:?}");
}

/// What:
///  A refused signal that is not "no such process" keeps the ceiling armed.
/// Why:
///   Finding 3 of the third review at the level of the ceiling:
/// a refusal such as `EPERM` says nothing about whether the step is still running,
/// so `TheStepEnded` there takes the ceiling away from a step that is still there and waiting.
#[cfg(target_os = "linux")]
#[test]
fn a_refused_signal_that_is_not_no_such_process_keeps_the_ceiling_armed() {
    let mut step: Child = a_step();
    let watched: ChildNumber = ChildNumber::not_started_pinned();
    watched.publish(i32::try_from(step.id()).expect("a process number"));
    crate::member_signalled_through_its_pidfd::refuse_the_next_signal_with(libc::EPERM);
    let came: CeilingCame = ceiling_came(&watched).came;
    let ended: Option<i32> = ended_within(&mut step, STILL_RUNNING_BOUND);
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_eq!(came, CeilingCame::Unknown, "a refused signal proves nothing about the step, so the ceiling stays armed");
    assert!(ended.is_none(), "the step was reported gone and left alone although the refusal was not `ESRCH`: {ended:?}");
}

/// What:
///  A walk that cannot be made does not keep the pinned step from being signalled.
/// Why:
///   Finding 2 of the fourth review:
/// the walk was read before the signal and its failure returned `Unknown` first,
/// so a step whose descriptor names it perfectly well ran on although the ceiling had come,
/// and the notice retried for as long as `/proc` refused the walk.
///  The step here is alive and its descriptor was made at its start;
/// only the walk below it fails.
#[cfg(target_os = "linux")]
#[test]
fn a_walk_that_cannot_be_made_does_not_keep_the_step_from_being_signalled() {
    let mut step: Child = a_step();
    let watched: ChildNumber = ChildNumber::not_started_pinned();
    watched.publish(i32::try_from(step.id()).expect("a process number"));
    refuse_the_next_walk();
    let came: CeilingCame = ceiling_came(&watched).came;
    let ended: Option<i32> = ended_within(&mut step, STILL_RUNNING_BOUND);
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert!(ended.is_some(), "the step the descriptor names ran on although the walk below it failed: {came:?}");
    assert_eq!(came, CeilingCame::Stopped, "a step that was signalled through its descriptor was stopped");
}
