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
use super::support::{Fixture, Observed, direct, fixture, git, observe, without_wait_notices};
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
