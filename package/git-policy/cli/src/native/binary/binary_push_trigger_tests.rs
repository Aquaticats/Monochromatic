//! What:
//!  The commands other than `git commit` that add commits to a branch (merge,
//! pull's merge,
//! cherry-pick,
//! revert,
//! am) hand
//!  the push to the service when they leave the branch ahead of its upstream and not diverged from it.
//! Why:
//!  A clean `git merge` used to stay local until the next commit pushed it.
//! The owner chose that every command adding
//!  commits queues the push as a commit does (2026-11-02),
//! and that a rebase does not,
//! because a rebased branch would need a
//!  forced push.
//! The remote is a real bare repository,
//! the service a real process,
//! the commands the built wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await wrapped(['merge', 'topic']); await untilSettled(); expect(remoteTip('main')).toBe(localTip('main'));
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, fixture, git, observe, repository, settle_service_work, start_fixture_service, FixtureService};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A repository on `main` that tracks a bare remote's `main`,
/// a clone that stands for another person,
/// and a service.
struct World {
    fixture: Fixture,
    repo: PathBuf,
    remote: PathBuf,
    other: PathBuf,
    service: FixtureService,
}

/// The world with `main` published and a `topic` branch one commit ahead of it,
/// checked out `main`.
fn world(name: &str) -> World {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    git(&fixture, fixture.root.as_path(), &["clone", "--quiet", remote.to_str().expect("UTF-8"), "other"]);
    let other: PathBuf = fixture.root.join("other");
    git(&fixture, &repo, &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(repo.join("b.txt"), "topic\n").expect("b");
    git(&fixture, &repo, &["add", "b.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=topic work"]);
    git(&fixture, &repo, &["switch", "--quiet", "main"]);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return World { fixture, repo, remote, other, service };
}

/// Run a wrapped command in `directory` with the fixture service reachable,
/// and return what the caller saw.
fn wrapped_in(world: &World, directory: &Path, arguments: &[&str]) -> Observed {
    let mut command = wrapped_commit(&world.fixture, directory, arguments);
    command.env("XDG_RUNTIME_DIR", &world.service.runtime);
    return observe(&mut command, b"");
}

/// The same in the repository under test.
fn run(world: &World, arguments: &[&str]) -> Observed {
    return wrapped_in(world, world.repo.as_path(), arguments);
}

/// What a ref holds in a repository,
/// trimmed.
fn tip(world: &World, repository: &Path, name: &str) -> String {
    return String::from_utf8_lossy(&git(&world.fixture, repository, &["rev-parse", name]).stdout).trim().to_owned();
}

/// How many push records of any state the service holds for the repository.
fn push_records(world: &World) -> usize {
    let work: PathBuf = world.repo.join(".git/monochromatic-git-policy-cli/work");
    return ["pending", "claimed", "recovering", "done", "failed"]
        .iter()
        .map(|state| {
            return std::fs::read_dir(work.join(state)).map_or(0, |listing| {
                return listing.flatten().filter(|entry| { return entry.file_name().to_string_lossy().starts_with("push-"); }).count();
            });
        })
        .sum();
}

/// Assert the command succeeded,
/// wait for the service,
/// and assert the remote's `main` is the local one.
fn assert_pushed(world: &World, observed: &Observed) {
    let text: String = String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
    assert_eq!(observed.code, Some(0), "{text}");
    settle_service_work(world.repo.as_path());
    let local: String = tip(world, world.repo.as_path(), "main");
    assert_eq!(tip(world, world.remote.as_path(), "main"), local, "the remote follows the branch: {text}");
}

/// Assert the command succeeded and nothing was queued or pushed.
fn assert_not_pushed(world: &World, observed: &Observed, remote_before: &str) {
    let text: String = String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
    assert_eq!(observed.code, Some(0), "{text}");
    settle_service_work(world.repo.as_path());
    assert_eq!(push_records(world), 0, "no push was handed over: {text}");
    assert_eq!(tip(world, world.remote.as_path(), "main"), remote_before, "the remote did not move: {text}");
}

/// A merge commit is pushed once the command returns.
#[test]
fn a_merge_commit_is_pushed() {
    let world: World = world("trigger-merge");
    let observed: Observed = run(&world, &["merge", "--no-ff", "--no-edit", "topic"]);
    assert_pushed(&world, &observed);
    remove_settled(&world.fixture);
}

/// A fast-forward merge moves the tip without making a commit,
/// and is pushed too.
#[test]
fn a_fast_forward_merge_is_pushed() {
    let world: World = world("trigger-fast-forward");
    let observed: Observed = run(&world, &["merge", "--ff-only", "topic"]);
    assert_pushed(&world, &observed);
    remove_settled(&world.fixture);
}

/// A cherry-pick is pushed.
#[test]
fn a_cherry_pick_is_pushed() {
    let world: World = world("trigger-cherry-pick");
    let observed: Observed = run(&world, &["cherry-pick", "topic"]);
    assert_pushed(&world, &observed);
    remove_settled(&world.fixture);
}

/// A revert is pushed.
#[test]
fn a_revert_is_pushed() {
    let world: World = world("trigger-revert");
    git(&world.fixture, &world.repo, &["merge", "--quiet", "--ff-only", "topic"]);
    git(&world.fixture, &world.repo, &["push", "--quiet", "origin", "main"]);
    let observed: Observed = run(&world, &["revert", "--no-edit", "HEAD"]);
    assert_pushed(&world, &observed);
    remove_settled(&world.fixture);
}

/// A mailbox patch applied with `am` is pushed.
#[test]
fn an_applied_patch_is_pushed() {
    let world: World = world("trigger-am");
    let patch = git(&world.fixture, &world.repo, &["format-patch", "--stdout", "main..topic"]).stdout;
    let file: PathBuf = world.fixture.root.join("topic.patch");
    std::fs::write(&file, patch).expect("patch");
    let observed: Observed = run(&world, &["am", file.to_str().expect("UTF-8")]);
    assert_pushed(&world, &observed);
    remove_settled(&world.fixture);
}

/// `git pull` that merges what another person pushed is pushed,
/// on top of the local commit.
#[test]
fn a_pull_that_merges_is_pushed() {
    let world: World = world("trigger-pull-merge");
    std::fs::write(world.other.join("c.txt"), "remote\n").expect("c");
    git(&world.fixture, &world.other, &["add", "c.txt"]);
    git(&world.fixture, &world.other, &["commit", "--quiet", "--message=remote work"]);
    git(&world.fixture, &world.other, &["push", "--quiet", "origin", "main"]);
    std::fs::write(world.repo.join("d.txt"), "local\n").expect("d");
    git(&world.fixture, &world.repo, &["add", "d.txt"]);
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--message=local work"]);
    let observed: Observed = run(&world, &["pull", "--no-rebase", "--no-edit"]);
    assert_pushed(&world, &observed);
    remove_settled(&world.fixture);
}

/// `git pull --rebase` is a rebase,
/// which the owner excludes:
/// the rewritten branch is not handed over.
#[test]
fn a_pull_that_rebases_is_not_pushed() {
    let world: World = world("trigger-pull-rebase");
    std::fs::write(world.other.join("c.txt"), "remote\n").expect("c");
    git(&world.fixture, &world.other, &["add", "c.txt"]);
    git(&world.fixture, &world.other, &["commit", "--quiet", "--message=remote work"]);
    git(&world.fixture, &world.other, &["push", "--quiet", "origin", "main"]);
    std::fs::write(world.repo.join("d.txt"), "local\n").expect("d");
    git(&world.fixture, &world.repo, &["add", "d.txt"]);
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--message=local work"]);
    let remote_before: String = tip(&world, world.remote.as_path(), "main");
    let observed: Observed = run(&world, &["pull", "--rebase"]);
    assert_not_pushed(&world, &observed, remote_before.as_str());
    remove_settled(&world.fixture);
}

/// A rebase is not handed over.
#[test]
fn a_rebase_is_not_pushed() {
    let world: World = world("trigger-rebase");
    std::fs::write(world.repo.join("d.txt"), "local\n").expect("d");
    git(&world.fixture, &world.repo, &["add", "d.txt"]);
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--message=local work"]);
    git(&world.fixture, &world.repo, &["switch", "--quiet", "topic"]);
    let remote_before: String = tip(&world, world.remote.as_path(), "main");
    let observed: Observed = run(&world, &["rebase", "main"]);
    assert_not_pushed(&world, &observed, remote_before.as_str());
    remove_settled(&world.fixture);
}

/// A merge that stops on a conflict adds nothing and queues nothing.
#[test]
fn a_conflicting_merge_is_not_pushed() {
    let world: World = world("trigger-conflict");
    git(&world.fixture, &world.repo, &["switch", "--quiet", "topic"]);
    std::fs::write(world.repo.join("a.txt"), "topic side\n").expect("a");
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--all", "--message=topic edit"]);
    git(&world.fixture, &world.repo, &["switch", "--quiet", "main"]);
    std::fs::write(world.repo.join("a.txt"), "main side\n").expect("a");
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--all", "--message=main edit"]);
    git(&world.fixture, &world.repo, &["push", "--quiet", "origin", "main"]);
    let remote_before: String = tip(&world, world.remote.as_path(), "main");
    let observed: Observed = run(&world, &["merge", "--no-edit", "topic"]);
    assert_eq!(observed.code, Some(1));
    settle_service_work(world.repo.as_path());
    assert_eq!(push_records(&world), 0);
    assert_eq!(tip(&world, world.remote.as_path(), "main"), remote_before);
    remove_settled(&world.fixture);
}

/// A merge that finds nothing to merge leaves the tip where it was and queues nothing.
#[test]
fn a_merge_that_moves_nothing_is_not_pushed() {
    let world: World = world("trigger-up-to-date");
    git(&world.fixture, &world.repo, &["merge", "--quiet", "--ff-only", "topic"]);
    git(&world.fixture, &world.repo, &["push", "--quiet", "origin", "main"]);
    let remote_before: String = tip(&world, world.remote.as_path(), "main");
    let observed: Observed = run(&world, &["merge", "topic"]);
    assert_not_pushed(&world, &observed, remote_before.as_str());
    remove_settled(&world.fixture);
}

/// A branch that has diverged from its upstream (the remote moved and was fetched) is not handed over:
/// the push would be
/// rejected,
/// and the owner asked for ahead and not diverged.
#[test]
fn a_merge_into_a_diverged_branch_is_not_pushed() {
    let world: World = world("trigger-diverged");
    std::fs::write(world.other.join("c.txt"), "remote\n").expect("c");
    git(&world.fixture, &world.other, &["add", "c.txt"]);
    git(&world.fixture, &world.other, &["commit", "--quiet", "--message=remote work"]);
    git(&world.fixture, &world.other, &["push", "--quiet", "origin", "main"]);
    git(&world.fixture, &world.repo, &["fetch", "--quiet", "origin"]);
    let remote_before: String = tip(&world, world.remote.as_path(), "main");
    let observed: Observed = run(&world, &["merge", "--no-ff", "--no-edit", "topic"]);
    assert_not_pushed(&world, &observed, remote_before.as_str());
    remove_settled(&world.fixture);
}

/// A branch without an upstream is not handed over by a merge:
/// the first push of a branch is a commit's business.
#[test]
fn a_merge_into_a_branch_without_an_upstream_is_not_pushed() {
    let world: World = world("trigger-no-upstream");
    git(&world.fixture, &world.repo, &["switch", "--quiet", "--create", "loose"]);
    let remote_before: String = tip(&world, world.remote.as_path(), "main");
    let observed: Observed = run(&world, &["merge", "--no-ff", "--no-edit", "topic"]);
    assert_not_pushed(&world, &observed, remote_before.as_str());
    remove_settled(&world.fixture);
}

/// A merge that native Git refuses only because the index holds staged changes is completed by the wrapper from objects
/// (`merge_isolation.rs`);
/// it is a command that adds a commit like any other,
/// and it is pushed,
/// leaving the staged file staged.
#[test]
fn a_merge_the_wrapper_completes_with_staged_changes_is_pushed() {
    let world: World = world("trigger-isolated-merge");
    std::fs::write(world.repo.join("main-side.txt"), "main\n").expect("file");
    git(&world.fixture, &world.repo, &["add", "main-side.txt"]);
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--message=main side"]);
    git(&world.fixture, &world.repo, &["push", "--quiet", "origin", "main"]);
    std::fs::write(world.repo.join("staged.txt"), "staged\n").expect("file");
    git(&world.fixture, &world.repo, &["add", "staged.txt"]);
    let observed: Observed = run(&world, &["merge", "--no-ff", "--no-edit", "topic"]);
    assert_pushed(&world, &observed);
    let status: String = String::from_utf8_lossy(&git(&world.fixture, &world.repo, &["status", "--porcelain"]).stdout).into_owned();
    assert_eq!(status.trim(), "A  staged.txt", "the staged file is still staged");
    remove_settled(&world.fixture);
}

/// The ending of a stopped cherry-pick counts as the command that adds the commit:
/// `--continue` moves the tip and is pushed.
#[test]
fn a_cherry_pick_continued_after_a_conflict_is_pushed() {
    let world: World = world("trigger-continue");
    git(&world.fixture, &world.repo, &["switch", "--quiet", "topic"]);
    std::fs::write(world.repo.join("a.txt"), "topic side\n").expect("a");
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--all", "--message=topic edit"]);
    git(&world.fixture, &world.repo, &["switch", "--quiet", "main"]);
    std::fs::write(world.repo.join("a.txt"), "main side\n").expect("a");
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--all", "--message=main edit"]);
    git(&world.fixture, &world.repo, &["push", "--quiet", "origin", "main"]);
    let remote_before: String = tip(&world, world.remote.as_path(), "main");
    let stopped: Observed = run(&world, &["cherry-pick", "topic"]);
    assert_eq!(stopped.code, Some(1));
    settle_service_work(world.repo.as_path());
    assert_eq!(tip(&world, world.remote.as_path(), "main"), remote_before, "a stopped pick pushes nothing");
    std::fs::write(world.repo.join("a.txt"), "resolved\n").expect("a");
    git(&world.fixture, &world.repo, &["add", "a.txt"]);
    let mut command = wrapped_commit(&world.fixture, world.repo.as_path(), &["cherry-pick", "--continue"]);
    command.env("XDG_RUNTIME_DIR", &world.service.runtime).env("GIT_EDITOR", "true");
    let continued: Observed = observe(&mut command, b"");
    assert_pushed(&world, &continued);
    remove_settled(&world.fixture);
}

/// The user's own configuration file,
/// holding a setting that Git accepts and the in-process reader cannot prove harmless (the live report had `core.fsmonitor = true`,
/// which starts a daemon and so cannot be used in a fixture;
/// `diff.dirstat` is the same kind of key for the reader).
fn user_configuration(world: &World) -> PathBuf {
    let file: PathBuf = world.fixture.root.join("user.gitconfig");
    std::fs::write(&file, "[diff]\n\tdirstat = changes\n").expect("user configuration");
    return file;
}

/// The repository of the live report:
/// `extensions.worktreeConfig`,
/// `push.default = matching`,
/// a linked worktree,
/// and a user whose own configuration file holds a setting the reader declines.
/// The merge made through the wrapper is pushed,
/// and the trigger's record says it was handed over.
/// Before,
/// the watch never started and nothing said why.
#[test]
fn a_merge_is_pushed_for_a_user_whose_configuration_the_reader_cannot_prove() {
    let world: World = world("trigger-user-configuration");
    git(&world.fixture, &world.repo, &["config", "extensions.worktreeConfig", "true"]);
    git(&world.fixture, &world.repo, &["config", "push.default", "matching"]);
    git(&world.fixture, &world.repo, &["worktree", "add", "--quiet", "--detach", world.fixture.root.join("linked").to_str().expect("UTF-8")]);
    let file: PathBuf = user_configuration(&world);
    let mut command = wrapped_commit(&world.fixture, world.repo.as_path(), &["merge", "--no-ff", "--no-edit", "topic"]);
    command.env("XDG_RUNTIME_DIR", &world.service.runtime).env("GIT_CONFIG_GLOBAL", &file);
    let observed: Observed = observe(&mut command, b"");
    assert_pushed(&world, &observed);
    let record: String = std::fs::read_to_string(world.repo.join(".git/monochromatic-git-policy-cli-push-trigger.jsonl")).expect("the trigger's record");
    assert!(record.contains("\"outcome\":\"handed-over\"") && record.contains("\"command\":\"merge\""), "{record}");
    remove_settled(&world.fixture);
}

/// A command that leaves nothing to push says why in the repository's record,
/// and prints nothing:
/// the merge finds nothing to merge,
/// so the tip did not move.
#[test]
fn a_declined_trigger_leaves_its_reason_in_the_record() {
    let world: World = world("trigger-record");
    git(&world.fixture, &world.repo, &["merge", "--quiet", "--ff-only", "topic"]);
    git(&world.fixture, &world.repo, &["push", "--quiet", "origin", "main"]);
    let observed: Observed = run(&world, &["merge", "topic"]);
    assert_eq!(observed.code, Some(0));
    let record: String = std::fs::read_to_string(world.repo.join(".git/monochromatic-git-policy-cli-push-trigger.jsonl")).expect("the trigger's record");
    assert!(record.contains("\"outcome\":\"declined\"") && record.contains("\"reason\":\"tip-unchanged\""), "{record}");
    assert!(record.contains("\"branch\":\"refs/heads/main\""), "{record}");
    let text: String = String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
    assert!(!text.contains("tip-unchanged"), "the terminal is not told: {text}");
    remove_settled(&world.fixture);
}

/// A merge that stops on a conflict records that Git failed,
/// so a missing push after a failed command is explained too.
#[test]
fn a_failed_command_is_recorded_as_such() {
    let world: World = world("trigger-failed-record");
    git(&world.fixture, &world.repo, &["switch", "--quiet", "topic"]);
    std::fs::write(world.repo.join("a.txt"), "topic side\n").expect("a");
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--all", "--message=topic edit"]);
    git(&world.fixture, &world.repo, &["switch", "--quiet", "main"]);
    std::fs::write(world.repo.join("a.txt"), "main side\n").expect("a");
    git(&world.fixture, &world.repo, &["commit", "--quiet", "--all", "--message=main edit"]);
    let observed: Observed = run(&world, &["merge", "--no-edit", "topic"]);
    assert_eq!(observed.code, Some(1));
    let record: String = std::fs::read_to_string(world.repo.join(".git/monochromatic-git-policy-cli-push-trigger.jsonl")).expect("the trigger's record");
    assert!(record.contains("\"reason\":\"command-failed\"") && record.contains("status 1"), "{record}");
    // The file is shared by every worktree: the line names the worktree and the branch the failed command ran on.
    assert!(record.contains("\"branch\":\"refs/heads/main\""), "the branch is named: {record}");
    let root: String = world.repo.to_string_lossy().into_owned();
    assert!(record.contains(format!("\"worktree\":{}", git_policy_cli::json_record::quote(root.as_str())).as_str()), "the worktree is named: {record}");
    assert!(record.contains("\"instance\":\"") && record.contains("\"level\":\"info\""), "{record}");
    remove_settled(&world.fixture);
}

/// A watch that cannot start (the head is detached) leaves its reason in the record too,
/// though the command itself runs.
#[test]
fn a_watch_that_could_not_start_is_recorded() {
    let world: World = world("trigger-detached-record");
    git(&world.fixture, &world.repo, &["switch", "--quiet", "--detach"]);
    let observed: Observed = run(&world, &["merge", "--no-ff", "--no-edit", "topic"]);
    assert_eq!(observed.code, Some(0), "{}", String::from_utf8_lossy(observed.stderr.as_slice()));
    let record: String = std::fs::read_to_string(world.repo.join(".git/monochromatic-git-policy-cli-push-trigger.jsonl")).expect("the trigger's record");
    assert!(record.contains("\"reason\":\"head-detached\"") && record.contains("\"command\":\"merge\""), "{record}");
    let root: String = world.repo.to_string_lossy().into_owned();
    assert!(record.contains(format!("\"worktree\":{}", git_policy_cli::json_record::quote(root.as_str())).as_str()), "the worktree is named: {record}");
    remove_settled(&world.fixture);
}
