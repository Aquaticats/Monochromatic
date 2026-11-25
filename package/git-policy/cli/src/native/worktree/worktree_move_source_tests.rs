//! Controls for the source of `git worktree move`:
//! the plain forms,
//! the option and separator shapes,
//! and every form the reading leaves to Git.

use super::*;
use crate::command_test_support::os_arguments;
use crate::test_support::{fixture, remove};

/// What:
///  The plain form names the first operand and leaves the destination to Git.
/// Why:
///  A person may name the worktree by its whole path or by a last component,
///  and every one of those spellings is the same source.
#[test]
fn the_plain_form_names_the_first_operand() {
    let root: PathBuf = fixture("move-source");
    std::fs::create_dir(root.join("slot")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("slot")).expect("canonical");
    for values in [
        vec!["worktree", "move", "slot", "../elsewhere"],
        vec!["worktree", "move", "--", "slot", "../elsewhere"],
        vec!["worktree", "move", "-f", "slot", "../elsewhere"],
        vec!["worktree", "move", "--force", "--", "slot", "../elsewhere"],
        vec!["worktree", "move", "--relative-paths", "slot", "../elsewhere"],
    ] {
        let found = parse_move_source(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None);
        assert_eq!(found, Some(MoveSource { source: canonical.clone(), operand: OsString::from("slot") }), "{values:?}");
    }
    remove(root.as_path());
}

/// What:
///  A form without the destination,
///  another subcommand,
///  and an operand that names no directory are Git's.
/// Why:
///  The refusal exists for one move,
///  and a form Git will refuse itself must not be read as another command's.
#[test]
fn every_other_form_is_left_to_git() {
    let root: PathBuf = fixture("move-source-refused");
    std::fs::create_dir(root.join("slot")).expect("directory");
    for values in [
        vec!["worktree", "move"],
        vec!["worktree", "move", "slot"],
        vec!["worktree", "move", "-f"],
        vec!["worktree", "move", "-f", "--"],
        vec!["worktree", "move", "", "../elsewhere"],
        vec!["worktree", "move", "absent", "../elsewhere"],
        vec!["worktree", "add", "slot", "HEAD"],
        vec!["worktree", "remove", "slot"],
        vec!["worktree", "prune"],
        vec!["commit", "move", "slot", "../elsewhere"],
    ] {
        assert_eq!(parse_move_source(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None), None, "{values:?}");
    }
    remove(root.as_path());
}

/// What:
///  A last component that matches one registered worktree names that worktree,
///  which is the name Git itself picks.
/// Why:
///  Git matches the typed operand against the registered paths before it reads
///  it as a path,
///  and the refusal must read the slot the command renames rather than the one
///  a path join would name.
#[test]
fn a_registered_last_component_names_the_registration() {
    let root: PathBuf = fixture("move-source-registered");
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir(repository.as_path()).expect("repository");
    crate::test_support::git(repository.as_path(), &["init", "--quiet"]);
    crate::test_support::git(repository.as_path(), &["commit", "--quiet", "--allow-empty", "--message", "base"]);
    let slot: PathBuf = root.join("elsewhere-topic");
    crate::test_support::git(repository.as_path(), &["worktree", "add", "--quiet", "--no-checkout", slot.to_str().expect("utf-8"), "HEAD"]);
    let common: PathBuf = repository.join(".git");
    let found = parse_move_source(os_arguments(&["worktree", "move", "elsewhere-topic", "../moved"]).as_slice(), 0, root.as_path(), Some(common.as_path()));
    assert_eq!(found, Some(MoveSource { source: std::fs::canonicalize(slot.as_path()).expect("canonical"), operand: OsString::from("elsewhere-topic") }));
    remove(root.as_path());
}
