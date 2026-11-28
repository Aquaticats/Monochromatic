//! Controls for the plan of `git worktree remove`:
//! the one accepted shape and every declined one.

use super::*;
use crate::command_test_support::os_arguments;
use crate::test_support::{
    fixture,
    remove,
};

/// The accepted forms name exactly one existing directory and at most one `-f`/`--force`;
/// every other option and every other count is declined.
#[test]
fn the_plan_names_one_directory_and_no_option() {
    let root: PathBuf = fixture("remove-plan");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    for values in [
        vec!["worktree", "remove", "wt"],
        vec!["worktree", "remove", "--", "wt"],
        vec!["worktree", "remove", "./wt/"],
    ] {
        let plan = parse_remove_plan(
            os_arguments(values.as_slice()).as_slice(),
            0,
            root.as_path(),
            None,
        );
        assert_eq!(
            plan,
            Some(RemovePlan {
                target: canonical.clone(),
                operand: OsString::from(values[values.len() - 1]),
                force: false
            }),
            "{values:?}"
        );
    }
    for values in [
        vec!["worktree", "remove"],
        vec!["worktree", "remove", "wt", "extra"],
        vec!["worktree", "remove", "absent"],
        vec!["worktree", "add", "wt"],
        vec!["worktree", "prune"],
    ] {
        assert_eq!(
            parse_remove_plan(
                os_arguments(values.as_slice()).as_slice(),
                0,
                root.as_path(),
                None
            ),
            None,
            "{values:?}"
        );
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
    for values in [
        vec!["worktree", "remove", "-f", "wt"],
        vec!["worktree", "remove", "--force", "wt"],
        vec!["worktree", "remove", "--force", "--", "wt"],
        vec!["worktree", "remove", "-f", "--", "wt"],
    ] {
        let plan = parse_remove_plan(
            os_arguments(values.as_slice()).as_slice(),
            0,
            root.as_path(),
            None,
        );
        assert_eq!(
            plan,
            Some(RemovePlan {
                target: canonical.clone(),
                operand: OsString::from("wt"),
                force: true
            }),
            "{values:?}"
        );
    }
    // A second force is Git's: it is what removes a locked worktree, and a spelling the wrapper does not read is not one
    // force.
    for values in [
        vec!["worktree", "remove", "-f", "-f", "wt"],
        vec!["worktree", "remove", "-ff", "wt"],
        vec!["worktree", "remove", "--forc", "wt"],
        vec!["worktree", "remove", "-fq", "wt"],
        vec!["worktree", "remove", "-f"],
    ] {
        assert_eq!(
            parse_remove_plan(
                os_arguments(values.as_slice()).as_slice(),
                0,
                root.as_path(),
                None
            ),
            None,
            "{values:?}"
        );
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
    for values in [
        vec!["worktree", "remove", ""],
        vec!["worktree", "remove", "--", ""],
    ] {
        assert_eq!(
            parse_remove_plan(
                os_arguments(values.as_slice()).as_slice(),
                0,
                root.as_path(),
                None
            ),
            None,
            "{values:?}"
        );
    }
    // The current directory spelled out is a name, and Git removes it.
    let plan = parse_remove_plan(
        os_arguments(&["worktree", "remove", "."]).as_slice(),
        0,
        root.as_path(),
        None,
    );
    assert_eq!(
        plan.map(|found| {
            return found.target;
        }),
        Some(std::fs::canonicalize(root.as_path()).expect("canonical"))
    );
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
    std::fs::write(
        common.join("worktrees/topic/gitdir"),
        format!("{}/.git\n", topic.display()),
    )
    .expect("gitdir");
    let canonical: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical");
    for (arguments, typed) in [
        (vec!["worktree", "remove", "topic"], "topic"),
        (vec!["worktree", "remove", "--", "topic"], "topic"),
    ] {
        let plan = parse_remove_plan(
            os_arguments(arguments.as_slice()).as_slice(),
            0,
            repo.as_path(),
            Some(common.as_path()),
        );
        assert_eq!(
            plan,
            Some(RemovePlan {
                target: canonical.clone(),
                operand: OsString::from(typed),
                force: false
            }),
            "{arguments:?}"
        );
    }
    assert_eq!(
        parse_remove_plan(
            os_arguments(&["worktree", "remove", "TOPIC"]).as_slice(),
            0,
            repo.as_path(),
            Some(common.as_path())
        ),
        None,
        "Git's byte comparison picks no worktree for a name that differs only by case, and no directory stands at it"
    );
    assert_eq!(
        parse_remove_plan(
            os_arguments(&["worktree", "remove", "absent"]).as_slice(),
            0,
            repo.as_path(),
            Some(common.as_path())
        ),
        None,
        "no registration ends in that name and no directory stands at it"
    );
    // The force the person typed reaches a name the registrations picked too, because the plan carries it whichever way it read
    // the directory.
    assert_eq!(
        parse_remove_plan(
            os_arguments(&["worktree", "remove", "-f", "topic"]).as_slice(),
            0,
            repo.as_path(),
            Some(common.as_path())
        ),
        Some(RemovePlan {
            target: canonical.clone(),
            operand: OsString::from("topic"),
            force: true
        }),
        "a forced removal of a worktree the registrations picked"
    );
    let other: PathBuf = root.join("other/topic");
    std::fs::create_dir_all(common.join("worktrees/other")).expect("second registration");
    std::fs::create_dir_all(other.as_path()).expect("second worktree");
    std::fs::write(
        common.join("worktrees/other/gitdir"),
        format!("{}/.git\n", other.display()),
    )
    .expect("gitdir");
    assert_eq!(
        parse_remove_plan(
            os_arguments(&["worktree", "remove", "topic"]).as_slice(),
            0,
            repo.as_path(),
            Some(common.as_path())
        ),
        None,
        "Git's suffix match is unique or nothing, and no directory here stands at the name"
    );
    remove(root.as_path());
}

/// What:
///  Every spelling of `git worktree remove` that names one worktree names it in
///  the refusal's reading too,
///  whatever options stand before the operand.
/// Why:
///  The plan serves only the plain form,
///  and a slot a whole clone is bound to is refused for every spelling:
///  Git's own lock hint steers a person to `-f -f`,
///  and `SPEC.md` exempts none.
#[test]
fn every_spelling_that_names_one_worktree_names_it_for_the_refusal() {
    let root: PathBuf = fixture("removal-target-spellings");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    for values in [
        vec!["worktree", "remove", "wt"],
        vec!["worktree", "remove", "-f", "wt"],
        vec!["worktree", "remove", "-f", "-f", "wt"],
        vec!["worktree", "remove", "-ff", "wt"],
        vec!["worktree", "remove", "--force", "wt"],
        vec!["worktree", "remove", "--force", "--force", "wt"],
        vec!["worktree", "remove", "--force", "-f", "wt"],
        vec!["worktree", "remove", "--forc", "wt"],
        vec!["worktree", "remove", "-f", "-f", "--", "wt"],
    ] {
        assert_eq!(
            parse_removal_target(
                os_arguments(values.as_slice()).as_slice(),
                0,
                Some(root.as_path()),
                None
            ),
            Some(canonical.clone()),
            "{values:?}"
        );
    }
    // A command that names two worktrees, one that names none, one that names no directory, and every other subcommand are Git's
    // own answers, and the refusal reads none of them.
    for values in [
        vec!["worktree", "remove", "wt", "extra"],
        vec!["worktree", "remove"],
        vec!["worktree", "remove", "-f"],
        vec!["worktree", "remove", "absent"],
        vec!["worktree", "remove", ""],
        vec!["worktree", "add", "wt"],
        vec!["worktree", "move", "wt", "../elsewhere"],
        vec!["worktree", "prune"],
    ] {
        assert_eq!(
            parse_removal_target(
                os_arguments(values.as_slice()).as_slice(),
                0,
                Some(root.as_path()),
                None
            ),
            None,
            "{values:?}"
        );
    }
    remove(root.as_path());
}

/// What:
///  A name that ends one registered worktree path names that worktree for the
///  refusal,
///  as it does for the plan.
/// Why:
///  Git matches the typed word against the registered paths before it reads it
///  as a path,
///  so the refusal must name the worktree Git would act on.
#[test]
fn a_registered_last_component_names_the_worktree_for_the_refusal() {
    let root: PathBuf = fixture("removal-target-registered");
    let repo: PathBuf = root.join("repo");
    let common: PathBuf = repo.join(".git");
    let topic: PathBuf = root.join("topic");
    std::fs::create_dir_all(common.join("worktrees/topic")).expect("registration");
    std::fs::create_dir_all(topic.as_path()).expect("worktree");
    std::fs::write(
        common.join("worktrees/topic/gitdir"),
        format!("{}/.git\n", topic.display()),
    )
    .expect("gitdir");
    let canonical: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical");
    assert_eq!(
        parse_removal_target(
            os_arguments(&["worktree", "remove", "-f", "-f", "topic"]).as_slice(),
            0,
            Some(repo.as_path()),
            Some(common.as_path())
        ),
        Some(canonical),
        "the double force of a worktree the registrations picked"
    );
    remove(root.as_path());
}

/// What:
///  Every spelling Git accepts counts the way Git counts it.
/// Why:
///  The measured gradation of Git 2.56.1 on this machine is:
///  `-f`,
///  `--force`,
///  `--forc` and `--f` pass a dirty worktree and leave a
///  locked one refused,
///  while `-f -f`,
///  `-ff` and `--force --force` pass both,
///  and `--no-force` sets the count back to zero
///  (`builtin/worktree.c:1450` and `:1464` at Git `v2.56.1`,
///  and the transcript in this round's report).
///  The fixture is a table of the words and the count Git reads from them.
#[test]
fn every_spelling_of_git_s_force_counts_the_way_git_counts_it() {
    let cases: [(&[&str], usize); 14] = [
        (&[], 0),
        (&["one"], 0),
        (&["-f", "one"], 1),
        (&["--force", "one"], 1),
        (&["--forc", "one"], 1),
        (&["--f", "one"], 1),
        (&["one", "-f"], 1),
        (&["one", "-f", "-f"], 2),
        (&["-ff", "one"], 2),
        (&["--force", "--force", "one"], 2),
        (&["--no-force", "one"], 0),
        (&["--force", "--no-force", "one"], 0),
        (&["-f", "-f", "--no-f", "one"], 0),
        (&["-f", "--", "-f"], 1),
    ];
    for (words, wanted) in cases {
        assert_eq!(
            count_of_the_force(os_arguments(words).as_slice()),
            wanted,
            "{words:?}"
        );
    }
}

/// What:
///  A command carrying an option word Git does not accept for this command
///  counts no force at all.
/// Why:
///  Git answers such a word with its usage and no removal at all,
///  and this wrapper answers the spelling before Git would,
///  so no word of a refused command may pass a refusal it would not have passed
///  (`parse-options.c:426` to `:440` at Git `v2.56.1`).
#[test]
fn a_command_carrying_an_option_git_refuses_counts_no_force() {
    for words in [
        vec!["-fq", "one"],
        vec!["-qf", "one"],
        vec!["-f", "-q"],
        vec!["-x", "one"],
    ] {
        assert_eq!(
            count_of_the_force(os_arguments(words.as_slice()).as_slice()),
            0,
            "{words:?}"
        );
    }
}
