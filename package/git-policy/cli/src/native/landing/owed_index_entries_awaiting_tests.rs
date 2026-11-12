//! Controls for the awaited faces of the entries this module reads:
//! both faces read one tree,
//! one index,
//! one whole listing and one move the same,
//! and a read whose Git must be
//!  asked is answered inside a task where the synchronous face is refused.

use super::{Entry, head_entries, head_entries_awaiting, index_entries, index_entries_awaiting, moved_since_listing, moved_since_listing_awaiting, tree_entries, tree_entries_awaiting, whole_listing, whole_listing_awaiting};
use crate::service_protocol::FailureCode;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::transaction_git::GitContext;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The paths the controls ask about:
/// two names that differ only in case,
/// and one below the top of the worktree.
const THE_PATHS: [&[u8]; 3] = [b"A.txt", b"a.txt", b"sub/f"];

/// What:
///  A repository whose branch holds two commits over three paths,
/// whose index was copied before it moved on,
/// and whose
///  index has since staged another content there.
/// Why:
///  The five reads of this module answer differently over such a repository:
/// a tree holds the landed
///  content,
/// the index holds what was staged after it,
/// and the copy holds what the index held before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); commit('base', ['A.txt', 'a.txt', 'sub/f']); copy(index, 'commit.index'); stage('sub/f', 'two')
/// ```
fn a_repository_whose_index_moved(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir_all(repo.join("sub")).expect("sub directory");
    std::fs::write(repo.join("A.txt"), b"upper\n").expect("write A.txt");
    std::fs::write(repo.join("a.txt"), b"lower\n").expect("write a.txt");
    std::fs::write(repo.join("sub/f"), b"one\n").expect("write sub/f");
    git(repo.as_path(), &["add", "--", "A.txt", "a.txt", "sub/f"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let index: PathBuf = repo.join(".git").join("index");
    let copy: PathBuf = root.join("commit.index");
    std::fs::copy(index.as_path(), copy.as_path()).expect("a copy of the index");
    std::fs::write(repo.join("sub/f"), b"two\n").expect("write sub/f again");
    git(repo.as_path(), &["add", "--", "sub/f"]);
    return (root, repo, copy);
}

/// The paths as this module's questions name them.
fn the_paths() -> Vec<Vec<u8>> {
    let mut named: Vec<Vec<u8>> = Vec::with_capacity(THE_PATHS.len());
    for path in THE_PATHS {
        named.push(path.to_vec());
    }
    return named;
}

/// What:
///  The answer of each of the five faces over the same repository,
/// the synchronous one first.
/// Why:
///  A control compares them without asking which face answered,
/// and names in its own words which of the five
///  disagreed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pair = (sync, awaited) => ({ sync: sync(), awaited: await awaiting() });
/// ```
fn both_answers<T, F: std::future::Future<Output = T>>(synchronous: T, awaited: F) -> (T, T) {
    let now: T = synchronous;
    let later: T = crate::lazy_runtime::run_on_runtime(awaited).expect("the runtime");
    return (now, later);
}

/// What:
///  The answer of one read made inside a task of `runtime`.
/// Why:
///  The controls that pin a read's place on the runtime run every read as a task,
/// which is what a commit path that
///  is a task does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await runtime.spawn(work).await;
/// ```
fn in_a_task<T, F>(runtime: &tokio::runtime::Runtime, work: F) -> T
where
    T: Send + 'static,
    F: std::future::Future<Output = T> + Send + 'static,
{
    return runtime.block_on(async { return tokio::spawn(work).await.expect("the task that reads ends"); });
}

/// What:
///  Both faces answer one thing about a tree,
/// an index,
/// a whole listing,
/// the head's entries and the paths that moved since a copy.
/// Why:
///   The repair reads these through the synchronous face today and through the awaited one once the commit path is a task,
/// so the two must agree path for path,
/// or a commit reads another index once it is a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(...bothAnswers(() => treeEntries(...), treeEntriesAwaiting(...)));
/// ```
#[test]
fn the_awaited_reads_answer_what_the_synchronous_ones_answer() {
    let (root, repo, copy) = a_repository_whose_index_moved("owed-entries-awaiting-agrees");
    let context: GitContext = git_context();
    let index: PathBuf = repo.join(".git").join("index");
    let names: Vec<Vec<u8>> = the_paths();
    let (tree_sync, tree_later) = {
        let paths_tree: Vec<Vec<u8>> = names.clone();
        let (for_tree, at_tree) = (context.clone(), repo.clone());
        both_answers(tree_entries(&context, repo.as_path(), "HEAD", paths_tree.as_slice()), async move {
            return tree_entries_awaiting(&for_tree, at_tree.as_path(), "HEAD", paths_tree.as_slice()).await;
        })
    };
    assert_eq!(tree_sync, tree_later, "the two faces answer one thing about the entries of a tree");
    let (index_sync, index_later) = {
        let paths_index: Vec<Vec<u8>> = names.clone();
        let (for_index, at_index, at_file) = (context.clone(), repo.clone(), index.clone());
        both_answers(index_entries(&context, repo.as_path(), index.as_path(), paths_index.as_slice()), async move {
            return index_entries_awaiting(&for_index, at_index.as_path(), at_file.as_path(), paths_index.as_slice()).await;
        })
    };
    assert_eq!(index_sync, index_later, "the two faces answer one thing about the entries of an index");
    let (whole_sync, whole_later) = {
        let (for_whole, at_whole, at_whole_file) = (context.clone(), repo.clone(), index.clone());
        both_answers(whole_listing(&context, repo.as_path(), index.as_path()), async move {
            return whole_listing_awaiting(&for_whole, at_whole.as_path(), at_whole_file.as_path()).await;
        })
    };
    assert_eq!(whole_sync, whole_later, "the two faces answer one thing about a whole listing of an index");
    let (head_sync, head_later) = {
        let paths_head: Vec<Vec<u8>> = names.clone();
        let (for_head, at_head) = (context.clone(), repo.clone());
        both_answers(head_entries(&context, repo.as_path(), paths_head.as_slice()), async move {
            return head_entries_awaiting(&for_head, at_head.as_path(), paths_head.as_slice()).await;
        })
    };
    assert_eq!(head_sync, head_later, "the two faces answer one thing about the entries of the worktree's head");
    let (moved_sync, moved_later) = {
        let (for_moved, at_moved, file_moved, held_moved) = (context.clone(), repo.clone(), index.clone(), copy.clone());
        both_answers(moved_since_listing(&context, repo.as_path(), index.as_path(), copy.as_path()), async move {
            return moved_since_listing_awaiting(&for_moved, at_moved.as_path(), file_moved.as_path(), held_moved.as_path()).await;
        })
    };
    assert_eq!(moved_sync, moved_later, "the two faces name one set of paths that moved since the commit copied the index");
    // A listing that answered nothing would make every comparison above equal for the wrong reason.
    let listed: BTreeMap<Vec<u8>, Entry> = tree_sync.expect("the tree of the fixture reads");
    assert_eq!(listed.len(), THE_PATHS.len(), "the fixture's tree holds every path the control asks about");
    assert_eq!(moved_sync, Ok(vec![b"sub/f".to_vec()]), "the fixture's copy is one staged path behind the index");
    remove(root.as_path());
}

/// What:
///  Each awaited read answers inside a task,
/// and the synchronous read of the same question is refused there.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// so a commit path that
///  is a task needs these faces;
/// the answers they give are Git's own,
/// which is what pins that the children ran and were awaited
///  rather than abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = await runtime.spawn(() => indexEntriesAwaiting(...)); const refused = await runtime.spawn(() => indexEntries(...));
/// ```
#[test]
fn the_awaited_reads_answer_inside_a_task_where_the_synchronous_ones_are_refused() {
    let (root, repo, copy) = a_repository_whose_index_moved("owed-entries-awaiting-in-a-task");
    let context: GitContext = git_context();
    let index: PathBuf = repo.join(".git").join("index");
    let names: Vec<Vec<u8>> = the_paths();
    let expected_tree: Result<BTreeMap<Vec<u8>, Entry>, FailureCode> = tree_entries(&context, repo.as_path(), "HEAD", names.as_slice());
    let expected_index: Result<BTreeMap<Vec<u8>, Entry>, FailureCode> = index_entries(&context, repo.as_path(), index.as_path(), names.as_slice());
    let expected_head: Result<BTreeMap<Vec<u8>, Entry>, FailureCode> = head_entries(&context, repo.as_path(), names.as_slice());
    let expected_moved: Result<Vec<Vec<u8>>, FailureCode> = moved_since_listing(&context, repo.as_path(), index.as_path(), copy.as_path());
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let (for_tree, at_tree, paths_tree) = (context.clone(), repo.clone(), names.clone());
    let tree = in_a_task(&runtime, async move { return tree_entries_awaiting(&for_tree, at_tree.as_path(), "HEAD", paths_tree.as_slice()).await; });
    let (for_index, at_index, paths_index, file_index) = (context.clone(), repo.clone(), names.clone(), index.clone());
    let one = in_a_task(&runtime, async move { return index_entries_awaiting(&for_index, at_index.as_path(), file_index.as_path(), paths_index.as_slice()).await; });
    let (for_head, at_head, paths_head) = (context.clone(), repo.clone(), names.clone());
    let head = in_a_task(&runtime, async move { return head_entries_awaiting(&for_head, at_head.as_path(), paths_head.as_slice()).await; });
    let (for_moved, at_moved, file_moved, held_moved) = (context.clone(), repo.clone(), index.clone(), copy.clone());
    let moved = in_a_task(&runtime, async move { return moved_since_listing_awaiting(&for_moved, at_moved.as_path(), file_moved.as_path(), held_moved.as_path()).await; });
    assert_eq!(tree, expected_tree, "the awaited face answers the tree of the fixture inside a task");
    assert_eq!(one, expected_index, "the awaited face answers the index of the fixture inside a task");
    assert_eq!(head, expected_head, "the awaited face answers the head of the fixture inside a task");
    assert_eq!(moved, expected_moved, "the awaited face answers the paths that moved inside a task");
    let (for_sync, at_sync, file_sync, paths_sync) = (context.clone(), repo.clone(), index.clone(), names.clone());
    let asking = in_a_task(&runtime, async move { return index_entries(&for_sync, at_sync.as_path(), file_sync.as_path(), paths_sync.as_slice()); });
    assert_eq!(
        asking,
        Err(FailureCode::Unavailable),
        "the synchronous face cannot start its Git from inside the runtime, which is why a task asks the awaited face"
    );
    remove(root.as_path());
}
