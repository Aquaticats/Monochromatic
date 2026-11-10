//! Controls for the one-shot watched run that ends at a ceiling (`StepBound::EndAt`):
//! its output is read through pipes on the
//!  runtime and comes back whole,
//! a call from inside the runtime is refused before a child starts,
//! no descriptor stays open
//!  after a run,
//! and the run no longer waits for the child with a blocking call of the standard library.
#![cfg(unix)]

use super::*;
use crate::child_class::ChildClass;
use crate::output_capture::STEP_OUTPUT_LIMIT;
use std::process::{Command, Output, Stdio};

/// More bytes than the captured steps keep (`STEP_OUTPUT_LIMIT` is the tail they keep),
/// so a run that kept only a tail would show.
const BEYOND_THE_KEPT_TAIL: usize = STEP_OUTPUT_LIMIT + STEP_OUTPUT_LIMIT / 4;

/// Terms of the Git-question class:
/// its window is far longer than any control here runs.
fn long_window_terms() -> WatchTerms {
    return WatchTerms::for_class(ChildClass::Object);
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
/// in order:
/// the one-shot run that ends at
///  a ceiling returns all the output,
/// as `Command::output` does,
/// and not the tail the captured files keep.
#[test]
fn a_step_that_writes_far_more_than_the_kept_tail_returns_every_byte_of_both_streams() {
    let line: String = format!(
        "head -c {BEYOND_THE_KEPT_TAIL} /dev/zero | tr '\\0' a; head -c {BEYOND_THE_KEPT_TAIL} /dev/zero | tr '\\0' b >&2; exit 3"
    );
    let output: Output = output_watched(&mut shell(line.as_str()), "a loud step", long_window_terms()).expect("the step ran to its end");
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(output.stdout.len(), BEYOND_THE_KEPT_TAIL);
    assert_eq!(output.stderr.len(), BEYOND_THE_KEPT_TAIL);
    assert!(output.stdout.iter().all(|byte| { return *byte == b'a'; }), "standard output holds only what the step wrote to it");
    assert!(output.stderr.iter().all(|byte| { return *byte == b'b'; }), "standard error holds only what the step wrote to it");
}

/// A call from inside the runtime is refused with an error before any child starts,
/// where waiting for the runtime from within it
///  would never end:
/// the marker the step would create is not there.
#[test]
fn a_call_from_inside_the_runtime_is_refused_and_starts_no_child() {
    let root: std::path::PathBuf = crate::test_support::fixture("child-watch-refused-inside-the-runtime");
    let marker: std::path::PathBuf = root.join("started");
    let line: String = format!("touch '{}'", marker.display());
    let inner: std::io::Result<Output> = crate::lazy_runtime::run_on_runtime(async {
        return output_watched(&mut shell(line.as_str()), "a step started from inside the runtime", long_window_terms());
    })
    .expect("the runtime");
    let started: bool = marker.exists();
    crate::test_support::remove(root.as_path());
    assert!(inner.is_err(), "a call from inside the runtime must be refused");
    assert!(!started, "a refused call must not have started the step");
}

/// A run,
/// finished or ended at its bound,
/// leaves the descriptors of this process as it found them:
/// a write end of a pipe kept
///  open here would keep the reader from ever seeing the end of the output.
#[test]
fn a_run_leaves_no_descriptor_open() {
    // The first run builds the runtime and its reactor, whose own descriptors stay for the life of the process.
    output_watched(&mut shell("exit 0"), "the warm-up", long_window_terms()).expect("the warm-up ran");
    let before: usize = open_descriptors();
    for _ in 0..3 {
        output_watched(&mut shell("printf out; printf err >&2"), "a run to its end", long_window_terms()).expect("the run");
    }
    let sink: crate::test_support::SharedSink = crate::test_support::SharedSink::default();
    let terms: WatchTerms = WatchTerms {
        limit: Duration::from_millis(300),
        cadence: crate::wait_progress::Cadence { grace: Duration::from_millis(60), interval: Duration::from_millis(60) },
        notices: Box::new(sink),
    };
    output_watched(&mut shell("sleep 30"), "a run ended at its window", terms).expect_err("a silent step is ended");
    assert_eq!(open_descriptors(), before);
}

/// The one-shot run that ends at a ceiling does not wait for its child with `wait_with_output` or any other blocking call of the
///  standard library:
/// the wait is the runtime's.
#[test]
fn the_one_shot_run_that_ends_at_a_ceiling_no_longer_blocks_a_thread_on_the_child() {
    let source: &str = include_str!("child_watch.rs");
    let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    assert!(!code.contains("wait_with_output"), "the run still waits for its child with `wait_with_output`");
}
