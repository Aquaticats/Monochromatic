//! What:
//!  The controls of R8-2: concurrent ledger append and tail repair by two worktrees' lanes.
//! Why:
//!  Revision 8's shared ledger is red (Y truncates X's append in progress); revision 9's one ledger per lane is green.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision eight: a lane repairs another lane's append', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_ledger_lanes::{Rules, check, initial, invariant, steps};

/// Revision 8 (red): X has written part of its line; Y's L4 repairs "the torn tail" of the shared ledger; X then finishes and
/// acknowledges a line that is not there.
#[test]
fn eight_lets_a_lane_repair_the_append_in_progress_of_another() {
    let reached = replay(
        initial(Rules::Eight),
        steps,
        &["lane X: appends the first part of its ledger line", "L4: lane Y repairs the torn tail of the shared ledger"],
    );
    assert_eq!(invariant(&reached), Some("a lane truncated the tail of another lane's append in progress".to_string()));
    assert!(check(Rules::Eight).violation.is_some());
}

/// Revision 9 (green): one ledger per lane, repaired only by its own lane; a reader treats a torn tail as not yet written.
#[test]
fn nine_repairs_only_the_own_ledger() {
    let report: Report = check(Rules::Nine);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("ledger lanes nine: {} states, deepest {}", report.states, report.deepest);
}
