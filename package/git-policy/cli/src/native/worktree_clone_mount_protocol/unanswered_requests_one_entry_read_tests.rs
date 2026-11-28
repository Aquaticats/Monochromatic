//! What:
//!  Controls of the read of one entry:
//!  the link,
//!  the widened mode,
//!  another account's file,
//!  the page's bound,
//!  and the garbage a reader must refuse.
//! Why:
//!  The read stands on a command's own path before Git runs,
//!  so a file it believed wrongly is a clone nudged for ever,
//!  and a file it followed could be another one entirely
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  test 15).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(() => readOne(folder, name)).toThrow(); // a link is no entry
//! ```

use super::read_one;
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_mount_protocol::unanswered_request_entry::{
    MAXIMUM_ENTRY_BYTES,
    RequestEntry,
    RequestOutcome,
};
use crate::worktree_clone_mount_protocol::unanswered_request_names::EntryName;
use crate::worktree_clone_mount_protocol::unanswered_requests_folder::{
    UnansweredRequests,
    this_account,
};
use crate::worktree_clone_mount_protocol::unanswered_requests_refusals::RequestsRefusal;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The clone every control writes.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("1234567890123456789").expect("the control GUID is canonical");
}

/// One waiting entry of this clone.
fn waiting() -> RequestEntry {
    return RequestEntry {
        guid: guid(),
        nonce: RequestNonce::parse("0123456789abcdef".repeat(2).as_str())
            .expect("the control nonce is 32 hexadecimal digits"),
        operation: MountOperation::Mount,
        written_at: String::from("2026-11-27T21:14:14.000Z"),
        outcome: RequestOutcome::Waiting,
        dataset: String::from("zpcachyos/nosnap/clones/one"),
    };
}

/// A fixture folder holding one entry this account wrote.
fn folder_with_one_entry(name: &str) -> (UnansweredRequests, EntryName) {
    let root: PathBuf = fixture(name);
    let folder: UnansweredRequests =
        UnansweredRequests::at(root.join("zfs-unanswered-requests"), this_account());
    let attempt: EntryName = folder
        .record_the_request(&waiting())
        .expect("the entry is published");
    return (folder, attempt);
}

/// An entry the account wrote reads back as itself.
#[test]
fn an_entry_the_account_wrote_reads_back() {
    let (folder, attempt): (UnansweredRequests, EntryName) =
        folder_with_one_entry("entry-read-round-trip");
    assert!(
        matches!(read_one(folder.path(), this_account(), attempt), Ok(read) if read == waiting()),
        "the entry reads back as the one the wait wrote"
    );
}

/// An entry a link names is refused,
/// because a reader that followed one could be led to another file or wait on
/// a pipe.
#[test]
fn an_entry_a_link_names_is_refused() {
    let (folder, attempt): (UnansweredRequests, EntryName) =
        folder_with_one_entry("entry-read-link");
    let elsewhere: PathBuf = folder.path().join("elsewhere");
    std::fs::write(elsewhere.as_path(), waiting().text()).expect("the control writes a real entry");
    std::fs::set_permissions(elsewhere.as_path(), std::fs::Permissions::from_mode(0o600))
        .expect("the control makes the target an entry the account's own, so only the open's own kind can refuse it");
    let entry_path: PathBuf = folder.path().join(attempt.file_name());
    std::fs::remove_file(entry_path.as_path()).expect("the control takes the entry away");
    std::os::unix::fs::symlink(elsewhere.as_path(), entry_path.as_path())
        .expect("the control plants a link");
    let refusal: RequestsRefusal =
        read_one(folder.path(), this_account(), attempt).expect_err("a link is no entry");
    assert!(
        refusal.message().contains("not a record this route wrote"),
        "{}",
        refusal.message()
    );
}

/// A file whose mode lets others read it,
/// and one another account owns,
/// are both refused before a byte is parsed.
#[test]
fn a_file_of_another_account_or_another_mode_is_refused() {
    let (folder, attempt): (UnansweredRequests, EntryName) =
        folder_with_one_entry("entry-read-owner");
    let entry_path: PathBuf = folder.path().join(attempt.file_name());
    std::fs::set_permissions(entry_path.as_path(), std::fs::Permissions::from_mode(0o644))
        .expect("the control widens the entry");
    let widened: RequestsRefusal =
        read_one(folder.path(), this_account(), attempt).expect_err("others may not read it");
    assert!(
        widened
            .message()
            .contains("only the account may read or write it"),
        "{}",
        widened.message()
    );
    std::fs::set_permissions(entry_path.as_path(), std::fs::Permissions::from_mode(0o600))
        .expect("the control narrows the entry again");
    let another: UnansweredRequests =
        UnansweredRequests::at(folder.path().to_path_buf(), this_account().wrapping_add(1));
    let foreign: RequestsRefusal =
        read_one(another.path(), another.account(), attempt).expect_err("it is not that account's");
    assert!(
        foreign.message().contains(&this_account().to_string()),
        "{}",
        foreign.message()
    );
}

/// A file larger than one page is refused,
/// and a file that is not an entry is refused with what an entry is.
#[test]
fn a_file_over_one_page_or_of_another_shape_is_refused() {
    let (folder, attempt): (UnansweredRequests, EntryName) =
        folder_with_one_entry("entry-read-bounds");
    let entry_path: PathBuf = folder.path().join(attempt.file_name());
    std::fs::write(entry_path.as_path(), "x".repeat(MAXIMUM_ENTRY_BYTES + 1))
        .expect("the control writes a page and a byte");
    std::fs::set_permissions(entry_path.as_path(), std::fs::Permissions::from_mode(0o600))
        .expect("the control keeps the entry the account's alone");
    let long: RequestsRefusal =
        read_one(folder.path(), this_account(), attempt).expect_err("a page is the bound");
    assert!(
        long.message().contains(&MAXIMUM_ENTRY_BYTES.to_string()),
        "{}",
        long.message()
    );
    std::fs::write(entry_path.as_path(), b"not an entry\n").expect("the control writes garbage");
    std::fs::set_permissions(entry_path.as_path(), std::fs::Permissions::from_mode(0o600))
        .expect("the control keeps the entry the account's alone");
    let garbage: RequestsRefusal =
        read_one(folder.path(), this_account(), attempt).expect_err("garbage is no entry");
    assert!(
        garbage.message().contains("cannot be read"),
        "the refusal names the file and what an entry is: {}",
        garbage.message()
    );
}
