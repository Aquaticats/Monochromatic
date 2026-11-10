//! Controls for the reflog message a landing writes.
//! Why:
//!  The message is compared with what native Git 2.56.0 writes for the same commits
//! (`builtin/commit.c` `reflog_msg` and `update_head_with_reflog` in `sequencer.c`),
//! and the first line is the only part of the message it carries,
//!  so a body with a newline or an unusual byte cannot change the row.

use super::{Committed, first_line, kind, message};
use crate::test_support::fixture;
use std::path::{Path, PathBuf};

/// A directory standing for a batch's copy of the conclusion state,
/// holding the state files named.
fn copy_holding(name: &str, files: &[&str]) -> PathBuf {
    let root: PathBuf = fixture(format!("reflog-{name}").as_str());
    let shadow: PathBuf = root.join("shadow");
    std::fs::create_dir_all(shadow.as_path()).expect("the copy directory");
    for file in files {
        std::fs::write(shadow.join(file), b"state\n").expect("a state file");
    }
    return root;
}

fn text(value: std::ffi::OsString) -> String {
    return value.to_string_lossy().into_owned();
}

/// Each kind writes the word native Git writes for it.
#[test]
fn each_kind_carries_native_gits_word() {
    assert_eq!(text(message(Committed::Plain, b"plain\n\nbody\n")), "commit: plain");
    assert_eq!(text(message(Committed::Initial, b"first\n")), "commit (initial): first");
    assert_eq!(text(message(Committed::Amend, b"amended\n")), "commit (amend): amended");
    assert_eq!(text(message(Committed::Merge, b"Merge branch 'side'\n")), "commit (merge): Merge branch 'side'");
    assert_eq!(text(message(Committed::CherryPick, b"picked\n")), "commit (cherry-pick): picked");
    assert_eq!(text(message(Committed::Revert, b"reverted\n")), "commit (revert): reverted");
}

/// The whole first line travels,
/// and the body never does:
/// a second line,
/// a carriage return,
/// a byte that is not UTF-8,
/// and a NUL all stay out of the row,
/// as native Git's copy of the first line ends at the first newline.
#[test]
fn the_first_line_travels_whole_and_the_body_never_does() {
    assert_eq!(text(message(Committed::Plain, b"one two three\nsecond line\n")), "commit: one two three");
    assert_eq!(text(message(Committed::Plain, b"no newline at all")), "commit: no newline at all");
    assert_eq!(text(message(Committed::Plain, b"\n")), "commit: ");
    let unusual: &[u8] = b"caf\xc3\xa9 and \xff\nbody\n";
    assert_eq!(message(Committed::Plain, unusual).as_encoded_bytes(), b"commit: caf\xc3\xa9 and \xff");
    assert_eq!(text(message(Committed::Plain, b"before the nul\0after\n")), "commit: before the nul");
    assert_eq!(first_line(b"line\nrest"), b"line");
}

/// The kind is the one native Git would use:
/// a branch with no head is initial,
/// an amend is amend,
/// and a conclusion is the state file the batch removes.
#[test]
fn the_kind_follows_the_state_native_git_reads() {
    let merge: PathBuf = copy_holding("merge", &["MERGE_HEAD", "MERGE_MSG"]);
    let cherry: PathBuf = copy_holding("cherry", &["CHERRY_PICK_HEAD"]);
    let revert: PathBuf = copy_holding("revert", &["REVERT_HEAD"]);
    let plain: PathBuf = copy_holding("plain", &[]);
    assert_eq!(kind(false, true, 0, None), Committed::Initial);
    assert_eq!(kind(true, false, 1, None), Committed::Amend);
    assert_eq!(kind(false, false, 2, None), Committed::Merge);
    assert_eq!(kind(false, false, 1, Some(Path::new(merge.as_path()))), Committed::Merge);
    assert_eq!(kind(false, false, 1, Some(Path::new(cherry.as_path()))), Committed::CherryPick);
    assert_eq!(kind(false, false, 1, Some(Path::new(revert.as_path()))), Committed::Revert);
    assert_eq!(kind(false, false, 1, Some(Path::new(plain.as_path()))), Committed::Plain);
    assert_eq!(kind(false, false, 1, None), Committed::Plain);
}
