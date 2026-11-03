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
use crate::model_m1_crash::{Rules, check, end_state, initial, invariant, steps};

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
    let report: Report = check(Rules::Nine);
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
