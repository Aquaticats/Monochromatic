//! Controls for the `git worktree add` plan: the accepted forms, every refusal, and a real Git 2.56.0 run of each
//!  accepted form that registers the destination the plan names.

use super::*;
use crate::command_test_support::os_arguments;
use crate::test_support::{fixture, git, remove, repository};
use std::process::Output;

/// The arguments of an invocation as the plan reads them: the global prefix, `worktree`, `add` and the rest.
fn plan_of(values: &[&str], subcommand_index: usize, start: &Path) -> Option<AddPlan> {
    return parse_add_plan(os_arguments(values).as_slice(), subcommand_index, start);
}

/// The forms of the table name the destination and the place directly after `add` for the insertion.
#[test]
fn an_accepted_form_names_its_destination_and_the_insertion_point() {
    let root: PathBuf = fixture("add-plan-accepted");
    let canonical: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    for (values, index, operand) in [
        (vec!["worktree", "add", "topic"], 0, "topic"),
        (vec!["worktree", "add", "-b", "feature", "topic"], 0, "topic"),
        (vec!["worktree", "add", "topic", "-b", "feature", "main"], 0, "topic"),
        (vec!["worktree", "add", "--detach", "--quiet", "topic", "main"], 0, "topic"),
        (vec!["worktree", "add", "-d", "-q", "--lock", "--reason", "why", "topic"], 0, "topic"),
        (vec!["worktree", "add", "--reason=why", "--checkout", "topic"], 0, "topic"),
        (vec!["worktree", "add", "--", "-odd"], 0, "-odd"),
        (vec!["--no-pager", "worktree", "add", "topic"], 1, "topic"),
    ] {
        let plan: AddPlan = plan_of(values.as_slice(), index, root.as_path()).unwrap_or_else(|| { panic!("{values:?} is accepted"); });
        assert_eq!(plan.destination, canonical.join(operand), "{values:?}");
        assert_eq!(plan.insert_at, index + 2, "{values:?}");
        assert_eq!(os_arguments(values.as_slice())[plan.insert_at - 1], "add", "{values:?}");
        let quiet: bool = values.contains(&"--quiet") || values.contains(&"-q");
        assert_eq!(plan.quiet, quiet, "{values:?}");
    }
    remove(root.as_path());
}

/// Every form outside the table is a plain Git run: unknown and bundled options, the options that change the
///  checkout, a wrong operand count, and a word other than `add`.
#[test]
fn a_form_outside_the_table_is_declined() {
    let root: PathBuf = fixture("add-plan-declined");
    for values in [
        vec!["worktree", "add"],
        vec!["worktree", "add", "a", "b", "c"],
        vec!["worktree", "add", "--no-checkout", "topic"],
        vec!["worktree", "add", "--force", "topic"],
        vec!["worktree", "add", "-f", "topic"],
        vec!["worktree", "add", "-B", "feature", "topic"],
        vec!["worktree", "add", "--orphan", "topic"],
        vec!["worktree", "add", "--track", "topic"],
        vec!["worktree", "add", "--detac", "topic"],
        vec!["worktree", "add", "-dq", "topic"],
        vec!["worktree", "add", "-bfeature", "topic"],
        vec!["worktree", "add", "-b"],
        vec!["worktree", "add", "topic", "-b"],
        vec!["worktree", "add", "-b", "feature"],
        vec!["worktree", "move", "topic", "other"],
        vec!["worktree", "ad", "topic"],
        vec!["worktree", "--force", "add", "topic"],
        vec!["worktree", "add", "."],
        vec!["worktree", "add", ".."],
        vec!["worktree", "add", "missing-parent/topic"],
        vec!["add", "topic"],
    ] {
        assert_eq!(plan_of(values.as_slice(), 0, root.as_path()), None, "{values:?}");
    }
    remove(root.as_path());
}

/// A global option that can change how the ladder's own Git runs read the repository declines the start directory;
///  `-C` moves it and a `..` component in it is declined.
#[test]
fn the_global_prefix_decides_the_start_directory() {
    let root: PathBuf = fixture("add-plan-prefix");
    std::fs::create_dir(root.join("inner")).expect("inner");
    let canonical: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    let moved: PathBuf = start_directory(os_arguments(&["-C", "inner", "--no-pager"]).as_slice(), root.as_path()).expect("moved");
    let plan: AddPlan = parse_add_plan(os_arguments(&["worktree", "add", "topic"]).as_slice(), 0, moved.as_path()).expect("plan");
    assert_eq!(plan.destination, canonical.join("inner").join("topic"));
    let chained: PathBuf = start_directory(os_arguments(&["-C", "inner", "-C", "."]).as_slice(), root.as_path()).expect("chained");
    assert_eq!(chained, root.join("inner").join("."));
    for prefix in [
        vec!["-c", "core.autocrlf=true"],
        vec!["--git-dir", "x"],
        vec!["--work-tree", "x"],
        vec!["--config-env", "a=B"],
        vec!["-C", "inner/../inner"],
        vec!["-C"],
        vec!["--bare"],
    ] {
        assert_eq!(start_directory(os_arguments(prefix.as_slice()).as_slice(), root.as_path()), None, "{prefix:?}");
    }
    assert_eq!(start_directory(&[], root.as_path()), Some(root.clone()));
    remove(root.as_path());
}

/// The destination of a symbolic link parent is the link's target, as Git records it.
#[test]
fn a_symbolic_link_parent_is_resolved_like_git_does() {
    let root: PathBuf = fixture("add-plan-link");
    std::fs::create_dir(root.join("real")).expect("real");
    std::os::unix::fs::symlink(root.join("real"), root.join("link")).expect("link");
    let plan = plan_of(&["worktree", "add", "link/topic"], 0, root.as_path()).expect("plan");
    assert_eq!(plan.destination, std::fs::canonicalize(root.join("real")).expect("canonical").join("topic"));
    remove(root.as_path());
}

/// Positive control of the table against real Git 2.56.0: each accepted form creates a worktree at the planned
///  destination, and `worktree list` records the path the plan computed.
#[test]
fn real_git_registers_the_planned_destination_for_each_accepted_form() {
    let root: PathBuf = fixture("add-plan-real-git");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir(root.join("space dir")).expect("directory");
    std::os::unix::fs::symlink(root.join("space dir"), root.join("via-link")).expect("link");
    for (position, values) in [
        vec!["worktree", "add", "../one"],
        vec!["worktree", "add", "-b", "feature", "../two", "main"],
        vec!["worktree", "add", "--detach", "--quiet", "../three", "main"],
        vec!["worktree", "add", "../four", "-b", "other"],
        vec!["worktree", "add", "--lock", "--reason", "plan", "../five", "-b", "locked-one"],
        vec!["worktree", "add", "--", "../six"],
        vec!["worktree", "add", "../via-link/seven", "-b", "seventh"],
    ]
    .into_iter()
    .enumerate()
    {
        let plan: AddPlan = plan_of(values.as_slice(), 0, repo.as_path()).unwrap_or_else(|| { panic!("{values:?} is accepted"); });
        let before: String = String::from_utf8_lossy(&git(repo.as_path(), &["worktree", "list", "--porcelain"]).stdout).into_owned();
        assert!(!before.contains(plan.destination.to_string_lossy().as_ref()), "{position}: {values:?}");
        git(repo.as_path(), values.as_slice());
        let listing: Output = git(repo.as_path(), &["worktree", "list", "--porcelain"]);
        let text: String = String::from_utf8_lossy(&listing.stdout).into_owned();
        assert!(text.contains(format!("worktree {}\n", plan.destination.display()).as_str()), "{values:?}: {text}");
    }
    remove(root.as_path());
}
