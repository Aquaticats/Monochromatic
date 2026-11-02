//! Controls for the clone rung: the plain-path refusal, the clone of files, links, modes and times with the source's index
//!  installed, a source that changed after the check, and the clearing of a partial destination.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{git, git_context, git_text, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, main_identity, tracked};
use crate::worktree_snapshot_source::common_source;

/// An ordinary-directory repository with an executable file, a link, a nested file, aged, and a `--no-checkout` worktree.
fn clone_fixture(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), ".gitignore", b"ignored/\n");
    tracked(repo.as_path(), "plain.txt", b"plain\n");
    tracked(repo.as_path(), "bin/tool.sh", b"#!/bin/sh\n");
    std::fs::set_permissions(repo.join("bin/tool.sh"), std::fs::Permissions::from_mode(0o755)).expect("executable");
    tracked(repo.as_path(), "empty.txt", b"");
    std::os::unix::fs::symlink("plain.txt", repo.join("link")).expect("link");
    crate::test_support::age(repo.join("link").as_path());
    commit_all(repo.as_path(), "files");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    return (repo, destination, admin);
}

/// A path that climbs out, is absolute, is empty or enters `.git` is refused; a plain one joins the root.
#[test]
fn only_plain_paths_become_entries() {
    let root: &Path = Path::new("/r");
    assert_eq!(entry_path(root, b"a/b.txt"), Ok(PathBuf::from("/r/a/b.txt")));
    for path in [b"../x".as_slice(), b"/etc/passwd", b"", b".git/config", b"a/../../b", b"a/.git/x"] {
        assert!(entry_path(root, path).is_err(), "{}", String::from_utf8_lossy(path));
    }
}

/// Files are cloned with the source's bytes, mode and modification time, the link has the same target, and the installed
///  index makes the new worktree clean at once.
#[test]
fn btrfs_files_links_modes_and_times_are_cloned_and_the_index_is_installed() {
    let Some(root) = btrfs_fixture("clone-files") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    clone_tracked_files(&source, destination.as_path(), admin.as_path()).expect("clones");
    assert_eq!(std::fs::read(destination.join("plain.txt")).expect("plain"), b"plain\n");
    assert_eq!(std::fs::read_link(destination.join("link")).expect("link"), PathBuf::from("plain.txt"));
    for name in ["plain.txt", "bin/tool.sh", "empty.txt"] {
        let (from, to) = (std::fs::metadata(repo.join(name)).expect("source"), std::fs::metadata(destination.join(name)).expect("clone"));
        assert_eq!((to.mode() & 0o7777, to.mtime(), to.mtime_nsec(), to.len()), (from.mode() & 0o7777, from.mtime(), from.mtime_nsec(), from.len()), "{name}");
    }
    let status: String = git_text(destination.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(status, "", "the first status of the clone is clean");
    remove(root.as_path());
}

/// A source that changed after the check is refused by the comparison with its own index, and a partial destination is
///  cleared down to Git's pointer.
#[test]
fn btrfs_a_changed_source_is_refused_and_a_partial_destination_is_cleared() {
    let Some(root) = btrfs_fixture("clone-changed") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    std::fs::write(repo.join("plain.txt"), b"PLAIN\n").expect("edit after the check");
    crate::test_support::age(repo.join("plain.txt").as_path());
    let reason: String = clone_tracked_files(&source, destination.as_path(), admin.as_path()).expect_err("refused");
    assert!(reason.contains("the source changed while it was cloned"), "{reason}");
    assert!(!admin.join("index").exists(), "no index for a refused clone");
    assert!(destination.join("plain.txt").exists(), "the partial clones are there");
    clear_destination(destination.as_path()).expect("clear");
    let left: Vec<String> = std::fs::read_dir(destination.as_path()).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(left, vec![String::from(".git")]);
    remove(root.as_path());
}

/// A tracked file the source lost ends the rung with the file's name.
#[test]
fn btrfs_a_missing_source_file_ends_the_rung() {
    let Some(root) = btrfs_fixture("clone-missing") else { return };
    let (repo, destination, admin) = clone_fixture(root.as_path());
    let source = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    std::fs::remove_file(repo.join("plain.txt")).expect("remove");
    let reason: String = clone_tracked_files(&source, destination.as_path(), admin.as_path()).expect_err("refused");
    assert!(reason.contains("plain.txt"), "{reason}");
    remove(root.as_path());
}
