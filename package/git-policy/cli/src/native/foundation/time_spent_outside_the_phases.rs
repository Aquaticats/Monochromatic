//! What:
//!  The two costs a wrapped commit pays outside its phases,
//!  and the writer that measures one of them:
//!  the removal of the commit's scratch tree,
//!  and writing to standard error,
//!  which waits for whoever reads it.
//! Why:
//!  The commit timing record's `otherMs` is what no phase covers,
//!  and the record could not say how much of it was the scratch tree's removal
//!  and how much was the wrapper writing what the person reads
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
//!  "What this could not determine").
//!  Both are measured here,
//!  one clock read around each,
//!  and written in the record as `scratchRemovalMs`,
//!  `removalsThatDidNotFinish` and `stderrMs`.
//!  The clocks are read only where the command already writes or removes,
//!  so the measurement starts no process and flushes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! let scratch = 0, stderr = 0, unfinished = 0;
//! export const noteScratchRemoval = (took: number, finished: boolean) => { scratch += took; if (!finished) unfinished++; };
//! export const noteStderrWrite = (took: number) => { stderr += took; };
//! export const outsideThePhases = () => ({ scratch, stderr, unfinished });
//! ```

/// The counters,
/// shared by the threads of one command.
use std::sync::atomic::{AtomicU64, Ordering};
/// Writing text out,
/// which is what the measured stream does.
use std::io::Write;
/// A span of time.
use std::time::Duration;

/// Microseconds a command spent removing scratch trees.
static SCRATCH_REMOVAL_MICROSECONDS: AtomicU64 = AtomicU64::new(0);

/// Removals of a scratch tree that ended without finishing within their ceiling.
static REMOVALS_THAT_DID_NOT_FINISH: AtomicU64 = AtomicU64::new(0);

/// Microseconds a command spent writing to standard error.
static STANDARD_ERROR_MICROSECONDS: AtomicU64 = AtomicU64::new(0);

/// What:
///  The two costs a command paid outside its phases.
/// Why:
///   The commit timing record writes them beside `otherMs`,
///  so a reading can say which of the two a slow `otherMs` was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OutsideThePhases = { scratchRemoval: number; standardError: number; removalsThatDidNotFinish: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OutsideThePhases {
    /// How long removing scratch trees took.
    pub scratch_removal: Duration,
    /// How many of those removals ended without finishing within their ceiling.
    pub removals_that_did_not_finish: u64,
    /// How long writing to standard error took.
    pub standard_error: Duration,
}

/// One span of time added to a counter,
/// in whole microseconds,
/// held at the counter's end rather than wrapped around it.
fn add(counter: &AtomicU64, took: Duration) {
    let microseconds: u64 = u64::try_from(took.as_micros()).unwrap_or(u64::MAX);
    // A saturated counter is a figure that cannot be right; a record is a diagnostic and never fails a command, so the sum is
    //  held at the counter's end rather than wrapped around it.
    let mut held: u64 = counter.load(Ordering::Relaxed);
    loop {
        let next: u64 = held.saturating_add(microseconds);
        match counter.compare_exchange_weak(held, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_added) => break,
            Err(current) => held = current,
        }
    }
}

/// Whole milliseconds of a microsecond counter,
/// rounded down as every other figure of the record is.
fn milliseconds(counter: &AtomicU64) -> u64 {
    return counter.load(Ordering::Relaxed) / 1000;
}

/// What:
///  Adds one scratch removal to the command's record.
/// Why:
///   Every removal of a scratch tree reports here,
///  whether it finished,
///  hit its ceiling or failed,
///  so a command that pays one says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteScratchRemoval(elapsed, true);
/// ```
pub fn note_scratch_removal(took: Duration, finished: bool) {
    add(&SCRATCH_REMOVAL_MICROSECONDS, took);
    if !finished {
        let _count: u64 = REMOVALS_THAT_DID_NOT_FINISH.fetch_add(1, Ordering::Relaxed);
    }
}

/// What:
///  Adds one write to standard error to the command's record.
/// Why:
///   A write to a pipe blocks until the reader takes it,
///  so a slow reader is time the command spent outside every phase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteStderrWrite(elapsed);
/// ```
pub fn note_standard_error(took: Duration) {
    add(&STANDARD_ERROR_MICROSECONDS, took);
}

/// What:
///  What the command paid outside its phases,
///  in whole milliseconds.
/// Why:
///   The record is written once for a command,
///  and the process ends with it,
///  so the counters are read and not emptied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { scratch, stderr, unfinished } = outsideThePhases();
/// ```
pub fn outside_the_phases() -> OutsideThePhases {
    return OutsideThePhases {
        scratch_removal: Duration::from_millis(milliseconds(&SCRATCH_REMOVAL_MICROSECONDS)),
        removals_that_did_not_finish: REMOVALS_THAT_DID_NOT_FINISH.load(Ordering::Relaxed),
        standard_error: Duration::from_millis(milliseconds(&STANDARD_ERROR_MICROSECONDS)),
    };
}

/// What:
///  Standard error,
///  or anything else that takes text,
///  with every write and flush added to the command's record.
/// Why:
///   The commit's own lines are written in several places:
///  the policy events before and after landing,
///  the stop messages and the hand-over's notices.
///  One writer measures them all,
///  and for standard error it is the same stream the person reads,
///  so nothing the command prints changes.
///  The stream it holds is a parameter,
///  which is what lets a control give it one that spends time writing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MeasuredStream { write(text) { const began = now(); this.inner.write(text); noteStderrWrite(now() - began); } }
/// ```
#[derive(Debug, Default)]
pub struct MeasuredStandardError<W: Write> {
    /// The stream the bytes go to.
    inner: W,
}

impl MeasuredStandardError<std::io::Stderr> {
    /// The measured process's standard error,
    /// which is what a commit writes what the person reads to.
    pub fn new() -> Self {
        return Self { inner: std::io::stderr() };
    }
}

impl<W: Write> MeasuredStandardError<W> {
    /// The measured stream over another one,
    /// which a control uses to make a write spend time.
    pub fn over(inner: W) -> Self {
        return Self { inner };
    }
}

impl<W: Write> Write for MeasuredStandardError<W> {
    /// Writes the whole buffer,
    /// timed.
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let began: std::time::Instant = std::time::Instant::now();
        let written: std::io::Result<usize> = self.inner.write(buffer);
        note_standard_error(began.elapsed());
        return written;
    }

    /// Flushes the stream,
    /// timed.
    fn flush(&mut self) -> std::io::Result<()> {
        let began: std::time::Instant = std::time::Instant::now();
        let flushed: std::io::Result<()> = self.inner.flush();
        note_standard_error(began.elapsed());
        return flushed;
    }
}

/// The record's fields and the writer are controlled in the test module.
#[cfg(test)]
#[path = "time_spent_outside_the_phases_tests.rs"]
mod tests;
