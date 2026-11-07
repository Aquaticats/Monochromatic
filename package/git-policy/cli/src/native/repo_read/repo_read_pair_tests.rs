//! Differential controls for the tree-to-tree listings the commit service and the post-commit gate ask Git for:
//!  `diff-tree` names between two commits or between a commit and its parent.
//!  Each answered command line prints what real Git prints in the same repository,
//! and every state Git words
//!  differently (a root commit,
//! a merge commit,
//! a file that became a directory,
//! a gitlink) is declined.

use super::tests::{answered, declined, text};
use crate::test_support::{fixture, git, remove};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

/// A history of three linear commits over varied entries (nested directories,
/// a space,
/// a non-UTF-8 name,
/// an executable,
/// a
/// symbolic link),
/// a root commit,
/// and a merge commit on top;
/// returns the repository and the commit names,
/// oldest first.
fn history(root: &Path, name: &str, format: &str) -> (PathBuf, Vec<String>) {
    let repo = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--object-format={format}")]);
    std::fs::create_dir_all(repo.join("dir/sub")).expect("dirs");
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    std::fs::write(repo.join("exec.sh"), "#!/bin/sh\n").expect("exec");
    std::os::unix::fs::symlink("a.txt", repo.join("link")).expect("link");
    std::fs::write(repo.join("dir/b.txt"), "bee\n").expect("b");
    std::fs::write(repo.join("dir/sub/c.txt"), "sea\n").expect("c");
    std::fs::write(repo.join("sp ace.txt"), "space\n").expect("space");
    std::fs::write(repo.join(std::ffi::OsString::from_vec(vec![b'r', 0xff, b'.', b't'])), "raw\n").expect("raw");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("a.txt"), "one\ntwo\n").expect("edit");
    std::fs::remove_file(repo.join("dir/b.txt")).expect("rm b");
    std::fs::write(repo.join("dir/sub/new.txt"), "new\n").expect("new");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    std::fs::set_permissions(repo.join("exec.sh"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
    std::fs::write(repo.join("dir/sub/c.txt"), "sea\nmore\n").expect("c again");
    std::fs::write(repo.join("z-added.txt"), "z\n").expect("z");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=third"]);
    let names: Vec<String> = ["HEAD~2", "HEAD~1", "HEAD"].iter().map(|spec| { return text(repo.as_path(), &["rev-parse", spec]); }).collect();
    return (repo, names);
}

/// Every pair of commits of the history,
/// in both directions,
/// equals Git,
/// with and without the rename switch the service
/// spells in one of its two questions.
#[test]
fn name_lists_between_two_commits_equal_git() {
    let root = fixture("repo-read-pair-names");
    for format in ["sha1", "sha256"] {
        let (repo, commits) = history(root.as_path(), format, format);
        for old in &commits {
            for new in &commits {
                answered(root.as_path(), repo.as_path(), &["diff-tree", "-r", "--name-only", "-z", "--no-renames", old, new]);
                answered(root.as_path(), repo.as_path(), &["diff-tree", "-r", "--name-only", "-z", old, new]);
            }
        }
        let tree = text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
        answered(root.as_path(), repo.as_path(), &["diff-tree", "-r", "--name-only", "-z", "--no-renames", &commits[0], &tree]);
    }
    remove(root.as_path());
}

/// The one-commit spelling of the post-commit gate and the review lists the paths against the first parent.
#[test]
fn name_lists_of_one_commit_equal_git() {
    let root = fixture("repo-read-pair-one");
    for format in ["sha1", "sha256"] {
        let (repo, commits) = history(root.as_path(), format, format);
        for commit in &commits[1..] {
            answered(root.as_path(), repo.as_path(), &["diff-tree", "--no-commit-id", "--name-only", "-r", "-z", commit]);
        }
    }
    remove(root.as_path());
}

/// A root commit,
/// a merge commit,
/// a file that became a directory and a gitlink are left to Git.
#[test]
fn states_that_need_git_are_declined() {
    let root = fixture("repo-read-pair-declined");
    let (repo, commits) = history(root.as_path(), "plain", "sha1");
    // The first commit has no parent, and a one-commit listing of it is empty in Git without `--root`.
    declined(root.as_path(), repo.as_path(), &["diff-tree", "--no-commit-id", "--name-only", "-r", "-z", &commits[0]]);
    // A merge commit.
    git(repo.as_path(), &["switch", "--quiet", "--create", "side", &commits[0]]);
    std::fs::write(repo.join("side.txt"), "side\n").expect("side");
    git(repo.as_path(), &["add", "side.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=side"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    git(repo.as_path(), &["merge", "--quiet", "--no-ff", "--message=merged", "side"]);
    let merge = text(repo.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), repo.as_path(), &["diff-tree", "--no-commit-id", "--name-only", "-r", "-z", &merge]);
    // A file that became a directory.
    std::fs::remove_file(repo.join("a.txt")).expect("rm");
    std::fs::create_dir(repo.join("a.txt")).expect("dir");
    std::fs::write(repo.join("a.txt/inside"), "x\n").expect("inside");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=dir"]);
    let directory = text(repo.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), repo.as_path(), &["diff-tree", "-r", "--name-only", "-z", "--no-renames", &merge, &directory]);
    // A gitlink.
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", &format!("160000,{directory},module")]);
    git(repo.as_path(), &["commit", "--quiet", "--message=gitlink"]);
    let linked = text(repo.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), repo.as_path(), &["diff-tree", "-r", "--name-only", "-z", "--no-renames", &directory, &linked]);
    // Options the readers do not model.
    declined(root.as_path(), repo.as_path(), &["diff-tree", "-r", "--name-only", "-z", "-M", &commits[0], &commits[1]]);
    // A recursive listing reads the whole tree and touches no index, so Git keeps it.
    declined(root.as_path(), repo.as_path(), &["ls-tree", "-r", "-z", "--full-tree", &commits[1]]);
    remove(root.as_path());
}
