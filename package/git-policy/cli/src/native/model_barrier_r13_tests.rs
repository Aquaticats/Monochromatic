//! What:
//!  The controls of rule V's barriers: for every saving exit, the reclamation and the preservation step, revision 12 (no barrier) is
//!  red after the power loss, revision 13 (the D-set made durable in order first) is green over every interleaving of writes,
//!  write-back and the power loss, and a barrier that forgets one fact of its D-set is red again.
//! Why:
//!  The twelfth review's controls 9 to 12 and 20: `published` then a lost ledger tail, a staged-only version whose saved file is lost,
//!  the carry-forward with the filesystem's real ordering (content, then name, then record), and every exit at every durability
//!  boundary. Each scenario's green is checked against the same scenario's red, so the invariant is shown able to fail.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const scenario of SCENARIOS) { expect(check('twelve', scenario)).toBeRed(); expect(check('thirteen', scenario)).toBeGreen(); }
//! ```

use crate::model_barrier_r13::{
    Answer, INDEX_NEW, StopPolicy, LEDGER_LINE, LEDGER_NAME, RELEASED, Rules, SAVED_CONTENT, SAVED_NAME, SAVED_RECORD, SCENARIOS, State, check, initial,
    invariant, recoverable, steps,
};
use crate::model_explore::{Report, replay};
use crate::model_observe::World;

/// Revision 12 (red) for every scenario: the promise is made with nothing durable and a power loss breaks it.
#[test]
fn twelve_every_scenario_breaks_its_promise_at_a_power_loss() {
    for (index, scenario) in SCENARIOS.iter().enumerate() {
        let report: Report = check(Rules::Twelve, index);
        let violation = report.violation.unwrap_or_else(|| { panic!("{} must be red under revision 12", scenario.name) });
        assert!(violation.message.contains(scenario.name), "{}", violation.message);
    }
}

/// Revision 13 (green) for every scenario that has no stop disposition and no named content beyond the lane's files; those two kinds
/// are the thirteenth review's schedules and are red below.
#[test]
fn thirteen_keeps_the_promises_that_need_no_stop_and_no_closure() {
    for (index, scenario) in SCENARIOS.iter().enumerate().filter(|(_, scenario)| { return scenario.stop == 0 && scenario.closure == 0; }) {
        let report: Report = check(Rules::Thirteen, index);
        assert!(report.violation.is_none() && report.exhausted, "{}: {:?}", scenario.name, report.violation.map(|found| { return found.message; }));
    }
}

/// Controls 4 and 5 (R13-2): the request and its adoption are durable, W1 (or the unresolved exit) creates the mark, `WAIT_DEPART`
/// expires before the mark is durable, power is lost. Revision 13's single D-set forces the mark last, so the command answers "not
/// confirmed" with the stop lost: the request is live and recovery can publish it (or start an attempt after the one in flight).
#[test]
fn thirteen_a_timed_departure_can_lose_only_the_mark() {
    for name in ["W1 withdrawn", "the unresolved COMMITTING exit"] {
        let index: usize = SCENARIOS.iter().position(|scenario| { return scenario.name == name; }).expect("the scenario");
        let violation = check(Rules::Thirteen, index).violation.unwrap_or_else(|| { panic!("{} must be red under revision 13", name) });
        assert!(violation.message.contains("stop disposition"), "{}", violation.message);
    }
}

/// Control 21 (R13-6): the exit names a built commit whose parent holds content only another writer wrote; the barrier of revision 13
/// forces the request and the lane's files only, so a power loss leaves the named commit without its parent's objects.
#[test]
fn thirteen_a_named_commit_whose_parent_was_not_forced_is_lost() {
    let index: usize = SCENARIOS.iter().position(|scenario| { return scenario.name.starts_with("a named built commit"); }).expect("the scenario");
    assert!(check(Rules::Thirteen, index).violation.is_some());
    let fourteen: Report = check(Rules::Fourteen(StopPolicy::Unbounded), index);
    assert!(fourteen.violation.is_none() && fourteen.exhausted);
}

/// Revision 14 with question 6a answered B (the stop is waited for without a limit): green for every scenario. This is one answer's
/// result, not the design's; the other answers follow.
#[test]
fn fourteen_with_an_unbounded_stop_keeps_every_promise() {
    for (index, scenario) in SCENARIOS.iter().enumerate() {
        let report: Report = check(Rules::Fourteen(StopPolicy::Unbounded), index);
        assert!(report.violation.is_none() && report.exhausted, "{}: {:?}", scenario.name, report.violation.map(|found| { return found.message; }));
    }
}

/// Question 6a answered A (wait up to `WAIT_STOP`, then leave and say the stop is not confirmed): every scenario without a stop is green,
/// and the scenarios with a stop have exactly the residue the question states, a power loss inside the window loses the mark. The
/// residue is shown reachable; the model does not call it green and does not call the answer chosen.
#[test]
fn fourteen_with_a_bounded_stop_has_the_stated_residue() {
    for (index, scenario) in SCENARIOS.iter().enumerate() {
        let report: Report = check(Rules::Fourteen(StopPolicy::BoundedThenLeave), index);
        if scenario.stop == 0 {
            assert!(report.violation.is_none() && report.exhausted, "{}", scenario.name);
        } else {
            let violation = report.violation.expect("the stated residue of option A");
            assert!(violation.message.contains("stop disposition"), "{}", violation.message);
            assert!(violation.trace.iter().any(|label| { return label.contains("not confirmed"); }), "{:?}", violation.trace);
        }
    }
}

/// Question 6a answered C (the background forces the stop; the command does not): red for the scenarios with a stop, which is the
/// option's stated cost ("gives up the power-cut protection of the stop").
#[test]
fn fourteen_with_a_background_stop_gives_up_the_power_cut_protection() {
    for (index, scenario) in SCENARIOS.iter().enumerate().filter(|(_, scenario)| { return scenario.stop != 0; }) {
        assert!(check(Rules::Fourteen(StopPolicy::Background), index).violation.is_some(), "{}", scenario.name);
    }
}

/// A barrier that forgets the last fact of its D-set is red for every scenario (the positive control of the barrier's list).
#[test]
fn a_barrier_that_forgets_one_fact_is_red_again() {
    for (index, scenario) in SCENARIOS.iter().enumerate() {
        let report: Report = check(Rules::MissesLastName, index);
        assert!(report.violation.is_some(), "{} must be red when the last fact is not forced", scenario.name);
    }
}

/// Control 9: the terminal outcome is only visible, the request's recovery copy is released, the ledger tail is lost; a resend finds no
/// request and no outcome. Under revision 13 the release waits for the ledger line, its directory entry and the removal record.
#[test]
fn a_release_on_a_visible_outcome_loses_the_request_and_the_outcome() {
    let reclaim: usize = SCENARIOS.iter().position(|scenario| { return scenario.name.starts_with("T1"); }).expect("the T1 scenario");
    let released: State = replay(
        initial(Rules::Twelve, reclaim),
        steps,
        &["client: writes the next fact", "client: writes the next fact", "client: writes the next fact", "client: writes the next fact", "environment: power is lost"],
    );
    assert!(released.gated_done, "the recovery copy was released");
    assert!(released.world.visible & RELEASED == 0, "the release itself did not survive, and neither did the outcome");
    assert!(invariant(&released).is_some(), "request and outcome are both unusable after the loss");
    let thirteen: State = initial(Rules::Thirteen, reclaim);
    let before_barrier: State = replay(thirteen, steps, &["client: writes the next fact", "client: writes the next fact", "client: writes the next fact"]);
    assert!(
        steps(&before_barrier).iter().all(|step| { return step.0 != "client: writes the next fact"; }),
        "the release is not offered before the D-set is durable"
    );
}

/// Control 11: the staged-only version `S` is read at install, the index entry is replaced, and the saved file and its record are
/// lost at the power loss: nothing holds `S`. Under revision 13 the replacement waits for the saved file, its name and its record.
#[test]
fn replacing_the_index_entry_before_the_saved_file_is_durable_loses_the_version() {
    let mg3: usize = SCENARIOS.iter().position(|scenario| { return scenario.name.starts_with("MG3"); }).expect("the MG3 scenario");
    let lost: State = replay(
        initial(Rules::Twelve, mg3),
        steps,
        &["client: writes the next fact", "client: writes the next fact", "client: writes the next fact", "client: writes the next fact", "environment: power is lost"],
    );
    assert!(lost.world.durable & INDEX_NEW != 0 && lost.world.visible & SAVED_CONTENT == 0);
    assert!(invariant(&lost).is_some());
}

/// Control 12: the filesystem's ordering is content, then name, then record; a name that persists with a torn content, or a record
/// that persists without its name, is not a saved version (recovery drops it).
#[test]
fn a_record_or_name_without_its_dependency_is_not_a_saved_version() {
    let deps: &[(u32, u32)] = &[(SAVED_RECORD, SAVED_CONTENT | SAVED_NAME), (SAVED_NAME, SAVED_CONTENT)];
    let record_only: World = World { visible: SAVED_RECORD, durable: SAVED_RECORD };
    assert!(!recoverable(record_only, SAVED_RECORD, deps));
    let name_without_content: World = World { visible: SAVED_NAME | SAVED_RECORD, durable: SAVED_NAME | SAVED_RECORD };
    assert!(!recoverable(name_without_content, SAVED_RECORD, deps));
    let whole: World = World { visible: SAVED_CONTENT | SAVED_NAME | SAVED_RECORD, durable: SAVED_CONTENT | SAVED_NAME | SAVED_RECORD };
    assert!(recoverable(whole, SAVED_RECORD, deps));
}

/// Control 20: W1 that finds a terminal outcome answers with that outcome only when its ledger line and the ledger's directory entry are
/// durable; a command that answers on a visible line is red, and the unconfirmed answer at the bound is not a promise.
#[test]
fn a_terminal_outcome_is_reported_only_when_its_line_and_entry_are_durable() {
    let found: usize = SCENARIOS.iter().position(|scenario| { return scenario.name.starts_with("W1 finds"); }).expect("the scenario");
    let twelve: State = replay(
        initial(Rules::Twelve, found),
        steps,
        &["client: writes the next fact", "client: writes the next fact", "client: answers confirmed", "environment: power is lost"],
    );
    assert_eq!(twelve.answer, Answer::Confirmed);
    assert!(twelve.world.visible & (LEDGER_LINE | LEDGER_NAME) == 0);
    assert!(invariant(&twelve).is_some());
    let unconfirmed: State = replay(
        initial(Rules::Thirteen, found),
        steps,
        &["client: writes the next fact", "client: writes the next fact", "client: the departure bound passed; answers saved and not confirmed", "environment: power is lost"],
    );
    assert_eq!(unconfirmed.answer, Answer::NotConfirmed);
    assert_eq!(invariant(&unconfirmed), None, "an unconfirmed answer promises nothing");
}

/// F7 reopened against the inbox file: client A links `inbox/<id>` (visible, not durable), client B sends the same bytes, its link finds
/// `EEXIST`, and B then withdraws and promises the request as saved. A barrier that skips what another actor linked first is red; B must
/// force the existing record and the `inbox/` entry itself, whoever created them.
#[test]
fn a_resend_that_finds_the_file_linked_must_still_force_it_before_it_promises() {
    let resend: usize = SCENARIOS.iter().position(|scenario| { return scenario.name.starts_with("a resend that finds"); }).expect("the scenario");
    let skips: Report = check(Rules::SkipsExisting, resend);
    assert!(skips.violation.is_some(), "skipping the existing record loses the request at a power loss");
    let twelve: Report = check(Rules::Twelve, resend);
    assert!(twelve.violation.is_some());
    let thirteen: Report = check(Rules::Thirteen, resend);
    assert!(thirteen.violation.is_none() && thirteen.exhausted, "{:?}", thirteen.violation.map(|found| { return found.message; }));
}
