//! Controls for the service's after-commit work consumer, with handlers that record what they are asked.

use super::*;
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, PENDING, enqueue, state_directory};
use std::cell::RefCell;

const OID: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER_OID: &str = "89abcdef0123456789abcdef0123456789abcdef";
const BRANCH: &str = "refs/heads/main";

/// Handlers that record the calls and answer from a fixed verdict.
struct Recorder {
    verdict: Verdict,
    pushes: RefCell<Vec<Vec<String>>>,
    upkeeps: RefCell<Vec<String>>,
    reviews: RefCell<Vec<String>>,
}

impl Recorder {
    fn answering(verdict: Verdict) -> Recorder {
        return Recorder { verdict, pushes: RefCell::new(Vec::new()), upkeeps: RefCell::new(Vec::new()), reviews: RefCell::new(Vec::new()) };
    }
}

impl WorkHandlers for Recorder {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        self.pushes.borrow_mut().push(items.iter().map(|item| { return item.oid.clone(); }).collect());
        return self.verdict.clone();
    }

    fn maintenance(&self, item: &WorkItem) -> Verdict {
        self.upkeeps.borrow_mut().push(item.repository.clone());
        return self.verdict.clone();
    }

    fn review(&self, item: &WorkItem) -> Verdict {
        self.reviews.borrow_mut().push(item.oid.clone());
        return self.verdict.clone();
    }
}

/// A repository directory standing in for a common Git directory, with a worktree directory beside it.
struct Repo {
    root: PathBuf,
    common: PathBuf,
    worktree: PathBuf,
}

fn repo(name: &str) -> Repo {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir_all(&common).expect("common directory");
    std::fs::create_dir_all(&worktree).expect("worktree directory");
    return Repo { root, common, worktree };
}

fn item(repo: &Repo, kind: WorkKind, oid: &str, branch: &str) -> WorkItem {
    return WorkItem {
        kind,
        repository: repo.worktree.display().to_string(),
        git_dir: repo.common.display().to_string(),
        branch_ref: branch.to_string(),
        oid: oid.to_string(),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
}

fn enqueue_all(repo: &Repo, items: &[WorkItem]) {
    for entry in items {
        enqueue(work_directory(repo.common.as_path()).as_path(), entry).expect("enqueue");
    }
}

fn names(repo: &Repo, state: &str) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(state_directory(work_directory(repo.common.as_path()).as_path(), state))
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    found.sort();
    return found;
}

/// Several pending pushes of one branch are one push of the newest commit, and every item is settled as done.
#[test]
fn pending_pushes_of_one_branch_become_one_push() {
    let repo: Repo = repo("work-coalesce");
    let items: Vec<WorkItem> = vec![item(&repo, WorkKind::Push, OID, BRANCH), item(&repo, WorkKind::Push, OTHER_OID, BRANCH)];
    enqueue_all(&repo, &items);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, SystemTime::now());
    let pushes = recorder.pushes.borrow();
    assert_eq!(pushes.len(), 1, "one push for the branch");
    assert_eq!(pushes[0].len(), items.len(), "the push covers every pending item");
    assert_eq!(names(&repo, DONE).len(), items.len());
    assert!(names(&repo, PENDING).is_empty() && names(&repo, CLAIMED).is_empty());
    remove(repo.root.as_path());
}

/// Pushes of different branches stay separate, and upkeep runs once per repository however many items it has.
#[test]
fn different_branches_push_separately_and_upkeep_runs_once() {
    let repo: Repo = repo("work-branches");
    let other_branch: &str = "refs/heads/other";
    let items: Vec<WorkItem> = vec![
        item(&repo, WorkKind::Push, OID, BRANCH),
        item(&repo, WorkKind::Push, OTHER_OID, other_branch),
        item(&repo, WorkKind::Maintenance, OID, BRANCH),
        item(&repo, WorkKind::Maintenance, OTHER_OID, other_branch),
    ];
    enqueue_all(&repo, &items);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, SystemTime::now());
    assert_eq!(recorder.pushes.borrow().len(), 2);
    assert_eq!(recorder.upkeeps.borrow().len(), 1);
    assert_eq!(names(&repo, DONE).len(), items.len());
    remove(repo.root.as_path());
}

/// A failed push is settled as failed and leaves one notice that names the commit and what Git said, shown once.
#[test]
fn a_failed_push_leaves_a_notice_that_is_shown_once() {
    let repo: Repo = repo("work-notice");
    let said: &str = "remote: permission denied";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(said.to_string())), SystemTime::now());
    assert_eq!(names(&repo, FAILED).len(), 1);
    let shown: Vec<String> = take_notices(repo.common.as_path());
    assert_eq!(shown, vec![push_failed_notice(OID, said)]);
    assert!(take_notices(repo.common.as_path()).is_empty(), "a notice is shown once");
    remove(repo.root.as_path());
}

/// Each review item is its own review (never coalesced), settled as done, and it runs after the pushes of the same round.
#[test]
fn every_review_item_is_reviewed_once_after_the_pushes() {
    let repo: Repo = repo("work-review");
    let items: Vec<WorkItem> = vec![
        item(&repo, WorkKind::Review, OID, BRANCH),
        item(&repo, WorkKind::Review, OTHER_OID, BRANCH),
        item(&repo, WorkKind::Push, OID, BRANCH),
    ];
    enqueue_all(&repo, &items);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, SystemTime::now());
    let mut reviewed: Vec<String> = recorder.reviews.borrow().clone();
    reviewed.sort();
    let mut expected: Vec<String> = vec![OID.to_string(), OTHER_OID.to_string()];
    expected.sort();
    assert_eq!(reviewed, expected);
    assert_eq!(recorder.pushes.borrow().len(), 1);
    assert_eq!(names(&repo, DONE).len(), items.len());
    assert!(names(&repo, PENDING).is_empty() && names(&repo, CLAIMED).is_empty());
    remove(repo.root.as_path());
}

/// A review that fails is settled as failed (a plain record), leaves no notice for the person, and is not run again.
#[test]
fn a_failed_review_is_a_plain_record_not_a_stuck_item() {
    let repo: Repo = repo("work-review-failed");
    let review: WorkItem = item(&repo, WorkKind::Review, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&review));
    let recorder: Recorder = Recorder::answering(Verdict::Failed("the log could not be written".to_string()));
    drain(repo.common.as_path(), &recorder, SystemTime::now());
    assert_eq!(names(&repo, FAILED), vec![review.id()]);
    assert!(names(&repo, PENDING).is_empty() && names(&repo, CLAIMED).is_empty());
    assert!(take_notices(repo.common.as_path()).is_empty(), "a shadow review never nags the person");
    drain(repo.common.as_path(), &recorder, SystemTime::now());
    assert_eq!(recorder.reviews.borrow().len(), 1, "a failed review is not retried");
    remove(repo.root.as_path());
}

/// An item whose fields do not fit this repository is failed without running anything.
#[test]
fn an_item_for_another_repository_or_with_a_bad_reference_is_refused() {
    let repo: Repo = repo("work-refused");
    let elsewhere: WorkItem = WorkItem { git_dir: repo.root.join("elsewhere").display().to_string(), ..item(&repo, WorkKind::Push, OID, BRANCH) };
    let bad_reference: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, "refs/heads/a b");
    for refused in [&elsewhere, &bad_reference] {
        assert!(!is_acceptable(refused, repo.common.as_path()), "{refused:?}");
    }
    assert!(is_acceptable(&item(&repo, WorkKind::Push, OID, BRANCH), repo.common.as_path()));
    enqueue_all(&repo, &[elsewhere, bad_reference]);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, SystemTime::now());
    assert!(recorder.pushes.borrow().is_empty());
    assert_eq!(names(&repo, FAILED).len(), 2);
    remove(repo.root.as_path());
}

/// A claim whose consumer died is taken back after the lease and run again, then given up after the tries.
#[test]
fn an_expired_claim_is_run_again_and_given_up_after_the_tries() {
    let repo: Repo = repo("work-lease");
    let stuck: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&stuck));
    let work_dir: PathBuf = work_directory(repo.common.as_path());
    let started: SystemTime = SystemTime::now();
    // The first consumer claims it and dies.
    assert!(claim_next_of(work_dir.as_path(), started, &SERVICE_KINDS).expect("claim").is_some());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, started + LEASE / 2);
    assert!(recorder.pushes.borrow().is_empty(), "a live claim is not taken");
    drain(repo.common.as_path(), &recorder, started + LEASE * 2);
    assert_eq!(recorder.pushes.borrow().len(), 1, "an expired claim runs again");
    assert_eq!(names(&repo, DONE), vec![stuck.id()]);
    remove(repo.root.as_path());
}

/// The credential a wake brought is held for that repository only, and is gone once its round ends.
#[test]
fn the_review_key_is_held_per_repository_and_forgotten() {
    let handlers: GitHandlers = GitHandlers::new(crate::test_support::git_context());
    let first: PathBuf = PathBuf::from("/srv/one/.git");
    let second: PathBuf = PathBuf::from("/srv/two/.git");
    let key: ReviewKey = ReviewKey::new("test-key-not-a-secret").expect("a key");
    assert_eq!(handlers.review_key_of(first.as_path()), None);
    handlers.remember_review_key(first.as_path(), key.clone());
    assert_eq!(handlers.review_key_of(first.as_path()), Some(key));
    assert_eq!(handlers.review_key_of(second.as_path()), None, "another repository does not see it");
    handlers.forget_review_key(first.as_path());
    assert_eq!(handlers.review_key_of(first.as_path()), None);
}
