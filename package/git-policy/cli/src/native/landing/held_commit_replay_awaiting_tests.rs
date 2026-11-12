//! Controls for the awaited faces of a held batch's replay:
//! both faces carry the batch onto its branch the same way,
//! and the replay runs inside a task where the synchronous
//!  face is refused.

use super::{Held, Replay, onto, onto_awaiting, value_now, value_now_awaiting};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_git::GitContext;
use std::path::PathBuf;

/// The path the prepared commit changes.
const THE_PREPARED_PATH: &str = "prepared.txt";

/// The path another branch's commit changes,
/// so a replay onto it merges two sides.
const THE_MOVED_PATH: &str = "moved.txt";

/// What:
///  A repository whose branch holds one commit,
/// with a second commit prepared on top of it and then left off the
///  branch,
/// as a batch the service wrote and held.
/// Why:
///   The replay carries `prepared` onto the branch from the commit it was prepared on,
/// so the fixture is the one branch
///  value the record names and the one prepared commit whose parent is that value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); const old = head(); write(prepared.txt, 'prepared'); const prepared = commit(); reset('--hard', old);
/// ```
struct Prepared {
    /// The fixture directory the repository is made in.
    root: PathBuf,
    /// The worktree the batch is landed in.
    repo: PathBuf,
    /// The common Git directory.
    common_dir: PathBuf,
    /// The commit the prepared one was made on,
    /// which the branch still names.
    old: String,
    /// The prepared commit,
    /// whose parent is `old`.
    prepared: String,
}

/// One repository with a prepared commit that is not on the branch.
fn one_prepared_commit(name: &str) -> Prepared {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let old: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join(THE_PREPARED_PATH), b"prepared\n").expect("write the prepared path");
    git(repo.as_path(), &["add", "--", THE_PREPARED_PATH]);
    git(repo.as_path(), &["commit", "--quiet", "--message=the prepared commit"]);
    let prepared: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["reset", "--hard", old.as_str()]);
    let common_dir: PathBuf = repo.join(".git");
    return Prepared { root, repo, common_dir, old, prepared };
}

/// The record of the hold over one prepared commit,
/// written where the settlement reads it.
fn held_over(prepared: &Prepared) -> Held {
    return Held {
        seq: 1,
        journal: prepared.root.join("journal"),
        repository: prepared.repo.clone(),
        common_dir: prepared.common_dir.clone(),
        git_dir: prepared.common_dir.clone(),
        reference: String::from("refs/heads/main"),
        old: prepared.old.clone(),
        new: prepared.prepared.clone(),
        commits: vec![prepared.prepared.clone()],
        index: prepared.common_dir.join("index"),
        reflog: b"commit: the prepared commit".to_vec(),
        locks: Vec::new(),
        created: String::from("2026-11-11T00:00:00-05:00"),
    };
}

/// What:
///  Both faces carry the batch onto the branch the same way,
/// over a branch that did not move and over one that moved on another
///  path.
/// Why:
///   A commit's name is its content,
/// so two replays that write the same tree,
/// parent,
/// author,
/// committer and message
///  give one name,
/// and the batch a person reads is the one both faces made.
/// The branch that moved is the case the
///  replay exists for:
/// its two sides must merge,
/// and a face that merged another base writes another commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(replayOnto(prepared, head), await replayOntoAwaiting(prepared, head));
/// ```
#[test]
fn the_awaited_replay_carries_the_batch_what_the_synchronous_one_does() {
    let prepared: Prepared = one_prepared_commit("held-replay-awaiting-agrees");
    let context: GitContext = git_context();
    let held: Held = held_over(&prepared);
    let current: String = git_text(prepared.repo.as_path(), &["rev-parse", "HEAD"]);
    let synchronous: Replay = onto(&context, &held, current.as_str(), prepared.common_dir.as_path());
    let awaited: Replay =
        crate::lazy_runtime::run_on_runtime(onto_awaiting(&context, &held, current.as_str(), prepared.common_dir.as_path())).expect("the runtime");
    assert!(matches!(synchronous, Replay::Replayed { .. }), "the fixture's batch is carried onto a branch that did not move: {synchronous:?}");
    assert_eq!(synchronous, awaited, "the two faces carry the batch onto a branch that did not move the same way");
    // The branch moved on another path while the lock stood, so the replay merges the two sides.
    std::fs::write(prepared.repo.join(THE_MOVED_PATH), b"moved\n").expect("write the moved path");
    git(prepared.repo.as_path(), &["add", "--", THE_MOVED_PATH]);
    git(prepared.repo.as_path(), &["commit", "--quiet", "--message=a commit that landed"]);
    let moved: String = git_text(prepared.repo.as_path(), &["rev-parse", "HEAD"]);
    let synchronous_moved: Replay = onto(&context, &held, moved.as_str(), prepared.common_dir.as_path());
    let awaited_moved: Replay =
        crate::lazy_runtime::run_on_runtime(onto_awaiting(&context, &held, moved.as_str(), prepared.common_dir.as_path())).expect("the runtime");
    assert!(matches!(synchronous_moved, Replay::Replayed { .. }), "the fixture's two sides merge: {synchronous_moved:?}");
    assert_eq!(synchronous_moved, awaited_moved, "the two faces carry the batch onto a branch that moved the same way");
    assert_ne!(synchronous, synchronous_moved, "a branch that moved gives another commit than one that did not");
    assert_eq!(value_now(&context, &held).as_deref(), Some(moved.as_str()), "the branch the record names is still the one the fixture moved");
    remove(prepared.root.as_path());
}

/// What:
///  The replay runs inside a task,
/// and the synchronous face of the same replay is refused there.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// so a settlement
///  that is a task carries a held batch onto its branch through this face;
/// the commit it answers is the synchronous face's,
///  which is what pins that every child ran and was awaited rather than abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(await runtime.spawn(() => ontoAwaiting(...)) === replayed); assert.deepEqual(await runtime.spawn(() => onto(...)), { cannot: 'the commit ... cannot be read' });
/// ```
#[test]
fn the_replay_is_made_inside_a_task_where_the_synchronous_one_is_refused() {
    let prepared: Prepared = one_prepared_commit("held-replay-awaiting-in-a-task");
    let context: GitContext = git_context();
    let held: Held = held_over(&prepared);
    let current: String = git_text(prepared.repo.as_path(), &["rev-parse", "HEAD"]);
    let expected: Replay = onto(&context, &held, current.as_str(), prepared.common_dir.as_path());
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let answered: Replay = runtime.block_on(async {
        return onto_awaiting(&context, &held, current.as_str(), prepared.common_dir.as_path()).await;
    });
    assert_eq!(answered, expected, "the awaited face carries the batch the synchronous one carried, inside a task");
    let refused: Replay = runtime.block_on(async { return onto(&context, &held, current.as_str(), prepared.common_dir.as_path()); });
    assert_eq!(
        refused,
        Replay::Cannot { because: format!("the commit {} cannot be read", prepared.prepared) },
        "the synchronous face cannot start its Git from inside the runtime, which is why a task asks the awaited face"
    );
    let value: Option<String> = runtime.block_on(async { return value_now_awaiting(&context, &held).await; });
    assert_eq!(value, value_now(&context, &held), "the awaited look at the branch answers what the synchronous one answers inside a task");
    remove(prepared.root.as_path());
}
