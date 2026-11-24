//! What:
//!  Controls for the caller's request budget.
//! Why:
//!  The budget is the one number the split is made against,
//! and it must stay the
//!  caller's:
//! this piece ships no limit of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(RequestBudget.ofInputBytes(4096).inputBytes()).toBe(4096);
//! ```

use super::RequestBudget;

/// The number reads back in its own unit.
#[test]
fn the_budget_carries_the_callers_number() {
    assert_eq!(RequestBudget::of_input_bytes(4096).input_bytes(), 4096);
}

/// The diagnostic says the number and its unit,
/// so a report cannot read bytes as names.
#[test]
fn the_budget_prints_its_unit() {
    assert_eq!(
        RequestBudget::of_input_bytes(1).to_string(),
        "1 bytes of encoded request input"
    );
}
