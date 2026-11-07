//! Controls for the plan of `git worktree remove`:
//! the one accepted shape and every declined one.

use super::*;
use crate::command_test_support::os_arguments;
use crate::test_support::{fixture, remove};

/// The accepted forms name exactly one existing directory;
/// every option and every other count is declined.
#[test]
fn the_plan_names_one_directory_and_no_option() {
    let root: PathBuf = fixture("remove-plan");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    for values in [vec!["worktree", "remove", "wt"], vec!["worktree", "remove", "--", "wt"], vec!["worktree", "remove", "./wt/"]] {
        let plan = parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path());
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from(values[values.len() - 1]) }), "{values:?}");
    }
    for values in [
        vec!["worktree", "remove"],
        vec!["worktree", "remove", "--force", "wt"],
        vec!["worktree", "remove", "-f", "wt"],
        vec!["worktree", "remove", "wt", "extra"],
        vec!["worktree", "remove", "absent"],
        vec!["worktree", "add", "wt"],
        vec!["worktree", "prune"],
    ] {
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
