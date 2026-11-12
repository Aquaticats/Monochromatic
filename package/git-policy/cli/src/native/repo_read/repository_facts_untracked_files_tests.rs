//! What:
//!  The untracked-and-tracked fact read against a repository real Git holds.
//! Why:
//!   The query's own shape is checked beside the builder that makes it;
//!  what is left here is the answer Git gives,
//!  and the way a reading must never become "nothing stands in the way":
//!  a tree that cannot be read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(untrackedFilesTheTreeTracks(git, 'HEAD')).toEqual(['victim.txt']);
//! ```

/// The reading under test.
use super::untracked_files_the_tree_tracks;
/// The provider, built on a fixture repository.
use crate::repository_facts::{GitFacts, git_facts};
/// A disposable repository, its real Git and its removal.
use crate::command_test_support::{REAL_GIT, fixture, git, remove, repository};
/// The file operations the controls make.
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The provider whose one global option names `repo`,
///  so Git answers for that repository and not for this process's directory.
fn facts_for(repo: &Path) -> GitFacts {
    let global: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    return git_facts(Path::new(REAL_GIT), global.as_slice(), &[]);
}

/// A tracked file the worktree lost and a person put back as an untracked file is named;
///  the same repository without that file answers nothing.
#[test]
fn the_untracked_file_at_a_tracked_path_is_named() {
    let root: PathBuf = fixture("untracked-and-tracked");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("victim.txt"), b"tracked\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", "victim.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let mut facts: GitFacts = facts_for(repo.as_path());
    assert_eq!(
        untracked_files_the_tree_tracks(&mut facts, OsStr::new("HEAD")).expect("the tree reads"),
        Vec::<String>::new(),
        "a tracked file Git knows is not an untracked one"
    );
    std::fs::remove_file(repo.join("victim.txt")).expect("the tracked file goes");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "victim.txt"]);
    std::fs::write(repo.join("victim.txt"), b"the person's own file\n").expect("the untracked file");
    let mut facts: GitFacts = facts_for(repo.as_path());
    assert_eq!(
        untracked_files_the_tree_tracks(&mut facts, OsStr::new("HEAD")).expect("the tree reads"),
        vec![String::from("victim.txt")],
        "the untracked file at a path HEAD tracks is the one Git refuses to write over"
    );
    // A file at a path the tree never held is not in the way of the tree.
    std::fs::write(repo.join("elsewhere.txt"), b"unrelated\n").expect("another untracked file");
    let mut facts: GitFacts = facts_for(repo.as_path());
    assert_eq!(
        untracked_files_the_tree_tracks(&mut facts, OsStr::new("HEAD")).expect("the tree reads"),
        vec![String::from("victim.txt")],
        "only the untracked file at a path the tree tracks is named"
    );
    remove(&root);
}

/// A revision Git cannot resolve is a failure,
///  never an empty answer that reads as clean.
#[test]
fn a_tree_that_cannot_be_read_is_a_failure() {
    let root: PathBuf = fixture("untracked-and-tracked-missing-tree");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut facts: GitFacts = facts_for(repo.as_path());
    let answer: Result<Vec<String>, String> = untracked_files_the_tree_tracks(&mut facts, OsStr::new("no-such-revision"));
    assert!(answer.is_err(), "a tree Git cannot read is not an empty answer: {answer:?}");
    remove(&root);
}
