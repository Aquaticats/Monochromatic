//! What:
//!  Whether the unit-test run is moving:
//! every task of this process and of the processes it started,
//! their states,
//! their
//!  processor time and the bytes they moved,
//! and the clock that ends a run which has shown none of them for a window.
//! Why:
//!  A wall-clock limit cannot tell a stuck run from a slow one.
//! The tests that copy a checkout into btrfs subvolumes spend nearly all of their time
//!  waiting in the kernel (measured 2026-11-08 on this host,
//! `doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`):
//! 43 of 47 s of one
//!  of them alone,
//! and 64 of 69 s of another while its family ran,
//! in state `D` inside the btrfs transaction and log
//!  commit waits,
//! so the run passed a 120 s wall clock as soon as the disk
//!  was busy and the failures followed the host's load instead of the code.
//! What a run that is stuck looks like instead is a run that shows nothing:
//! no task able to run,
//!  no task waiting for the disk,
//!  no processor time and no bytes moved,
//!  and no process or task started.
//! A figure that fell says a task ended or could not be read,
//! and neither is the run working:
//! a look adds up the tasks it found,
//! and a task that ends takes every byte and tick it moved out of those totals,
//! so the totals can fall while a task that stayed moved.
//! Each task's figures are therefore compared with the ones that task showed before,
//! so a task whose own figures grew is work,
//! and a task a look finds where the look before found none is a task that started.
//!  Those are the signs the wrapper's own wait for a child process reads (`push_progress.rs`),
//! so a test run
//!  and a shipped wait are judged alike.
//! A run that keeps moving is left alone,
//! and a run that moves for ever is
//!  ended by the test runner's own per-test limit (`.config/nextest.toml`,
//! `slow-timeout` 60 s times
//!  `terminate-after` 5),
//! so nothing waits without a bound.
//! A look that cannot be read (a task that ended while it was read,
//!  or a process out of file descriptors)
//!  is neither a sign of work nor proof that the run stopped:
//! the clock keeps running through it,
//! so a run
//!  whose counters never come back still ends,
//! and a later look that reads the run counts again.
//! Every reading a look takes is counted,
//! and the next look of that thread takes exactly them off,
//! so a reader's
//!  own readings cannot make the run look busy at every look.
//! What a reader moves that is not a reading is the run's work,
//!  as the disk work of a wait's predicate is,
//! so a thread that reads this run must not write while it watches.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const judge = new Judge(windowMs); setInterval(() => judge.observe(elapsedMs, lookAt(process.pid)), SAMPLE_MS);
//! ```

/// What:
///  The threads that read this run,
///  and how many waits each is inside.
/// Why:
///   Every reader registers,
/// so no look counts another reader's readings as the run's work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { begin, threads } from './run_progress_readers'
/// ```
#[path = "run_progress_readers.rs"]
mod readers;

/// What:
///  One look at the run,
///  and the tallies it adds up over every task.
/// Why:
///   The judge compares one whole reading of the run with the one before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { look } from './run_progress_one_look'
/// ```
#[path = "run_progress_one_look.rs"]
mod one_look;

/// What:
///  Why one run was ended,
///  in the words a person reads.
/// Why:
///   A stuck test,
/// unreadable counters and a system that counts no work end for different reasons.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { Ending } from './run_progress_ending'
/// ```
#[path = "run_progress_ending.rs"]
mod ending;

/// What:
///  The clock that compares each look with the one before it.
/// Why:
///   The ending is the caller's decision,
/// so controls can drive the judgment with made-up looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { Judge } from './run_progress_judge'
/// ```
#[path = "run_progress_judge.rs"]
mod judge;

/// What:
///  The watchdog thread,
/// the look a control takes now,
/// and the count of looks one thread took.
/// Why:
///   A stuck run ends its own process,
/// and a control's wait is judged by the same evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import { watch, lookNow, looksTakenByThisThread } from './run_progress_watchdog'
/// ```
#[path = "run_progress_watchdog.rs"]
mod watchdog;

/// What one look found of each task,
/// and the two numbers that name a task.
#[path = "run_progress_look_figures.rs"]
mod look_figures;

/// What:
///  The figure one task is named by.
/// Why:
///  The controls of the look name a task the way a look does.
pub(crate) use look_figures::task_number;
/// What:
///  The look the judge compares,
///  and the figures of one task it holds.
/// Why:
///  Callers and controls hand a look to the judge.
pub(crate) use look_figures::{
    Look,
    TaskFigures,
};

/// What:
///  A thread registered as a reader of this run,
///  for as long as the value lives.
/// Why:
///  A wait registers once and every look taken meanwhile leaves its readings out.
pub(crate) use readers::Reading;
/// What:
///  The threads a look leaves out.
/// Why:
///  The controls check that every reader of the process is left out.
pub(crate) use readers::reading_threads;

/// What:
///  Why one run was ended.
/// Why:
///  Callers and controls both read it.
pub(crate) use ending::Ending;

/// What:
///  The clock of the run.
/// Why:
///  Callers and controls both drive it.
pub(crate) use judge::Judge;

/// What:
///  The watchdog,
///  one look a control takes now,
///  and how many looks a thread took.
/// Why:
///  The fixtures arm the watchdog,
/// and a control measures its own wait by the looks it took.
pub(crate) use watchdog::{
    look_now,
    looks_taken_by_this_thread,
    watch,
};

/// Controls for the judgment,
/// the look and the ending.
#[path = "run_progress_tests.rs"]
mod tests;
