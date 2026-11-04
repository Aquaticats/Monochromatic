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
    Answer, INDEX_NEW, LEDGER_LINE, LEDGER_NAME, RELEASED, Rules, SAVED_CONTENT, SAVED_NAME, SAVED_RECORD, SCENARIOS, State, check, initial,
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

/// Revision 13 (green) for every scenario, exhaustive over the interleavings with write-back and one power loss.
#[test]
fn thirteen_every_scenario_keeps_its_promise_at_every_power_loss() {
    for (index, scenario) in SCENARIOS.iter().enumerate() {
        let report: Report = check(Rules::Thirteen, index);
        assert!(report.violation.is_none() && report.exhausted, "{}: {:?}", scenario.name, report.violation.map(|found| { return found.message; }));
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
