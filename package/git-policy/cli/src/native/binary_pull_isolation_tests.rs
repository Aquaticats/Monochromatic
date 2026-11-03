//! What:
//!  A `git pull` that merges, through the built executable: with another session's staged work in the index it ends as
//!  native Git's pull ends on a clean copy, and every pull the wrapper does not complete keeps native Git's own run.
//! Why:
//!  Native Git's pull runs `git merge` as a child of Git (`builtin/pull.c`, `run_merge`), so the merge never reached the
//!  wrapper and kept native Git's failures: a staged index, a held `index.lock`, a late ref update. The owner's rules are
//!  "Auto merging should never fail." and "Manual merge should never fail too.", so the merge phase of a pull goes
//!  through the same isolated merge as `git merge` (`doc/planning/cli-git-merge-auto-resolution.md`, "Pull").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ours = await wrapped(['pull', '--no-rebase', '--no-edit'], dirtyCopy); const native = await direct(['pull', '--no-rebase', '--no-edit'], cleanCopy);
//! expect(await commitShape(ours)).toEqual(await commitShape(native));
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::merge_isolation::{copy_of, diverged, snapshot, stage_unrelated_work};
use super::support::{Fixture, Observed, direct, fixture, git, observe, repository, without_wait_notices};
use std::path::{Path, PathBuf};

/// A repository whose `main` follows a bare remote that holds the `feature` history as its `main`, so a pull merges.
fn diverged_with_remote(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = diverged(fixture, "dirty");
    let remote: PathBuf = fixture.root.join("remote.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
    git(fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(fixture, repo.as_path(), &["push", "--quiet", "origin", "feature:main"]);
    git(fixture, repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(fixture, repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    return repo;
}

/// Run a wrapped command to its end, without the wait notices a slow step may print.
fn run(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    let observed: Observed = observe(&mut wrapped_commit(fixture, repo, arguments), b"");
    return Observed { stderr: without_wait_notices(observed.stderr.as_slice()), ..observed };
}

/// Run native Git directly, the oracle on a clean copy.
fn native(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    return observe(direct(fixture).current_dir(repo).args(arguments), b"");
}

/// The parts of a merge commit that must equal native Git's: tree, both parents, message, author and committer names.
fn commit_shape(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return vec![
        git_text(fixture, repo, &["rev-parse", "HEAD^{tree}"]),
        git_text(fixture, repo, &["rev-parse", "HEAD^1"]),
        git_text(fixture, repo, &["rev-parse", "HEAD^2"]),
        git_text(fixture, repo, &["log", "-1", "--format=%an <%ae>|%cn <%ce>|%B"]),
    ];
}

/// A pull that merges over another session's staged work is completed, leaves that work as it was, and equals native Git's
///  pull on a clean copy in tree, parents, message and words.
#[test]
fn a_pull_that_merges_is_completed_over_staged_work_and_equals_native_gits() {
    let fixture: Fixture = fixture("pull-isolated-oracle");
    let dirty: PathBuf = diverged_with_remote(&fixture);
    let clean: PathBuf = copy_of(&fixture, dirty.as_path(), "clean");
    stage_unrelated_work(&fixture, dirty.as_path());
    let staged: Vec<String> = git_text(&fixture, dirty.as_path(), &["diff", "--cached", "--name-status"]).lines().map(String::from).collect();
    let arguments: [&str; 3] = ["pull", "--no-rebase", "--no-edit"];
    let theirs: Observed = native(&fixture, clean.as_path(), &arguments);
    assert_eq!(theirs.code, Some(0), "native Git pulls into the clean copy: {}", String::from_utf8_lossy(&theirs.stderr));
    let ours: Observed = run(&fixture, dirty.as_path(), &arguments);
    assert_eq!(ours.code, Some(0), "{}", String::from_utf8_lossy(&ours.stderr));
    assert_eq!(String::from_utf8_lossy(&ours.stdout), String::from_utf8_lossy(&theirs.stdout), "output");
    assert_eq!(String::from_utf8_lossy(&ours.stderr), String::from_utf8_lossy(&theirs.stderr), "diagnostics");
    assert_eq!(commit_shape(&fixture, dirty.as_path()), commit_shape(&fixture, clean.as_path()));
    let after: Vec<String> = git_text(&fixture, dirty.as_path(), &["diff", "--cached", "--name-status", "HEAD"]).lines().map(String::from).collect();
    assert_eq!(after, staged, "the other session's staged work is still staged and nothing else is");
    assert!(!snapshot(&fixture, dirty.as_path()).status.is_empty(), "the staged work is still there");
    remove_settled(&fixture);
}

/// The same pull into a clean index, which the isolated merge completes since it completes a clean-index merge, equals native Git's.
#[test]
fn a_pull_into_a_clean_index_equals_native_gits() {
    let fixture: Fixture = fixture("pull-isolated-clean");
    let ours_repo: PathBuf = diverged_with_remote(&fixture);
    let theirs_repo: PathBuf = copy_of(&fixture, ours_repo.as_path(), "clean");
    let arguments: [&str; 3] = ["pull", "--no-rebase", "--no-edit"];
    let theirs: Observed = native(&fixture, theirs_repo.as_path(), &arguments);
    let ours: Observed = run(&fixture, ours_repo.as_path(), &arguments);
    assert_eq!((ours.code, theirs.code), (Some(0), Some(0)), "{}", String::from_utf8_lossy(&ours.stderr));
    assert_eq!(String::from_utf8_lossy(&ours.stdout), String::from_utf8_lossy(&theirs.stdout), "output");
    assert_eq!(String::from_utf8_lossy(&ours.stderr), String::from_utf8_lossy(&theirs.stderr), "diagnostics");
    assert_eq!(commit_shape(&fixture, ours_repo.as_path()), commit_shape(&fixture, theirs_repo.as_path()));
    remove_settled(&fixture);
}

/// A pull the wrapper does not read exactly, or that Git refuses by its own advice, is native Git's whole, in words and status.
#[test]
fn a_pull_native_git_refuses_or_the_wrapper_does_not_read_is_native_gits() {
    let cases: [(&str, &[&str]); 4] = [
        ("advice", &["pull", "--no-edit"]),
        ("ff-only", &["pull", "--ff-only"]),
        ("rebase", &["pull", "--rebase"]),
        ("abbreviation", &["pull", "--no-reb", "--no-edit"]),
    ];
    for (name, arguments) in cases {
        let fixture: Fixture = fixture(format!("pull-native-{name}").as_str());
        let ours_repo: PathBuf = diverged_with_remote(&fixture);
        let theirs_repo: PathBuf = copy_of(&fixture, ours_repo.as_path(), "clean");
        let theirs: Observed = native(&fixture, theirs_repo.as_path(), arguments);
        let ours: Observed = run(&fixture, ours_repo.as_path(), arguments);
        assert_eq!(ours.code, theirs.code, "{name}: {}", String::from_utf8_lossy(&ours.stderr));
        assert_eq!(String::from_utf8_lossy(&ours.stdout), String::from_utf8_lossy(&theirs.stdout), "{name}: output");
        assert_eq!(String::from_utf8_lossy(&ours.stderr), String::from_utf8_lossy(&theirs.stderr), "{name}: diagnostics");
        assert_eq!(git_text(&fixture, ours_repo.as_path(), &["rev-parse", "HEAD"]), git_text(&fixture, theirs_repo.as_path(), &["rev-parse", "HEAD"]), "{name}: head");
        remove_settled(&fixture);
    }
}

/// A fetch that moves the checked-out branch takes the working tree along, then the pull merges, as in native Git.
#[test]
fn a_fetch_that_moves_the_checked_out_branch_updates_the_working_tree_as_native_git_does() {
    let fixture: Fixture = fixture("pull-moved-head");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
    git(&fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("f.txt"), b"one\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "f.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=one"]);
    std::fs::write(repo.join("f.txt"), b"two\n").expect("file");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    git(&fixture, repo.as_path(), &["push", "--quiet", "origin", "main"]);
    git(&fixture, repo.as_path(), &["reset", "--quiet", "--hard", "HEAD~1"]);
    git(&fixture, repo.as_path(), &["config", "remote.origin.fetch", "+refs/heads/*:refs/heads/*"]);
    git(&fixture, repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(&fixture, repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    let clean: PathBuf = copy_of(&fixture, repo.as_path(), "clean");
    let arguments: [&str; 3] = ["pull", "--no-rebase", "--no-edit"];
    let theirs: Observed = native(&fixture, clean.as_path(), &arguments);
    let ours: Observed = run(&fixture, repo.as_path(), &arguments);
    assert_eq!((ours.code, theirs.code), (Some(0), Some(0)), "{}", String::from_utf8_lossy(&ours.stderr));
    assert_eq!(String::from_utf8_lossy(&ours.stdout), String::from_utf8_lossy(&theirs.stdout), "output");
    assert_eq!(String::from_utf8_lossy(&ours.stderr), String::from_utf8_lossy(&theirs.stderr), "diagnostics");
    assert_eq!(std::fs::read_to_string(repo.join("f.txt")).expect("file"), "two\n", "the working tree followed the fetch");
    assert_eq!(git_text(&fixture, repo.as_path(), &["status", "--porcelain=v1"]), "", "nothing is left modified");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), git_text(&fixture, clean.as_path(), &["rev-parse", "HEAD"]));
    remove_settled(&fixture);
}
