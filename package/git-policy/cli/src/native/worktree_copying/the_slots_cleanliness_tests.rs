//! Controls for Git's own cleanliness reading at a slot:
//! a clean worktree,
//! one holding a modified tracked file,
//! and one holding an untracked file.
//! Why the three:
//!  Git's removal refuses both kinds of change with the same sentence and
//!  deletes a clean worktree without a force
//!  (`builtin/worktree.c:1464` at Git `v2.56.1`),
//!  so each arm is a state a person meets.

use super::*;
use crate::test_support::{
    fixture,
    git,
    git_context,
    remove,
};
use std::path::PathBuf;

/// What:
///  One repository with one commit and one linked worktree of it,
///  whose registration is the one Git made.
/// Why:
///  The proof reads the registration's index and `HEAD`,
///  so a fixture that writes a registration by hand would prove something no
///  worktree meets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [root, worktree, registration] = aRepositoryAndItsWorktree('cleanliness');
/// ```
fn a_repository_and_its_worktree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repository: PathBuf = root.join("repo");
    std::fs::create_dir_all(repository.as_path()).expect("the repository folder");
    git(repository.as_path(), &["init", "--quiet"]);
    std::fs::write(repository.join("a.txt"), b"one\n").expect("the tracked file writes");
    git(repository.as_path(), &["add", "a.txt"]);
    git(repository.as_path(), &["commit", "--quiet", "-m", "one"]);
    let worktree: PathBuf = root.join("worktree");
    git(
        repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            worktree.to_str().expect("the fixture path is text"),
        ],
    );
    let registration: PathBuf = repository.join(".git/worktrees/worktree");
    return (root, worktree, registration);
}

/// One reading of the three places a cleanliness check needs.
pub(super) fn the_places<'a>(
    slot: &'a Path,
    registration: &'a Path,
    common_dir: &'a Path,
) -> TheSlotToCheck<'a> {
    return TheSlotToCheck {
        slot,
        registration,
        common_dir,
    };
}

/// What:
///  A worktree Git would delete without a force is proved clean,
///  and one holding a modified tracked file or an untracked file is not.
/// Why:
///  The refusal is what stops a removal without force from destroying files a
///  person never committed,
///  and a reading that answered clean for either change would take them.
#[test]
fn a_worktree_with_changes_is_not_clean() {
    let (root, worktree, registration) = a_repository_and_its_worktree("cleanliness-changes");
    let common: PathBuf = root.join("repo/.git");
    let context: crate::transaction_git::GitContext = git_context();
    let reading = TheLiveCleanliness {
        context: &context,
    };
    let untouched = reading.the_slot_is_clean(&the_places(
        worktree.as_path(),
        registration.as_path(),
        common.as_path(),
    ));
    assert!(
        untouched.is_ok(),
        "a worktree Git added and nobody touched is clean: {untouched:?}"
    );
    std::fs::write(worktree.join("a.txt"), b"changed\n").expect("the edit writes");
    let modified: String = reading
        .the_slot_is_clean(&the_places(
            worktree.as_path(),
            registration.as_path(),
            common.as_path(),
        ))
        .expect_err("a modified tracked file is not clean");
    assert!(
        modified.contains("a.txt") || modified.to_lowercase().contains("modif"),
        "{modified}"
    );
    std::fs::write(worktree.join("a.txt"), b"one\n").expect("the file is put back");
    std::fs::write(worktree.join("new.txt"), b"untracked\n").expect("the untracked file writes");
    let untracked: String = reading
        .the_slot_is_clean(&the_places(
            worktree.as_path(),
            registration.as_path(),
            common.as_path(),
        ))
        .expect_err("an untracked file is not clean");
    assert!(
        untracked.contains("new.txt") || untracked.to_lowercase().contains("untracked"),
        "{untracked}"
    );
    remove(root.as_path());
}
