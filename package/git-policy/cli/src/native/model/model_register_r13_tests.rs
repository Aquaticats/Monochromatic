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
    BUDGET, CAPTURE, COMMITTING, DEPART, ENTRIES, EXEMPT, FOREGROUND, BARRIER_ENTRIES, MATERIALIZE, Phase, Rules, STOP, Stage, State, WITHDRAW, check, check_exempt,
    entry_bound, initial, invariant, path_bound, steps,
};

const STUCK: &str = "a wait with nothing of the protocol's own left that can end it";

/// A path of one entry, with a lifetime the explorer's states can hold.
fn single(index: usize) -> &'static [usize] {
    return Box::leak(vec![index].into_boxed_slice());
}

/// Every entry of the register, alone, under revision 14 (green): the command exits within the bound derived from the table, whatever
/// the other parties do, an operation may block and the command's own deadline is the escape, and no exit breaks its wording.
#[test]
fn fourteen_every_entry_reaches_an_exit_within_its_bound() {
    for (index, entry) in ENTRIES.iter().enumerate() {
        let report: Report = check(Rules::Fourteen, single(index));
        assert!(report.violation.is_none() && report.exhausted, "{}: {:?}", entry.name, report.violation.map(|found| { return found.message; }));
    }
}

/// Controls 4 and 19 (R13-2, R13-9): STOP and MATERIALIZE are register entries only in revision 14. Under 13 the departure's stop and the
/// wait for materialization have no bound of their own, so a slow disk or a stalled install leaves the command with nothing of its own
/// that ends it (red); under 14 each has a named bound and an action.
#[test]
fn thirteen_has_no_bound_for_the_stop_receipt_and_for_materialization() {
    for index in [STOP, MATERIALIZE] {
        let violation = check(Rules::Thirteen, single(index)).violation.unwrap_or_else(|| { panic!("{} must be red under 13", ENTRIES[index].name) });
        assert_eq!(violation.message, STUCK, "{}", ENTRIES[index].name);
    }
}

/// Review 4a: a local operation can block and not return. Revision 12 has no escape (red: stuck); revision 14's own deadline is
/// independently executable and the command goes on.
#[test]
fn a_blocked_local_operation_is_escaped_by_the_commands_own_deadline() {
    let start_index: usize = 0;
    let blocked = |rules: Rules| -> State {
        return replay(
            initial(rules, single(start_index)),
            steps,
            &["timer: the monotonic clock advances one tick", "timer: the monotonic clock advances one tick", "client: the deadline is reached and the action starts", "client: the operation blocks and does not return"],
        );
    };
    assert_eq!(check(Rules::Twelve, single(start_index)).violation.expect("blocked for ever").message, STUCK);
    let fourteen: State = blocked(Rules::Fourteen);
    assert!(fourteen.stalled);
    assert!(steps(&fourteen).iter().any(|step| { return step.0.contains("own deadline expires while the operation is blocked"); }));
    assert!(matches!(ENTRIES[start_index].stages[0].1, Stage::Local));
}

/// R13-2 with a failed or blocked mark creation: the stop authority was not recorded, so the request may still land. Revision 13 exited
/// with the weaker "not confirmed" wording (red); revision 14 says that the request may still land.
#[test]
fn a_stop_that_could_not_be_recorded_is_not_reported_as_an_unconfirmed_save() {
    let thirteen: Report = check(Rules::Thirteen, single(FOREGROUND));
    assert_eq!(thirteen.violation.expect("the wording").message, "the stop could not be recorded and the exit did not say that the request may still land");
    let fourteen: Report = check(Rules::Fourteen, single(FOREGROUND));
    assert!(fourteen.violation.is_none());
    let committing: Report = check(Rules::Thirteen, single(COMMITTING));
    assert!(committing.violation.is_some());
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

/// Revision 12, CAPTURE (red): each pass of the loop that re-reads the lane draws a fresh deadline, so continuous invalidation keeps the
/// command in the loop for as long as the lane changes, past any number of its own steps. The schedule is replayed (the exhaustive search
/// finds an earlier red, the blocked operation, so the renewal is asserted on its own).
#[test]
fn twelve_a_retry_renews_the_deadline_and_the_loop_never_ends() {
    let invalidate = "environment: the lane's state changes under the loop";
    let retry = "client: the loop reads the state again";
    let schedule = [invalidate, retry, invalidate, retry, invalidate, retry, invalidate, retry];
    let twelve: State = replay(initial(Rules::Twelve, single(CAPTURE)), steps, &schedule);
    assert_eq!(twelve.elapsed, 0, "every pass renewed the deadline");
    assert!(steps(&twelve).iter().all(|step| { return step.0 != "client: the deadline is reached and the action starts"; }));
    assert!(check(Rules::Twelve, single(CAPTURE)).violation.is_some());
    assert!(path_bound(single(CAPTURE)) < schedule.len(), "the bound derived from the table is far below the length of a loop that never ends");
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

/// Revision 12, COMMITTING (red): once the stages complete, the exit is 0 and promises a landing after the command returned. The schedule
/// is replayed through the stages, so the assertion is the promise clause and not an earlier stuck state.
#[test]
fn twelve_committing_exits_zero_and_promises_a_landing() {
    let reached: State = replay(
        initial(Rules::Twelve, single(COMMITTING)),
        steps,
        &[
            "timer: the monotonic clock advances one tick",
            "timer: the monotonic clock advances one tick",
            "client: the deadline is reached and the action starts",
            "client: the stage succeeds",
            "environment: the other party completes the stage",
            "client: the stage succeeds",
            "client: the command exits",
        ],
    );
    assert_eq!(invariant(&reached), Some("the command exited 0 promising a landing after it returned".to_string()));
}

/// A command through several entries (green): the deadlines are per entry, a retry in CAPTURE does not renew any, and the bound is
/// the sum of the entries' bounds.
#[test]
fn thirteen_a_command_through_several_entries_meets_the_sum_of_the_bounds() {
    for path in [&[0usize, CAPTURE, FOREGROUND][..], &[4, FOREGROUND][..], &[CAPTURE, FOREGROUND, COMMITTING][..], &[FOREGROUND, WITHDRAW, DEPART][..]] {
        let static_path: &'static [usize] = Box::leak(path.to_vec().into_boxed_slice());
        let report: Report = check(Rules::Fourteen, static_path);
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

/// Owner decision 14: under revision 15 every entry reaches an exit within its bound except while the departure barrier forces the final
/// save, which has no bound and no escape (a sync that never completes is an outside cause). The report names the exemption instead of
/// claiming a bounded return through the final save.
#[test]
fn fifteen_every_entry_is_bounded_except_the_final_save_which_is_exempt_by_decision() {
    for (index, entry) in ENTRIES.iter().enumerate() {
        let report: Report = check(Rules::Fifteen, single(index));
        assert!(report.violation.is_none() && report.exhausted, "{}: {:?}", entry.name, report.violation.map(|found| { return found.message; }));
    }
    for name in BARRIER_ENTRIES {
        let index: usize = ENTRIES.iter().position(|entry| { return entry.name == name; }).expect("entry");
        assert!(ENTRIES[index].stages.iter().any(|stage| { return stage.1 == Stage::Barrier; }), "{} has a barrier stage", name);
    }
}

/// Decision 14 removes the timeout from the barrier: in a barrier stage the command has no step of its own to leave it, while the
/// earlier revisions give it one.
#[test]
fn fifteen_gives_the_final_save_no_escape_where_fourteen_does() {
    let depart: usize = DEPART;
    let at_barrier = |rules: Rules| -> State {
        return replay(
            initial(rules, single(depart)),
            steps,
            &["timer: the monotonic clock advances one tick", "timer: the monotonic clock advances one tick", "client: the deadline is reached and the action starts"],
        );
    };
    let own = |state: &State| -> usize { return steps(state).iter().filter(|step| { return step.0.starts_with("client:"); }).count(); };
    assert_eq!(own(&at_barrier(Rules::Fifteen)), 0);
    assert!(own(&at_barrier(Rules::Fourteen)) > 0);
}

/// Models review 6a, 1 (P1, was red): ordinary wait completion, final-save completion and normal return are different events. The control
/// that keeps the ordinary "wait ends" step carries a departing command to a normal return with no final save (red); revision 15 has no such
/// step for a departure, and the search over its entries is green.
#[test]
fn p1_an_ordinary_wait_completion_cannot_stand_in_for_the_final_save() {
    let shortcut = "environment: the other party progresses and the wait ends";
    for index in [DEPART, STOP] {
        let done: State = replay(initial(Rules::FifteenOrdinaryWait, single(index)), steps, &[shortcut]);
        assert_eq!(done.phase, Phase::Done);
        assert!(invariant(&done).is_some_and(|message| { return message.contains("final save"); }), "{} returned normally with no final save", ENTRIES[index].name);
        assert!(check(Rules::FifteenOrdinaryWait, single(index)).violation.is_some());
        let fifteen: State = initial(Rules::Fifteen, single(index));
        assert!(steps(&fifteen).iter().all(|step| { return step.0 != shortcut; }), "{} has no ordinary completion under revision 15", ENTRIES[index].name);
        let report: Report = check(Rules::Fifteen, single(index));
        assert!(report.violation.is_none() && report.exhausted);
    }
}

/// Models review 6a, 3 (P1, was red): forcing the final save cannot create a mark that was never created. When the mark stage fails the
/// command returns by the outside-fault exit of revision 17 and not normally, whatever the barrier that follows did; the control that
/// returns normally with a warning is red.
#[test]
fn p1_a_failed_mark_creation_never_returns_normally() {
    let schedule = [
        "timer: the monotonic clock advances one tick",
        "timer: the monotonic clock advances one tick",
        "client: the deadline is reached and the action starts",
        "client: the stage succeeds",
        "client: the stage fails",
        "client: the stage succeeds",
        "environment: the other party completes the stage",
        "environment: the other party completes the stage",
        "client: the stage succeeds",
        "client: the command exits",
    ];
    let warned: State = replay(initial(Rules::FifteenNormalAfterLoss, single(FOREGROUND)), steps, &schedule);
    assert!(warned.authority_lost && warned.saved);
    assert!(invariant(&warned).is_some_and(|message| { return message.contains("normal return"); }), "{:?}", invariant(&warned));
    let faulted: State = replay(initial(Rules::Fifteen, single(FOREGROUND)), steps, &schedule);
    assert!(faulted.outside_fault && invariant(&faulted).is_none());
    assert!(check(Rules::FifteenNormalAfterLoss, single(FOREGROUND)).violation.is_some());
    let report: Report = check(Rules::Fifteen, single(FOREGROUND));
    assert!(report.violation.is_none() && report.exhausted);
}

/// Every route from the start to a normal `Done`, with the labels it took (a depth-first walk of the finite, acyclic path graph).
fn done_routes(rules: Rules, path: &'static [usize]) -> Vec<(Vec<String>, State)> {
    let mut found: Vec<(Vec<String>, State)> = Vec::new();
    let mut pending: Vec<(Vec<String>, State)> = vec![(Vec::new(), initial(rules, path))];
    while let Some((labels, state)) = pending.pop() {
        if state.phase == Phase::Done {
            found.push((labels, state));
            continue;
        }
        for (label, after) in steps(&state) {
            let mut next: Vec<String> = labels.clone();
            next.push(label);
            pending.push((next, after));
        }
    }
    return found;
}

/// Models review 7, 1 (P1, red against the previous model): ordinary successful completion of FOREGROUND and COMMITTING reached a normal
/// return with no final save and no record of what completed. Every normal return of those entries must be one of two typed completions:
/// the request was published (nothing is owed: the receipt forbids a later attempt and L4 queues it without forcing) or a terminal outcome
/// arrived (the absorbing outcome's ledger line is owed first and forced by the departure barrier). An unsaved normal return is allowed only
/// through the first, and the second is not a route that reaches `Done` without the save.
#[test]
fn p7_ordinary_completion_of_foreground_and_committing_is_published_or_saved() {
    let published = "environment: the other party publishes the request and the wait ends";
    for index in [FOREGROUND, COMMITTING] {
        let routes = done_routes(Rules::Fifteen, single(index));
        assert!(!routes.is_empty());
        for (labels, state) in routes {
            let unsaved_normal: bool = !state.saved && !state.outside_fault && !state.authority_lost;
            assert!(!unsaved_normal || labels.iter().any(|label| { return label == published; }), "{}: an unsaved normal return through {:?}", ENTRIES[index].name, labels);
        }
    }
}
