//! What:
//!  Controls for the record that says the wrapper made one linked worktree.
//! Why:
//!   Every automatic removal asks this record first,
//!  so a record that a later path change could satisfy would remove a person's worktree,
//!  and a record that a replaced registration could satisfy would remove another one's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(addedByTheWrapper(admin, worktree)).toBe(true);
//! ```

use crate::test_support::{fixture, git, git_text, repository};
use crate::worktree_added_by_the_wrapper::{ADDED_RECORD_NAME, added_by_the_wrapper, record};
use std::path::PathBuf;

/// A repository and one linked worktree of it.
struct Added {
    /// The repository's common Git directory.
    common: PathBuf,
    /// The worktree.
    worktree: PathBuf,
}

/// What:
///  A repository with one linked worktree.
/// Why:
///   The record lives beside that worktree's registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = aRepositoryWithOneWorktree(name);
/// ```
fn a_repository_with_one_worktree(name: &str) -> Added {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let worktree: PathBuf = root.join("worktree");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    return Added { common: main.join(".git"), worktree };
}

/// The one registration below a common directory.
fn only_registration(common: &std::path::Path) -> PathBuf {
    let found: Vec<PathBuf> = std::fs::read_dir(common.join("worktrees"))
        .expect("the registrations")
        .flatten()
        .map(|entry| { return entry.path() })
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    return found.into_iter().next().expect("one registration");
}

#[test]
fn the_record_names_the_worktree_the_wrapper_made() {
    let added: Added = a_repository_with_one_worktree("added-record");
    record(added.common.as_path(), added.worktree.as_path()).expect("the record is written");
    let admin: PathBuf = only_registration(added.common.as_path());
    assert!(added_by_the_wrapper(admin.as_path(), added.worktree.as_path()), "the record answers for this worktree");
    assert!(admin.join(ADDED_RECORD_NAME).exists(), "the record sits beside the registration");
}

#[test]
fn a_record_of_another_worktree_answers_no() {
    let added: Added = a_repository_with_one_worktree("added-other");
    record(added.common.as_path(), added.worktree.as_path()).expect("the record is written");
    let admin: PathBuf = only_registration(added.common.as_path());
    assert!(!added_by_the_wrapper(admin.as_path(), added.worktree.join("somewhere else").as_path()), "another path is not this worktree");
}

#[test]
fn an_absent_record_answers_no() {
    let added: Added = a_repository_with_one_worktree("added-absent");
    let admin: PathBuf = only_registration(added.common.as_path());
    assert!(!added_by_the_wrapper(admin.as_path(), added.worktree.as_path()), "no record, no proof");
}

#[test]
fn a_directory_that_is_no_worktree_gets_no_record() {
    let root: PathBuf = fixture("added-plain");
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(plain.as_path()).expect("a plain directory");
    let failure: String = record(root.as_path(), plain.as_path()).expect_err("a plain directory is no linked worktree");
    assert!(failure.contains("cannot be read as a file"), "{failure}");
    assert!(!plain.join(ADDED_RECORD_NAME).exists(), "nothing is written");
}

/// The record must never be taken for a worktree Git does not register there.
#[test]
fn the_registration_is_read_from_the_worktree_pointer() {
    let added: Added = a_repository_with_one_worktree("added-pointer");
    record(added.common.as_path(), added.worktree.as_path()).expect("the record is written");
    let pointer: String = git_text(added.worktree.as_path(), &["rev-parse", "--absolute-git-dir"]);
    let admin: PathBuf = std::fs::canonicalize(pointer).expect("the administrative directory");
    assert!(admin.join(ADDED_RECORD_NAME).exists(), "the record sits in the directory the worktree names: {}", admin.display());
}

/// What:
///  A worktree whose own path ends in a space answers the record the wrapper wrote for it.
/// Why:
///   Every ASCII whitespace byte was taken from the record,
///  so the space that belongs to the path went with the terminator
/// and the only proof that lets a later command remove this worktree by itself was never true.
#[test]
fn a_worktree_whose_path_ends_in_a_space_answers_its_record() {
    let root: PathBuf = fixture("added-trailing-space");
    let main: PathBuf = repository(root.as_path(), "main");
    let worktree: PathBuf = root.join("spaced ");
    git(
        main.as_path(),
        &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")],
    );
    let common: PathBuf = main.join(".git");
    record(common.as_path(), worktree.as_path()).expect("the record is written");
    let admin: PathBuf = only_registration(common.as_path());
    assert!(
        added_by_the_wrapper(admin.as_path(), worktree.as_path()),
        "the record answers for a worktree whose path ends in a space"
    );
}

/// What:
///  A registration whose administrative directory name ends in a space is still read from the worktree's pointer.
/// Why:
///   Every ASCII whitespace byte was taken from the pointer,
///  so the space that belongs to the directory name went with the terminator,
/// the registration resolved to a directory that is not there,
///  and the worktree was never recorded as one the wrapper made.
#[test]
fn the_registration_is_read_from_a_pointer_that_ends_in_a_space() {
    let added: Added = a_repository_with_one_worktree("added-pointer-space");
    let administration: PathBuf = only_registration(added.common.as_path());
    let spaced: PathBuf = administration.with_file_name("spaced ");
    std::fs::rename(administration.as_path(), spaced.as_path()).expect("the administrative directory is renamed");
    let mut pointer: Vec<u8> = Vec::from(b"gitdir: ".as_slice());
    pointer.extend_from_slice(crate::path_bytes::path_bytes_of(spaced.as_path()));
    pointer.push(b'\n');
    std::fs::write(added.worktree.join(".git"), pointer).expect("pointer file");
    record(added.common.as_path(), added.worktree.as_path()).expect("the record is written beside the registration the pointer names");
    assert!(spaced.join(ADDED_RECORD_NAME).exists(), "the record sits in the renamed registration");
}
