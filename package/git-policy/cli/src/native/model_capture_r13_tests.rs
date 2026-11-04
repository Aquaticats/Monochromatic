//! What:
//!  The controls of the capture models: a withdrawn request's base, the deferred merge install against later staging, and a lagging
//!  file captured as the three-way merge. Each is red under revision 12 and green over every schedule under revision 13.
//! Why:
//!  Controls 13 and 14 of the twelfth review. The schedules are replayed as written, then the whole space is searched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('capture against a withdrawn base: red under 12, green under 13', ...);
//! ```

use crate::model_capture_r13::{
    Rules, X, base_initial, base_invariant, base_steps, check_base, check_install, check_lines, lines_initial, lines_invariant, lines_steps,
};
use crate::model_explore::{Report, replay};

/// The schedule of control 13: R1 pending, W1 marks and returns, the person stages `y`, R2 captures before W2 (red under 12); the
/// same schedule has no capture step under 13 until W2 applied, and the search is green.
#[test]
fn r2_captures_against_the_withdrawn_base_under_twelve_and_is_refused_under_thirteen() {
    let schedule = &["W1: creates the withdrawal mark of R1", "W1: R1's command exits", "environment: the person stages y", "client: C1 captures for R2"];
    let twelve = replay(base_initial(Rules::Twelve), base_steps, schedule);
    assert_eq!(twelve.captured, Some(2));
    assert_eq!(base_invariant(&twelve), Some("R2's capture omits x although R1 was withdrawn and its command returned".to_string()));
    let waiting = replay(base_initial(Rules::Thirteen), base_steps, &["W1: creates the withdrawal mark of R1", "environment: the person stages y"]);
    assert!(base_steps(&waiting).iter().all(|step| { return step.0 != "client: C1 captures for R2"; }), "C1 refuses a base that holds a marked request");
    let after_w2 = replay(waiting, base_steps, &["owner: W2 applies the withdrawal and re-bases the descendants", "client: C1 captures for R2"]);
    assert_eq!(after_w2.captured.map(|paths| { return paths & X; }), Some(X));
    assert!(check_base(Rules::Twelve).violation.is_some());
    let thirteen: Report = check_base(Rules::Thirteen);
    assert!(thirteen.violation.is_none() && thirteen.exhausted);
}

/// Control 14, install: the merge returns with the install deferred, the person stages `z`, MG3 runs; revision 12 replaces `z` with
/// `M`, revision 13 compares with the publication-time entry `E`, leaves the entry and reports `later-staging`.
#[test]
fn a_later_staging_is_replaced_under_twelve_and_left_under_thirteen() {
    let twelve: Report = check_install(Rules::Twelve);
    assert_eq!(twelve.violation.expect("replaced").message, "the merge result replaced the version the person staged during the lag");
    let thirteen: Report = check_install(Rules::Thirteen);
    assert!(thirteen.violation.is_none() && thirteen.exhausted);
}

/// Control 14, lines: `M` changed line a, the lagging file still has `O` there, the person edits line b, `commit -a` captures; the
/// whole-file capture reverts line a (red under 12), the three-way merge keeps both (green under 13), whenever the edit happens.
#[test]
fn a_lagging_file_is_captured_as_the_three_way_merge() {
    let twelve = replay(lines_initial(Rules::Twelve), lines_steps, &["environment: the person edits line b of the lagging file", "client: commit -a captures the file"]);
    assert_eq!(lines_invariant(&twelve), Some("the full-file capture reverted the merge's change to line a".to_string()));
    assert!(check_lines(Rules::Twelve).violation.is_some());
    let thirteen: Report = check_lines(Rules::Thirteen);
    assert!(thirteen.violation.is_none() && thirteen.exhausted);
}
