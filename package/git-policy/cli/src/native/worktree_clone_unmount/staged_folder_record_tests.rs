//! What:
//!  Controls for the staged-folder record's one name pair,
//!  its whole-object write and its bounded read.
//! Why:
//!  A reader that accepted half a record,
//!  a record from another GUID or another shape would act on a folder nothing
//!  described,
//!  so each refusal is proved by its own fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeStagedFolderRecord(cache, record); const text = readRecord(cache, guid);
//! ```

use crate::staged_folder_identity::StagedFolderIdentity;
use crate::staged_folder_record::{read_record_text, record_from_text, write_record, StagedFolderRecord, StagedFolderRefusal};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::os::unix::fs::MetadataExt;

/// The GUID the fixtures use.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("1234567890123456789").expect("the fixture GUID parses");
}

/// One whole record.
fn record() -> StagedFolderRecord {
    return StagedFolderRecord {
        component: crate::staging_folder_name::staging_folder_name(guid()),
        dataset: String::from("pool/clones/one"),
        guid: guid(),
        identity: StagedFolderIdentity { device: 42, inode: 7, birth: Some((1_700_000_000, 12)), owner: 1000, mode: 0o700 },
        handle_type: 1,
        handle: vec![0xab, 0xcd, 0x00, 0xff],
        nonce: String::from("0123456789abcdef0123456789abcdef"),
        namespace_device: 3,
        namespace_inode: 9,
        boot: String::from("11111111-2222-3333-4444-555555555555"),
    };
}

/// The whole record survives its own encoding.
#[test]
fn one_record_survives_its_encoding() {
    let written: StagedFolderRecord = record();
    let text: String = written.encode();
    assert!(text.ends_with('\n'));
    assert_eq!(record_from_text(text.as_str(), guid()), Ok(written));
}

/// A record whose handle is absent is no record this route acts on.
#[test]
fn a_record_without_a_handle_is_refused() {
    let text: String = record().encode().replace("\"handle\":\"abcd00ff\"", "\"handle\":\"\"");
    assert!(matches!(record_from_text(text.as_str(), guid()), Err(StagedFolderRefusal::Handle { .. })));
}

/// A record renamed beside another clone describes nothing.
#[test]
fn a_record_of_another_guid_is_refused() {
    let other: DatasetGuid = DatasetGuid::parse("987").expect("the other GUID parses");
    assert!(matches!(record_from_text(record().encode().as_str(), other), Err(StagedFolderRefusal::WrongGuid { .. })));
}

/// A record of another shape is refused rather than read as a current one.
#[test]
fn a_foreign_shape_is_refused() {
    assert!(matches!(record_from_text("[]", guid()), Err(StagedFolderRefusal::Malformed { .. })));
    let version: String = record().encode().replace("\"version\":\"v1\"", "\"version\":\"v2\"");
    assert!(matches!(record_from_text(version.as_str(), guid()), Err(StagedFolderRefusal::Malformed { .. })));
    let mode: String = record().encode().replace("\"mode\":\"0700\"", "\"mode\":\"700z\"");
    assert!(matches!(record_from_text(mode.as_str(), guid()), Err(StagedFolderRefusal::Field { field: "mode", .. })));
}

/// The write leaves one whole record and no temporary name.
#[test]
fn the_write_leaves_one_whole_record() {
    let root: std::path::PathBuf = crate::testing::test_support::fixture_root().join("staging-record-write");
    std::fs::create_dir_all(root.as_path()).expect("the fixture folder is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture folder opens");
    write_record(&cache, &record()).expect("the record is written");
    let text: String = read_record_text(&cache, guid()).expect("the record is read").expect("the record is there");
    assert_eq!(record_from_text(text.as_str(), guid()), Ok(record()));
    let temporary: std::path::PathBuf = root.join(crate::staging_folder_name::staging_temporary_name(guid()));
    assert!(!temporary.exists(), "the temporary name is gone after the rename");
    let written: std::path::PathBuf = root.join(crate::staging_folder_name::staging_record_name(guid()));
    assert_eq!(written.metadata().expect("the record exists").mode() & 0o777, 0o644);
}

/// A record past its bound is refused rather than read whole.
#[test]
fn an_oversized_record_is_refused() {
    let root: std::path::PathBuf = crate::testing::test_support::fixture_root().join("staging-record-oversized");
    std::fs::create_dir_all(root.as_path()).expect("the fixture folder is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture folder opens");
    let name: std::path::PathBuf = root.join(crate::staging_folder_name::staging_record_name(guid()));
    std::fs::write(name.as_path(), vec![b'x'; usize::try_from(crate::staged_folder_record::MAXIMUM_RECORD_BYTES).expect("the bound fits") + 1]).expect("the oversized record is written");
    assert!(matches!(read_record_text(&cache, guid()), Err(StagedFolderRefusal::TooLarge { .. })));
}

/// An absent record is no record,
/// not a refusal.
#[test]
fn an_absent_record_is_nothing() {
    let root: std::path::PathBuf = crate::testing::test_support::fixture_root().join("staging-record-absent");
    std::fs::create_dir_all(root.as_path()).expect("the fixture folder is made");
    let cache: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture folder opens");
    assert_eq!(read_record_text(&cache, guid()), Ok(None));
}
