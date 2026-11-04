//! What:
//!  The controls of revision 18's effective ordered base: the P/Q/A/D counterexample (red under revision 17, green under 18), repeated
//!  removals of supporting owners, a replay that changes support, a crash and a checkpoint between the steps of a rebuild, held, disjoint,
//!  overlapping and superseded requests, and the single-defect controls, each compared with a sequential reference interpreter that is a
//!  separately written execution of the same operations over plain lists.
//! Why:
//!  The acceptance target for C1, B1 to B3 and Q1 asks for these showing controls; every green is exhaustive over the reachable states of
//!  its start (a tail change, a rebuild, a checkpoint and a crash are separate steps that interleave), so a schedule that a rule cannot
//!  survive is found by the search and not by the author's choice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('P/Q/A/D', () => { expect(check(pqad(Seventeen)).violation).toBeDefined(); expect(check(pqad(Eighteen)).violation).toBeUndefined(); });
//! ```

use crate::model_explore::{Report, replay};
use crate::model_support_r18::{Rules, State, VALUES, check, effective, initial, invariant, pqad, steps};

const REMOVE_A: &str = "lane: removes request 3 from the tail (it ended without publishing)";
const REMOVE_P: &str = "lane: removes request 1 from the tail (it ended without publishing)";
const REMOVE_Q: &str = "lane: removes request 2 from the tail (it ended without publishing)";
const D_PUBLISHES: &str = "lane: D publishes";
const REBUILD: &str = "lane: rebuilds D's decision from the tail and the recorded removals";
const CHECKPOINT: &str = "lane: a checkpoint persists D's current decision";
const CRASH: &str = "environment: a crash loses the in-memory decision and recovery starts";

/// Revision 17's schedule: P `x=1`, Q `x=2`, A `x=1`, D `x=1`; the removals happen; the rest publish; D lands.
fn publish_all(rules: Rules, removals: &[&str]) -> State {
    let mut labels: Vec<&str> = removals.to_vec();
    if rules != Rules::Seventeen {
        labels.push(REBUILD);
    }
    labels.extend(["lane: request 1 publishes", "lane: request 2 publishes"]);
    if !removals.contains(&REMOVE_A) {
        labels.push("lane: request 3 publishes");
    }
    labels.push(D_PUBLISHES);
    return replay(pqad(rules), steps, &labels);
}

/// The state after the rebuild step, when the decision is stale.
fn rebuilt(state: State) -> State {
    if state.stale {
        return replay(state, steps, &[REBUILD]);
    }
    return state;
}

/// R16a-5, R17a-4 (the acceptance target's P/Q/A/D counterexample): under revision 17 D re-keys to P when A is removed and loses `x=1`;
/// under revision 18 the effective value before D is Q's, so D adds `x=1` and lands it. The reference interpreter agrees with the
/// revision 18 landing and disagrees with revision 17's.
#[test]
fn pqad_is_red_under_seventeen_and_green_under_eighteen() {
    let lost: State = publish_all(Rules::Seventeen, &[REMOVE_A]);
    assert_eq!(lost.landed, Some(2));
    assert_eq!(lost.reference.landed, Some(1));
    assert!(invariant(&lost).is_some_and(|message| { return message.contains("reference interpreter"); }), "{:?}", invariant(&lost));
    let kept: State = publish_all(Rules::Eighteen, &[REMOVE_A]);
    assert_eq!(kept.landed, Some(1));
    assert_eq!(kept.reference.landed, kept.landed);
    assert_eq!(invariant(&kept), None);
    let red: Report = check(pqad(Rules::Seventeen));
    assert!(red.violation.is_some());
    let green: Report = check(pqad(Rules::Eighteen));
    assert!(green.violation.is_none() && green.exhausted, "{:?}", green.violation.map(|found| { return (found.message, found.trace); }));
}

/// The decision is not a function of the tail alone while it is stale: after a removal D cannot publish until the decision is rebuilt, and
/// the control that publishes with a stale decision lands the wrong value.
#[test]
fn a_stale_decision_blocks_publication_and_the_control_that_ignores_it_is_red() {
    let drained: State = replay(pqad(Rules::Eighteen), steps, &[REMOVE_A, "lane: request 1 publishes", "lane: request 2 publishes"]);
    assert!(drained.stale && drained.tail.is_empty());
    assert!(steps(&drained).iter().all(|step| { return step.0 != D_PUBLISHES; }), "D published with a stale decision");
    let wrong: State = replay(pqad(Rules::EighteenPublishesWhileStale), steps, &[REMOVE_A, "lane: request 1 publishes", "lane: request 2 publishes", D_PUBLISHES]);
    assert_eq!(wrong.landed, Some(2));
    assert!(invariant(&wrong).is_some());
    assert!(check(pqad(Rules::EighteenPublishesWhileStale)).violation.is_some());
}

/// Before any removal the tail P, Q, A, D holds `x=1` as an equal entry that nothing later overrides (A is last), so D stays deferred; once
/// A is removed and the decision rebuilt the effective value is Q's.
#[test]
fn an_equal_entry_that_a_later_differing_entry_overrides_supports_nothing() {
    let start: State = pqad(Rules::Eighteen);
    assert!(!start.added, "A's entry equals the selection and is the last, so the selection stays deferred");
    let without_a: State = rebuilt(replay(start, steps, &[REMOVE_A]));
    assert!(without_a.added, "Q's x=2 is the effective value");
    assert_eq!(effective(without_a.tip, &without_a.tail), 2);
    let any_equal: Rules = Rules::EighteenAnyEqualEntry;
    let wrongly: State = rebuilt(replay(pqad(any_equal), steps, &[REMOVE_A]));
    assert!(!wrongly.added, "the control counts P's overridden entry as support");
    assert!(check(pqad(any_equal)).violation.is_some());
}

/// Repeated removals of supporting owners: A, then Q (so P is last and equal again), then P. Revision 18 follows the fold at every rebuild.
#[test]
fn repeated_removals_of_supporting_owners_follow_the_fold() {
    let mut state: State = pqad(Rules::Eighteen);
    state = rebuilt(replay(state, steps, &[REMOVE_A]));
    assert!(state.added);
    state = rebuilt(replay(state, steps, &[REMOVE_Q]));
    assert!(!state.added, "P's x=1 is now the effective value and equals the selection: deferred again");
    state = rebuilt(replay(state, steps, &[REMOVE_P]));
    assert!(state.added, "no entry and the tip holds x=0: the selection is added to the delta");
    assert_eq!(effective(state.tip, &state.tail), 0);
}

/// A replay that changes support: revision 18 marks the decision stale and rebuilds; the control that marks it stale only on removal keeps D
/// deferred on an entry that no longer matches and loses the selection.
#[test]
fn a_replay_that_changes_support_is_followed_and_a_stale_decision_loses_the_selection() {
    let replayed_label = "lane: a replay changes request 1's entry to Some(2)";
    let supporter: State = initial(Rules::Eighteen, 0, &[Some(1)], 1);
    assert!(!supporter.added);
    let replayed: State = replay(supporter, steps, &[replayed_label]);
    assert!(replayed.stale, "the replay made the decision stale");
    assert!(rebuilt(replayed).added);
    let control: State = initial(Rules::EighteenStaleAfterReplay, 0, &[Some(1)], 1);
    let after: State = replay(control.clone(), steps, &[replayed_label]);
    assert!(!after.stale && !after.added, "the control did not notice");
    assert!(check(control).violation.is_some());
}

/// A crash and a checkpoint between the steps of a rebuild: the checkpoint persists the decision of its moment, a removal follows, the crash
/// loses the in-memory decision, and recovery rebuilds from the tail. Revision 18 survives every interleaving; the control whose recovery
/// trusts the checkpoint's decision lands the stale value.
#[test]
fn a_crash_between_a_checkpoint_and_a_rebuild_is_rebuilt_from_the_tail() {
    let schedule = [CHECKPOINT, REMOVE_A, CRASH];
    let recovered: State = replay(pqad(Rules::Eighteen), steps, &schedule);
    assert!(recovered.stale, "recovery leaves the decision to be rebuilt");
    assert!(steps(&recovered).iter().any(|step| { return step.0 == REBUILD; }));
    let trusting: State = replay(pqad(Rules::EighteenTrustsCheckpoint), steps, &schedule);
    assert!(!trusting.stale && !trusting.added, "the control believes the checkpoint's decision");
    let report: Report = check(pqad(Rules::EighteenTrustsCheckpoint));
    assert!(report.violation.is_some(), "the control must be red");
    let green: Report = check(pqad(Rules::Eighteen));
    assert!(green.violation.is_none() && green.exhausted);
}

/// Held, disjoint, overlapping and superseded requests, over every start of two requests ahead of D with the selection varied, and four
/// starts of three: revision 18 is green in all of them, exhaustively (removals, replays, supersedes, holds, publications, rebuilds,
/// checkpoints and crashes interleave), and revision 17 is red in at least one start of the same family.
#[test]
fn every_start_is_green_under_eighteen_and_the_family_is_red_under_seventeen() {
    let mut seventeen_red: usize = 0;
    let options: Vec<Option<u8>> = std::iter::once(None).chain(VALUES.iter().map(|value| { return Some(*value); })).collect();
    let mut starts: Vec<Vec<Option<u8>>> = Vec::new();
    for first in &options {
        for second in &options {
            starts.push(vec![*first, *second]);
        }
    }
    starts.extend([vec![Some(1), Some(2), None], vec![Some(2), Some(1), Some(1)], vec![None, Some(1), Some(2)], vec![Some(1), None, Some(1)]]);
    for entries in &starts {
        for selection in [1u8, 2u8] {
            let green: Report = check(initial(Rules::Eighteen, 0, entries, selection));
            assert!(green.violation.is_none() && green.exhausted, "{:?} s={}: {:?}", entries, selection, green.violation.map(|found| { return (found.message, found.trace); }));
            if check(initial(Rules::Seventeen, 0, entries, selection)).violation.is_some() {
                seventeen_red += 1;
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

/// A held request: a hook holds the head, so nothing publishes behind it and D cannot land until the hook returns; the decision is unchanged.
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

/// A superseded request: the new version of A without an entry on `x` stops supporting D once the decision is rebuilt; the new version that
/// keeps the entry keeps D deferred.
#[test]
fn a_superseding_version_changes_support_only_through_its_own_entry() {
    let without: State = rebuilt(replay(pqad(Rules::Eighteen), steps, &["person: request 3 is superseded by a new version with entry None"]));
    assert!(without.added);
    let with: State = rebuilt(replay(pqad(Rules::Eighteen), steps, &["person: request 3 is superseded by a new version with entry Some(1)"]));
    assert!(!with.added);
    assert_eq!(with.tail[2].id, 4, "the successor has a new identity");
}

/// The reference interpreter is a separate execution: its tail follows the operations through its own code, so a model that mishandled a
/// removal or a supersede would be caught by the tail comparison before any landing is compared.
#[test]
fn the_reference_world_follows_every_operation_through_its_own_code() {
    let state: State = replay(
        pqad(Rules::Eighteen),
        steps,
        &[REMOVE_Q, "lane: a replay changes request 1's entry to Some(2)", "person: request 3 is superseded by a new version with entry None", REBUILD],
    );
    assert_eq!(state.reference.tail, vec![(1, Some(2)), (4, None)]);
    assert_eq!(invariant(&state), None);
}
