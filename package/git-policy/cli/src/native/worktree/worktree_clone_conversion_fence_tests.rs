//! Controls for the fence's reads that need no Git:
//! which paths name an attributes file,
//! and which untracked attributes file above a tracked path refuses the rung.

use super::*;
use crate::subvolume_index::StoredStat;
use crate::test_support::{fixture, remove};

/// A tracked entry of one path,
/// with the stat fields a clean fixture writes.
fn entry(path: &str, size: u32) -> StatEntry {
    return StatEntry { path: path.as_bytes().to_vec(), flags: 0, mode: 0o100644, stored: StoredStat { ctime_seconds: 1, ctime_nanos: 0, mtime_seconds: 1, mtime_nanos: 0, ino: 1, uid: 0, gid: 0, size } };
}

#[test]
fn only_the_two_spellings_of_an_attributes_path_are_attributes_paths() {
    assert!(is_attributes_path(b".gitattributes"));
    assert!(is_attributes_path(b"sub/.gitattributes"));
    for path in [b"gitattributes".as_slice(), b"sub/x.gitattributes", b".gitattributes.bak", b"sub/.gitattributes/x"] {
        assert!(!is_attributes_path(path), "{}", String::from_utf8_lossy(path));
    }
}

#[test]
fn a_tracked_attributes_file_refuses_nothing() {
    let root = fixture("fence-tracked-attributes");
    std::fs::write(root.join(".gitattributes"), "* text=auto eol=lf\n").expect("attributes");
    let entries = vec![entry(".gitattributes", 20), entry("a.txt", 18)];
    assert_eq!(first_untracked_attributes(entries.as_slice(), root.as_path()), None);
    remove(root.as_path());
}

#[test]
fn an_untracked_attributes_file_above_a_tracked_path_refuses_the_rung() {
    let root = fixture("fence-untracked-attributes");
    std::fs::create_dir_all(root.join("sub")).expect("directory");
    std::fs::write(root.join("sub/.gitattributes"), "*.txt ident\n").expect("attributes");
    let entries = vec![entry("sub/a.txt", 18), entry("b.txt", 18)];
    let reason = first_untracked_attributes(entries.as_slice(), root.as_path()).expect("a refusal");
    assert!(reason.starts_with("sub/.gitattributes is in the source and not in its index"), "{reason}");
    remove(root.as_path());
}

#[test]
fn an_attributes_file_beside_paths_no_entry_holds_refuses_nothing() {
    let root = fixture("fence-unrelated-attributes");
    std::fs::create_dir_all(root.join("untracked")).expect("directory");
    std::fs::write(root.join("untracked/.gitattributes"), "*.txt ident\n").expect("attributes");
    let entries = vec![entry("a.txt", 18)];
    assert_eq!(first_untracked_attributes(entries.as_slice(), root.as_path()), None);
    remove(root.as_path());
}
