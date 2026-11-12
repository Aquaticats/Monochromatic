//! Controls for the one budget a command gives the commit service to answer.

use super::*;

/// What:
///  A budget shortens a step's own bound to what is left of it.
/// Why:
///  The whole exchange is inside one budget,
///  so a step may not keep a bound that outlasts it.
#[test]
fn a_budget_shortens_each_steps_own_bound() {
    let patience: Patience = Patience::of(Duration::from_secs(2));
    let shorter: Duration = patience.bound(Duration::from_secs(5));
    assert!(shorter <= Duration::from_secs(2) && shorter > Duration::from_millis(1_900), "{shorter:?}");
    assert_eq!(patience.bound(Duration::from_micros(500)), Duration::from_millis(1), "a bound below the shortest wait the system takes is raised to it");
}

/// What:
///  A complete message starts the wait for the next one again.
/// Why:
///  The budget is a budget of silence:
///  a service that keeps saying what it does is waited for as long as it works,
///  which is what a landing that takes minutes needs.
#[test]
fn a_complete_message_starts_the_wait_for_the_next_one_again() {
    let mut patience: Patience = Patience::of(Duration::from_millis(200));
    std::thread::sleep(Duration::from_millis(150));
    let before: Duration = patience.left();
    assert!(before < Duration::from_millis(120), "the budget runs down with nothing said: {before:?}");
    patience.heard();
    let after: Duration = patience.left();
    assert!(after > before && after > Duration::from_millis(150), "the message started the wait again: {after:?}");
}

/// What:
///  A spent budget asks for the shortest wait the system accepts, and says so.
/// Why:
///   A timeout of no time at all is refused by the system,
///  so the step that reads after the budget is spent asks for a millisecond and meets silence,
///  and the exchange ends there instead of in a bound of its own.
#[test]
fn a_spent_budget_asks_for_the_shortest_wait_there_is() {
    let patience: Patience = Patience::of(Duration::from_millis(1));
    std::thread::sleep(Duration::from_millis(20));
    assert!(patience.spent(), "nothing was said for the whole budget");
    assert_eq!(patience.left(), Duration::ZERO);
    assert_eq!(patience.bound(Duration::from_secs(5)), Duration::from_millis(1));
}
