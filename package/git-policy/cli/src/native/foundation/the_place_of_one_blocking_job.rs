//! What:
//!  The place of the process's bound on blocking jobs that one job holds,
//!  shared between the frame that waits for the job and the job itself,
//!  so exactly one of them gives it back.
//! Why:
//!  Two things are true at once and pull in opposite directions
//! (`blocking_jobs_within_the_budget.rs`):
//!  a caller who stops waiting must not keep a place,
//! because a job the system refused a thread for keeps it queued for the life of the process and later jobs
//!  would wait for that place for ever;
//! and every job that runs must hold a place,
//! because the bound is the process's ceiling on blocking work.
//!  The frame cannot know which of the two a refused job is,
//! so the place lives in an object both can reach:
//!  the frame gives it back when it takes the claim on the job's work,
//! and a job that took the claim first keeps it until its work ends.
//! The slot is taken once, so a second give-back does nothing.
//!  One small lock covers that take,
//! never held across a wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const place = thePlaceOfOneJob(permit); // one of the two gives it back, once
//! ```

use std::sync::{Arc, Mutex};
use tokio::sync::OwnedSemaphorePermit;

/// What:
///  The place one blocking job holds of the process's bound.
/// Why:
///  See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ThePlaceOfOneJob { private place: Permit | undefined; }
/// ```
pub(crate) struct ThePlaceOfOneJob {
    /// The place,
    /// until one of the two holders gives it back.
    place: Mutex<Option<OwnedSemaphorePermit>>,
}

impl ThePlaceOfOneJob {
    /// The place of one job,
    /// shared by the frame that waits for it and the job itself.
    pub(crate) fn holding(place: OwnedSemaphorePermit) -> Arc<ThePlaceOfOneJob> {
        return Arc::new(ThePlaceOfOneJob { place: Mutex::new(Some(place)) });
    }

    /// What:
    ///  Gives the place back to the process,
    ///  once.
    /// Why:
    ///  The frame gives it back when the job will never run,
    /// and the job gives it back when its work ends;
    ///  the second of the two finds nothing to give back.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// place.giveItBack();
    /// ```
    pub(crate) fn give_it_back(&self) {
        let given = self.place.lock().expect("lock").take();
        drop(given);
    }
}

/// What:
///  Holds one job's place for as long as the value lives,
///  giving it back when the work it covers ends.
/// Why:
///  A job that runs after its frame stopped waiting must still hold the place,
/// and an unwinding work must give it back too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using held = aPlaceGivenBackWhenItEnds(place); work();
/// ```
pub(crate) struct APlaceGivenBackWhenItEnds(Arc<ThePlaceOfOneJob>);

impl APlaceGivenBackWhenItEnds {
    /// The place given back when this value is dropped.
    pub(crate) fn covering(place: Arc<ThePlaceOfOneJob>) -> APlaceGivenBackWhenItEnds {
        return APlaceGivenBackWhenItEnds(place);
    }
}

impl Drop for APlaceGivenBackWhenItEnds {
    fn drop(&mut self) {
        self.0.give_it_back();
    }
}

/// Place controls stay out of the release executable.
#[cfg(test)]
#[path = "the_place_of_one_blocking_job_tests.rs"]
mod tests;
