//! Controls for the awaited faces of the side references a landing leaves behind:
//! both faces clear the same fixtures the same way,
//! a look at a batch's intent that gives no answer
//!  sets no removal in motion,
//! and a task can clear a dropped batch where the synchronous face cannot.

use super::*;
use crate::blocking_jobs_within_the_budget::a_controls_answer::{AStatesAnswer, state_the_answer};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

/// The batch every control here names.
const THE_BATCH: u64 = 1;

/// What:
///  A repository whose batch `THE_BATCH` left a scratch reference naming its only commit,
/// with a journal of its own.
/// Why:
///  That is the state a landing that ended before its intent reached the journal leaves,
/// and it is what the removal
///  is asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); git(repo, ['update-ref', scratchRef(THE_BATCH), headOf(repo)]);
/// ```
fn a_repository_with_a_dropped_batch(name: &str) -> (PathBuf, PathBuf, PathBuf, GitContext, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("a journal directory");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-ref", scratch_ref(THE_BATCH).as_str(), head.as_str()]);
    let context: GitContext = GitContext { common_dir: Some(repo.join(".git")), ..git_context() };
    return (root, repo, journal, context, head);
}

/// What:
///  The kept reference a control reads back after a removal,
/// which is the commit the person is told to look at.
/// Why:
///  A removal that removes the scratch reference without keeping its commit first loses the commit,
/// so the answer of the removal
///  alone is not the whole result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = valueOf(git, repo, keptRef(THE_BATCH));
/// ```
fn kept_after(git: &GitContext, repo: &Path) -> RefValue {
    return value_of(git, repo, kept_ref(THE_BATCH).as_str());
}

/// What:
///  The context the removal of a failed landing's batch reads,
/// which is the landing's own with this fixture's journal.
/// Why:
///  The intent the removal looks for is written in the instance's journal,
/// so a control must point
///  the removal at one it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landing = new LandContext({ git, journal, onObjectsWritten: null, lockCeiling });
/// ```
fn the_landing_context(git: &GitContext, journal: &Path) -> LandContext {
    return LandContext {
        git: git.clone(),
        journal: journal.to_path_buf(),
        on_objects_written: None,
        lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING,
    };
}

/// What:
///  One request that names the fixture's repository,
/// which is all the removal of a failed landing reads of a batch.
/// Why:
///  A failed landing's batch is described by its requests,
/// so a control hands the removal the request that names the batch
///  it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { id, repository: repo, commonDir: join(repo, '.git'), gitDir: join(repo, '.git'), reference, base, changes: [], ... };
/// ```
fn a_request_for(repo: &Path) -> CommitRequest {
    let common: PathBuf = repo.join(".git");
    return CommitRequest {
        id: String::from("a-control-request"),
        repository: repo.to_path_buf(),
        common_dir: common.clone(),
        git_dir: common.clone(),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: "0".repeat(40),
        changes: Vec::new(),
        message: String::from("a control\n"),
        author: None,
        committer: String::from("A Control <control@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: common.join("index"),
        scratch: common.join("monochromatic-git-policy-cli-service-a-control"),
        shadow: None,
        captured: 0,
        holder: String::new(),
    };
}

/// What:
///  The awaited clearing removes and keeps exactly what the synchronous one does.
/// Why:
///  The two faces are one recording of the same person's work,
/// so a difference in what they keep or remove is a commit that
///  survives on one path and not the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(await clearAfterDroppedBatchAwaiting(...), clearAfterDroppedBatch(...)); assert.deepEqual(keptA, keptB)
/// ```
#[test]
fn the_awaited_clearing_answers_what_the_synchronous_one_answers() {
    let (root, repo, _journal, context, head) = a_repository_with_a_dropped_batch("scratch-ref-faces");
    let synchronous: ScratchRefRemoval = clear_after_dropped_batch(&context, repo.as_path(), THE_BATCH);
    let synchronous_kept: RefValue = kept_after(&context, repo.as_path());
    let (awaited_root, awaited_repo, _awaited_journal, awaited_context, awaited_head) = a_repository_with_a_dropped_batch("scratch-ref-faces-awaited");
    let awaited: ScratchRefRemoval = crate::lazy_runtime::run_on_runtime(clear_after_dropped_batch_awaiting(&awaited_context, awaited_repo.as_path(), THE_BATCH))
        .expect("the runtime");
    assert_eq!(synchronous, ScratchRefRemoval::Removed, "a batch that did not land has its scratch reference removed once its commit is kept");
    assert_eq!(awaited, synchronous, "both faces answer one thing about the same batch");
    assert_eq!(synchronous_kept, RefValue::Names(head), "the commit of the synchronous face's batch is kept where the person is told to look");
    assert_eq!(
        kept_after(&awaited_context, awaited_repo.as_path()),
        RefValue::Names(awaited_head),
        "the awaited face keeps the commit of its own batch, which is the one the person is told to look at there"
    );
    remove(root.as_path());
    remove(awaited_root.as_path());
}

/// What:
///  A kept reference that already names another commit leaves the scratch reference where it is,
/// through both faces.
/// Why:
///  Replacing the kept reference would lose the commit another batch was kept under,
/// which is the one thing the removal must
///  never do;
/// the case is the same on both faces or the guarantee holds on one path only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(repo, ['update-ref', keptRef(1), otherCommit]); assert.equal(clear(...), 'left') for both faces
/// ```
#[test]
fn a_kept_reference_another_batch_holds_leaves_the_scratch_reference_through_both_faces() {
    let (root, repo, _journal, context, _head) = a_repository_with_a_dropped_batch("scratch-ref-kept-elsewhere");
    let elsewhere: String = git_text(repo.as_path(), &["commit-tree", "-m", "another batch", "HEAD^{tree}"]);
    git(repo.as_path(), &["update-ref", kept_ref(THE_BATCH).as_str(), elsewhere.as_str()]);
    let synchronous: ScratchRefRemoval = clear_after_dropped_batch(&context, repo.as_path(), THE_BATCH);
    let (awaited_root, awaited_repo, _awaited_journal, awaited_context, _awaited_head) = a_repository_with_a_dropped_batch("scratch-ref-kept-elsewhere-awaited");
    let awaited_elsewhere: String = git_text(awaited_repo.as_path(), &["commit-tree", "-m", "another batch", "HEAD^{tree}"]);
    git(awaited_repo.as_path(), &["update-ref", kept_ref(THE_BATCH).as_str(), awaited_elsewhere.as_str()]);
    let awaited: ScratchRefRemoval =
        crate::lazy_runtime::run_on_runtime(clear_after_dropped_batch_awaiting(&awaited_context, awaited_repo.as_path(), THE_BATCH)).expect("the runtime");
    assert_eq!(synchronous, ScratchRefRemoval::Left, "a kept reference another batch holds leaves the scratch reference where it is");
    assert_eq!(awaited, synchronous, "both faces leave the scratch reference where it is for the same reason");
    assert_eq!(kept_after(&awaited_context, awaited_repo.as_path()), RefValue::Names(awaited_elsewhere), "the kept reference of the other batch is not replaced");
    remove(root.as_path());
    remove(awaited_root.as_path());
}

/// What:
///  A look at a batch's intent that gives no answer leaves the scratch reference where it is,
/// and marks nothing done.
/// Why:
///  The look is a `stat` of a file in the instance's journal,
/// which waits in the kernel on a mount that stopped answering,
/// and a task may not
///  wait for it.
/// A look nobody answered must set no removal in motion:
/// "no intent" hands the batch to the removal,
/// and
///  that is what would lose the commit of a batch a restart still holds.
/// The answer a control states is the one such a look gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using _stated = stateTheAnswer('noAnswer'); assert.equal(await clearAfterAFailedLandingAwaiting(...), 'left')
/// ```
#[test]
fn a_look_at_an_intent_that_gives_no_answer_sets_no_removal_in_motion() {
    let (root, repo, journal, context, head) = a_repository_with_a_dropped_batch("scratch-ref-intent-no-answer");
    let landing: LandContext = the_landing_context(&context, journal.as_path());
    let requests: Vec<CommitRequest> = vec![a_request_for(repo.as_path())];
    let _stated = state_the_answer(AStatesAnswer::NoAnswer);
    let removal: ScratchRefRemoval = crate::lazy_runtime::run_on_runtime(clear_after_a_failed_landing_awaiting(&landing, requests.as_slice(), THE_BATCH))
        .expect("the runtime");
    assert_eq!(removal, ScratchRefRemoval::Left, "a look at the intent that gave no answer leaves the batch's scratch reference where it is");
    assert_eq!(value_of(&context, repo.as_path(), scratch_ref(THE_BATCH).as_str()), RefValue::Names(head), "the commit of the batch is still named by its scratch reference");
    assert_eq!(kept_after(&context, repo.as_path()), RefValue::Absent, "nothing was kept, because nothing was read");
    remove(root.as_path());
}

/// What:
///  A failed landing's batch is cleared inside a task,
/// where the synchronous face cannot clear it at all.
/// Why:
///  The synchronous face starts its Git by entering the runtime,
/// which a task may not do,
/// so a landing that becomes a task
///  needs this face;
/// the scratch reference goes and the commit is kept,
/// which is what pins that the Git steps ran and were awaited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const awaited = await spawn(() => clearAfterAFailedLandingAwaiting(...)); const refused = await spawn(() => clearAfterAFailedLanding(...)); assert(awaited === 'removed' && refused === 'left')
/// ```
#[test]
fn a_failed_landings_batch_is_cleared_inside_a_task() {
    let (root, repo, journal, context, head) = a_repository_with_a_dropped_batch("scratch-ref-in-a-task");
    let requests: Vec<CommitRequest> = vec![a_request_for(repo.as_path())];
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let awaited_landing: LandContext = the_landing_context(&context, journal.as_path());
    let awaited_requests: Vec<CommitRequest> = requests.clone();
    let awaited: ScratchRefRemoval = runtime.block_on(async move {
        return clear_after_a_failed_landing_awaiting(&awaited_landing, awaited_requests.as_slice(), THE_BATCH).await;
    });
    assert_eq!(awaited, ScratchRefRemoval::Removed, "the awaited face clears the batch of a failed landing inside a task");
    assert_eq!(kept_after(&context, repo.as_path()), RefValue::Names(head.clone()), "the commit of the batch is kept at the reference the person is told to look at");
    // The same work, through the synchronous face, on the fixture as it now stands: the batch has no scratch reference left.
    git(repo.as_path(), &["update-ref", scratch_ref(THE_BATCH).as_str(), head.as_str()]);
    let refused_landing: LandContext = the_landing_context(&context, journal.as_path());
    let refused_requests: Vec<CommitRequest> = requests.clone();
    let refused: ScratchRefRemoval = runtime.block_on(async move {
        let asking = tokio::spawn(async move {
            return clear_after_a_failed_landing(&refused_landing, refused_requests.as_slice(), THE_BATCH);
        });
        return asking.await.expect("the task that asks ends");
    });
    assert_eq!(refused, ScratchRefRemoval::Left, "the synchronous face cannot start its Git from inside the runtime, so it leaves the reference where it is");
    remove(root.as_path());
}
