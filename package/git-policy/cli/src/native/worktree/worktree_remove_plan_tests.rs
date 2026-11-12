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
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from(values[values.len() - 1]), force: ForceAsked::No }), "{values:?}");
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
///  A removal forced once and one forced twice are both plans,
///  and the count is what the plan carries.
/// Why:
///   A second force is what a person types when the first removal failed,
///  which is exactly the removal this wrapper exists to finish,
///  and Git reads the count twice over:
/// any force skips its cleanliness proof
///  (`remove_worktree`,
/// `if (!force) check_clean_worktree(...)`),
///  and a force given twice is also the permission to unlock a worktree.
///  A form the plan did not read was the whole removal handed to Git's own walk,
///  and a folder a writer defeated then stayed behind with nothing pointing at it.
#[test]
fn a_forced_removal_names_the_directory_and_the_force() {
    let root: PathBuf = fixture("remove-plan-forced");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    for values in [vec!["worktree", "remove", "-f", "wt"], vec!["worktree", "remove", "--force", "wt"], vec!["worktree", "remove", "--force", "--", "wt"], vec!["worktree", "remove", "-f", "--", "wt"]] {
        let plan = parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None);
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from("wt"), force: ForceAsked::Once }), "{values:?}");
    }
    // Every spelling Git's own option table accepts for the force of this command: the short form in a cluster of `f`s, and any
    // abbreviation of the long name, once or twice.
    let twice: [(&[&str], ForceAsked); 7] = [
        (&["worktree", "remove", "-f", "-f", "wt"], ForceAsked::Twice),
        (&["worktree", "remove", "-ff", "wt"], ForceAsked::Twice),
        (&["worktree", "remove", "--force", "--force", "wt"], ForceAsked::Twice),
        (&["worktree", "remove", "-f", "--force", "wt"], ForceAsked::Twice),
        (&["worktree", "remove", "-ff", "--", "wt"], ForceAsked::Twice),
        (&["worktree", "remove", "--forc", "wt"], ForceAsked::Once),
        (&["worktree", "remove", "--f", "wt"], ForceAsked::Once),
    ];
    for (values, force) in twice {
        let plan = parse_remove_plan(os_arguments(values).as_slice(), 0, root.as_path(), None);
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from("wt"), force }), "{values:?}");
    }
    // A third force, an option whose effect this plan would not know, a value `--force` does not take, and an operand that
    // never comes are Git's own answers.
    for values in [
        vec!["worktree", "remove", "-f", "-f", "-f", "wt"],
        vec!["worktree", "remove", "-fff", "wt"],
        vec!["worktree", "remove", "-fq", "wt"],
        vec!["worktree", "remove", "--force=1", "wt"],
        vec!["worktree", "remove", "-f"],
        vec!["worktree", "remove", "-f", "wt", "extra"],
    ] {
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
/// The comparison is byte for byte,
/// as Git's own is unless `core.ignorecase` folds the case;
/// a name that differs only by case is therefore Git's own decision and is left to Git.
/// A name two registrations end in is Git's ambiguous case and is left to Git,
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
        (vec!["worktree", "remove", "--", "topic"], "topic"),
    ] {
        let plan = parse_remove_plan(os_arguments(arguments.as_slice()).as_slice(), 0, repo.as_path(), Some(common.as_path()));
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from(typed), force: ForceAsked::No }), "{arguments:?}");
    }
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "TOPIC"]).as_slice(), 0, repo.as_path(), Some(common.as_path())),
        None,
        "Git's byte comparison picks no worktree for a name that differs only by case, and no directory stands at it"
    );
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "absent"]).as_slice(), 0, repo.as_path(), Some(common.as_path())),
        None,
        "no registration ends in that name and no directory stands at it"
    );
    // The force the person typed reaches a name the registrations picked too, because the plan carries it whichever way it read
    // the directory.
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "-f", "topic"]).as_slice(), 0, repo.as_path(), Some(common.as_path())),
        Some(RemovePlan { target: canonical.clone(), operand: OsString::from("topic"), force: ForceAsked::Once }),
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
