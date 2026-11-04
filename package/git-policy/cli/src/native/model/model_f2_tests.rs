//! What:
//!  The controls of finding F2 of the sixth review and of F2A and F2B of the seventh: the lifetime of the lane's owner
//!  and the cleanup a new owner does.
//! Why:
//!  Each review schedule is replayed literally against the rule it breaks, and the exhaustive search must find a
//!  violation under revisions 6 and 7 and none under the candidate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision seven: the fork-to-exec window blocks the next owner', ...);
//! ```

// What:     `use crate::...` imports the model's entry points and the explorer's helpers.
// Why:      The tests replay schedules and explore the model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, initial, steps, invariant, endState, Rules } from './model_f2_lifetime';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f2_lifetime::{Rules, State, check, end_state, initial, invariant, steps};

/// What:
///  Whether any step of the new owner is enabled in `state`.
///  `.iter().any(|entry| ...)` asks whether some entry satisfies the test; `|entry| expr` is a closure
///  (TS: `(entry) => expr`) and `.0` reads the first field of a pair.
/// Why:
///  A retained-descriptor deadlock leaves other steps (the environment's, the hook's) enabled but none of the new owner's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const newOwnerCanStep = (s: State) => steps(s).some(([label]) => label.startsWith('new owner:'));
/// ```
fn new_owner_can_step(state: &State) -> bool {
    return steps(state).iter().any(|entry| { return entry.0.starts_with("new owner:"); });
}

/// Revision 6, retained descriptor: the hook keeps the lock description, the owner dies, the next owner cannot acquire.
#[test]
fn revision_six_retained_descriptor_blocks_the_next_owner() {
    let reached = replay(
        initial(Rules::Six, false),
        steps,
        &[
            "owner: forks the hook child, which holds the lock description",
            "owner: dies",
            "child: executes the hook (close-on-exec closes the lock description)",
            "hook: leaves a daemon that closed the lock description",
        ],
    );
    // The hook may still choose to close its copy, but nothing the new owner can do is enabled.
    assert!(!new_owner_can_step(&reached));
    assert!(end_state(&reached).is_some());
}

/// Revision 6, closed descriptor: the hook closed its copy and still writes when the lock frees, and the new owner
/// mutates beside it.
#[test]
fn revision_six_mutates_beside_a_descendant_that_closed_the_descriptor() {
    let reached = replay(
        initial(Rules::Six, false),
        steps,
        &[
            "owner: forks the hook child, which holds the lock description",
            "child: executes the hook (close-on-exec closes the lock description)",
            "hook: closes its copy of the lock description and carries on",
            "owner: dies",
            "new owner: acquires the lane lock",
            "new owner: begins to mutate the lane",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("the new owner mutates while a descendant of the previous incarnation lives".to_string())
    );
}

/// Revision 7, F2A: the child is stopped between fork and exec, still holding the lock description; the owner dies;
/// the next owner cannot acquire, so it cannot reach the cleanup that would kill the child's scope.
#[test]
fn revision_seven_fork_to_exec_window_blocks_the_next_owner() {
    let reached = replay(
        initial(Rules::Seven, false),
        steps,
        &[
            "owner: writes lane/incarnation and fsyncs it",
            "owner: forks the hook child, which holds the lock description",
            "environment: the child is stopped and never runs again",
            "owner: dies",
        ],
    );
    assert!(!new_owner_can_step(&reached));
    assert!(steps(&reached).is_empty());
    assert!(end_state(&reached).is_some());
}

/// Revision 7, F2B: the replacement service owner shares the unit's cgroup path, reads the recorded scope and kills
/// the scope it is in.
#[test]
fn revision_seven_replacement_service_owner_kills_itself() {
    let reached = replay(
        initial(Rules::Seven, true),
        steps,
        &[
            "owner: writes lane/incarnation and fsyncs it",
            "owner: dies",
            "new owner: acquires the lane lock",
            "new owner: reads lane/incarnation",
            "new owner: writes cgroup.kill on the recorded scope",
            "kernel: delivers the kill to the new owner, a member of the same cgroup",
        ],
    );
    assert_eq!(invariant(&reached), Some("the replacement owner was killed by the cleanup it ran".to_string()));
}

/// The exhaustive search finds a violation under each of the two earlier rules.
#[test]
fn the_search_finds_a_violation_under_revisions_six_and_seven() {
    for rules in [Rules::Six, Rules::Seven] {
        let report: Report = check(rules);
        assert!(report.violation.is_some(), "no violation under {:?}", rules);
    }
}

/// The candidate: read the incarnation without the lock, confirm the owner dead, kill the old invocation's own scope,
/// wait for it to be empty, then acquire. Every schedule, a frozen child, a daemon and an owner dying at any step included.
#[test]
fn candidate_reaps_before_it_acquires_and_never_targets_itself() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F2 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// Without the wait for an empty scope the new owner mutates while the kill is still being delivered.
#[test]
fn candidate_without_the_empty_wait_is_red() {
    let report: Report = check(Rules::CandidateNoWait);
    let violation = report.violation.expect("a violation without the wait");
    assert_eq!(violation.message, "the new owner mutates while a descendant of the previous incarnation lives");
}

/// Revision 8 draft: per-incarnation cgroups mean a replacement service owner reaps without targeting itself, a closed-
/// descriptor daemon is reaped before the new owner mutates, and the one wait left is the draft's own stated narrowing.
#[test]
fn eight_reaps_by_incarnation_and_states_its_one_wait() {
    let report: Report = check(Rules::Eight);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F2 eight: {} states, deepest {}", report.states, report.deepest);
    // The narrowing is reachable (positive control): a child stopped between fork and exec blocks L1.
    let blocked = replay(
        initial(Rules::Eight, false),
        steps,
        &[
            "owner: writes lane/incarnation and fsyncs it",
            "owner: forks the hook child, which holds the lock description",
            "environment: the child is stopped and never runs again",
            "owner: dies",
        ],
    );
    assert!(!new_owner_can_step(&blocked));
    assert_eq!(end_state(&blocked), None);
}
