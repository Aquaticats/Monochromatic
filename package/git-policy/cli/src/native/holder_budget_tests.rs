//! Controls for the holder's shared time budget.

use super::*;

/// With no budget in force nothing is clamped and nothing is exhausted.
#[test]
fn without_a_budget_a_deadline_is_its_own() {
    let deadline: Duration = Duration::from_secs(9);
    assert_eq!(remaining(), None);
    assert_eq!(clamp(deadline), deadline);
    assert!(!exhausted());
}

/// A deadline longer than what is left is cut to what is left; a shorter one is kept; dropping the guard ends it.
#[test]
fn a_budget_cuts_a_longer_deadline_and_ends_with_its_guard() {
    let total: Duration = Duration::from_secs(40);
    let guard: HolderBudget = enter(total);
    let left: Duration = remaining().expect("a budget is in force");
    assert!(left <= total);
    assert!(clamp(total * 2) <= total, "a longer deadline is cut to what is left");
    assert!(clamp(total * 2) > Duration::ZERO);
    let short: Duration = Duration::from_millis(1);
    assert_eq!(clamp(short), short);
    drop(guard);
    assert_eq!(remaining(), None);
    assert_eq!(clamp(total * 2), total * 2);
}

/// A spent budget is exhausted and clamps every deadline to zero.
#[test]
fn a_spent_budget_is_exhausted() {
    let _guard: HolderBudget = enter(Duration::ZERO);
    assert!(exhausted());
    assert_eq!(clamp(Duration::from_secs(5)), Duration::ZERO);
}
