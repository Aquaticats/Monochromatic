//! Controls for the service's after-commit work consumer, with handlers that record what they are asked.

use super::*;
use crate::push_failure_notice::{push_failure_text, take_push_failures};
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, PENDING, enqueue, find_claim, id_of_entry, requeue_expired, state_directory};
use std::cell::RefCell;
use std::time::Duration;

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

/// The ids of the claimed items; a claim's file name has the id, a generation and a clock reading.
fn claimed_ids(repo: &Repo) -> Vec<String> {
    return names(repo, CLAIMED).iter().map(|name| { return id_of_entry(name.as_str()).to_owned(); }).collect();
}

/// Several pending pushes of one branch are one push of the newest commit, and every item is settled as done.
#[test]
fn pending_pushes_of_one_branch_become_one_push() {
    let repo: Repo = repo("work-coalesce");
    let items: Vec<WorkItem> = vec![item(&repo, WorkKind::Push, OID, BRANCH), item(&repo, WorkKind::Push, OTHER_OID, BRANCH)];
    enqueue_all(&repo, &items);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, &ClaimTime::read());
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
    drain(repo.common.as_path(), &recorder, &ClaimTime::read());
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
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(said.to_string())), &ClaimTime::read());
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
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(String::from("remote: rejected"))), &ClaimTime::read());
    let third: &str = "fedcba9876543210fedcba9876543210fedcba98";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, third, BRANCH)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Done), &ClaimTime::read());
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown, vec![push_failure_text(other_branch, OTHER_OID, "remote: rejected")], "only the branch that went through is cleared");
    remove(repo.root.as_path());
}

/// A skipped push settles as done, because nothing is left to retry, and leaves a notice that says so; an earlier failure of
///  the branch is still true and stays beside it, where a successful push would have removed it.
#[test]
fn a_skipped_push_leaves_a_notice_and_keeps_an_earlier_failure() {
    let repo: Repo = repo("work-notice-skipped");
    let said: &str = "remote: rejected";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(said.to_string())), &ClaimTime::read());
    let reason: &str = "cli-git: the branch refs/heads/main tracks no remote branch that can be named; skipping auto-push.";
    let skipped: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&skipped));
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Skipped(reason.to_string())), &ClaimTime::read());
    assert_eq!(names(&repo, DONE), vec![skipped.id()], "nothing is left to retry");
    assert!(names(&repo, PENDING).is_empty() && names(&repo, CLAIMED).is_empty());
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown.len(), 2, "the failure and the skip: {shown:?}");
    assert!(shown.contains(&push_failure_text(BRANCH, OID, said)), "{shown:?}");
    assert!(shown.iter().any(|text| { return text.contains(reason) && text.contains(OTHER_OID) && text.contains("was not made"); }), "{shown:?}");
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
    drain(repo.common.as_path(), &recorder, &ClaimTime::read());
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
    drain(repo.common.as_path(), &recorder, &ClaimTime::read());
    assert_eq!(names(&repo, FAILED), vec![review.id()]);
    assert!(names(&repo, PENDING).is_empty() && names(&repo, CLAIMED).is_empty());
    assert!(take_push_failures(repo.common.as_path()).is_empty(), "a shadow review never nags the person");
    drain(repo.common.as_path(), &recorder, &ClaimTime::read());
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
    drain(repo.common.as_path(), &recorder, &ClaimTime::read());
    assert!(recorder.pushes.borrow().is_empty());
    assert_eq!(names(&repo, FAILED).len(), 2);
    remove(repo.root.as_path());
}

/// A claim a service left when it died is run again by the next service's drain once its lease is over, not before: the
///  holder may be alive beside the new instance. The drain says how long to wait, so the service looks again then.
#[test]
fn a_claim_left_by_an_earlier_service_is_run_again_when_its_lease_is_over() {
    let repo: Repo = repo("work-lease");
    let stuck: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&stuck));
    let work_dir: PathBuf = work_directory(repo.common.as_path());
    let started: ClaimTime = ClaimTime::read();
    // The earlier service claims it and dies.
    assert!(claim_next_of(work_dir.as_path(), &started, &SERVICE_KINDS).expect("claim").is_some());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    let asked: Option<Duration> = drain(repo.common.as_path(), &recorder, &started);
    assert!(recorder.pushes.borrow().is_empty(), "the holder may be alive, so its claim is left to it");
    assert_eq!(names(&repo, CLAIMED).len(), 1);
    let wait: Duration = asked.expect("the drain says when to look again");
    assert!(wait > Duration::ZERO && wait <= CLAIM_LEASE, "{wait:?}");
    let after: Option<Duration> = drain(repo.common.as_path(), &recorder, &started.advanced(CLAIM_LEASE + Duration::from_secs(1)));
    assert_eq!(recorder.pushes.borrow().len(), 1, "a claim whose lease is over runs again");
    assert_eq!(names(&repo, DONE), vec![stuck.id()]);
    assert!(names(&repo, CLAIMED).is_empty());
    assert_eq!(after, None, "nothing is left to wait for");
    remove(repo.root.as_path());
}

/// A handler that holds its push open until the test lets it go, and says when the push began; the other handler of a pair
/// drains the same directory meanwhile.
struct Gate {
    began: std::sync::mpsc::Sender<()>,
    release: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
    pushes: std::sync::atomic::AtomicUsize,
}

impl WorkHandlers for Gate {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        self.pushes.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.began.send(()).expect("the test is listening");
        // The bound only ends a failing run.
        let _released = self.release.lock().unwrap_or_else(std::sync::PoisonError::into_inner).recv_timeout(Duration::from_secs(30));
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// A restarted service that drains while the instance it replaces is still in a slow push leaves that push's claim alone for
/// as long as the holder renews it, however many leases the push outlasts; the push runs once and the holder settles it.
#[test]
fn a_drain_beside_a_slow_push_leaves_its_claim_alone() {
    let repo: Repo = repo("work-overlap");
    let slow: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&slow));
    let lease: Duration = Duration::from_millis(300);
    let (began, began_receiver) = std::sync::mpsc::channel();
    let (release, release_receiver) = std::sync::mpsc::channel();
    let dying: Gate = Gate { began, release: std::sync::Mutex::new(release_receiver), pushes: std::sync::atomic::AtomicUsize::new(0) };
    let restarted: Recorder = Recorder::answering(Verdict::Done);
    let wait: Option<Duration> = std::thread::scope(|scope| {
        let holder = scope.spawn(|| { drain_with_lease(repo.common.as_path(), &dying, &ClaimTime::read(), lease); });
        began_receiver.recv_timeout(Duration::from_secs(30)).expect("the push began");
        // Several leases pass while the push runs; only a renewing holder keeps its claim through them.
        std::thread::sleep(lease * 4);
        let wait: Option<Duration> = drain_with_lease(repo.common.as_path(), &restarted, &ClaimTime::read(), lease);
        release.send(()).expect("the push is waiting");
        holder.join().expect("the first instance finished");
        return wait;
    });
    assert!(restarted.pushes.borrow().is_empty(), "the second instance did not run the item the first is running");
    assert!(wait.is_some(), "it was told to look again when the claim's lease would be over");
    assert_eq!(dying.pushes.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(names(&repo, DONE), vec![slow.id()], "the holder settled its own claim");
    assert!(names(&repo, CLAIMED).is_empty() && names(&repo, PENDING).is_empty());
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
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, &ClaimTime::read());
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
    let now: ClaimTime = ClaimTime::read();
    assert!(claim_next_of(work_dir.as_path(), &now, &SERVICE_KINDS).expect("claim").is_some());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, &now);
    assert!(recorder.pushes.borrow().is_empty(), "a live claim is left to its holder");
    assert_eq!(claimed_ids(&repo), vec![held.id()]);
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, &now.advanced(CLAIM_LEASE + Duration::from_secs(1)));
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
    drain_common_records(repo.common.as_path(), &recorder, &ClaimTime::read());
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
    let now: ClaimTime = ClaimTime::read();
    assert!(claim_next_of(work_dir.as_path(), &now, &SERVICE_KINDS).expect("claim").is_some(), "the main instance holds one item");
    enqueue_all(&repo, std::slice::from_ref(&pending));
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    let after_the_lease: ClaimTime = now.advanced(CLAIM_LEASE + Duration::from_secs(1));
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, &after_the_lease);
    assert!(recorder.pushes.borrow().is_empty(), "the main worktree's records are not run by a linked instance");
    assert_eq!(names(&repo, PENDING), vec![pending.id()]);
    assert_eq!(claimed_ids(&repo), vec![held.id()]);
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
    let now: ClaimTime = ClaimTime::read();
    assert!(claim_next_of(work_dir.as_path(), &now, &SERVICE_KINDS).expect("claim").is_some());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, &now);
    assert!(recorder.pushes.borrow().is_empty(), "a live claim is left to its holder");
    assert_eq!(claimed_ids(&repo), vec![held.id()]);
    drain(repo.common.as_path(), &recorder, &now.advanced(CLAIM_LEASE + Duration::from_secs(1)));
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

/// With `extensions.worktreeConfig` on, Git reads `core.bare` from the common directory's `config.worktree` after its `config`
///  (checked by hand with Git 2.56.0: `git rev-parse --is-bare-repository` prints `true` for a bare repository whose `config`
///  has no `core.bare` and whose `config.worktree` has it), so that spelling makes the directory bare too, and the last of the
///  two files decides. Without the extension Git does not read the file, and neither does this reader.
#[test]
fn a_common_directory_is_bare_when_config_worktree_says_so_and_the_extension_is_on() {
    let repo: Repo = repo("work-bare-worktree-config");
    let enabled: &str = "[core]\n\trepositoryformatversion = 0\n[extensions]\n\tworktreeConfig = true\n";
    for (config, worktree, expected) in [
        (enabled, "[core]\n\tbare = true\n", true),
        (enabled, "[core]\n\tbare = false\n", false),
        ("[core]\n\trepositoryformatversion = 0\n\tbare = true\n[extensions]\n\tworktreeConfig = true\n", "[core]\n\tbare = false\n", false),
        ("[core]\n\trepositoryformatversion = 0\n\tbare = false\n[extensions]\n\tworktreeConfig = true\n", "[core]\n\tbare = true\n", true),
        ("[core]\n\trepositoryformatversion = 0\n", "[core]\n\tbare = true\n", false),
        ("[core]\n\trepositoryformatversion = 0\n[extensions]\n\tworktreeConfig = false\n", "[core]\n\tbare = true\n", false),
        ("[extensions]\n\tworktreeConfig = true\n", "[core]\n\tbare = true\n", false),
        (enabled, "not a config file", false),
    ] {
        std::fs::write(repo.common.join("config"), config).expect("config");
        std::fs::write(repo.common.join("config.worktree"), worktree).expect("config.worktree");
        assert_eq!(is_bare_common_directory(repo.common.as_path()), expected, "{config:?} then {worktree:?}");
    }
    remove(repo.root.as_path());
}

/// The records a bare repository's earlier releases left are run by a linked instance when only `config.worktree` says bare.
#[test]
fn a_linked_worktrees_wake_drains_the_records_of_a_bare_repository_that_says_bare_in_config_worktree() {
    let repo: Repo = repo("work-common-legacy-worktree-config");
    std::fs::write(repo.common.join("config"), "[core]\n\trepositoryformatversion = 0\n[extensions]\n\tworktreeConfig = true\n").expect("config");
    std::fs::write(repo.common.join("config.worktree"), "[core]\n\tbare = true\n").expect("config.worktree");
    let legacy: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&legacy));
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain_common_records(linked_git_dir(&repo).as_path(), &recorder, &ClaimTime::read());
    assert_eq!(recorder.pushes.borrow().len(), 1, "the legacy push ran");
    assert_eq!(names(&repo, DONE), vec![legacy.id()]);
    remove(repo.root.as_path());
}

/// Handlers whose push waits until every claim of the round has been renewed, which only a running renewal does.
struct RenewalProbe {
    work_dir: PathBuf,
    ids: Vec<String>,
    renewed: RefCell<Vec<bool>>,
}

impl RenewalProbe {
    /// The file name each of the round's claims has now; nothing for a claim that was settled by an earlier push of the round.
    fn names_now(&self) -> Vec<Option<String>> {
        return self
            .ids
            .iter()
            .map(|id| { return find_claim(self.work_dir.as_path(), id.as_str()).map(|path| { return path.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default(); }); })
            .collect();
    }
}

impl WorkHandlers for RenewalProbe {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        let first: Vec<Option<String>> = self.names_now();
        // The bound only ends a failing run: a passing run leaves this loop as soon as the renewal has renamed every claim.
        let bound: std::time::Instant = std::time::Instant::now() + std::time::Duration::from_secs(20);
        let all_renewed = || -> bool {
            return self.names_now().iter().zip(&first).all(|(now, before)| { return now.is_none() || now != before; });
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

/// A drain renews every claim it holds while it runs the first push, including the claim of an item that waits behind it, so
///  none of them looks abandoned to another instance however long the push takes.
#[test]
fn a_drain_renews_the_claims_it_holds_while_it_works() {
    let repo: Repo = repo("work-renewal");
    make_bare(&repo);
    let items: Vec<WorkItem> = vec![item(&repo, WorkKind::Push, OID, BRANCH), item(&repo, WorkKind::Push, OTHER_OID, "refs/heads/other")];
    enqueue_all(&repo, &items);
    let lease: Duration = Duration::from_millis(300);
    let probe: RenewalProbe = RenewalProbe { work_dir: work_directory(repo.common.as_path()), ids: items.iter().map(WorkItem::id).collect(), renewed: RefCell::new(Vec::new()) };
    drain_common_records_with_lease(linked_git_dir(&repo).as_path(), &probe, &ClaimTime::read(), lease);
    assert_eq!(*probe.renewed.borrow(), vec![true; probe.renewed.borrow().len()], "every claim was renewed during the first push");
    assert_eq!(probe.renewed.borrow().len(), items.len(), "one push per branch ran");
    assert_eq!(names(&repo, DONE).len(), items.len());
    remove(repo.root.as_path());
}

/// A handler whose first push is the moment another instance takes over every claim of the directory, and that waits for this
/// instance's renewal to notice.
struct TakenOver {
    work_dir: PathBuf,
    lease: Duration,
    pushed: RefCell<Vec<String>>,
}

impl WorkHandlers for TakenOver {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        let first: bool = self.pushed.borrow().is_empty();
        self.pushed.borrow_mut().extend(items.iter().map(|item| { return item.oid.clone(); }));
        if first {
            // The other instance reads the clocks a long time on, so every claim of this one is over its lease; this instance
            //  renews every third of the lease, and one renewal is all it takes to find its claims gone.
            let far_on: ClaimTime = ClaimTime::read().advanced(Duration::from_secs(3600));
            assert_eq!(requeue_expired(self.work_dir.as_path(), &far_on, self.lease, MAX_TRIES), 2, "both claims were taken over");
            std::thread::sleep(self.lease);
        }
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// A claim that another instance took over while an earlier push of the round ran is not run from this instance's old claim:
/// the item is claimed again and run once from the new claim, and the first push settles nothing of it.
#[test]
fn a_claim_taken_over_before_its_push_is_not_run_from_the_old_claim() {
    let repo: Repo = repo("work-taken-over");
    let first: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    let second: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, "refs/heads/other");
    enqueue_all(&repo, &[first.clone(), second.clone()]);
    let lease: Duration = Duration::from_millis(300);
    let handlers: TakenOver = TakenOver { work_dir: work_directory(repo.common.as_path()), lease, pushed: RefCell::new(Vec::new()) };
    drain_with_lease(repo.common.as_path(), &handlers, &ClaimTime::read(), lease);
    // Run from the old claims, the round would push the first branch, then the second, then both again from their new claims.
    assert_eq!(*handlers.pushed.borrow(), vec![OID.to_string(), OID.to_string(), OTHER_OID.to_string()]);
    let mut done: Vec<String> = names(&repo, DONE);
    done.sort();
    let mut expected: Vec<String> = vec![first.id(), second.id()];
    expected.sort();
    assert_eq!(done, expected, "both settle once, from their new claims");
    assert!(names(&repo, PENDING).is_empty() && names(&repo, CLAIMED).is_empty());
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
    run_round(wakes, &handlers, &ClaimTime::read());
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
    use crate::standards_review_input::{TRACKED_LOG_RELATIVE, shared_log_path};
    use crate::standards_review_mock::{Answer, Mock, http_reply};
    use crate::test_support::{git, git_context, git_text, repository};
    const SECRET: &str = "test-key-not-a-secret";
    const ANSWER: &str = r#"{"answers":[{"choice":"no","probabilities":[{"value":"no","probability":1}]}]}"#;
    let parent: PathBuf = fixture("work-real-review");
    let root: PathBuf = repository(parent.as_path(), "repo");
    let log: PathBuf = root.join(TRACKED_LOG_RELATIVE);
    std::fs::create_dir_all(log.parent().expect("parent")).expect("directories");
    std::fs::write(log.as_path(), "# Log\n").expect("log");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/a.ts"), "export const a = 1\n").expect("code");
    git(root.as_path(), &["add", "--", TRACKED_LOG_RELATIVE, "src/a.ts"]);
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
    run_round(vec![(common.clone(), ReviewKey::new(SECRET))], &handlers, &ClaimTime::read());
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
    let written: String = std::fs::read_to_string(shared_log_path(common.as_path())).expect("the live log");
    assert!(written.starts_with("- Date:\n") && written.contains("cost_verdict: no"), "{written}");
    assert!(!written.contains(SECRET));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("tracked log"), "# Log\n", "the tracked log is never written");
    remove(parent.as_path());
}
