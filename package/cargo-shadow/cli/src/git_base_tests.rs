//! The unit tests of the choice of the trust base, against real throwaway git repositories.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::git_base::{git_lock_reader_at, resolve_trust_base};
use crate::test_scratch::scratch;

/// The system git, not the `git` on PATH: that one may be a policy wrapper that refuses a bare `git commit` in a fixture.
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
/// Returns the working directory, the pushed commit and the local-only commit.
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

/// Delegated pick 1 (2026-11-03): a local run compares with the upstream branch, so a lockfile that arrived with a commit that is
/// not on the upstream yet is new since the base. The remembered pass is keyed by the commit.
#[test]
fn a_local_run_compares_with_the_upstream_branch_not_head() {
    let (_root, work, pushed, local) = repository_ahead_of_its_upstream();

    let base = resolve_trust_base(&work, false, None);

    assert_eq!(base.commit, Ok(pushed.clone()));
    assert_ne!(pushed, local);
    assert_eq!(base.spelling, "@{upstream}");
}

/// CI keeps `HEAD`, and an explicit `CARGO_SHADOW_TRUST_BASE` wins everywhere.
#[test]
fn ci_compares_with_head_and_an_explicit_base_wins() {
    let (_root, work, pushed, local) = repository_ahead_of_its_upstream();

    assert_eq!(resolve_trust_base(&work, true, None).commit, Ok(local.clone()));
    assert_eq!(resolve_trust_base(&work, false, Some(&local)).commit, Ok(local));
    assert_eq!(resolve_trust_base(&work, true, Some(&pushed)).commit, Ok(pushed));
}

/// Without an upstream the base falls back to the remote's default branch, and without that to `HEAD`.
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

/// The reader names a commit and reads the lockfile text there, empty when it did not exist.
#[test]
fn the_reader_reads_a_lockfile_at_the_commit_and_an_absent_one_as_empty() {
    let (_root, work, pushed, _local) = repository_ahead_of_its_upstream();
    let read = git_lock_reader_at(&work, &pushed);

    assert!(read("package/a/one").starts_with("pushed lock"));
    assert_eq!(read("package/a/two"), "");
}

/// A base that names no commit is an error naming it, not a panic.
#[test]
fn a_base_that_names_no_commit_is_an_error() {
    let (_root, work, _pushed, _local) = repository_ahead_of_its_upstream();

    let base = resolve_trust_base(&work, false, Some("no-such-revision"));

    assert!(base.commit.expect_err("not a commit").contains("no-such-revision"));
}
