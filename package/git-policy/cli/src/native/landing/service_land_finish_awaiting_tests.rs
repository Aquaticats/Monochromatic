//! Controls for the awaited Git questions a restart decides an open batch with:
//! they decide what the synchronous questions decide,
//! for a branch at
//!  the old value,
//! at the new one,
//! past it,
//! gone and elsewhere,
//! and a Git that gives no answer decides nothing on either face.

use super::*;
/// The awaited decision of an open batch lives beside the synchronous one,
/// in the file the restart's branch questions are kept in.
use super::branch_questions::decision_for_awaiting;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// What:
///  An intent of one batch that names `repository` and `reference`,
///  the branch value before the batch and the commit it landed.
/// Why:
///  The decision reads four fields of an intent,
/// and a control makes them itself so it needs no journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intent = { seq: 1, repository, commonDir: join(repository, '.git'), reference: 'refs/heads/main', old, new, ids: [], commits: [], paths: [], derived: false, cleanup: null, index: join(repository, '.git/index'), gitDir: join(repository, '.git') };
/// ```
fn an_intent(repository: &Path, reference: &str, old: &str, new: &str) -> Intent {
    let common: PathBuf = repository.join(".git");
    return Intent {
        seq: 1,
        repository: repository.to_path_buf(),
        common_dir: common.clone(),
        reference: reference.to_string(),
        old: old.to_string(),
        new: new.to_string(),
        ids: Vec::new(),
        commits: Vec::new(),
        paths: Vec::new(),
        derived: false,
        cleanup: None,
        index: common.join("index"),
        git_dir: common,
    };
}

/// The decision one intent gets through both faces.
fn both_decisions(git: &GitContext, intent: &Intent) -> (Option<Recovery>, Option<Recovery>) {
    let synchronous: Option<Recovery> = decision_for(git, intent);
    let awaited: Option<Recovery> = crate::lazy_runtime::run_on_runtime(decision_for_awaiting(git, intent)).expect("the runtime");
    return (synchronous, awaited);
}

/// What:
///  Both faces decide one open batch the same way,
///  and the awaited face decides it inside the runtime,
/// where the
///  synchronous face cannot start Git at all.
/// Why:
///  The decision is the difference between finishing a batch whose commit landed and dropping one that never
///  did,
/// so a face that decided differently would forget a record of a commit the person made or keep
///  one that will never land.
/// Every arm is here:
/// the branch at the new commit,
/// the branch past it,
/// at the old
///  one,
/// gone,
/// and moved elsewhere,
/// because only the first two are decided by the same answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(...bothDecisions(git, intent)); assert.equal(awaited, 'finish');
/// ```
#[test]
fn the_awaited_decision_is_the_one_the_synchronous_face_makes() {
    let root: PathBuf = fixture("finish-awaiting-decision");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let old: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let new: String = git_text(repo.as_path(), &["commit-tree", "-m", "the batch", tree.as_str()]);
    let past: String = git_text(repo.as_path(), &["commit-tree", "-p", new.as_str(), "-m", "after the batch", tree.as_str()]);
    let elsewhere: String = git_text(repo.as_path(), &["commit-tree", "-m", "elsewhere", tree.as_str()]);
    let intent: Intent = an_intent(repo.as_path(), "refs/heads/main", old.as_str(), new.as_str());
    git(repo.as_path(), &["update-ref", "refs/heads/main", new.as_str()]);
    assert_eq!(both_decisions(&context, &intent).0, Some(Recovery::Finish), "a branch at the batch's commit holds the batch");
    assert_eq!(both_decisions(&context, &intent), (Some(Recovery::Finish), Some(Recovery::Finish)), "both faces finish a batch the branch shows");
    git(repo.as_path(), &["update-ref", "refs/heads/main", past.as_str()]);
    assert_eq!(
        both_decisions(&context, &intent),
        (Some(Recovery::Finish), Some(Recovery::Finish)),
        "both faces finish a batch a later commit sits on top of"
    );
    git(repo.as_path(), &["update-ref", "refs/heads/main", old.as_str()]);
    assert_eq!(
        both_decisions(&context, &intent),
        (Some(Recovery::Drop), Some(Recovery::Drop)),
        "both faces drop a batch the branch never moved for"
    );
    git(repo.as_path(), &["update-ref", "refs/heads/main", elsewhere.as_str()]);
    assert_eq!(
        both_decisions(&context, &intent),
        (Some(Recovery::Drop), Some(Recovery::Drop)),
        "both faces drop a batch the branch moved away from"
    );
    git(repo.as_path(), &["update-ref", "-d", "refs/heads/main"]);
    assert_eq!(both_decisions(&context, &intent), (Some(Recovery::Drop), Some(Recovery::Drop)), "both faces drop a batch whose branch is gone");
    remove(root.as_path());
}

/// What:
///  A Git that gives no answer decides nothing through either face.
/// Why:
///  The owner's rule is "never discard user data",
/// and a batch is dropped on an answer that the branch never moved:
/// a Git that could not be started has said nothing,
/// so both faces answer nothing and the intent is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blind = { ...gitContext, realGit: '/nonexistent/git' }; assert.deepEqual(...bothDecisions(blind, intent), [undefined, undefined]);
/// ```
#[test]
fn a_git_that_gives_no_answer_decides_nothing_through_either_face() {
    let root: PathBuf = fixture("finish-awaiting-no-answer");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent-git-for-a-control");
    let old: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let new: String = git_text(repo.as_path(), &["commit-tree", "-m", "the batch", "HEAD^{tree}"]);
    let intent: Intent = an_intent(repo.as_path(), "refs/heads/main", old.as_str(), new.as_str());
    assert_eq!(
        both_decisions(&context, &intent),
        (None, None),
        "a Git that cannot be asked leaves the batch open on both faces"
    );
    remove(root.as_path());
}
