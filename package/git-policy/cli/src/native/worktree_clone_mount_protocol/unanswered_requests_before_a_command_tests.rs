//! What:
//!  Controls of the check one forwarded command makes:
//!  a repository with no folder reads nothing,
//!  an entry a receipt answers is removed with no device,
//!  an entry no receipt answers is left alone,
//!  and every failure on the path is silent.
//! Why:
//!  The check stands on the command's own path,
//!  so it may neither fail nor wait nor print,
//!  and the cheap negative must be proved by an arm that removes an entry,
//!  or a green run would say nothing about a check that read nothing at all
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  test 15).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! checkAt(folder, receipts); expect(folder.theEntries()).toEqual([]);
//! ```

use super::{
    check_at,
    check_before_a_command,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{
    CloneMountReceipt,
    ReceiptOutcome,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_mount_protocol::unanswered_request_entry::{
    RequestEntry,
    RequestOutcome,
};
use crate::worktree_clone_mount_protocol::unanswered_request_names::EntryName;
use crate::worktree_clone_mount_protocol::unanswered_requests_folder::{
    UnansweredRequests,
    this_account,
};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The clone every control writes.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("1234567890123456789").expect("the control GUID is canonical");
}

/// The nonce a control's request was filed under.
fn nonce() -> RequestNonce {
    return RequestNonce::parse("0123456789abcdef".repeat(2).as_str())
        .expect("the control nonce is 32 hexadecimal digits");
}

/// One waiting entry of this clone.
fn waiting() -> RequestEntry {
    return RequestEntry {
        guid: guid(),
        nonce: nonce(),
        operation: MountOperation::Mount,
        written_at: String::from("2026-11-27T21:14:14.000Z"),
        outcome: RequestOutcome::Waiting,
        dataset: String::from("zpcachyos/nosnap/clones/one"),
    };
}

/// The receipt the handler files for this control's request.
fn answered_receipt() -> CloneMountReceipt {
    return CloneMountReceipt {
        dataset: guid(),
        operation: MountOperation::Mount,
        nonce: nonce(),
        outcome: ReceiptOutcome::Done,
        mount: None,
        failed_step: None,
        errno: None,
        boot: String::from("0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b"),
        time: String::from("2026-11-27T21:14:20.000Z"),
    };
}

/// A fixture folder for the requests and one for the receipts.
fn folders(name: &str) -> (UnansweredRequests, ReceiptsFolder, PathBuf) {
    let root: PathBuf = fixture(name);
    let requests: PathBuf = root.join("git/zfs-unanswered-requests");
    let receipts: PathBuf = root.join("run/results");
    std::fs::create_dir_all(root.join("git").as_path())
        .expect("the control makes the common directory");
    std::fs::create_dir_all(receipts.as_path()).expect("the control makes the results folder");
    std::fs::set_permissions(receipts.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the results folder takes its mode");
    return (
        UnansweredRequests::at(requests, this_account()),
        ReceiptsFolder::at(receipts, this_account()),
        root,
    );
}

/// What:
///  A repository whose folder of unanswered requests is not there reads
///  nothing and says nothing,
///  and the same check removes an entry a receipt answers,
///  which is the arm that proves the negative above can move.
/// Why:
///   Rule `QPC`:
///  a "no difference" or "nothing happened" answer counts only from a probe
///  shown able to show one,
///  so the two arms are one control.
#[test]
fn the_check_reads_nothing_without_a_folder_and_removes_an_answered_entry() {
    let (requests, receipts, root): (UnansweredRequests, ReceiptsFolder, PathBuf) =
        folders("check-no-folder");
    let common_dir: PathBuf = root.join("git");
    check_before_a_command(Some(common_dir.as_path()));
    assert!(
        !common_dir.join(crate::worktree_clone_mount_protocol::unanswered_requests_folder::REQUESTS_FOLDER_NAME)
            .exists(),
        "a command that finds no folder makes none"
    );
    let _entry: EntryName = requests
        .record_the_request(&waiting())
        .expect("the control plants the entry the receipt answers");
    receipts
        .publish(&answered_receipt())
        .expect("the control files the answer");
    assert!(requests.stands(), "the planted entry stands");
    // What: the positive control goes through the folder and the receipts a control names, because the live check
    //  reads `/run` and a control writes no root-owned record.
    // Why:  rule `QPC`: the arm above says something only from a probe shown able to move.
    check_at(&requests, &receipts);
    assert!(
        !requests.stands(),
        "the positive control moved: the answered entry and its folder went"
    );
}

/// An entry no receipt answers is left where it stands,
/// because the trigger it is must survive a command that could not answer it.
#[test]
fn an_entry_no_receipt_answers_is_left_alone() {
    let (requests, receipts, _root): (UnansweredRequests, ReceiptsFolder, PathBuf) =
        folders("check-unanswered");
    let name: EntryName = requests
        .record_the_request(&waiting())
        .expect("the control plants an unanswered entry");
    check_at(&requests, &receipts);
    assert_eq!(requests.the_entries(), vec![(name, waiting())]);
}

/// A receipt for another nonce is not this entry's answer,
/// so a stale result removes nothing.
#[test]
fn a_receipt_for_another_nonce_removes_nothing() {
    let (requests, receipts, _root): (UnansweredRequests, ReceiptsFolder, PathBuf) =
        folders("check-stale");
    let name: EntryName = requests
        .record_the_request(&waiting())
        .expect("the control plants an unanswered entry");
    let stale: CloneMountReceipt = CloneMountReceipt {
        nonce: RequestNonce::parse("f".repeat(32).as_str()).expect("the control nonce is valid"),
        ..answered_receipt()
    };
    receipts
        .publish(&stale)
        .expect("the control files another request's answer");
    check_at(&requests, &receipts);
    assert_eq!(
        requests.the_entries(),
        vec![(name, waiting())],
        "a stale receipt authorizes no removal"
    );
}

/// A folder the account cannot read and a results folder that is not there are
/// both silent:
/// the check changes nothing and does not panic on the command's path.
#[test]
fn a_folder_or_a_receipt_root_that_cannot_be_read_is_silent() {
    let (requests, receipts, _root): (UnansweredRequests, ReceiptsFolder, PathBuf) =
        folders("check-unreadable");
    let name: EntryName = requests
        .record_the_request(&waiting())
        .expect("the control plants an entry");
    let closed: ReceiptsFolder =
        ReceiptsFolder::at(receipts.directory().join("not-there"), this_account());
    check_at(&requests, &closed);
    assert_eq!(
        requests.the_entries(),
        vec![(name, waiting())],
        "a missing results folder answers nothing and removes nothing"
    );
    std::fs::set_permissions(requests.path(), std::fs::Permissions::from_mode(0o000))
        .expect("the control closes the folder");
    check_at(&requests, &receipts);
    std::fs::set_permissions(requests.path(), std::fs::Permissions::from_mode(0o700))
        .expect("the control opens the folder again");
    assert_eq!(
        requests.the_entries(),
        vec![(name, waiting())],
        "a folder the account cannot read is no request and no removal"
    );
}

/// A repository whose layout the wrapper declined names no common directory,
/// so the check reads nothing at all.
#[test]
fn a_repository_with_no_common_directory_reads_nothing() {
    // What: the call takes no path at all, which is the answer a layout this reader declines gives.
    // Why:  it can neither stat, list nor unlink anything, so the command pays nothing for a repository the
    //       wrapper could not place (the design's L18 and A12).
    check_before_a_command(None);
}
