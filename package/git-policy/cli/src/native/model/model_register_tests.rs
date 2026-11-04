//! What:
//!  The controls of the register of waits: each of the nine entries reaches its bound under revision 11 and none does under 10.
//! Why:
//!  The owner's rule "nothing should ever wait forever or indefinitely" as a check over every entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const [name] of ENTRIES) test(`${name} reaches its bound`, ...);
//! ```

use crate::model_register::{ENTRIES, Rules, check};

/// Revision 10 (red): for every entry, a party that never progresses leaves the process with nothing of its own to end the wait.
#[test]
fn ten_leaves_every_entry_waiting_with_nothing_to_end_it() {
    for (index, (name, _)) in ENTRIES.iter().enumerate() {
        let violation = check(Rules::Ten, index).violation.unwrap_or_else(|| { panic!("{} must be red under revision 10", name) });
        assert_eq!(violation.message, "a wait with nothing of the protocol's own left that can end it", "{}", name);
    }
}

/// Revision 11 (green): for every entry the bound is reached and the action taken within the bound, whatever the other party does.
#[test]
fn eleven_ends_every_entry_within_its_bound() {
    for (index, (name, _)) in ENTRIES.iter().enumerate() {
        let report = check(Rules::Eleven, index);
        assert!(report.violation.is_none() && report.exhausted, "{}", name);
    }
}
