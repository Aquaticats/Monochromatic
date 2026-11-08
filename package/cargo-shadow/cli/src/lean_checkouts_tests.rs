//! The tests of the checkout listing:
//! the main checkout and its registered worktrees are found from a worktree or from the main directory,
//! and a worktree whose directory is gone is not listed.

use std::{fs, path::PathBuf};

use crate::lean_checkouts::{common_git_directory, list_checkouts};
use crate::test_scratch::scratch;

/// Makes a main checkout with one live and one gone worktree.
fn repository(root: &std::path::Path) -> (PathBuf, PathBuf, PathBuf) {
    let main: PathBuf = root.join("main");
    fs::create_dir_all(main.join(".git").join("worktrees")).expect("main git directory");
    let live: PathBuf = root.join("live");
    let gone: PathBuf = root.join("gone");
    fs::create_dir_all(&live).expect("live worktree");
    for (name, path) in [("live", &live), ("gone", &gone)] {
        let own: PathBuf = main.join(".git").join("worktrees").join(name);
        fs::create_dir_all(&own).expect("worktree entry");
        fs::write(own.join("gitdir"), format!("{}\n", path.join(".git").display())).expect("gitdir");
        fs::write(own.join("commondir"), "../..\n").expect("commondir");
    }
    fs::write(live.join(".git"), format!("gitdir: {}\n", main.join(".git").join("worktrees").join("live").display())).expect("dot git file");
    return (main, live, gone);
}

#[test]
fn the_main_checkout_and_the_live_worktrees_are_listed_from_either_place() {
    let root = scratch();
    let (main, live, gone) = repository(root.path());
    let from_main: Vec<PathBuf> = list_checkouts(&main);
    let from_worktree: Vec<PathBuf> = list_checkouts(&live);
    assert_eq!(from_main, from_worktree);
    assert!(from_main.contains(&main) && from_main.contains(&live));
    assert!(!from_main.contains(&gone));
}

#[test]
fn the_common_directory_of_a_worktree_is_the_main_git_directory() {
    let root = scratch();
    let (main, live, _gone) = repository(root.path());
    let expected: PathBuf = fs::canonicalize(main.join(".git")).expect("canonical");
    assert_eq!(common_git_directory(&live), Some(expected));
}

#[test]
fn a_directory_that_is_no_checkout_lists_nothing() {
    let root = scratch();
    assert!(list_checkouts(root.path()).is_empty());
}
