//! What:
//!  Controls for the later sweep's reading of a record.
//! Why:
//!  A sweep is a non-binding pass:
//!  an absent,
//!  void or unrecorded record unmounts nothing and reports which it found,
//!  and an empty folder on the cache's own mount is empty rather than a mount
//!  to take down.
//!  The unmount itself needs a mount and is proved in the guest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(finishAStagedUnmount(sweep).word).toBe('empty');
//! ```

use super::{finish_a_staged_unmount, StagedSweep, SweepOutcome};
use crate::staged_folder_identity::StagedFolderIdentity;
use crate::staged_folder_record::{write_record, StagedFolderRecord};
use crate::staging_calls_control::RecordingCalls;
use crate::staging_folder_name::staging_folder_name;
use crate::user_cache_pin::UserCachePin;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The GUID the fixtures use.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("808").expect("the fixture GUID parses");
}

/// A cache fixture with a staging folder and one record of this boot.
fn fixture(name: &str) -> (std::path::PathBuf, std::fs::File, UserCachePin) {
    let root: PathBuf = crate::testing::test_support::fixture_root().join(name);
    std::fs::create_dir_all(root.as_path()).expect("the fixture cache is made");
    let folder_path: PathBuf = root.join(staging_folder_name(guid()));
    std::fs::create_dir_all(folder_path.as_path()).expect("the fixture folder is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture cache opens");
    let folder: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the fixture folder opens");
    let record: StagedFolderRecord = StagedFolderRecord {
        component: staging_folder_name(guid()),
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
    let pin: UserCachePin = UserCachePin { source: String::from("pool/cache"), root_inode: cache.metadata().expect("the cache's metadata is read").ino(), device: 1 };
    return (root, cache, pin);
}

/// An absent record is reported and nothing is unmounted.
#[test]
fn an_absent_record_is_reported() {
    let root: PathBuf = crate::testing::test_support::fixture_root().join("sweep-absent");
    std::fs::create_dir_all(root.as_path()).expect("the fixture cache is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture cache opens");
    let pin: UserCachePin = UserCachePin { source: String::from("pool/cache"), root_inode: cache.metadata().expect("the cache's metadata is read").ino(), device: 1 };
    let calls = RecordingCalls::default();
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome: SweepOutcome = finish_a_staged_unmount(&sweep);
    assert_eq!(outcome.word(), "not-current");
    assert!(!calls.ran(crate::staging_calls::StagingCall::Unmount));
}

/// A record from another boot is void and its handle is not opened.
#[test]
fn another_boots_record_is_void() {
    let (root, cache, pin) = fixture("sweep-other-boot");
    let mut record: StagedFolderRecord = crate::staged_folder_record::read_record_text(&cache, guid()).expect("the record is read").map(|text| return crate::staged_folder_record::record_from_text(text.as_str(), guid()).expect("the record parses")).expect("the record is there");
    record.boot = String::from("an-older-boot");
    write_record(&cache, &record).expect("the older record is written");
    let calls = RecordingCalls::default();
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome: SweepOutcome = finish_a_staged_unmount(&sweep);
    assert_eq!(outcome.word(), "not-current");
    assert!(!calls.ran(crate::staging_calls::StagingCall::OpenHandle));
}

/// A current record is read,
/// and the pin is the next check before any unmount.
#[test]
fn a_current_record_reaches_the_pin_before_any_unmount() {
    let (root, cache, pin) = fixture("sweep-empty");
    let folder_path: PathBuf = root.join(staging_folder_name(guid()));
    let folder: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the fixture folder opens");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Ok(folder));
    calls.mount_ids.borrow_mut().extend([7, 7, 7]);
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome: SweepOutcome = finish_a_staged_unmount(&sweep);
    assert_eq!(outcome.word(), "empty", "{outcome:?}");
    assert!(!calls.ran(crate::staging_calls::StagingCall::Unmount));
}

/// A current record whose folder path a mount row names is not empty,
/// even though the handle reopens the folder on the cache mount.
#[test]
fn a_current_record_with_a_row_is_not_empty() {
    let (root, cache, pin) = fixture("sweep-row");
    let folder_path: PathBuf = root.join(staging_folder_name(guid()));
    let folder: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the fixture folder opens");
    let calls = RecordingCalls::default();
    *calls.open_handle.borrow_mut() = Some(Ok(folder));
    calls.mount_ids.borrow_mut().extend([7, 7]);
    *calls.table.borrow_mut() = vec![crate::fs_mount_options::MountFacts {
        mount_id: 51,
        parent_id: 7,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: folder_path.clone(),
        filesystem_type: String::from("zfs"),
        source: String::from("tmpfs"),
        mount_options: Vec::new(),
        super_options: Vec::new(),
    }];
    let sweep = StagedSweep { cache: &cache, cache_path: root.as_path(), guid: guid(), boot: "this-boot", pin: &pin, calls: &calls };
    let outcome: SweepOutcome = finish_a_staged_unmount(&sweep);
    assert_ne!(outcome.word(), "empty", "a row at the folder path is not an empty folder: {outcome:?}");
}
