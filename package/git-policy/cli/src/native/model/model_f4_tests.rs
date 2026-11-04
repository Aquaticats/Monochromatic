//! What:
//!  The controls of finding F4 of the sixth review and its seventh-review follow-up: cancelling a stalled `update-ref`
//!  and cleaning up its lock.
//! Why:
//!  Each schedule is replayed literally against the rule it breaks; the exhaustive search must find a violation under
//!  revisions 6 and 7 and none under the candidate, whose two mutations must stay red.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision seven: a foreign lock born inside the child's life is deleted', ...);
//! ```

// What:     imports the model's entry points and the explorer's helpers.
// Why:      The tests replay schedules and explore the model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, initial, steps, invariant, Rules } from './model_f4_ref_lock';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f4_ref_lock::{Rules, check, initial, invariant, steps};

/// Revision 6: the stalled child is killed while it holds the lock, nothing removes the lock, every later
/// publication finds it held.
#[test]
fn revision_six_kill_leaves_a_lock_nothing_removes() {
    let reached = replay(
        initial(Rules::Six, true),
        steps,
        &[
            "owner: observes branch.lock absent and launches update-ref",
            "git: creates branch.lock",
            "owner: kills the stalled child",
            "owner: reports the transaction pending",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("the wrapper's own cancellation left a lock that nothing will ever remove".to_string())
    );
}

/// Revision 7, first schedule: the child stalls before taking the lock; a foreign writer creates it inside the
/// child's life; the owner removes it.
#[test]
fn revision_seven_deletes_a_foreign_lock_born_in_a_stalled_childs_life() {
    let reached = replay(
        initial(Rules::Seven, true),
        steps,
        &[
            "owner: observes branch.lock absent and launches update-ref",
            "foreign: creates branch.lock",
            "owner: sends the terminate signal to the stalled child",
            "owner: kills the stalled child",
            "owner: stats branch.lock: absent before launch, born inside the child's life",
            "owner: unlinks branch.lock",
        ],
    );
    assert_eq!(invariant(&reached), Some("a lock the wrapper did not create was removed".to_string()));
}

/// Revision 7, second schedule: the child released its lock and stalls before exit; a foreign writer takes the lock;
/// the owner removes it.
#[test]
fn revision_seven_deletes_a_foreign_lock_born_after_the_child_released_its_own() {
    let reached = replay(
        initial(Rules::Seven, true),
        steps,
        &[
            "owner: observes branch.lock absent and launches update-ref",
            "git: creates branch.lock",
            "git: finishes the update and releases branch.lock",
            "foreign: creates branch.lock",
            "owner: sends the terminate signal to the stalled child",
            "owner: kills the stalled child",
            "owner: stats branch.lock: absent before launch, born inside the child's life",
            "owner: unlinks branch.lock",
        ],
    );
    assert_eq!(invariant(&reached), Some("a lock the wrapper did not create was removed".to_string()));
}

/// Revision 7, pathname race: the owner stats the child's lock; a person removes the stale lock; a foreign writer
/// creates a new one at the same path; the owner's unlink removes it.
#[test]
fn revision_seven_stat_then_unlink_removes_a_replacement_lock() {
    let reached = replay(
        initial(Rules::Seven, true),
        steps,
        &[
            "owner: observes branch.lock absent and launches update-ref",
            "git: creates branch.lock",
            "owner: sends the terminate signal to the stalled child",
            "owner: kills the stalled child",
            "owner: stats branch.lock: absent before launch, born inside the child's life",
            "outside: a person removes the stale branch.lock by hand",
            "foreign: creates branch.lock",
            "owner: unlinks branch.lock",
        ],
    );
    assert_eq!(invariant(&reached), Some("a lock the wrapper did not create was removed".to_string()));
}

/// Revision 7, no birth time: the child's own lock is left, which the wrapper's own cancellation caused.
#[test]
fn revision_seven_without_a_birth_time_leaves_its_own_lock() {
    let reached = replay(
        initial(Rules::Seven, false),
        steps,
        &[
            "owner: observes branch.lock absent and launches update-ref",
            "git: creates branch.lock",
            "owner: sends the terminate signal to the stalled child",
            "owner: kills the stalled child",
            "owner: treats branch.lock as foreign and reports the transaction pending",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("the wrapper's own cancellation left a lock that nothing will ever remove".to_string())
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

/// The candidate: the owner links a journaled unnamed file as `branch.lock`; a recovering owner removes the lock
/// only when its inode is the journaled one. No foreign lock is removed and no own lock is left.
#[test]
fn candidate_removes_only_the_lock_it_journaled() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F4 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// Without the journal the recovering owner cannot tell its own lock and leaves it.
#[test]
fn candidate_without_the_journal_is_red() {
    let report: Report = check(Rules::CandidateNoJournal);
    let violation = report.violation.expect("a violation without the journal");
    assert_eq!(violation.message, "the wrapper's own cancellation left a lock that nothing will ever remove");
}

/// A recovery that removes whatever lock is present deletes a foreign lock.
#[test]
fn candidate_that_removes_any_lock_is_red() {
    let report: Report = check(Rules::CandidateRemovesAny);
    let violation = report.violation.expect("a violation when any lock is removed");
    assert_eq!(violation.message, "a lock the wrapper did not create was removed");
}

/// Revision 8 (red): X1 classified the child at spawn time as unable to dispatch a protected program; a person then installs a
/// `reference-transaction` hook, Git dispatches it, the child stalls and the cancellation ends the hook.
#[test]
fn eight_cancels_an_update_ref_whose_hook_was_installed_after_classification() {
    let reached = replay(
        initial(Rules::Eight, true),
        steps,
        &[
            "owner: observes branch.lock absent and launches update-ref",
            "person: installs a reference-transaction hook",
            "git: creates branch.lock",
            "git: dispatches the reference-transaction hook",
            "owner: sends the terminate signal to the stalled child",
        ],
    );
    assert_eq!(invariant(&reached), Some("a cancellation ended a running person's hook".to_string()));
    assert!(check(Rules::Eight).violation.is_some());
}

/// Revision 9 (green): `update-ref` is exempt from X1 by kind, so no schedule cancels it; no foreign lock is removed and no
/// own lock is left by a cancellation; the stated wait (a stalled update-ref keeps the request pending) is reachable.
#[test]
fn nine_never_cancels_update_ref_and_never_removes_a_lock() {
    let report: Report = check(Rules::Nine);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F4 nine: {} states, deepest {}", report.states, report.deepest);
    let waiting = replay(
        initial(Rules::Nine, true),
        steps,
        &[
            "owner: observes branch.lock absent and launches update-ref",
            "person: installs a reference-transaction hook",
            "git: creates branch.lock",
            "git: dispatches the reference-transaction hook",
        ],
    );
    assert!(!steps(&waiting).iter().any(|entry| { return entry.0.contains("terminate signal"); }));
    assert_eq!(invariant(&waiting), None);
}
