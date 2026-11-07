//! Controls for the migration's lock files:
//! the paths of a tree,
//! creation,
//! recognition of the migration's own lock,
//!  and release that never touches another lock.

use super::*;
use crate::test_support::{fixture, remove};

/// A tree with a main `.git`,
/// two administrative directories of linked worktrees and a stray file among them.
fn tree_with_worktrees(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let tree: PathBuf = parent.join("tree");
    std::fs::create_dir_all(tree.join(".git/worktrees/b")).expect("b");
    std::fs::create_dir_all(tree.join(".git/worktrees/a")).expect("a");
    std::fs::write(tree.join(".git/worktrees/stray"), b"x").expect("a file is not an administrative directory");
    return (parent, tree);
}

/// The lock paths are the main one and one per administrative directory,
/// sorted,
/// and relative forms strip the tree.
#[test]
fn the_lock_paths_are_the_main_one_and_one_per_linked_worktree() {
    let (parent, tree) = tree_with_worktrees("lock-paths");
    assert_eq!(lock_paths(tree.as_path()), vec![tree.join(".git/index.lock"), tree.join(".git/worktrees/a/index.lock"), tree.join(".git/worktrees/b/index.lock")]);
    assert_eq!(
        relative_lock_paths(tree.as_path()),
        vec![PathBuf::from(".git/index.lock"), PathBuf::from(".git/worktrees/a/index.lock"), PathBuf::from(".git/worktrees/b/index.lock")]
    );
    assert_eq!(lock_paths(parent.join("absent").as_path()), vec![parent.join("absent/.git/index.lock")]);
    remove(parent.as_path());
}

/// A lock is created with the token,
/// accepted again with the same token,
/// refused with another,
/// and a foreign lock is
///  never accepted or released.
#[test]
fn only_a_lock_holding_the_token_is_accepted_and_released() {
    let (parent, tree) = tree_with_worktrees("lock-tokens");
    let (mine, other): (&str, &str) = ("migration one\n", "migration two\n");
    for lock in lock_paths(tree.as_path()) {
        acquire(lock.as_path(), mine).expect("created");
        acquire(lock.as_path(), mine).expect("accepted again");
        assert!(is_ours(lock.as_path(), mine) && !is_ours(lock.as_path(), other));
        let refused: MigrateError = acquire(lock.as_path(), other).expect_err("another migration's lock");
        assert!(refused.to_string().contains("is not this migration's lock") && refused.to_string().contains("index.lock"), "{refused}");
    }
    std::fs::write(tree.join(".git/worktrees/a/index.lock"), b"a Git command\n").expect("a foreign lock replaces one");
    assert_eq!(release_all(tree.as_path(), mine).expect("released"), 2);
    assert!(tree.join(".git/worktrees/a/index.lock").exists() && !tree.join(".git/index.lock").exists());
    assert_eq!(release_all(tree.as_path(), mine).expect("nothing left of ours"), 0);
    remove(parent.as_path());
}

/// A directory where a lock would be is an error from the operating system,
/// named with the path.
#[test]
fn a_lock_that_cannot_be_created_names_the_path() {
    let (parent, tree) = tree_with_worktrees("lock-error");
    let error: MigrateError = acquire(tree.join("no-such-directory/index.lock").as_path(), "token\n").expect_err("no directory");
    assert!(error.to_string().contains("cannot create the lock") && error.to_string().contains("no-such-directory"), "{error}");
    remove(parent.as_path());
}
