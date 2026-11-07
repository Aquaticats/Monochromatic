//! Controls for the sentence that says why an add was not a snapshot:
//! each cause is told in its own words with the remedy that
//! fits it,
//! a real source in each state produces the cause its state has,
//! a path cannot carry control characters into the
//! line,
//! and a filesystem that is not btrfs gets no sentence.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{git, git_context, remove, repository};
use crate::worktree_ladder_test_support::{main_identity, plan_to, subvolume_repository};
use crate::worktree_snapshot_source::source_refusal;
use std::path::PathBuf;

/// The route a real source in its current state gets,
/// or the panic that says the source was not refused.
fn route_of(repo: &Path) -> SlowRoute {
    let identity: WorktreeIdentity = main_identity(repo);
    let refusal: Refusal = source_refusal(&git_context(), &identity).expect_err("the source is refused");
    return explain_refusal(&git_context(), &identity, &refusal).expect("a sentence");
}

/// A modified tracked file is counted and named,
/// and the remedy is to commit or stash.
#[test]
fn btrfs_a_modified_file_is_an_uncommitted_change() {
    let Some(root) = btrfs_fixture("slow-modified") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let positive: Result<_, _> = source_refusal(&git_context(), &main_identity(repo.as_path()));
    assert!(positive.is_ok(), "positive control: the untouched source is accepted");
    std::fs::write(repo.join("shared.txt"), b"edited\n").expect("edit");
    let route: SlowRoute = route_of(repo.as_path());
    assert_eq!(route.word(), "uncommitted-changes");
    assert!(route.line().starts_with(LEAD), "{}", route.line());
    assert!(route.line().contains("the main checkout has uncommitted changes in 1 path (for example shared.txt)"), "{}", route.line());
    assert!(route.line().contains("Commit or stash them"), "{}", route.line());
    assert!(!route.line().contains('\n'), "one line");
    remove(root.as_path());
}

/// Two untracked files are two paths,
/// and a control character in a name never reaches the line.
#[test]
fn btrfs_untracked_files_are_counted_and_their_names_made_plain() {
    let Some(root) = btrfs_fixture("slow-untracked") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    std::fs::write(repo.join("a\u{1b}[31mred.txt"), b"u").expect("untracked");
    std::fs::write(repo.join("b.txt"), b"u").expect("untracked");
    let route: SlowRoute = route_of(repo.as_path());
    assert!(route.line().contains("uncommitted changes in 2 paths (for example a?[31mred.txt)"), "{}", route.line());
    assert!(!route.line().chars().any(char::is_control), "{}", route.line());
    remove(root.as_path());
}

/// A running Git command (`index.lock`) is named as such,
/// with the wait as the remedy.
#[test]
fn btrfs_a_lock_file_is_a_running_git_command() {
    let Some(root) = btrfs_fixture("slow-lock") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    std::fs::write(repo.join(".git/index.lock"), b"").expect("lock");
    let route: SlowRoute = route_of(repo.as_path());
    assert_eq!(route.word(), "git-running");
    assert!(route.line().contains("another Git command is running in the main checkout (its index.lock exists)"), "{}", route.line());
    assert!(route.line().contains("Add again when it has finished"), "{}", route.line());
    remove(root.as_path());
}

/// An unfinished merge is named as an operation,
/// with finishing it as the remedy.
#[test]
fn btrfs_a_merge_in_progress_is_an_unfinished_operation() {
    let Some(root) = btrfs_fixture("slow-merge") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let head: String = String::from_utf8_lossy(&git(repo.as_path(), &["rev-parse", "HEAD"]).stdout).trim().to_string();
    std::fs::write(repo.join(".git/MERGE_HEAD"), format!("{head}\n")).expect("merge mark");
    let route: SlowRoute = route_of(repo.as_path());
    assert_eq!(route.word(), "operation-unfinished");
    assert!(route.line().contains("MERGE_HEAD exists in its Git directory"), "{}", route.line());
    assert!(route.line().contains("Finish or abort it"), "{}", route.line());
    remove(root.as_path());
}

/// A file written in the second the index was written cannot be vouched for by the index,
/// though nothing changed:
/// `git status`
///  is the remedy,
/// and the sentence does not claim a change.
#[test]
fn btrfs_a_file_as_new_as_the_index_is_a_stale_record_and_not_a_change() {
    let Some(root) = btrfs_fixture("slow-racy") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    // Rewrite the same bytes now, so the file is as new as the index and Git still lists nothing as changed.
    let content: Vec<u8> = std::fs::read(repo.join("shared.txt")).expect("read");
    git(repo.as_path(), &["update-index", "--refresh"]);
    std::fs::write(repo.join("shared.txt"), content).expect("rewrite");
    git(repo.as_path(), &["update-index", "--really-refresh"]);
    // The index is rewritten by the refresh above and is now newer than the file, so make the file as new as the index again.
    let index_time = std::fs::metadata(repo.join(".git/index")).expect("index").modified().expect("time");
    std::fs::File::options().write(true).open(repo.join("shared.txt")).expect("open").set_modified(index_time).expect("set time");
    let route: SlowRoute = route_of(repo.as_path());
    assert_eq!(route.word(), "stale-index", "{}", route.line());
    assert!(route.line().contains("Run `git status` there once"), "{}", route.line());
    assert!(!route.line().contains("uncommitted"), "{}", route.line());
    remove(root.as_path());
}

/// An ordinary directory on btrfs is told to be converted,
/// with the command that fits main or a linked worktree;
/// a source on
///  another filesystem gets no sentence.
#[test]
fn btrfs_a_source_that_is_no_subvolume_is_told_to_convert_and_another_filesystem_gets_nothing() {
    let Some(root) = btrfs_fixture("slow-cheap") else { return };
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    let identity: WorktreeIdentity = main_identity(repo.as_path());
    let plan = plan_to(root.join("topic"));
    let reason: String = crate::worktree_snapshot_source::snapshot_refusal(&identity, &plan).expect("an ordinary directory is refused");
    let route: SlowRoute = explain_cheap_refusal(&identity, reason.as_str()).expect("a sentence on btrfs");
    assert_eq!(route.word(), "not-subvolume");
    assert!(route.line().contains("the main checkout is not a btrfs subvolume. Run `git monochromatic-git-policy-cli migrate-to-subvolume` in it"), "{}", route.line());
    let linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree { common_dir: repo.join(".git"), git_dir: repo.join(".git/worktrees/x"), worktree_root: repo.clone() };
    let linked_route: SlowRoute = explain_cheap_refusal(&linked, reason.as_str()).expect("a sentence");
    assert!(linked_route.line().contains("this worktree is not a btrfs subvolume. Run `git monochromatic-git-policy-cli convert-worktree-to-subvolume`"), "{}", linked_route.line());
    let elsewhere: WorktreeIdentity = WorktreeIdentity::MainWorktree { common_dir: PathBuf::from("/proc/.git"), git_dir: PathBuf::from("/proc/.git"), worktree_root: PathBuf::from("/proc") };
    assert!(explain_cheap_refusal(&elsewhere, "not btrfs").is_none(), "nothing the person does makes a snapshot on a filesystem that is not btrfs");
    assert!(explain_cheap_refusal(&WorktreeIdentity::OutsideWorktree, "no worktree").is_none());
    remove(root.as_path());
}

/// A feature a snapshot does not reproduce,
/// and a snapshot that failed,
/// are told in the check's own words,
/// and no sentence
///  names an internal step.
#[test]
fn unsupported_features_and_failures_are_told_in_the_checks_words() {
    let unsupported: SlowRoute = explain_unsupported("core.sparseCheckout is set");
    assert_eq!(unsupported.line(), format!("{LEAD}core.sparseCheckout is set."));
    assert_eq!(unsupported.word(), "unsupported");
    assert_eq!(unsupported.detail(), "core.sparseCheckout is set");
    let failed: SlowRoute = explain_failure("the stage could not be swapped into place");
    assert_eq!(failed.word(), "snapshot-failed");
    assert!(failed.line().contains("the snapshot could not be finished (the stage could not be swapped into place), so Git's own checkout made the worktree."), "{}", failed.line());
    for line in [unsupported.line(), failed.line()] {
        assert!(!line.to_lowercase().contains("rung"), "{line}");
    }
}

/// A Git status that failed is told with the run,
/// its exit code,
/// what Git said,
/// and both remedies (read its error,
/// or wait for a
///  running command);
/// with a stale index and a failing status the sentence names the failure and never "out of date".
#[test]
fn a_failed_status_is_told_with_its_error_and_the_remedies() {
    let root: PathBuf = crate::test_support::fixture("slow-status-failed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let identity: WorktreeIdentity = main_identity(repo.as_path());
    let stub: PathBuf = root.join("failing-git");
    crate::test_support::executable(stub.as_path(), b"#!/bin/sh\necho 'fatal: index file corrupt' >&2\nexit 128\n");
    let failing = crate::transaction_git::GitContext { real_git: stub, ..git_context() };
    let failed: Refusal = Refusal { reason: String::from("`git status` ended with exit code 128: fatal: index file corrupt"), kind: RefusalKind::StatusFailed };
    let route: SlowRoute = explain_refusal(&failing, &identity, &failed).expect("a sentence");
    assert_eq!(route.word(), "status-failed");
    let line: String = route.line();
    assert!(line.contains("the main checkout (`git status` ended with exit code 128: fatal: index file corrupt)"), "{line}");
    assert!(line.contains(format!("Run `git -C {} status` to read its error", repo.display()).as_str()) && line.contains("wait if another Git command is running"), "{line}");
    // The unclean sentence reads Git's status itself, and a status that fails there is told the same way.
    let racy: Refusal = Refusal { reason: String::from("a.txt is as new as the index, so its stat fields prove nothing"), kind: RefusalKind::Unclean(CleanRefusalKind::Racy) };
    let from_unclean: SlowRoute = explain_refusal(&failing, &identity, &racy).expect("a sentence");
    assert_eq!(from_unclean.word(), "status-failed");
    assert!(from_unclean.line().contains("exit code 128"), "{}", from_unclean.line());
    assert!(!from_unclean.line().contains("out of date"), "{}", from_unclean.line());
    remove(root.as_path());
}
