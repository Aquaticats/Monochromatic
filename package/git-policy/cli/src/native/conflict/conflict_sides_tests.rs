//! What:
//!  The decision on a command line that names conflicted files,
//! the saved copies,
//! and their notices.
//! Why:
//!  A path wrongly judged conflicted lets a clean file be overwritten;
//! a copy that is missing,
//! partial or in the wrong
//!  mode loses the half-resolved file the owner's decision exists to keep;
//! a copy failure that is swallowed discards
//!  unsaved work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(decideSides(['a.rs'], '', root, new Set(['a.rs']))).toEqual({ kind: 'allow', paths: ['a.rs'] });
//! ```

use super::{
    Operation, SaveFailure, SavedCopy, SideDecision, clean_paths_note, conflicted_paths, decide_sides, literal_path, operation_in_progress,
    save_copies, save_copies_watching, saved_notice, stamp, still_being_written_note,
};
use crate::conflict_side_decision::{magic_name, magic_on_conflicted};
use crate::repo_read_index::IndexEntry;
use crate::test_support::{fixture, remove};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// An index entry at `stage` for `path`.
fn entry(path: &str, stage: u8) -> IndexEntry {
    return IndexEntry { path: path.as_bytes().to_vec(), mode: 0o100_644, id: vec![0; 20], stage, extended: 0 };
}

/// A set of conflicted paths from text.
fn set(paths: &[&str]) -> BTreeSet<Vec<u8>> {
    return paths.iter().map(|each| { return each.as_bytes().to_vec(); }).collect();
}

/// Tokens from text.
fn tokens(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// A path is conflicted when any of its entries is at stage 1,
/// 2 or 3,
/// however many entries it has.
#[test]
fn conflicted_paths_are_the_unmerged_entries() {
    let entries = vec![entry("a.rs", 0), entry("b.rs", 1), entry("b.rs", 2), entry("b.rs", 3), entry("c.rs", 2), entry("d.rs", 0)];
    assert_eq!(conflicted_paths(entries.as_slice()), set(&["b.rs", "c.rs"]));
    assert_eq!(conflicted_paths(&[]), set(&[]));
}

/// Each state file names its operation;
/// `am` shares the rebase directory and is not one of them.
#[test]
fn the_operation_in_progress_is_read_from_the_state_files() {
    let root: PathBuf = fixture("sides-operation");
    assert_eq!(operation_in_progress(root.as_path()), None);
    for (file, expected) in [("MERGE_HEAD", Operation::Merge), ("CHERRY_PICK_HEAD", Operation::CherryPick), ("REVERT_HEAD", Operation::Revert)] {
        std::fs::write(root.join(file), b"0\n").expect("state file");
        assert_eq!(operation_in_progress(root.as_path()), Some(expected), "{file}");
        std::fs::remove_file(root.join(file)).expect("remove");
    }
    std::fs::create_dir(root.join("rebase-merge")).expect("merge backend");
    assert_eq!(operation_in_progress(root.as_path()), Some(Operation::Rebase));
    std::fs::remove_dir(root.join("rebase-merge")).expect("remove");
    std::fs::create_dir(root.join("rebase-apply")).expect("apply backend");
    assert_eq!(operation_in_progress(root.as_path()), None, "an am keeps rebase-apply without rebasing");
    std::fs::write(root.join("rebase-apply/applying"), b"").expect("am");
    assert_eq!(operation_in_progress(root.as_path()), None);
    std::fs::write(root.join("rebase-apply/rebasing"), b"").expect("rebase");
    assert_eq!(operation_in_progress(root.as_path()), Some(Operation::Rebase));
    remove(root.as_path());
}

/// A literal spelling resolves below the working directory;
/// magic,
/// wildcards,
/// directories and paths outside do not resolve.
#[test]
fn only_a_literal_file_spelling_resolves() {
    let root: &Path = Path::new("/work/tree");
    for (prefix, token, expected) in [
        ("", "a.rs", Some("a.rs")),
        ("sub/", "a.rs", Some("sub/a.rs")),
        ("sub/", "../a.rs", Some("a.rs")),
        ("", "./dir/./a.rs", Some("dir/a.rs")),
        ("", "dir//a.rs", Some("dir/a.rs")),
        ("", "/work/tree/dir/a.rs", Some("dir/a.rs")),
        ("", "/elsewhere/a.rs", None),
        ("", "../a.rs", None),
        ("", ".", None),
        ("", "", None),
        ("", "dir/", None),
        ("", "*.rs", None),
        ("", "a?.rs", None),
        ("", "[ab].rs", None),
        ("", ":(glob)a.rs", None),
        ("", ":/a.rs", None),
        ("", "a\\b", None),
    ] {
        assert_eq!(
            literal_path(prefix.as_bytes(), root, OsString::from(token).as_os_str()),
            expected.map(|each| { return each.as_bytes().to_vec(); }),
            "{prefix:?} {token:?}"
        );
    }
}

/// A line that names only conflicted files is allowed with each path once;
/// one clean path refuses the whole line and names it.
#[test]
fn a_line_is_allowed_only_when_every_path_is_conflicted() {
    let root: &Path = Path::new("/work/tree");
    let conflicted: BTreeSet<Vec<u8>> = set(&["a.rs", "sub/b.rs"]);
    assert_eq!(
        decide_sides(tokens(&["a.rs", "sub/b.rs", "./a.rs"]).as_slice(), b"", root, &conflicted),
        SideDecision::Allow(vec![b"a.rs".to_vec(), b"sub/b.rs".to_vec()])
    );
    assert_eq!(decide_sides(tokens(&["b.rs"]).as_slice(), b"sub/", root, &conflicted), SideDecision::Allow(vec![b"sub/b.rs".to_vec()]));
    assert_eq!(
        decide_sides(tokens(&["a.rs", "clean.rs", "sub", "*.rs"]).as_slice(), b"", root, &conflicted),
        SideDecision::Refuse(tokens(&["clean.rs", "sub", "*.rs"]))
    );
    assert_eq!(decide_sides(tokens(&["clean.rs"]).as_slice(), b"", root, &conflicted), SideDecision::Refuse(tokens(&["clean.rs"])));
    assert_eq!(decide_sides(tokens(&["a.rs"]).as_slice(), b"", root, &set(&[])), SideDecision::Refuse(tokens(&["a.rs"])));
}

/// A fixture with a worktree and a common Git directory,
/// and a file of each kind in it.
fn tree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let base: PathBuf = fixture(name);
    let root: PathBuf = base.join("tree");
    let common: PathBuf = base.join("common");
    std::fs::create_dir_all(root.join("sub")).expect("worktree");
    std::fs::create_dir_all(&common).expect("common directory");
    return (base, root, common);
}

/// The copy holds the file's bytes,
/// permission bits and symbolic-link target,
/// below a directory named by the time,
/// and the
///  notice names each place.
#[test]
fn a_copy_keeps_content_mode_and_link_target() {
    let (base, root, common) = tree("sides-copy");
    std::fs::write(root.join("a.rs"), b"half \xff resolved\n").expect("a");
    std::fs::set_permissions(root.join("a.rs"), std::fs::Permissions::from_mode(0o754)).expect("mode");
    std::fs::write(root.join("sub/b.rs"), b"").expect("b");
    std::os::unix::fs::symlink("target/elsewhere", root.join("link")).expect("link");
    let paths: Vec<Vec<u8>> = vec![b"a.rs".to_vec(), b"sub/b.rs".to_vec(), b"link".to_vec(), b"gone.rs".to_vec()];
    let milliseconds: i64 = 1_793_628_067_000;
    let copies: Vec<SavedCopy> = save_copies(root.as_path(), common.as_path(), paths.as_slice(), milliseconds).expect("saved").copies;
    let directory: PathBuf = common.join("monochromatic-git-policy-cli/discarded").join(stamp(milliseconds));
    assert_eq!(stamp(milliseconds), "2026-11-02T14-01-07");
    assert_eq!(copies[0].saved_to, Some(directory.join("a.rs")));
    assert_eq!(std::fs::read(directory.join("a.rs")).expect("copy"), b"half \xff resolved\n");
    assert_eq!(std::fs::metadata(directory.join("a.rs")).expect("mode").permissions().mode() & 0o777, 0o754);
    assert_eq!(std::fs::read(directory.join("sub/b.rs")).expect("empty copy"), b"");
    assert_eq!(std::fs::read_link(directory.join("link")).expect("link copy"), PathBuf::from("target/elsewhere"));
    assert_eq!(copies[3], SavedCopy { path: String::from("gone.rs"), saved_to: None });
    let notice: String = saved_notice(copies.as_slice());
    assert!(notice.contains(format!("saved your copy of a.rs to\n  {}", directory.join("a.rs").display()).as_str()), "{notice}");
    assert!(notice.contains("gone.rs was conflicted and is deleted in your worktree, so there is no copy to save"), "{notice}");
    remove(base.as_path());
}

/// A file another program keeps writing does not hold the command:
///  the copy is what the file held when it was opened, it is kept, and the caller is told the source moved on.
#[test]
fn a_source_that_keeps_growing_is_copied_once_and_named() {
    let (base, root, common) = tree("sides-growing");
    let content: Vec<u8> = (0..300_000_u32).map(|each| { return u8::try_from(each % 251).expect("fits"); }).collect();
    std::fs::write(root.join("a.rs"), content.as_slice()).expect("a");
    let mut appended: usize = 0;
    // A writer that appends after every chunk the copy writes, bounded so this control cannot run away.
    let mut writer = || {
        if appended >= 20 {
            return;
        }
        appended += 1;
        let mut file = std::fs::OpenOptions::new().append(true).open(root.join("a.rs")).expect("open");
        std::io::Write::write_all(&mut file, b"x").expect("append");
    };
    let saved = save_copies_watching(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], 1_793_628_067_000, &mut writer)
        .expect("a writer that keeps adding does not refuse the command");
    assert_eq!(saved.still_being_written, vec![String::from("a.rs")], "the source that moved on is named: {saved:?}");
    let place: &Path = saved.copies[0].saved_to.as_deref().expect("a copy");
    let kept: Vec<u8> = std::fs::read(place).expect("the copy is kept");
    assert_eq!(kept.len(), content.len(), "the copy holds the bytes the file had when it was opened");
    assert!(appended < 20, "the copy read the size it opened and did not chase the writer: {appended} appends");
    let notice: String = format!("{}{}", saved_notice(saved.copies.as_slice()), still_being_written_note(saved.still_being_written.as_slice()));
    assert!(notice.contains("was still being written while the wrapper copied it"), "{notice}");
    assert!(notice.contains("a write that lands after the copy is not saved"), "{notice}");
    remove(base.as_path());
}

/// A second invocation in the same second gets its own directory and never overwrites the first copy.
#[test]
fn a_second_save_in_the_same_second_keeps_the_first() {
    let (base, root, common) = tree("sides-twice");
    std::fs::write(root.join("a.rs"), b"first\n").expect("a");
    let milliseconds: i64 = 1_793_628_067_000;
    let first = save_copies(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], milliseconds).expect("first").copies;
    std::fs::write(root.join("a.rs"), b"second\n").expect("a");
    let second = save_copies(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], milliseconds).expect("second").copies;
    assert_ne!(first[0].saved_to, second[0].saved_to);
    assert_eq!(std::fs::read(first[0].saved_to.as_ref().expect("first place")).expect("first copy"), b"first\n");
    assert_eq!(std::fs::read(second[0].saved_to.as_ref().expect("second place")).expect("second copy"), b"second\n");
    remove(base.as_path());
}

/// A place that cannot hold the copies,
/// or a file that cannot be copied,
/// is a failure naming the subject;
/// nothing is claimed saved.
#[test]
fn a_copy_that_cannot_be_written_is_a_failure() {
    let (base, root, common) = tree("sides-failure");
    std::fs::write(root.join("a.rs"), b"mine\n").expect("a");
    // `monochromatic-git-policy-cli` is a file, so no directory can be made below it.
    std::fs::write(common.join("monochromatic-git-policy-cli"), b"in the way").expect("blocker");
    let failure: SaveFailure = save_copies(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], 0).expect_err("no place");
    assert!(failure.subject.contains("discarded"), "{failure:?}");
    assert!(!failure.reason.is_empty(), "{failure:?}");
    std::fs::remove_file(common.join("monochromatic-git-policy-cli")).expect("remove blocker");
    // A directory where the conflicted file is expected cannot be copied as a file.
    std::fs::create_dir(root.join("dir.rs")).expect("directory");
    let directory_failure: SaveFailure = save_copies(root.as_path(), common.as_path(), &[b"dir.rs".to_vec()], 0).expect_err("a directory");
    assert_eq!(directory_failure.subject, "dir.rs");
    assert!(directory_failure.reason.contains("neither a file nor a symbolic link"), "{directory_failure:?}");
    remove(base.as_path());
}

/// The sentence added to a rejection names every path that is not conflicted and the operation.
#[test]
fn the_rejection_names_the_paths_that_are_not_conflicted() {
    let note: String = clean_paths_note(tokens(&["b.rs", "dir"]).as_slice(), Operation::CherryPick);
    assert!(note.contains("This cherry-pick is in progress"), "{note}");
    assert!(note.contains("`b.rs`, `dir`"), "{note}");
}

/// Magic that leaves the named file the same resolves to that file and its spelling;
/// any other magic,
/// a wildcard under it,
/// or
///  a clean file under it does not become a note.
#[test]
fn magic_on_a_plain_name_resolves_and_other_magic_does_not() {
    let root: &Path = Path::new("/work/tree");
    for (prefix, token, expected) in [
        ("", ":(icase)a.rs", Some((":(icase)", "a.rs"))),
        ("sub/", ":(glob)b.rs", Some((":(glob)", "sub/b.rs"))),
        ("sub/", ":(top)a.rs", Some((":(top)", "a.rs"))),
        ("sub/", ":/a.rs", Some((":/", "a.rs"))),
        ("", ":!a.rs", Some((":!", "a.rs"))),
        ("", ":^a.rs", Some((":^", "a.rs"))),
        ("", ":!:a.rs", Some((":!:", "a.rs"))),
        ("", ":(icase,literal)a.rs", Some((":(icase,literal)", "a.rs"))),
        ("", ":(exclude)a.rs", Some((":(exclude)", "a.rs"))),
        ("", ":(attr:text)a.rs", None),
        ("", ":(prefix:1)a.rs", None),
        ("", ":(icase)*.rs", None),
        ("", ":a.rs", None),
        ("", "a.rs", None),
        ("", ":(icase)", None),
        ("", ":(icase", None),
    ] {
        assert_eq!(
            magic_name(prefix.as_bytes(), root, OsString::from(token).as_os_str()),
            expected.map(|(magic, path)| { return (String::from(magic), path.as_bytes().to_vec()); }),
            "{prefix:?} {token:?}"
        );
    }
    let conflicted: BTreeSet<Vec<u8>> = set(&["a.rs"]);
    assert_eq!(
        magic_on_conflicted(tokens(&[":(icase)a.rs", ":!a.rs"]).as_slice(), b"", root, &conflicted),
        Some(vec![(String::from(":(icase)"), String::from("a.rs")), (String::from(":!"), String::from("a.rs"))])
    );
    assert_eq!(magic_on_conflicted(tokens(&[":(icase)a.rs", ":(icase)c.rs"]).as_slice(), b"", root, &conflicted), None);
    assert_eq!(magic_on_conflicted(tokens(&[":(icase)a.rs", "c.rs"]).as_slice(), b"", root, &conflicted), None);
    assert_eq!(magic_on_conflicted(&[], b"", root, &conflicted), None);
}
