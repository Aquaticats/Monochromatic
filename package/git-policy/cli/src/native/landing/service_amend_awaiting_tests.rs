//! Controls for the awaited reading of the commit an amend replaces:
//! it reads the commit the synchronous reading reads,
//! for a plain commit,
//! a root commit and a merge,
//!  and a Git that cannot run is refused in the same words.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// What:
///  A repository with a root commit,
/// a second commit on top of it and a side branch that is merged,
///  and the commit names they gave.
/// Why:
///  An amend reads the parents,
/// the first parent's tree,
/// the author and the message of the commit it replaces,
/// so the
///  control needs a commit with a parent,
/// one without,
/// and one with two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); const root = head(); commit(repo, 'second'); git(repo, ['checkout', '-b', 'side', root]); commit(repo, 'side'); git(repo, ['checkout', 'main']); git(repo, ['merge', '--no-ff', 'side']);
/// ```
fn a_history_of_every_shape(name: &str) -> (PathBuf, PathBuf, String, String, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let root_commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a file");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let second: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "side", root_commit.as_str()]);
    std::fs::write(repo.join("b.txt"), "two\n").expect("a file");
    git(repo.as_path(), &["add", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=side"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    git(repo.as_path(), &["merge", "--quiet", "--no-ff", "--no-edit", "side"]);
    let merge: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (root, repo, root_commit, second, merge);
}

/// What:
///  What one reading of the amend's source finds,
///  through both faces,
///  so a control compares them without asking which is which.
/// Why:
///  The awaited face must carry over what the synchronous one carries over;
/// a difference here is an amend that keeps
///  another commit's parents,
/// tree,
/// author or message once the commit path is a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [sync, awaited] = bothReadings(context, repo);
/// ```
fn both_readings(context: &GitContext, repo: &Path) -> (Result<Option<AmendSource>, TransactionStop>, Result<Option<AmendSource>, TransactionStop>) {
    let synchronous: Result<Option<AmendSource>, TransactionStop> = read_amend_source(context, repo);
    let awaited: Result<Option<AmendSource>, TransactionStop> =
        crate::lazy_runtime::run_on_runtime(read_amend_source_awaiting(context, repo)).expect("the runtime");
    return (synchronous, awaited);
}

/// What:
///  Both faces read the same commit,
/// for a commit with a parent,
/// a root commit and a merge,
///  and the awaited face reads it inside the runtime,
/// where the synchronous face cannot start Git at all.
/// Why:
///  An amend that carried over nothing would be recorded as a new commit by native Git,
/// and one that carried over
///  another commit's parents would rewrite history the person did not name;
/// the merge is what tells the first parent's
///  tree from the commit's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const commit of [merge, second, root]) { git(repo, ['checkout', '--detach', commit]); assert.deepEqual(...bothReadings()); }
/// ```
#[test]
fn the_awaited_amend_source_is_what_the_synchronous_one_reads() {
    let (root, repo, root_commit, second, merge) = a_history_of_every_shape("amend-awaiting-agrees");
    let context: GitContext = git_context();
    for (label, commit, parents) in [("a merge", merge.as_str(), 2usize), ("a commit with a parent", second.as_str(), 1), ("a root commit", root_commit.as_str(), 0)] {
        git(repo.as_path(), &["checkout", "--quiet", "--detach", commit]);
        let (synchronous, awaited) = both_readings(&context, repo.as_path());
        let read: &AmendSource = synchronous.as_ref().expect("the reading succeeds").as_ref().expect("the commit is covered");
        assert_eq!(read.parents.len(), parents, "{label}: the synchronous reading is what the two faces are compared on");
        assert_eq!(
            awaited.as_ref().ok().cloned(),
            synchronous.as_ref().ok().cloned(),
            "{label}: the two faces read one commit"
        );
    }
    remove(root.as_path());
}

/// What:
///  A Git that cannot be run is refused in the words the synchronous face refuses it in.
/// Why:
///  The words are the stop the amending command prints,
/// so a face that reworded them tells the person about another
///  problem than the one that stopped it;
/// the same reading through both faces is what keeps one sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const context = { ...gitContext, realGit: '/nonexistent/git' }; assert.equal(wordOf(await readAmendSourceAwaiting(context, repo)), wordOf(readAmendSource(context, repo)));
/// ```
#[test]
fn a_git_that_cannot_run_is_refused_in_the_words_of_the_synchronous_face() {
    let (root, repo, _root_commit, _second, _merge) = a_history_of_every_shape("amend-awaiting-refused");
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent-git-for-a-control");
    let synchronous: Result<Option<AmendSource>, TransactionStop> = read_amend_source(&context, repo.as_path());
    let awaited: Result<Option<AmendSource>, TransactionStop> =
        crate::lazy_runtime::run_on_runtime(read_amend_source_awaiting(&context, repo.as_path())).expect("the runtime");
    assert!(synchronous.is_err(), "a Git that cannot be run covers no commit: {synchronous:?}");
    assert_eq!(
        format!("{:?}", awaited),
        format!("{:?}", synchronous),
        "the two faces refuse a Git that cannot be run in one sentence"
    );
    remove(root.as_path());
}
