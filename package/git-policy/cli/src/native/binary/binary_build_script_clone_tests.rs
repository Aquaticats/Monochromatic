//! What:
//!  The guard of uncommitted work inside a clone a build step made for itself,
//! through the built executable:
//!  the checkout runs where Cargo's `OUT_DIR` or the Cargo build directory holds that clone,
//! a hard reset or a forced clean there copies what it would discard first and then runs,
//! and the same checkout in the same clone is still refused where neither names a directory around it.
//! Why:
//!  A crate that vendors another project clones it into its own output directory
//! (`libghostty-vt-sys` cloned Ghostty there and checked a commit out),
//! and that clone belongs to the build step,
//! not to the person
//!  (`doc/troubleshooting/git-wrapper-rejects-checkout-in-no-checkout-clone.md`).
//! `linked-worktree-only` exempts a repository under an allowlisted tool directory,
//!  so the exemption is the build script's own directory,
//! or the whole Cargo build directory around it,
//! and nothing wider;
//! a form that only moves `HEAD` asks no measurement and therefore prints no exemption notice,
//! which `policy_checks_tests` records.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // command.env('OUT_DIR', outDir); spawnSync(wrapper, ['checkout', commit], { cwd: clone }).status === 0
//! // command.env('CARGO_TARGET_DIR', buildDir); spawnSync(wrapper, ['checkout', commit], { cwd: clone }).status === 0
//! ```

use super::support::{
    Fixture, Observed, bounded, fixture, git, observe, remove, repository, stderr_of, wrapper_first_path,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The Cargo build directory of one fixture workspace,
/// the shape this host's build cache uses.
const CARGO_BUILD_DIRECTORY_TAIL: &str = "cargo-build/aa/0123456789abcd";

/// A build-script output directory inside that build directory.
const BUILD_SCRIPT_DIRECTORY_TAIL: &str = "debug/build/libghostty-vt-sys-1/out";

/// The build directory of the fixture workspace,
///  the directory the whole of Cargo's intermediate output sits under.
fn cargo_build_directory(fixture: &Fixture) -> PathBuf {
    return fixture.root.join(CARGO_BUILD_DIRECTORY_TAIL);
}

/// The build-script output directory of the fixture workspace,
///  the directory Cargo names in `OUT_DIR`.
fn build_script_output_dir(fixture: &Fixture) -> PathBuf {
    return cargo_build_directory(fixture).join(BUILD_SCRIPT_DIRECTORY_TAIL);
}

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

/// A clone of `source_repository` inside `directory`,
/// holding an edit no person made,
/// the state a second build step (or an interrupted first one) leaves.
fn edited_clone(fixture: &Fixture, directory: &Path) -> PathBuf {
    std::fs::create_dir_all(directory).expect("the build step's directory");
    let source: PathBuf = fixture.root.join("source");
    let source_text: &str = source.to_str().expect("fixture paths are UTF-8");
    git(fixture, directory, &["clone", "--quiet", source_text, "ghostty-src"]);
    let clone: PathBuf = directory.join("ghostty-src");
    std::fs::write(clone.join("tracked.txt"), b"edited by the build step\n").expect("edit");
    return clone;
}

/// Run the wrapper in `directory` with `variable` naming `value`,
///  or with that variable absent from the child's environment when `value` is `None`.
fn checkout_with_variable(
    fixture: &Fixture,
    directory: &Path,
    variable: &str,
    value: Option<&OsStr>,
    commit: &str,
) -> Observed {
    let mut command: Command = bounded(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
    );
    // What: `if let Some(named) = value` sets the variable only when one is named.
    // Why:  The absent case is the control that an unset variable exempts nothing.
    if let Some(named) = value {
        command.env(variable, named);
    }
    command.current_dir(directory).args(["checkout", commit]);
    return observe(&mut command, b"");
}

/// Run the wrapper in `directory` with Cargo's `OUT_DIR` naming `out_dir`.
fn checkout_with_out_dir(fixture: &Fixture, directory: &Path, out_dir: &Path, commit: &str) -> Observed {
    return checkout_with_variable(fixture, directory, "OUT_DIR", Some(out_dir.as_os_str()), commit);
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

/// Run the wrapper in `directory` with each of `variables` set, and with `arguments` after the subcommand.
/// Why:
///   A build step's own discard is the same command a person runs,
///  and only the environment says which tree it runs in.
fn run_with_variables(
    fixture: &Fixture,
    directory: &Path,
    variables: &[(&str, &OsStr)],
    arguments: &[&str],
) -> Observed {
    let mut command: Command = bounded(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
    );
    for (name, value) in variables {
        command.env(name, value);
    }
    command.current_dir(directory).args(arguments);
    return observe(&mut command, b"");
}

/// The files one repository's saves wrote,
///  as `<name below the worktree>` and its bytes,
///  read from the folder below its common Git directory.
fn saved_copies(common_dir: &Path) -> Vec<(String, Vec<u8>)> {
    let root: PathBuf = common_dir.join("monochromatic-git-policy-cli/discarded");
    let mut found: Vec<(String, Vec<u8>)> = Vec::<(String, Vec<u8>)>::new();
    let Ok(stamps) = std::fs::read_dir(root.as_path()) else {
        return found;
    };
    for stamp in stamps {
        let Ok(stamp) = stamp else { continue };
        let Ok(copies) = std::fs::read_dir(stamp.path()) else { continue };
        for copy in copies {
            let Ok(copy) = copy else { continue };
            let name: String = copy.file_name().to_string_lossy().into_owned();
            let bytes: Vec<u8> = std::fs::read(copy.path()).expect("a copy holds bytes");
            found.push((name, bytes));
        }
    }
    return found;
}

/// What the repository's discard audit recorded,
///  read from every line of the file the wrapper appends to.
fn audit_lines(common_dir: &Path) -> String {
    let mut text: String = String::new();
    let Ok(entries) = std::fs::read_dir(common_dir) else {
        return text;
    };
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("monochromatic-git-policy-cli") || !name.contains("audit") || !name.ends_with(".jsonl") {
            continue;
        }
        text.push_str(&std::fs::read_to_string(entry.path()).unwrap_or_default());
    }
    return text;
}

/// What:
///  A linked worktree of the build script's clone,
///  checked out where `directory` says,
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
    let out_dir: PathBuf = build_script_output_dir(&fixture);
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
    let out_dir: PathBuf = build_script_output_dir(&fixture);
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
    let out_dir: PathBuf = build_script_output_dir(&fixture);
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
    let out_dir: PathBuf = build_script_output_dir(&fixture);
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

/// What:
///  A clone under the invocation's Cargo build directory is exempt even when the step running Git
///  has no `OUT_DIR` of its own.
/// Why:
///   The owner's answer of 2026-11-11 exempts the build directory (`#3024`),
///  where a crate that vendors another project keeps the clone it runs Git in,
///  so a checkout there runs and Git carries the edit over.
#[test]
fn a_checkout_in_a_clone_under_the_cargo_build_directory_runs() {
    let fixture: Fixture = fixture("cargo-build-directory-exempt");
    let commit: String = source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    let observed: Observed = checkout_with_variable(
        &fixture,
        clone.as_path(),
        "CARGO_TARGET_DIR",
        Some(build_dir.as_os_str()),
        commit.as_str(),
    );
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

/// The same variable naming a directory beside the clone keeps the guard:
///  only the directory the variable names is exempt.
#[test]
fn a_checkout_in_a_clone_beside_the_cargo_build_directory_is_refused() {
    let fixture: Fixture = fixture("cargo-build-directory-guarded");
    let commit: String = source_repository(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    let beside: PathBuf = fixture.root.join("another-build");
    let observed: Observed = checkout_with_variable(
        &fixture,
        clone.as_path(),
        "CARGO_TARGET_DIR",
        Some(beside.as_os_str()),
        commit.as_str(),
    );
    assert!(rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(1), "{}", stderr_of(&observed));
    assert_eq!(
        std::fs::read(clone.join("tracked.txt")).expect("the refused command changed nothing"),
        b"edited by the build step\n"
    );
    remove(&fixture);
}

/// An unset variable exempts nothing:
/// the same clone keeps the guard it had.
#[test]
fn a_checkout_in_a_clone_under_the_cargo_build_directory_is_refused_with_the_variable_unset() {
    let fixture: Fixture = fixture("cargo-build-directory-unset");
    let commit: String = source_repository(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    let observed: Observed = checkout_with_variable(&fixture, clone.as_path(), "CARGO_TARGET_DIR", None, commit.as_str());
    assert!(rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(1), "{}", stderr_of(&observed));
    assert_eq!(
        std::fs::read(clone.join("tracked.txt")).expect("the refused command changed nothing"),
        b"edited by the build step\n"
    );
    remove(&fixture);
}

/// A relative value names nothing:
///  `CARGO_TARGET_DIR=.` would name the clone itself if the wrapper resolved it against its own
///  working directory,
/// and Cargo resolves the variable against a directory the Git child does not work in.
#[test]
fn a_checkout_in_a_clone_under_a_relative_cargo_build_directory_is_refused() {
    let fixture: Fixture = fixture("cargo-build-directory-relative");
    let commit: String = source_repository(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    let observed: Observed = checkout_with_variable(
        &fixture,
        clone.as_path(),
        "CARGO_TARGET_DIR",
        Some(OsStr::new(".")),
        commit.as_str(),
    );
    assert!(rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(1), "{}", stderr_of(&observed));
    assert_eq!(
        std::fs::read(clone.join("tracked.txt")).expect("the refused command changed nothing"),
        b"edited by the build step\n"
    );
    remove(&fixture);
}

/// What:
///  A hard reset in a clone under the Cargo build directory copies the tracked edit it overwrites and then runs.
/// Why:
///   The review of the exemption found this exact loss:
///  with `CARGO_TARGET_DIR` naming an ancestor,
///  `git reset --hard HEAD` exited 0 and replaced the uncommitted edit,
///  and the exemption notice itself said nothing was saved.
///  A build is not a person and what it discards in its own clone is its own,
///  so the form must still succeed;
///  the person's edit inside that tree must still be on disk, which is what the copy is for.
#[test]
fn a_hard_reset_in_a_clone_under_the_cargo_build_directory_saves_the_edit_and_runs() {
    let fixture: Fixture = fixture("cargo-build-directory-reset");
    source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    let observed: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        &["reset", "--hard", "HEAD"],
    );
    assert!(!rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    // The reset ran: Git wrote the committed content over the edit.
    assert_eq!(
        std::fs::read(clone.join("tracked.txt")).expect("the reset ran"),
        b"one\n"
    );
    // The edit the reset overwrote is on disk beside the audit log.
    assert_eq!(
        saved_copies(clone.join(".git").as_path()),
        vec![(String::from("tracked.txt"), b"edited by the build step\n".to_vec())]
    );
    assert!(stderr_of(&observed).contains("tracked.txt => "), "{}", stderr_of(&observed));
    let audit: String = audit_lines(clone.join(".git").as_path());
    assert!(audit.contains("\"outcome\":\"saved-exempt\""), "{audit}");
    assert!(audit.contains("\"exemptedBy\":\"CARGO_TARGET_DIR\""), "{audit}");
    remove(&fixture);
}

/// What:
///  A forced clean in a clone under the Cargo build directory copies the untracked file it deletes and then runs.
/// Why:
///   The same review found `git clean -fdx` losing an untracked file there;
///  a build that cleans its own clone must still succeed,
///  and the file the clean deletes must be readable in the folder of saved copies.
#[test]
fn a_forced_clean_in_a_clone_under_the_cargo_build_directory_saves_the_untracked_file_and_runs() {
    let fixture: Fixture = fixture("cargo-build-directory-clean");
    source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    std::fs::write(clone.join("untracked.txt"), b"untracked user bytes\n").expect("the untracked file");
    let observed: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        &["clean", "-fdx"],
    );
    assert!(!rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    // The clean ran: the file is gone from the worktree.
    assert!(!clone.join("untracked.txt").exists(), "the clean ran");
    // Its bytes are on disk beside the audit log.
    assert_eq!(
        saved_copies(clone.join(".git").as_path()),
        vec![(String::from("untracked.txt"), b"untracked user bytes\n".to_vec())]
    );
    assert!(stderr_of(&observed).contains("untracked.txt => "), "{}", stderr_of(&observed));
    remove(&fixture);
}

/// What:
///  A relative `CARGO_BUILD_DIR`, which Cargo never reads, does not mask the absolute `CARGO_TARGET_DIR` beside it.
/// Why:
///   The review found the checkout refused here,
///  so a build step that exports the unread name lost the exemption its own absolute directory names;
///  the checkout runs and Git carries the edit over.
#[test]
fn a_checkout_under_an_absolute_cargo_build_directory_runs_beside_a_relative_alias() {
    let fixture: Fixture = fixture("cargo-build-directory-relative-alias");
    let commit: String = source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    let observed: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[
            ("CARGO_TARGET_DIR", build_dir.as_os_str()),
            ("CARGO_BUILD_DIR", OsStr::new("relative-not-read-by-cargo")),
        ],
        &["checkout", commit.as_str()],
    );
    assert!(!rejected_by_policy(&observed), "{}", stderr_of(&observed));
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(
        std::fs::read(clone.join("tracked.txt")).expect("the checkout kept the file"),
        b"edited by the build step\n"
    );
    let moved: Output = git(&fixture, clone.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(String::from_utf8_lossy(&moved.stdout).trim(), commit);
    remove(&fixture);
}
