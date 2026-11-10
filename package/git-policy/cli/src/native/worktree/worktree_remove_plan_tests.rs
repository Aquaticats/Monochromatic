//! Controls for the plan of `git worktree remove`:
//! the one accepted shape and every declined one.

use super::*;
use crate::command_test_support::os_arguments;
use crate::test_support::{fixture, remove};

/// The accepted forms name exactly one existing directory and at most one `-f`/`--force`;
/// every other option and every other count is declined.
#[test]
fn the_plan_names_one_directory_and_no_option() {
    let root: PathBuf = fixture("remove-plan");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    for values in [vec!["worktree", "remove", "wt"], vec!["worktree", "remove", "--", "wt"], vec!["worktree", "remove", "./wt/"]] {
        let plan = parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path());
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from(values[values.len() - 1]), force: false }), "{values:?}");
    }
    for values in [
        vec!["worktree", "remove"],
        vec!["worktree", "remove", "wt", "extra"],
        vec!["worktree", "remove", "absent"],
        vec!["worktree", "add", "wt"],
        vec!["worktree", "prune"],
    ] {
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path()), None, "{values:?}");
    }
    remove(root.as_path());
}

/// What:
///  A forced removal names the same directory and is a plan,
///  so the wrapper destroys the worktree's subvolume instead of leaving Git to
///  unlink every entry of it.
/// Why:
///   Git deletes a dirty worktree only with `-f`/`--force` (`remove_worktree`,
///  `if (!force) check_clean_worktree(...)`),
///  and those worktrees are the ones with the largest ignored trees,
///  so a forced removal is today the slowest one while every other command waits on the volume.
#[test]
fn a_forced_removal_names_the_directory_and_the_force() {
    let root: PathBuf = fixture("remove-plan-forced");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    for values in [vec!["worktree", "remove", "-f", "wt"], vec!["worktree", "remove", "--force", "wt"], vec!["worktree", "remove", "--force", "--", "wt"], vec!["worktree", "remove", "-f", "--", "wt"]] {
        let plan = parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path());
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from("wt"), force: true }), "{values:?}");
    }
    // A second force is Git's: it is what removes a locked worktree, and a spelling the wrapper does not read is not one
    // force.
    for values in [vec!["worktree", "remove", "-f", "-f", "wt"], vec!["worktree", "remove", "-ff", "wt"], vec!["worktree", "remove", "--forc", "wt"], vec!["worktree", "remove", "-fq", "wt"], vec!["worktree", "remove", "-f"]] {
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path()), None, "{values:?}");
    }
    remove(root.as_path());
}

/// An empty operand names no directory:
/// joined to the start directory it would name the current one,
/// and Git answers that
///  it is not a working tree.
#[test]
fn an_empty_operand_names_no_directory() {
    let root: PathBuf = fixture("remove-plan-empty");
    for values in [vec!["worktree", "remove", ""], vec!["worktree", "remove", "--", ""]] {
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path()), None, "{values:?}");
    }
    // The current directory spelled out is a name, and Git removes it.
    let plan = parse_remove_plan(os_arguments(&["worktree", "remove", "."]).as_slice(), 0, root.as_path());
    assert_eq!(plan.map(|found| { return found.target; }), Some(std::fs::canonicalize(root.as_path()).expect("canonical")));
    remove(root.as_path());
}
