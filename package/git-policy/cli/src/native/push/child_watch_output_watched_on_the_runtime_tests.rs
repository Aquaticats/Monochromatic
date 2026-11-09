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
