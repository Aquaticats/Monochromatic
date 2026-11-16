//! What:
//!  Controls for the Git directory the lock pass searches.
//! Why:
//!  The pass must repair exactly the repository the command itself will work in,
//!  and the reading of `-C`,
//!  `--git-dir`,
//!  `GIT_DIR` and discovery is the whole of that choice.
//!  Each answer here is compared with the one real Git gives for the same directory,
//!  options and variables,
//!  so a reading that drifts from Git's own fails.
//!  The shapes the pass must leave alone are controls of their own:
//!  a bare repository,
//!  a directory this account cannot enter,
//!  and a path that is no repository.

use super::git_directory_of_command_in;
use crate::test_support::{fixture, git, remove, repository};
use std::ffi::{OsStr, OsString};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The command word the controls run after the global prefix,
/// so both readers see the shape a person types.
const SUBCOMMAND: &str = "status";

/// Own fixture argument words.
fn words(parts: &[&OsStr]) -> Vec<OsString> {
    return parts.iter().map(|part| { return part.to_os_string(); }).collect();
}

/// Own fixture environment pairs.
fn variables(pairs: &[(&str, &OsStr)]) -> Vec<(OsString, OsString)> {
    return pairs.iter().map(|(name, value)| { return (OsString::from(*name), value.to_os_string()); }).collect();
}

/// The Git directory real Git answers for `prefix` with a command word,
/// or nothing when Git refuses the invocation.
fn git_answer(current: &Path, prefix: &[OsString], environment: &[(&str, &OsStr)]) -> Option<PathBuf> {
    let mut spelling: Vec<OsString> = prefix.to_vec();
    spelling.extend([OsString::from("rev-parse"), OsString::from("--path-format=absolute"), OsString::from("--git-dir")]);
    let mut command: std::process::Command = std::process::Command::new(crate::test_support::REAL_GIT);
    let _quiet = command
        .current_dir(current)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(spelling.as_slice());
    for (name, value) in environment {
        let _set = command.env(name, value);
    }
    let output: std::process::Output = command.output().expect("real Git for the control");
    if !output.status.success() {
        return None;
    }
    let text: String = String::from_utf8(output.stdout).expect("Git prints a path this control can read");
    return Some(PathBuf::from(text.trim_end_matches(['\r', '\n'])));
}

/// What:
///  The pass's repository is real Git's own answer for the same invocation.
/// Why:
///  Git is the authority on which repository a command line selects,
///  and every shape here is compared with it.
fn assert_the_same_as_git(current: &Path, prefix: &[OsString], environment: &[(&str, &OsStr)], control: &str) -> Option<PathBuf> {
    let expected: Option<PathBuf> = git_answer(current, prefix, environment);
    let mut typed: Vec<OsString> = prefix.to_vec();
    typed.push(OsString::from(SUBCOMMAND));
    let found: Option<PathBuf> = git_directory_of_command_in(current, typed.as_slice(), variables(environment).as_slice());
    assert_eq!(found, expected, "{control}");
    return found;
}

/// What:
///  A command run inside a repository searches it,
///  as does one whose `-C` chain ends there,
///  and one run in a directory below the top level.
/// Why:
///  The directory the command runs in is the first half of Git's own selection.
#[test]
fn the_directory_the_command_runs_in_or_reaches_is_the_repository() {
    let root: PathBuf = fixture("lock-pass-directory");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let below: PathBuf = repo.join("below");
    std::fs::create_dir(below.as_path()).expect("directory below the top level");

    let inside: Option<PathBuf> = assert_the_same_as_git(repo.as_path(), &[], &[], "inside the repository");
    assert_eq!(inside, Some(repo.join(".git")));
    let _same = assert_the_same_as_git(below.as_path(), &[], &[], "below the top level");
    let _one = assert_the_same_as_git(root.as_path(), &words(&[OsStr::new("-C"), repo.as_os_str()]), &[], "one -C");
    let _chain = assert_the_same_as_git(
        root.as_path(),
        &words(&[OsStr::new("-C"), root.as_os_str(), OsStr::new("-C"), OsStr::new("repo")]),
        &[],
        "a chain of -C options",
    );
    let _empty = assert_the_same_as_git(repo.as_path(), &words(&[OsStr::new("-C"), OsStr::new("")]), &[], "an empty -C moves Git nowhere");
    remove(root.as_path());
}

/// What:
///  `--git-dir` and `GIT_DIR` name the repository the pass searches,
///  a relative `GIT_DIR` from the directory the `-C` chain ended in,
///  and the option wins over the variable.
/// Why:
///  A killed Git's lock in the repository such a command names is the one its own Git would meet.
#[test]
fn a_named_git_directory_is_the_repository() {
    let root: PathBuf = fixture("lock-pass-named");
    let named: PathBuf = repository(root.as_path(), "named");
    let other: PathBuf = repository(root.as_path(), "other");
    let named_git: PathBuf = named.join(".git");
    let other_git: PathBuf = other.join(".git");

    let beside: Option<PathBuf> = assert_the_same_as_git(
        root.as_path(),
        &words(&[OsStr::new("--git-dir"), named_git.as_os_str(), OsStr::new("--work-tree"), named.as_os_str()]),
        &[],
        "--git-dir with --work-tree",
    );
    assert_eq!(beside, Some(named_git.clone()), "the repository the option names, not the one the directory is in");
    let inline: OsString = OsString::from(format!("--git-dir={}", named_git.display()));
    let _inline = assert_the_same_as_git(root.as_path(), &[inline], &[], "the inline spelling");
    let _variable = assert_the_same_as_git(root.as_path(), &[], &[("GIT_DIR", named_git.as_os_str())], "GIT_DIR in the environment");
    let relative: Option<PathBuf> = assert_the_same_as_git(
        root.as_path(),
        &words(&[OsStr::new("-C"), named.as_os_str()]),
        &[("GIT_DIR", OsStr::new(".git"))],
        "a relative GIT_DIR read from the directory the chain reached",
    );
    assert_eq!(relative, Some(named_git.clone()), "a relative GIT_DIR is not read from the caller's own directory");
    let both: Option<PathBuf> = assert_the_same_as_git(
        root.as_path(),
        &words(&[OsStr::new("--git-dir"), other_git.as_os_str()]),
        &[("GIT_DIR", named_git.as_os_str())],
        "the option wins over the variable",
    );
    assert_eq!(both, Some(other_git), "the variable names a repository the command does not use");
    remove(root.as_path());
}

/// What:
///  A linked worktree is searched as the directory it owns,
///  whether the command runs in it or reaches it with `-C`.
/// Why:
///  Its index lock lives in its own Git directory,
///  and a lock there is the one a commit in that worktree meets.
#[test]
fn a_linked_worktree_is_the_directory_it_owns() {
    let root: PathBuf = fixture("lock-pass-linked");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    let _added = git(repo.as_path(), &["worktree", "add", "--quiet", linked.to_str().expect("UTF-8 fixture path"), "-b", "linked"]);
    let owned: PathBuf = repo.join(".git").join("worktrees").join("linked");
    let inside: Option<PathBuf> = assert_the_same_as_git(linked.as_path(), &[], &[], "inside the linked worktree");
    assert_eq!(inside, Some(owned.clone()));
    let reached: Option<PathBuf> = assert_the_same_as_git(root.as_path(), &words(&[OsStr::new("-C"), linked.as_os_str()]), &[], "reached with -C");
    assert_eq!(reached, Some(owned), "the linked worktree's own Git directory, not the common one");
    remove(root.as_path());
}

/// What:
///  A bare repository a command names is left alone,
///  so the pass behaves as it did before it read `-C` and `--git-dir`.
/// Why:
///  A bare repository keeps no index a commit would take,
///  and the pass searched none of the spellings that name one.
#[test]
fn a_bare_repository_a_command_names_is_left_alone() {
    let root: PathBuf = fixture("lock-pass-bare");
    let bare: PathBuf = root.join("bare.git");
    let _created = git(root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", bare.to_str().expect("UTF-8 fixture path")]);
    let is_bare: String = String::from_utf8(git(bare.as_path(), &["rev-parse", "--is-bare-repository"]).stdout).expect("Git prints text");
    assert_eq!(is_bare.trim_end(), "true", "positive control: the fixture is a bare repository Git answers for");
    assert!(
        git_answer(root.as_path(), &words(&[OsStr::new("-C"), bare.as_os_str()]), &[]).is_some(),
        "positive control: Git works in this repository"
    );
    let reached: Vec<OsString> = words(&[OsStr::new("-C"), bare.as_os_str()]);
    assert_eq!(
        git_directory_of_command_in(root.as_path(), reached.as_slice(), &[]),
        None,
        "a bare repository named by -C is left as native Git leaves it"
    );
    let named: Vec<OsString> = words(&[OsStr::new("--git-dir"), bare.as_os_str()]);
    assert_eq!(
        git_directory_of_command_in(root.as_path(), named.as_slice(), &[]),
        None,
        "a bare repository named by --git-dir is left as native Git leaves it"
    );
    remove(root.as_path());
}

/// What:
///  A directory this account cannot enter,
///  a directory that does not exist,
///  and a Git directory that does not exist answer nothing.
/// Why:
///  Such a command reaches no repository,
///  so the pass may not search one above the directory the command names,
///  and nothing may wait on the way.
#[test]
fn a_directory_the_command_cannot_reach_answers_nothing() {
    let root: PathBuf = fixture("lock-pass-unreachable");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let closed: PathBuf = repo.join("closed");
    std::fs::create_dir(closed.as_path()).expect("closed directory");
    assert!(std::fs::set_permissions(closed.as_path(), std::fs::Permissions::from_mode(0o000)).is_ok(), "the directory is closed to this account");
    let reached: Vec<OsString> = words(&[OsStr::new("-C"), closed.as_os_str()]);
    assert_eq!(git_answer(root.as_path(), reached.as_slice(), &[]), None, "positive control: Git refuses the invocation");
    let mut typed: Vec<OsString> = reached.clone();
    typed.push(OsString::from(SUBCOMMAND));
    assert_eq!(
        git_directory_of_command_in(root.as_path(), typed.as_slice(), &[]),
        None,
        "a directory the command cannot enter names nothing, not the repository above it"
    );
    assert!(std::fs::set_permissions(closed.as_path(), std::fs::Permissions::from_mode(0o755)).is_ok(), "the directory is given back");

    let absent: Vec<OsString> = words(&[OsStr::new("-C"), OsStr::new("/a-directory-this-control-never-made")]);
    assert_eq!(git_answer(root.as_path(), absent.as_slice(), &[]), None, "positive control: Git refuses an absent directory");
    let mut typed_absent: Vec<OsString> = absent.clone();
    typed_absent.push(OsString::from(SUBCOMMAND));
    assert_eq!(git_directory_of_command_in(root.as_path(), typed_absent.as_slice(), &[]), None);
    let named_absent: Vec<OsString> = words(&[OsStr::new("--git-dir"), root.join("absent.git").as_os_str(), OsStr::new(SUBCOMMAND)]);
    assert_eq!(git_directory_of_command_in(root.as_path(), named_absent.as_slice(), &[]), None, "a Git directory that does not exist names nothing");
    remove(root.as_path());
}

/// What:
///  The wrapper's own controls name no path,
///  so a control written before the `-C` chain is removed and Git's own options are read as Git reads them.
/// Why:
///  A control read as a path would make the pass search a directory the command never uses.
#[test]
fn a_wrapper_control_is_never_read_as_a_path() {
    let root: PathBuf = fixture("lock-pass-control");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let arguments: Vec<OsString> = words(&[OsStr::new("--no-enforce-require-root"), OsStr::new("-C"), repo.as_os_str(), OsStr::new(SUBCOMMAND)]);
    let found: Option<PathBuf> = git_directory_of_command_in(root.as_path(), arguments.as_slice(), &[]);
    assert_eq!(found, Some(repo.join(".git")));
    remove(root.as_path());
}
