//! Controls for the service's after-commit work consumer, with handlers that record what they are asked.

use super::*;
use crate::push_failure_notice::{push_failure_text, take_push_failures};
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

/// A failed push is settled as failed and leaves one record that names the branch, the commit and what Git said, shown once.
#[test]
fn a_failed_push_leaves_a_notice_that_is_shown_once() {
    let repo: Repo = repo("work-notice");
    let said: &str = "remote: permission denied";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(said.to_string())), SystemTime::now());
    assert_eq!(names(&repo, FAILED).len(), 1);
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown, vec![push_failure_text(BRANCH, OID, said)]);
    assert!(take_push_failures(repo.common.as_path()).is_empty(), "a notice is shown once");
    remove(repo.root.as_path());
}

/// A later push of the branch that goes through removes the branch's records that nobody has seen yet; another branch's
///  record stays.
#[test]
fn a_later_successful_push_of_the_branch_clears_its_unshown_failure() {
    let repo: Repo = repo("work-notice-cleared");
    let other_branch: &str = "refs/heads/other";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH), item(&repo, WorkKind::Push, OTHER_OID, other_branch)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(String::from("remote: rejected"))), SystemTime::now());
    let third: &str = "fedcba9876543210fedcba9876543210fedcba98";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, third, BRANCH)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Done), SystemTime::now());
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown, vec![push_failure_text(other_branch, OTHER_OID, "remote: rejected")], "only the branch that went through is cleared");
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
    assert!(take_push_failures(repo.common.as_path()).is_empty(), "a shadow review never nags the person");
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

/// A claim a service left when it died or was replaced is run again by the next service's first drain at once, not after
///  the lease.
#[test]
fn a_claim_left_by_an_earlier_service_is_run_again_at_once() {
    let repo: Repo = repo("work-lease");
    let stuck: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&stuck));
    let work_dir: PathBuf = work_directory(repo.common.as_path());
    let started: SystemTime = SystemTime::now();
    // The earlier service claims it and dies.
    assert!(claim_next_of(work_dir.as_path(), started, &SERVICE_KINDS).expect("claim").is_some());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, started);
    assert_eq!(recorder.pushes.borrow().len(), 1, "a claim left by an earlier service runs again without waiting for the lease");
    assert_eq!(names(&repo, DONE), vec![stuck.id()]);
    assert!(names(&repo, CLAIMED).is_empty());
    remove(repo.root.as_path());
}

/// A linked worktree's Git directory, whose `commondir` file names the repository's common directory the way Git writes it.
fn linked_git_dir(repo: &Repo) -> PathBuf {
    let linked: PathBuf = repo.common.join("worktrees").join("w");
    std::fs::create_dir_all(&linked).expect("linked git directory");
    std::fs::write(linked.join("commondir"), "../..\n").expect("commondir");
    return linked;
}

/// Marks the common directory as the one of a bare repository, the way `git clone --bare` writes it.
fn make_bare(repo: &Repo) {
    std::fs::write(repo.common.join("config"), "[core]\n\tbare = true\n").expect("config");
}

/// Records the releases before one service per worktree left in the common directory are run by the instance of a linked
///  worktree of a bare repository, which has no main worktree whose instance would run them.
#[test]
fn a_linked_worktrees_wake_drains_the_records_left_in_the_common_directory_of_a_bare_repository() {
    let repo: Repo = repo("work-common-legacy");
    make_bare(&repo);
    let legacy: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&legacy));
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, SystemTime::now());
    assert_eq!(recorder.pushes.borrow().len(), 1, "the legacy push ran");
    assert_eq!(names(&repo, DONE), vec![legacy.id()]);
    remove(repo.root.as_path());
}

/// A claim in the common directory of a bare repository is only taken back after its lease: another linked worktree's
///  instance may be running it.
#[test]
fn a_claim_in_the_common_directory_is_taken_back_only_after_its_lease() {
    let repo: Repo = repo("work-common-lease");
    make_bare(&repo);
    let held: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&held));
    let work_dir: PathBuf = work_directory(repo.common.as_path());
    let now: SystemTime = SystemTime::now();
    assert!(claim_next_of(work_dir.as_path(), now, &SERVICE_KINDS).expect("claim").is_some());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, now);
    assert!(recorder.pushes.borrow().is_empty(), "a live claim is left to its holder");
    assert_eq!(names(&repo, CLAIMED), vec![held.id()]);
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, now + SHARED_CLAIM_LEASE + std::time::Duration::from_secs(1));
    assert_eq!(recorder.pushes.borrow().len(), 1, "an expired claim runs again");
    assert_eq!(names(&repo, DONE), vec![held.id()]);
    remove(repo.root.as_path());
}

/// The main worktree's instance drains the common directory as its own, so a wake from it adds nothing here.
#[test]
fn the_common_directory_is_not_drained_twice_from_the_main_worktree() {
    let repo: Repo = repo("work-common-main");
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain_common_records(repo.common.as_path(), &recorder, SystemTime::now());
    assert!(recorder.pushes.borrow().is_empty());
    assert_eq!(names(&repo, PENDING).len(), 1);
    remove(repo.root.as_path());
}

/// A repository with a main worktree keeps its common directory for the main worktree's instance, which has its own records
///  there: a linked worktree's instance neither runs them nor takes back a claim the main instance holds, however old.
#[test]
fn a_linked_worktrees_wake_leaves_the_common_directory_of_a_repository_with_a_main_worktree_alone() {
    let repo: Repo = repo("work-common-main-owned");
    let pending: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    let held: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, "refs/heads/other");
    enqueue_all(&repo, std::slice::from_ref(&held));
    let work_dir: PathBuf = work_directory(repo.common.as_path());
    let now: SystemTime = SystemTime::now();
    assert!(claim_next_of(work_dir.as_path(), now, &SERVICE_KINDS).expect("claim").is_some(), "the main instance holds one item");
    enqueue_all(&repo, std::slice::from_ref(&pending));
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    let after_the_lease: SystemTime = now + SHARED_CLAIM_LEASE + std::time::Duration::from_secs(1);
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, after_the_lease);
    assert!(recorder.pushes.borrow().is_empty(), "the main worktree's records are not run by a linked instance");
    assert_eq!(names(&repo, PENDING), vec![pending.id()]);
    assert_eq!(names(&repo, CLAIMED), vec![held.id()]);
    remove(repo.root.as_path());
}

/// The common directory of a bare repository is shared by its linked instances, so its own drain does not take back a claim
///  at once either; only the lease does.
#[test]
fn a_bare_repositorys_common_directory_is_not_drained_with_the_claims_of_a_live_instance_taken_back() {
    let repo: Repo = repo("work-bare-own");
    make_bare(&repo);
    let held: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&held));
    let work_dir: PathBuf = work_directory(repo.common.as_path());
    let now: SystemTime = SystemTime::now();
    assert!(claim_next_of(work_dir.as_path(), now, &SERVICE_KINDS).expect("claim").is_some());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, now);
    assert!(recorder.pushes.borrow().is_empty(), "a live claim is left to its holder");
    assert_eq!(names(&repo, CLAIMED), vec![held.id()]);
    drain(repo.common.as_path(), &recorder, now + SHARED_CLAIM_LEASE + std::time::Duration::from_secs(1));
    assert_eq!(recorder.pushes.borrow().len(), 1, "an expired claim runs again");
    remove(repo.root.as_path());
}

/// Only `core.bare` of the common file makes a directory bare, in Git's boolean spellings.
#[test]
fn a_common_directory_is_bare_only_when_its_config_says_so() {
    let repo: Repo = repo("work-bare-spellings");
    assert!(!is_bare_common_directory(repo.common.as_path()), "no config file");
    for (text, expected) in [
        ("[core]\n\tbare = true\n", true),
        ("[Core]\n\tBare = yes\n", true),
        ("[core]\n\tbare\n", true),
        ("[core]\n\tbare = false\n", false),
        ("[core]\n\tbare = true\n\tbare = false\n", false),
        ("[core]\n\tfilemode = true\n", false),
        ("not a config file", false),
    ] {
        std::fs::write(repo.common.join("config"), text).expect("config");
        assert_eq!(is_bare_common_directory(repo.common.as_path()), expected, "{text:?}");
    }
    remove(repo.root.as_path());
}

/// Handlers whose push waits until every claim of the round has been renewed, which only a running renewal does.
struct RenewalProbe {
    work_dir: PathBuf,
    ids: Vec<String>,
    stale_before: SystemTime,
    renewed: RefCell<Vec<bool>>,
}

impl WorkHandlers for RenewalProbe {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        // The bound only ends a failing run: a passing run leaves this loop as soon as the renewal has stamped every claim.
        let bound: std::time::Instant = std::time::Instant::now() + std::time::Duration::from_secs(20);
        let all_renewed = || -> bool {
            return self.ids.iter().all(|id| {
                // A claim that is no longer in `claimed` was settled by an earlier push of the round.
                return match std::fs::metadata(state_directory(self.work_dir.as_path(), CLAIMED).join(id)) {
                    Ok(meta) => meta.modified().is_ok_and(|stamp| return stamp > self.stale_before),
                    Err(_) => true,
                };
            });
        };
        while !all_renewed() && std::time::Instant::now() < bound {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        self.renewed.borrow_mut().push(all_renewed());
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// A drain of a shared directory renews every claim it holds while it runs the first push, including the claim of an item
///  that waits behind it, so none of them looks abandoned to another instance however long the push takes.
#[test]
fn a_shared_drain_renews_the_claims_it_holds_while_it_works() {
    let repo: Repo = repo("work-renewal");
    make_bare(&repo);
    let items: Vec<WorkItem> = vec![item(&repo, WorkKind::Push, OID, BRANCH), item(&repo, WorkKind::Push, OTHER_OID, "refs/heads/other")];
    enqueue_all(&repo, &items);
    let lease: std::time::Duration = std::time::Duration::from_millis(300);
    // Claims are stamped an hour in the past, as if their lease had long run out unless something renews it.
    let stale: SystemTime = SystemTime::now() - std::time::Duration::from_secs(3600);
    let probe: RenewalProbe = RenewalProbe {
        work_dir: work_directory(repo.common.as_path()),
        ids: items.iter().map(WorkItem::id).collect(),
        stale_before: stale + std::time::Duration::from_secs(1800),
        renewed: RefCell::new(Vec::new()),
    };
    drain_common_records_with_lease(linked_git_dir(&repo).as_path(), &probe, stale, lease);
    assert_eq!(*probe.renewed.borrow(), vec![true; probe.renewed.borrow().len()], "every claim was renewed during the first push");
    assert_eq!(probe.renewed.borrow().len(), items.len(), "one push per branch ran");
    assert_eq!(names(&repo, DONE).len(), items.len());
    remove(repo.root.as_path());
}

/// A claim taken late in a round is stamped with the current time as it is taken, not with the time the round started and not
///  after a renewal interval: the interval here is longer than the whole test, so only the stamp at claim time can satisfy it.
#[test]
fn a_shared_claim_is_stamped_with_the_current_time_when_it_is_taken() {
    let repo: Repo = repo("work-claim-stamp");
    make_bare(&repo);
    let items: Vec<WorkItem> = vec![item(&repo, WorkKind::Push, OID, BRANCH)];
    enqueue_all(&repo, &items);
    let lease: std::time::Duration = std::time::Duration::from_secs(3000);
    let round_start: SystemTime = SystemTime::now() - std::time::Duration::from_secs(3600);
    let probe: RenewalProbe = RenewalProbe {
        work_dir: work_directory(repo.common.as_path()),
        ids: items.iter().map(WorkItem::id).collect(),
        stale_before: round_start + std::time::Duration::from_secs(1800),
        renewed: RefCell::new(Vec::new()),
    };
    drain_common_records_with_lease(linked_git_dir(&repo).as_path(), &probe, round_start, lease);
    assert_eq!(*probe.renewed.borrow(), vec![true]);
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

/// A round keeps a wake's credential only until its repository has been drained: nothing of it is left afterwards, for the
/// repository that brought it or any other, and a repeated directory is drained once.
#[test]
fn a_round_forgets_the_credentials_it_was_given() {
    let first: Repo = repo("work-round");
    let second: Repo = repo("work-round-other");
    let handlers: GitHandlers = GitHandlers::new(crate::test_support::git_context());
    let key: ReviewKey = ReviewKey::new("test-key-not-a-secret").expect("a key");
    let wakes: Vec<Wake> = vec![
        (first.common.clone(), Some(key.clone())),
        (first.common.clone(), None),
        (second.common.clone(), Some(key)),
    ];
    run_round(wakes, &handlers, SystemTime::now());
    assert_eq!(handlers.review_key_of(first.common.as_path()), None);
    assert_eq!(handlers.review_key_of(second.common.as_path()), None);
    remove(first.root.as_path());
    remove(second.root.as_path());
}

/// A review runs end to end through the real handlers with the context the service really has, which names no
/// repository: the work item is claimed, the commit is read with real Git (the patch, and the file texts through a
/// Git command that takes input), the loopback gateway is asked with the key from the wake, and the entry is appended.
#[test]
fn a_review_runs_through_the_real_handlers_in_a_service_context() {
    use crate::standards_review_input::LOG_RELATIVE;
    use crate::standards_review_mock::{Answer, Mock, http_reply};
    use crate::test_support::{git, git_context, git_text, repository};
    const SECRET: &str = "test-key-not-a-secret";
    const ANSWER: &str = r#"{"answers":[{"choice":"no","probabilities":[{"value":"no","probability":1}]}]}"#;
    let parent: PathBuf = fixture("work-real-review");
    let root: PathBuf = repository(parent.as_path(), "repo");
    let log: PathBuf = root.join(LOG_RELATIVE);
    std::fs::create_dir_all(log.parent().expect("parent")).expect("directories");
    std::fs::write(log.as_path(), "# Log\n").expect("log");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/a.ts"), "export const a = 1\n").expect("code");
    git(root.as_path(), &["add", "--", LOG_RELATIVE, "src/a.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=feat(a): add a"]);
    let common: PathBuf = root.join(".git");
    let review: WorkItem = WorkItem {
        kind: WorkKind::Review,
        repository: root.display().to_string(),
        git_dir: common.display().to_string(),
        branch_ref: BRANCH.to_string(),
        oid: git_text(root.as_path(), &["rev-parse", "HEAD"]),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(common.as_path()).as_path(), &review).expect("enqueue");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    // The service's own context: no repository is named, because one service serves many.
    let service_context: GitContext = GitContext { common_dir: None, ..git_context() };
    let handlers: GitHandlers = GitHandlers::new(service_context).with_review_endpoint(mock.url.as_str());
    run_round(vec![(common.clone(), ReviewKey::new(SECRET))], &handlers, SystemTime::now());
    let seen: String = mock.finish().to_ascii_lowercase();
    let settled = |state: &str| -> Vec<String> {
        return std::fs::read_dir(state_directory(work_directory(common.as_path()).as_path(), state))
            .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
            .unwrap_or_default();
    };
    assert_eq!(settled(FAILED), Vec::<String>::new(), "the review must not fail");
    assert_eq!(settled(DONE), vec![review.id()]);
    assert!(seen.contains(&format!("authorization: bearer {SECRET}\r\n")), "{seen}");
    assert!(seen.contains("src/a.ts"), "the request carries the changed file");
    let written: String = std::fs::read_to_string(log.as_path()).expect("log");
    assert!(written.starts_with("# Log\n") && written.contains("cost_verdict: no"), "{written}");
    assert!(!written.contains(SECRET));
    remove(parent.as_path());
}
