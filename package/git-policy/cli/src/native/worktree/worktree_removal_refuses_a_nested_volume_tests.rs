//! What:
//!  Controls for the words of a removal refused because the worktree holds a subvolume or a mount point of its own.
//! Why:
//!   The destroy can never take such a worktree,
//! so the report is the only thing the person acts on:
//! it has to name
//!  what stands in the way,
//! what that holds,
//! and the one command that clears it without losing the work there,
//!  which is a different command for a linked worktree and for a subvolume that is none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refusalForNestedVolume({ root, target, commonDir, found })).toContain('git worktree remove');
//! ```

use super::*;
use crate::subvolume_remove_walk::Boundary;
use crate::test_support::{fixture, git, repository};
use std::path::PathBuf;

/// The words a walk uses for a directory that holds another subvolume.
const A_NESTED_SUBVOLUME: &str = "a nested subvolume or another filesystem";

/// What:
///  A linked worktree inside the worktree is named with its branch,
///  with the command that removes it while keeping that branch,
///  and with the statement that nothing was removed.
/// Why:
///   `git worktree remove` for the inner worktree keeps its branch and refuses while anything there is uncommitted,
/// which is the
///  way forward that loses nothing;
/// a report that named `btrfs subvolume delete` instead would take the
///  inner worktree's files.
#[test]
fn a_nested_linked_worktree_is_named_with_its_branch_and_the_command_that_removes_it() {
    let root: PathBuf = fixture("nested-volume-named");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let outer: PathBuf = root.join("wt");
    let inner: PathBuf = outer.join(".claude/worktrees/inner");
    std::fs::create_dir_all(inner.as_path()).expect("the folder a worktree inside a worktree stands in");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "inner", inner.to_str().expect("a UTF-8 fixture path")]);
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let registered: PathBuf = std::fs::canonicalize(inner.as_path()).expect("canonical");
    let report: String = refusal_for_nested_volume(
        outer.as_path(),
        common_dir.as_path(),
        &Boundary { path: registered.clone(), what: A_NESTED_SUBVOLUME },
    );
    assert!(report.contains(registered.to_str().expect("UTF-8")), "the report names the worktree inside: {report}");
    assert!(report.contains("refs/heads/inner"), "and the branch it has checked out: {report}");
    assert!(report.contains(&format!("git worktree remove {inner}", inner = registered.display())), "and the command that removes it: {report}");
    assert!(report.contains("Nothing was removed"), "and says nothing changed: {report}");
    assert!(!report.contains("btrfs subvolume delete"), "and never sends the person to delete the inner worktree's files: {report}");
}

/// What:
///  A subvolume that is no worktree of this repository is named with the command that deletes it,
///  and nothing was removed.
/// Why:
///   A subvolume of the person's own is the same to the kernel as a nested worktree,
/// and no branch can be named
///  for it,
/// so the report gives the one command that takes it and the way to keep it.
#[test]
fn a_subvolume_that_is_no_worktree_is_named_with_the_command_that_deletes_it() {
    let root: PathBuf = fixture("nested-volume-unnamed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let target: PathBuf = root.join("wt");
    let mine: PathBuf = target.join("mine");
    let report: String = refusal_for_nested_volume(
        target.as_path(),
        common_dir.as_path(),
        &Boundary { path: mine.clone(), what: A_NESTED_SUBVOLUME },
    );
    assert!(report.contains(mine.to_str().expect("UTF-8")), "the report names the volume that stands there: {report}");
    assert!(report.contains(&format!("btrfs subvolume delete {mine}", mine = mine.display())), "and the command that deletes it: {report}");
    assert!(report.contains("Nothing was removed"), "and says nothing changed: {report}");
}
