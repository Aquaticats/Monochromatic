//! What:
//!  The controls of R8-8 for M1: a kill at each boundary, and recovery by identity.
//! Why:
//!  Revision 8 strands the killed process's `index.lock`; revision 9 recovers by the journaled inode and never touches a
//!  foreign file; a recovery that unlinks whatever lock is there stays red.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision eight: a kill after the link strands index.lock', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_m1_crash::{Rules, check, check_without_reuse, end_state, initial, initial_reuse, invariant, steps};

/// The steps up to the link, shared by the tests.
const TO_LINK: [&str; 3] = [
    "M1 (1): builds the new index in an unnamed file and fsyncs it",
    "M1 (2): journals the identity of the file and the intent",
    "M1 (3): links the file to index.lock, the single try",
];

/// Revision 8 (red): killed after the link, the lock stays; the repeat takes it for a foreign lock.
#[test]
fn eight_strands_index_lock_when_the_process_is_killed_after_the_link() {
    let mut schedule: Vec<&str> = TO_LINK.to_vec();
    schedule.push("environment: the process is killed with SIGKILL, no exit handler runs");
    schedule.push("L4: repeats M1 from the journal");
    let reached = replay(initial(Rules::Eight), steps, &schedule);
    assert_eq!(end_state(&reached), Some("a stranded index.lock of our own file is left for good".to_string()));
    assert!(check(Rules::Eight).violation.is_some());
}

/// Revision 9 (green): L4 unlinks the journaled inode, finishes (7) after a rename, leaves a foreign lock alone; every kill
/// point and every foreign interleaving is covered.
#[test]
fn nine_recovers_every_kill_point_by_identity() {
    // With the assumption that an inode number is never reused (revision 9's); the next test removes it.
    let report: Report = check_without_reuse(Rules::Nine);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("M1 crash nine: {} states, deepest {}", report.states, report.deepest);
    let mut schedule: Vec<&str> = TO_LINK.to_vec();
    schedule.push("environment: the process is killed with SIGKILL, no exit handler runs");
    let reached = replay(initial(Rules::Nine), steps, &schedule);
    assert!(steps(&reached).iter().any(|entry| { return entry.0.starts_with("L4:"); }));
}

/// A recovery that does not test the identity removes a foreign lock: the guard is shown red.
#[test]
fn nine_without_the_identity_test_removes_a_foreign_lock() {
    let report: Report = check(Rules::NineUnlinksAny);
    let violation = report.violation.expect("a violation without the identity test");
    assert_eq!(violation.message, "a foreign index.lock was removed by the wrapper");
    assert_eq!(invariant(&initial(Rules::Nine)), None);
}

/// The schedule of rule N: M1 builds and journals an unnamed file and the owner dies before linking it; a foreign program takes
/// `index.lock` and the file system gives it the inode number of the dead file; the next owner recovers.
const REUSE: [&str; 6] = [
    "M1 (1): builds the new index in an unnamed file and fsyncs it",
    "M1 (2): journals the identity of the file and the intent",
    "environment: the process is killed with SIGKILL, no exit handler runs",
    "foreign: takes index.lock",
    "",
    "",
];

/// Revision 9 (red): the comparison by inode number removes the foreign lock as ours.
#[test]
fn nine_removes_a_foreign_lock_that_carries_a_reused_inode_number() {
    let reached = replay(
        initial_reuse(Rules::Nine),
        steps,
        &[REUSE[0], REUSE[1], REUSE[2], REUSE[3], "L4: unlinks index.lock only if it is the journaled inode, finishes (7) if the index is the journaled inode"],
    );
    assert_eq!(invariant(&reached), Some("a foreign index.lock was removed by the wrapper".to_string()));
}

/// Revision 10 (green): L4 removes no shared name and reports the lock; every kill point and every foreign interleaving, reuse
/// included, is covered. A stranded lock of our own is the stated residue: it is reported, not removed.
#[test]
fn ten_reports_a_lock_and_removes_nothing_it_may_share() {
    let reached = replay(
        initial_reuse(Rules::Ten),
        steps,
        &[REUSE[0], REUSE[1], REUSE[2], REUSE[3], "L4 (rule N): reports index.lock and removes nothing that another program may share"],
    );
    assert_eq!(invariant(&reached), None);
    assert!(reached.reported);
    let report: Report = check(Rules::Ten);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("M1 crash ten: {} states, deepest {}", report.states, report.deepest);
    assert!(check(Rules::Nine).violation.is_some());
}
