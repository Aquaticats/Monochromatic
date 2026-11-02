//! Controls for the steps after real Git returned: the snapshot rung serves a clean source with or without a tree
//!  difference and prints Git's line; every refusal destroys the stage and completes the worktree with `reset --hard`;
//!  `--quiet` is honored on both; and no step runs a hook.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::test_support::{REAL_GIT, git, git_context, git_text, remove};
use crate::worktree_copy::GitStep;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_ladder_run::begin_ladder;
use crate::worktree_ladder_test_support::{main_identity, plan_to, subvolume_repository};
use std::cell::RefCell;
use std::process::{Command, Stdio};

/// Every step the runner was asked for, as the words of its arguments.
type Steps = RefCell<Vec<Vec<String>>>;

/// A runner that records each step and runs the commands of `Other` steps with real Git in an isolated environment.
fn recording_runner(steps: &Steps, fail_reset: bool) -> impl FnMut(Option<&str>, &GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> + '_ {
    return move |_token, step| {
        let GitStep::Other { arguments, environment } = step else { panic!("only the ladder's own steps are expected") };
        steps.borrow_mut().push(arguments.iter().map(|word: &OsString| { return word.to_string_lossy().into_owned(); }).collect());
        if fail_reset && arguments.iter().any(|word: &OsString| { return word == "reset"; }) {
            return Ok(GitStatus { code: Some(1) });
        }
        let status = Command::new(REAL_GIT)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .envs(environment.iter().cloned())
            .args(arguments.iter())
            .stdin(Stdio::null())
            .status()
            .expect("Git starts");
        return Ok(GitStatus { code: status.code() });
    };
}

/// Run Git without a checkout the way the lifecycle does, then the ladder's `finish`; return the result, the steps and
///  what was printed.
fn add_through_the_ladder(repo: &Path, root: &Path, extra: &[&str], fail_reset: bool, spoil: impl FnOnce(&Path)) -> (LadderResult, Vec<Vec<String>>, String) {
    let destination: PathBuf = root.join("topic");
    let mut plan = plan_to(destination.clone());
    plan.quiet = extra.contains(&"--quiet");
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "the source is eligible");
    let mut words: Vec<&str> = vec!["worktree", "add", "--no-checkout"];
    words.extend_from_slice(extra);
    words.extend(["-b", "topic", "../topic"]);
    git(repo, words.as_slice());
    spoil(destination.as_path());
    let steps: Steps = RefCell::new(Vec::new());
    let mut printed: Vec<u8> = Vec::new();
    let result: LadderResult = {
        let mut runner = recording_runner(&steps, fail_reset);
        ladder.finish(GitStatus { code: Some(0) }, None, &mut runner, &git_context(), &mut printed)
    };
    return (result, steps.into_inner(), String::from_utf8_lossy(printed.as_slice()).into_owned());
}

/// The first, unchanged tree needs no Git step at all and prints Git's line.
#[test]
fn btrfs_a_source_at_the_target_commit_is_served_with_no_git_step() {
    let Some(root) = btrfs_fixture("finish-same") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |_| {});
    assert_eq!(result, LadderResult { git: GitStatus { code: Some(0) }, carried: Some(root.join("topic")) });
    assert!(steps.is_empty(), "{steps:?}");
    let head: String = git_text(repo.as_path(), &["log", "-1", "--format=HEAD is now at %h %s"]);
    assert_eq!(printed, format!("{head}\n"));
    assert!(is_subvolume_root(root.join("topic").as_path()).expect("probe"));
    assert!(root.join("topic/ignored/pkg/index.js").is_file());
    remove(root.as_path());
}

/// `--quiet` prints nothing.
#[test]
fn btrfs_quiet_prints_no_line() {
    let Some(root) = btrfs_fixture("finish-quiet") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, _steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &["--quiet"], false, |_| {});
    assert!(result.carried.is_some());
    assert_eq!(printed, "");
    remove(root.as_path());
}

/// A destination that holds more than Git's pointer is not served: the stage is destroyed, `reset --hard` completes the
///  worktree with the work tree and git directory Git gives its own checkout, and nothing is carried.
#[test]
fn btrfs_a_refused_step_destroys_the_stage_and_completes_the_checkout_like_git() {
    let Some(root) = btrfs_fixture("finish-fallback") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, steps, printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &[], false, |destination| {
        std::fs::write(destination.join("stray"), b"x").expect("a stray file");
    });
    assert_eq!(result, LadderResult { git: GitStatus { code: Some(0) }, carried: None });
    assert_eq!(printed, "", "Git's own reset prints the line, not the ladder");
    assert_eq!(steps.len(), 1, "{steps:?}");
    assert_eq!(&steps[0][2..], ["reset", "--hard", "--no-recurse-submodules"], "{steps:?}");
    assert!(root.join("topic/shared.txt").is_file(), "the checkout completed");
    assert!(!is_subvolume_root(root.join("topic").as_path()).expect("probe"), "the stage was not swapped in");
    let leftovers: Vec<String> = std::fs::read_dir(root.as_path()).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).filter(|name: &String| { return name.starts_with(".cli-git"); }).collect();
    assert_eq!(leftovers, Vec::<String>::new(), "the stage is gone");
    remove(root.as_path());
}

/// A failing `reset --hard` gives its status to the person, as Git's own would.
#[test]
fn btrfs_a_failing_checkout_step_gives_its_status() {
    let Some(root) = btrfs_fixture("finish-reset-fails") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (result, _steps, _printed) = add_through_the_ladder(repo.as_path(), root.as_path(), &["--quiet"], true, |destination| {
        std::fs::write(destination.join("stray"), b"x").expect("a stray file");
    });
    assert_eq!(result, LadderResult { git: GitStatus { code: Some(1) }, carried: None });
    remove(root.as_path());
}
