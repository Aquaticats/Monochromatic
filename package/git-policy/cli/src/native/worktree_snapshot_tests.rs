//! Controls for the snapshot mechanics on btrfs: the stage becomes a private snapshot with the new worktree's pointer, no
//!  stubs and the source's index; a source that changed after the clean check is refused; and the swap leaves the
//!  stage name holding what Git made.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::test_support::{age, git, git_context, remove};
use crate::worktree_ladder_test_support::{main_identity, plan_to, subvolume_repository};
use crate::worktree_snapshot_source::choose_snapshot_source;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// Git's own `--no-checkout` worktree beside the repository: the empty directory with a `.git` file, and its
///  administrative directory.
fn no_checkout_worktree(repo: &Path, name: &str) -> (PathBuf, PathBuf, Vec<u8>) {
    git(repo, &["worktree", "add", "--quiet", "--no-checkout", "-b", name, format!("../{name}").as_str()]);
    let destination: PathBuf = std::fs::canonicalize(repo.join("..").join(name)).expect("destination");
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).expect("pointer");
    let admin: PathBuf = repo.join(".git/worktrees").join(name);
    return (destination, admin, pointer);
}

/// The snapshot is a private subvolume that holds the files, the new pointer, the index, and no stub of the destination.
#[test]
fn btrfs_the_stage_becomes_a_worktree_with_the_new_pointer_and_the_sources_index() {
    let Some(root) = btrfs_fixture("snapshot-stage") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination.clone());
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    take_snapshot(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect("snapshot");
    assert!(is_subvolume_root(stage.as_path()).expect("probe"));
    assert_eq!(std::fs::metadata(stage.as_path()).expect("metadata").mode() & 0o777, 0o700, "private until placed");
    assert_eq!(std::fs::read(stage.join(".git")).expect("pointer"), pointer);
    assert_eq!(std::fs::read(stage.join("ignored/pkg/index.js")).expect("ignored state"), b"module.exports = 1;\n");
    let index: Vec<u8> = std::fs::read(admin.join("index")).expect("index");
    let entries = parse_index(index.as_slice(), ObjectFormat::Sha1).expect("a plain index");
    assert_eq!(entries.len(), source.clean.tracked);
    git(stage.as_path(), &["--git-dir", admin.to_str().expect("utf8"), "--work-tree", stage.to_str().expect("utf8"), "status", "--porcelain=v1"]);
    remove(root.as_path());
}

/// A source that changed after the clean check is refused by the comparison of the snapshot with the copied index.
#[test]
fn btrfs_a_source_that_changed_after_the_check_is_refused() {
    let Some(root) = btrfs_fixture("snapshot-changed") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    // The same size, a different time: Git's fast path sees a change.
    std::fs::write(repo.join("shared.txt"), b"SECOND, LONGER\n").expect("edit after the check");
    age(repo.join("shared.txt").as_path());
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let reason: String = take_snapshot(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect_err("refused");
    assert!(reason.contains("the source changed while it was copied"), "{reason}");
    assert!(!admin.join("index").exists(), "no index is installed for a refused snapshot");
    remove(root.as_path());
}

/// The swap puts the stage at the destination with the source's mode and leaves Git's directory at the stage name,
///  which `remove_leftover` removes.
#[test]
fn btrfs_the_swap_puts_the_stage_at_the_destination() {
    let Some(root) = btrfs_fixture("snapshot-swap") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination.clone());
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    take_snapshot(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect("snapshot");
    put_in_place(stage.as_path(), destination.as_path(), 0o750).expect("swap");
    assert!(is_subvolume_root(destination.as_path()).expect("probe"));
    assert_eq!(std::fs::metadata(destination.as_path()).expect("metadata").mode() & 0o7777, 0o750);
    assert_eq!(std::fs::read(destination.join("shared.txt")).expect("tracked"), b"second, longer\n");
    assert!(stage.join(".git").is_file() && !stage.join("shared.txt").exists(), "Git's directory is at the stage name");
    remove_leftover(stage.as_path()).expect("leftover");
    assert!(!stage.exists());
    assert_eq!(git(destination.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    remove(root.as_path());
}
