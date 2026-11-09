//! Controls for the manual-push candidate scope of one push,
//! against real repositories.
//! The corpus is the `manual-push-lifecycle` behavior of
//! `package/git-policy/cli/src/manual-push.integration.unit.test.ts` (blocked on new branch,
//! prior destination,
//! probe failure),
//! deleted on 2026-10-13 in `642b6d72cb` with the TypeScript wrapper,
//! so `git show 642b6d72cb^` holds it.

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
        lfs: false,
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
    git(&repo, &["config", "user.name", "monochromatic-git-policy-cli fixture"]);
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

/// Duplicate commits are listed once,
/// and a deletion contributes nothing.
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

/// A destination Git cannot reach fails the scope,
/// so the push is refused rather than scanned empty.
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

/// The lifecycle failure prints as its diagnostic,
/// so a refused push names the failed step.
#[test]
fn the_failure_displays_its_message() {
    let failure = ManualPushFailure::step(ManualPushCause::SelfExplained, String::from("the step failed"));
    assert_eq!(failure.to_string(), "the step failed");
}

/// A commit that another branch of the destination already holds is published,
/// so the push does not scan it again.
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

/// What:
///  Collecting a push's scope looks at each named commit and tip once.
/// Why:
///  `AGENTS.md` `RG2`:
/// a push can publish thousands of commits,
///  and asking every earlier collected name whether the candidate repeats itself
///  is quadratic in them.
///  The seam counts the collected names a question looks at,
///  which is one per name when a set answers and every earlier name when a scan does
///  (`foundation/collected_elements_looked_at_tests.rs` proves the count separates the two).
#[test]
fn collecting_a_pushed_scope_looks_at_each_name_once() {
    let width: usize = 512;
    let mut scopes: Vec<UpdateScope> = Vec::new();
    // Every name twice, in two updates of the same push: the second update publishes what the first did.
    for _repeat in 0..2 {
        let mut commits: Vec<String> = Vec::new();
        let mut trees: Vec<String> = Vec::new();
        for number in 0..width {
            commits.push(format!("{number:040x}"));
            trees.push(format!("{:040x}", width + number));
        }
        scopes.push(UpdateScope::Commits(commits));
        scopes.push(UpdateScope::FinalTree(format!("{:040x}", 2 * width)));
        scopes.push(UpdateScope::Commits(trees));
    }
    let names: usize = 4 * width + 2;
    let before: u64 = crate::collected_elements_looked_at::looked_at();
    let scope = pushed_scope(&scopes).expect("scope");
    let looked: u64 = crate::collected_elements_looked_at::looked_at() - before;
    assert_eq!(scope.commits.len(), 2 * width, "every commit once, in first-seen order");
    assert_eq!(scope.trees, vec![parse_object_id(format!("{:040x}", 2 * width).as_bytes()).expect("object name")]);
    assert_eq!(
        looked,
        names as u64,
        "{looked} collected names looked at for {names} named commits and tips; one per name is linear"
    );
}
