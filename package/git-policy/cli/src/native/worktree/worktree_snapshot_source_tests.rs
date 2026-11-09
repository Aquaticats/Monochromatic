//! Controls for the snapshot source:
//! a clean subvolume main checkout is one,
//! and each condition that rules a source out is
//!  refused with its reason.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::parallel_test_support::{PATIENCE, hold_the_only_place};
use crate::test_support::{age, git, git_context, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, subvolume_repository, tracked};

/// What:
///  The three checks of a source are waits of the process's runtime,
/// and the filesystem proof inside the clean check is a job of its blocking pool.
/// Why:
///   The owner asked every wait to sit on the runtime,
/// and a synchronous walk of every tracked file must not run on a task
///  that other work of the check shares.
/// Only a job of the blocking pool asks for a place of the process's bound on blocking jobs,
/// so an ask is what says the proof left
///  the calling thread;
/// the holder gives its place back on that ask and never on a clock.
#[test]
fn btrfs_the_checks_of_the_source_take_a_place_of_the_processs_bound() {
    let Some(root) = btrfs_fixture("source-a-place-of-the-bound") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let (_given_back, finished) = hold_the_only_place();
    let source: Result<SnapshotSource, String> = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan);
    assert!(source.is_ok(), "{:?}", source.err());
    assert!(
        finished.recv_timeout(PATIENCE).expect("the job that holds the only place never returned"),
        "the files of the source were proven clean without asking for a place of the process's bound on blocking jobs"
    );
    remove(root.as_path());
}

/// A clean subvolume main checkout is a source,
/// and it names the commit and tree of its `HEAD`.
#[test]
fn btrfs_a_clean_subvolume_main_checkout_is_a_source() {
    let Some(root) = btrfs_fixture("source-clean") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let source: SnapshotSource = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    assert_eq!(source.root, repo);
    assert_eq!(source.clean.head_commit, String::from_utf8_lossy(&git(repo.as_path(), &["rev-parse", "HEAD"]).stdout).trim());
    assert!(source.nested.is_empty());
    remove(root.as_path());
}

/// Each refusal names its reason,
/// and the refused source is left as it was.
#[test]
fn btrfs_each_condition_that_rules_a_source_out_is_refused_with_its_reason() {
    let Some(root) = btrfs_fixture("source-refused") else { return };
    let plan = plan_to(root.join("topic"));
    let refusal = |repo: &Path| -> String {
        return choose_snapshot_source(&git_context(), &main_identity(repo), &plan).expect_err("refused");
    };
    // An ordinary directory is no subvolume root.
    let ordinary: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    assert!(refusal(ordinary.as_path()).contains("is not a btrfs subvolume"));
    // A bare repository and a missing worktree have nothing to snapshot.
    let bare = WorktreeIdentity::BareRepository { common_dir: root.clone(), git_dir: root.clone() };
    assert!(choose_snapshot_source(&git_context(), &bare, &plan).expect_err("bare").contains("no worktree"));
    // An uncommitted change.
    let repo: PathBuf = subvolume_repository(root.as_path());
    assert!(choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).is_ok(), "positive control");
    std::fs::write(repo.join("shared.txt"), b"edited\n").expect("edit");
    assert!(refusal(repo.as_path()).contains("shared.txt"));
    git(repo.as_path(), &["checkout", "--quiet", "--", "shared.txt"]);
    age(repo.join("shared.txt").as_path());
    git(repo.as_path(), &["update-index", "--refresh"]);
    // A sparse checkout.
    std::fs::create_dir_all(repo.join(".git/info")).expect("info");
    std::fs::write(repo.join(".git/info/sparse-checkout"), b"/*\n").expect("patterns");
    assert!(refusal(repo.as_path()).contains("sparse checkout"));
    std::fs::remove_file(repo.join(".git/info/sparse-checkout")).expect("remove patterns");
    // A worktree-specific configuration.
    git(repo.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    git(repo.as_path(), &["config", "--worktree", "core.autocrlf", "true"]);
    assert!(refusal(repo.as_path()).contains("worktree-specific configuration sets core.autocrlf"));
    git(repo.as_path(), &["config", "--worktree", "--unset", "core.autocrlf"]);
    // A nested worktree that is an ordinary directory.
    tracked(repo.as_path(), ".gitignore", b"ignored/\nnested/\n");
    commit_all(repo.as_path(), "ignore nested");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "nested", "nested/wt"]);
    // A snapshot of the main checkout sets such a worktree aside (`worktree_trash.rs`), so it no longer blocks the main checkout;
    //  it still blocks a snapshot of a linked worktree that holds one.
    let main_plan = plan_to(std::fs::canonicalize(root.join("topic")).unwrap_or_else(|_missing| { return root.join("topic"); }));
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &main_plan).expect("the main checkout is a source");
    assert_eq!(snapshot_blocker(&source), None);
    let mut as_linked = source.clone();
    as_linked.git_dir = repo.join(".git/worktrees/wt");
    let nested: String = snapshot_blocker(&as_linked).expect("a linked source is blocked");
    assert!(nested.contains("inside the source is an ordinary directory"), "{nested}");
    remove(root.as_path());
}

/// A registered nested worktree whose recorded path passes through a link is a blocker of the snapshot rung;
/// one reached by
///  its real path is not,
/// and one that is gone is nothing.
#[test]
fn a_nested_worktree_reached_through_a_link_is_found() {
    let root: PathBuf = crate::test_support::fixture("source-nested-link");
    let real: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    std::fs::create_dir_all(real.join("moved/wt")).expect("moved");
    std::os::unix::fs::symlink(real.join("moved"), real.join("group")).expect("link");
    assert_eq!(nested_through_link(&[real.join("group/wt")]), Some(real.join("group/wt")));
    assert_eq!(nested_through_link(&[real.join("moved/wt"), real.join("gone/wt")]), None);
    crate::test_support::remove(root.as_path());
}

/// A nested worktree whose record still spells the name the repository had before it moved behind a link is found by the
/// directory it names,
/// so the snapshot rung is refused for an ordinary directory it would otherwise carry in full.
#[test]
fn btrfs_a_nested_worktree_registered_before_the_move_behind_a_link_is_found() {
    let Some(root) = btrfs_fixture("source-nested-moved") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    tracked(repo.as_path(), ".gitignore", b"ignored/\nnested/\n");
    commit_all(repo.as_path(), "ignore nested");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "nested", "nested/wt"]);
    // The repository moved behind a link: a link is left at the old name,
    // so the registration still spells the old name while naming the worktree inside the checkout.
    let moved: PathBuf = root.join("moved-repo");
    std::fs::rename(&repo, &moved).expect("move the checkout");
    std::os::unix::fs::symlink(&moved, &repo).expect("leave a link at the old name");
    let source: SnapshotSource = choose_snapshot_source(&git_context(), &main_identity(moved.as_path()), &plan_to(root.join("topic"))).expect("a source");
    assert_eq!(source.nested, vec![moved.join("nested/wt")], "the registration names the worktree inside the source");
    // The leave-out of a main checkout's snapshot finds it by the path inside the source,
    // and a linked source is refused for the ordinary directory it would carry in full.
    let mut as_linked: SnapshotSource = source.clone();
    as_linked.git_dir = moved.join(".git/worktrees/nested");
    let blocker: String = snapshot_blocker(&as_linked).expect("an ordinary directory inside the source blocks the rung");
    assert!(blocker.contains(&moved.join("nested/wt").display().to_string()), "{blocker}");
    assert!(blocker.contains("inside the source is an ordinary directory"), "{blocker}");
    remove(root.as_path());
}

/// A worktree whose every file equals its commit by content has an answer,
/// not a failure:
/// nothing is listed,
/// whether the stat
///  data of a tracked file is stale or the index is older than the files;
/// a real edit is listed with its path.
#[test]
fn changed_paths_lists_nothing_for_files_the_stat_data_cannot_vouch_for_and_one_path_for_an_edit() {
    let root: PathBuf = crate::test_support::fixture("changed-paths-empty");
    let repo: PathBuf = repository(root.as_path(), "repo");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    commit_all(repo.as_path(), "first");
    let context = git_context();
    let listed = changed_paths(&context, repo.as_path()).expect("a clean worktree has an answer");
    assert_eq!((listed.count, listed.first.clone(), listed.attributes_changed), (0, None, false), "positive control: clean");
    // The same bytes with another modification time than the index entry records.
    let file = std::fs::File::options().write(true).open(repo.join("shared.txt")).expect("open");
    file.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(30)).expect("touch");
    let touched = changed_paths(&context, repo.as_path()).expect("a touched file has an answer");
    assert_eq!((touched.count, touched.first.clone()), (0, None), "a touched file of the same bytes is no change");
    assert!(!touched.has_changes());
    std::fs::write(repo.join("shared.txt"), b"edited\n").expect("edit");
    let edited = changed_paths(&context, repo.as_path()).expect("an edit has an answer");
    assert_eq!((edited.count, edited.first.as_deref()), (1, Some("shared.txt")));
    assert!(edited.has_changes());
    remove(root.as_path());
}

/// An untracked attributes file deep in the tree is noticed even when the caller's environment turns pathspec
/// magic off:
/// the listing's own `:(glob)**/.gitattributes` is magic,
/// and a missed one would let a restore run under rules it never checked.
#[test]
fn changed_paths_notices_an_untracked_deep_attributes_file_under_literal_pathspec_mode() {
    let root: PathBuf = crate::test_support::fixture("changed-paths-literal-mode");
    let repo: PathBuf = repository(root.as_path(), "repo");
    tracked(repo.as_path(), "top.txt", b"top\n");
    commit_all(repo.as_path(), "first");
    std::fs::create_dir_all(repo.join("sub")).expect("sub");
    std::fs::write(repo.join("sub/.gitattributes"), b"*.txt text\n").expect("attributes");
    let mut context = git_context();
    context.overlay.push((
        std::ffi::OsString::from("GIT_LITERAL_PATHSPECS"),
        std::ffi::OsString::from("1"),
    ));
    let listed = changed_paths(&context, repo.as_path()).expect("an untracked attributes file has an answer");
    assert!(listed.attributes_changed, "an untracked deep attributes file must be noticed");
    remove(root.as_path());
}

/// Git failing is a failure with its words:
/// the run,
/// the exit code and what Git said,
/// so the person can read the same error.
#[test]
fn changed_paths_names_the_run_the_exit_code_and_what_git_said_when_git_fails() {
    let root: PathBuf = crate::test_support::fixture("changed-paths-failure");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let stub: PathBuf = root.join("failing-git");
    crate::test_support::executable(stub.as_path(), b"#!/bin/sh\necho 'fatal: index file corrupt' >&2\nexit 128\n");
    let failing = crate::transaction_git::GitContext { real_git: stub.clone(), ..git_context() };
    let reason: String = changed_paths(&failing, repo.as_path()).expect_err("Git failed");
    assert!(reason.contains("`git status`") && reason.contains("exit code 128") && reason.contains("fatal: index file corrupt"), "{reason}");
    let missing = crate::transaction_git::GitContext { real_git: root.join("no-such-git"), ..git_context() };
    let unstarted: String = changed_paths(&missing, repo.as_path()).expect_err("Git did not start");
    assert!(unstarted.contains("`git status`") && unstarted.contains("could not be started"), "{unstarted}");
    remove(root.as_path());
}
