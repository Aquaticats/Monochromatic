//! What:
//!  The controls of revision 13's register: every entry reaches an exit within its derived bound with the clock in the state, the
//!  composite commands share no renewed deadline, revision 12's renewal and unbounded foreign stages are red, the exit wording
//!  is honest about failed stages, and the entries with no bound by decision 9 are shown to be unbounded and listed apart.
//! Why:
//!  The earlier register completed every action by assignment and checked a count of steps; this one checks the same property
//!  with a clock, with stages that fail and stall, and with a positive control for each clause.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const entry of ENTRIES) test(`${entry.name} exits within its bound`, ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_register_r13::{
    BUDGET, CAPTURE, COMMITTING, DEPART, ENTRIES, EXEMPT, FOREGROUND, Phase, Rules, State, WITHDRAW, check, check_exempt, entry_bound, initial,
    invariant, path_bound, steps,
};

const STUCK: &str = "a wait with nothing of the protocol's own left that can end it";

/// A path of one entry, with a lifetime the explorer's states can hold.
fn single(index: usize) -> &'static [usize] {
    return Box::leak(vec![index].into_boxed_slice());
}

/// Every entry of the register, alone, under revision 13 (green): the command exits within the bound derived from the table,
/// whatever the other parties do, and no state breaks the exit wording.
#[test]
fn thirteen_every_entry_reaches_an_exit_within_its_bound() {
    for (index, entry) in ENTRIES.iter().enumerate() {
        let report: Report = check(Rules::Thirteen, single(index));
        assert!(report.violation.is_none() && report.exhausted, "{}: {:?}", entry.name, report.violation.map(|found| { return found.message; }));
    }
}

/// The entries the revision names are all rows of the table: CAPTURE, SERVICE-ACQUIRE, the L6 failed try, WITHDRAW, DEPART, and
/// COMMITTING whose action no longer exits 0.
#[test]
fn the_named_entries_are_rows_and_committing_no_longer_exits_zero() {
    let names: Vec<&str> = ENTRIES.iter().map(|entry| { return entry.name; }).collect();
    for wanted in ["CAPTURE", "SERVICE-ACQUIRE", "L6-FAILED-TRY", "WITHDRAW", "DEPART", "FOREGROUND", "COMMITTING"] {
        assert!(names.contains(&wanted), "{}", wanted);
    }
    assert_eq!(ENTRIES[CAPTURE].name, "CAPTURE");
    assert_eq!(ENTRIES[FOREGROUND].name, "FOREGROUND");
    assert_eq!(ENTRIES[COMMITTING].name, "COMMITTING");
    assert_eq!(ENTRIES[WITHDRAW].name, "WITHDRAW");
    assert_eq!(ENTRIES[DEPART].name, "DEPART");
    assert!(ENTRIES[COMMITTING].stages.iter().all(|stage| { return !stage.0.contains("exits 0"); }));
    assert!(!ENTRIES[7].waits || ENTRIES[7].continues, "WITHDRAW leads on to the departure barrier");
}

/// Revision 12, CAPTURE (red): each pass of the loop that re-reads the lane draws a fresh deadline, so continuous invalidation
/// keeps the command in the loop past any number of its own steps.
#[test]
fn twelve_a_retry_renews_the_deadline_and_the_loop_never_ends() {
    let report: Report = check(Rules::Twelve, single(CAPTURE));
    let violation = report.violation.expect("renewal is unbounded");
    assert_eq!(violation.message, format!("a wait lasted more than {} steps of the protocol's own", path_bound(single(CAPTURE))));
}

/// Thirteen, the same schedule (green): the retry costs a tick; after `BUDGET` ticks the loop cannot run again and the action starts.
#[test]
fn thirteen_a_retry_costs_a_tick_and_the_deadline_stands() {
    let invalidate = "environment: the lane's state changes under the loop";
    let retry = "client: the loop reads the state again";
    let reached: State = replay(initial(Rules::Thirteen, single(CAPTURE)), steps, &[invalidate, retry, invalidate, retry]);
    assert_eq!(reached.elapsed, BUDGET);
    assert!(steps(&reached).iter().all(|step| { return step.0 != retry; }), "past the deadline no pass of the loop starts");
    let invalid_again: State = replay(reached, steps, &[invalidate]);
    assert!(steps(&invalid_again).iter().all(|step| { return step.0 != retry; }));
}

/// Revision 12, FOREGROUND's W1 (red): the wait for W2 and the departure barrier have no bound of their own, so a lane that never
/// records the withdrawal leaves the command with nothing of its own that ends it.
#[test]
fn twelve_a_foreign_stage_without_a_bound_leaves_the_command_stuck() {
    let report: Report = check(Rules::Twelve, single(FOREGROUND));
    assert_eq!(report.violation.expect("an unbounded stage").message, STUCK);
    let depart: Report = check(Rules::Twelve, single(DEPART));
    assert_eq!(depart.violation.expect("an unbounded barrier").message, STUCK);
}

/// Revision 12, COMMITTING (red): the exit is 0 and promises a landing after the command returned.
#[test]
fn twelve_committing_exits_zero_and_promises_a_landing() {
    let report: Report = check(Rules::Twelve, single(COMMITTING));
    let violation = report.violation.expect("exit 0 promising a landing");
    assert!(
        violation.message == "the command exited 0 promising a landing after it returned" || violation.message == STUCK,
        "{}",
        violation.message
    );
}

/// A command through several entries (green): the deadlines are per entry, a retry in CAPTURE does not renew any, and the bound is
/// the sum of the entries' bounds.
#[test]
fn thirteen_a_command_through_several_entries_meets_the_sum_of_the_bounds() {
    for path in [&[0usize, CAPTURE, FOREGROUND][..], &[4, FOREGROUND][..], &[CAPTURE, FOREGROUND, COMMITTING][..], &[FOREGROUND, WITHDRAW, DEPART][..]] {
        let static_path: &'static [usize] = Box::leak(path.to_vec().into_boxed_slice());
        let report: Report = check(Rules::Thirteen, static_path);
        assert!(report.violation.is_none() && report.exhausted, "{:?}: {:?}", path, report.violation.map(|found| { return found.message; }));
        let expected: usize = path.iter().map(|index| { return entry_bound(ENTRIES[*index]); }).sum();
        assert_eq!(path_bound(path), expected);
    }
}

/// The honest wording (control): a revision-13 command whose exit ignores the failed stages claims confirmation after a failure;
/// the real rule never does.
#[test]
fn an_exit_that_ignores_failed_stages_claims_confirmation_and_is_caught() {
    let report: Report = check(Rules::IgnoresFailures, single(FOREGROUND));
    assert_eq!(report.violation.expect("the wording claim").message, "the exit said everything was confirmed after a stage failed or hit its own bound");
    let honest: State = replay(
        initial(Rules::Thirteen, single(FOREGROUND)),
        steps,
        &["timer: the monotonic clock advances one tick", "timer: the monotonic clock advances one tick", "client: the deadline is reached and the action starts"],
    );
    assert_eq!(honest.phase, Phase::Acting(0));
    assert_eq!(invariant(&honest), None);
}

/// A stage that stalls is escaped by its own bound and the exit says the work is not confirmed (WAIT_DEPART and WAIT_WITHDRAW).
#[test]
fn a_stalled_departure_barrier_ends_with_a_not_confirmed_exit() {
    let reached: State = replay(
        initial(Rules::Thirteen, single(DEPART)),
        steps,
        &[
            "timer: the monotonic clock advances one tick",
            "timer: the monotonic clock advances one tick",
            "client: the deadline is reached and the action starts",
            "client: the stage's own bound is reached and the command goes on unconfirmed",
            "client: the command exits",
        ],
    );
    assert!(reached.unconfirmed && !reached.claims_confirmed && reached.phase == Phase::Done);
}

/// The entries with no bound by decision 9 are listed on their own and are genuinely unbounded: the search reports a wait with
/// nothing of the protocol's own to end it, which is the stated exemption.
#[test]
fn the_entries_with_no_bound_are_listed_apart_and_are_unbounded() {
    let names: Vec<&str> = EXEMPT.iter().map(|entry| { return entry.0; }).collect();
    assert!(names.contains(&"FORWARDED-MAINTENANCE") && names.contains(&"PERSON-PROGRAMS"));
    assert!(ENTRIES.iter().all(|entry| { return !names.contains(&entry.name); }), "no exempt entry also has a bound");
    assert_eq!(check_exempt().violation.expect("unbounded by decision 9").message, STUCK);
}
