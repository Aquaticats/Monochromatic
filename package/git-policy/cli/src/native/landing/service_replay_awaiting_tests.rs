//! Controls for the awaited faces of the replay's own reads:
//! both faces answer one tree from one private index and one commit,
//! a private index that gives no answer is told apart from one that
//!  is gone,
//! and both faces answer inside a task where the synchronous ones are refused.

use super::{captured_tree, captured_tree_awaiting, tree_of_commit, tree_of_commit_awaiting};
use crate::test_support::{fixture, git_context, git_text, remove, repository};
use crate::transaction_git::GitContext;
use std::path::PathBuf;

/// The file the prepared change touches.
const THE_PREPARED_PATH: &str = "prepared.txt";

/// What:
///  A repository whose worktree holds one commit,
///  a private index holding a staged change,
///  and the object directory a replay writes into.
/// Why:
///   The replay reads the change out of the private index of the attempt that captured it,
/// which is the file this control
///  asks both faces about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); stage('prepared.txt'); copy(index, privateIndex); return { repo, privateIndex, objects: join(repo, '.git/objects') }
/// ```
struct Staged {
    /// The fixture directory.
    root: PathBuf,
    /// The worktree the change is staged in.
    repo: PathBuf,
    /// The private index of the attempt that captured the change.
    private_index: PathBuf,
    /// The object store the write-tree reads.
    objects: PathBuf,
}

/// One repository with a staged change in a private index of its own.
fn one_staged_change(name: &str) -> Staged {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(THE_PREPARED_PATH), b"prepared\n").expect("write the prepared path");
    let git_dir: PathBuf = repo.join(".git");
    let private_index: PathBuf = root.join("captured.index");
    let output: std::process::Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(repo.as_path())
        .env("GIT_INDEX_FILE", private_index.as_path())
        .args(["add", "--", THE_PREPARED_PATH])
        .output()
        .expect("Git runs");
    assert!(output.status.success(), "the private index is staged: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    return Staged { root, repo, private_index, objects: git_dir.join("objects") };
}

/// What:
///  Both faces answer one tree from one commit,
/// and one tree from one private index.
/// Why:
///   The commit a replay makes is written on the tree these two answer,
/// so a face that read another commit or another
///  index writes another commit than the one the person asked for,
/// and a commit's name is its content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(treeOfCommit(commit), await treeOfCommitAwaiting(commit));
/// ```
#[test]
fn the_awaited_reads_of_the_replay_answer_what_the_synchronous_ones_answer() {
    let staged: Staged = one_staged_change("replay-awaiting-agrees");
    let context: GitContext = git_context();
    let head: String = git_text(staged.repo.as_path(), &["rev-parse", "HEAD"]);
    let synchronous_tree: String = tree_of_commit(&context, staged.repo.as_path(), head.as_str()).expect("the commit's tree reads");
    let awaited_tree: String =
        crate::lazy_runtime::run_on_runtime(tree_of_commit_awaiting(&context, staged.repo.as_path(), head.as_str())).expect("the runtime").expect("the commit's tree reads");
    assert_eq!(synchronous_tree, awaited_tree, "the two faces answer one tree for one commit");
    let synchronous_captured: String =
        captured_tree(&context, staged.repo.as_path(), staged.private_index.as_path(), staged.objects.as_path()).expect("the captured change is written out");
    let awaited_captured: String = crate::lazy_runtime::run_on_runtime(captured_tree_awaiting(&context, staged.repo.as_path(), staged.private_index.as_path(), staged.objects.as_path()))
        .expect("the runtime")
        .expect("the captured change is written out");
    assert_eq!(synchronous_captured, awaited_captured, "the two faces answer one tree for one private index");
    let staged_tree: String = git_text(staged.repo.as_path(), &["rev-parse", format!("{head}^{{tree}}").as_str()]);
    assert_ne!(synchronous_captured, staged_tree, "the private index holds a change the commit does not");
    remove(staged.root.as_path());
}

/// What:
///  A private index that is gone and one whose look gives no answer each stop the commit,
///  with their own sentences.
/// Why:
///   `git write-tree` answers the empty tree for an index that is gone,
/// so a look that found nothing would rebuild the commit as a
///  deletion of every file;
/// a look nobody answered in time must not be a look that found nothing either,
/// and no fixture on this
///  host makes a look wait in the kernel,
/// so the answer is stated for the job the way a control states the two answers a
///  thread refusal gives (`foundation/the_answer_a_control_states.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.match(await capturedTreeAwaiting(gone).catch(String), /is not there/); stateTheAnswer('noAnswer'); assert.match(..., /could not be looked at in time/)
/// ```
#[test]
fn a_private_index_that_is_gone_or_gives_no_answer_stops_the_commit() {
    let staged: Staged = one_staged_change("replay-awaiting-stops");
    let context: GitContext = git_context();
    let gone: PathBuf = staged.root.join("gone.index");
    let synchronous: String = format!(
        "{:?}",
        captured_tree(&context, staged.repo.as_path(), gone.as_path(), staged.objects.as_path()).expect_err("a private index that is not there stops the commit")
    );
    assert!(synchronous.contains("is not there"), "the synchronous face's sentence says the index is gone: {synchronous}");
    let refused: String = format!(
        "{:?}",
        crate::lazy_runtime::run_on_runtime(captured_tree_awaiting(&context, staged.repo.as_path(), gone.as_path(), staged.objects.as_path()))
            .expect("the runtime")
            .expect_err("a private index that is not there stops the commit")
    );
    assert!(refused.contains("is not there"), "the awaited face's sentence says the index is gone too: {refused}");
    // A look that gave nothing in time is stated for the job, because no fixture on this host makes one wait in the kernel.
    let unanswered: String = {
        let _stated = crate::blocking_jobs_within_the_budget::a_controls_answer::state_the_answer(crate::blocking_jobs_within_the_budget::a_controls_answer::AStatesAnswer::NoAnswer);
        format!(
            "{:?}",
            crate::lazy_runtime::run_on_runtime(captured_tree_awaiting(&context, staged.repo.as_path(), staged.private_index.as_path(), staged.objects.as_path()))
                .expect("the runtime")
                .expect_err("a private index whose look gives no answer stops the commit")
        )
    };
    assert!(unanswered.contains("could not be"), "the sentence says what happened rather than that the index is gone: {unanswered}");
    assert!(!unanswered.contains("is not there"), "a look nobody answered is never answered as an index that is gone: {unanswered}");
    remove(staged.root.as_path());
}

/// What:
///  Both reads answer inside a task,
/// and the synchronous faces of the same reads are refused there.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// so a replay that is a
///  task reads its commit and its captured change through these faces;
/// the answers they give are the synchronous ones,
///  which is what pins that their children ran and were awaited rather than abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(await runtime.spawn(() => treeOfCommitAwaiting(head)), treeOfCommit(head));
/// ```
#[test]
fn the_reads_answer_inside_a_task_where_the_synchronous_ones_are_refused() {
    let staged: Staged = one_staged_change("replay-awaiting-in-a-task");
    let context: GitContext = git_context();
    let head: String = git_text(staged.repo.as_path(), &["rev-parse", "HEAD"]);
    let expected_tree: String = tree_of_commit(&context, staged.repo.as_path(), head.as_str()).expect("the commit's tree reads");
    let expected_captured: String =
        captured_tree(&context, staged.repo.as_path(), staged.private_index.as_path(), staged.objects.as_path()).expect("the captured change is written out");
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let tree: String = runtime.block_on(async { return tree_of_commit_awaiting(&context, staged.repo.as_path(), head.as_str()).await.expect("the commit's tree reads in a task"); });
    assert_eq!(tree, expected_tree, "the awaited face answers the commit's tree inside a task");
    let captured: String = runtime
        .block_on(async { return captured_tree_awaiting(&context, staged.repo.as_path(), staged.private_index.as_path(), staged.objects.as_path()).await })
        .expect("the captured change is written out in a task");
    assert_eq!(captured, expected_captured, "the awaited face answers the captured change inside a task");
    let refused: String = format!("{:?}", runtime.block_on(async { return tree_of_commit(&context, staged.repo.as_path(), head.as_str()); }).expect_err("the synchronous face cannot start its Git from inside the runtime"));
    assert!(refused.contains("from inside itself"), "the refusal says the runtime was asked for from inside itself: {refused}");
    remove(staged.root.as_path());
}
