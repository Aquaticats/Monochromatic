//! Controls for the words a push stops with when the transport settings of its command are gone:
//! what they say,
//! and that the command they name pushes the branch as printed.

use super::*;
use std::path::PathBuf;

/// What:
///  A repository on `main`,
///  its bare origin,
///  the push's Git,
///  and the commit the branch names.
/// Why:
///   The command the words name must be run against a real repository and a real remote,
/// so the proof is that the remote holds the commit afterwards,
/// not that the line reads correctly.
struct World {
    /// The root of this control's fixtures.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// Its bare origin.
    origin: PathBuf,
    /// The commit the branch names.
    commit: String,
}

/// Build the world of one control.
fn world(name: &str) -> World {
    let root: PathBuf = crate::test_support::fixture(name);
    let origin: PathBuf = root.join("origin.git");
    crate::test_support::git(root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", origin.to_str().expect("a root that is text")]);
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    crate::test_support::git(repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("a root that is text")]);
    let commit: String = crate::test_support::git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return World { root, repo, origin, commit };
}

/// The push's Git,
/// asking the fixture's own real Git with no configuration of the account's.
fn push_git(world: &World) -> PushGit {
    return PushGit { real_git: PathBuf::from(crate::test_support::REAL_GIT), cwd: world.repo.clone(), environment: crate::test_support::scratch_environment(), lfs: false };
}

/// The push record an earlier `git push` of this branch would have written.
fn push_item(world: &World) -> WorkItem {
    return WorkItem {
        kind: crate::work_item::WorkKind::Push,
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        branch_ref: String::from("refs/heads/main"),
        oid: world.commit.clone(),
        parent: String::new(),
        created: String::from("2026-11-12T12:00:00.000Z"),
        tries: 0,
        transport: true,
    };
}

/// The words for the push of `world`,
/// as the person's next command shows them.
fn words_of(world: &World) -> String {
    let git: PushGit = push_git(world);
    let item: WorkItem = push_item(world);
    return crate::lazy_runtime::run_on_runtime(super::words(&git, &item)).expect("the plan asks only the local repository");
}

/// What:
///  The words name the repository,
///  the branch,
///  why the settings are gone,
///  and the command that pushes instead,
///  as the last line and nothing else on it.
/// Why:
///   A person copies the line,
/// and a control runs exactly what was printed,
/// so the command may carry no other text.
#[test]
fn the_words_name_the_push_and_the_command_that_makes_it() {
    let world: World = world("transport-gone");
    let words: String = words_of(&world);
    assert!(words.contains(world.repo.to_str().expect("a path that is text")), "{words}");
    assert!(words.contains("refs/heads/main"), "{words}");
    assert!(words.contains("held only in the memory of the commit service"), "{words}");
    assert!(words.contains("nothing was sent through another transport"), "{words}");
    let last: &str = words.rsplit('\n').next().expect("the words have a last line");
    assert!(last.starts_with(&format!("Run this in {}: ", world.repo.display())), "{last}");
    let command: &str = last.split_once(": ").expect("the line names a command").1;
    assert_eq!(
        command,
        format!("git push --set-upstream origin refs/heads/main:refs/heads/main"),
        "the command is the one this push would have run: the branch, its remote and the upstream a branch without one takes"
    );
    crate::test_support::remove(world.root.as_path());
}

/// What:
///  The command the words print pushes the branch,
///  run exactly as printed.
/// Why:
///   A message that hands a person a command that fails is worse than no command:
/// the push it replaces was stopped so that the commit could still reach the remote by hand.
#[test]
fn the_command_that_is_printed_pushes_the_branch() {
    let world: World = world("transport-gone-runs");
    let words: String = words_of(&world);
    let command: String = words.rsplit('\n').next().expect("the words have a last line").split_once(": ").expect("the line names a command").1.to_string();
    let run = std::process::Command::new("/bin/sh").current_dir(world.repo.as_path()).arg("-c").arg(command.as_str()).env_clear().env("PATH", "/usr/bin:/bin").env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config").output().expect("the printed command runs");
    assert!(run.status.success(), "the printed command pushes: {}\n{}", String::from_utf8_lossy(run.stdout.as_slice()), String::from_utf8_lossy(run.stderr.as_slice()));
    let on_the_remote: std::process::Output = crate::test_support::fixture_git_commands::git_output(world.origin.as_path(), &["rev-parse", "refs/heads/main"]);
    assert_eq!(
        String::from_utf8_lossy(on_the_remote.stdout.as_slice()).trim(),
        world.commit,
        "the remote holds the commit the stopped push would have sent"
    );
    crate::test_support::remove(world.root.as_path());
}

/// What:
///  A branch with no upstream and no `origin` has nowhere to be pushed,
///  so the words name no command rather than one that cannot work.
/// Why:
///   The push this replaces would have been skipped for the same reason,
/// and a command naming a remote that does not exist sends the person to a second failure.
#[test]
fn a_branch_with_nowhere_to_go_names_no_command() {
    let world: World = world("transport-gone-nowhere");
    crate::test_support::git(world.repo.as_path(), &["remote", "remove", "origin"]);
    let words: String = words_of(&world);
    assert!(words.contains("names no remote to push to"), "{words}");
    assert!(!words.contains("git push"), "no command is printed: {words}");
    crate::test_support::remove(world.root.as_path());
}
