//! Controls for finding a command's Git directories without starting Git:
//! a `.git` pointer file,
//! a Git directory that names a common directory,
//! and both of those names holding a named pipe instead,
//! which must answer at once rather than hold the command.
//! Why:
//!  The sweep of a stopped child's locks reads these two files when Git has just failed,
//!  so a pipe at either name would hold every wrapped command in such a repository for ever
//!  (`crate::plain_file`;
//! issue 6415 for the same shape at the wrapper's own records).

/// The rules under test.
use super::{
    git_directory_of,
    roots_for,
};
/// Fixture folders and their removal.
use crate::test_support::{
    finished_within,
    fixture,
    remove,
};
use std::path::PathBuf;

/// A `.git` pointer file names the Git directory it holds,
/// relative to the working directory that holds the file.
#[test]
fn a_git_file_names_its_git_directory() {
    let root: PathBuf = fixture("git-roots-pointer");
    std::fs::create_dir_all(root.join("git/worktrees/w").as_path()).expect("the Git directory");
    std::fs::write(root.join(".git"), "gitdir: git/worktrees/w\n").expect("the pointer");
    assert_eq!(
        git_directory_of(root.as_path()),
        Some(root.join("git/worktrees/w"))
    );
    remove(root.as_path());
}

/// A `.git` that is a named pipe names no Git directory here,
/// and the walk answers at once:
/// opening a named pipe for reading waits for a writer that may never come,
/// which is what a plain read of the same name does,
/// and what the read of a name swapped for a pipe after the look would have done.
#[test]
fn a_git_file_that_is_a_named_pipe_answers_at_once() {
    let root: PathBuf = fixture("git-roots-named-pipe");
    crate::subvolume_sync_attrs::make_fifo(root.join(".git").as_path(), 0o600)
        .expect("a named pipe where a .git goes");
    let inside: PathBuf = root.clone();
    let walked = finished_within(move || return git_directory_of(inside.as_path()));
    let Some(found) = walked else {
        panic!("a `.git` that is a named pipe held the walk")
    };
    assert_ne!(
        found,
        Some(root.clone()),
        "a pipe is no pointer file, so this directory's Git directory is not named"
    );
    // The read this module made before, on the same fixture under the same bound, so the bound above is shown to see a wait.
    let pointer: PathBuf = root.join(".git");
    let waited = finished_within(move || return std::fs::read_to_string(pointer).ok());
    assert!(
        waited.is_none(),
        "a plain read of the same name waits, so the bound above can see a wait"
    );
    remove(root.as_path());
}

/// A Git directory that names a common directory gives both as roots,
/// and a `commondir` that is a named pipe gives the one directory rather than holding the sweep.
#[test]
fn a_commondir_that_is_a_named_pipe_gives_one_root_at_once() {
    let root: PathBuf = fixture("git-roots-commondir");
    let common: PathBuf = root.join("common");
    let own: PathBuf = common.join("worktrees/w");
    std::fs::create_dir_all(own.as_path()).expect("the linked worktree's Git directory");
    std::fs::write(own.join("commondir"), "../..\n").expect("the pointer");
    let both: Vec<PathBuf> = roots_for(root.as_path(), Some(own.clone()));
    assert_eq!(
        both.len(),
        2,
        "a linked worktree has its own Git directory and the common one: {both:?}"
    );
    std::fs::remove_file(own.join("commondir")).expect("the plain `commondir`");
    crate::subvolume_sync_attrs::make_fifo(own.join("commondir").as_path(), 0o600)
        .expect("a named pipe where a `commondir` goes");
    let inside: PathBuf = own.clone();
    let working: PathBuf = root.clone();
    let walked = finished_within(move || return roots_for(working.as_path(), Some(inside)));
    let Some(one) = walked else {
        panic!("a `commondir` that is a named pipe held the sweep")
    };
    assert_eq!(
        one,
        vec![own],
        "a pipe names no common directory, so the Git directory is the one root"
    );
    remove(root.as_path());
}
