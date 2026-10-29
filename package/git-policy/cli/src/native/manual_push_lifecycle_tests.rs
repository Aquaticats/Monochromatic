//! Controls for the manual-push candidate scope of one push, against real repositories.
//! The TypeScript corpus is `src/policy-engine/manual-push-lifecycle` behavior in
//! `src/policy-engine/manual-push.integration.unit.test.ts` (blocked on new branch, prior destination, probe failure).

use super::*;
use crate::test_support::{fixture, git, remove};
use std::path::{Path, PathBuf};

/// Environment of every Git child of these fixtures.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    return [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ]
    .iter()
    .map(|(name, value)| {
        return (OsString::from(name), OsString::from(value));
    })
    .collect();
}

/// Coordinator Git for a fixture repository.
fn git_for(root: &Path) -> PushGit {
    // The probe's per-user scratch directory is made under a home beside the repository.
    let mut environment = fixture_environment();
    environment.push((OsString::from("HOME"), root.parent().unwrap_or(root).join("home").into_os_string()));
    return PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.to_path_buf(),
        environment,
    };
}

/// A repository with a published `main` and one unpublished commit that adds `new.txt`.
fn repository_with_unpublished_commit(name: &str) -> (PathBuf, String) {
    let root = fixture(name);
    let origin = root.join("origin.git");
    let repo = root.join("repo");
    git(
        &root,
        &["init", "--quiet", "--bare", origin.to_str().expect("utf-8")],
    );
    std::fs::create_dir_all(&repo).expect("repo");
    git(&repo, &["init", "--quiet", "--initial-branch=main"]);
    git(&repo, &["config", "user.email", "cli-git@example.invalid"]);
    git(&repo, &["config", "user.name", "cli-git fixture"]);
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=base"],
    );
    git(
        &repo,
        &["remote", "add", "origin", origin.to_str().expect("utf-8")],
    );
    git(
        &repo,
        &["push", "--quiet", "origin", "HEAD:refs/heads/main"],
    );
    std::fs::write(repo.join("new.txt"), "new content\n").expect("new file");
    git(&repo, &["add", "new.txt"]);
    git(&repo, &["commit", "--quiet", "-m", "new"]);
    let tip = String::from_utf8_lossy(&git(&repo, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
    return (repo, tip);
}

/// Duplicate commits are listed once, and a deletion contributes nothing.
#[test]
fn the_pushed_scope_lists_each_commit_once() {
    let first = parse_object_id("a".repeat(40).as_bytes()).expect("object name");
    let scopes = vec![
        UpdateScope::Commits(vec!["a".repeat(40)]),
        UpdateScope::None,
        UpdateScope::Commits(vec!["a".repeat(40)]),
    ];
    let scope = pushed_scope(&scopes).expect("scope");
    assert_eq!(scope.commits, vec![first]);
    assert!(scope.trees.is_empty());
}

/// A pushed tip with no published base lists its whole tree once and no commit.
#[test]
fn the_pushed_scope_lists_each_final_tree_once() {
    let tree = parse_object_id("b".repeat(40).as_bytes()).expect("object name");
    let scopes = vec![
        UpdateScope::FinalTree("b".repeat(40)),
        UpdateScope::FinalTree("b".repeat(40)),
    ];
    let scope = pushed_scope(&scopes).expect("scope");
    assert_eq!(scope.trees, vec![tree]);
    assert!(scope.commits.is_empty());
}

/// A pushed commit whose base is published scans its own unpublished commits.
#[test]
fn a_push_of_an_unpublished_commit_scans_that_commit() {
    let (repo, tip) = repository_with_unpublished_commit("mp-lifecycle-scope");
    let scope = manual_push_scope(
        &git_for(&repo),
        &[],
        &[OsString::from("origin"), OsString::from("main")],
    )
    .expect("scope");
    let expected = parse_object_id(tip.as_bytes()).expect("object name");
    assert_eq!(scope.commits, vec![expected]);
    remove(repo.parent().expect("root"));
}

/// A destination Git cannot reach fails the scope, so the push is refused rather than scanned empty.
#[test]
fn an_unreachable_destination_fails_the_scope() {
    let (repo, _tip) = repository_with_unpublished_commit("mp-lifecycle-unreachable");
    let missing = repo.parent().expect("root").join("missing.git");
    let result = manual_push_scope(
        &git_for(&repo),
        &[],
        &[
            OsString::from(missing.to_str().expect("utf-8")),
            OsString::from("main"),
        ],
    );
    assert!(result.is_err());
    remove(repo.parent().expect("root"));
}

/// The lifecycle failure prints as its diagnostic, so a refused push names the failed step.
#[test]
fn the_failure_displays_its_message() {
    let failure = ManualPushFailure {
        message: String::from("the step failed"),
    };
    assert_eq!(failure.to_string(), "the step failed");
}

/// A commit that another branch of the destination already holds is published, so the push does not scan it again.
#[test]
fn a_commit_another_branch_of_the_destination_holds_is_not_scanned_again() {
    let (repo, _tip) = repository_with_unpublished_commit("mp-lifecycle-held-elsewhere");
    git(
        &repo,
        &["push", "--quiet", "origin", "HEAD:refs/heads/other"],
    );
    let scope = manual_push_scope(
        &git_for(&repo),
        &[],
        &[OsString::from("origin"), OsString::from("main")],
    )
    .expect("scope");
    assert!(scope.commits.is_empty(), "{:?}", scope.commits);
    assert!(scope.trees.is_empty(), "{:?}", scope.trees);
    remove(repo.parent().expect("root"));
}
