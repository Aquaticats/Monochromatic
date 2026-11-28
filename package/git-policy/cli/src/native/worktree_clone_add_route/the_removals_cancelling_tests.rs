//! What:
//!  The controls of the value the forwarded removal path takes from this folder.
//! Why:
//!  The value decides nothing,
//!  and what it must do is hold still:
//!  a removal's stop points all ask the same answer,
//!  and a fresh value per call would make two stops of one removal disagree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theRemovalsCancelling()).toBe(theRemovalsCancelling());
//! ```

use super::the_removals_cancelling;

#[test]
fn the_answer_handed_over_is_one_value_for_every_stop_point() {
    let first: *const dyn crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheRemovalCancelling =
        the_removals_cancelling() as *const dyn crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheRemovalCancelling;
    let second: *const dyn crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheRemovalCancelling =
        the_removals_cancelling() as *const dyn crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheRemovalCancelling;
    assert!(
        std::ptr::addr_eq(first, second),
        "two calls hand the removal the same answer"
    );
    assert!(
        !first.is_null(),
        "the removal is handed an answer rather than no pointer at all"
    );
}
