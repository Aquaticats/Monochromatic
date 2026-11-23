//! What:
//!  Controls for the record's read order and its three answers.
//! Why:
//!  A record from another boot must not have its handle opened,
//!  an `ESTALE` alone must not void,
//!  and a current record whose folder moved must refuse;
//!  each arm scripts one answer and asserts the order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(reading.kind).toBe('void'); expect(calls.ran('open-handle')).toBe(false);
//! ```

use crate::staged_folder_identity::StagedFolderIdentity;
use crate::staged_folder_record::{write_record, StagedFolderRecord};
use crate::staged_record_reading::{read_staged_record, BindingDecision, RecordReading};
use crate::staging_calls::{StagingCall, StagingCallRefusal};
use crate::staging_calls_control::RecordingCalls;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::PathBuf;

/// The GUID every arm uses.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("321").expect("the fixture GUID parses");
}

/// A fixture cache folder with one record written beside the folder.
fn fixture(name: &str) -> (std::path::PathBuf, std::fs::File, std::fs::File, StagedFolderRecord) {
    let root: PathBuf = crate::testing::test_support::fixture_root().join(name);
    std::fs::create_dir_all(root.as_path()).expect("the fixture cache is made");
    let folder_path: PathBuf = root.join(crate::staging_folder_name::staging_folder_name(guid()));
    std::fs::create_dir_all(folder_path.as_path()).expect("the fixture staging folder is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture cache opens");
    let folder: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the fixture staging folder opens");
    let record: StagedFolderRecord = StagedFolderRecord {
        component: crate::staging_folder_name::staging_folder_name(guid()),
        dataset: String::from("pool/clones/one"),
        guid: guid(),
        identity: StagedFolderIdentity::of(&folder).expect("the fixture identity is read"),
        handle_type: 1,
        handle: vec![1, 2, 3],
        nonce: String::from("0123456789abcdef0123456789abcdef"),
        namespace_device: 7,
        namespace_inode: 9,
        boot: String::from("this-boot"),
    };
    write_record(&cache, &record).expect("the record is written");
    return (folder_path, cache, folder, record);
}

/// One mount row at the folder's path.
fn row(path: &std::path::Path) -> MountFacts {
    return MountFacts {
        mount_id: 51,
        parent_id: 41,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: path.to_path_buf(),
        filesystem_type: String::from("zfs"),
        source: String::from("pool/clones/one"),
        mount_options: Vec::new(),
        super_options: Vec::new(),
    };
}

/// An absent record is absent.
#[test]
fn an_absent_record_is_absent() {
    let root: PathBuf = crate::testing::test_support::fixture_root().join("reading-absent");
    std::fs::create_dir_all(root.as_path()).expect("the fixture cache is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture cache opens");
    let calls = RecordingCalls::default();
    let reading: RecordReading = read_staged_record(&cache, guid(), "this-boot", root.as_path(), &calls);
    assert_eq!(reading.word(), "absent");
}

/// A record from another boot is void before its handle is opened.
#[test]
fn another_boot_is_void_before_the_handle() {
    let (folder_path, cache, _folder, mut record) = fixture("reading-other-boot");
    record.boot = String::from("an-older-boot");
    write_record(&cache, &record).expect("the other-boot record is written");
    let calls = RecordingCalls::default();
    // A row at the folder path would make a stale handle refuse rather than void; the boot test must run first and keep
    // the answer void without consulting the handle or the row.
    *calls.table.borrow_mut() = vec![row(folder_path.as_path())];
    let reading: RecordReading = read_staged_record(&cache, guid(), "this-boot", folder_path.as_path(), &calls);
    assert_eq!(reading.word(), "void");
    assert!(!calls.ran(StagingCall::OpenHandle));
}

/// A current record whose handle resolves and whose identity matches is current.
#[test]
fn a_current_record_is_current() {
    let (folder_path, cache, folder, _record) = fixture("reading-current");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Ok(folder));
    let reading: RecordReading = read_staged_record(&cache, guid(), "this-boot", folder_path.as_path(), &calls);
    assert_eq!(reading.word(), "current");
    assert_eq!(BindingDecision::of(&reading), BindingDecision::Rewrite);
}

/// `ESTALE` with no row at the folder path voids.
#[test]
fn a_stale_handle_with_no_row_is_void() {
    let (folder_path, cache, _folder, _record) = fixture("reading-stale-gone");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Err(StagingCallRefusal { call: StagingCall::OpenHandle, errno: libc::ESTALE, detail: String::from("stale") }));
    let reading: RecordReading = read_staged_record(&cache, guid(), "this-boot", folder_path.as_path(), &calls);
    assert_eq!(reading.word(), "void");
    assert_eq!(BindingDecision::of(&reading), BindingDecision::WriteFresh);
}

/// `ESTALE` with a row at the folder path refuses rather than voiding.
#[test]
fn a_stale_handle_with_a_row_is_refused() {
    let (folder_path, cache, _folder, _record) = fixture("reading-stale-live");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Err(StagingCallRefusal { call: StagingCall::OpenHandle, errno: libc::ESTALE, detail: String::from("stale") }));
    *calls.table.borrow_mut() = vec![row(folder_path.as_path())];
    let reading: RecordReading = read_staged_record(&cache, guid(), "this-boot", folder_path.as_path(), &calls);
    assert_eq!(reading.word(), "refused");
    assert_eq!(BindingDecision::of(&reading), BindingDecision::Refuse);
}

/// A handle that fails otherwise than `ESTALE` refuses.
#[test]
fn another_handle_failure_refuses() {
    let (folder_path, cache, _folder, _record) = fixture("reading-handle-perm");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Err(StagingCallRefusal { call: StagingCall::OpenHandle, errno: libc::EPERM, detail: String::from("no capability") }));
    let reading: RecordReading = read_staged_record(&cache, guid(), "this-boot", folder_path.as_path(), &calls);
    assert_eq!(reading.word(), "refused");
}

/// A folder whose identity moved refuses.
#[test]
fn a_folder_whose_identity_moved_refuses() {
    let (folder_path, cache, folder, mut record) = fixture("reading-identity-moved");
    record.identity.inode += 1;
    write_record(&cache, &record).expect("the moved-identity record is written");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Ok(folder));
    let reading: RecordReading = read_staged_record(&cache, guid(), "this-boot", folder_path.as_path(), &calls);
    assert_eq!(reading.word(), "refused");
    assert_eq!(BindingDecision::of(&reading), BindingDecision::Refuse);
}

/// A malformed record is unrecorded and replaceable.
#[test]
fn a_malformed_record_is_unrecorded() {
    let root: PathBuf = crate::testing::test_support::fixture_root().join("reading-malformed");
    std::fs::create_dir_all(root.as_path()).expect("the fixture cache is made");
    let name: PathBuf = root.join(crate::staging_folder_name::staging_record_name(guid()));
    std::fs::write(name.as_path(), b"not one object").expect("the malformed record is written");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture cache opens");
    let calls = RecordingCalls::default();
    let reading: RecordReading = read_staged_record(&cache, guid(), "this-boot", root.as_path(), &calls);
    assert_eq!(reading.word(), "unrecorded");
    assert_eq!(BindingDecision::of(&reading), BindingDecision::WriteFresh);
}
