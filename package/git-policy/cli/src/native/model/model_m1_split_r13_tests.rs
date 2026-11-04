//! What:
//!  The controls of M1's split compare and unlink: a recovery that compares and later unlinks (red), a recovery that never removes a
//!  lock without a live creator (green), and the stated residue of a live creator's own abort.
//! Why:
//!  Review 3b's M1 finding: comparison and unlink were one transition, so the replacement between them was excluded by construction.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a recovery that compares and then unlinks removes a replacement', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_m1_split_r13::{Rules, check, initial, invariant, steps};

/// Revision 12 (red): the creator dies, the recovery compares and agrees, a foreign program replaces the file, the recovery unlinks
/// the replacement.
#[test]
fn a_recovery_that_compares_and_then_unlinks_removes_the_replacement() {
    let reached = replay(
        initial(Rules::Twelve),
        steps,
        &[
            "environment: the creator dies with its lock file in place",
            "owner: compares the file identity with the journaled one and they agree",
            "environment: a foreign program replaces the file at the name",
            "owner: unlinks the pathname",
        ],
    );
    assert_eq!(invariant(&reached), Some("the lane removed a foreign lock after a comparison, with no broken premise to blame".to_string()));
    assert!(check(Rules::Twelve).violation.is_some());
}

/// Revision 13 (green, historical since the owner decision of 2026-11-04 that the lane reclaims its own dead incarnation's lock; the current
/// rule is in `model_m1_reclaim_r19_tests.rs`): a lock without a live creator has no comparison step and no unlink, so a replacement is
/// never removed.
#[test]
fn a_lock_without_a_live_creator_is_never_removed() {
    let report: Report = check(Rules::Thirteen);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return found.trace; }));
    let dead = replay(initial(Rules::Thirteen), steps, &["environment: the creator dies with its lock file in place"]);
    assert!(steps(&dead).iter().all(|step| { return !step.0.starts_with("owner:"); }), "nothing of the lane's touches a lock without a live creator");
}

/// The stated residue: a live creator's own abort compares and unlinks, and a foreign replacement between the two is reachable only
/// through the step that breaks the premise (and is then labelled so, not blamed on the lane).
#[test]
fn the_live_creators_own_abort_has_the_stated_residue() {
    let reached = replay(
        initial(Rules::Thirteen),
        steps,
        &[
            "owner: compares the file identity with the journaled one and they agree",
            "environment: a foreign program replaces the file at the name",
            "owner: unlinks the pathname",
        ],
    );
    assert!(reached.removed_foreign && reached.premise_broken);
    assert_eq!(invariant(&reached), None, "the premise was broken first; the claim is exactly that");
}
