//! What:
//!  The controls of revision 18's effective ordered base: the P/Q/A/D counterexample (red under revision 17, green under 18), repeated
//!  removals of supporting owners, a replay that changes support, a checkpoint and recovery between rebuild steps, held, disjoint,
//!  overlapping and superseded requests, and the single-defect controls, each compared with the sequential reference interpreter.
//! Why:
//!  The acceptance target for C1, B1 to B3 and Q1 asks for these showing controls; every green is exhaustive over the reachable states
//!  of its start, so a schedule that a rule cannot survive is found by the search and not by the author's choice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('P/Q/A/D', () => { expect(check(pqad(Seventeen)).violation).toBeDefined(); expect(check(pqad(Eighteen)).violation).toBeUndefined(); });
//! ```

use crate::model_explore::{Report, replay};
use crate::model_support_r18::{Rules, State, VALUES, check, effective, initial, invariant, pqad, reference_landing, steps};

const REMOVE_A: &str = "lane: removes request 3 from the tail (it ended without publishing)";
const REMOVE_P: &str = "lane: removes request 1 from the tail (it ended without publishing)";
const REMOVE_Q: &str = "lane: removes request 2 from the tail (it ended without publishing)";
const D_PUBLISHES: &str = "lane: D publishes";

/// Revision 17's schedule: P `x=1`, Q `x=2`, A `x=1`, D `x=1`; A is removed; P and Q publish; D lands.
fn publish_all(rules: Rules, removals: &[&str]) -> State {
    let mut labels: Vec<&str> = removals.to_vec();
    labels.extend(["lane: request 1 publishes", "lane: request 2 publishes"]);
    if !removals.contains(&REMOVE_A) {
        labels.push("lane: request 3 publishes");
    }
    labels.push(D_PUBLISHES);
    return replay(pqad(rules), steps, &labels);
}

/// R16a-5, R17a-4 (the acceptance target's P/Q/A/D counterexample): under revision 17 D re-keys to P when A is removed and loses `x=1`;
/// under revision 18 the effective value before D is Q's, so D adds `x=1` and lands it. Both agree with the reference interpreter or not.
#[test]
fn pqad_is_red_under_seventeen_and_green_under_eighteen() {
    let lost: State = publish_all(Rules::Seventeen, &[REMOVE_A]);
    assert_eq!(lost.landed, Some(2));
    assert_eq!(invariant(&lost), Some("D landed x=2 and the person had staged x=1".to_string()));
    let kept: State = publish_all(Rules::Eighteen, &[REMOVE_A]);
    assert_eq!(kept.landed, Some(1));
    assert_eq!(invariant(&kept), None);
    assert_eq!(kept.landed, Some(reference_landing(&kept)));
    let red: Report = check(pqad(Rules::Seventeen));
    assert!(red.violation.is_some());
    let green: Report = check(pqad(Rules::Eighteen));
    assert!(green.violation.is_none() && green.exhausted, "{:?}", green.violation.map(|found| { return (found.message, found.trace); }));
}

/// Before any removal the tail P, Q, A, D holds `x=1` as an equal entry that nothing later overrides (A is last), so D stays deferred; once
/// A is removed the effective value is Q's.
#[test]
fn an_equal_entry_that_a_later_differing_entry_overrides_supports_nothing() {
    let start: State = pqad(Rules::Eighteen);
    assert!(!start.added, "A's entry equals the selection and is the last, so the selection stays deferred");
    let without_a: State = replay(start, steps, &[REMOVE_A]);
    assert!(without_a.added, "Q's x=2 is the effective value");
    assert_eq!(effective(without_a.tip, &without_a.tail), 2);
    let any_equal: Rules = Rules::EighteenAnyEqualEntry;
    let wrongly: State = replay(pqad(any_equal), steps, &[REMOVE_A]);
    assert!(!wrongly.added, "the control counts P's overridden entry as support");
    assert!(check(pqad(any_equal)).violation.is_some());
}

/// Repeated removals of supporting owners: A, then Q (so P is last and equal again), then P. Revision 18 follows the fold at every step and
/// stays correct; the schedule is a path of the exhaustive search.
#[test]
fn repeated_removals_of_supporting_owners_follow_the_fold() {
    let mut state: State = pqad(Rules::Eighteen);
    state = replay(state, steps, &[REMOVE_A]);
    assert!(state.added);
    state = replay(state, steps, &[REMOVE_Q]);
    assert!(!state.added, "P's x=1 is now the effective value and equals the selection: deferred again");
    state = replay(state, steps, &[REMOVE_P]);
    assert!(state.added, "no entry and the tip holds x=0: the selection is added to the delta");
    assert_eq!(effective(state.tip, &state.tail), 0);
}

/// A replay that changes support: a predecessor's replay changes its resulting entry before publication. Revision 18 re-evaluates; the control
/// that evaluates only on removal keeps D deferred on an entry that no longer matches and loses the selection.
#[test]
fn a_replay_that_changes_support_is_followed_and_a_stale_decision_loses_the_selection() {
    let supporter: State = initial(Rules::Eighteen, 0, &[Some(1)], 1);
    assert!(!supporter.added);
    let replayed: State = replay(supporter, steps, &["lane: a replay changes request 1's entry to Some(2)"]);
    assert!(replayed.added, "revision 18 evaluates the fold again after a replay");
    let stale: State = initial(Rules::EighteenStaleAfterReplay, 0, &[Some(1)], 1);
    let stale_after: State = replay(stale.clone(), steps, &["lane: a replay changes request 1's entry to Some(2)"]);
    assert!(!stale_after.added, "the control did not re-evaluate");
    assert!(check(stale).violation.is_some());
}

/// A checkpoint then recovery between rebuild steps: the derived decision is rebuilt from the immutable bytes and the recorded removals and
/// equals the one the incremental path reached, in every reachable state of the P/Q/A/D start (the exhaustive search takes the recovery
/// step at every point).
#[test]
fn recovery_between_rebuild_steps_rebuilds_the_same_decision() {
    let recovery = "lane: a checkpoint is followed by recovery, which rebuilds D's decision";
    let mut state: State = pqad(Rules::Eighteen);
    state = replay(state, steps, &[REMOVE_A]);
    let before: bool = state.added;
    assert!(steps(&state).iter().all(|step| { return step.0 != recovery; }), "recovery of an already derived state changes nothing");
    state.added = false;
    state = replay(state, steps, &[recovery]);
    assert_eq!(state.added, before);
}

/// Held, disjoint, overlapping and superseded requests, over every start of up to three requests ahead of D with the tip and the selection
/// varied: revision 18 is green in all of them, exhaustively (removals, replays, supersedes, holds, publications and recoveries interleave),
/// and the reference interpreter agrees with every landing. Revision 17 is red in at least one start of the same family.
#[test]
fn every_start_is_green_under_eighteen_and_the_family_is_red_under_seventeen() {
    let mut seventeen_red: usize = 0;
    let options: Vec<Option<u8>> = std::iter::once(None).chain(VALUES.iter().map(|value| { return Some(*value); })).collect();
    for first in &options {
        for second in &options {
            for third in &options {
                for selection in [1u8, 2u8] {
                    let entries = [*first, *second, *third];
                    let green: Report = check(initial(Rules::Eighteen, 0, &entries, selection));
                    assert!(green.violation.is_none() && green.exhausted, "{:?} s={}: {:?}", entries, selection, green.violation.map(|found| { return (found.message, found.trace); }));
                    if check(initial(Rules::Seventeen, 0, &entries, selection)).violation.is_some() {
                        seventeen_red += 1;
                    }
                }
            }
        }
    }
    assert!(seventeen_red > 0);
}

/// A request with no entry on the path (disjoint) neither supports nor overrides: it leaves the effective value to the entries around it.
#[test]
fn a_disjoint_request_neither_supports_nor_overrides() {
    let state: State = initial(Rules::Eighteen, 0, &[Some(1), None], 1);
    assert!(!state.added);
    assert_eq!(effective(state.tip, &state.tail), 1);
}

/// A held request: a hook holds the head, so nothing publishes behind it and D cannot land until the hook returns; the decision about `x` is
/// unchanged by the hold.
#[test]
fn a_held_head_blocks_publication_and_leaves_the_decision_alone() {
    let start: State = pqad(Rules::Eighteen);
    let held: State = replay(start.clone(), steps, &["environment: a hook holds the head"]);
    assert!(held.tail[0].held);
    let labels: Vec<String> = steps(&held).into_iter().map(|step| { return step.0; }).collect();
    assert!(labels.iter().all(|label| { return !label.ends_with("publishes"); }), "{:?}", labels);
    assert_eq!(held.added, start.added);
    let released: State = replay(held, steps, &["environment: the hook holding the head returns", "lane: request 1 publishes"]);
    assert_eq!(released.tip, 1);
}

/// A superseded request: the new version of A without an entry on `x` stops supporting D (Q's value is effective again); the new version
/// that keeps the entry keeps D deferred.
#[test]
fn a_superseding_version_changes_support_only_through_its_own_entry() {
    let without: State = replay(pqad(Rules::Eighteen), steps, &["person: request 3 is superseded by a new version with entry None"]);
    assert!(without.added);
    let with: State = replay(pqad(Rules::Eighteen), steps, &["person: request 3 is superseded by a new version with entry Some(1)"]);
    assert!(!with.added);
    assert_eq!(with.tail[2].id, 4, "the successor has a new identity");
}
