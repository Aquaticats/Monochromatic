//! Controls for the count of a checkout's entries: every kind of entry counts once, a link is not followed, and an
//!  unreadable or missing directory counts what could be read instead of failing.

use super::*;
use crate::test_support::{fixture, remove};

/// Files, directories, an empty directory and a link to a directory each count once, and the link's target is not walked.
#[test]
fn every_entry_counts_once_and_links_are_not_followed() {
    let parent: PathBuf = fixture("entry-count-kinds");
    let root: PathBuf = parent.join("tree");
    let nested: PathBuf = root.join("a").join("b");
    std::fs::create_dir_all(&nested).expect("nested directories");
    std::fs::create_dir(root.join("empty")).expect("empty directory");
    let files: u64 = 5;
    for number in 0..files {
        std::fs::write(nested.join(format!("file-{number}")), b"x").expect("file");
    }
    std::os::unix::fs::symlink(root.join("a"), root.join("link")).expect("link");
    // Entries: a, a/b, empty, link, and the files; the root itself is not an entry of its own contents.
    let expected: u64 = 4 + files;
    assert_eq!(count_entries(root.as_path()), expected);
    remove(parent.as_path());
}

/// A root that cannot be listed has no entries to count, and a file has none either.
#[test]
fn a_missing_root_and_a_plain_file_count_nothing() {
    let parent: PathBuf = fixture("entry-count-nothing");
    assert_eq!(count_entries(parent.join("absent").as_path()), 0);
    std::fs::write(parent.join("file"), b"x").expect("file");
    assert_eq!(count_entries(parent.join("file").as_path()), 0);
    remove(parent.as_path());
}
