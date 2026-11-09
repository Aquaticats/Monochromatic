//! What:
//!  Which end of a commit's scratch tree a submitted request leaves it to.
//! Why:
//!  A tree a landing may still read is removed by that landing or by the age sweep,
//!  and a tree the service confirmed it never took is the command's own to remove.
//!  The decision has to reach the removal,
//!  not only the classifier.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(theServiceNeverTook(withdrawalUnconfirmed)).toBe(false);
//! ```

/// The decision under control.
use super::the_service_never_took;
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
///  and only a definite answer that the service never took the request gives the tree back.
///  An answer that asks for another preparation is the other case that gives it back:
///  the service keeps that tree for the retry and this command is the one that removes it.
/// Why:
///  A branch write that is still running reads the private index and the conclusion copies in that tree,
///  so a client that removed it would take them from under a landing that then reads nothing.
///  The cases that are no answer at all come first.
#[test]
fn an_unanswered_submission_keeps_the_tree_and_a_definite_refusal_does_not() {
    let landed: Result<Submitted, ClientFailure> = Ok(Submitted::Landed { commit: String::from("abc"), already: false });
    let moved: Result<Submitted, ClientFailure> = Ok(Submitted::Moved);
    let again: Result<Submitted, ClientFailure> = Ok(Submitted::PrepareAgain { base: String::from("abc"), landed: Vec::new() });
    assert!(!the_service_never_took(&landed), "a landed commit is the service's tree to remove");
    assert!(the_service_never_took(&moved), "the service keeps the tree of a branch that moved, and the retry reads it");
    assert!(the_service_never_took(&again), "the service keeps the tree of a commit it prepares again, and the retry reads it");
    for unanswered in [
        ClientFailure::WithdrawalUnconfirmed { blocker: None },
        ClientFailure::ServiceSilent,
        ClientFailure::MayNotHaveLanded,
        ClientFailure::IndexInUse(String::from("Another program")),
        ClientFailure::Unavailable,
        ClientFailure::StepTooSlow,
    ] {
        assert!(!the_service_never_took(&Err(unanswered.clone())), "no definite answer, so the tree stays: {unanswered:?}");
    }
    for refused in [
        ClientFailure::StillWithService { minutes: 20, blocker: None },
        ClientFailure::Refused,
        ClientFailure::DidNotStart,
        ClientFailure::NoActivation(crate::service_provision::platform_without_manager()),
    ] {
        assert!(the_service_never_took(&Err(refused.clone())), "the service never took the request: {refused:?}");
    }
}

/// What:
///  The tree of an unanswered submission is still there when the command ends,
///  and the tree of a request the service confirmed it dropped is removed by the command.
/// Why:
///  The decision has to reach the removal,
///  not only the classifier:
///  the first tree is the one a running branch write reads,
///  and the second is one no landing will ever read.
#[test]
fn a_tree_a_landing_may_still_read_is_left_to_the_service_and_one_it_never_took_is_not() {
    let root: PathBuf = fixture("service-commit-tree-ownership");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let mut unanswered: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree");
    let tree: PathBuf = unanswered.0.clone();
    let answered: Result<Submitted, ClientFailure> = Err(ClientFailure::WithdrawalUnconfirmed { blocker: None });
    unanswered.held_by_the_service(!the_service_never_took(&answered));
    drop(unanswered);
    assert!(tree.exists(), "the tree a write may still read is left to the service: {}", tree.display());
    let mut refused: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree again");
    let dropped: Result<Submitted, ClientFailure> = Err(ClientFailure::StillWithService { minutes: 20, blocker: None });
    refused.held_by_the_service(!the_service_never_took(&dropped));
    drop(refused);
    assert!(!tree.exists(), "a request the service confirmed it dropped leaves the tree to the command: {}", tree.display());
    // An answer that asks for another preparation keeps the tree in the service and hands its removal to this command, which is
    //  the one that still reads it for the retry: the write-tree the retry makes needs the private index inside it.
    let mut preparing: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree once more");
    let prepared_again: Result<Submitted, ClientFailure> = Ok(Submitted::PrepareAgain { base: String::from("abc"), landed: Vec::new() });
    preparing.held_by_the_service(!the_service_never_took(&prepared_again));
    drop(preparing);
    assert!(!tree.exists(), "the tree of a commit prepared again is this command's to remove: {}", tree.display());
    remove(root.as_path());
}
