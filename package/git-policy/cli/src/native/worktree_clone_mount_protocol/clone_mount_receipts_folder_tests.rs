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
    CloneMountReceipt, MAXIMUM_PATH_BYTES, MAXIMUM_RECEIPT_BYTES, ReceiptMountIdentity, ReceiptOutcome,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The dataset every control files under.
const GUID: u64 = 42;

/// Another dataset,
/// which has no receipt of its own.
const ANOTHER_GUID: u64 = 43;

/// The receipt a control files.
fn receipt() -> CloneMountReceipt {
    return CloneMountReceipt {
        dataset: DatasetGuid::parse(GUID.to_string().as_str()).expect("the control GUID is canonical"),
        operation: MountOperation::Mount,
        nonce: RequestNonce::parse("0".repeat(NONCE_HEX_DIGITS).as_str()).expect("the control nonce is valid"),
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

/// The dataset every control reads through,
/// as the value the receipt is filed under.
fn control_guid() -> DatasetGuid {
    return DatasetGuid::parse(GUID.to_string().as_str()).expect("the control GUID is canonical");
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
    let other: DatasetGuid = DatasetGuid::parse(ANOTHER_GUID.to_string().as_str()).expect("the other GUID is canonical");
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
    let refusal: ReceiptsRefusal = folder.read(control_guid()).expect_err("another mode is refused");
    assert!(matches!(refusal, ReceiptsRefusal::DirectoryMode { found: 0o700, .. }), "{refusal:?}");
    let wrong_owner: ReceiptsFolder = ReceiptsFolder::at(folder.directory().to_path_buf(), this_uid() + 1);
    let owner_refusal: ReceiptsRefusal = wrong_owner.read(control_guid()).expect_err("another owner is refused");
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

/// A receipt filed under a GUID but carrying another GUID is refused.
#[test]
fn a_receipt_for_another_dataset_is_refused() {
    let folder: ReceiptsFolder = live_folder("receipts-wrong-dataset");
    let mut receipt: CloneMountReceipt = receipt();
    receipt.dataset = DatasetGuid::parse(ANOTHER_GUID.to_string().as_str()).expect("the other GUID is canonical");
    let text: String = receipt.text();
    let path: PathBuf = folder.directory().join(GUID.to_string());
    std::fs::write(path.as_path(), text).expect("the foreign receipt is written");
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(RECEIPT_MODE)).expect("the foreign receipt takes the mode");
    let refusal: ReceiptsRefusal = folder.read(control_guid()).expect_err("another GUID's receipt is refused");
    assert!(matches!(refusal, ReceiptsRefusal::WrongDataset { ref found, .. } if found == "43"), "{refusal:?}");
}

/// A FIFO at a GUID's name is refused without blocking the reader.
#[test]
fn a_fifo_at_a_receipt_name_is_refused() {
    let folder: ReceiptsFolder = live_folder("receipts-fifo");
    let path: PathBuf = folder.directory().join(GUID.to_string());
    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("the path is text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call in a fixture folder this test owns.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o644) };
    assert_eq!(made, 0, "the control makes its FIFO: {}", std::io::Error::last_os_error());
    let refusal: ReceiptsRefusal = folder.read(receipt().dataset).expect_err("a FIFO is not a receipt");
    assert!(matches!(refusal, ReceiptsRefusal::NotRegular { .. }), "{refusal:?}");
}

/// A staging sibling is never read as a receipt,
/// because no GUID's name is its name,
/// and a killed writer's sibling carries its own process id,
/// so the next writer's name is free.
#[test]
fn the_staging_sibling_is_never_a_receipt_name() {
    let folder: ReceiptsFolder = live_folder("receipts-staging-leftover");
    let receipt: CloneMountReceipt = receipt();
    let stale_staging: PathBuf = folder.directory().join(format!("{}.staging.1", GUID));
    std::fs::write(stale_staging.as_path(), "v1\n").expect("the killed writer's sibling is written");
    folder.publish(&receipt).expect("a killed writer's sibling blocks no later publish");
    assert_eq!(folder.read(receipt.dataset).expect("the new receipt is read"), Some(receipt));
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

/// What:
///  A first publish in an empty parent makes the results directory at its
///  designed mode and files the receipt,
///  which is what a fresh boot needs.
/// Why:
///  Nothing else creates the results directory,
///  so a publish that refused without it could not work after a boot,
///  and the guest arm made the directory itself and hid exactly that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = ReceiptsFolder.at(empty.join('never-made'), uid); folder.publish(receipt); expect(folder.read(guid)).toEqual(receipt);
/// ```
#[test]
fn a_first_publish_makes_its_directory() {
    let parent: PathBuf = fixture("receipts-fresh-boot");
    let directory: PathBuf = parent.join("never-made");
    let folder: ReceiptsFolder = ReceiptsFolder::at(directory.clone(), this_uid());
    let receipt: CloneMountReceipt = receipt();
    folder.publish(&receipt).expect("the first publish makes its directory");
    let mode: u32 = std::fs::metadata(directory.as_path()).expect("the directory exists").permissions().mode() & 0o777;
    assert_eq!(mode, DIRECTORY_MODE);
    assert_eq!(folder.read(receipt.dataset).expect("the receipt is read"), Some(receipt));
}

/// What:
///  A receipt whose canonical path is at the path bound is one the reader reads
///  back,
///  and it fits the receipt's own byte bound.
/// Why:
///   The file bound and the path bound must agree,
///  or a deep checkout leaves the handler's own answer unreadable and nothing
///  says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// receipt.mount.canonicalPath = '/' + 'a'.repeat(MAXIMUM_PATH_BYTES - 1); publish(receipt); expect(read(guid)).toEqual(receipt);
/// ```
#[test]
fn a_receipt_with_the_largest_admitted_path_reads_back() {
    let folder: ReceiptsFolder = live_folder("receipts-path-bound");
    let mut receipt: CloneMountReceipt = receipt();
    let longest: String = format!("/{}", "a".repeat(MAXIMUM_PATH_BYTES - 1));
    assert_eq!(longest.len(), MAXIMUM_PATH_BYTES);
    receipt.mount = Some(ReceiptMountIdentity {
        mount_id: u64::MAX,
        device: u64::MAX,
        inode: u64::MAX,
        source: "z".repeat(255),
        canonical_path: longest,
    });
    assert!(receipt.text().len() <= MAXIMUM_RECEIPT_BYTES, "the longest receipt the writer builds fits its own bound");
    folder.publish(&receipt).expect("the receipt at the bound is filed");
    assert_eq!(folder.read(receipt.dataset).expect("the receipt at the bound is read back"), Some(receipt));
}

/// What:
///  A receipt whose text its own reader refuses is not written at all.
/// Why:
///  The writer would otherwise file a result nobody can read,
///  and the caller would wait out its deadline on a file that already exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// receipt.mount.source = 'a name with a newline\n'; expect(() => folder.publish(receipt)).toThrow(); expect(read(guid)).toBeNull();
/// ```
#[test]
fn a_receipt_no_reader_accepts_is_not_written() {
    let folder: ReceiptsFolder = live_folder("receipts-hostile");
    let mut receipt: CloneMountReceipt = receipt();
    receipt.mount = Some(ReceiptMountIdentity {
        mount_id: 7,
        device: 2049,
        inode: 3,
        source: String::from("a source with a space"),
        canonical_path: String::from("/home/user/repo/.claude/worktrees/one"),
    });
    let refusal: ReceiptsRefusal = folder.publish(&receipt).expect_err("a source with a space is no receipt field");
    assert!(matches!(refusal, ReceiptsRefusal::NotAReceipt { .. }), "{refusal:?}");
    assert_eq!(folder.read(receipt.dataset).expect("nothing was filed"), None);
    assert!(!folder.directory().join(folder.staging_name(receipt.dataset)).exists(), "neither was a staging sibling");
}

/// What:
///  A failed publish behind a name the rename cannot take leaves no staging
///  sibling,
///  so this process's next publish is not blocked by it.
/// Why:
///   A handler that retries after a failed publish in the same process met
///   `EEXIST` on its own sibling and could never file again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdir(dir.join(guid)); expect(() => folder.publish(receipt)).toThrow(); expect(staging).not.toExist(); rmdir(dir.join(guid)); folder.publish(receipt);
/// ```
#[test]
fn a_failed_publish_leaves_no_staging_sibling() {
    let folder: ReceiptsFolder = live_folder("receipts-failed-publish");
    let receipt: CloneMountReceipt = receipt();
    let blocker: PathBuf = folder.directory().join(folder.entry_name(receipt.dataset));
    std::fs::create_dir(blocker.as_path()).expect("a folder stands where the receipt belongs");
    let refusal: ReceiptsRefusal = folder.publish(&receipt).expect_err("a rename onto a folder fails");
    assert!(matches!(refusal, ReceiptsRefusal::Staging { .. }), "{refusal:?}");
    assert!(!folder.directory().join(folder.staging_name(receipt.dataset)).exists(), "the writer removed its own sibling");
    std::fs::remove_dir(blocker.as_path()).expect("the folder is removed");
    folder.publish(&receipt).expect("this process's next publish is not blocked");
    assert_eq!(folder.read(receipt.dataset).expect("the receipt is read"), Some(receipt));
}

/// What:
///  A staging sibling this process left in an earlier run blocks nothing.
/// Why:
///   Every attempt has a name of its own,
///  so a leftover from this process's earlier failure is neither reused nor in
///  the way,
///  and the control keeps the plant so the property is held rather than assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFile(folder.stagingName(oldAttempt), 'v1\n'); folder.publish(receipt); expect(read(guid)).toEqual(receipt);
/// ```
#[test]
fn a_staging_sibling_of_this_process_blocks_nothing() {
    let folder: ReceiptsFolder = live_folder("receipts-own-sibling");
    let receipt: CloneMountReceipt = receipt();
    let sibling: PathBuf = folder.directory().join(folder.staging_name(receipt.dataset));
    let next: String = folder.staging_name(receipt.dataset);
    assert_ne!(next, folder.staging_name(receipt.dataset), "two attempts never share a staging name");
    std::fs::write(sibling.as_path(), "v1\n").expect("this process's earlier sibling is planted");
    folder.publish(&receipt).expect("a sibling of an earlier attempt blocks nothing");
    assert!(sibling.exists(), "the planted leftover is left where it was, and no receipt reads from it");
    assert_eq!(folder.read(receipt.dataset).expect("the receipt is read"), Some(receipt));
}

/// What:
///  Two publishes of one dataset in one process,
///  at the same time,
///  leave one whole receipt and no staging sibling,
///  so neither can remove the other's live sibling or rename a half-written
///  file into the receipt's name.
/// Why:
///   Parallel sweeps reach one process,
///  and a staging name shared by both would let the second publish unlink the
///  first one's file while it is being written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.all([folder.publish(a), folder.publish(b)]); expect(parse(read(guid))).toBeOneOf([a, b]);
/// ```
#[test]
fn two_publishes_at_once_leave_one_whole_receipt() {
    let folder: ReceiptsFolder = live_folder("receipts-two-writers");
    let mut first: CloneMountReceipt = receipt();
    let mut second: CloneMountReceipt = receipt();
    second.outcome = ReceiptOutcome::Refused;
    second.mount = None;
    second.failed_step = Some(String::from("read-intent"));
    first.time = String::from("2026-11-21T15:00:01Z");
    second.time = String::from("2026-11-21T15:00:02Z");
    let gate: std::sync::Barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        for receipt in [&first, &second] {
            let writer: ReceiptsFolder = folder.clone();
            let filing: CloneMountReceipt = receipt.clone();
            let barrier: &std::sync::Barrier = &gate;
            scope.spawn(move || {
                barrier.wait();
                writer.publish(&filing).expect("a writer files its receipt");
            });
        }
    });
    let found: Option<CloneMountReceipt> = folder.read(first.dataset).expect("the receipt is read");
    assert!(found.as_ref() == Some(&first) || found.as_ref() == Some(&second), "one whole receipt is the answer: {found:?}");
    let siblings: Vec<PathBuf> = std::fs::read_dir(folder.directory())
        .expect("the directory lists")
        .flatten()
        .map(|entry| return entry.path())
        .filter(|path| return path.to_string_lossy().contains(".staging."))
        .collect();
    assert!(siblings.is_empty(), "no staging sibling survives: {siblings:?}");
}

/// Every refusal carries the way forward and says nothing acted on the file.
#[test]
fn every_refusal_carries_the_way_forward() {
    let folder: ReceiptsFolder = ReceiptsFolder::at(PathBuf::from("/nonexistent/receipts"), 0);
    let missing_refusal: ReceiptsRefusal = folder.read(control_guid()).expect_err("a missing directory is refused");
    assert!(matches!(missing_refusal, ReceiptsRefusal::DirectoryAbsent { .. }), "{missing_refusal:?}");
    let message: String = missing_refusal.message();
    assert!(message.contains("Nothing was matched"), "{message}");
    assert!(message.contains("journalctl -u reconcile-git-worktree-clone-mounts.service"), "{message}");
}
