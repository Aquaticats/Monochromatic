//! Controls for the clone route's own Git runner:
//!  a step's variables reach the Git it starts,
//!  its status is kept,
//!  and a step of another shape is refused.

use super::RouteGit;
use crate::invocation::forwarding::NoChildRecorded;
use crate::test_support::{REAL_GIT, fixture, git, remove};
use crate::worktree_copy::{GitRunner, GitStep};
use std::ffi::OsString;
use std::path::PathBuf;

/// A repository the runner can name.
fn a_repository(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    return root;
}

#[test]
fn a_steps_own_variables_reach_the_git_it_starts() {
    let root: PathBuf = a_repository("route-git-env");
    let real: PathBuf = PathBuf::from(REAL_GIT);
    let mut runner: RouteGit<'_> = RouteGit { real_git: real.as_path(), overlay: &[] };
    let arguments: [OsString; 2] = [OsString::from("rev-parse"), OsString::from("--git-dir")];
    let environment: [(OsString, OsString); 1] = [(OsString::from("GIT_DIR"), root.join(".git").into_os_string())];
    let step: GitStep<'_> = GitStep::Other { arguments: arguments.as_slice(), environment: environment.as_slice() };
    let status = runner.run(&step, &mut NoChildRecorded).expect("the run starts");
    assert!(status.succeeded(), "the step's own GIT_DIR reached Git: {status:?}");
    remove(root.as_path());
}

#[test]
fn a_git_that_answers_a_status_keeps_it() {
    let root: PathBuf = a_repository("route-git-status");
    let real: PathBuf = PathBuf::from(REAL_GIT);
    let mut runner: RouteGit<'_> = RouteGit { real_git: real.as_path(), overlay: &[] };
    let arguments: [OsString; 5] = [OsString::from("-C"), root.as_os_str().to_os_string(), OsString::from("rev-parse"), OsString::from("--verify"), OsString::from("HEAD~5")];
    let step: GitStep<'_> = GitStep::Other { arguments: arguments.as_slice(), environment: &[] };
    let status = runner.run(&step, &mut NoChildRecorded).expect("the run starts");
    // The same command started by this test, so the runner's answer is compared
    // with Git's own rather than with a status this test decided.
    let own: std::process::ExitStatus = std::process::Command::new(REAL_GIT).args(arguments.iter()).status().expect("the control's Git starts");
    assert!(!own.success(), "the control's command fails");
    assert_eq!(status.code, own.code(), "a Git that cannot answer keeps its own status");
    remove(root.as_path());
}

#[test]
fn a_step_the_restore_does_not_run_is_refused() {
    let real: PathBuf = PathBuf::from(REAL_GIT);
    let mut runner: RouteGit<'_> = RouteGit { real_git: real.as_path(), overlay: &[] };
    let refusal = runner.run(&GitStep::Person, &mut NoChildRecorded).expect_err("the person's own step is not the runner's");
    assert!(refusal.0.contains("only its own Git commands"), "{}", refusal.0);
}
