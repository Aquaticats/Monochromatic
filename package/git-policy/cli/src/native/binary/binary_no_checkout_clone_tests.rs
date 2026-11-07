//! What:
//!  The guard of uncommitted work,
//! through the built executable,
//! on a clone made with `--no-checkout`:
//!  the checkout of a commit there runs,
//! and the same checkout is still refused where tracked changes really exist.
//! Why:
//!  A `git clone --no-checkout` leaves no index file,
//! so Git itself sees an initial checkout and has nothing to protect
//!  (`is_index_unborn` in `read-cache.c`,
//! read by `builtin/checkout.c`).
//! `git status` lists every file as a staged deletion
//!  in such a clone,
//! and the guard read that as uncommitted work and refused the checkout that
//!  `libghostty-vt-sys`'s build script runs in its fresh clone of Ghostty.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // run('git', ['clone', '--no-checkout', source, clone]);
//! // spawnSync(wrapper, ['checkout', sha], { cwd: clone }).status === 0
//! ```

use super::support::{Fixture, Observed, fixture, git, remove, repository, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;

/// A repository with one committed file `tracked.txt` and a second branch `topic` at the same commit.
fn source_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("source"));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    git(fixture, repo.as_path(), &["branch", "topic"]);
    return repo;
}

/// The commit `HEAD` names in `repo`.
fn head_commit(fixture: &Fixture, repo: &Path) -> String {
    let output: Output = git(fixture, repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// Whether the wrapper itself rejected the command.
fn rejected_by_policy(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// In a `--no-checkout` clone,
/// which has no index file yet,
/// checking out a commit runs and writes the files.
#[test]
fn a_commit_is_checked_out_in_a_clone_made_without_checkout() {
    let fixture: Fixture = fixture("no-checkout-clone-passes");
    let source: PathBuf = source_repository(&fixture);
    let commit: String = head_commit(&fixture, source.as_path());
    git(&fixture, fixture.root.as_path(), &["clone", "--quiet", "--no-checkout", "source", "clone"]);
    let clone: PathBuf = fixture.root.join("clone");
    assert!(!clone.join(".git").join("index").exists(), "a no-checkout clone has no index file");
    let observed: Observed = run_wrapped(&fixture, clone.as_path(), &["checkout", commit.as_str()]);
    assert!(!rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(std::fs::read(clone.join("tracked.txt")).expect("the checkout wrote the file"), b"one\n");
    remove(&fixture);
}

/// A modified tracked file in a worktree that has an index still refuses the checkout of another branch,
/// and changes nothing.
#[test]
fn a_checkout_is_refused_while_a_tracked_file_is_modified() {
    let fixture: Fixture = fixture("no-checkout-clone-modified");
    let repo: PathBuf = source_repository(&fixture);
    std::fs::write(repo.join("tracked.txt"), b"two\n").expect("edit");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "topic"]);
    assert!(rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(std::fs::read(repo.join("tracked.txt")).expect("tracked file"), b"two\n");
    remove(&fixture);
}

/// A staged deletion of a file that is also gone from the working tree,
/// in a worktree that has an index,
/// still refuses the
/// checkout:
/// the index file exists,
/// so Git itself would protect the deletion and so does the guard.
#[test]
fn a_checkout_is_refused_while_a_deletion_is_staged_and_the_file_is_gone() {
    let fixture: Fixture = fixture("no-checkout-clone-staged-deletion");
    let repo: PathBuf = source_repository(&fixture);
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--", "tracked.txt"]);
    assert!(!repo.join("tracked.txt").exists(), "the file is gone from the working tree");
    assert!(repo.join(".git").join("index").exists(), "the index file exists");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "topic"]);
    assert!(rejected_by_policy(&observed), "{}", stderr_of(&observed));
    remove(&fixture);
}
