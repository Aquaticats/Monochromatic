//! What:
//!  The controls of the capture models: a withdrawn request's base, the deferred merge install against later staging, and a lagging
//!  file captured as the three-way merge. Each is red under revision 12 and green over every schedule under revision 13.
//! Why:
//!  Controls 13 and 14 of the twelfth review and control 13 of the thirteenth. The schedules are replayed as written, then the whole space
//!  of each model is searched; each green is a local lemma under the abstractions its module states, not closure of the subsystem.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('capture against a withdrawn base: red under 12, green under 13', ...);
//! ```

use crate::model_capture_r13::{
    Rules, X, base_initial, check_sampled, sampled_initial, sampled_invariant, sampled_steps, check_consumption, consumption_initial, consumption_invariant, consumption_steps, base_invariant, base_steps, check_base, check_install, check_lines, lines_initial, lines_invariant, lines_steps,
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

/// F9 reopened: R1 is refused, and a plain commit captures before the cleanup that drops R1's consumption entry. A cleanup in a separate
/// replacement leaves a window in which the staging is logically hidden (red); one state-log record that holds the whole document
/// closes it (green), so the person's staging stays logically available to the next capture.
#[test]
fn a_capture_after_a_refusal_sees_the_staging_the_refused_request_had_consumed() {
    let window = replay(consumption_initial(Rules::Twelve), consumption_steps, &["owner: R1 is refused and the document is replaced", "client: a plain commit captures"]);
    assert_eq!(consumption_invariant(&window), Some("a capture after the refusal omitted the staging the refused request had consumed".to_string()));
    assert!(check_consumption(Rules::Twelve).violation.is_some());
    let atomic: Report = check_consumption(Rules::Thirteen);
    assert!(atomic.violation.is_none() && atomic.exhausted);
}

/// R13-7, control 13: R2 reads the generation `g` and the marks with no mark, R1's W1 creates the mark, the bounded wait expires and R1's
/// command returns without W2, the person stages `y`, R2 reads the index and validates the unchanged `g`. The base held R1, so the delta
/// omits `x`, which nobody will now commit. Revision 13 validates the generation only (red); revision 14 samples the marks again, sees the
/// mark that appeared, repeats the capture, refuses the base that still holds the marked request, and captures after W2 (green).
#[test]
fn a_mark_that_appears_between_the_samples_repeats_the_capture() {
    let overlap = &[
        "client: C1 reads the document generation and the marks (sample one)",
        "W1: creates the withdrawal mark of R1",
        "W1: WAIT_WITHDRAW expires and R1's command returns without W2",
        "environment: the person stages y",
        "client: C1 reads the index",
    ];
    let thirteen = replay(sampled_initial(Rules::Thirteen), sampled_steps, overlap);
    let accepted = replay(thirteen, sampled_steps, &["client: C1 validates the generation and accepts"]);
    assert_eq!(sampled_invariant(&accepted), Some("R2's accepted capture omits x although R1 was withdrawn and its command had returned".to_string()));
    assert!(check_sampled(Rules::Thirteen).violation.is_some());
    let fourteen = replay(sampled_initial(Rules::Fourteen), sampled_steps, overlap);
    let repeated = replay(fourteen, sampled_steps, &["client: C1 samples the marks again, sees a mark that appeared, and repeats the capture"]);
    assert!(sampled_steps(&repeated).iter().any(|step| { return step.0 == "client: C1 refuses a base that still holds a marked request"; }));
    let green: Report = check_sampled(Rules::Fourteen);
    assert!(green.violation.is_none() && green.exhausted, "{:?}", green.violation.map(|found| { return found.trace; }));
}
