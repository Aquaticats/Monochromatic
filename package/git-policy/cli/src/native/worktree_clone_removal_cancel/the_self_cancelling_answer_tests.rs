//! What:
//!  The controls of the account's answer to a removal that a holder stopped:
//!  a binding that cannot be read answers a sentence and writes nothing,
//!  and each design stop point is named by the cancellation's own words.
//! Why:
//!  The answer takes every write the reversal makes,
//!  so the one path a host control may reach is its refusal:
//!  a reversal needs a mount request and a receipt,
//!  which no control on a person's machine may make.
//!  The point mapping is asserted here because a stop point read as another
//!  would print a report about the wrong state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theAnswer().cancelTheRemoval(stopped).sentence).toContain('could not be read');
//! ```

use super::{
    TheSelfCancellingAnswer,
    where_it_stopped,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_removal_cancel::the_cancelled_removal::WhereItStopped;
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
fn a_slot_with_no_binding_answers_its_own_sentence_and_writes_nothing() {
    let common: PathBuf = fixture("a_slot_with_no_binding_answers_its_own_sentence");
    let answer: TheSelfCancellingAnswer = TheSelfCancellingAnswer;
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
        "the sentence names the slot whose record could not be read: {}",
        refused.sentence
    );
    assert!(
        !crate::worktree_clone_binding_record::registration_home::plans_folder(common.as_path())
            .exists(),
        "a refusal writes no plan folder"
    );
}

#[test]
fn each_design_stop_point_is_named_by_the_cancellations_own_word() {
    let named: [(TheStopPoint, WhereItStopped); 3] = [
        (TheStopPoint::InStaging, WhereItStopped::InStaging),
        (
            TheStopPoint::ByAScanBeforeADelete,
            WhereItStopped::ByAScanBeforeADelete,
        ),
        (
            TheStopPoint::AtTheHeadDestroy,
            WhereItStopped::AtTheHeadDestroy,
        ),
    ];
    for (seam, own) in named {
        assert_eq!(
            where_it_stopped(seam),
            own,
            "{seam:?} is the cancellation's {own:?}"
        );
        assert_eq!(
            seam.clause(),
            own.clause(),
            "a report and a removal's sentence say the same thing about {seam:?}"
        );
    }
}
