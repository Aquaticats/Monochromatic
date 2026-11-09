//! Controls for the watched one-shot run made by a task of the runtime:
//! its output comes back whole,
//! the same step awaited on the runtime works where the synchronous face is
//!  refused from inside it,
//! a step that is never ended is awaited to its own end,
//! and no descriptor stays open after a run.
#![cfg(unix)]

use super::*;
use crate::output_capture::STEP_OUTPUT_LIMIT;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;
use std::process::Stdio;

/// More bytes than the captured steps keep (`STEP_OUTPUT_LIMIT` is the tail they keep),
/// so a run that kept only a tail would show.
const BEYOND_THE_KEPT_TAIL: usize = STEP_OUTPUT_LIMIT + STEP_OUTPUT_LIMIT / 4;

/// Terms of the Git-question class:
/// its window is far longer than any control here runs.
fn long_window_terms() -> WatchTerms {
    return WatchTerms::for_class(crate::child_class::ChildClass::Object);
}

/// A shell command line as a command with no input.
fn shell(line: &str) -> Command {
    let mut command: Command = Command::new("/bin/sh");
    command.arg("-c").arg(line).stdin(Stdio::null());
    return command;
}

/// How many descriptors this process holds.
fn open_descriptors() -> usize {
    return std::fs::read_dir("/proc/self/fd").expect("the descriptor table").count();
}

/// A step that writes more than the kept tail on both streams returns every byte of both,
/// awaited by a task that never
///  enters the runtime from inside it:
/// the pipes are read to their end and the child's own end is awaited beside them.
#[test]
fn a_step_awaited_on_the_runtime_returns_every_byte_of_both_streams() {
    let line: String = format!(
        "head -c {BEYOND_THE_KEPT_TAIL} /dev/zero | tr '\\0' a; head -c {BEYOND_THE_KEPT_TAIL} /dev/zero | tr '\\0' b >&2; exit 3"
    );
    let bound: StepBound = StepBound::EndAt(crate::wait_ceilings::OBJECT_STEP_CEILING);
    let output: Output = crate::lazy_runtime::run_on_runtime(async {
        return output_watched_within_for_a_stop_awaiting(&mut shell(line.as_str()), "a loud step", "a loud step", long_window_terms(), true, bound, false).await;
    })
    .expect("the runtime")
    .expect("the step ran to its end");
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(output.stdout.len(), BEYOND_THE_KEPT_TAIL);
    assert_eq!(output.stderr.len(), BEYOND_THE_KEPT_TAIL);
    assert!(output.stdout.iter().all(|byte| { return *byte == b'a'; }), "standard output holds only what the step wrote to it");
    assert!(output.stderr.iter().all(|byte| { return *byte == b'b'; }), "standard error holds only what the step wrote to it");
}

/// What:
///  The same step,
/// awaited on the runtime and asked of the synchronous face from inside it,
/// shows the two faces
///  differ in exactly the way this file exists for.
/// Why:
///   Without the synchronous face's refusal here,
/// the first control would pass for a reason other than the
///  one it claims.
#[test]
fn the_synchronous_face_of_the_same_step_is_refused_from_inside_the_runtime() {
    let bound: StepBound = StepBound::EndAt(crate::wait_ceilings::OBJECT_STEP_CEILING);
    let refused: std::io::Result<Output> = crate::lazy_runtime::run_on_runtime(async {
        return crate::child_watch::output_watched_within_for_a_stop(&mut shell("exit 0"), "a step", "a step", long_window_terms(), true, bound, false);
    })
    .expect("the runtime");
    assert!(refused.is_err(), "the synchronous face registers its pipes by entering the runtime, which is refused from inside it");
}

/// A step the landing waits for without a bound is awaited to its own end,
/// and its output is the whole of what it wrote
///  into the captured files.
#[test]
fn a_step_that_is_never_ended_is_awaited_to_its_own_end() {
    let output: Output = crate::lazy_runtime::run_on_runtime(async {
        return output_watched_within_for_a_stop_awaiting(&mut shell("printf landed; exit 0"), "a landing step", "a landing step", long_window_terms(), false, StepBound::Unbounded, false)
            .await;
    })
    .expect("the runtime")
    .expect("the step ran to its end");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()), "landed");
}

/// A run leaves the descriptors of this process as it found them:
/// a write end of a pipe kept open here would keep the reader from ever seeing
///  the end of the output.
#[test]
fn a_step_awaited_on_the_runtime_leaves_no_descriptor_open() {
    let bound: StepBound = StepBound::EndAt(crate::wait_ceilings::OBJECT_STEP_CEILING);
    // The first run builds the runtime and its reactor, whose own descriptors stay for the life of the process.
    crate::lazy_runtime::run_on_runtime(async {
        return output_watched_within_for_a_stop_awaiting(&mut shell("exit 0"), "the warm-up", "the warm-up", long_window_terms(), true, bound, false).await;
    })
    .expect("the runtime")
    .expect("the warm-up ran");
    let before: usize = open_descriptors();
    for _run in 0..2 {
        let output: Output = crate::lazy_runtime::run_on_runtime(async {
            return output_watched_within_for_a_stop_awaiting(&mut shell("exit 0"), "a dull step", "a dull step", long_window_terms(), true, bound, false).await;
        })
        .expect("the runtime")
        .expect("the step ran to its end");
        assert_eq!(output.status.code(), Some(0));
    }
    let after: usize = open_descriptors();
    assert_eq!(after, before, "the run left {} descriptors of its own open", after.saturating_sub(before));
}

/// A step whose input and output are files of its own runs on the blocking-pool-free path the same way:
/// it writes its marker,
/// ends with the code it chose,
/// and leaves the fixture for the next control.
#[test]
fn a_step_on_a_directory_of_its_own_runs_and_leaves_its_marker() {
    let root: PathBuf = fixture("child-watch-awaiting-unstartable");
    let marker: PathBuf = root.join("started");
    let bound: StepBound = StepBound::EndAt(crate::wait_ceilings::OBJECT_STEP_CEILING);
    let line: String = format!("touch '{}'", marker.display());
    let mut command: Command = Command::new("/bin/sh");
    command.arg("-c").arg(line.as_str()).stdin(Stdio::null());
    let refused: std::io::Result<Output> = crate::lazy_runtime::run_on_runtime(async {
        return output_watched_within_for_a_stop_awaiting(&mut command, "a step on a temporary directory", "a step on a temporary directory", long_window_terms(), true, bound, false)
            .await;
    })
    .expect("the runtime");
    let output: Output = refused.expect("the step ran to its end");
    assert_eq!(output.status.code(), Some(0));
    assert!(marker.exists(), "the step ran, so its marker is there");
    remove(root.as_path());
}

/// What:
///  Runs a step that outlives the bound it is given, releases it, and answers what the command said and whether the
///  step's end was collected.
/// Why:
///  Both controls of the left-running path need the same step and the same measure, and they differ only in the state
///  they put the runtime in.
///  A step that ends while this process lives and is not collected is a zombie of it, which
///  `process_tree::use_of` reports with the state the kernel holds it in.
async fn a_step_left_running_released_and_collected(step_name: &str, bound: StepBound) -> (std::io::Error, bool) {
    let directory: PathBuf = fixture(step_name);
    let pid_file: PathBuf = directory.join("pid");
    let release: PathBuf = directory.join("release");
    let script: PathBuf = directory.join("step.sh");
    std::fs::write(
        &script,
        format!("echo $$ > '{pid}'\nwhile [ ! -e '{release}' ]; do sleep 0.02; done\n", pid = pid_file.display(), release = release.display()),
    )
    .expect("the step");
    let mut command: Command = Command::new("/bin/sh");
    command.arg(script.as_path()).stdin(Stdio::null());
    let failure: std::io::Error = output_watched_within_for_a_stop_awaiting(&mut command, "a write", "a write", long_window_terms(), false, bound, false)
        .await
        .expect_err("the command stopped waiting for the step");
    // The step was started before the command stopped waiting, and it writes its number as it begins.
    let mut waited: u32 = 0;
    while !pid_file.exists() && waited < 250 {
        tokio::time::sleep(Duration::from_millis(20)).await;
        waited = waited.saturating_add(1);
    }
    let pid: i32 = std::fs::read_to_string(&pid_file).expect("the step recorded its process").trim().parse().expect("a process number");
    // The start time proves which process the number names, so a number that passed to another one is not read as the step.
    let started: u64 = crate::process_tree::use_of(pid).expect("the step runs as it waits").started;
    // The step ends now, and whoever was given it must collect that end.
    std::fs::write(&release, b"").expect("let the step end");
    let mut collected: bool = false;
    let mut looked: u32 = 0;
    while !collected && looked < 500 {
        match crate::process_tree::use_of(pid) {
            // The step is gone, or its number names a process that started later: this one's end was collected.
            None => collected = true,
            Some(seen) if seen.started != started => collected = true,
            // A process that is still there is the step, waiting as a zombie for a wait that never comes.
            Some(_still_there) => tokio::time::sleep(Duration::from_millis(20)).await,
        }
        looked = looked.saturating_add(1);
    }
    // The cleanup runs before the answers are used, so a failing control leaves nothing running.
    if !collected {
        // SAFETY: `kill` takes two integers and touches no memory; the number is this control's own step, and a zombie ignores it.
        let _ended = unsafe { libc::kill(pid, libc::SIGKILL) };
    }
    remove(directory.as_path());
    return (failure, collected);
}

/// What:
///  A step left running by a caller on the runtime is collected where the process's build of the runtime missed its
///  deadline and the runtime it built arrived afterwards.
/// Why:
///  This path asked how the runtime was first built,
/// so a late build was read as a process with no worker at all:
/// the child was
///  taken out of its owner and dropped, and nothing collected it when it ended
///  (the fifth review's finding 4).
///  The question is the one the rest of the branch asks,
/// whether a task can run now, which a late build answers yes to
///  (`a_control_stands_in_for_a_late_runtime`).
#[cfg(target_os = "linux")]
#[test]
fn a_step_left_running_on_the_runtime_is_collected_where_the_build_of_the_runtime_was_late() {
    let late: crate::lazy_runtime::LateRuntimeForTheControl = crate::lazy_runtime::a_control_stands_in_for_a_late_runtime();
    assert!(
        matches!(crate::lazy_runtime::built_flavor(), Some(crate::lazy_runtime::BuiltFlavor::CurrentThreadAfterDeadline)),
        "the control stands in for a build that missed its deadline"
    );
    assert!(crate::lazy_runtime::tasks_can_run(), "the runtime this process built can run a task, so the late build has workers");
    let (failure, collected): (std::io::Error, bool) = crate::lazy_runtime::run_on_runtime(a_step_left_running_released_and_collected(
        "child-watch-late-runtime",
        StepBound::LeaveRunning { after: Duration::from_millis(300), extra: Duration::from_millis(200) },
    ))
    .expect("the runtime");
    drop(late);
    assert!(failure.to_string().contains("left running"), "the command reports the step as left running: {failure}");
    assert!(collected, "the step that was left running on the runtime was never collected: nothing waited for it");
}

/// What:
///  A step left running where no task of this process can run is collected all the same.
/// Why:
///  With no worker the child is left running on purpose,
/// and it was taken out of its owner and dropped:
/// a step that
///  ended while this process lived stayed a zombie of it, which the reaper list exists to prevent
///  (the fifth review's finding 4, at the branch this control takes).
#[cfg(target_os = "linux")]
#[test]
fn a_step_left_running_where_no_task_can_run_is_collected_by_a_reaper() {
    let refused: crate::lazy_runtime::RefusalForTheControl = crate::lazy_runtime::a_control_refuses_every_task();
    assert!(!crate::lazy_runtime::tasks_can_run(), "the control stands in for a system that can run no task of this process");
    let (failure, collected): (std::io::Error, bool) = crate::lazy_runtime::run_on_runtime(a_step_left_running_released_and_collected(
        "child-watch-no-task-to-wait",
        StepBound::LeaveRunning { after: Duration::from_millis(300), extra: Duration::from_millis(200) },
    ))
    .expect("the runtime");
    drop(refused);
    assert!(failure.to_string().contains("left running"), "the command reports the step as left running: {failure}");
    assert!(collected, "the step that was left running where no task could run was never collected: nothing waited for it");
}
