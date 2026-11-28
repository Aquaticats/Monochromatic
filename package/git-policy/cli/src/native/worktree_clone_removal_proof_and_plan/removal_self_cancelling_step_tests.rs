//! What:
//!  The controls of the one call each stop point makes:
//!  no answer wired,
//!  an answer that cancels the removal,
//!  and an answer that cannot.
//! Why:
//!  A stop point's two shapes are the whole of what this step decides,
//!  and a control reads them without a pool,
//!  a namespace or a repository,
//!  so the release's own reading of a stop is what is asserted rather than a
//!  fixture's guess at it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(cancelledAt({ cancelling: undefined, ... })).toBeUndefined();
//! ```

use super::{
    TheStopTheRemovalTakes,
    cancelled_at,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::{
    TheCancellationFailed,
    TheCancelledRemoval,
    TheRemovalCancelling,
    TheStopPoint,
    TheStoppedRemoval,
};
use std::cell::RefCell;
use std::path::Path;

/// The lines an answer that cancels a removal reports.
const THE_ANSWERS_LINES: [&str; 2] = [
    "removal of `one` cancelled itself: a holder kept the clone's unmount busy, so nothing was destroyed and every snapshot survives. The clone is mounted at its slot.",
    "The destruction plan at /tmp/one stands as the loss record.",
];

/// The sentence an answer that cannot cancel reports.
const THE_FAILURE_SENTENCE: &str = "the handler answered no receipt before this removal's own deadline.";

/// What:
///  An answer a control scripts,
///  which either cancels the removal or says why it cannot.
/// Why:
///  The removal's step reads only the answer's lines and status,
///  so a control that keeps the request proves what the step asked without a
///  pool or a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = { refuses: undefined, asked: [] };
/// ```
struct ScriptedAnswer {
    refuses: Option<&'static str>,
    asked: RefCell<Vec<(String, usize, TheStopPoint)>>,
}

impl ScriptedAnswer {
    fn cancelling() -> ScriptedAnswer {
        return ScriptedAnswer {
            refuses: None,
            asked: RefCell::new(Vec::new()),
        };
    }

    fn refusing() -> ScriptedAnswer {
        return ScriptedAnswer {
            refuses: Some(THE_FAILURE_SENTENCE),
            asked: RefCell::new(Vec::new()),
        };
    }
}

impl TheRemovalCancelling for ScriptedAnswer {
    fn cancel_the_removal(
        &self,
        request: &TheStoppedRemoval<'_>,
    ) -> Result<TheCancelledRemoval, TheCancellationFailed> {
        self.asked.borrow_mut().push((
            String::from(request.slot),
            request.destroyed.len(),
            request.stop,
        ));
        return match self.refuses {
            None => Ok(TheCancelledRemoval {
                lines: THE_ANSWERS_LINES
                    .iter()
                    .map(|line| return String::from(*line))
                    .collect(),
                code: 1,
            }),
            Some(sentence) => Err(TheCancellationFailed {
                sentence: String::from(sentence),
            }),
        };
    }
}

/// One stop at the scan before a delete over the fixture's own common folder.
fn the_stop<'a>(
    cancelling: Option<&'a dyn TheRemovalCancelling>,
    plan: Option<&'a DestructionPlan>,
    holders: &'a [String],
) -> TheStopTheRemovalTakes<'a> {
    return TheStopTheRemovalTakes {
        cancelling,
        common_dir: Path::new("/tmp/the-removal-step"),
        slot: "one",
        plan,
        holders,
        stop: TheStopPoint::ByAScanBeforeADelete,
    };
}

#[test]
fn a_removal_with_no_answer_wired_keeps_the_stop_it_had() {
    let holders: Vec<String> = vec![String::from("mnt:[4026532001] holds /srv/holder")];
    assert_eq!(
        cancelled_at(&the_stop(None, None, holders.as_slice())),
        None,
        "a control that hands no answer must read the stop the removal made rather than a cancellation"
    );
}

#[test]
fn an_answer_that_cancels_the_removal_prints_its_own_report() {
    let answer: ScriptedAnswer = ScriptedAnswer::cancelling();
    let holders: Vec<String> = vec![String::from("mnt:[4026532001] holds /srv/holder")];
    let stopped: RemovalStop = cancelled_at(&the_stop(
        Some(&answer),
        None,
        holders.as_slice(),
    ))
    .expect("an answer is wired, so the stop is the answer's");
    let RemovalStop::TheRemovalCancelledItself {
        said,
    } = &stopped
    else {
        panic!("the answer cancelled the removal, and the stop is {stopped:?}");
    };
    assert_eq!(
        said.as_str(),
        THE_ANSWERS_LINES.join("\n"),
        "the report a person reads is the answer's own, line for line"
    );
    assert_eq!(
        stopped.message(),
        THE_ANSWERS_LINES.join("\n"),
        "the removal prints the report as it stands rather than wrapping it"
    );
    assert_eq!(
        answer.asked.borrow().as_slice(),
        [(String::from("one"), 0, TheStopPoint::ByAScanBeforeADelete)].as_slice(),
        "the answer is asked under the removal's slot, its destroyed names and its stop point"
    );
}

#[test]
fn an_answer_that_cannot_cancel_names_the_stop_and_its_own_sentence() {
    let answer: ScriptedAnswer = ScriptedAnswer::refusing();
    let stopped: RemovalStop = cancelled_at(&the_stop(Some(&answer), None, &[]))
        .expect("an answer is wired, so the stop is the answer's");
    let RemovalStop::TheRemovalCouldNotCancelItself {
        stop,
        said,
    } = &stopped
    else {
        panic!("the answer could not cancel, and the stop is {stopped:?}");
    };
    assert_eq!(
        stop, &TheStopPoint::ByAScanBeforeADelete.clause(),
        "the sentence names where the removal stopped"
    );
    assert_eq!(said, THE_FAILURE_SENTENCE);
    assert!(
        stopped.message().contains(THE_FAILURE_SENTENCE),
        "the sentence a person reads carries the answer's own words"
    );
    assert!(
        stopped.message().contains("cancel-worktree-removal"),
        "a removal that could not put its clone back names the command that can"
    );
}

#[test]
fn a_plan_that_witnessed_no_destroy_tells_its_answer_so() {
    let answer: ScriptedAnswer = ScriptedAnswer::cancelling();
    let plan: DestructionPlan = crate::worktree_clone_removal_proof_and_plan::fixtures::sample_plan();
    let _ = cancelled_at(&the_stop(Some(&answer), Some(&plan), &[]));
    let asked = answer.asked.borrow();
    let (_, destroyed, _) = asked.first().expect("the answer was asked");
    assert_eq!(
        *destroyed, 0,
        "a plan whose first write stands records no destroyed snapshot, so its answer is told none"
    );
}
