//! What:
//!  The controls of reclamation during capture: the components' rule is green over every schedule of write-back, one power loss, the
//!  departure's capture, force, confirmation, return and recapture, and the reclaimer's candidate and removal; each clause has a control.
//! Why:
//!  Without the controls a green search could be one that never reaches a removal, a recapture or a return. Two reviews found that the
//!  first versions of this model checked too little (physical survival of the old file, then the existence of some root); the invariant is
//!  now the kept support, and the confirmation is against the root selected now.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('reclamation during capture', () => { expect(check(Checked).violation).toBeUndefined(); expect(check(NoNonDrop).violation).toBeDefined(); });
//! ```

use crate::model_capture_reclaim_r23::{Departure, Root, Rules, State, check, durable_root, initial, invariant, steps, terminal};
use crate::model_explore::{Report, replay};

const CAPTURES: &str = "departure: captures the acknowledgment of the record the log selects now";
const FORCES: &str = "departure: forces the file it captured and the closure";
const FORCES_ORPHAN: &str = "departure: forces the descriptor it opened before the unlink";
const RETURNS: &str = "departure: verifies the log by name and returns success";
const GONE: &str = "departure: opening the old segment finds it gone and the log is selected again";
const RECAPTURES: &str = "departure: the selected root is not the one it captured, so it captures again";
const FAILS: &str = "departure: its attempts are spent and it ends with an error";
const DESCENDS: &str = "owner: writes a checkpoint candidate that descends from the promised record";
const RETAINING: &str = "owner: writes a checkpoint candidate that changed the document and keeps the support";
const DROPPING: &str = "owner: writes a checkpoint candidate that drops the support";
const FORCES_CANDIDATE: &str = "owner: forces the candidate and the closure it retains";
const FORCES_NAME: &str = "owner: forces the candidate's directory entry";
const REMOVES: &str = "owner: removes the old segment";
const LOSS: &str = "environment: power is lost";
const PERSISTS_REMOVAL: &str = "environment: write-back persists the removal of the old segment";

/// The first violation, or a panic when the search finds none.
fn red(rules: Rules) -> (String, Vec<String>) {
    let found = check(rules).violation.unwrap_or_else(|| { panic!("{:?} must be red", rules) });
    return (found.message, found.trace);
}

/// Whether `label` is an enabled step of `state`.
fn enabled(state: &State, label: &str) -> bool {
    return steps(state).iter().any(|(step, _)| { return step == label; });
}

/// The components' rule over every schedule is green and exhaustive.
#[test]
fn the_checked_rule_is_green_over_every_schedule() {
    let report: Report = check(Rules::Checked);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// Positive control for the removal: a departure that returned before the removal leaves a log whose root is the candidate that replaced
/// the segment, and the root survives the power loss.
#[test]
fn the_green_search_reaches_a_return_and_a_removal_in_one_schedule() {
    let done: State = replay(initial(Rules::Checked), steps, &[CAPTURES, FORCES, RETURNS, DESCENDS, FORCES_CANDIDATE, FORCES_NAME, REMOVES, PERSISTS_REMOVAL]);
    assert_eq!(done.departure, Departure::Returned);
    assert_eq!(durable_root(&done), Root::Newer);
    assert!(invariant(&done).is_none());
    assert_eq!(durable_root(&replay(done, steps, &[LOSS])), Root::Newer);
}

/// Review of step 2, part B, 1: capture, force, and an unrelated higher-generation candidate that is published but not persisted. The root
/// selected now is the candidate; the return must not be enabled on the strength of the durable root, and the departure captures again. The
/// control that confirms against the durable root can return in the same state.
#[test]
fn an_unpublished_unrelated_candidate_that_is_already_selected_makes_the_departure_capture_again() {
    let selected: State = replay(initial(Rules::Checked), steps, &[CAPTURES, FORCES, RETAINING]);
    assert_eq!(durable_root(&selected), Root::Old, "the candidate is not persisted");
    assert!(!enabled(&selected, RETURNS), "the departure returned on the durable root while another root is selected now");
    assert!(enabled(&selected, RECAPTURES));
    let returned: State = replay(selected, steps, &[RECAPTURES, CAPTURES, FORCES, RETURNS]);
    assert_eq!(returned.departure, Departure::Returned);
    let mismatch: State = replay(initial(Rules::ConfirmsAgainstDurableRoot), steps, &[CAPTURES, FORCES, RETAINING]);
    assert!(enabled(&mismatch, RETURNS), "the control confirms against the durable root and returns");
}

/// The review's first schedule of step 1 (finding 5): capture, force the old segment, write an unrelated candidate, force it and its
/// name. The selected root is the candidate and does not cover the capture; the departure captures the new root, forces it, and returns.
#[test]
fn an_unrelated_higher_generation_candidate_becomes_the_root_and_the_departure_captures_again() {
    let selected: State = replay(initial(Rules::Checked), steps, &[CAPTURES, FORCES, RETAINING, FORCES_CANDIDATE, FORCES_NAME]);
    assert_eq!(durable_root(&selected), Root::Newer);
    assert!(!enabled(&selected, RETURNS), "a return on the survival of the old file alone");
    let returned: State = replay(selected, steps, &[RECAPTURES, CAPTURES, FORCES, RETURNS]);
    assert_eq!(returned.departure, Departure::Returned);
}

/// The ENOENT schedule: the departure selected the old segment, the reclaimer unlinked it, and the open of the path fails; the departure
/// selects again. The other branch of the same state: the open had succeeded before the unlink, and the descriptor is forced (an orphan
/// inode); the confirmation against the root selected now then sends the departure to capture the candidate.
#[test]
fn a_departure_that_finds_the_old_segment_gone_selects_again_and_one_that_had_it_open_forces_the_orphan() {
    let gone: State = replay(initial(Rules::Checked), steps, &[CAPTURES, DESCENDS, FORCES_CANDIDATE, FORCES_NAME, REMOVES]);
    assert!(enabled(&gone, GONE), "the open of the removed path has its own step");
    assert!(enabled(&gone, FORCES_ORPHAN), "the descriptor opened before the unlink can be forced");
    let returned: State = replay(gone.clone(), steps, &[GONE, CAPTURES, FORCES, RETURNS]);
    assert_eq!(returned.departure, Departure::Returned);
    let orphan: State = replay(gone, steps, &[FORCES_ORPHAN]);
    assert!(enabled(&orphan, RETURNS), "the candidate descends from the record, so the confirmation against the selected root holds");
}

/// Without the departure's recapture an unrelated candidate leaves it with a capture that no root covers: red, with the candidate in the trace.
#[test]
fn a_departure_that_never_captures_again_is_stuck_when_the_root_changes() {
    let (message, trace) = red(Rules::NoSubstitution);
    assert_eq!(message, "a departure that captured a record can never return");
    assert!(trace.iter().any(|label| { return label == RETAINING; }), "{:?}", trace);
    let stuck: State = replay(initial(Rules::NoSubstitution), steps, &[CAPTURES, FORCES, RETAINING]);
    assert!(steps(&stuck).iter().all(|(label, _)| { return label.starts_with("environment:") || label.starts_with("owner:"); }), "the departure has a step of its own left");
    assert!(terminal(&stuck).is_some());
}

/// The code's finite attempts: with a single attempt the departure ends with an error when the root changed, and that end is not a success,
/// not a violation and not a stuck state. Under the checked rule's budget the same schedule returns.
#[test]
fn the_bounded_error_exit_is_reachable_with_one_attempt_and_is_not_a_success() {
    let spent: State = replay(initial(Rules::OneAttempt), steps, &[CAPTURES, FORCES, RETAINING, RECAPTURES]);
    assert!(enabled(&spent, FAILS), "the attempts are spent");
    let failed: State = replay(spent, steps, &[FAILS]);
    assert_eq!(failed.departure, Departure::Failed);
    assert!(invariant(&failed).is_none() && terminal(&failed).is_none());
    let report: Report = check(Rules::OneAttempt);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// With recapture and the non-drop rule, removing the old segment for any durable candidate is green: every candidate the component can
/// write keeps the support. This is not a claim that lineage is unnecessary; recapture gives progress before the return, and the support is
/// kept after it by the non-drop rule and by the durable publication and dependency forcing, which the next control removes.
#[test]
fn with_recapture_and_non_drop_removal_for_any_durable_candidate_is_green() {
    let report: Report = check(Rules::ReclaimsAnyCandidate);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// The non-drop rule is what keeps the support after a return: the control without it writes a candidate that drops the support, and the
/// old segment goes after it.
#[test]
fn without_the_non_drop_rule_a_returned_departure_loses_the_support_it_retained() {
    let (message, trace) = red(Rules::NoNonDrop);
    assert!(message.contains("does not keep the support"), "{}", message);
    assert!(trace.iter().any(|label| { return label == DROPPING; }), "{:?}", trace);
}

/// The reclaimer removes the segment only after the candidate is durable: the control that removes it at once leaves a log with neither the
/// segment nor the candidate when the removal persists and the candidate does not.
#[test]
fn a_reclaimer_that_removes_the_segment_before_the_candidate_is_durable_leaves_no_root_in_a_power_loss() {
    let (message, trace) = red(Rules::RemovesBeforeCandidateDurable);
    assert!(message.contains("no root"));
    assert!(trace.iter().any(|label| { return label == REMOVES; }), "{:?}", trace);
    let lost: State = replay(initial(Rules::RemovesBeforeCandidateDurable), steps, &[CAPTURES, FORCES, RETURNS, DESCENDS, REMOVES, PERSISTS_REMOVAL]);
    assert!(invariant(&lost).is_some(), "the removal is durable and the candidate is not");
}

/// The visible-cover control is a combined weakening (it allows a return straight from the capture and replaces the confirmation and the
/// forcing by the visible cover), not a single-guard mutation: it returns with the record and its support unforced, and the power loss takes
/// them.
#[test]
fn a_departure_that_returns_on_a_visible_cover_returns_with_nothing_durable() {
    let (message, trace) = red(Rules::ReturnsOnVisibleCover);
    assert!(message.contains("no root"));
    assert!(trace.iter().any(|label| { return label == RETURNS; }), "{:?}", trace);
}
