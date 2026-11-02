//! Controls for the snapshot source: a clean subvolume main checkout is one, and each condition that rules a source out is
//!  refused with its reason.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{age, git, git_context, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, subvolume_repository, tracked};

/// A clean subvolume main checkout is a source, and it names the commit and tree of its `HEAD`.
#[test]
fn btrfs_a_clean_subvolume_main_checkout_is_a_source() {
    let Some(root) = btrfs_fixture("source-clean") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let source: SnapshotSource = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    assert_eq!(source.root, repo);
    assert_eq!(source.clean.head_commit, String::from_utf8_lossy(&git(repo.as_path(), &["rev-parse", "HEAD"]).stdout).trim());
    assert!(source.nested.is_empty());
    remove(root.as_path());
}

/// Each refusal names its reason, and the refused source is left as it was.
#[test]
fn btrfs_each_condition_that_rules_a_source_out_is_refused_with_its_reason() {
    let Some(root) = btrfs_fixture("source-refused") else { return };
    let plan = plan_to(root.join("topic"));
    let refusal = |repo: &Path| -> String {
        return choose_snapshot_source(&git_context(), &main_identity(repo), &plan).expect_err("refused");
    };
    // An ordinary directory is no subvolume root.
    let ordinary: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    assert!(refusal(ordinary.as_path()).contains("is not a btrfs subvolume"));
    // A bare repository and a missing worktree have nothing to snapshot.
    let bare = WorktreeIdentity::BareRepository { common_dir: root.clone(), git_dir: root.clone() };
    assert!(choose_snapshot_source(&git_context(), &bare, &plan).expect_err("bare").contains("no worktree"));
    // An uncommitted change.
    let repo: PathBuf = subvolume_repository(root.as_path());
    assert!(choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).is_ok(), "positive control");
    std::fs::write(repo.join("shared.txt"), b"edited\n").expect("edit");
    assert!(refusal(repo.as_path()).contains("shared.txt"));
    git(repo.as_path(), &["checkout", "--quiet", "--", "shared.txt"]);
    age(repo.join("shared.txt").as_path());
    git(repo.as_path(), &["update-index", "--refresh"]);
    // A sparse checkout.
    std::fs::create_dir_all(repo.join(".git/info")).expect("info");
    std::fs::write(repo.join(".git/info/sparse-checkout"), b"/*\n").expect("patterns");
    assert!(refusal(repo.as_path()).contains("sparse checkout"));
    std::fs::remove_file(repo.join(".git/info/sparse-checkout")).expect("remove patterns");
    // A worktree-specific configuration.
    git(repo.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    git(repo.as_path(), &["config", "--worktree", "core.autocrlf", "true"]);
    assert!(refusal(repo.as_path()).contains("worktree-specific configuration sets core.autocrlf"));
    git(repo.as_path(), &["config", "--worktree", "--unset", "core.autocrlf"]);
    // A nested worktree that is an ordinary directory.
    tracked(repo.as_path(), ".gitignore", b"ignored/\nnested/\n");
    commit_all(repo.as_path(), "ignore nested");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "nested", "nested/wt"]);
    let nested: String = refusal(repo.as_path());
    assert!(nested.contains("inside the source is an ordinary directory"), "{nested}");
    remove(root.as_path());
}

/// A registered nested worktree whose recorded path passes through a link is a blocker of the snapshot rung; one reached by
///  its real path is not, and one that is gone is nothing.
#[test]
fn a_nested_worktree_reached_through_a_link_is_found() {
    let root: PathBuf = crate::test_support::fixture("source-nested-link");
    let real: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    std::fs::create_dir_all(real.join("moved/wt")).expect("moved");
    std::os::unix::fs::symlink(real.join("moved"), real.join("group")).expect("link");
    assert_eq!(nested_through_link(&[real.join("group/wt")]), Some(real.join("group/wt")));
    assert_eq!(nested_through_link(&[real.join("moved/wt"), real.join("gone/wt")]), None);
    crate::test_support::remove(root.as_path());
}
