//! What:
//!  The common Git directory of a worktree's own Git directory,
//!  through each of the three answers a Git directory can give.
//! Why:
//!  The timing record of a linked worktree must land in the repository's own directory,
//!  and a `commondir` with a trailing carriage return,
//!  a relative path or a place that is gone
//!  must not send it anywhere but where Git would.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(commonDirectoryOf(join(common, 'worktrees/w'))).toBe(common);
//! ```

/// The rule under test.
use super::common_directory_of;
use crate::command_test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// A Git directory with no `commondir` file is its own common directory.
#[test]
fn a_git_directory_without_a_commondir_file_is_its_own() {
    let root: PathBuf = fixture("common-dir-own");
    let git_dir: PathBuf = root.join(".git");
    std::fs::create_dir_all(&git_dir).expect("git directory");
    assert_eq!(common_directory_of(git_dir.as_path()), git_dir);
    remove(root.as_path());
}

/// A linked worktree's Git directory answers with the directory its `commondir` names,
/// however many line endings the file ends with.
#[test]
fn a_linked_worktrees_commondir_names_the_repositorys_directory() {
    let root: PathBuf = fixture("common-dir-linked");
    let common: PathBuf = root.join("main/.git");
    let own: PathBuf = common.join("worktrees/w");
    std::fs::create_dir_all(&own).expect("worktree git directory");
    for ending in ["../..\n", "../..\r\n", "../..\r\n\r\n"] {
        std::fs::write(own.join("commondir"), ending).expect("commondir");
        assert_eq!(common_directory_of(own.as_path()), std::fs::canonicalize(&common).expect("canonical"), "{ending:?}");
    }
    remove(root.as_path());
}

/// An empty `commondir` file leaves the Git directory its own common directory,
/// and so does one that names a place that is gone,
/// which is used as written.
#[test]
fn an_empty_or_missing_place_leaves_a_path_that_is_used() {
    let root: PathBuf = fixture("common-dir-empty");
    let own: PathBuf = root.join("w");
    std::fs::create_dir_all(&own).expect("git directory");
    std::fs::write(own.join("commondir"), b"\n").expect("commondir");
    assert_eq!(common_directory_of(own.as_path()), own);
    std::fs::write(own.join("commondir"), b"../gone\n").expect("commondir");
    // A place that cannot be resolved is used as written, so a record still lands under the name the file gives rather than
    //  nowhere at all.
    assert_eq!(common_directory_of(own.as_path()), own.join("../gone"));
    remove(root.as_path());
}

/// The path a rule answers is what the caller joins a record onto,
/// so a relative `commondir` never lands beside the worktree's own Git directory.
#[test]
fn a_relative_commondir_is_resolved_against_the_git_directory() {
    let root: PathBuf = fixture("common-dir-relative");
    let common: PathBuf = root.join("main/.git");
    let own: PathBuf = common.join("worktrees/w");
    std::fs::create_dir_all(&own).expect("worktree git directory");
    std::fs::write(own.join("commondir"), b"../..\n").expect("commondir");
    let answered: PathBuf = common_directory_of(own.as_path());
    assert!(answered.ends_with(".git"), "{}", answered.display());
    assert!(!answered.ends_with(Path::new("worktrees/w")), "{}", answered.display());
    remove(root.as_path());
}
