//! What:
//!  Which end of a commit's scratch tree a submitted request leaves it to.
//! Why:
//!  A tree a landing may still read is removed by that landing or by the age sweep,
//!  and a tree the service answered as not landed is the command's own to remove once it has captured the change.
//!  The decision has to reach the removal,
//!  not only the classifier.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(theTreeGoesBackToThisProcess(moved)).toBe(true);
//! ```

/// The decision under control.
use super::the_tree_goes_back_to_this_process;
/// What a submission came to.
use crate::service_client::Submitted;
/// Why a submission failed.
use crate::service_messages::ClientFailure;
/// A tree the command made and the place it is made in.
use crate::service_prepare_ready_shapes::Scratch;
/// Its own disposable directory.
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  A submission the service never answered counts as taken,
///  whatever it said,
///  and a definite answer that the service never took the request gives the tree back,
///  as does an answer that the commit was not landed.
/// Why:
///  A branch write that is still running reads the private index and the conclusion copies in that tree,
///  so a client that removed it would take them from under a landing that then reads nothing.
///  The cases that are no answer at all come first.
#[test]
fn an_unanswered_submission_keeps_the_tree_and_an_answer_that_nothing_landed_gives_it_back() {
    let landed: Result<Submitted, ClientFailure> = Ok(Submitted::Landed { commit: String::from("abc"), already: false });
    let moved: Result<Submitted, ClientFailure> = Ok(Submitted::Moved);
    let again: Result<Submitted, ClientFailure> = Ok(Submitted::PrepareAgain { base: String::from("abc"), landed: Vec::new() });
    assert!(!the_tree_goes_back_to_this_process(&landed), "a landed commit is the service's tree to remove");
    assert!(the_tree_goes_back_to_this_process(&moved), "the rebuild captures the change from this attempt's private index");
    assert!(the_tree_goes_back_to_this_process(&again), "the preparation again captures the change from this attempt's private index");
    for unanswered in [
        ClientFailure::WithdrawalUnconfirmed { blocker: None },
        ClientFailure::ServiceSilent,
        ClientFailure::MayNotHaveLanded,
        ClientFailure::IndexInUse(String::from("Another program")),
        ClientFailure::Unavailable,
        ClientFailure::StepTooSlow,
    ] {
        assert!(!the_tree_goes_back_to_this_process(&Err(unanswered.clone())), "no definite answer, so the tree stays: {unanswered:?}");
    }
    for refused in [
        ClientFailure::StillWithService { minutes: 20, blocker: None },
        ClientFailure::Refused,
        ClientFailure::DidNotStart,
        ClientFailure::NoActivation(crate::service_provision::platform_without_manager()),
    ] {
        assert!(the_tree_goes_back_to_this_process(&Err(refused.clone())), "the service never took the request: {refused:?}");
    }
}

/// What:
///  The tree of an unanswered submission is still there when the command ends,
///  and the trees of a request the service answered as not landed,
///  or confirmed it dropped,
///  are removed by the command.
/// Why:
///  The decision has to reach the removal,
///  not only the classifier:
///  the first tree is the one a running branch write reads,
///  and the others are ones no landing will ever read again.
#[test]
fn a_tree_a_landing_may_still_read_is_left_to_the_service_and_one_it_answered_is_not() {
    let root: PathBuf = fixture("service-commit-tree-ownership");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let mut unanswered: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree");
    let tree: PathBuf = unanswered.0.clone();
    let answered: Result<Submitted, ClientFailure> = Err(ClientFailure::WithdrawalUnconfirmed { blocker: None });
    unanswered.held_by_the_service(!the_tree_goes_back_to_this_process(&answered));
    drop(unanswered);
    assert!(tree.exists(), "the tree a write may still read is left to the service: {}", tree.display());
    let mut refused: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree again");
    let dropped: Result<Submitted, ClientFailure> = Err(ClientFailure::StillWithService { minutes: 20, blocker: None });
    refused.held_by_the_service(!the_tree_goes_back_to_this_process(&dropped));
    drop(refused);
    assert!(!tree.exists(), "a request the service confirmed it dropped leaves the tree to the command: {}", tree.display());
    // The service answered that the commit is not on the branch: the command captures the change from this tree and then
    //  removes it, because the service leaves a tree it answered to that command.
    let mut sent_back: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree of an attempt");
    let again: Result<Submitted, ClientFailure> = Ok(Submitted::PrepareAgain { base: String::from("abc"), landed: Vec::new() });
    sent_back.held_by_the_service(!the_tree_goes_back_to_this_process(&again));
    drop(sent_back);
    assert!(!tree.exists(), "a tree the service answered as not landed is removed by the command once it read it: {}", tree.display());
    remove(root.as_path());
}
