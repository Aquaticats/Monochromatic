//! What:
//!  The controls of the lane's reclaim of a proven-stale `index.lock` of its own dead incarnation (owner decision of 2026-11-04): green over
//!  every schedule with the `flock` hold and with the claim chain, red for the superseded never-remove rule, red for each single defect
//!  (no hold, no name check, an unproven holder), and the stated mixed-mount residue as a labelled red.
//! Why:
//!  The earlier row encoded that the lane never removes such a lock. The two properties that stay are checks here: no foreign lock is
//!  removed and a live lock is never removed. The part that waits for the review of the shipped mechanism is the claim chain's slot protocol.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('the lane reclaims its own stale lock', () => { expect(check(Flock).violation).toBeUndefined(); expect(check(NeverRemove).violation).toBeDefined(); });
//! ```

use crate::model_explore::{Report, replay};
use crate::model_m1_reclaim_r19::{Name, Rec, Rules, State, check, initial, invariant, steps, terminal};

const JUDGES_0: &str = "reclaimer 0: proves the holder gone and judges the file at the name";
const JUDGES_1: &str = "reclaimer 1: proves the holder gone and judges the file at the name";
const FLOCK_0: &str = "reclaimer 0: takes the exclusive flock on the stale file";
const FLOCK_1: &str = "reclaimer 1: takes the exclusive flock on the stale file";
const BACKS_OFF_1: &str = "reclaimer 1: finds the flock held and backs off";
const CHECK_0: &str = "reclaimer 0: checks that the name still refers to the judged file";
const CHECK_1: &str = "reclaimer 1: checks that the name still refers to the judged file";
const UNLINK_0: &str = "reclaimer 0: unlinks whatever is at the name and releases";
const UNLINK_1: &str = "reclaimer 1: unlinks whatever is at the name and releases";
const LANDING: &str = "environment: a live landing places its lock";
const PROCEEDS_0: &str = "reclaimer 0: proceeds with no exclusion";
const PROCEEDS_1: &str = "reclaimer 1: proceeds with no exclusion";
const CLAIMS_0: &str = "reclaimer 0: places its claim at slot 0";
const CLAIMS_1_AT_1: &str = "reclaimer 1: places its claim at slot 1";
const HOLDS_0: &str = "reclaimer 0: re-reads the lower slots, all gone, and holds";
const HOLDS_1: &str = "reclaimer 1: re-reads the lower slots, all gone, and holds";
const DIES_0: &str = "environment: reclaimer 0 dies";

/// Whether the search found nothing wrong and covered every state.
fn green(rules: Rules) -> bool {
    let report: Report = check(rules);
    assert!(report.violation.is_none(), "{:?}: {:?}", rules, report.violation.map(|found| { return (found.message, found.trace); }));
    return report.exhausted;
}

/// The message of the first violation the search finds, or a panic when it finds none.
fn red(rules: Rules) -> String {
    return check(rules).violation.unwrap_or_else(|| { panic!("{:?} must be red", rules) }).message;
}

/// The lane reclaims a proven-stale lock of its own dead incarnation through one reclaimer, with the `flock` hold: two reclaimers judge, one
/// holds, the other backs off, the holder checks and unlinks, and the lock is gone. Over every schedule neither a live nor a foreign lock is
/// removed, two reclaimers never hold at once, and no stale lock is left.
#[test]
fn a_proven_stale_lock_is_reclaimed_by_one_reclaimer_under_the_flock() {
    let done: State = replay(initial(Rules::Flock), steps, &[JUDGES_0, JUDGES_1, FLOCK_0, BACKS_OFF_1, CHECK_0, UNLINK_0]);
    assert_eq!(done.name, Name::Absent);
    assert_eq!(terminal(&done), None);
    assert!(green(Rules::Flock));
}

/// The same property where no `flock` exists: the claim chain. The first claimer holds at slot zero, the second finds a running claim and
/// backs off; the claimer that dies holding leaves a gone claim, which the next reclaimer passes over at the next slot.
#[test]
fn a_proven_stale_lock_is_reclaimed_by_one_reclaimer_under_the_claim_chain_and_a_dead_claim_is_passed_over() {
    let done: State = replay(
        initial(Rules::ClaimChain),
        steps,
        &[JUDGES_0, CLAIMS_0, HOLDS_0, CHECK_0, UNLINK_0],
    );
    assert_eq!(done.name, Name::Absent);
    let passed_over: State = replay(
        initial(Rules::ClaimChain),
        steps,
        &[JUDGES_0, CLAIMS_0, HOLDS_0, DIES_0, JUDGES_1, CLAIMS_1_AT_1, HOLDS_1, CHECK_1, UNLINK_1],
    );
    assert_eq!(passed_over.name, Name::Absent, "the second reclaimer passed over the gone claim and reclaimed");
    assert_eq!(passed_over.recs[0], Rec::Dead);
    assert!(green(Rules::ClaimChain));
}

/// The superseded rule: a lane that never removes the lock leaves its dead incarnation's lock behind with no reclaimer dead.
#[test]
fn the_superseded_rule_that_never_removes_the_lock_leaves_it_for_good() {
    assert!(red(Rules::NeverRemove).contains("was left although no reclaimer died"));
}

/// No exclusion (red): both reclaimers judge, both proceed, one unlinks the stale lock, a landing places its live lock, the other unlinks it.
#[test]
fn without_the_hold_a_second_reclaimer_removes_a_live_lock() {
    let reached: State = replay(
        initial(Rules::NoHold),
        steps,
        &[JUDGES_0, JUDGES_1, PROCEEDS_0, PROCEEDS_1, CHECK_0, CHECK_1, UNLINK_0, LANDING, UNLINK_1],
    );
    assert!(reached.removed_live);
    assert!(invariant(&reached).is_some_and(|message| { return message.contains("two reclaimers hold at once") || message.contains("live lock"); }));
    assert!(red(Rules::NoHold).contains("hold at once") || red(Rules::NoHold).contains("live lock"));
}

/// No name check (red): the second reclaimer judged the stale file, the first removed it, a landing placed its lock, and the second, taking
/// the flock after the first released it, unlinks the live lock because it never compared the name with the file it judged.
#[test]
fn without_the_name_check_a_late_reclaimer_removes_the_landings_lock() {
    let reached: State = replay(
        initial(Rules::NoNameCheck),
        steps,
        &[JUDGES_1, JUDGES_0, FLOCK_0, CHECK_0, UNLINK_0, LANDING, FLOCK_1, CHECK_1, UNLINK_1],
    );
    assert!(reached.removed_live && !reached.premise_broken);
    assert!(invariant(&reached).is_some_and(|message| { return message.contains("live lock"); }));
    assert!(red(Rules::NoNameCheck).contains("live lock") || red(Rules::NoNameCheck).contains("foreign lock"));
    // With the check the same schedule stops at the late reclaimer: it finds the name no longer refers to the file it judged.
    let checked: State = replay(initial(Rules::Flock), steps, &[JUDGES_1, JUDGES_0, FLOCK_0, CHECK_0, UNLINK_0, LANDING, FLOCK_1]);
    assert!(steps(&checked).iter().any(|step| { return step.0 == "reclaimer 1: finds the name no longer refers to the judged file and leaves"; }));
}

/// An unproven holder (red): a reclaimer that judges a live landing's lock without proving its holder gone removes it.
#[test]
fn a_reclaimer_that_does_not_prove_the_holder_gone_removes_a_live_lock() {
    assert!(red(Rules::Unproven).contains("live lock"));
}

/// The stated residue that no call closes: one reclaimer holds the `flock` and the other a claim, and neither sees the other. The model
/// reaches it under the mixed rule and the map row for the mechanism says so.
#[test]
fn the_mixed_mount_residue_is_reachable_and_is_the_stated_one() {
    let message: String = red(Rules::Mixed);
    assert!(message.contains("hold at once") || message.contains("live lock") || message.contains("foreign lock"), "{}", message);
}

/// What stays unremoved: a foreign lock at the name is never judged stale, so no schedule removes one (the exhaustive green above), and a
/// stale lock a person removed by hand and a foreign program replaced is the labelled premise break, not a defect of the lane.
#[test]
fn the_persons_hand_removal_is_the_labelled_premise_break() {
    let reached: State = replay(
        initial(Rules::Flock),
        steps,
        &[JUDGES_0, FLOCK_0, CHECK_0, "environment: a person removes the stale lock by hand", LANDING, UNLINK_0],
    );
    assert!(reached.removed_live && reached.premise_broken);
    assert_eq!(invariant(&reached), None, "the premise was broken first; the claim is exactly that");
}
