//! What:
//!  Controls of the results directory and its receipt files:
//!  the GUID's name and the mode,
//!  a whole publish and its read,
//!  the absence of a receipt,
//!  and every file the route must not read as one.
//! Why:
//!  A caller acts on a receipt only when it can prove root filed it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  "Decided defaults"),
//!  so the controls hold the accept and each refusal the same way.
//!  The fixture directory is owned by the account running the tests,
//!  and the folder is told that owner,
//!  while the live folder is held to root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = ReceiptsFolder.at(fixture('receipts'), process.getuid()); folder.publish(receipt);
//! ```

use super::{
    DIRECTORY_MODE, RECEIPT_MODE, RESULTS_DIRECTORY, ReceiptsFolder, ReceiptsRefusal,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{
    CloneMountReceipt, MAXIMUM_RECEIPT_BYTES, ReceiptMountIdentity, ReceiptOutcome,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The dataset every control files under.
const GUID: u64 = 42;

/// The receipt a control files.
fn receipt() -> CloneMountReceipt {
    return CloneMountReceipt {
        dataset: DatasetGuid::parse(GUID.to_string().as_str()).expect("the control GUID is canonical"),
        operation: MountOperation::Mount,
        nonce: RequestNonce::parse("0".repeat(32).as_str()).expect("the control nonce is valid"),
        outcome: ReceiptOutcome::Done,
        mount: Some(ReceiptMountIdentity {
            mount_id: 7,
            device: 2049,
            inode: 3,
            source: String::from("zpcachyos/nosnap/clones/one"),
            canonical_path: String::from("/home/user/repo/.claude/worktrees/one"),
        }),
        failed_step: None,
        errno: None,
        boot: String::from("0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b"),
        time: String::from("2026-11-21T15:00:00Z"),
    };
}

/// This account's own number,
/// which owns every fixture directory.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// A fresh fixture folder with its designed mode,
/// and the folder object that expects this account.
fn live_folder(name: &str) -> ReceiptsFolder {
    let root: PathBuf = fixture(name);
    std::fs::set_permissions(root.as_path(), std::fs::Permissions::from_mode(DIRECTORY_MODE)).expect("the fixture takes the directory mode");
    return ReceiptsFolder::at(root, this_uid());
}

/// The live folder is the designed path and requires root.
#[test]
fn the_live_folder_is_the_designed_root_owned_path() {
    let folder: ReceiptsFolder = ReceiptsFolder::live();
    assert_eq!(folder.directory().to_str(), Some(RESULTS_DIRECTORY));
    assert_eq!(RESULTS_DIRECTORY, "/run/git-worktree-clone-mount-results");
    assert_eq!(DIRECTORY_MODE, 0o755);
    assert_eq!(RECEIPT_MODE, 0o644);
}

/// A published receipt is the GUID's file,
/// at `0644`,
/// and reads back whole;
/// the staging sibling is gone.
#[test]
fn a_receipt_is_filed_under_its_guid_and_reads_back() {
    let folder: ReceiptsFolder = live_folder("receipts-round-trip");
    let receipt: CloneMountReceipt = receipt();
    folder.publish(&receipt).expect("the receipt is filed");
    let path: PathBuf = folder.directory().join(GUID.to_string());
    let mode: u32 = std::fs::metadata(path.as_path()).expect("the receipt exists").permissions().mode() & 0o777;
    assert_eq!(mode, RECEIPT_MODE);
    assert_eq!(std::fs::read_to_string(path.as_path()).expect("the receipt is text"), receipt.text());
    assert!(!folder.directory().join(folder.staging_name(receipt.dataset)).exists(), "the staging sibling is renamed");
    assert_eq!(folder.read(receipt.dataset).expect("the receipt is read"), Some(receipt));
    let other: DatasetGuid = DatasetGuid::parse("43").expect("43 is canonical");
    assert_eq!(folder.read(other).expect("another GUID has no receipt"), None);
}

/// A second publish replaces the dataset's own receipt whole,
/// which is what a live clone's next result does.
#[test]
fn a_second_publish_replaces_the_same_receipt() {
    let folder: ReceiptsFolder = live_folder("receipts-replace");
    let mut first: CloneMountReceipt = receipt();
    folder.publish(&first).expect("the first receipt is filed");
    first.outcome = ReceiptOutcome::Refused;
    first.mount = None;
    first.failed_step = Some(String::from("read-intent"));
    folder.publish(&first).expect("the second receipt is filed");
    assert_eq!(folder.read(first.dataset).expect("the receipt is read"), Some(first));
    assert_eq!(std::fs::read_dir(folder.directory()).expect("the directory lists").count(), 1);
}

/// A directory with another mode,
/// and one owned by another account,
/// are refused before any receipt is read.
#[test]
fn a_wrong_directory_mode_or_owner_is_refused() {
    let folder: ReceiptsFolder = live_folder("receipts-directory-mode");
    std::fs::set_permissions(folder.directory(), std::fs::Permissions::from_mode(0o700)).expect("the fixture takes another mode");
    let refusal: ReceiptsRefusal = folder.read(DatasetGuid::parse("42").expect("canonical")).expect_err("another mode is refused");
    assert!(matches!(refusal, ReceiptsRefusal::DirectoryMode { found: 0o700, .. }), "{refusal:?}");
    let wrong_owner: ReceiptsFolder = ReceiptsFolder::at(folder.directory().to_path_buf(), this_uid() + 1);
    let owner_refusal: ReceiptsRefusal = wrong_owner.read(DatasetGuid::parse("42").expect("canonical")).expect_err("another owner is refused");
    assert!(matches!(owner_refusal, ReceiptsRefusal::DirectoryOwner { wanted, .. } if wanted == this_uid() + 1));
}

/// A receipt with another mode is refused,
/// and the refusal names the mode found.
#[test]
fn a_receipt_with_another_mode_is_refused() {
    let folder: ReceiptsFolder = live_folder("receipts-file-mode");
    folder.publish(&receipt()).expect("the receipt is filed");
    let path: PathBuf = folder.directory().join(GUID.to_string());
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o600)).expect("the receipt takes another mode");
    let refusal: ReceiptsRefusal = folder.read(receipt().dataset).expect_err("another mode is refused");
    assert!(matches!(refusal, ReceiptsRefusal::FileMode { found: 0o600, .. }), "{refusal:?}");
}

/// A link named after a GUID is not a receipt,
/// and the read does not follow it.
#[test]
fn a_link_named_after_a_guid_is_refused() {
    let folder: ReceiptsFolder = live_folder("receipts-link");
    let elsewhere: PathBuf = folder.directory().join("elsewhere");
    std::fs::write(elsewhere.as_path(), receipt().text()).expect("the other file is written");
    std::fs::set_permissions(elsewhere.as_path(), std::fs::Permissions::from_mode(RECEIPT_MODE)).expect("the other file takes the mode");
    std::os::unix::fs::symlink(elsewhere.as_path(), folder.directory().join(GUID.to_string())).expect("the link is made");
    let refusal: ReceiptsRefusal = folder.read(receipt().dataset).expect_err("a link is not a receipt");
    assert!(matches!(refusal, ReceiptsRefusal::File { .. }), "{refusal:?}");
}

/// A folder named after a GUID is not a regular receipt.
#[test]
fn a_folder_named_after_a_guid_is_refused() {
    let folder: ReceiptsFolder = live_folder("receipts-folder-name");
    std::fs::create_dir(folder.directory().join(GUID.to_string())).expect("the folder is made");
    let refusal: ReceiptsRefusal = folder.read(receipt().dataset).expect_err("a folder is not a receipt");
    assert!(matches!(refusal, ReceiptsRefusal::NotRegular { .. }), "{refusal:?}");
}

/// A file over the byte bound,
/// one that is not text,
/// and one that is not a receipt are each refused.
#[test]
fn each_unreadable_receipt_is_refused() {
    let folder: ReceiptsFolder = live_folder("receipts-shapes");
    let path: PathBuf = folder.directory().join(GUID.to_string());
    std::fs::write(path.as_path(), vec![b'x'; MAXIMUM_RECEIPT_BYTES + 1]).expect("the large file is written");
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(RECEIPT_MODE)).expect("the large file takes the mode");
    assert!(matches!(folder.read(receipt().dataset), Err(ReceiptsRefusal::TooLarge { .. })));
    std::fs::write(path.as_path(), [0xff_u8, 0xfe_u8]).expect("the binary file is written");
    assert!(matches!(folder.read(receipt().dataset), Err(ReceiptsRefusal::NotText { .. })));
    std::fs::write(path.as_path(), "v1\nkind something-else\n").expect("the foreign file is written");
    assert!(matches!(folder.read(receipt().dataset), Err(ReceiptsRefusal::NotAReceipt { .. })));
}

/// A staging sibling is never read as a receipt,
/// because no GUID's name is its name.
#[test]
fn the_staging_sibling_is_not_a_receipt() {
    let folder: ReceiptsFolder = live_folder("receipts-staging");
    let receipt: CloneMountReceipt = receipt();
    let staging: String = folder.staging_name(receipt.dataset);
    assert!(staging.starts_with(&GUID.to_string()));
    assert_ne!(staging, folder.entry_name(receipt.dataset));
    assert_eq!(folder.entry_name(receipt.dataset), GUID.to_string());
}

/// Every refusal carries the way forward and says nothing acted on the file.
#[test]
fn every_refusal_carries_the_way_forward() {
    let folder: ReceiptsFolder = ReceiptsFolder::at(PathBuf::from("/nonexistent/receipts"), 0);
    let missing_refusal: ReceiptsRefusal = folder.read(DatasetGuid::parse("42").expect("canonical")).expect_err("a missing directory is refused");
    let message: String = missing_refusal.message();
    assert!(message.contains("Nothing was matched"), "{message}");
    assert!(message.contains("journalctl -u reconcile-git-worktree-clone-mounts.service"), "{message}");
}
