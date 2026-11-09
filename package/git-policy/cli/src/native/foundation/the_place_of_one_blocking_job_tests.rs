//! The place of one blocking job:
//! whichever of the frame and the job gives it back first is the one that does,
//! and the other finds nothing.

use super::*;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// A place of a bound of one,
/// and the semaphore it came from,
/// so a control can count the places free.
fn one_place_of_a_bound() -> (Arc<Semaphore>, OwnedSemaphorePermit) {
    let bound: Arc<Semaphore> = Arc::new(Semaphore::new(1));
    let place: OwnedSemaphorePermit = Arc::clone(&bound).try_acquire_owned().expect("the place is free");
    return (bound, place);
}

/// The frame gives the place back when the job will never run,
/// and the job's own give-back then finds nothing.
#[test]
fn the_frame_gives_the_place_back_once_and_the_job_finds_nothing() {
    let (bound, place) = one_place_of_a_bound();
    let held: Arc<ThePlaceOfOneJob> = ThePlaceOfOneJob::holding(place);
    assert_eq!(bound.available_permits(), 0, "the place is held");
    held.give_it_back();
    assert_eq!(bound.available_permits(), 1, "the frame gave the place back");
    held.give_it_back();
    assert_eq!(bound.available_permits(), 1, "a second give-back does not free a place twice");
}

/// A job that took the claim first holds its place until its work ends,
/// and nothing else can take it back sooner.
#[test]
fn the_job_holds_the_place_until_its_work_ends() {
    let (bound, place) = one_place_of_a_bound();
    let held: Arc<ThePlaceOfOneJob> = ThePlaceOfOneJob::holding(place);
    {
        let _kept: APlaceGivenBackWhenItEnds = APlaceGivenBackWhenItEnds::covering(Arc::clone(&held));
        assert_eq!(bound.available_permits(), 0, "the job holds the place while it works");
    }
    assert_eq!(bound.available_permits(), 1, "the place comes back when the work ends");
}

/// A work that panics gives the place back as it unwinds.
#[test]
fn a_work_that_panics_gives_the_place_back() {
    let (bound, place) = one_place_of_a_bound();
    let held: Arc<ThePlaceOfOneJob> = ThePlaceOfOneJob::holding(place);
    let unwound = std::panic::catch_unwind({
        let for_the_panic = Arc::clone(&held);
        move || {
            let _kept: APlaceGivenBackWhenItEnds = APlaceGivenBackWhenItEnds::covering(for_the_panic);
            panic!("the work panicked");
        }
    });
    assert!(unwound.is_err(), "the work panicked");
    assert_eq!(bound.available_permits(), 1, "the place came back as the work unwound");
}
