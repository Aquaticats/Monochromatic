//! What:
//!  Controls of the detached helper launch where a step fails after the first fork: the first child's second `fork`, the helper's stream
//!  setup and its descriptor closing. Each is injected through the launch's test seam in a copy of the test process that has one thread.
//! Why:
//!  Review 9a section 3A: a launch that reports success while the first child exited with a failure code, or while the helper never got
//!  to run, claims a helper that does not exist. The launcher is the only party that can tell the caller, so the launch waits for the
//!  helper's own ready report and names which step failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a second fork that fails is a failed launch', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_helper_launch_tests::launch_code;
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_helper_launch::{LaunchFault, inject_launch_fault, launch_detached};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The exit code `launch_code` gives each error; named here so the assertions read as outcomes.
const FIRST_CHILD_FAILED: i32 = 13;
const SETUP_FAILED: i32 = 14;

/// Launches an entry that would write a file, with `fault` injected; the code of the launch and whether the entry ever ran.
fn launch_with(label: &str, fault: LaunchFault) -> (Option<i32>, bool) {
    let lane: PathBuf = scratch(label);
    let ran: PathBuf = lane.join("ran");
    let marker: PathBuf = ran.clone();
    let code: Option<i32> = in_single_threaded_process(move || {
        inject_launch_fault(Some(fault));
        return launch_code(launch_detached(move || {
            fs::write(&marker, b"ran").expect("the entry's mark");
            return 0;
        }));
    });
    let deadline: Instant = Instant::now() + Duration::from_millis(300);
    while !ran.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    return (code, ran.exists());
}

/// The first fork succeeds and the second fails: the first child leaves with a failure code and no helper exists. The launch says so.
#[test]
fn a_failed_second_fork_is_a_failed_launch_and_no_helper_runs() {
    let (code, ran) = launch_with("launch-second-fork", LaunchFault::SecondFork);
    assert_eq!(code, Some(FIRST_CHILD_FAILED), "the first child's failure reaches the caller");
    assert!(!ran, "no helper exists");
}

/// The helper cannot point its streams at `/dev/null`: it exits before its entry and the launch reports the setup failure.
#[test]
fn a_helper_whose_stream_setup_fails_is_reported_to_the_launcher() {
    let (code, ran) = launch_with("launch-streams", LaunchFault::Streams);
    assert_eq!(code, Some(SETUP_FAILED), "the helper's setup failure reaches the caller");
    assert!(!ran, "the entry never ran");
}

/// The helper cannot close the descriptors it inherited: the entry must not run with them, and the launch reports the failure.
#[test]
fn a_helper_whose_descriptor_closing_fails_is_reported_to_the_launcher() {
    let (code, ran) = launch_with("launch-descriptors", LaunchFault::Descriptors);
    assert_eq!(code, Some(SETUP_FAILED), "the helper's setup failure reaches the caller");
    assert!(!ran, "the entry never ran");
}

/// Review 10a section 3D: a ready byte returned success even when the intermediate child had not been reaped; paused after its second
/// fork, it still held every inherited lock and pipe description. The launch accounts for the intermediate child too: it kills and reaps
/// a first child that does not leave in time, and returns success only with no child of the launcher left unreaped.
#[test]
fn a_ready_helper_does_not_make_the_launch_succeed_while_the_intermediate_child_is_unreaped() {
    let code: Option<i32> = in_single_threaded_process(|| {
        inject_launch_fault(Some(LaunchFault::FirstChildHangs));
        let launched: i32 = launch_code(launch_detached(|| { return 0; }));
        if launched != 0 {
            return launched;
        }
        // SAFETY: `waitpid` with a null status asks only whether any child is left: -1 with `ECHILD` means none, 0 means one still runs.
        let left: libc::pid_t = unsafe { libc::waitpid(-1, std::ptr::null_mut(), libc::WNOHANG) };
        return if left == -1 { 0 } else { 40 };
    });
    assert_eq!(code, Some(0), "the launch left its intermediate child unreaped");
}
