//! Controls for the landing's follow-up with real Git: the index entries of a commit Git built, whatever its names and
//!  however many, and the conclusion state removal.

use super::*;
use crate::recording_git::{recorded_starts, recording_context};
use crate::repo_read_hook::SWITCH_VARIABLE;
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// A repository whose index is at the first commit while the branch is at a second commit that adds the given files.
fn index_behind(name: &str, files: &[Vec<u8>]) -> (PathBuf, PathBuf, String, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    for file in files {
        let path: PathBuf = repo.join(std::ffi::OsStr::from_bytes(file.as_slice()));
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
        std::fs::write(path, b"x\n").expect("write");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let second: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["read-tree", first.as_str()]);
    return (root, repo, first, second);
}


/// What the index still differs from the commit by.
fn differs(repo: &Path, commit: &str) -> bool {
    return !git_output(repo, &["diff", "--cached", "--quiet", commit]).status.success();
}

/// The paths that differ between the old branch and the commit are updated, as bytes, whatever their number.
#[test]
fn derived_paths_cover_names_that_are_not_text_and_lists_past_the_request_limit() {
    let mut files: Vec<Vec<u8>> = vec![b"not-utf8-\xff\xfe.txt".to_vec(), b"back\\slash \"quoted\".txt".to_vec(), b"line\nbreak.txt".to_vec()];
    files.extend((0..crate::service_protocol::MAX_CHANGES + 50).map(|number| { return format!("many/file-{number}.txt").into_bytes(); }));
    let (root, repo, first, second) = index_behind("service-follow-derived", files.as_slice());
    assert!(differs(repo.as_path(), second.as_str()), "the index starts behind the commit");
    update_index(&git_context(), repo.as_path(), second.as_str(), IndexPaths::Derived { old: first.as_str() }).expect("updated");
    assert!(!differs(repo.as_path(), second.as_str()));
    remove(root.as_path());
}

/// A branch that did not exist has an all-zero old value, and the commit's whole tree is what the index takes.
#[test]
fn a_root_commit_is_updated_from_the_all_zero_old_value() {
    let (root, repo, first, _second) = index_behind("service-follow-root", &[b"new.txt".to_vec()]);
    git(repo.as_path(), &["read-tree", "--empty"]);
    let zero: String = "0".repeat(first.len());
    update_index(&git_context(), repo.as_path(), first.as_str(), IndexPaths::Derived { old: zero.as_str() }).expect("updated");
    assert!(!differs(repo.as_path(), first.as_str()));
    remove(root.as_path());
}

/// An empty list changes nothing, because `reset` with no path would reset the whole index.
#[test]
fn an_empty_list_leaves_the_index_alone() {
    let (root, repo, first, second) = index_behind("service-follow-empty", &[b"new.txt".to_vec()]);
    update_index(&git_context(), repo.as_path(), second.as_str(), IndexPaths::Listed(&[])).expect("nothing to do");
    assert!(differs(repo.as_path(), second.as_str()));
    update_index(&git_context(), repo.as_path(), second.as_str(), IndexPaths::Derived { old: second.as_str() }).expect("no difference");
    assert!(differs(repo.as_path(), second.as_str()), "{first}");
    remove(root.as_path());
}

/// A removal request whose scratch directory is gone has nothing left to compare and is done.
#[test]
fn a_cleanup_without_its_scratch_directory_is_done() {
    let root: PathBuf = fixture("service-follow-gone");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let cleanup: Cleanup = Cleanup { git_dir: repo.join(".git"), scratch: repo.join(".git").join("gone"), reftable: false };
    assert!(remove_conclusion_state(&git_context(), repo.as_path(), &cleanup));
    remove(root.as_path());
}

/// What:
///  Every start of the index update, in each form it takes, as the service makes it.
/// Why:
///   The owner's rule is that no Git start of the service gets `--quiet` or `-q`: the three forms of the update (a listed
///  path, a named path as bytes, a path derived from the old branch) each start `reset`, and each is looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const paths of [listed, named, derived]) await updateIndex(recordingGit, repo, second, paths); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_index_update_starts_reset_without_quiet_in_every_form() {
    let (root, repo, first, second) = index_behind("service-follow-no-quiet", &[b"new.txt".to_vec()]);
    let (mut context, log) = crate::recording_git::recording_context(root.as_path());
    // The named form hands its paths on standard input, which a request makes in the common directory.
    context.common_dir = Some(repo.join(".git"));
    let listed: Vec<String> = vec![String::from("new.txt")];
    let named: Vec<Vec<u8>> = vec![b"new.txt".to_vec()];
    for paths in [IndexPaths::Listed(listed.as_slice()), IndexPaths::Named(named.as_slice()), IndexPaths::Derived { old: first.as_str() }] {
        git(repo.as_path(), &["read-tree", first.as_str()]);
        update_index(&context, repo.as_path(), second.as_str(), paths).expect("updated");
        assert!(!differs(repo.as_path(), second.as_str()), "{paths:?} brings the index to the commit");
    }
    let resets: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.starts_with("reset "); }).collect();
    assert_eq!(resets.len(), 3, "one reset for each of the three forms: {resets:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(root.as_path());
}

/// What `reset` says while the index is updated (the list of files still changed in the worktree) is kept in the service's log.
#[test]
fn what_reset_says_is_kept_in_the_journal() {
    let (root, repo, _first, second) = index_behind("service-follow-words", &[b"new.txt".to_vec()]);
    let shim: PathBuf = root.join("saying-git");
    let words: &str = "Unstaged changes after reset:";
    let script: String = format!("#!/bin/sh\ncase \" $* \" in *\" reset \"*) printf '%s\\n' '{words}';; esac\nexec {} \"$@\"\n", crate::test_support::REAL_GIT);
    crate::test_support::executable(shim.as_path(), script.as_bytes());
    let mut context: GitContext = git_context();
    context.real_git = shim;
    let listed: Vec<String> = vec![String::from("new.txt")];
    let entries = crate::service_timing::capture::entries_of(|| {
        update_index(&context, repo.as_path(), second.as_str(), IndexPaths::Listed(listed.as_slice())).expect("updated");
    });
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert!(entries[0].message.contains(words), "{:?}", entries[0]);
    remove(root.as_path());
}

/// The landing's index update starts no Git `reset` and no Git `diff-tree`, whichever way the paths are given, and the index it
/// builds equals the commit for those paths.
#[test]
fn the_follow_up_starts_no_git_reset_and_no_git_diff_tree() {
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec(), b"dir/inner.txt".to_vec()];
    let listed: Vec<String> = vec![String::from("top.txt"), String::from("dir/inner.txt")];
    for form in ["listed", "named", "derived"] {
        let (root, repo, first, second) = index_behind(format!("service-follow-starts-{form}").as_str(), files.as_slice());
        let (mut context, log) = recording_context(root.as_path());
        // The recorder switches the in-process readers off; the update under test must run with them on.
        context.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
        let paths: IndexPaths<'_> = match form {
            "listed" => IndexPaths::Listed(listed.as_slice()),
            "named" => IndexPaths::Named(files.as_slice()),
            _ => IndexPaths::Derived { old: first.as_str() },
        };
        let index: PathBuf = repo.join(".git").join("index");
        // The service builds the index while it holds the live index lock.
        let held = crate::service_index_lock::try_acquire(index.as_path(), 1).expect("the lock is free");
        let built: PathBuf = build_index(&context, repo.as_path(), second.as_str(), paths, index.as_path(), 1).expect("built");
        let words: Vec<String> = recorded_starts(log.as_path()).iter().map(|start| { return start.split(' ').next().unwrap_or_default().to_string(); }).collect();
        assert!(!words.iter().any(|word| { return word == "reset" || word == "diff-tree"; }), "{form} started {words:?}");
        held.release_with(built.as_path()).expect("installed");
        assert!(!differs(repo.as_path(), second.as_str()), "{form} left the index behind the commit");
        remove(root.as_path());
    }
}

/// The in-process update runs only for a landing that holds the live index lock, and the lock is held from the copy of the live
/// index to its replacement: without the lock the follow-up asks Git (which takes the lock itself or fails), with it no Git
/// starts, a foreign `git add` finds the lock taken until the built index is in place, and what it staged before is kept.
#[test]
fn the_in_process_update_runs_only_under_the_live_index_lock() {
    use crate::service_index_lock::{lock_path_of, try_acquire};
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec(), b"dir/inner.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-lock", files.as_slice());
    let (mut context, log) = recording_context(root.as_path());
    context.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
    context.common_dir = Some(repo.join(".git"));
    let index: PathBuf = repo.join(".git").join("index");
    let paths: Vec<Vec<u8>> = files.clone();
    // Without the lock Git's own command runs.
    let unlocked: PathBuf = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&paths), index.as_path(), 1).expect("built");
    let words: Vec<String> = recorded_starts(log.as_path()).iter().map(|start| { return start.split(' ').next().unwrap_or_default().to_string(); }).collect();
    assert!(words.iter().any(|word| { return word == "reset"; }), "an unlocked update must ask Git, saw {words:?}");
    std::fs::remove_file(unlocked).expect("removed");
    // With the lock no Git starts, and the lock stays until the built index is installed.
    let before: usize = recorded_starts(log.as_path()).len();
    let held = try_acquire(index.as_path(), 2).expect("the lock is free");
    assert!(lock_path_of(index.as_path()).exists());
    let staged_elsewhere: std::process::Output = git_output(repo.as_path(), &["add", "--all"]);
    assert!(!staged_elsewhere.status.success(), "a foreign git add must find the lock taken");
    let built: PathBuf = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&paths), index.as_path(), 2).expect("built");
    assert_eq!(recorded_starts(log.as_path()).len(), before, "no Git started under the lock");
    assert!(lock_path_of(index.as_path()).exists(), "the lock is still held after the build");
    held.release_with(built.as_path()).expect("installed");
    assert!(!lock_path_of(index.as_path()).exists());
    assert!(!differs(repo.as_path(), second.as_str()));
    remove(root.as_path());
}
