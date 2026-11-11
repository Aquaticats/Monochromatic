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
        let plan = parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None);
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from(values[values.len() - 1]), force: false }), "{values:?}");
    }
    for values in [
        vec!["worktree", "remove"],
        vec!["worktree", "remove", "wt", "extra"],
        vec!["worktree", "remove", "absent"],
        vec!["worktree", "add", "wt"],
        vec!["worktree", "prune"],
    ] {
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None), None, "{values:?}");
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
///  so before this the forced removal,
/// which the wrapper's own reports tell a person to run,
///  always left every entry of the tree to real Git to unlink.
/// How much a worktree of that shape holds was not measured here;
/// the page
///  `doc/planning/cli-git-worktree-removal-cost-and-disk-time.md` is where it is measured.
#[test]
fn a_forced_removal_names_the_directory_and_the_force() {
    let root: PathBuf = fixture("remove-plan-forced");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    for values in [vec!["worktree", "remove", "-f", "wt"], vec!["worktree", "remove", "--force", "wt"], vec!["worktree", "remove", "--force", "--", "wt"], vec!["worktree", "remove", "-f", "--", "wt"]] {
        let plan = parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None);
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from("wt"), force: true }), "{values:?}");
    }
    // A second force is Git's: it is what removes a locked worktree, and a spelling the wrapper does not read is not one
    // force.
    for values in [vec!["worktree", "remove", "-f", "-f", "wt"], vec!["worktree", "remove", "-ff", "wt"], vec!["worktree", "remove", "--forc", "wt"], vec!["worktree", "remove", "-fq", "wt"], vec!["worktree", "remove", "-f"]] {
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None), None, "{values:?}");
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
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None), None, "{values:?}");
    }
    // The current directory spelled out is a name, and Git removes it.
    let plan = parse_remove_plan(os_arguments(&["worktree", "remove", "."]).as_slice(), 0, root.as_path(), None);
    assert_eq!(plan.map(|found| { return found.target; }), Some(std::fs::canonicalize(root.as_path()).expect("canonical")));
    remove(root.as_path());
}

/// A name that ends one registered worktree path at a directory boundary names that worktree,
/// because Git matches the last path components before it reads the name as a path.
/// ASCII case is ignored,
/// as Git's match does;
/// a name two registrations end in is Git's ambiguous case and is left to Git,
/// and so is a name no registration ends in.
#[test]
fn a_name_git_picks_by_its_last_component_names_the_registered_worktree() {
    let root: PathBuf = fixture("remove-plan-suffix");
    let repo: PathBuf = root.join("repo");
    let common: PathBuf = repo.join(".git");
    let topic: PathBuf = root.join("topic");
    std::fs::create_dir_all(common.join("worktrees/topic")).expect("registration");
    std::fs::create_dir_all(topic.as_path()).expect("worktree");
    std::fs::write(common.join("worktrees/topic/gitdir"), format!("{}/.git\n", topic.display())).expect("gitdir");
    let canonical: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical");
    for (arguments, typed) in [
        (vec!["worktree", "remove", "topic"], "topic"),
        (vec!["worktree", "remove", "TOPIC"], "TOPIC"),
        (vec!["worktree", "remove", "--", "topic"], "topic"),
    ] {
        let plan = parse_remove_plan(os_arguments(arguments.as_slice()).as_slice(), 0, repo.as_path(), Some(common.as_path()));
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from(typed), force: false }), "{arguments:?}");
    }
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "absent"]).as_slice(), 0, repo.as_path(), Some(common.as_path())),
        None,
        "no registration ends in that name and no directory stands at it"
    );
    // The force the person typed reaches a name the registrations picked too, because the plan carries it whichever way it read
    // the directory.
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "-f", "topic"]).as_slice(), 0, repo.as_path(), Some(common.as_path())),
        Some(RemovePlan { target: canonical.clone(), operand: OsString::from("topic"), force: true }),
        "a forced removal of a worktree the registrations picked"
    );
    let other: PathBuf = root.join("other/topic");
    std::fs::create_dir_all(common.join("worktrees/other")).expect("second registration");
    std::fs::create_dir_all(other.as_path()).expect("second worktree");
    std::fs::write(common.join("worktrees/other/gitdir"), format!("{}/.git\n", other.display())).expect("gitdir");
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "topic"]).as_slice(), 0, repo.as_path(), Some(common.as_path())),
        None,
        "Git's suffix match is unique or nothing, and no directory here stands at the name"
    );
    remove(root.as_path());
}
