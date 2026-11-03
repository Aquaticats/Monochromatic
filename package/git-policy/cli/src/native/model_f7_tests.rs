//! What:
//!  The control of finding F7 of the sixth review: the same identity with different content from two clients.
//! Why:
//!  The design promises that an accepted request is immutable and that an identity is bound to its content for good.
//!  This test states that promise as a check over every schedule of two clients and the lane.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('an identity carries one content under the current rule', () => { expect(check('six').violation).toBeUndefined(); });
//! ```

// What:     `use crate::...` imports the model's entry point, its rule enum and the explorer's report type.
// Why:      The test explores the model and reads the report.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, Rules } from './model_f7_identity';
// ```
use crate::model_explore::Report;
use crate::model_f7_identity::{Rules, check};

/// The property under the rule the design has now: every schedule keeps one content per identity and keeps every
/// acknowledged request.
#[test]
fn identity_binding_holds_under_the_current_rule() {
    let report: Report = check(Rules::Six);
    // What:     `if let Some(violation) = &report.violation` enters when a violation exists, borrowing it.
    // Why:      Show the counterexample schedule in the failure message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (report.violation !== undefined) throw new Error(`${message}: ${trace.join(' ; ')}`);
    // ```
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
}
