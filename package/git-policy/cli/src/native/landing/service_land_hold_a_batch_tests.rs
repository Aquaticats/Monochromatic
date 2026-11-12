//! What:
//!  Controls for the endings of a landing that never moves its branch:
//!  a batch answered as a failure keeps the commit it built,
//!  and a batch whose commit is rebuilt lets go of the reference that held it only while that reference still names it.
//! Why:
//!  A commit no branch names and no reference names is one `git gc --prune=now` takes,
//!  and a commit the person is told was not made must still be findable
//!  (the owner's rule "never discard user data").
//!  The removal is Git's compare-and-delete, so a reference another command wrote between the read and the removal is
//!  never removed.

use super::{DropTheBatch, end_a_batch_the_write_never_started};
use crate::service_index_lock::Announce;
use crate::service_journal::{Intent, OPEN_EXTENSION, batch_path, write_intent};
use crate::service_land_context::LandContext;
use crate::service_protocol::FailureCode;
use crate::service_scratch_ref_of_a_dropped_batch::{kept_ref, scratch_ref};
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};
use std::path::PathBuf;

/// The batch these controls give back.
const THE_BATCH: u64 = 1;

/// What one ending came to.
struct Ended {
    /// The folder the fixture lives in,
    /// removed once its control read everything.
    root: PathBuf,
    /// The repository the landing ran in.
    repo: PathBuf,
    /// What the ending answered.
    answered: FailureCode,
    /// The lines the person was told.
    said: Vec<String>,
}

/// What:
///  One landing that built a commit, left it on the batch's own reference, wrote its intent, and then ends at `answered`.
/// Why:
///   That is the state every ending that does not move the branch reads:
///  the commit exists,
///  the reference names it,
///  and the intent would let a restart land it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = endTheBatch({ answer: 'unavailable', takeTheKeptName: false })
/// ```
fn end_the_batch(name: &str, answered: FailureCode, take_the_kept_name: bool) -> Ended {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let built: String = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-p", base.as_str(), "-m", "the commit the batch built"]);
    let scratch: String = scratch_ref(THE_BATCH);
    git(repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", scratch.as_str(), built.as_str()]);
    if take_the_kept_name {
        git(repo.as_path(), &["update-ref", kept_ref(THE_BATCH).as_str(), base.as_str()]);
    }
    let intent: Intent = Intent {
        seq: THE_BATCH,
        repository: repo.clone(),
        common_dir: repo.join(".git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: base,
        new: built,
        ids: vec![String::from("request-1")],
        commits: Vec::new(),
        holders: Vec::new(),
        paths: Vec::new(),
        derived: false,
        cleanup: None,
        index: repo.join(".git/index"),
        git_dir: repo.join(".git"),
    };
    write_intent(journal.as_path(), &intent).expect("the intent");
    let context: LandContext = LandContext { git: git_context(), journal: journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING };
    let said: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    let announce = |note: Announce| {
        if let crate::service_index_lock::Announce::Waiting(text) = note {
            said.lock().expect("lock").push(text);
        }
    };
    let drop: DropTheBatch<'_> = DropTheBatch { git: &context.git, context: &context, repository: repo.as_path(), seq: THE_BATCH, intent: &intent, scratch: scratch.as_str(), new: intent.new.as_str(), announce: &announce };
    end_a_batch_the_write_never_started(&drop, answered);
    let lines: Vec<String> = said.lock().expect("lock").clone();
    return Ended { root, repo, answered: FailureCode::Unavailable, said: lines };
}

/// What:
///  A batch answered as a failure keeps the commit it built on the reference a commit that did not land is kept on,
///  lets go of the reference that held it once that name is written,
///  and the commit survives a maintenance that drops every object no reference names.
/// Why:
///   The person is told the commit was not made and where it is kept;
///  a commit named by nothing is one `git gc --prune=now` takes,
///  and what a person made is never discarded
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = endTheBatch({ answer: 'unavailable' }); assert.equal(revParse(keptRef(1)), revParse('built')); prune(); assert.equal(catFile(keptRef(1)), 'commit')
/// ```
#[test]
fn a_batch_answered_as_a_failure_keeps_the_commit_it_built() {
    let ended: Ended = end_the_batch("hold-a-batch-failure-keeps", FailureCode::Unavailable, false);
    let kept: String = git_text(ended.repo.as_path(), &["rev-parse", kept_ref(THE_BATCH).as_str()]);
    assert_eq!(git_text(ended.repo.as_path(), &["log", "-1", "--format=%s", kept_ref(THE_BATCH).as_str()]), "the commit the batch built", "the kept reference holds the commit Git built");
    assert_eq!(git_text(ended.repo.as_path(), &["for-each-ref", scratch_ref(THE_BATCH).as_str()]), "", "the reference the batch held the commit on goes once the kept reference names it");
    assert!(!batch_path(ended.root.join("journal").as_path(), THE_BATCH, OPEN_EXTENSION).exists(), "no intent outlives the batch");
    let line: String = ended.said.join("");
    assert!(line.contains(kept_ref(THE_BATCH).as_str()), "the person is told where the commit is kept: {line}");
    assert!(!kept.is_empty(), "the kept reference names a commit");
    git(ended.repo.as_path(), &["prune", "--expire=now"]);
    assert_eq!(git_text(ended.repo.as_path(), &["cat-file", "-t", kept_ref(THE_BATCH).as_str()]), "commit", "the commit survives a prune");
    let _ = ended.answered;
    remove(ended.root.as_path());
}

/// What:
///  A batch answered as a failure whose keep cannot be written leaves the commit on the reference that held it,
///  and tells the person the commit's name and that reference.
/// Why:
///   The reference is the only name of the built commit then,
///  and removing it while no other name exists is what loses the commit to `git gc --prune=now`
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// take('refs/cli-git/kept/1', other); const ended = endTheBatch({ answer: 'unavailable' }); assert.equal(revParse(scratchRef(1)), built); prune(); assert.equal(catFile(scratchRef(1)), 'commit')
/// ```
#[test]
fn a_batch_answered_as_a_failure_whose_keep_is_refused_leaves_the_commit_named() {
    let ended: Ended = end_the_batch("hold-a-batch-failure-keep-refused", FailureCode::Unavailable, true);
    let held: String = git_text(ended.repo.as_path(), &["rev-parse", scratch_ref(THE_BATCH).as_str()]);
    assert!(!held.is_empty(), "the reference that held the commit stays, because no other reference names it");
    assert_eq!(git_text(ended.repo.as_path(), &["log", "-1", "--format=%s", scratch_ref(THE_BATCH).as_str()]), "the commit the batch built", "the reference that stays holds the commit Git built");
    let line: String = ended.said.join("");
    assert!(line.contains(held.as_str()) && line.contains(scratch_ref(THE_BATCH).as_str()), "the person is told the commit's name and where it is kept: {line}");
    git(ended.repo.as_path(), &["prune", "--expire=now"]);
    assert_eq!(git_text(ended.repo.as_path(), &["cat-file", "-t", held.as_str()]), "commit", "the commit survives a prune");
    remove(ended.root.as_path());
}

/// What:
///  A batch whose commit is rebuilt removes the reference that held it only while that reference still names the commit the batch built,
///  so a commit another command wrote there is not left named by nothing.
/// Why:
///   A command that reads a reference and then removes it by name alone removes whatever another command wrote in between
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// take(scratchRef(1), other); dropTheBatch(); assert.equal(revParse(scratchRef(1)), other)
/// ```
#[test]
fn a_dropped_batch_does_not_remove_a_scratch_reference_another_command_wrote() {
    let root: PathBuf = fixture("hold-a-batch-dropped-other-writer");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let built: String = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-p", base.as_str(), "-m", "the commit the batch built"]);
    let other: String = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-p", base.as_str(), "-m", "another command's commit"]);
    let scratch: String = scratch_ref(THE_BATCH);
    // Another command wrote its own commit to the name this batch's reference had.
    git(repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", scratch.as_str(), other.as_str()]);
    let intent: Intent = Intent {
        seq: THE_BATCH,
        repository: repo.clone(),
        common_dir: repo.join(".git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: base,
        new: built,
        ids: vec![String::from("request-1")],
        commits: Vec::new(),
        holders: Vec::new(),
        paths: Vec::new(),
        derived: false,
        cleanup: None,
        index: repo.join(".git/index"),
        git_dir: repo.join(".git"),
    };
    write_intent(journal.as_path(), &intent).expect("the intent");
    let context: LandContext = LandContext { git: git_context(), journal, on_objects_written: None, lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING };
    let drop: DropTheBatch<'_> = DropTheBatch { git: &context.git, context: &context, repository: repo.as_path(), seq: THE_BATCH, intent: &intent, scratch: scratch.as_str(), new: intent.new.as_str(), announce: &|_note: Announce| {} };
    super::drop_the_batch(&drop);
    let still: String = git_text(repo.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()]);
    assert_eq!(still, other, "the reference another command wrote is not removed");
    git(repo.as_path(), &["prune", "--expire=now"]);
    assert_eq!(git_output(repo.as_path(), &["cat-file", "-e", other.as_str()]).status.success(), true, "the other command's commit is still in the object store after a prune");
    remove(root.as_path());
}
