//! The rule that a repair touches only the worktree it serves:
//! a directory that is the worktree's own,
//! one reached through a link that leaves it,
//! one that is not there yet,
//! and a name outside the worktree at all.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// What:
///  A worktree with a directory a link leaves,
/// a directory a link keeps inside it,
/// a directory that is not there yet,
/// and a name outside the worktree altogether.
/// Why:
///   The four answers are the whole rule:
/// only the first is the worktree's own,
/// and a clean or an install in the other three would reach somebody else's files or make nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(isInsideTheWorktree(wt, join(wt, 'target')));
/// assert(!isInsideTheWorktree(wt, join(wt, 'linked-target')));
/// ```
#[test]
fn a_directory_is_the_worktrees_own_only_when_its_links_stay_inside_it() {
    let root: PathBuf = fixture("own-directory");
    let worktree: PathBuf = root.join("worktree");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(worktree.join("target/debug")).expect("the worktree's own build directory");
    std::fs::create_dir_all(elsewhere.join("debug")).expect("a directory outside the worktree");
    std::fs::create_dir_all(worktree.join("inside")).expect("a directory inside the worktree");
    std::os::unix::fs::symlink(elsewhere.as_path(), worktree.join("linked-target")).expect("a link out of the worktree");
    std::os::unix::fs::symlink(worktree.join("inside"), worktree.join("linked-inside")).expect("a link inside the worktree");
    assert!(is_inside_the_worktree(worktree.as_path(), worktree.join("target").as_path()), "a directory of the worktree's own is its own");
    assert!(!is_inside_the_worktree(worktree.as_path(), worktree.join("linked-target").as_path()), "a link out of the worktree reaches another directory's files");
    assert!(is_inside_the_worktree(worktree.as_path(), worktree.join("linked-inside").as_path()), "a link that stays inside the worktree is the worktree's own");
    assert!(is_inside_the_worktree(worktree.as_path(), worktree.join("not-made-yet").as_path()), "a directory the worktree has not made yet is its own place to make");
    assert!(!is_inside_the_worktree(worktree.as_path(), elsewhere.as_path()), "a directory outside the worktree is never its own");
    remove(&root);
}

/// What:
///  A worktree path that is not there has no inside.
/// Why:
///   A worktree the repair cannot resolve cannot be shown to hold the directory,
/// so the answer is no rather than a path comparison that would delete files nothing proved the worktree's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(!isInsideTheWorktree('/nonexistent', '/nonexistent/target'));
/// ```
#[test]
fn a_worktree_that_cannot_be_resolved_has_no_inside() {
    let absent: &Path = Path::new("/nonexistent/monochromatic-prewarm-control/worktree");
    assert!(!is_inside_the_worktree(absent, absent.join("target").as_path()), "nothing is the own directory of a worktree that is not there");
}
