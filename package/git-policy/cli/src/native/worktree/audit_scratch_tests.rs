//! Throwaway controls for the audit of the worktree-add route.
//! Not committed: they are quoted in `doc/planning/cli-git-worktree-add-route-findings-on-main.md`.
//!
//! Each control drives one full rung with a seam instead of a race:
//! the rung is chosen by `begin_ladder`, real Git runs the `--no-checkout` add,
//! and the rung is finished through the ladder's own `finish` with a runner that starts real Git.

use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{REAL_GIT, git, git_context, git_text, remove};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_copy::{GitStatus, GitStep};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_ladder_run::{LadderResult, begin_ladder};
use crate::worktree_ladder_test_support::{commit_all, main_identity, tracked};
use std::cell::RefCell;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};


/// Run real Git in a fixture with a private index and optional standard input,
/// and give its standard output; the same environment `test_support::git` gives.
fn git_with_index(repo: &Path, arguments: &[&str], index: &Path, input: &[u8]) -> String {
    use std::io::Write as _;
    let mut child = Command::new(REAL_GIT)
        .current_dir(repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_INDEX_FILE", index)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Git starts");
    child.stdin.take().expect("stdin").write_all(input).expect("write the input");
    let output = child.wait_with_output().expect("Git ends");
    assert!(output.status.success(), "git {arguments:?} failed: {}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// A branch whose commit ignores nothing,
/// made from plumbing against a private index,
/// so the main checkout and its index are never touched and stay provably clean.
fn branch_ignoring_nothing(repo: &Path, from: &str, branch: &str) {
    let index: PathBuf = repo.join(".git/fixture-branch-index");
    let base: String = git_with_index(repo, &["rev-parse", from], &index, b"");
    let base_tree: String = git_with_index(repo, &["rev-parse", "HEAD^{tree}"], &index, b"");
    git_with_index(repo, &["read-tree", base_tree.as_str()], &index, b"");
    let blob: String = git_with_index(repo, &["hash-object", "-w", "--stdin"], &index, b"# nothing is ignored here\n");
    git_with_index(repo, &["update-index", "--add", "--cacheinfo", format!("100644,{blob},.gitignore").as_str()], &index, b"");
    let tree: String = git_with_index(repo, &["write-tree"], &index, b"");
    let commit: String = git_with_index(repo, &["commit-tree", tree.as_str(), "-p", base.as_str(), "-m", "ignore nothing"], &index, b"");
    git(repo, &["branch", branch, commit.as_str()]);
    let _ = std::fs::remove_file(index.as_path());
    let _ = base_tree;
}

/// A runner that records each step and runs the ladder's own Git commands.
fn recording_runner(steps: &RefCell<Vec<Vec<String>>>) -> impl FnMut(&GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> + '_ {
    return move |step| {
        let GitStep::Other { arguments, environment } = step else { panic!("only the ladder's own steps are expected") };
        steps.borrow_mut().push(arguments.iter().map(|word: &OsString| { return word.to_string_lossy().into_owned(); }).collect());
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

/// Whether a path exists, following no link.
fn present(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// What one add through the ladder found.
struct Added {
    /// The ladder's result.
    result: LadderResult,
    /// The steps the ladder ran.
    steps: Vec<Vec<String>>,
}

/// Add `branch` as a new worktree at `destination` through the ladder,
/// after real Git made it without a checkout.
fn add_through_the_ladder(repo: &Path, destination: &Path, branch: &str, operand: &str, commit: &str) -> Added {
    let plan: AddPlan = AddPlan { destination: destination.to_path_buf(), insert_at: 2, quiet: false };
    let ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "the source is eligible for a rung");
    git(repo, &["worktree", "add", "--no-checkout", "-b", branch, operand, commit]);
    let steps: RefCell<Vec<Vec<String>>> = RefCell::new(Vec::new());
    let result: LadderResult = {
        let mut runner = recording_runner(&steps);
        ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink())
    };
    return Added { result, steps: steps.into_inner() };
}

/// The name of the stage container of an add in flight, as `new_stage_container` spells it.
const OTHER_ADDS_STAGE: &str = ".cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543";

/// Control 1:
/// the staged payload of another add is carried into a snapshot worktree,
/// because the leave-out's rules name caches and checkouts and this directory is neither.
#[test]
fn btrfs_a_snapshot_carries_the_stage_container_of_another_add_into_the_new_worktree() {
    let Some(root) = btrfs_fixture("audit-stage-carried") else { return };
    let repo: PathBuf = root.join("repo");
    subvolume_create_at(repo.as_path()).expect("repository subvolume");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    tracked(repo.as_path(), ".gitignore", b"ignored/\n.claude/\n");
    tracked(repo.as_path(), "tracked.txt", b"one\n");
    commit_all(repo.as_path(), "base");
    let repo: PathBuf = std::fs::canonicalize(repo).expect("canonical");
    // The folder the worktrees of this repository are made in, and the stage of an add in flight in it.
    std::fs::create_dir_all(repo.join(".claude/worktrees")).expect("the folder that holds worktrees");
    let stage: PathBuf = repo.join(".claude/worktrees").join(OTHER_ADDS_STAGE);
    std::fs::create_dir_all(stage.join("payload/node_modules/left-pad")).expect("staged payload");
    std::fs::write(stage.join("payload/node_modules/left-pad/index.js"), b"module.exports = 2;\n").expect("staged file");
    let destination: PathBuf = repo.join(".claude/worktrees/topic");
    let added: Added = add_through_the_ladder(repo.as_path(), destination.as_path(), "topic", ".claude/worktrees/topic", "HEAD");
    let carried: PathBuf = destination.join(".claude/worktrees").join(OTHER_ADDS_STAGE).join("payload/node_modules/left-pad/index.js");
    eprintln!("CONTROL 1 steps: {:?}", added.steps);
    eprintln!("CONTROL 1 the ladder served the add: {:?}", added.result.carried);
    eprintln!("CONTROL 1 the stage is still in the source: {}", present(stage.as_path()));
    eprintln!("CONTROL 1 the stage's payload is in the new worktree: {}", present(carried.as_path()));
    eprintln!(
        "CONTROL 1 the new worktree's .claude/worktrees lists: {:?}",
        std::fs::read_dir(destination.join(".claude/worktrees")).map(|listing| {
            let mut names: Vec<String> = listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
            names.sort();
            return names;
        })
    );
    assert_eq!(added.result.carried, Some(destination.clone()), "the snapshot served the add");
    remove(root.as_path());
}

/// Control 2:
/// the ignored-state copy of a snapshot is not what the copy route hands over when the target
/// commit ignores less than the source,
/// and the source with one uncommitted edit is the source that loses the files.
#[test]
fn btrfs_a_restored_snapshot_drops_the_files_the_target_commit_does_not_ignore() {
    let Some(root) = btrfs_fixture("audit-ignore-rules") else { return };
    let repo: PathBuf = root.join("repo");
    subvolume_create_at(repo.as_path()).expect("repository subvolume");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    tracked(repo.as_path(), ".gitignore", b"cache/\n");
    tracked(repo.as_path(), "tracked.txt", b"one\n");
    commit_all(repo.as_path(), "base");
    // A second branch whose commit ignores nothing, made without checking anything out,
    // so the main checkout and its index stay the ones the clean check proved.
    branch_ignoring_nothing(repo.as_path(), "main", "other");
    eprintln!("CONTROL 2 the clean check of the fixture: {:?}", crate::worktree_clean::check_clean(&git_context(), repo.as_path(), &repo.join(".git"), &repo.join(".git")).map(|source| { return source.tracked; }));
    // The ignored state both adds are meant to hand over.
    std::fs::create_dir_all(repo.join("cache")).expect("cache");
    std::fs::write(repo.join("cache/data.bin"), b"payload\n").expect("ignored file");
    let repo: PathBuf = std::fs::canonicalize(repo).expect("canonical");

    // The clean source first: no uncommitted change, so no restore runs.
    let clean_destination: PathBuf = root.join("clean");
    let clean: Added = add_through_the_ladder(repo.as_path(), clean_destination.as_path(), "clean", "../clean", "other");
    eprintln!("CONTROL 2 the clean add's steps: {:?}", clean.steps);
    eprintln!("CONTROL 2 the clean add carried: {:?}", clean.result.carried);
    eprintln!("CONTROL 2 clean: cache/data.bin in the new worktree: {}", present(&clean_destination.join("cache/data.bin")));
    eprintln!("CONTROL 2 clean: git status in the new worktree: {:?}", git_text(clean_destination.as_path(), &["status", "--porcelain"]));

    // The same repository with one uncommitted edit to a tracked file: the restore runs.
    std::fs::write(repo.join("tracked.txt"), b"one, edited\n").expect("an uncommitted change");
    let dirty_destination: PathBuf = root.join("dirty");
    let dirty: Added = add_through_the_ladder(repo.as_path(), dirty_destination.as_path(), "dirty", "../dirty", "other");
    eprintln!("CONTROL 2 the dirty add's steps: {:?}", dirty.steps);
    eprintln!("CONTROL 2 the dirty add carried: {:?}", dirty.result.carried);
    eprintln!("CONTROL 2 dirty: cache/data.bin in the new worktree: {}", present(&dirty_destination.join("cache/data.bin")));
    eprintln!("CONTROL 2 dirty: git status in the new worktree: {:?}", git_text(dirty_destination.as_path(), &["status", "--porcelain"]));
    remove(root.as_path());
}

/// Control 3:
/// an add killed between real Git and the swap,
/// as a claim of a dead owner and a registered destination without a checkout,
/// is reported by one line of the next creating command and forgotten,
/// while its worktree and its stage stay.
#[test]
fn btrfs_an_interrupted_add_is_reported_once_and_its_leftovers_stay() {
    let Some(root) = btrfs_fixture("audit-interrupted-add") else { return };
    let repo: PathBuf = root.join("repo");
    subvolume_create_at(repo.as_path()).expect("repository subvolume");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    tracked(repo.as_path(), ".gitignore", b"ignored/\n.claude/\n");
    tracked(repo.as_path(), "tracked.txt", b"one\n");
    commit_all(repo.as_path(), "base");
    let repo: PathBuf = std::fs::canonicalize(repo).expect("canonical");
    std::fs::create_dir_all(repo.join(".claude/worktrees")).expect("the folder that holds worktrees");
    let destination: PathBuf = repo.join(".claude/worktrees/topic");
    let common_dir: PathBuf = repo.join(".git");

    // The claim an add writes before real Git runs, on behalf of a process that is now dead.
    let dead: crate::worktree_copy_owner::CopyOwner = crate::worktree_copy_owner::CopyOwner {
        pid: 4_194_303,
        birth_identity: String::from("0"),
        pid_namespace: None,
    };
    let claim: crate::worktree_copy_stage_claim::StageClaim = crate::worktree_copy_stage_claim::claim_stage_for_add_as(common_dir.as_path(), destination.as_path(), &dead).expect("a claim");
    // Real Git ran and registered the destination, and the add bound itself to what Git made.
    git(repo.as_path(), &["worktree", "add", "--no-checkout", "-b", "topic", ".claude/worktrees/topic"]);
    let bound = crate::worktree_add_binding::bind_registration(common_dir.as_path(), destination.as_path()).expect("bound");
    claim.record_binding(&bound.binding).expect("the binding is recorded");
    // The snapshot the rung had taken before the kill.
    let stage: PathBuf = claim.container().to_path_buf();
    subvolume_create_at(stage.as_path()).expect("the stage subvolume");
    std::fs::write(stage.join("payload-of-a-snapshot"), b"bytes\n").expect("staged bytes");
    drop(claim);

    // The next command that takes the lifecycle: recovery over the dead owner's records.
    let mut first: Vec<u8> = Vec::new();
    crate::worktree_copy_recovery::recover_pending_worktree_copies(common_dir.as_path(), &mut first).expect("recovery");
    eprintln!("CONTROL 3 what the first later command printed:\n{}", String::from_utf8_lossy(first.as_slice()));
    let left_records: Vec<String> = std::fs::read_dir(crate::worktree_copy_journal::journal_root(common_dir.as_path()).as_path()).map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect::<Vec<String>>(); }).unwrap_or_default();
    eprintln!("CONTROL 3 records left after that command: {:?}", left_records);
    eprintln!("CONTROL 3 the stage is still there: {}", present(stage.as_path()));
    eprintln!("CONTROL 3 the worktree holds: {:?}", std::fs::read_dir(destination.as_path()).map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect::<Vec<String>>(); }));
    eprintln!("CONTROL 3 git worktree list:\n{}", git_text(repo.as_path(), &["worktree", "list"]));
    eprintln!("CONTROL 3 git status in that worktree:\n{}", git_text(destination.as_path(), &["status", "--porcelain"]));

    // Every later command prints nothing more about it.
    let mut second: Vec<u8> = Vec::new();
    crate::worktree_copy_recovery::recover_pending_worktree_copies(common_dir.as_path(), &mut second).expect("recovery");
    eprintln!("CONTROL 3 what the second later command printed: {:?}", String::from_utf8_lossy(second.as_slice()));
    eprintln!("CONTROL 3 the worktree is still registered and empty: {} / {:?}", present(&destination.join(".git")), std::fs::read_dir(destination.as_path()).map(|listing| { return listing.count(); }));
    remove(root.as_path());
}
