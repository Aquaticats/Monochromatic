//! What:
//!  The one job a runner holds at a time,
//! as the runner knows it:
//! nothing,
//! one the caller waits for,
//! or one the caller stopped waiting for and that still runs,
//! and what a contained job answered.
//! Why:
//!  Admission,
//! the caller's give-up and the worker's end are each one atomic change of this state,
//! so two callers cannot both be admitted,
//! and a worker that panics releases it by a guard that runs on unwinding.
//! The production runner is `RUNNER`;
//! a control makes its own,
//! so the controls do not see one another's jobs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Contained<T> = { kind: 'finished'; value: T } | { kind: 'panicked' } | { kind: 'no-thread'; sentence: string } | { kind: 'ceiling'; afterMs: number } | { kind: 'busy' };
//! // class Containment { outstanding(): number }
//! ```

/// How long the runner's caller may wait,
/// and how long a job the caller gave up on may still run.
use std::time::Duration;

/// What:
///  How a contained job ended.
/// Why:
///   The caller words a panic and a ceiling differently:
/// one is a defect,
/// the other is a review that ran as long as the service lets
///  one run;
/// and a thread the system refused is neither,
/// so it carries its own sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Contained<T> = { kind: 'finished'; value: T } | { kind: 'panicked' } | { kind: 'no-thread'; sentence: string } | { kind: 'ceiling'; afterMs: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Contained<T> {
    /// The job returned its value.
    Finished(T),
    /// The job panicked.
    Panicked,
    /// The job's thread could not be started,
    /// with the sentence that says what refused it:
    /// nothing ran,
    /// so this is no defect of the job.
    NoThread(String),
    /// The job was still running when the ceiling passed;
    /// the caller stopped waiting and the job's thread is left to end with the
    ///  process,
    /// or when the step it is stuck in returns.
    /// Until then the runner counts it as outstanding.
    CeilingReached(Duration),
    /// The job was not started:
    /// an earlier one that the caller stopped waiting for is still running and may hold what this one
    ///  needs (the review log),
    /// so no second thread is added behind it.
    Busy,
}

/// What:
///  The one job a runner has at a time,
/// as the runner knows it:
/// nothing,
/// one the caller waits for,
/// or one the caller stopped waiting
///  for and that still runs.
/// Why:
///   Admission,
/// the caller's give-up and the worker's end are each one atomic change of this state,
/// so two callers cannot both be
///  admitted (sol-2 round 2,
/// P2),
/// and a worker that panics releases it by a guard that runs on unwinding (sol-2 round 2,
/// P1).
/// The
///  production runner is `RUNNER`;
/// a control makes its own,
/// so the controls do not see one another's jobs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Containment { state: 'idle' | 'running' | 'abandoned' }
/// ```
pub struct Containment {
    /// The generation of the job in the high bits and `IDLE`,
    /// `RUNNING` or `ABANDONED` in the low two.
    /// A change names the generation
    ///  it expects,
    /// so a caller that gave up on job 7 can never change the state of job 8.
    state: std::sync::atomic::AtomicU64,
    /// How long the caller waits for the value of a job that ended as the caller gave up,
    /// when the value does not come:
    /// the short
    ///  extra wait of the owner's "7 s (Recommended)" (`wait_ceilings.rs`,
    /// `SHORT_EXTRA_WAIT`).
    /// A job that released the runner has
    ///  not necessarily sent its value,
    /// so the wait is bounded (sol-1 round 3,
    /// P1 D).
    pub(super) handoff_wait: Duration,
    /// A control holds this to keep a worker between its release of the runner and its send of the value.
    #[cfg(test)]
    pub(super) handoff_gate: std::sync::Mutex<()>,
}

/// No job.
pub(super) const IDLE: u64 = 0;
/// A job the caller waits for.
pub(super) const RUNNING: u64 = 1;
/// A job the caller stopped waiting for,
/// still running.
pub(super) const ABANDONED: u64 = 2;
/// The bits of the state that hold the status.
pub(super) const STATUS: u64 = 3;

/// The runner of the service's reviews.
pub static RUNNER: Containment = Containment::new();

impl Containment {
    /// A runner with no job.
    pub const fn new() -> Self {
        return Self::with_handoff_wait(crate::wait_ceilings::SHORT_EXTRA_WAIT);
    }

    /// A runner whose caller waits `handoff_wait` for the value of a job that ended as the caller gave up.
    pub const fn with_handoff_wait(handoff_wait: Duration) -> Self {
        return Self {
            state: std::sync::atomic::AtomicU64::new(IDLE),
            handoff_wait,
            #[cfg(test)]
            handoff_gate: std::sync::Mutex::new(()),
        };
    }

    /// How many jobs the caller stopped waiting for are still running:
    /// zero or one.
    pub fn outstanding(&self) -> usize {
        return usize::from(
            self.state.load(std::sync::atomic::Ordering::SeqCst) & STATUS == ABANDONED,
        );
    }

    /// Admit a job when there is none:
    /// the new generation,
    /// or nothing.
    pub(super) fn admit(&self) -> Option<u64> {
        use std::sync::atomic::Ordering;
        let seen: u64 = self.state.load(Ordering::SeqCst);
        if seen & STATUS != IDLE {
            return None;
        }
        let generation: u64 = (seen >> 2) + 1;
        return match self.state.compare_exchange(
            seen,
            (generation << 2) | RUNNING,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => Some(generation),
            Err(_) => None,
        };
    }

    /// Move job `generation` from `from` to `to`;
    /// false when the state is not that job's in that status any more.
    pub(super) fn change(
        &self,
        generation: u64,
        from: u64,
        to: u64,
    ) -> bool {
        use std::sync::atomic::Ordering;
        return self
            .state
            .compare_exchange(
                (generation << 2) | from,
                (generation << 2) | to,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok();
    }
}

impl Default for Containment {
    fn default() -> Self {
        return Self::new();
    }
}
