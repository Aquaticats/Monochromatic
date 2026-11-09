//! What:
//!  A one-shot claim on the work of a job,
//! taken either by the job before its first effect or by the reporter that says the job never started,
//! and never by both.
//! Why:
//!  The blocking pool queues a job before it asks the operating system for a thread
//! (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
//! `spawn_task`:
//! the task is pushed at line 624 and a thread is asked for only at line 628),
//!  so a caller told "the system refused a thread" cannot conclude that the job will never run:
//! the pool's next thread runs what it queued.
//! A report that says "the work could not be started, so the file is as it was" is therefore a promise
//! about the person's data,
//! and only a claim can keep it:
//! whichever of the two takes the claim first decides,
//! so a job reported as never started can never touch the file afterwards,
//! and a job that took the claim first makes the reporter say that its result is not known.
//! The claim is one atomic exchange,
//! so nothing waits and no lock is taken.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = new Claim(); if (claim.theJobTakesIt()) { work(); } else { /* the report gave up on it */ }
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// Nothing has taken the claim yet.
const FREE: u8 = 0;

/// The job took the claim, so it runs or has run.
const THE_JOB: u8 = 1;

/// The reporter took the claim, so the job must not run.
const THE_REPORTER: u8 = 2;

/// What:
///  The claim on one job's work,
/// shared between the job and the reporter that may give up on it.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheClaimOnAJobsWork { private takenBy: 'free' | 'job' | 'reporter' = 'free'; }
/// ```
pub struct TheClaimOnAJobsWork {
    /// Which of the two took it first.
    taken_by: AtomicU8,
}

impl TheClaimOnAJobsWork {
    /// A claim nobody has taken.
    pub fn new() -> Arc<TheClaimOnAJobsWork> {
        return Arc::new(TheClaimOnAJobsWork { taken_by: AtomicU8::new(FREE) });
    }

    /// What:
    ///  The job takes the claim,
    ///  before its first effect.
    /// Why:
    ///  False means the reporter has already given up on this job,
    ///  and the job must do nothing at all:
    /// the report has told the person that the work did not run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (!claim.theJobTakesIt()) { return withoutDoingAnything(); }
    /// ```
    pub fn the_job_takes_it(&self) -> bool {
        return self.taken_by.compare_exchange(FREE, THE_JOB, Ordering::SeqCst, Ordering::SeqCst).is_ok();
    }

    /// What:
    ///  The reporter takes the claim,
    ///  after it was told the job could not be started.
    /// Why:
    ///  True means the job had not taken the claim,
    ///  so the job can never do this work now and the report may say it did not run.
    /// False means the job took it first,
    ///  so the job runs or has run and nothing here can say what it did.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const neverStarted = claim.theReporterTakesIt(); // false: say the result is not known instead
    /// ```
    pub fn the_reporter_takes_it(&self) -> bool {
        return self.taken_by.compare_exchange(FREE, THE_REPORTER, Ordering::SeqCst, Ordering::SeqCst).is_ok();
    }
}

/// Claim controls stay out of the release executable.
#[cfg(test)]
#[path = "the_claim_a_job_takes_before_running_it_tests.rs"]
mod tests;
