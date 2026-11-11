//! Controls for the housekeeping slice of one round:
//! a round that has not spent its bound may start housekeeping,
//! and one whose bound is over may not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(new HousekeepingOfOneRound(ZERO).spent()).toBe(true); expect(new HousekeepingOfOneRound(THE_BOUND).mayStart()).toBe(true);
//! ```

use super::*;

/// A slice that has just begun may start housekeeping,
/// and reports the bound it was given.
#[test]
fn a_slice_that_has_not_spent_its_bound_may_start() {
    let slice: HousekeepingOfOneRound = HousekeepingOfOneRound::starting_now(HOUSEKEEPING_OF_ONE_ROUND);
    assert!(slice.may_start(), "a round that has just begun its housekeeping may start one");
    assert!(!slice.spent(), "nothing is spent at the first instant");
    assert_eq!(slice.bound(), HOUSEKEEPING_OF_ONE_ROUND, "the slice reports the bound it was given");
    assert!(slice.spent_ms() < HOUSEKEEPING_OF_ONE_ROUND.as_millis(), "a round that has just begun has not spent its whole bound");
}

/// A slice whose bound is already over may not start housekeeping,
/// and says so by both questions.
#[test]
fn a_slice_whose_bound_is_over_may_not_start() {
    let slice: HousekeepingOfOneRound = HousekeepingOfOneRound::starting_now(Duration::ZERO);
    assert!(slice.spent(), "a bound of nothing is spent at once");
    assert!(!slice.may_start(), "and nothing may be started under it");
}

/// The pause a deferred round asks for is inside the window a client waits in:
/// a later round comes before the service would otherwise look idle.
#[test]
fn the_look_again_pause_is_shorter_than_the_window_a_client_waits_in() {
    use crate::service_protocol::HEARTBEAT_SILENCE;
    assert!(LOOK_AGAIN_AFTER_DEFERRING <= HOUSEKEEPING_OF_ONE_ROUND, "a deferred round is woken before its bound would let it start again");
    assert!(LOOK_AGAIN_AFTER_DEFERRING <= HEARTBEAT_SILENCE, "the wake is inside the window a waiting client hears nothing in");
}
