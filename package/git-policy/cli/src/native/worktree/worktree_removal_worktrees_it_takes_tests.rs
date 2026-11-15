//! Controls for the list of worktrees one removal takes:
//! the one it names,
//! every worktree registered inside it,
//! a registration that cannot be read,
//! and a registration Git prunes.
//!
//! Each control states one answer the list gives,
//! because every question of every route is a question about this list.

use super::{Taken, the_worktrees_it_takes};
use crate::test_support::{fixture, git, remove, repository};
use std::path::PathBuf;

/// What:
///  A removal of an empty worktree takes that worktree alone.
/// Why:
///  Every other control adds a worktree inside it,
///  and one that always answered "everything" would pass them all.
#[test]
fn a_removal_of_a_worktree_with_none_inside_takes_it_alone() {
    let root: PathBuf = fixture("worktrees-it-takes-alone");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let worktree: PathBuf = root.join("topic");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", worktree.to_str().expect("a UTF-8 fixture path")]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    let target: PathBuf = std::fs::canonicalize(worktree.as_path()).expect("the worktree's real path");
    let admin: PathBuf = common.join("worktrees").join("topic");
    assert_eq!(the_worktrees_it_takes(common.as_path(), target.as_path(), admin.as_path()), Ok(vec![Taken { root: target, admin }]));
    remove(root.as_path());
}

/// What:
///  A worktree registered inside the target is on the list,
///  after the target itself.
/// Why:
///  The removal deletes the inner checkout and its own Git directory with the outer one,
///  so every question must see it
///  (a review of `8858f843ee`, finding 2).
#[test]
fn a_removal_takes_the_worktree_registered_inside_it() {
    let root: PathBuf = fixture("worktrees-it-takes-nested");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let outer: PathBuf = root.join("outer");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "outer", outer.to_str().expect("a UTF-8 fixture path")]);
    let inner: PathBuf = outer.join("inner");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "inner", inner.to_str().expect("a UTF-8 fixture path")]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    let target: PathBuf = std::fs::canonicalize(outer.as_path()).expect("the worktree's real path");
    let inner_real: PathBuf = std::fs::canonicalize(inner.as_path()).expect("the nested worktree's real path");
    let taken: Vec<Taken> = the_worktrees_it_takes(common.as_path(), target.as_path(), common.join("worktrees/outer").as_path()).expect("the registrations can be read");
    assert_eq!(taken.len(), 2, "the target and the worktree inside it: {taken:?}");
    assert_eq!(taken[0].root, target, "the target comes first");
    assert_eq!(taken[1].root, inner_real, "and the worktree inside it after it");
    assert_eq!(taken[1].admin, common.join("worktrees/inner"), "with its own Git directory");
    remove(root.as_path());
}

/// What:
///  A registration that cannot be read is the reason nothing is removed.
/// Why:
///  Such a registration may name a worktree inside the target,
///  and answering a list without it is how the nested checkout went with its hidden edit
///  (a review of `8858f843ee`, finding 2).
#[test]
fn an_unreadable_registration_is_the_reason_no_list_is_given() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("worktrees-it-takes-unreadable");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let outer: PathBuf = root.join("outer");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "outer", outer.to_str().expect("a UTF-8 fixture path")]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    let target: PathBuf = std::fs::canonicalize(outer.as_path()).expect("the worktree's real path");
    let registration: PathBuf = common.join("worktrees/outer/gitdir");
    std::fs::set_permissions(registration.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a registration nothing may read");
    let refused: Result<Vec<Taken>, String> = the_worktrees_it_takes(common.as_path(), target.as_path(), common.join("worktrees/outer").as_path());
    assert!(refused.is_err(), "a registration this program cannot read is the reason nothing is removed");
    std::fs::set_permissions(registration.as_path(), std::fs::Permissions::from_mode(0o644)).expect("readable again");
    remove(root.as_path());
}
