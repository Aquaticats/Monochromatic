//! Controls for the restore of a snapshot of a main checkout with changes:
//! the proof accepts what Git's checkout leaves and
//! names what it does not,
//! the Git runs are exactly the two the design lists,
//! and a restore never runs in the source.

use super::*;
use crate::test_support::{git, git_context, remove, repository};
use crate::worktree_copy::{GitStatus, GitStep};
use crate::worktree_copy_error::WorktreeCopyError;
use std::path::PathBuf;

/// The words every step starts with:
/// `-C` and the stage.
const LEADING_WORDS: usize = 2;

/// A target for the commit `HEAD` names in `repo`,
/// whose Git directory is its own.
fn target_of(repo: &Path) -> Target {
    let commit: String = String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout).trim().to_string();
    return Target { admin: repo.join(".git"), pointer: Vec::new(), commit, differs: false, line: String::new() };
}

/// A repository with one committed file.
fn committed(root: &Path) -> PathBuf {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "restored")).expect("canonical");
    std::fs::write(repo.join("a.txt"), b"a\n").expect("file");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    git(repo.as_path(), &["update-index", "--refresh"]);
    return repo;
}

/// The proof accepts a worktree that equals its commit and names each way it does not.
#[test]
fn the_proof_accepts_what_gits_checkout_leaves_and_names_what_it_does_not() {
    let root: PathBuf = crate::test_support::fixture("restore-proof");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    assert_eq!(prove_restored(&git_context(), &target, repo.as_path()), Ok(()), "positive control");
    std::fs::write(repo.join("untracked.txt"), b"u").expect("untracked");
    let untracked: String = prove_restored(&git_context(), &target, repo.as_path()).expect_err("an untracked file");
    assert!(untracked.contains("untracked.txt") && untracked.contains("still untracked"), "{untracked}");
    std::fs::remove_file(repo.join("untracked.txt")).expect("remove");
    git(repo.as_path(), &["rm", "--cached", "--quiet", "a.txt"]);
    assert!(prove_restored(&git_context(), &target, repo.as_path()).expect_err("a staged deletion").contains("index"));
    git(repo.as_path(), &["add", "a.txt"]);
    std::fs::write(repo.join("a.txt"), b"changed\n").expect("edit");
    assert!(prove_restored(&git_context(), &target, repo.as_path()).expect_err("an edit").contains("differs"));
    remove(root.as_path());
}

/// The restore runs `read-tree --reset -u <commit>` through the lifecycle's runner and then a captured `clean --force -d`,
/// in that
///  order,
/// each in the stage under the new worktree's Git directory;
/// nothing else is run through the runner,
/// and the `clean`
///  carries neither `--quiet` nor `-q`.
#[test]
fn the_restore_runs_read_tree_through_the_runner_and_a_clean_without_quiet_in_the_stage() {
    let root: PathBuf = crate::test_support::fixture("restore-steps");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    let stage: PathBuf = repo.clone();
    let mut seen: Vec<Vec<String>> = Vec::new();
    let mut environments: Vec<Vec<(String, String)>> = Vec::new();
    let mut runner = |step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> {
        let GitStep::Other { arguments, environment } = step else { panic!("only the ladder's own steps are expected") };
        seen.push(arguments.iter().map(|word| { return word.to_string_lossy().into_owned(); }).collect());
        environments.push(environment.iter().map(|(name, value)| { return (name.to_string_lossy().into_owned(), value.to_string_lossy().into_owned()); }).collect());
        return Ok(GitStatus { code: Some(0) });
    };
    let other_root: PathBuf = PathBuf::from("/nonexistent-source-root");
    let source = SnapshotSource {
        root: other_root.clone(),
        git_dir: other_root.join(".git"),
        common_dir: other_root.join(".git"),
        clean: crate::worktree_clean::CleanSource { head_commit: target.commit.clone(), head_tree: Vec::new(), tracked: 0, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let (recording, log) = crate::recording_git::recording_context(root.as_path());
    restore_dirty(&source, &target, stage.as_path(), &mut runner, &recording).expect("the proof holds for a worktree that equals its commit");
    let words = |step: &Vec<String>| -> Vec<String> { return step[LEADING_WORDS..].to_vec(); };
    let expected: Vec<Vec<String>> = vec![["read-tree", "--reset", "-u", target.commit.as_str()].map(String::from).to_vec()];
    assert_eq!(seen.iter().map(words).collect::<Vec<Vec<String>>>(), expected, "exactly this step goes through the runner");
    let cleans: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.split(' ').any(|word| { return word == "clean"; }); }).collect();
    assert_eq!(cleans.len(), 1, "one captured clean: {cleans:?}");
    assert!(cleans[0].ends_with("clean --force -d"), "{cleans:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    for (arguments, environment) in seen.iter().zip(&environments) {
        assert_eq!(&arguments[..LEADING_WORDS], ["-C", stage.to_string_lossy().as_ref()]);
        assert!(environment.contains(&(String::from("GIT_DIR"), target.admin.to_string_lossy().into_owned())), "{environment:?}");
        assert!(environment.contains(&(String::from("GIT_WORK_TREE"), stage.to_string_lossy().into_owned())), "{environment:?}");
    }
    remove(root.as_path());
}

/// A restore that would run in the source's own directory or Git directory runs nothing.
#[test]
fn a_restore_never_runs_in_the_source() {
    let root: PathBuf = crate::test_support::fixture("restore-guard");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    let mut runner = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> { panic!("no Git run is expected") };
    let source = |source_root: PathBuf, git_dir: PathBuf| -> SnapshotSource {
        return SnapshotSource {
            root: source_root,
            git_dir: git_dir.clone(),
            common_dir: git_dir,
            clean: crate::worktree_clean::CleanSource { head_commit: target.commit.clone(), head_tree: Vec::new(), tracked: 0, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
            nested: Vec::new(),
            config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
            trash_ignored: false,
            dirty: None,
        };
    };
    let in_source = restore_dirty(&source(repo.clone(), repo.join(".git")), &target, repo.as_path(), &mut runner, &git_context()).expect_err("the stage is the source");
    assert!(in_source.contains("source's own directory"), "{in_source}");
    let elsewhere: PathBuf = root.join("elsewhere");
    let in_git_dir = restore_dirty(&source(elsewhere.clone(), repo.join(".git")), &target, elsewhere.as_path(), &mut runner, &git_context()).expect_err("the Git directory is the source's");
    assert!(in_git_dir.contains("source's own directory"), "{in_git_dir}");
    remove(root.as_path());
}

/// A failing step or a failing proof is an error,
/// so the rung ends and Git's checkout makes the worktree.
#[test]
fn a_failed_step_or_a_failed_proof_is_an_error() {
    let root: PathBuf = crate::test_support::fixture("restore-failure");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    let other_root: PathBuf = PathBuf::from("/nonexistent-source-root");
    let source = SnapshotSource {
        root: other_root.clone(),
        git_dir: other_root.join(".git"),
        common_dir: other_root.join(".git"),
        clean: crate::worktree_clean::CleanSource { head_commit: target.commit.clone(), head_tree: Vec::new(), tracked: 0, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let mut failing = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> { return Ok(GitStatus { code: Some(128) }); };
    assert!(restore_dirty(&source, &target, repo.as_path(), &mut failing, &git_context()).expect_err("the reset failed").contains("read-tree"));
    let mut succeeding = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> { return Ok(GitStatus { code: Some(0) }); };
    std::fs::write(repo.join("left.txt"), b"left").expect("a file the clean left behind");
    // A Git whose `clean` does nothing, so the file stays and the proof is what finds it.
    let idle_clean: PathBuf = root.join("idle-clean-git");
    let script: String = format!("#!/bin/sh\ncase \" $* \" in *\" clean \"*) exit 0;; esac\nexec {} \"$@\"\n", crate::test_support::REAL_GIT);
    crate::test_support::executable(idle_clean.as_path(), script.as_bytes());
    let mut leaving: GitContext = git_context();
    leaving.real_git = idle_clean;
    assert!(restore_dirty(&source, &target, repo.as_path(), &mut succeeding, &leaving).expect_err("the proof fails").contains("left.txt"));
    remove(root.as_path());
}

/// The note counts the paths in the singular and the plural.
#[test]
fn the_note_counts_paths() {
    let one = ChangedPaths { count: 1, first: Some(String::from("a")), attributes_changed: false };
    let many = ChangedPaths { count: 3, first: Some(String::from("a")), attributes_changed: false };
    assert!(left_out_note(&one).contains(format!("({} path)", one.count).as_str()), "{}", left_out_note(&one));
    assert!(left_out_note(&many).contains(format!("({} paths)", many.count).as_str()), "{}", left_out_note(&many));
    assert!(left_out_note(&one).ends_with('\n'));
}
