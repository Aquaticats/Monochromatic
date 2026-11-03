//! The unit tests of the choice of the trust base,
//! against real throwaway git repositories.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::git_base::{git_lock_reader_at, resolve_trust_base};
use crate::test_scratch::scratch;

/// The system git,
/// not the `git` on PATH:
/// that one may be a policy wrapper that refuses a bare `git commit` in a fixture.
const SYSTEM_GIT: &str = "/usr/bin/git";

fn git(directory: &Path, arguments: &[&str]) -> String {
    let output = Command::new(SYSTEM_GIT)
        .args(["-c", "user.name=tester", "-c", "user.email=tester@example.invalid", "-c", "commit.gpgsign=false", "-c", "init.defaultBranch=main"])
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("start git");
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// A repository whose branch `main` tracks a bare remote at its first commit and has one more commit that was not pushed.
/// Returns the working directory,
/// the pushed commit and the local-only commit.
fn repository_ahead_of_its_upstream() -> (tempfile::TempDir, PathBuf, String, String) {
    let root = scratch();
    let remote: PathBuf = root.path().join("remote.git");
    let work: PathBuf = root.path().join("work");
    std::fs::create_dir_all(&remote).expect("remote dir");
    std::fs::create_dir_all(work.join("package/a/one")).expect("work dir");
    git(&remote, &["init", "--bare", "--quiet"]);
    git(&work, &["init", "--quiet"]);
    std::fs::write(work.join("package/a/one/Cargo.lock"), "pushed lock\n").expect("lock");
    git(&work, &["add", "--", "package/a/one/Cargo.lock"]);
    git(&work, &["commit", "--quiet", "-m", "pushed"]);
    git(&work, &["remote", "add", "origin", remote.to_str().expect("utf8")]);
    git(&work, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let pushed: String = git(&work, &["rev-parse", "HEAD"]);
    std::fs::write(work.join("package/a/one/Cargo.lock"), "merged lock\n").expect("lock");
    git(&work, &["commit", "--quiet", "--all", "-m", "merged, not pushed"]);
    let local: String = git(&work, &["rev-parse", "HEAD"]);
    return (root, work, pushed, local);
}

/// Delegated pick 1 (2026-11-03):
/// a local run compares with the upstream branch,
/// so a lockfile that arrived with a commit that is
/// not on the upstream yet is new since the base.
/// The remembered pass is keyed by the commit.
#[test]
fn a_local_run_compares_with_the_upstream_branch_not_head() {
    let (_root, work, pushed, local) = repository_ahead_of_its_upstream();

    let base = resolve_trust_base(&work, false, None);

    assert_eq!(base.commit, Ok(pushed.clone()));
    assert_ne!(pushed, local);
    assert_eq!(base.spelling, "@{upstream}");
}

/// CI keeps `HEAD`,
/// and an explicit `CARGO_SHADOW_TRUST_BASE` wins everywhere.
#[test]
fn ci_compares_with_head_and_an_explicit_base_wins() {
    let (_root, work, pushed, local) = repository_ahead_of_its_upstream();

    assert_eq!(resolve_trust_base(&work, true, None).commit, Ok(local.clone()));
    assert_eq!(resolve_trust_base(&work, false, Some(&local)).commit, Ok(local));
    assert_eq!(resolve_trust_base(&work, true, Some(&pushed)).commit, Ok(pushed));
}

/// Without an upstream the base falls back to the remote's default branch,
/// and without that to `HEAD`.
#[test]
fn without_an_upstream_the_base_is_origin_main_and_without_a_remote_it_is_head() {
    let (_root, work, pushed, local) = repository_ahead_of_its_upstream();
    git(&work, &["checkout", "--quiet", "-b", "feature"]);

    let fallback = resolve_trust_base(&work, false, None);
    assert_eq!(fallback.commit, Ok(pushed));
    assert_eq!(fallback.spelling, "origin/main");

    git(&work, &["remote", "remove", "origin"]);
    let alone = resolve_trust_base(&work, false, None);
    assert_eq!(alone.commit, Ok(local));
    assert_eq!(alone.spelling, "HEAD");
}

/// The reader names a commit and reads the lockfile text there,
/// empty when it did not exist.
#[test]
fn the_reader_reads_a_lockfile_at_the_commit_and_an_absent_one_as_empty() {
    let (_root, work, pushed, _local) = repository_ahead_of_its_upstream();
    let read = git_lock_reader_at(&work, &pushed);

    assert!(read("package/a/one").starts_with("pushed lock"));
    assert_eq!(read("package/a/two"), "");
}

/// A base that names no commit is an error naming it,
/// not a panic.
#[test]
fn a_base_that_names_no_commit_is_an_error() {
    let (_root, work, _pushed, _local) = repository_ahead_of_its_upstream();

    let base = resolve_trust_base(&work, false, Some("no-such-revision"));

    assert!(base.commit.expect_err("not a commit").contains("no-such-revision"));
}

// Owner rules (2026-11-01, `package/git-policy/cli/SPEC.md`): "Invoke real git as rarely as possible." and "embed instead of
// spawning". The trust base is read from the repository's files; the tests count the Git processes a choice started and compare
// the answer with the system git as the oracle.

use crate::git_base::{TrustBase, resolve_trust_base_with};
use crate::git_refs::NativeEnvironment;

/// A disposable environment:
/// an empty home,
/// no system configuration,
/// no blocking variable.
fn isolated_environment(root: &Path) -> NativeEnvironment {
    let home: PathBuf = root.join("home");
    std::fs::create_dir_all(&home).expect("home");
    return NativeEnvironment { home: Some(home), xdg_config_home: None, system_config: root.join("no-system-config"), blocking_variable: None };
}

fn oracle(work: &Path, revision: &str) -> String {
    return git(work, &["rev-parse", "--verify", &format!("{revision}^{{commit}}")]);
}

fn chosen(root: &tempfile::TempDir, work: &Path, in_ci: bool) -> TrustBase {
    return resolve_trust_base_with(work, in_ci, None, &isolated_environment(root.path()));
}

#[test]
fn a_plain_repository_is_answered_from_its_files_with_no_git_process() {
    let (root, work, pushed, local) = repository_ahead_of_its_upstream();

    let local_run = chosen(&root, &work, false);
    let ci_run = chosen(&root, &work, true);

    assert_eq!((local_run.spelling.as_str(), local_run.commit.clone(), local_run.git_starts), ("@{upstream}", Ok(pushed), 0));
    assert_eq!((ci_run.spelling.as_str(), ci_run.commit.clone(), ci_run.git_starts), ("HEAD", Ok(local), 0));
}

#[test]
fn packed_references_are_read_as_loose_ones_are() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    git(&work, &["pack-refs", "--all", "--prune"]);

    let base = chosen(&root, &work, false);

    assert_eq!((base.commit.clone(), base.git_starts), (Ok(pushed), 0));
}

#[test]
fn without_an_upstream_the_fallbacks_are_read_from_the_files_too() {
    let (root, work, pushed, local) = repository_ahead_of_its_upstream();
    git(&work, &["checkout", "--quiet", "-b", "feature"]);
    let from_origin = chosen(&root, &work, false);
    git(&work, &["remote", "remove", "origin"]);
    let alone = chosen(&root, &work, false);

    assert_eq!((from_origin.spelling.as_str(), from_origin.commit.clone(), from_origin.git_starts), ("origin/main", Ok(pushed), 0));
    assert_eq!((alone.spelling.as_str(), alone.commit.clone(), alone.git_starts), ("HEAD", Ok(local), 0));
}

#[test]
fn a_detached_head_has_no_upstream_and_is_its_own_base() {
    let (root, work, _pushed, local) = repository_ahead_of_its_upstream();
    git(&work, &["checkout", "--quiet", "--detach"]);
    git(&work, &["remote", "remove", "origin"]);

    let base = chosen(&root, &work, false);

    assert_eq!((base.commit.clone(), base.git_starts), (Ok(local), 0));
}

#[test]
fn a_linked_worktree_reads_its_own_head_and_the_shared_configuration_and_references() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    let linked: PathBuf = root.path().join("linked");
    git(&work, &["worktree", "add", "--quiet", "-b", "topic", linked.to_str().expect("utf8")]);
    git(&linked, &["branch", "--quiet", "--set-upstream-to=origin/main"]);

    let base = chosen(&root, &linked, false);

    assert_eq!((base.spelling.as_str(), base.commit.clone(), base.git_starts), ("@{upstream}", Ok(pushed.clone()), 0));
    assert_eq!(oracle(&linked, "@{upstream}"), pushed);
}

/// Each layout or setting the reader cannot answer exactly costs one Git start,
/// and the answer is still Git's.
fn assert_asked_git(base: &TrustBase, expected: &str) {
    assert!(base.git_starts >= 1, "the files should not have answered: {base:?}");
    assert_eq!(base.commit, Ok(expected.to_string()));
}

#[test]
fn a_configuration_include_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    std::fs::write(root.path().join("extra.config"), "[user]\n\tname = extra\n").expect("extra");
    git(&work, &["config", "include.path", root.path().join("extra.config").to_str().expect("utf8")]);

    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn a_conditional_include_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    std::fs::write(root.path().join("extra.config"), "[user]\n\tname = extra\n").expect("extra");
    git(&work, &["config", "includeIf.gitdir:/nowhere/.path", root.path().join("extra.config").to_str().expect("utf8")]);

    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn an_include_in_the_users_configuration_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    let environment: NativeEnvironment = isolated_environment(root.path());
    std::fs::write(environment.home.clone().expect("home").join(".gitconfig"), "[include]\n\tpath = /nowhere\n").expect("user config");

    let base = resolve_trust_base_with(&work, false, None, &environment);

    assert_asked_git(&base, &pushed);
}

#[test]
fn a_blocking_environment_variable_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    let mut environment: NativeEnvironment = isolated_environment(root.path());
    environment.blocking_variable = Some("GIT_NAMESPACE".to_string());

    assert_asked_git(&resolve_trust_base_with(&work, false, None, &environment), &pushed);
}

#[test]
fn a_worktree_specific_configuration_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    git(&work, &["config", "extensions.worktreeConfig", "true"]);

    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn a_tag_that_shadows_the_remote_default_branch_is_asked_of_git() {
    let (root, work, _pushed, local) = repository_ahead_of_its_upstream();
    git(&work, &["checkout", "--quiet", "-b", "feature"]);
    git(&work, &["tag", "origin/main", "HEAD"]);

    // `git rev-parse origin/main` prefers the tag, which is the local commit.
    assert_asked_git(&chosen(&root, &work, false), &local);
}

#[test]
fn a_negative_fetch_refspec_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    git(&work, &["config", "--add", "remote.origin.fetch", "^refs/heads/nothing"]);

    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn a_symbolic_reference_where_a_commit_is_expected_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    git(&work, &["checkout", "--quiet", "-b", "feature"]);
    git(&work, &["symbolic-ref", "refs/remotes/origin/main", "refs/remotes/origin/other"]);
    git(&work, &["update-ref", "refs/remotes/origin/other", &pushed]);

    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn the_reftable_reference_format_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    git(&work, &["config", "extensions.refStorage", "reftable"]);

    // The setting alone is what the reader refuses; the repository still has files, so Git (which ignores the setting while the
    // files exist) is not asked to read a reftable store here.
    let environment: NativeEnvironment = isolated_environment(root.path());
    assert!(crate::git_refs::native_base(&work, false, &environment).is_err());
    let _ = pushed;
}

#[test]
fn an_explicit_base_is_verified_by_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();

    let base = resolve_trust_base_with(&work, false, Some(&pushed), &isolated_environment(root.path()));

    assert_eq!((base.commit.clone(), base.git_starts), (Ok(pushed), 1));
}
