//! What:
//!  The control of the value the forwarded removal path takes from this folder:
//!  a stop point that asks it is answered by the account's own cancellation,
//!  not by a stand-in.
//! Why:
//!  The value decides nothing of its own,
//!  so what it owes the forwarded path is the answer itself:
//!  the sentence a slot with no binding gets is the cancellation's own,
//!  and a different value handed over here would print words no cancellation
//!  wrote.
//!  A pointer identity would prove nothing,
//!  because the answer holds no state and two values of it share no address
//!  a control may read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theRemovalsCancelling().cancelTheRemoval(noSuchSlot).sentence).toContain('a-slot-nothing-recorded');
//! ```

use super::the_removals_cancelling;
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::{
    TheRemovalCancelling,
    TheStopPoint,
    TheStoppedRemoval,
};
use std::path::PathBuf;

/// The slot the control asks about,
/// which has no binding record in its own repository.
const NO_SUCH_SLOT: &str = "a-slot-nothing-recorded";

#[test]
fn the_answer_handed_over_is_the_cancellations_own() {
    let common: PathBuf = fixture("the_answer_handed_over_is_the_cancellations_own");
    let answer: &dyn TheRemovalCancelling = the_removals_cancelling();
    let asked: TheStoppedRemoval<'_> = TheStoppedRemoval {
        common_dir: common.as_path(),
        slot: NO_SUCH_SLOT,
        plan: None,
        destroyed: &[],
        holders: &[],
        stop: TheStopPoint::ByAScanBeforeADelete,
    };
    let refused = answer
        .cancel_the_removal(&asked)
        .expect_err("a slot with no binding cannot be reversed");
    assert!(
        refused.sentence.contains(NO_SUCH_SLOT),
        "the value handed over answers with the cancellation's own sentence, which names the slot whose record could not be read: {}",
        refused.sentence
    );
    assert!(
        !plans_folder(common.as_path()).exists(),
        "a refusal writes no plan folder"
    );
}
