//! What:
//!  The controls of R9-8: a refusal followed by a foreign move of the branch before E1.
//! Why:
//!  Red under revision 9 (the hook runs again); green under revision 10.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision nine: a refusal then a foreign move repeats the hook', ...);
//! ```

use crate::model_e1_classify::{Rules, check, initial, invariant, steps};
use crate::model_explore::{Report, replay};

/// Revision 9 (red): the foreign advance makes `tip == old` false, E1 (d) runs crash recovery and the hook runs again.
#[test]
fn nine_repeats_a_refused_hook_after_a_foreign_move() {
    let reached = replay(
        initial(Rules::Nine),
        steps,
        &[
            "P2: update-ref exits nonzero with the reference-transaction hook's abort text",
            "foreign: advances the branch before E1 runs",
            "E1 (d): the evidence matches nothing, so crash recovery repeats the update",
        ],
    );
    assert_eq!(invariant(&reached), Some("a person's hook that refused ran again".to_string()));
    assert!(check(Rules::Nine).violation.is_some());
}

/// Revision 10 (green): the class comes from the exit status and the last line, whatever the branch did.
#[test]
fn ten_classifies_a_refusal_before_reading_state() {
    let report: Report = check(Rules::Ten);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("E1 ten: {} states, deepest {}", report.states, report.deepest);
}
