//! What:
//!  Disposable-repository controls for committed versions,
//!  invalidation and listing failures.
//! Why:
//!  A landed commit is judged by what it changed,
//!  a fix or replay must retire earlier
//!      listings,
//!  and a listing that cannot be trusted must fail instead of shrinking.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // store.invalidate(); await expect(store.bytes(staleCandidate)).rejects.toThrow();
//! ```
#![cfg(unix)]

/// Import the store under test,
///  its sibling fixtures and shared fixtures.
use super::CandidateStore;
use super::tests::{at, paths, rev_parse, staged_changes, store, write};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::{CandidateMode, ObjectId};
use crate::candidate_record::CandidateChange;
use crate::candidate_version::{Candidate, CandidateSource, CandidateVersion};
use crate::test_support::{fixture, git, remove, repository};
use std::path::PathBuf;
use std::rc::Rc;

/// A committed version lists what the commit changed:
///  against its parent,
///  against nothing for a root commit.
#[test]
fn committed_version_lists_the_commit_delta() {
    let root: PathBuf = fixture("store-committed");
    let repo: PathBuf = staged_changes(root.as_path());
    git(repo.as_path(), &["commit", "--quiet", "--message=changes"]);
    // The worktree and index move on; the committed version must not follow them.
    write(repo.as_path(), b"modify.txt", b"later worktree edit\n");
    git(repo.as_path(), &["add", "modify.txt"]);
    let landed: ObjectId = rev_parse(repo.as_path(), "HEAD");
    let base: ObjectId = rev_parse(repo.as_path(), "HEAD~1");
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::Committed(landed))
        .expect("committed version");
    assert_eq!(version.candidates().len(), 10);
    assert_eq!(
        at(&version, b"modify.txt").change,
        CandidateChange::Modified
    );
    assert_eq!(
        &subject.bytes(at(&version, b"modify.txt")).expect("bytes")[..],
        b"after\n"
    );
    assert_eq!(at(&version, b"delete.txt").change, CandidateChange::Deleted);
    assert_eq!(at(&version, b"sub").mode, CandidateMode::Gitlink);
    // The commit before it added six files to the fixture's empty first commit.
    let base_version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::Committed(base))
        .expect("base version");
    assert_eq!(base_version.candidates().len(), 6);
    assert_eq!(
        &subject
            .bytes(at(&base_version, b"modify.txt"))
            .expect("bytes")[..],
        b"before\n"
    );
    remove(root.as_path());
}

/// A root commit lists all of its entries,
///  and a merge lists each parent's comparison with the first record winning.
#[test]
fn root_and_merge_commits_are_listed() {
    let root: PathBuf = fixture("store-merge");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(
        repo.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    write(repo.as_path(), b"base.txt", b"base\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=root"]);
    let first: ObjectId = rev_parse(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "-b", "side"]);
    write(repo.as_path(), b"side.txt", b"side\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=side"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    write(repo.as_path(), b"main.txt", b"main\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=main"]);
    git(repo.as_path(), &["merge", "--quiet", "--no-edit", "side"]);
    let merge: ObjectId = rev_parse(repo.as_path(), "HEAD");
    let mut subject: CandidateStore = store(repo.as_path());
    let root_version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::Committed(first))
        .expect("root version");
    assert_eq!(paths(&root_version), [b"base.txt".to_vec()]);
    assert_eq!(
        at(&root_version, b"base.txt").change,
        CandidateChange::Added
    );
    // Against `main` the merge added side.txt; against `side` it added main.txt.
    let merge_version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::Committed(merge))
        .expect("merge version");
    assert_eq!(
        paths(&merge_version),
        [b"main.txt".to_vec(), b"side.txt".to_vec()]
    );
    assert_eq!(
        &subject
            .bytes(at(&merge_version, b"side.txt"))
            .expect("bytes")[..],
        b"side\n"
    );
    remove(root.as_path());
}

/// Within a generation a version is listed once;
///  after `invalidate` it is listed again and earlier candidates are refused.
#[test]
fn invalidation_retires_versions_and_their_candidates() {
    let root: PathBuf = fixture("store-invalidate");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), b"fixed.txt", b"before the fix\n");
    write(repo.as_path(), b"same.txt", b"untouched\n");
    git(repo.as_path(), &["add", "--all"]);
    let mut subject: CandidateStore = store(repo.as_path());
    let before: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("first version");
    let stale: Candidate = at(&before, b"fixed.txt").clone();
    assert_eq!(stale.identity.generation, 0);
    assert_eq!(
        &subject.bytes(&stale).expect("bytes")[..],
        b"before the fix\n"
    );
    // A fix stages new content for one path.
    write(repo.as_path(), b"fixed.txt", b"after the fix\n");
    git(repo.as_path(), &["add", "fixed.txt"]);
    // Without invalidation the version is the same shared value, still describing the earlier listing.
    let reused: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("reused version");
    assert!(Rc::ptr_eq(&before, &reused));
    assert_eq!(
        &subject.bytes(at(&reused, b"fixed.txt")).expect("bytes")[..],
        b"before the fix\n"
    );
    subject.invalidate();
    let error: CandidateError = subject.bytes(&stale).expect_err("stale candidate");
    assert_eq!(error.failure, CandidateFailure::StaleCandidate);
    assert!(
        error
            .message
            .starts_with("cli-git refused to read a candidate"),
        "{error}"
    );
    let after: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("second version");
    assert!(!Rc::ptr_eq(&before, &after));
    let fresh: &Candidate = at(&after, b"fixed.txt");
    assert_eq!(fresh.identity.generation, 1);
    assert_ne!(fresh.object, stale.object);
    assert_eq!(
        &subject.bytes(fresh).expect("bytes")[..],
        b"after the fix\n"
    );
    // The untouched path keeps its object, and its candidate of the new version is readable.
    assert_eq!(
        at(&after, b"same.txt").object,
        at(&before, b"same.txt").object
    );
    assert_eq!(
        &subject.bytes(at(&after, b"same.txt")).expect("bytes")[..],
        b"untouched\n"
    );
    // A second invalidation advances the generation again.
    subject.invalidate();
    let third: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("third version");
    assert_eq!(at(&third, b"fixed.txt").identity.generation, 2);
    assert_eq!(
        subject.bytes(fresh).expect_err("stale").failure,
        CandidateFailure::StaleCandidate
    );
    remove(root.as_path());
}

/// Different sources are kept apart:
///  each is listed for itself and returned again on request.
#[test]
fn versions_are_kept_per_source() {
    let root: PathBuf = fixture("store-sources");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), b"one.txt", b"one\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    let landed: ObjectId = rev_parse(repo.as_path(), "HEAD");
    let base: ObjectId = rev_parse(repo.as_path(), "HEAD~1");
    write(repo.as_path(), b"two.txt", b"two\n");
    git(repo.as_path(), &["add", "--all"]);
    let mut subject: CandidateStore = store(repo.as_path());
    let staged: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged");
    let against_base: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstCommit(base))
        .expect("staged against base");
    let committed: Rc<CandidateVersion> = subject
        .version(&CandidateSource::Committed(landed.clone()))
        .expect("committed");
    assert_eq!(paths(&staged), [b"two.txt".to_vec()]);
    assert_eq!(
        paths(&against_base),
        [b"one.txt".to_vec(), b"two.txt".to_vec()]
    );
    assert_eq!(paths(&committed), [b"one.txt".to_vec()]);
    assert!(Rc::ptr_eq(
        &committed,
        &subject
            .version(&CandidateSource::Committed(landed))
            .expect("committed again")
    ));
    assert!(Rc::ptr_eq(
        &staged,
        &subject
            .version(&CandidateSource::StagedAgainstHead)
            .expect("staged again")
    ));
    remove(root.as_path());
}
