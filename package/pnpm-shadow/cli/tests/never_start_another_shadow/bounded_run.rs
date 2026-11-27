//! What:
//! The deadline a `tests/never_start_another_shadow.rs` run gets:
//! the child is polled until it ends,
//! and killed when the bound passes,
//! so a test of a start loop fails instead of hanging the suite.
//! Why:
//! The bug these tests describe is two shadows replacing each other without end;
//! a test that waited for that run would never return.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Outcome = { kind: 'finished'; output: Output } | { kind: 'looped' };
//! function runBounded(command: Command): Outcome { /* poll until exit or LOOP_BOUND, then kill */ }
//! ```

/// Imports the child-process builder,
/// the child handle and the captured output.
use std::process::{
    Child,
    Command,
    Output,
    Stdio,
};
/// Imports the sleep that spaces the polls.
use std::thread::sleep;
/// Imports the two time types:
/// the deadline and the point it is measured from.
use std::time::{
    Duration,
    Instant,
};

/// How long a run may take before it counts as a loop.
/// A run that ends takes a few milliseconds;
/// the loop it replaces never ends.
/// The bound is far above the first and below the patience of a test run,
/// and it only matters in the failing case.
const LOOP_BOUND: Duration = Duration::from_secs(5);

/// How often a run is polled while it has not ended.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// What a bounded run did.
pub(crate) enum Outcome {
    /// It ended by itself.
    Finished(Output),
    /// It was still running at the bound and was killed.
    Looped,
}

/// Runs a prepared command with the deadline `LOOP_BOUND`.
pub(crate) fn run_bounded(mut command: Command) -> Outcome {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child: Child = command.spawn().expect("start the shadow");
    let started: Instant = Instant::now();
    loop {
        if child.try_wait().expect("poll the shadow").is_some() {
            return Outcome::Finished(child.wait_with_output().expect("collect the output"));
        }
        if started.elapsed() >= LOOP_BOUND {
            child.kill().expect("kill the looping shadow");
            child.wait().expect("reap the looping shadow");
            return Outcome::Looped;
        }
        sleep(POLL_INTERVAL);
    }
}

/// The output of a run that must have ended,
/// or a failure that says it looped.
pub(crate) fn finished(outcome: Outcome) -> Output {
    match outcome {
        Outcome::Finished(output) => return output,
        Outcome::Looped => panic!(
            "the run was still going after {LOOP_BOUND:?}: the shadows are starting each other"
        ),
    }
}
