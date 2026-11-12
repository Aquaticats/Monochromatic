//! Controls for the objects a batch lands:
//! the side reference that keeps a commit Git built reachable,
//! and who writes it.
//! Why:
//!  The reference is a name the wrapper owns,
//! so the landing can write it itself instead of starting a Git that does one rename;
//!  a control has to hold both that the reference appears without a Git start and that every shape the landing cannot
//!  prove goes to Git with its own words.

use super::*;
use crate::service_scratch_ref_of_a_dropped_batch::scratch_ref;
use crate::service_land_settings::Settings;
use crate::side_reference_written_here::{LockMark, SideReferenceWritePlan};
use crate::test_support::{fixture, git, git_text, repository};
use std::path::PathBuf;

/// The batch every control here lands.
const THE_BATCH: u64 = 7;

/// What:
///  A Git context whose Git cannot be started,
///  so a control proves which parts of a landing still reach a Git process.
/// Why:
///   The point of writing the side reference here is that the commit stops paying a Git start for it,
///  and a context that cannot start one is the sharpest reading of whether it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const git = { ...gitContext(), gitPath: '/nonexistent/git-the-landing-never-starts' };
/// ```
fn a_git_that_cannot_be_started() -> GitContext {
    return GitContext { real_git: PathBuf::from("/nonexistent/git-the-landing-never-starts"), ..crate::test_support::git_context() };
}

/// What:
///  The settings one control lands with:
///  the history Git shows,
///  no import flush,
///  and what to do about the batch's side reference.
/// Why:
///  The three are what the landing reads once,
///  and a control pins the third to the path it is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settings = { importFsync: undefined, plainHistory: true, sideReference: plan };
/// ```
fn the_settings(side_reference: SideReferenceWritePlan) -> Settings<'static> {
    return Settings { import_fsync: None, plain_history: true, side_reference };
}

/// What:
///  A repository whose tip is `base`,
///  and the commit Git built on top of it,
/// with the request that lands it.
/// Why:
///   A commit Git built is the one a landing keeps on the batch's side reference,
///  so a control needs exactly that shape:
///  the tip as the single parent,
///  the objects present,
///  and nothing else in the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const base = revParse('HEAD'); commit('a change'); return { base, built: revParse('HEAD'), request };
/// ```
fn a_repository_with_a_built_commit(name: &str) -> (PathBuf, String, String, CommitRequest) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("a-change.txt"), "one\n").expect("a change in the fixture");
    git(repo.as_path(), &["add", "--", "a-change.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=the commit Git built"]);
    let built: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let common: PathBuf = repo.join(".git");
    let request: CommitRequest = CommitRequest {
        id: String::from("a-control-request"),
        repository: repo.clone(),
        common_dir: common.clone(),
        git_dir: common.clone(),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: base.clone(),
        changes: Vec::new(),
        message: String::new(),
        author: None,
        committer: String::new(),
        amend: false,
        tree: None,
        commit: Some(built.clone()),
        cleanup: None,
        index: common.join("index"),
        scratch: common.join("monochromatic-git-policy-cli-service-a-control"),
        captured: 0,
        holder: String::new(),
    };
    return (root, base, built, request);
}

/// What:
///  A landing that writes its side reference itself lands a commit Git built without starting Git at all,
///  and the batch's reference names that commit.
/// Why:
///  This is the wait the change takes off the person's commit:
///  the reference is what the branch write needs,
///  and a Git start for one rename is what the commit used to pay for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await objectsOfBatch(gitThatCannotStart, ...); assert.equal(await revParse(scratchRef(7)), built)
/// ```
#[test]
fn a_batch_side_reference_is_written_without_a_git_start() {
    let (root, base, built, request) = a_repository_with_a_built_commit("side-reference-written-here");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("a journal directory");
    let scratch: String = scratch_ref(THE_BATCH);
    let settings: Settings<'_> = the_settings(SideReferenceWritePlan::Here(LockMark::None));
    let outcome: Result<(Vec<String>, String), FailureCode> = objects_of_batch(
        &a_git_that_cannot_be_started(),
        journal.as_path(),
        THE_BATCH,
        &request,
        scratch.as_str(),
        base.as_str(),
        &[&request],
        &[],
        &settings,
    );
    let (commits, new): (Vec<String>, String) = match outcome {
        Ok(landed) => landed,
        Err(code) => panic!("the objects of a batch Git built are written without a Git start, and this landing answered {code:?}"),
    };
    assert_eq!(commits, vec![built.clone()], "the batch lands the commit Git built");
    assert_eq!(new, built, "the tip the batch answers with");
    let written: String = git_text(request.repository.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()]);
    assert_eq!(written, built, "the batch's side reference names the commit Git built");
}

/// What:
///  A landing that leaves the side reference to Git still lands a commit Git built,
///  and the reference names it.
/// Why:
///   The reference is written by Git where the repository's own settings must be Git's to keep
///  (a reference store that is flushed,
/// or one that is not plain files),
///  so that path must keep working exactly as it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await objectsOfBatch(realGit, ..., { sideReferenceWrittenHere: false }); assert.equal(await revParse(scratchRef(7)), built)
/// ```
#[test]
fn a_landing_that_leaves_the_reference_to_git_lands_the_same_batch() {
    let (root, base, built, request) = a_repository_with_a_built_commit("side-reference-left-to-git");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("a journal directory");
    let scratch: String = scratch_ref(THE_BATCH);
    let context: GitContext = GitContext { common_dir: Some(request.common_dir.clone()), ..crate::test_support::git_context() };
    let settings: Settings<'_> = the_settings(SideReferenceWritePlan::LeftToGit);
    let outcome: Result<(Vec<String>, String), FailureCode> = objects_of_batch(
        &context,
        journal.as_path(),
        THE_BATCH,
        &request,
        scratch.as_str(),
        base.as_str(),
        &[&request],
        &[],
        &settings,
    );
    assert!(outcome.is_ok(), "a landing whose side reference Git writes lands the batch: {outcome:?}");
    let written: String = git_text(request.repository.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()]);
    assert_eq!(written, built, "Git's own write of the batch's side reference names the commit");
}
