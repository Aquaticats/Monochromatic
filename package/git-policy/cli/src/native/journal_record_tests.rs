//! Controls for the journal record writer: each durability level performs exactly the syncs it names.

use super::*;
use crate::private_storage::{take_directory_syncs, take_file_syncs};
use crate::test_support::{fixture, remove};

/// A record at the lowest level is whole on disk and made no file sync and no directory sync.
#[test]
fn a_record_with_no_sync_level_syncs_nothing() {
    let root: PathBuf = fixture("journal-record-nothing");
    take_file_syncs();
    take_directory_syncs();
    write_record(root.as_path(), "record.json", b"{}\n", RecordSync::Nothing).expect("written");
    assert_eq!(std::fs::read(root.join("record.json")).expect("record"), b"{}\n");
    assert_eq!(take_file_syncs(), Vec::<PathBuf>::new());
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    remove(root.as_path());
}

/// A record at the top level syncs its file and then its directory, and each exactly once.
#[test]
fn a_record_with_the_full_level_syncs_its_file_and_its_directory() {
    let root: PathBuf = fixture("journal-record-full");
    let canonical: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    take_file_syncs();
    take_directory_syncs();
    write_record(root.as_path(), "record.json", b"{}\n", RecordSync::FileAndDirectory).expect("written");
    assert_eq!(take_file_syncs(), vec![root.join("record.json")]);
    assert_eq!(take_directory_syncs(), vec![canonical]);
    remove(root.as_path());
}

/// Neither level rewrites a record, and the failure names the path.
#[test]
fn a_record_is_written_once_at_every_level() {
    let root: PathBuf = fixture("journal-record-once");
    for sync in [RecordSync::Nothing, RecordSync::FileAndDirectory] {
        let name: String = format!("{sync:?}.json");
        write_record(root.as_path(), name.as_str(), b"first", sync).expect("first write");
        let failure: String = write_record(root.as_path(), name.as_str(), b"second", sync).expect_err("second write");
        assert!(failure.contains(root.join(name.as_str()).to_string_lossy().as_ref()), "{failure}");
        assert_eq!(std::fs::read(root.join(name.as_str())).expect("unchanged"), b"first");
    }
    remove(root.as_path());
}

/// No staging file stays beside a record at either level.
#[test]
fn no_staging_file_stays_beside_a_record() {
    let root: PathBuf = fixture("journal-record-staging");
    write_record(root.as_path(), "a.json", b"a", RecordSync::Nothing).expect("a");
    write_record(root.as_path(), "b.json", b"b", RecordSync::FileAndDirectory).expect("b");
    let mut names: Vec<String> = std::fs::read_dir(root.as_path())
        .expect("directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .collect();
    names.sort();
    assert_eq!(names, vec![String::from("a.json"), String::from("b.json")]);
    remove(root.as_path());
}
