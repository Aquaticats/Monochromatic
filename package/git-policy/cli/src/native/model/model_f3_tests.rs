//! What:
//!  The controls of finding F3 of the sixth review and its seventh-review follow-ups: restarting a quiet owner.
//! Why:
//!  Each schedule is replayed literally against the rule it breaks; the exhaustive search must find a violation under
//!  revisions 6 and 7 and none under the candidate, whose gate-less mutation must stay red.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision seven: a restarter that dies after the signal lets the owner admit a hook', ...);
//! ```

// What:     imports the model's entry points and the explorer's helpers.
// Why:      The tests replay schedules and explore the model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, initial, steps, invariant, endState, Rules } from './model_f3_restart';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f3_restart::{Rules, check, end_state, initial, invariant, steps};

/// Revision 6: the owner dispatches a hook after the restarter observed none, and the restart ends it.
#[test]
fn revision_six_restart_ends_a_hook_dispatched_after_the_observation() {
    let reached = replay(
        initial(Rules::Six, false),
        steps,
        &[
            "restarter: sees a quiet owner and no hook",
            "owner: dispatches the person's hook",
            "restarter: asks the manager to restart the unit",
            "manager: executes the restart of the unit",
        ],
    );
    assert_eq!(invariant(&reached), Some("a running person's program was ended by a restart".to_string()));
}

/// Revision 6: a delayed unit-name request ends the replacement owner.
#[test]
fn revision_six_delayed_request_ends_the_replacement_owner() {
    let reached = replay(
        initial(Rules::Six, false),
        steps,
        &[
            "restarter: sees a quiet owner and no hook",
            "restarter: asks the manager to restart the unit",
            "outside: the owner dies",
            "manager: starts a replacement owner",
            "manager: executes the restart of the unit",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("a restart request ended an owner of an incarnation it did not observe".to_string())
    );
}

/// Revision 7, restarter death: the restarter signals, dies and releases the gate; the owner admits a hook; the
/// outstanding signal then ends the owner and the unit, with the hook in it.
#[test]
fn revision_seven_restarter_death_opens_the_gate_to_a_hook() {
    let reached = replay(
        initial(Rules::Seven, false),
        steps,
        &[
            "restarter: reads lane/incarnation and opens a pidfd",
            "restarter: takes the gate exclusively without waiting",
            "restarter: re-reads lane/incarnation and finds it unchanged",
            "restarter: sends the terminate signal through the pidfd",
            "environment: the restarter dies",
            "owner: takes the shared gate",
            "owner: dispatches the person's hook",
            "owner: handles the terminate signal and exits",
        ],
    );
    assert_eq!(invariant(&reached), Some("a running person's program was ended by a restart".to_string()));
}

/// Revision 7, wedged shared holder: the owner takes the shared gate and wedges before launching anything; no
/// restarter can ever take the gate exclusively.
#[test]
fn revision_seven_wedged_gate_holder_cannot_be_restarted() {
    let reached = replay(
        initial(Rules::Seven, false),
        steps,
        &[
            "owner: takes the shared gate",
            "environment: the owner wedges and takes no further step",
            "restarter: reads lane/incarnation and opens a pidfd",
            "environment: the restarter dies",
        ],
    );
    assert!(steps(&reached).iter().all(|entry| { return entry.0.starts_with("outside:") || entry.0.starts_with("restarter:"); }));
    assert!(!steps(&reached).iter().any(|entry| { return entry.0.starts_with("restarter: takes the gate"); }));
    assert!(end_state(&reached).is_some());
}

/// Revision 7, PID namespaces: the pid in the incarnation record names an unrelated process for a restarter in
/// another namespace, and the signal goes to it.
#[test]
fn revision_seven_misread_pid_kills_an_unrelated_process() {
    let reached = replay(
        initial(Rules::Seven, true),
        steps,
        &[
            "restarter: reads lane/incarnation and opens a pidfd",
            "restarter: takes the gate exclusively without waiting",
            "restarter: re-reads lane/incarnation and finds it unchanged",
            "restarter: sends the terminate signal through the pidfd",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("a process in another PID namespace was killed through a misread pid".to_string())
    );
}

/// The exhaustive search finds a violation under each of the two earlier rules.
#[test]
fn the_search_finds_a_violation_under_revisions_six_and_seven() {
    for rules in [Rules::Six, Rules::Seven] {
        let report: Report = check(rules);
        assert!(report.violation.is_some(), "no violation under {:?}", rules);
    }
}

/// The candidate: the launcher stub holds the gate for the program's life, the kill is uncatchable and comes through
/// the manager's pidfd. No program is ended, no other incarnation targeted, no unrelated process killed, and a wedged
/// owner is restartable unless a person's program runs.
#[test]
fn candidate_restarts_only_a_quiet_owner_and_only_the_observed_incarnation() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F3 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// Without the exclusive gate the kill races the hook's launch.
#[test]
fn candidate_without_the_gate_is_red() {
    let report: Report = check(Rules::CandidateNoGate);
    let violation = report.violation.expect("a violation without the gate");
    assert_eq!(violation.message, "a running person's program was ended by a restart");
}

/// Revision 8 draft: the restart-intent closes the restarter-death schedule, the pid-namespace precondition closes the
/// misread pid, and an owner wedged while holding the gate is the draft's stated narrowing (reachable, not a violation).
#[test]
fn eight_closes_restarter_death_and_namespaces_and_states_the_wedged_holder() {
    let dying = replay(
        initial(Rules::Eight, false),
        steps,
        &[
            "restarter: reads lane/incarnation and opens a pidfd",
            "restarter: takes the gate exclusively without waiting",
            "restarter: re-reads lane/incarnation and finds it unchanged",
            "L7 (4): the restarter creates the restart-intent and fsyncs it",
            "restarter: sends the terminate signal through the pidfd",
            "environment: the restarter dies",
            "owner: handles the terminate signal and exits",
        ],
    );
    assert_eq!(invariant(&dying), None);
    let foreign = replay(initial(Rules::Eight, true), steps, &["L7 precondition: the pid namespaces differ, no restart"]);
    assert_eq!(invariant(&foreign), None);
    let holder = replay(initial(Rules::Eight, false), steps, &["owner: takes the shared gate", "environment: the owner wedges and takes no further step"]);
    assert_eq!(end_state(&holder), None);
}

/// Revision 8 draft, a finding: an owner wedged in its own code while it holds no gate is "ended through the pidfd" only if
/// the end is an uncatchable signal; read as a terminate signal (which the intent file suggests), a wedged owner never
/// handles it and the restarter waits for ever.
#[test]
fn eight_cannot_end_a_wedged_owner_with_a_catchable_signal() {
    let reached = replay(
        initial(Rules::Eight, false),
        steps,
        &[
            "environment: the owner wedges and takes no further step",
            "restarter: reads lane/incarnation and opens a pidfd",
            "restarter: takes the gate exclusively without waiting",
            "restarter: re-reads lane/incarnation and finds it unchanged",
            "L7 (4): the restarter creates the restart-intent and fsyncs it",
            "restarter: sends the terminate signal through the pidfd",
        ],
    );
    assert!(end_state(&reached).is_some());
    let report: Report = check(Rules::Eight);
    assert_eq!(
        report.violation.expect("the wedge is found").message,
        "a wedged owner with no running program of the person's can never be restarted"
    );
}

/// Revision 9, L7 step (5): the uncatchable signal ends a wedged owner that revision 8's terminate signal could not, the
/// restarter-death and namespace schedules stay closed, the restart still needs the exclusive gate, and a wedged gate
/// holder is the stated narrowing (B-hand). Every other schedule is green.
#[test]
fn nine_ends_a_wedged_owner_with_sigkill_and_still_needs_the_gate() {
    let reached = replay(
        initial(Rules::Nine, false),
        steps,
        &[
            "environment: the owner wedges and takes no further step",
            "restarter: reads lane/incarnation and opens a pidfd",
            "restarter: takes the gate exclusively without waiting",
            "restarter: re-reads lane/incarnation and finds it unchanged",
            "L7 (4): the restarter creates the restart-intent and fsyncs it",
            "L7 (5): the restarter ends the owner with SIGKILL through the pidfd",
            "kernel: completes the kill of the owner",
        ],
    );
    assert!(!reached.owner_alive);
    let holder = replay(initial(Rules::Nine, false), steps, &["owner: takes the shared gate", "environment: the owner wedges and takes no further step"]);
    assert!(!steps(&holder).iter().any(|entry| { return entry.0.contains("takes the gate exclusively"); }));
    assert_eq!(end_state(&holder), None);
    let report: Report = check(Rules::Nine);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F3 nine: {} states, deepest {}", report.states, report.deepest);
}

/// Review 2a P2: in a foreign PID namespace the restarter's namespace precondition and "a new client starts" form a cycle that
/// `terminal` never examines; the general liveness check finds that a wedged owner can never be got unstuck there.
#[test]
fn nine_in_a_foreign_namespace_can_never_get_a_wedged_owner_unstuck() {
    let report = crate::model_f3_restart::check_eventually(Rules::Nine);
    let violation = report.violation.expect("a wedged owner no restarter can reach");
    assert_eq!(violation.message, "a state from which the protocol's own steps can never complete");
    assert!(violation.trace.iter().any(|label| { return label.contains("wedges"); }));
}
