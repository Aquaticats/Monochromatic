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
        let built: PathBuf = build_index(&context, repo.as_path(), second.as_str(), paths, &held, 1).expect("built");
        let words: Vec<String> = recorded_starts(log.as_path()).iter().map(|start| { return start.split(' ').next().unwrap_or_default().to_string(); }).collect();
        assert!(!words.iter().any(|word| { return word == "reset" || word == "diff-tree"; }), "{form} started {words:?}");
        held.release_with(built.as_path()).expect("installed");
        assert!(!differs(repo.as_path(), second.as_str()), "{form} left the index behind the commit");
        remove(root.as_path());
    }
}


/// The names Git lists as staged against `commit`.
fn staged_against(repo: &Path, commit: &str) -> Vec<String> {
    return git_text(repo, &["diff", "--cached", "--name-only", commit]).lines().map(str::to_string).collect();
}

/// A change another session staged before the lock was taken, to a path the commit does not name, survives the in-process
/// build and its installation; a stage attempted after the lock was taken is refused until the installation; and no Git starts.
#[test]
fn what_was_staged_before_the_lock_survives_and_a_stage_after_it_is_refused() {
    use crate::service_index_lock::{lock_path_of, try_acquire};
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec(), b"dir/inner.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-staged", files.as_slice());
    std::fs::write(repo.join("unselected.txt"), b"staged by another session\n").expect("write");
    // An entry recorded in the second of the lock's copy is racily clean and sends the update to Git; this one is older.
    crate::test_support::age(repo.join("unselected.txt").as_path());
    git(repo.as_path(), &["add", "unselected.txt"]);
    let (mut context, log) = recording_context(root.as_path());
    context.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
    context.common_dir = Some(repo.join(".git"));
    let index: PathBuf = repo.join(".git").join("index");
    let held = try_acquire(index.as_path(), 3).expect("the lock is free");
    std::fs::write(repo.join("late.txt"), b"staged too late\n").expect("write");
    assert!(!git_output(repo.as_path(), &["add", "late.txt"]).status.success(), "a stage after the lock is taken is refused");
    let built: PathBuf = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 3).expect("built");
    assert!(lock_path_of(index.as_path()).exists(), "the lock is held until the installation");
    held.release_with(built.as_path()).expect("installed");
    let words: Vec<String> = recorded_starts(log.as_path()).iter().map(|start| { return start.split(' ').next().unwrap_or_default().to_string(); }).collect();
    assert!(!words.iter().any(|word| { return word == "reset"; }), "no Git reset started: {words:?}");
    assert_eq!(staged_against(repo.as_path(), second.as_str()), vec![String::from("unselected.txt")], "only the earlier stage is left, and it survived");
    assert!(git_output(repo.as_path(), &["add", "late.txt"]).status.success(), "the lock is gone, so the stage works now");
    remove(root.as_path());
}

/// An installation that cannot be made leaves the old index and removes the lock; a lock another program replaced keeps the
/// index and stays with that program.
#[test]
fn a_failed_installation_keeps_the_index_and_a_replaced_lock_is_left_alone() {
    use crate::service_index_lock::{lock_path_of, try_acquire};
    let files: Vec<Vec<u8>> = vec![b"top.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("service-follow-install", files.as_slice());
    let mut context = git_context();
    context.common_dir = Some(repo.join(".git"));
    let index: PathBuf = repo.join(".git").join("index");
    let before: Vec<u8> = std::fs::read(&index).expect("index");
    // A lock replaced by another program's: the check sees it, the index is not replaced, the other program's lock stays.
    let held = try_acquire(index.as_path(), 4).expect("the lock is free");
    let built: PathBuf = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 4).expect("built");
    std::fs::remove_file(lock_path_of(index.as_path())).expect("removed");
    std::fs::write(lock_path_of(index.as_path()), b"another program's lock").expect("foreign lock");
    assert!(held.release_with(built.as_path()).is_err());
    assert_eq!(std::fs::read(&index).expect("index"), before, "the index was not replaced");
    assert_eq!(std::fs::read(lock_path_of(index.as_path())).expect("lock"), b"another program's lock".to_vec());
    std::fs::remove_file(lock_path_of(index.as_path())).expect("removed");
    std::fs::remove_file(&built).expect("built removed");
    // A rename that fails: the live index path holds a directory, so the copy cannot take its place; the lock goes with the error.
    let held = try_acquire(index.as_path(), 5).expect("the lock is free");
    let built: PathBuf = build_index(&context, repo.as_path(), second.as_str(), IndexPaths::Named(&files), &held, 5).expect("built");
    std::fs::remove_file(&index).expect("index removed");
    std::fs::create_dir_all(index.join("occupied")).expect("a directory where the index goes");
    assert!(held.release_with(built.as_path()).is_err());
    assert!(!lock_path_of(index.as_path()).exists(), "the lock is removed with the failure");
    assert!(index.join("occupied").is_dir());
    remove(root.as_path());
}
