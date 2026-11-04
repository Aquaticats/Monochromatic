//! What:
//!  The controls of the checkpoint cutover and dependency validation: R1 retired and its request deleted (review control 8), an old
//!  segment deleted under a checkpoint (control 9), a checkpoint whose live dependency is absent (control 12 and the fifteenth review's
//!  controls 1 to 3), each as a replayed schedule and as a search over every schedule.
//! Why:
//!  The earlier tests compared the revision 14 rule with itself. These check the crash states of a lane's schedule against properties
//!  that name no rule, with a recovery procedure that sees only surviving files; the unsafe checkpoint rule is a red variant of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(check('fifteen').violation).toBeUndefined(); expect(check('thirteen').violation).toBeDefined();
//! ```

use crate::model_discharge_r14::{ADOPT1, ADOPT2, CKPT, OUT1, REQ1_GONE, REQ2, Rules, SEG, SEG_GONE, check, initial, invariant, recover, steps};
use crate::model_explore::{Report, replay};

/// Control 8: R1 and R2 adopted in one chain, R1 retired and its request deleted, restart. Revision 13 cannot validate R1's adopt and
/// drops R2 (a false drop); revision 15 accepts R1's adopt through its terminal outcome.
#[test]
fn a_reclaimed_request_does_not_drop_the_later_adoption() {
    let surviving: u32 = SEG | ADOPT1 | ADOPT2 | REQ2 | OUT1 | REQ1_GONE;
    assert!(!recover(Rules::Thirteen, surviving).r2_live, "revision 13 drops R2");
    assert!(recover(Rules::Fifteen, surviving).r2_live);
    assert!(!recover(Rules::Fifteen, surviving).r1_live, "R1 is terminal");
}

/// Control 9: the old segment is deleted after the checkpoint is durable. Revision 13's checkpoint names its predecessor and cannot be a
/// root once the predecessor is gone (no root at all); revision 15's checkpoint is self-contained.
#[test]
fn a_checkpoint_survives_the_deletion_of_its_predecessor() {
    let surviving: u32 = CKPT | REQ2 | OUT1 | SEG_GONE;
    assert!(!recover(Rules::Thirteen, surviving).root);
    assert!(recover(Rules::Fifteen, surviving).root);
}

/// Control 12 and the fifteenth review's controls 1 and 2: a durable checkpoint whose live request is not durable. The unsafe rule trusts
/// it and calls R2 live with no file; revision 15 does not take it as a root and falls back to the old segment, which a guarded completion
/// has not removed.
#[test]
fn a_checkpoint_before_its_live_dependency_is_not_a_root_and_the_predecessor_remains() {
    let surviving: u32 = CKPT | SEG;
    assert!(recover(Rules::UnsafeCheckpoint, surviving).r2_live, "the unsafe rule trusts a checkpoint with a missing dependency");
    let fifteen = recover(Rules::Fifteen, surviving);
    assert!(fifteen.root && !fifteen.r2_live, "the old segment is the root");
}

/// The search over every schedule of the lane with write-back and one power loss: revision 13 and the unsafe rule are red (no root, or a
/// live request with no file); revision 15, whose completion waits for the checkpoint and its live dependency to be durable, is green.
#[test]
fn the_unsafe_rules_are_red_and_the_guarded_completion_is_green_over_every_schedule() {
    let thirteen: Report = check(Rules::Thirteen);
    assert!(thirteen.violation.is_some(), "revision 13 loses the root or drops a request");
    let unsafe_rule: Report = check(Rules::UnsafeCheckpoint);
    let violation = unsafe_rule.violation.expect("the unsafe checkpoint rule");
    assert!(
        violation.message == "recovery calls R2 live and its request file is gone" || violation.message == "a checkpoint or a segment survived and recovery found no root",
        "{}",
        violation.message
    );
    let fifteen: Report = check(Rules::Fifteen);
    assert!(fifteen.violation.is_none() && fifteen.exhausted, "{:?}", fifteen.violation.map(|found| { return (found.message, found.trace); }));
}

/// The completion guard is what makes the checkpoint rule safe (replayed). The unsafe rule may remove the old segment once only the
/// checkpoint is durable, while the live request is only visible; revision 15 offers the removal only after the request file is durable too.
#[test]
fn the_completion_guard_is_the_safety_of_the_checkpoint_rule() {
    let written = [
        "lane: writes the request file of R1",
        "lane: writes the request file of R2",
        "lane: appends the adopt line of R1 to the segment",
        "lane: appends the adopt line of R2 to the segment",
        "lane: writes the checkpoint whose document names R2 as live",
        "lane: forces the checkpoint",
    ];
    let removal = "lane: completes the checkpoint and removes the old segment";
    let enabled = |state: &crate::model_discharge_r14::Lane, label: &str| -> bool { return steps(state).iter().any(|step| { return step.0 == label; }); };
    let unsafe_state = replay(initial(Rules::UnsafeCheckpoint), steps, &written);
    assert!(enabled(&unsafe_state, removal), "the unsafe rule removes the segment with R2's file only visible");
    let after = replay(unsafe_state, steps, &[removal, "environment: power is lost"]);
    assert_eq!(invariant(&after), Some("recovery calls R2 live and its request file is gone".to_string()));
    // Revision 17 writes the checkpoint that names both requests (R1 has no terminal outcome here) and waits for both request files.
    let mut both: Vec<&str> = written.to_vec();
    both[4] = "lane: writes the checkpoint whose document names both requests as live";
    both[5] = "lane: forces the checkpoint naming both requests";
    let guarded = replay(initial(Rules::Fifteen), steps, &both);
    assert!(!enabled(&guarded, removal), "revision 15 waits for the live dependencies");
    let half = replay(guarded, steps, &["lane: forces R2's request file"]);
    assert!(!enabled(&half, removal), "both request files are needed");
    let ready = replay(half, steps, &["lane: forces R1's request file"]);
    assert!(enabled(&ready, removal));
}

/// Models review 6b (P1): two live requests. R1 and R2 are adopted, R1 has no terminal outcome, a checkpoint is written, forced and completed,
/// and power is lost. The surviving facts hold R1's request and adopt line and no outcome; recovery selects the checkpoint and must not drop R1.
/// Revision 17's non-drop obligation: the checkpoint's document is the replay of the predecessor's valid prefix, so every request that is live
/// there is live in it.
#[test]
fn p1_a_checkpoint_never_erases_a_still_live_request() {
    let schedule = [
        "lane: writes the request file of R1",
        "lane: writes the request file of R2",
        "lane: appends the adopt line of R1 to the segment",
        "lane: appends the adopt line of R2 to the segment",
        "lane: forces R1's request file",
        "lane: forces R2's request file",
        "lane: forces R1's adopt line",
        "lane: forces R2's adopt line",
        "lane: writes the checkpoint whose document names R2 as live",
        "lane: forces the checkpoint",
        "lane: completes the checkpoint and removes the old segment",
        "lane: forces the old segment's removal",
        "environment: power is lost",
    ];
    // The earlier variant names only R2 and the schedule reaches the drop (the red the review reported).
    let reached = replay(initial(Rules::FifteenDropsLive), steps, &schedule);
    let message = invariant(&reached).expect("R1 is live in the surviving facts and the checkpoint dropped it");
    assert!(message.contains("live"), "{}", message);
    // Revision 17's rule writes the checkpoint that names both requests; the same schedule (with that label) keeps R1, and the search is green.
    let mut kept: Vec<&str> = schedule.to_vec();
    kept[8] = "lane: writes the checkpoint whose document names both requests as live";
    kept[9] = "lane: forces the checkpoint naming both requests";
    let preserved = replay(initial(Rules::Fifteen), steps, &kept);
    assert_eq!(invariant(&preserved), None);
    assert!(recover(Rules::Fifteen, preserved.world.visible).r1_live && recover(Rules::Fifteen, preserved.world.visible).r2_live);
    assert!(check(Rules::FifteenDropsLive).violation.is_some());
    let search: Report = check(Rules::Fifteen);
    assert!(search.violation.is_none() && search.exhausted, "{:?}", search.violation.map(|found| { return (found.message, found.trace); }));
}
