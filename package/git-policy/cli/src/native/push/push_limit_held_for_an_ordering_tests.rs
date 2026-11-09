//! Controls for the hold a control may put on a run's limit:
//! no hold reaches a limit by time,
//! a hold with the ordering missing holds it,
//! a held ordering lets it pass,
//! a hold past its ceiling lets it pass,
//! and the guard ends the hold.

use super::*;

/// The limit a control asks for.
const LIMIT: Duration = Duration::from_millis(2500);

/// A hold nobody releases within the run.
const FOREVER: Duration = Duration::from_secs(3600);

/// An ordering that has not happened.
fn not_reached() -> bool {
    return false;
}

/// An ordering that has happened.
fn reached() -> bool {
    return true;
}

/// With no control installed a limit is reached by time,
/// as it was before the hold existed.
#[test]
fn no_hold_reaches_a_limit_by_time() {
    assert!(limit_has_passed(LIMIT, LIMIT));
    assert!(limit_has_passed(LIMIT * 2, LIMIT));
    assert!(!limit_has_passed(LIMIT - Duration::from_millis(1), LIMIT));
}

/// An ordering that has not happened holds the limit past the time it names.
#[test]
fn an_ordering_that_has_not_happened_holds_the_limit() {
    let _held = hold_limit_for_an_ordering(LimitHeldForAnOrdering { reached: not_reached, ceiling: FOREVER });
    assert!(!limit_has_passed(LIMIT * 100, LIMIT));
    assert!(!limit_has_passed(Duration::ZERO, LIMIT));
}

/// The same ordering,
/// once it has happened,
/// lets the limit pass on the clock it was given.
#[test]
fn an_ordering_that_has_happened_lets_the_limit_pass() {
    let _held = hold_limit_for_an_ordering(LimitHeldForAnOrdering { reached, ceiling: FOREVER });
    assert!(limit_has_passed(LIMIT, LIMIT));
    assert!(!limit_has_passed(LIMIT - Duration::from_millis(1), LIMIT));
}

/// A hold whose ceiling has passed lets the limit pass,
/// so a control whose ordering never happens reports its own failure instead of waiting for the watchdog.
#[test]
fn a_hold_past_its_ceiling_lets_the_limit_pass() {
    let _held = hold_limit_for_an_ordering(LimitHeldForAnOrdering { reached: not_reached, ceiling: Duration::ZERO });
    assert!(!limit_is_held());
    assert!(limit_has_passed(LIMIT, LIMIT));
}

/// The guard ends the hold with it,
/// so a later run on the same thread reaches its limit by time again.
#[test]
fn the_guard_ends_the_hold_with_it() {
    {
        let _held = hold_limit_for_an_ordering(LimitHeldForAnOrdering { reached: not_reached, ceiling: FOREVER });
        assert!(limit_is_held());
    }
    assert!(!limit_is_held());
    assert!(limit_has_passed(LIMIT, LIMIT));
}
