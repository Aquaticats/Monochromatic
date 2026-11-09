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
///  A step whose record could not be read keeps the ceiling armed instead of reading as ended.
/// Why:
///   Finding 3 of the second review:
/// the identity read the pin makes can fail on its own (`EACCES`, `EIO`, `ENOMEM`) while `/proc` answers,
/// and that failure was read as `Delivered::Gone`,
/// which `ceiling_came` turns into "the step ended":
/// a running Git child then loses its ceiling and the command waits with no bound over it.
///  The step here is alive throughout, so only the reading of it fails.
#[cfg(target_os = "linux")]
#[test]
fn a_step_whose_record_could_not_be_read_keeps_the_ceiling_armed() {
    let mut step: Child = a_step();
    let pid: i32 = i32::try_from(step.id()).expect("a process number");
    // The read the caller makes answers, and the one the pin makes fails:
    crate::process_tree::unreadable_after_the_next_reads(pid, 1);
    let came: CeilingCame = ceiling_came(pid);
    let ended: Option<i32> = ended_within(&mut step, STILL_RUNNING_BOUND);
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_ne!(came, CeilingCame::TheStepEnded, "a record that could not be read was read as the step having ended, so the ceiling was taken away from a step that is still running");
}

/// The positive control: a step that can be read is signalled and the stop is reported,
/// so the control above cannot pass by never signalling at all.
#[cfg(target_os = "linux")]
#[test]
fn a_step_that_can_be_read_is_stopped_at_the_ceiling() {
    let mut step: Child = a_step();
    let pid: i32 = i32::try_from(step.id()).expect("a process number");
    let came: CeilingCame = ceiling_came(pid);
    let ended: Option<i32> = ended_within(&mut step, STILL_RUNNING_BOUND);
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_eq!(came, CeilingCame::Stopped, "a step that was signalled is reported as stopped");
    assert!(ended.is_some(), "the signalled step ended");
}
