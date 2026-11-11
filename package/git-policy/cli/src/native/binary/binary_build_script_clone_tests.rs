//! What:
//!  The guard of uncommitted work inside a clone a build script made for itself,
//! through the built executable:
//!  the checkout runs where Cargo's `OUT_DIR` holds that clone,
//! and the same checkout in the same clone is still refused where `OUT_DIR` names another directory.
//! Why:
//!  A crate that vendors another project clones it into its build-script output directory
//! (`libghostty-vt-sys` cloned Ghostty there and checked a commit out),
//! and that clone belongs to the build step,
//! not to the person
//!  (`doc/troubleshooting/git-wrapper-rejects-checkout-in-no-checkout-clone.md`).
//! `linked-worktree-only` exempts a repository under an allowlisted tool cache,
//!  so the exemption is the build script's own directory and nothing wider;
//! a form that only moves `HEAD` asks no measurement and therefore prints no exemption notice,
//! which `policy_checks_tests` records.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // command.env('OUT_DIR', outDir); spawnSync(wrapper, ['checkout', commit], { cwd: clone }).status === 0
//! ```

use super::support::{
    Fixture, Observed, bounded, fixture, git, observe, remove, repository, stderr_of, wrapper_first_path,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Cargo's build-script output directory for one fixture crate,
/// the shape this host's build cache uses.
const OUT_DIR_TAIL: &str = "cargo-build/aa/0123456789abcd/debug/build/libghostty-vt-sys-1/out";

/// A repository with one commit,
/// and the commit it names.
///
/// The checked-out commit is the one the clone already holds,
/// so Git carries a local edit over instead of refusing the move itself.
fn source_repository(fixture: &Fixture) -> String {
    let repo: PathBuf = repository(fixture, OsStr::new("source"));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let named: Output = git(fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

/// A clone of `source_repository` inside `out_dir`,
/// holding an edit no person made,
/// the state a second build step (or an interrupted first one) leaves.
fn edited_clone(fixture: &Fixture, out_dir: &Path) -> PathBuf {
    std::fs::create_dir_all(out_dir).expect("build-script output directory");
    let source: PathBuf = fixture.root.join("source");
    let source_text: &str = source.to_str().expect("fixture paths are UTF-8");
    git(fixture, out_dir, &["clone", "--quiet", source_text, "ghostty-src"]);
    let clone: PathBuf = out_dir.join("ghostty-src");
    std::fs::write(clone.join("tracked.txt"), b"edited by the build step\n").expect("edit");
    return clone;
}

/// Run the wrapper in `directory` with Cargo's `OUT_DIR` naming `out_dir`.
fn checkout_with_out_dir(fixture: &Fixture, directory: &Path, out_dir: &Path, commit: &str) -> Observed {
    let mut command: Command = bounded(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
    );
    command
        .env("OUT_DIR", out_dir.as_os_str())
        .current_dir(directory)
        .args(["checkout", commit]);
    return observe(&mut command, b"");
}

/// Run the wrapper's `git reset --hard HEAD` in `directory` with Cargo's `OUT_DIR` naming `out_dir`,
///  the form that overwrites tracked files and is refused in a worktree holding uncommitted state.
fn reset_hard_with_out_dir(fixture: &Fixture, directory: &Path, out_dir: &Path) -> Observed {
    let mut command: Command = bounded(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
    );
    command
        .env("OUT_DIR", out_dir.as_os_str())
        .current_dir(directory)
        .args(["reset", "--hard", "HEAD"]);
    return observe(&mut command, b"");
}

/// What:
///  A linked worktree of the build script's clone, checked out where `directory` says,
///  holding an edit no person made.
/// Why:
///  A linked worktree keeps its Git directory inside the clone
///  (`<clone>/.git/worktrees/<name>`)
///  and its files wherever it was added,
///  so the two can stand on opposite sides of the build script's directory.
fn linked_worktree_of_the_clone(fixture: &Fixture, clone: &Path, directory: &Path) -> PathBuf {
    let named: &str = directory.to_str().expect("fixture paths are UTF-8");
    git(fixture, clone, &["worktree", "add", "--quiet", "--detach", named]);
    std::fs::write(directory.join("tracked.txt"), b"edited where the person works\n").expect("edit");
    return directory.to_path_buf();
}

/// Whether the wrapper itself rejected the command.
fn rejected_by_policy(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// The clone a build script made in its own output directory is exempt,
///  so the checkout runs and Git carries the edit over.
#[test]
fn a_checkout_in_a_build_scripts_own_clone_runs() {
    let fixture: Fixture = fixture("build-script-clone-exempt");
    let commit: String = source_repository(&fixture);
    let out_dir: PathBuf = fixture.root.join(OUT_DIR_TAIL);
    let clone: PathBuf = edited_clone(&fixture, out_dir.as_path());
    let observed: Observed = checkout_with_out_dir(&fixture, clone.as_path(), out_dir.as_path(), commit.as_str());
    assert!(!rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    // Git moved `HEAD` to the commit and carried the edit, which proves the command reached Git.
    assert_eq!(
        std::fs::read(clone.join("tracked.txt")).expect("the checkout kept the file"),
        b"edited by the build step\n"
    );
    let moved: Output = git(&fixture, clone.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(String::from_utf8_lossy(&moved.stdout).trim(), commit);
    remove(&fixture);
}

/// The same clone beside the build-script directory keeps its guard:
///  only the directory the variable names is exempt.
#[test]
fn a_checkout_in_a_clone_beside_the_build_scripts_directory_is_refused() {
    let fixture: Fixture = fixture("build-script-clone-guarded");
    let commit: String = source_repository(&fixture);
    let out_dir: PathBuf = fixture.root.join(OUT_DIR_TAIL);
    let clone: PathBuf = edited_clone(&fixture, out_dir.as_path());
    let elsewhere: PathBuf = fixture.root.join("a-sibling-of-the-output-directory");
    let observed: Observed = checkout_with_out_dir(&fixture, clone.as_path(), elsewhere.as_path(), commit.as_str());
    assert!(rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(1), "{}", stderr_of(&observed));
    assert_eq!(
        std::fs::read(clone.join("tracked.txt")).expect("the refused command changed nothing"),
        b"edited by the build step\n"
    );
    remove(&fixture);
}

/// What:
///  A linked worktree of the build script's clone that stands outside the build script's directory keeps its guard.
/// Why:
///   The clone's Git directory stays inside that directory even when its linked worktree does not,
///  so an exemption judged by the Git directory alone would let a form overwrite the person's work
///  (`git reset --hard HEAD` here once replaced the edit with committed content and exited 0).
#[test]
fn a_discard_in_a_linked_worktree_outside_the_build_scripts_directory_is_refused() {
    let fixture: Fixture = fixture("build-script-clone-linked-outside");
    source_repository(&fixture);
    let out_dir: PathBuf = fixture.root.join(OUT_DIR_TAIL);
    let clone: PathBuf = edited_clone(&fixture, out_dir.as_path());
    let outside: PathBuf = fixture.root.join("the-persons-worktree");
    std::fs::create_dir(&outside).expect("where the person asked for the worktree");
    let worktree: PathBuf = linked_worktree_of_the_clone(&fixture, clone.as_path(), outside.as_path());
    let observed: Observed = reset_hard_with_out_dir(&fixture, worktree.as_path(), out_dir.as_path());
    assert!(rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(1), "{}", stderr_of(&observed));
    assert_eq!(
        std::fs::read(worktree.join("tracked.txt")).expect("the refused command changed nothing"),
        b"edited where the person works\n"
    );
    remove(&fixture);
}

/// The same linked worktree inside the build script's directory belongs to the build step,
///  so the exemption still runs the command and Git carries the edit over.
#[test]
fn a_discard_in_a_linked_worktree_inside_the_build_scripts_directory_stays_exempt() {
    let fixture: Fixture = fixture("build-script-clone-linked-inside");
    source_repository(&fixture);
    let out_dir: PathBuf = fixture.root.join(OUT_DIR_TAIL);
    let clone: PathBuf = edited_clone(&fixture, out_dir.as_path());
    let inside: PathBuf = out_dir.join("worktrees/second-build-step");
    std::fs::create_dir_all(&inside).expect("the build script's own worktree directory");
    let worktree: PathBuf = linked_worktree_of_the_clone(&fixture, clone.as_path(), inside.as_path());
    let observed: Observed = reset_hard_with_out_dir(&fixture, worktree.as_path(), out_dir.as_path());
    assert!(!rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    // Git reached the command and wrote the committed content over the build step's own edit.
    assert_eq!(
        std::fs::read(worktree.join("tracked.txt")).expect("the exemption let the command run"),
        b"one\n"
    );
    remove(&fixture);
}
