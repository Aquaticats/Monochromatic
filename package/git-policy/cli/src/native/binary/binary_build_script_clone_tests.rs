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

/// Run the wrapper in `directory` with each of `variables` set,
/// and with `arguments` after the subcommand.
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
    for candidate in stamps {
        let Ok(stamp) = candidate else { continue };
        let Ok(copies) = std::fs::read_dir(stamp.path()) else { continue };
        for entry in copies {
            let Ok(copy) = entry else { continue };
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
    for candidate in entries {
        let Ok(entry) = candidate else { continue };
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

/// The wrapper's arguments with the guard's opt-out written directly after the subcommand,
///  which is the way through every refusal named here.
fn with_the_opt_out(arguments: &[&str]) -> Vec<String> {
    let mut words: Vec<String> = arguments.iter().map(|word| return (*word).to_string()).collect();
    words.insert(1, String::from("--no-enforce-worktree"));
    return words;
}

/// A nested repository inside `parent`, holding one commit and one uncommitted file,
///  which is the work no copy of the surrounding repository holds.
fn nested_repository(fixture: &Fixture, parent: &Path, name: &str) -> PathBuf {
    let nested: PathBuf = parent.join(name);
    std::fs::create_dir_all(nested.as_path()).expect("nested repository directory");
    git(fixture, nested.as_path(), &["init", "--quiet", "--initial-branch=inner"]);
    std::fs::write(nested.join("committed.txt"), b"nested committed\n").expect("nested tracked file");
    git(fixture, nested.as_path(), &["add", "--", "committed.txt"]);
    git(fixture, nested.as_path(), &["commit", "--quiet", "--message=nested"]);
    std::fs::write(nested.join("uncommitted.txt"), b"nested uncommitted user data\n").expect("nested untracked file");
    return nested;
}

/// A clone whose branch `other` tracks `inway.txt`,
///  with an untracked `inway.txt` holding user data on the checked-out branch.
/// Why:
///  Every tree-writing form replaces that file with the branch's content,
///  and the file exists nowhere else.
fn clone_with_an_untracked_file_in_the_way(fixture: &Fixture) -> PathBuf {
    let clone: PathBuf = edited_clone(fixture, build_script_output_dir(fixture).as_path());
    git(fixture, clone.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(clone.join("inway.txt"), b"TREE CONTENT\n").expect("the branch's file");
    git(fixture, clone.as_path(), &["add", "--", "inway.txt"]);
    git(fixture, clone.as_path(), &["commit", "--quiet", "--message=other"]);
    git(fixture, clone.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(clone.join("inway.txt"), b"USER DATA\n").expect("the untracked file in the way");
    return clone;
}

/// What:
///  A hard reset in a clone under the Cargo build directory runs over the build step's own tracked change
///  and copies nothing.
/// Why:
///  The exemption exists so a build step that resets its own vendored clone is not refused,
///  and the owner's rule of 2026-11-12 allows no saved copy:
///  the refusal is the protection, not a copy beside the audit log.
#[test]
fn a_hard_reset_in_a_clone_under_the_cargo_build_directory_runs_without_a_copy() {
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
    // The reset ran: Git wrote the committed content over the build step's own edit.
    assert_eq!(
        std::fs::read(clone.join("tracked.txt")).expect("the reset ran"),
        b"one\n"
    );
    // The wrapper keeps no copy of it.
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "an exempt form copied work it let through");
    let audit: String = audit_lines(clone.join(".git").as_path());
    assert!(audit.contains("\"outcome\":\"exempt\""), "{audit}");
    remove(&fixture);
}

/// What:
///  A forced clean in a clone under the Cargo build directory is refused over the untracked file it would delete,
///  the refusal names it, and its opt-out runs the clean as Git would with no copy made.
/// Why:
///  The review found `git clean -fdx` losing that file there unsaved;
///  the owner's rule of 2026-11-12 is a refusal that names the file,
///  with `--no-enforce-worktree` as the way through and no copy anywhere.
#[test]
fn a_forced_clean_in_a_clone_under_the_cargo_build_directory_refuses_the_untracked_file() {
    let fixture: Fixture = fixture("cargo-build-directory-clean");
    source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    std::fs::write(clone.join("untracked.txt"), b"untracked user bytes\n").expect("the untracked file");
    let refused: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        &["clean", "-fdx"],
    );
    assert!(rejected_by_policy(&refused), "{}", stderr_of(&refused));
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    let said: String = stderr_of(&refused);
    assert!(said.contains("untracked.txt"), "{said}");
    assert!(said.contains("--no-enforce-worktree"), "{said}");
    // The refused clean changed nothing, and no copy was made for it.
    assert_eq!(
        std::fs::read(clone.join("untracked.txt")).expect("the refused clean kept the file"),
        b"untracked user bytes\n"
    );
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "a refusal copied work");
    // The opt-out runs the clean as real Git would: the file is deleted and nothing is copied.
    let words: Vec<String> = with_the_opt_out(&["clean", "-fdx"]);
    let borrowed: Vec<&str> = words.iter().map(String::as_str).collect();
    let ran: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        borrowed.as_slice(),
    );
    assert_eq!(ran.code, Some(0), "{}", stderr_of(&ran));
    assert!(!clone.join("untracked.txt").exists(), "the opt-out ran the clean as Git would");
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "the opt-out copied work it ran over");
    remove(&fixture);
}

/// What:
///  A clean with a second force that deletes no ignored file is refused over a nested repository it would delete,
///  the refusal names the repository, and its opt-out deletes it as Git would with no copy.
/// Why:
///  Git's second force removes a nested repository whatever `-x` asks
///  (`builtin/clean.c`: `if (force > 1) rm_flags = 0`),
///  and the review found `git clean -ffd` in an exempt clone deleting one,
///  history and uncommitted file included, with nothing saved.
#[test]
fn a_forced_clean_without_ignored_files_in_a_clone_under_the_cargo_build_directory_refuses_a_nested_repository() {
    let fixture: Fixture = fixture("cargo-build-directory-clean-nested");
    source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    let nested: PathBuf = nested_repository(&fixture, clone.as_path(), "nested");
    // Real Git's own reading: the second force removes the repository, and `-x` is not part of it.
    let dry: Output = git(&fixture, clone.as_path(), &["clean", "-n", "-f", "-f", "-d"]);
    assert!(
        String::from_utf8_lossy(dry.stdout.as_slice()).contains("Would remove nested/"),
        "{}",
        String::from_utf8_lossy(dry.stdout.as_slice())
    );
    let refused: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        &["clean", "-f", "-f", "-d"],
    );
    assert!(rejected_by_policy(&refused), "{}", stderr_of(&refused));
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    let said: String = stderr_of(&refused);
    assert!(said.contains("nested"), "{said}");
    assert!(said.contains("--no-enforce-worktree"), "{said}");
    assert!(nested.join(".git").exists(), "the refused clean deleted the nested repository");
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "a refusal copied work");
    // The opt-out runs the clean as real Git would: the repository is gone and nothing is copied.
    let words: Vec<String> = with_the_opt_out(&["clean", "-f", "-f", "-d"]);
    let borrowed: Vec<&str> = words.iter().map(String::as_str).collect();
    let ran: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        borrowed.as_slice(),
    );
    assert_eq!(ran.code, Some(0), "{}", stderr_of(&ran));
    assert!(!nested.exists(), "the opt-out ran the clean as Git would");
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "the opt-out copied work it ran over");
    remove(&fixture);
}

/// What:
///  A forced clean of ignored files alone in a clone under the Cargo build directory runs and is logged,
///  with no refusal and no copy.
/// Why:
///  The owner decided on 2026-11-12 that `git clean -fX` and `git clean -fx` delete, exit 0 and are logged;
///  an ignored file is the one class this guard does not refuse over.
#[test]
fn a_forced_clean_of_ignored_files_in_a_clone_under_the_cargo_build_directory_runs_and_is_logged() {
    let fixture: Fixture = fixture("cargo-build-directory-clean-ignored");
    source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    std::fs::write(clone.join(".gitignore"), b"secret.env\n").expect("ignore file");
    git(&fixture, clone.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, clone.as_path(), &["commit", "--quiet", "--message=ignored"]);
    std::fs::write(clone.join("secret.env"), b"IGNORED USER DATA\n").expect("the ignored file");
    for arguments in [&["clean", "-fdx"], &["clean", "-fdX"]] {
        std::fs::write(clone.join("secret.env"), b"IGNORED USER DATA\n").expect("the ignored file");
        let observed: Observed = run_with_variables(
            &fixture,
            clone.as_path(),
            &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
            arguments,
        );
        assert!(!rejected_by_policy(&observed), "{arguments:?}: {}", stderr_of(&observed));
        assert_eq!(observed.code, Some(0), "{arguments:?}: {}", stderr_of(&observed));
        assert!(!clone.join("secret.env").exists(), "{arguments:?} did not delete the ignored file");
        assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "{arguments:?} copied an ignored file");
    }
    let audit: String = audit_lines(clone.join(".git").as_path());
    assert_eq!(audit.matches("\"outcome\":\"exempt\"").count(), 2, "{audit}");
    assert!(!audit.contains("saved"), "{audit}");
    remove(&fixture);
}

/// What:
///  Every tree-writing form that would overwrite an untracked file in a clone under the Cargo build directory
///  is refused and names the file, and its opt-out runs the form as Git would with no copy.
/// Why:
///  The review found `reset --hard`, a forced checkout, a forced switch, `restore`, `read-tree --reset -u`
///  and `checkout <rev> -- <path>` each replacing that file unsaved;
///  the owner's rule of 2026-11-12 is one refusal per form, naming what it would destroy,
///  with the guard's opt-out as the way through.
#[test]
fn every_tree_form_that_would_overwrite_an_untracked_file_is_refused() {
    let forms: [&[&str]; 6] = [
        &["reset", "--hard", "other"],
        &["checkout", "--force", "other"],
        &["switch", "--force", "other"],
        &["restore", "--source=other", "--", "inway.txt"],
        &["read-tree", "--reset", "-u", "other"],
        &["checkout", "other", "--", "inway.txt"],
    ];
    for (index, arguments) in forms.iter().enumerate() {
        let fixture: Fixture = fixture(format!("cargo-build-directory-form-{index}").as_str());
        source_repository(&fixture);
        let build_dir: PathBuf = cargo_build_directory(&fixture);
        let clone: PathBuf = clone_with_an_untracked_file_in_the_way(&fixture);
        let refused: Observed = run_with_variables(
            &fixture,
            clone.as_path(),
            &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
            arguments,
        );
        assert!(rejected_by_policy(&refused), "{arguments:?}: {}", stderr_of(&refused));
        let said: String = stderr_of(&refused);
        assert!(said.contains("inway.txt"), "{arguments:?}: {said}");
        assert!(said.contains("--no-enforce-worktree"), "{arguments:?}: {said}");
        assert_eq!(
            std::fs::read(clone.join("inway.txt")).expect("the refused form kept the file"),
            b"USER DATA\n",
            "{arguments:?}"
        );
        assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "{arguments:?} copied work");
        let words: Vec<String> = with_the_opt_out(arguments);
        let borrowed: Vec<&str> = words.iter().map(String::as_str).collect();
        let ran: Observed = run_with_variables(
            &fixture,
            clone.as_path(),
            &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
            borrowed.as_slice(),
        );
        assert_eq!(ran.code, Some(0), "{arguments:?} with the opt-out: {}", stderr_of(&ran));
        assert_eq!(
            std::fs::read(clone.join("inway.txt")).expect("the opt-out ran the form as Git would"),
            b"TREE CONTENT\n",
            "{arguments:?}"
        );
        assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "{arguments:?} copied work");
        remove(&fixture);
    }
}

/// What:
///  `mv --force` onto an untracked destination in a clone under the Cargo build directory is refused,
///  names the destination, and its opt-out overwrites it as Git would with no copy.
/// Why:
///  A move is not a tree write, so its written path is the destination itself;
///  the destination exists nowhere else once the move replaces it.
#[test]
fn a_forced_move_in_a_clone_under_the_cargo_build_directory_refuses_an_untracked_destination() {
    let fixture: Fixture = fixture("cargo-build-directory-move");
    source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    std::fs::write(clone.join("moved.txt"), b"USER DATA\n").expect("the untracked destination");
    let refused: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        &["mv", "--force", "tracked.txt", "moved.txt"],
    );
    assert!(rejected_by_policy(&refused), "{}", stderr_of(&refused));
    let said: String = stderr_of(&refused);
    assert!(said.contains("moved.txt"), "{said}");
    assert_eq!(
        std::fs::read(clone.join("moved.txt")).expect("the refused move kept the destination"),
        b"USER DATA\n"
    );
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "a refusal copied work");
    let words: Vec<String> = with_the_opt_out(&["mv", "--force", "tracked.txt", "moved.txt"]);
    let borrowed: Vec<&str> = words.iter().map(String::as_str).collect();
    let ran: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        borrowed.as_slice(),
    );
    assert_eq!(ran.code, Some(0), "{}", stderr_of(&ran));
    assert_eq!(
        std::fs::read(clone.join("moved.txt")).expect("the opt-out ran the move as Git would"),
        b"edited by the build step\n"
    );
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "the opt-out copied work");
    remove(&fixture);
}

/// What:
///  `checkout-index --force --prefix` over an untracked file in a clone under the Cargo build directory
///  is refused, names the file under its prefix, and its opt-out writes it as Git would with no copy.
/// Why:
///  The index's paths move under the prefix the command names,
///  and the untracked file there is replaced by the index's content;
///  the refusal must name the path as the command writes it.
#[test]
fn a_forced_checkout_index_in_a_clone_under_the_cargo_build_directory_refuses_an_untracked_file() {
    let fixture: Fixture = fixture("cargo-build-directory-checkout-index");
    source_repository(&fixture);
    let build_dir: PathBuf = cargo_build_directory(&fixture);
    let clone: PathBuf = edited_clone(&fixture, build_script_output_dir(&fixture).as_path());
    std::fs::create_dir_all(clone.join("sub")).expect("the prefix directory");
    std::fs::write(clone.join("sub/tracked.txt"), b"USER DATA\n").expect("the untracked file in the way");
    let refused: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        &["checkout-index", "--force", "--prefix=sub/", "--all"],
    );
    assert!(rejected_by_policy(&refused), "{}", stderr_of(&refused));
    let said: String = stderr_of(&refused);
    assert!(said.contains("sub/tracked.txt"), "{said}");
    assert!(said.contains("--no-enforce-worktree"), "{said}");
    assert_eq!(
        std::fs::read(clone.join("sub/tracked.txt")).expect("the refused form kept the file"),
        b"USER DATA\n"
    );
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "a refusal copied work");
    let words: Vec<String> = with_the_opt_out(&["checkout-index", "--force", "--prefix=sub/", "--all"]);
    let borrowed: Vec<&str> = words.iter().map(String::as_str).collect();
    let ran: Observed = run_with_variables(
        &fixture,
        clone.as_path(),
        &[("CARGO_TARGET_DIR", build_dir.as_os_str())],
        borrowed.as_slice(),
    );
    assert_eq!(ran.code, Some(0), "{}", stderr_of(&ran));
    assert_eq!(
        std::fs::read(clone.join("sub/tracked.txt")).expect("the opt-out ran the form as Git would"),
        b"one\n"
    );
    assert!(saved_copies(clone.join(".git").as_path()).is_empty(), "the opt-out copied work");
    remove(&fixture);
}

/// What:
///  A relative `CARGO_BUILD_DIR`,
///  which Cargo never reads,
///  does not mask the absolute `CARGO_TARGET_DIR` beside it.
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
