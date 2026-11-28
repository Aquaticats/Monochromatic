//! What:
//!  The controls of the finished removal's report:
//!  the names it destroyed,
//!  the names it did not,
//!  the origin cleanup's own line,
//!  and the slot it freed.
//! Why:
//!  The printed report is the only trace of a destruction that outlives the removal beyond its
//!  records,
//!  so the lines must say what is gone for good and what a person still has to do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(done.text()).toContain('is destroyed');
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_records_end::RecordsGone;
use crate::worktree_clone_removal_proof_and_plan::removal_report::{
    OriginCleanupDone,
    TheRemovalDone,
};
use crate::worktree_clone_removal_proof_and_plan::removal_settle::SettleAnswer;
use crate::worktree_clone_removal_proof_and_plan::removal_slot_claim::ClaimEnd;
use std::path::PathBuf;

/// One finished removal,
/// with the lines a healthy run prints.
fn a_finished_removal() -> TheRemovalDone {
    return TheRemovalDone {
        slot: String::from("fix-flaky-test"),
        dataset: String::from("tank/wt/fix-flaky-test"),
        destroyed: vec![
            String::from("tank/wt/fix-flaky-test@worktree-own"),
            String::from("tank/wt/fix-flaky-test@sanoid-hourly"),
        ],
        bookmarks: vec![String::from("tank/wt/fix-flaky-test#from-gone")],
        remaining: Vec::new(),
        head: HeadWord::Gone,
        holder_reads_unfinished: 1,
        origin: OriginCleanupDone::Finished {
            snapshot: Some(String::from(
                "tank/mono-checkout@git-worktree-clone-origin-00",
            )),
            line: Some(String::from(
                "the origin cleanup destroyed the snapshot `tank/mono-checkout@git-worktree-clone-origin-00`",
            )),
        },
        settle: SettleAnswer {
            wrote_the_lost_lock: false,
            released_the_lock: true,
            removed_the_registration: true,
            removed_the_underlay: true,
            lines: Vec::new(),
        },
        records: RecordsGone {
            files: Vec::new(),
            plan_removed: true,
            folder_removed: true,
        },
        claim: ClaimEnd::Released,
        // The report of a removal no force touched prints no act of one.
        forced:
            crate::worktree_clone_removal_proof_and_plan::removal_forced_acts::ForcedActs::nothing(),
        plan_path: PathBuf::from(
            "/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/7828944690444299989",
        ),
        requests: Vec::new(),
    };
}

#[test]
fn a_healthy_removal_names_what_is_gone_the_bookmarks_and_the_slot_it_freed() {
    let done: TheRemovalDone = a_finished_removal();
    let text: String = done.text();
    assert!(text.contains("is destroyed, with 2 of the snapshot(s) this removal recorded"));
    assert!(text.contains("destroyed snapshot tank/wt/fix-flaky-test@worktree-own"));
    assert!(text.contains("1 bookmark(s) went with the clone's head and cannot be restored"));
    assert!(text.contains("the clone's head destroy answered and its absence is proved"));
    assert!(text.contains("1 namespace read(s) of the fresh scans could not be completed"));
    assert!(text.contains("the origin cleanup destroyed the snapshot"));
    assert!(text.contains("frees the slot for the next add"));
    assert!(done.is_finished());
}

#[test]
fn a_blocked_origin_names_its_reason_and_where_the_record_stands() {
    let mut done: TheRemovalDone = a_finished_removal();
    done.origin = OriginCleanupDone::Blocked {
        snapshot: String::from("tank/mono-checkout@git-worktree-clone-origin-00"),
        reason: BlockReason::Dependency,
        detail: String::from("1 clone(s) and 0 hold(s) pin the snapshot"),
        line: Some(String::from(
            "the origin cleanup of the snapshot `tank/mono-checkout@git-worktree-clone-origin-00` is not finished: it is still pinned",
        )),
    };
    let text: String = done.text();
    assert!(text.contains("it is still pinned"));
}

#[test]
fn a_name_the_plan_did_not_settle_is_printed_as_not_destroyed() {
    let mut done: TheRemovalDone = a_finished_removal();
    done.remaining = vec![String::from("tank/wt/fix-flaky-test@sanoid-hourly")];
    done.head = HeadWord::Refused;
    let text: String = done.text();
    assert!(text.contains("NOT destroyed snapshot tank/wt/fix-flaky-test@sanoid-hourly"));
    assert!(text.contains("the clone's head destroy was refused"));
    assert!(!done.is_finished());
}

#[test]
fn the_released_claim_prints_that_the_next_command_starts_at_once() {
    let done: TheRemovalDone = a_finished_removal();
    let text: String = done.text();
    assert!(text.contains("the slot's per-slot claim is released, so nothing of this removal keeps another command off `fix-flaky-test`"), "{text}");
}

#[test]
fn a_claim_that_could_not_be_released_prints_why_and_what_waits() {
    let mut done: TheRemovalDone = a_finished_removal();
    done.claim = ClaimEnd::RefusedToRelease {
        said: String::from(
            "the claim on the slot fix-flaky-test could not be released (permission denied)",
        ),
    };
    let text: String = done.text();
    assert!(
        text.contains(
            "the claim on the slot fix-flaky-test could not be released (permission denied)"
        ),
        "{text}"
    );
    assert!(text.contains("another command on `fix-flaky-test` is refused until this process ends, when the claim stops counting as alive"), "{text}");
}

#[test]
fn a_claim_the_order_still_holds_prints_that_the_slot_is_not_free_yet() {
    let mut done: TheRemovalDone = a_finished_removal();
    done.claim = ClaimEnd::Held;
    let text: String = done.text();
    assert!(text.contains("the removal still holds the slot's per-slot claim: `fix-flaky-test` is not free until its last step runs"), "{text}");
}
