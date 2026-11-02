//! Controls for the removal of an incomplete worktree: the pair the repository confirms is removed whole, and every pair
//!  it does not confirm is left alone.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, git, git_text, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, tracked};

/// An ordinary repository with a worktree registered without a checkout beside it: the repository, the destination, and the
///  administrative directory.
fn registered(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "file.txt", b"content\n");
    commit_all(repo.as_path(), "first");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    return (repo, destination, admin);
}

/// A worktree holding only Git's pointer, or a partial checkout, is removed with its registration, and the repository lists
///  the main worktree alone afterwards.
#[test]
fn a_registered_incomplete_worktree_is_removed_with_its_registration() {
    for partial in [false, true] {
        let root: PathBuf = fixture(if partial { "rollback-partial" } else { "rollback-pointer" });
        let (repo, destination, admin) = registered(root.as_path());
        if partial {
            std::fs::create_dir_all(destination.join("dir")).expect("directory");
            std::fs::write(destination.join("dir/file"), b"half").expect("half a checkout");
        }
        remove_incomplete_worktree(repo.join(".git").as_path(), destination.as_path()).expect("removed");
        assert!(!destination.exists() && !admin.exists(), "partial {partial}");
        assert_eq!(git_text(repo.as_path(), &["worktree", "list", "--porcelain"]).lines().filter(|line: &&str| { return line.starts_with("worktree "); }).count(), 1);
        assert!(repo.join("file.txt").is_file(), "the main worktree is untouched");
        remove(root.as_path());
    }
}

/// A destination whose pointer names nothing of this repository, or whose registration names another place, is left whole.
#[test]
fn a_pair_the_repository_does_not_confirm_is_never_removed() {
    let root: PathBuf = fixture("rollback-unconfirmed");
    let (repo, destination, admin) = registered(root.as_path());
    let common: PathBuf = repo.join(".git");
    // A pointer at a directory outside the repository's worktrees.
    let foreign: PathBuf = root.join("foreign");
    std::fs::create_dir_all(foreign.join("admin")).expect("foreign admin");
    std::fs::write(foreign.join("admin/gitdir"), format!("{}\n", foreign.join("look/.git").display())).expect("gitdir");
    std::fs::create_dir_all(foreign.join("look")).expect("look");
    std::fs::write(foreign.join("look/.git"), format!("gitdir: {}\n", foreign.join("admin").display())).expect("pointer");
    std::fs::write(foreign.join("look/keep.txt"), b"mine").expect("keep");
    assert!(remove_incomplete_worktree(common.as_path(), foreign.join("look").as_path()).expect_err("not registered").contains("not registered"));
    assert!(foreign.join("look/keep.txt").exists() && foreign.join("admin").exists());
    // A registration that names another pointer.
    std::fs::write(admin.join("gitdir"), format!("{}\n", root.join("elsewhere/.git").display())).expect("re-pointed");
    assert!(remove_incomplete_worktree(common.as_path(), destination.as_path()).expect_err("not named back").contains("does not name"));
    assert!(destination.join(".git").exists() && admin.exists());
    // Positive control: the confirmed pair goes.
    std::fs::write(admin.join("gitdir"), format!("{}\n", destination.join(".git").display())).expect("restored");
    remove_incomplete_worktree(common.as_path(), destination.as_path()).expect("confirmed pair removed");
    assert!(!destination.exists() && !admin.exists());
    remove(root.as_path());
}

/// A destination reached through a link is not a real path, so nothing is removed through it.
#[test]
fn a_destination_named_through_a_link_removes_nothing() {
    let root: PathBuf = fixture("rollback-link");
    let (repo, destination, admin) = registered(root.as_path());
    let link: PathBuf = root.join("link");
    std::os::unix::fs::symlink(destination.as_path(), link.as_path()).expect("link");
    assert!(remove_incomplete_worktree(repo.join(".git").as_path(), link.as_path()).expect_err("a link").contains("not a real path"));
    assert!(destination.join(".git").exists() && admin.exists());
    remove(root.as_path());
}

/// A destination that is a snapshot subvolume with contents is destroyed whole.
#[test]
fn btrfs_a_subvolume_destination_with_contents_is_removed() {
    let Some(root) = btrfs_fixture("rollback-subvolume") else { return };
    let (repo, destination, admin) = registered(root.as_path());
    std::fs::remove_dir_all(destination.as_path()).expect("replace the directory");
    crate::fs_btrfs_ioctl::subvolume_create_at(destination.as_path()).expect("subvolume");
    std::fs::write(destination.join(".git"), format!("gitdir: {}\n", admin.display())).expect("pointer");
    std::fs::create_dir_all(destination.join("deep/er")).expect("nested");
    std::fs::write(destination.join("deep/er/file"), b"x").expect("file");
    remove_incomplete_worktree(repo.join(".git").as_path(), destination.as_path()).expect("removed");
    assert!(!destination.exists() && !admin.exists());
    remove(root.as_path());
}
