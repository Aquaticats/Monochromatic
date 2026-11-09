//! Controls for what a landing batch tells its requests about their place.

use super::*;

/// The threshold of a control:
/// the shipped one is measured in `wait_ceilings.rs`,
/// and its value plays no part in what a batch says.
const THRESHOLD: Duration = crate::wait_ceilings::WAITING_NOTICE_AFTER;

/// A batch of one whose request waited says it is next,
/// which is the place `positions` never held in the incident this answers:
/// a lone request was told nothing because only the requests after a batch's
///  first were told.
#[test]
fn a_lone_request_that_waited_is_told_it_is_next() {
    assert_eq!(place_notices(1, THRESHOLD, THRESHOLD), vec![(1, 1)]);
}

/// A batch that begins at once says nothing of its first request,
/// so a commit the service lands at once carries no new line.
#[test]
fn a_lone_request_that_waited_for_nothing_is_told_nothing() {
    assert_eq!(place_notices(1, THRESHOLD / 4, THRESHOLD), Vec::<(usize, usize)>::new());
}

/// Every request of a batch but the first is told how many commits go before it,
/// however short the wait:
/// that is what the service said before this change and still says.
#[test]
fn a_batch_tells_each_request_after_the_first_how_many_go_before_it() {
    assert_eq!(place_notices(3, THRESHOLD / 4, THRESHOLD), vec![(2, 3), (3, 3)]);
}

/// The first request of a batch that waited is told its place with the rest,
/// so no request of a batch is left without one.
#[test]
fn the_first_request_of_a_batch_that_waited_is_told_with_the_rest() {
    assert_eq!(place_notices(3, THRESHOLD, THRESHOLD), vec![(1, 3), (2, 3), (3, 3)]);
}
