//! Controls for the service's after-commit work consumer,
//! with handlers that record what they are asked.

use crate::work_item_recovery::requeue_expired;
use crate::work_item::{DONE, FAILED, WorkKind, claim_next_of};
use crate::work_item_state_directories_and_claim_names::take_listings;
use crate::push_retry::{Due, PushRetries};
use crate::service_work_common_directory::is_bare_common_directory;
use crate::service_work_run_round::run_round;
use super::*;
use crate::push_failure_notice::{PushOutcome, notice_directory, push_failure_text, push_failures};
use crate::push_retry::FIRST_PAUSE;
use crate::service_claim_renewal::{RENEWALS_PER_LEASE, renewal_interval};
use crate::service_claim_renewal_running::{DrivenRenewal, Renewal};
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, PENDING, enqueue, find_claim, id_of_entry, state_directory};
use std::cell::RefCell;
use std::ffi::OsString;
use std::os::unix::fs::OpenOptionsExt as _;
use std::time::{Duration, Instant};

const OID: &str = "0123456789abcdef0123456789abcdef01234567";

/// What:
///  The words of every record,
/// shown to a buffer and removed as a command shows and removes them.
/// Why:
///   A record goes only once its words are out,
/// so these controls read one through the step the person's terminal receives
///  it by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shownNotices(pushFailures(gitDir))
/// ```
fn take_push_failures(git_dir: &Path) -> Vec<String> {
    return crate::test_support::shown_notices(push_failures(git_dir));
}
const OTHER_OID: &str = "89abcdef0123456789abcdef0123456789abcdef";
const BRANCH: &str = "refs/heads/main";
const THIRD_OID: &str = "456789abcdef0123456789abcdef0123456789ab";

/// Handlers that record the calls and answer from a fixed verdict.
struct Recorder {
    verdict: Verdict,
    pushes: RefCell<Vec<Vec<String>>>,
    upkeeps: RefCell<Vec<PathBuf>>,
    reviews: RefCell<Vec<String>>,
    trashes: RefCell<Vec<String>>,
    deletions: RefCell<Vec<String>>,
    aheads: RefCell<Vec<String>>,
}

impl Recorder {
    fn answering(verdict: Verdict) -> Recorder {
        return Recorder {
            verdict,
            pushes: RefCell::new(Vec::new()),
            upkeeps: RefCell::new(Vec::new()),
            reviews: RefCell::new(Vec::new()),
            trashes: RefCell::new(Vec::new()),
            deletions: RefCell::new(Vec::new()),
            aheads: RefCell::new(Vec::new()),
        };
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

    fn trash(&self, item: &WorkItem) -> Verdict {
        self.trashes.borrow_mut().push(item.oid.clone());
        return self.verdict.clone();
    }

    /// A removal hands the path it renamed out of the person's way,
    /// so the record is that path rather than a commit id.
    fn subvolume_delete(&self, item: &WorkItem) -> Verdict {
        self.deletions.borrow_mut().push(item.parent.clone());
        return self.verdict.clone();
    }

    /// A snapshot ahead names the worktree root to copy,
    /// so the record is that root rather than a commit id.
    fn snapshot_ahead(&self, item: &WorkItem) -> Verdict {
        self.aheads.borrow_mut().push(item.repository.display().to_string());
        return self.verdict.clone();
    }
}

/// A repository directory standing in for a common Git directory,
/// with a worktree directory beside it.
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
        repository: repo.worktree.to_path_buf(),
        git_dir: repo.common.to_path_buf(),
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

/// The ids of the claimed items;
/// a claim's file name has the id,
/// a generation and a clock reading.
fn claimed_ids(repo: &Repo) -> Vec<String> {
    return names(repo, CLAIMED).iter().map(|name| { return id_of_entry(name.as_str()).to_owned(); }).collect();
}

/// Several pending pushes of one branch are one push of the newest commit,
/// and every item is settled as done.
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

/// Pushes of different branches stay separate,
/// and upkeep runs once per repository however many items it has.
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

/// A failed push is settled as failed and leaves one record that names the branch,
/// the commit and what Git said,
/// shown once.
#[test]
fn a_failed_push_leaves_a_notice_that_is_shown_once() {
    let repo: Repo = repo("work-notice");
    let said: &str = "remote: permission denied";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(said.to_string())), &ClaimTime::read());
    assert_eq!(names(&repo, FAILED).len(), 1);
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown, vec![push_failure_text(BRANCH, OID, PushOutcome::Refused, said)]);
    assert!(take_push_failures(repo.common.as_path()).is_empty(), "a notice is shown once");
    remove(repo.root.as_path());
}

/// A later push of the branch that goes through removes the branch's records that nobody has seen yet;
/// another branch's
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
    assert_eq!(shown, vec![push_failure_text(other_branch, OTHER_OID, PushOutcome::Refused, "remote: rejected")], "only the branch that went through is cleared");
    remove(repo.root.as_path());
}

/// A skipped push settles as done,
/// because nothing is left to retry,
/// and leaves a notice that says so;
/// an earlier failure of
///  the branch is still true and stays beside it,
/// where a successful push would have removed it.
#[test]
fn a_skipped_push_leaves_a_notice_and_keeps_an_earlier_failure() {
    let repo: Repo = repo("work-notice-skipped");
    let said: &str = "remote: rejected";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(said.to_string())), &ClaimTime::read());
    let reason: &str = "monochromatic-git-policy-cli: the branch refs/heads/main tracks no remote branch that can be named; skipping auto-push.";
    let skipped: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&skipped));
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Skipped(reason.to_string())), &ClaimTime::read());
    assert_eq!(names(&repo, DONE), vec![skipped.id()], "nothing is left to retry");
    assert!(names(&repo, PENDING).is_empty() && names(&repo, CLAIMED).is_empty());
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown.len(), 2, "the failure and the skip: {shown:?}");
    assert!(shown.contains(&push_failure_text(BRANCH, OID, PushOutcome::Refused, said)), "{shown:?}");
    assert!(shown.iter().any(|text| { return text.contains(reason) && text.contains(OTHER_OID) && text.contains("was not made"); }), "{shown:?}");
    remove(repo.root.as_path());
}

/// Each review item is its own review (never coalesced),
/// settled as done,
/// and it runs after the pushes of the same round.
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

/// A review that fails is settled as failed (a plain record),
/// leaves no notice for the person,
/// and is not run again.
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
    let elsewhere: WorkItem = WorkItem { git_dir: repo.root.join("elsewhere"), ..item(&repo, WorkKind::Push, OID, BRANCH) };
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

/// A claim a service left when it died is run again by the next service's drain once its lease is over,
/// not before:
/// the
///  holder may be alive beside the new instance.
/// The drain says how long to wait,
/// so the service looks again then.
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

/// The bound on a push the controls below hold open:
/// a failing run's way out of the wait,
/// and never how long a passing one waits.
const GATE_CEILING: Duration = Duration::from_secs(30);

/// What:
///  Lets the held push go however the control ends.
/// Why:
///   A control that fails before it lets the push go otherwise leaves the scope joining a holder that waits out
/// `GATE_CEILING`,
/// so the failure would take thirty seconds to report.
struct LetGo<'gate>(&'gate std::sync::mpsc::Sender<()>);

impl Drop for LetGo<'_> {
    /// Let the push go.
    fn drop(&mut self) {
        let _sent = self.0.send(());
    }
}

/// A handler that holds its push open until the test lets it go,
/// and says when the push began;
/// the other handler of a pair
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
        let _released = self.release.lock().unwrap_or_else(std::sync::PoisonError::into_inner).recv_timeout(GATE_CEILING);
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
/// as long as the holder renews it,
/// however many leases the push outlasts;
/// the push runs once and the holder settles it.
/// The renewal is driven by the control and the clocks are readings it makes,
/// so the host's scheduling decides nothing:
/// the holder renews at every third of the lease while twelve thirds pass,
/// and the second instance reads a clock four leases on.
#[test]
fn a_drain_beside_a_slow_push_leaves_its_claim_alone() {
    let repo: Repo = repo("work-overlap");
    let slow: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&slow));
    let lease: Duration = CLAIM_LEASE;
    let (began, began_receiver) = std::sync::mpsc::channel();
    let (release, release_receiver) = std::sync::mpsc::channel();
    let dying: Gate = Gate { began, release: std::sync::Mutex::new(release_receiver), pushes: std::sync::atomic::AtomicUsize::new(0) };
    let restarted: Recorder = Recorder::answering(Verdict::Done);
    let driver: DrivenRenewal = DrivenRenewal::default();
    let wait: Option<Duration> = std::thread::scope(|scope| {
        let holder = scope.spawn(|| { drain_with_renewal(repo.common.as_path(), &dying, &ClaimTime::read(), lease, Renewal::Driven(&driver)); });
        let letting_go: LetGo<'_> = LetGo(&release);
        began_receiver.recv_timeout(Duration::from_secs(60)).expect("the push began");
        // Four leases pass while the push runs; only a renewing holder keeps its claim through them.
        for third in 1..=(4 * RENEWALS_PER_LEASE) {
            assert!(driver.renew(&ClaimTime::read().advanced(lease / RENEWALS_PER_LEASE * third)), "the drain registered its claims");
        }
        let later: ClaimTime = ClaimTime::read().advanced(lease * 4);
        let wait: Option<Duration> = drain_with_lease(repo.common.as_path(), &restarted, &later, lease);
        release.send(()).expect("the push is waiting");
        drop(letting_go);
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

/// Without a renewal,
/// the same four leases are enough for the second instance to take the item and run it:
/// this is what the control above keeps from happening,
/// and the reason it renews.
#[test]
fn a_holder_that_does_not_renew_loses_its_claim_to_the_next_instance() {
    let repo: Repo = repo("work-no-renewal");
    let slow: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&slow));
    let lease: Duration = CLAIM_LEASE;
    let (began, began_receiver) = std::sync::mpsc::channel();
    let (release, release_receiver) = std::sync::mpsc::channel();
    let dying: Gate = Gate { began, release: std::sync::Mutex::new(release_receiver), pushes: std::sync::atomic::AtomicUsize::new(0) };
    let restarted: Recorder = Recorder::answering(Verdict::Done);
    let driver: DrivenRenewal = DrivenRenewal::default();
    std::thread::scope(|scope| {
        let holder = scope.spawn(|| { drain_with_renewal(repo.common.as_path(), &dying, &ClaimTime::read(), lease, Renewal::Driven(&driver)); });
        began_receiver.recv_timeout(Duration::from_secs(60)).expect("the push began");
        let later: ClaimTime = ClaimTime::read().advanced(lease * 4);
        drain_with_lease(repo.common.as_path(), &restarted, &later, lease);
        release.send(()).expect("the push is waiting");
        holder.join().expect("the first instance finished");
    });
    assert_eq!(restarted.pushes.borrow().len(), 1, "the lease ran out, so the second instance ran the item");
    remove(repo.root.as_path());
}

/// A push that every try of was cut short by a stopped service is given up by recovery with nothing having said why,
/// so the
///  drain leaves the notice for it;
/// an item that still has tries is simply run again.
#[test]
fn a_push_given_up_by_recovery_leaves_a_notice() {
    let repo: Repo = repo("work-given-up");
    let abandoned: WorkItem = WorkItem { tries: MAX_TRIES - 1, ..item(&repo, WorkKind::Push, OID, BRANCH) };
    enqueue_all(&repo, std::slice::from_ref(&abandoned));
    let started: ClaimTime = ClaimTime::read();
    assert!(claim_next_of(work_directory(repo.common.as_path()).as_path(), &started, &SERVICE_KINDS).expect("claim").is_some());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    drain(repo.common.as_path(), &recorder, &started.advanced(CLAIM_LEASE + Duration::from_secs(1)));
    assert!(recorder.pushes.borrow().is_empty(), "the push is not run again");
    assert_eq!(names(&repo, FAILED), vec![abandoned.id()]);
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(shown[0].contains(OID) && shown[0].contains("gave up") && shown[0].contains("branch main"), "{shown:?}");
    remove(repo.root.as_path());
}

/// A skip with nothing to say settles as done and leaves no notice,
/// but it is not a successful push:
/// an earlier failure of the
///  branch stays.
#[test]
fn a_silent_skip_keeps_an_earlier_failure_and_adds_no_notice() {
    let repo: Repo = repo("work-silent-skip");
    let said: &str = "remote: rejected";
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(said.to_string())), &ClaimTime::read());
    let skipped: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&skipped));
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Skipped(String::new())), &ClaimTime::read());
    assert_eq!(names(&repo, DONE), vec![skipped.id()]);
    assert_eq!(take_push_failures(repo.common.as_path()), vec![push_failure_text(BRANCH, OID, PushOutcome::Refused, said)]);
    remove(repo.root.as_path());
}

/// A skip whose notice cannot be written does not settle as done:
/// the item stays in `failed`,
/// where the missing notice can be
///  found,
/// instead of the person never hearing of it.
#[test]
fn a_skip_whose_notice_cannot_be_written_settles_as_failed() {
    let repo: Repo = repo("work-skip-unrecorded");
    let skipped: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&skipped));
    // A file where the notice directory must be makes writing any notice fail.
    std::fs::write(repo.common.join("monochromatic-git-policy-cli").join("notices"), "in the way").expect("blocker");
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Skipped(String::from("monochromatic-git-policy-cli: nothing to push to."))), &ClaimTime::read());
    assert_eq!(names(&repo, FAILED), vec![skipped.id()]);
    assert!(names(&repo, DONE).is_empty());
    remove(repo.root.as_path());
}

/// A linked worktree's Git directory,
/// whose `commondir` file names the repository's common directory the way Git writes it.
fn linked_git_dir(repo: &Repo) -> PathBuf {
    let linked: PathBuf = repo.common.join("worktrees").join("w");
    std::fs::create_dir_all(&linked).expect("linked git directory");
    std::fs::write(linked.join("commondir"), "../..\n").expect("commondir");
    return linked;
}

/// Marks the common directory as the one of a bare repository,
/// the way `git clone --bare` writes it.
fn make_bare(repo: &Repo) {
    std::fs::write(repo.common.join("config"), "[core]\n\tbare = true\n").expect("config");
}

/// Records the releases before one service per worktree left in the common directory are run by the instance of a linked
///  worktree of a bare repository,
/// which has no main worktree whose instance would run them.
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

/// A claim in the common directory of a bare repository is only taken back after its lease:
/// another linked worktree's
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

/// The main worktree's instance drains the common directory as its own,
/// so a wake from it adds nothing here.
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

/// A repository with a main worktree keeps its common directory for the main worktree's instance,
/// which has its own records
///  there:
/// a linked worktree's instance neither runs them nor takes back a claim the main instance holds,
/// however old.
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

/// The common directory of a bare repository is shared by its linked instances,
/// so its own drain does not take back a claim
///  at once either;
/// only the lease does.
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

/// Only `core.bare` of the common file makes a directory bare,
/// in Git's boolean spellings.
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

/// With `extensions.worktreeConfig` on,
/// Git reads `core.bare` from the common directory's `config.worktree` after its `config`
///  (checked by hand with Git 2.56.0:
/// `git rev-parse --is-bare-repository` prints `true` for a bare repository whose `config`
///  has no `core.bare` and whose `config.worktree` has it),
/// so that spelling makes the directory bare too,
/// and the last of the
///  two files decides.
/// Without the extension Git does not read the file,
/// and neither does this reader.
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

/// Handlers whose push waits until every claim of the round has been renewed,
/// which only a running renewal does.
struct RenewalProbe {
    work_dir: PathBuf,
    ids: Vec<String>,
    renewed: RefCell<Vec<bool>>,
}

impl RenewalProbe {
    /// The file name each of the round's claims has now;
    /// nothing for a claim that was settled by an earlier push of the round.
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

/// A drain renews every claim it holds while it runs the first push,
/// including the claim of an item that waits behind it,
/// so
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

/// A handler whose first push is the moment another instance takes over every claim of the directory,
/// and that waits for this
/// instance's renewal to notice.
struct TakenOver<'a> {
    work_dir: PathBuf,
    lease: Duration,
    pushed: RefCell<Vec<String>>,
    /// The renewal of the drain under test,
    /// driven once the other instance has taken the claims;
    /// none when the control is of a holder whose renewal has not run.
    renewal: Option<&'a DrivenRenewal>,
}

impl WorkHandlers for TakenOver<'_> {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        let first: bool = self.pushed.borrow().is_empty();
        self.pushed.borrow_mut().extend(items.iter().map(|item| { return item.oid.clone(); }));
        if first {
            // The other instance reads the clocks a long time on, so every claim of this one is over its lease; this instance
            //  renews every third of the lease, and one renewal is all it takes to find its claims gone.
            let far_on: ClaimTime = ClaimTime::read().advanced(Duration::from_secs(3600));
            assert_eq!(requeue_expired(self.work_dir.as_path(), &far_on, self.lease, MAX_TRIES), 2, "both claims were taken over");
            if let Some(driver) = self.renewal {
                assert!(driver.renew(&ClaimTime::read()), "the drain registered its claims");
            }
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
/// the item is claimed again and run once from the new claim,
/// and the first push settles nothing of it.
/// The renewal that finds the claims gone is driven by the control,
/// so no wait decides the result.
#[test]
fn a_claim_taken_over_before_its_push_is_not_run_from_the_old_claim() {
    let repo: Repo = repo("work-taken-over");
    let first: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    let second: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, "refs/heads/other");
    enqueue_all(&repo, &[first.clone(), second.clone()]);
    let lease: Duration = Duration::from_millis(300);
    let driver: DrivenRenewal = DrivenRenewal::default();
    let handlers: TakenOver = TakenOver { work_dir: work_directory(repo.common.as_path()), lease, pushed: RefCell::new(Vec::new()), renewal: Some(&driver) };
    drain_with_renewal(repo.common.as_path(), &handlers, &ClaimTime::read(), lease, Renewal::Driven(&driver));
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

/// A holder whose renewal has not run since another instance took its claims still does not run an item from a claim that is
/// no longer its own:
/// it looks at the claim's file just before it acts.
/// Before,
/// only a renewal that had found the claim gone stopped it,
/// and the renewal of a starved holder is the one that is late.
#[test]
fn a_claim_taken_over_when_no_renewal_has_run_is_not_run_from_the_old_claim() {
    let repo: Repo = repo("work-taken-over-late");
    let first: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    let second: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, "refs/heads/other");
    enqueue_all(&repo, &[first.clone(), second.clone()]);
    let lease: Duration = Duration::from_millis(300);
    let driver: DrivenRenewal = DrivenRenewal::default();
    let handlers: TakenOver = TakenOver { work_dir: work_directory(repo.common.as_path()), lease, pushed: RefCell::new(Vec::new()), renewal: None };
    drain_with_renewal(repo.common.as_path(), &handlers, &ClaimTime::read(), lease, Renewal::Driven(&driver));
    assert_eq!(*handlers.pushed.borrow(), vec![OID.to_string(), OID.to_string(), OTHER_OID.to_string()], "the second item was not pushed from its old claim");
    let mut done: Vec<String> = names(&repo, DONE);
    done.sort();
    let mut expected: Vec<String> = vec![first.id(), second.id()];
    expected.sort();
    assert_eq!(done, expected);
    remove(repo.root.as_path());
}

/// The credential a wake brought is held for that repository only,
/// in the copy of the handlers made for its round;
/// the handlers
///  the service keeps never hold one.
#[test]
fn the_review_key_is_held_per_repository_in_the_round_copy_only() {
    let handlers: GitHandlers = GitHandlers::new(crate::test_support::git_context());
    let first: PathBuf = PathBuf::from("/srv/one/.git");
    let second: PathBuf = PathBuf::from("/srv/two/.git");
    let key: ReviewKey = ReviewKey::new("test-key-not-a-secret").expect("a key");
    assert_eq!(handlers.review_key_of(first.as_path()), None);
    let round: GitHandlers = handlers.for_round(std::collections::HashMap::from([(first.clone(), key.clone())]));
    assert_eq!(round.review_key_of(first.as_path()), Some(key));
    assert_eq!(round.review_key_of(second.as_path()), None, "another repository does not see it");
    assert_eq!(handlers.review_key_of(first.as_path()), None, "the handlers the service keeps hold none");
}

/// A round keeps a wake's credential only for itself:
/// nothing of it is left in the service's handlers afterwards,
/// for the
/// repository that brought it or any other,
/// and a repeated directory is drained once.
#[test]
fn a_round_leaves_no_credential_in_the_handlers() {
    let first: Repo = repo("work-round");
    let second: Repo = repo("work-round-other");
    let handlers: GitHandlers = GitHandlers::new(crate::test_support::git_context());
    let key: ReviewKey = ReviewKey::new("test-key-not-a-secret").expect("a key");
    let wakes: Vec<Wake> = vec![
        (first.common.clone(), Some(key.clone())),
        (first.common.clone(), None),
        (second.common.clone(), Some(key)),
    ];
    run_round(wakes, &handlers, &mut PushRetries::with_first_pause(FIRST_PAUSE), &ClaimTime::read());
    assert_eq!(handlers.review_key_of(first.common.as_path()), None);
    assert_eq!(handlers.review_key_of(second.common.as_path()), None);
    remove(first.root.as_path());
    remove(second.root.as_path());
}

/// A review runs end to end through the real handlers with the context the service really has,
/// which names no
/// repository:
/// the work item is claimed,
/// the commit is read with real Git (the patch,
/// and the file texts through a
/// Git command that takes input),
/// the loopback gateway is asked with the key from the wake,
/// and the entry is appended.
/// The lane is what claims a review in the service,
/// beside the round that serves the pushes
/// (`service_work_slow_lane.rs`),
/// so the lane is what this control drives.
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
        repository: root.to_path_buf(),
        git_dir: common.to_path_buf(),
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
    let lane_handlers: GitHandlers = handlers.for_round(std::collections::HashMap::from([(common.clone(), ReviewKey::new(SECRET).expect("the key this control serves with"))]));
    let _left: Option<Duration> = crate::merge_second_pass::tests::await_it(crate::service_work_slow_lane::drain_the_slow_work_awaiting(
        common.as_path(),
        &lane_handlers,
        crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND,
    ));
    let seen: String = mock.finish().seen.to_ascii_lowercase();
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
    assert!(written.starts_with("\n- Date:\n") && written.contains("cost_verdict: no"), "{written}");
    assert!(!written.contains(SECRET));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("tracked log"), "# Log\n", "the tracked log is never written");
    remove(parent.as_path());
}

/// The `message` of the first captured entry that starts with `start`.
fn entry_starting<'a>(entries: &'a [crate::service_timing::capture::Captured], start: &str) -> &'a crate::service_timing::capture::Captured {
    return entries.iter().find(|entry| { return entry.message.starts_with(start); }).unwrap_or_else(|| { panic!("no entry starting {start:?} in {entries:?}") });
}

/// Every push the service runs leaves one entry with the branch,
/// the commit,
/// how many items it covered and how it ended,
/// at the priority that fits:
/// `info` for a push that went through,
/// `warning` for one that failed,
/// with Git's words after it.
#[test]
fn every_push_leaves_an_entry_that_names_the_branch_and_how_it_ended() {
    use crate::journald_native::Priority;
    let repo: Repo = repo("work-push-entry");
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH), item(&repo, WorkKind::Push, OTHER_OID, BRANCH)]);
    let done = crate::service_timing::capture::entries_of(|| { drain(repo.common.as_path(), &Recorder::answering(Verdict::Done), &ClaimTime::read()); });
    let pushed = entry_starting(done.as_slice(), "pushed ");
    assert_eq!(pushed.priority, Priority::Info);
    assert!(pushed.message.contains(format!("branch={BRANCH:?} oid={OTHER_OID} items=2 outcome=done push_ms=").as_str()), "{}", pushed.message);
    assert!(pushed.fields.contains(&(String::from("CLI_GIT_BRANCH"), String::from(BRANCH))) && pushed.fields.contains(&(String::from("CLI_GIT_OUTCOME"), String::from("done"))), "{:?}", pushed.fields);
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, THIRD_OID, BRANCH)]);
    let failed = crate::service_timing::capture::entries_of(|| {
        drain(repo.common.as_path(), &Recorder::answering(Verdict::Failed(String::from("remote: permission denied"))), &ClaimTime::read());
    });
    let entry = entry_starting(failed.as_slice(), "pushed ");
    assert_eq!(entry.priority, Priority::Warning);
    assert!(entry.message.contains("outcome=failed") && entry.message.contains(": remote: permission denied"), "{}", entry.message);
    remove(repo.root.as_path());
}

/// Upkeep that ran and a review that finished each leave an entry too,
/// so a quiet repository is told from a service that did
/// nothing.
#[test]
fn upkeep_and_review_that_ran_leave_entries() {
    let repo: Repo = repo("work-upkeep-entry");
    enqueue_all(&repo, &[item(&repo, WorkKind::Maintenance, OID, BRANCH), item(&repo, WorkKind::Review, OTHER_OID, BRANCH)]);
    let entries = crate::service_timing::capture::entries_of(|| { drain(repo.common.as_path(), &Recorder::answering(Verdict::Done), &ClaimTime::read()); });
    assert!(entry_starting(entries.as_slice(), "automatic maintenance ran maintenance_ms=").message.contains(" at_ms="));
    assert!(entry_starting(entries.as_slice(), format!("the review of {OTHER_OID} is done review_ms=").as_str()).message.contains(" at_ms="));
    remove(repo.root.as_path());
}

/// The production lease is two minutes and the holder renews every third of it,
/// so the margin for a starved service is two
/// intervals,
/// forty seconds each:
/// a renewal has to be more than eighty seconds late for a second instance to take a push
/// that is still running.
/// The figures are stated from the constants,
/// so a change of either shows here.
#[test]
fn the_production_lease_leaves_a_starved_holder_two_intervals_of_margin() {
    assert_eq!(CLAIM_LEASE, Duration::from_secs(120));
    assert_eq!(renewal_interval(CLAIM_LEASE), CLAIM_LEASE / RENEWALS_PER_LEASE);
    let margin: Duration = CLAIM_LEASE - renewal_interval(CLAIM_LEASE);
    assert_eq!(margin, Duration::from_secs(80));
    assert!(margin >= renewal_interval(CLAIM_LEASE) * 2, "a renewal late by two intervals still beats the lease");
}

/// Whether the last `core.lockfilePid` entry of a numbered `GIT_CONFIG_*` environment reads as true.
fn lock_pid_files_enabled(environment: &[(OsString, OsString)]) -> bool {
    let value_of = |name: &str| -> Option<String> {
        return environment.iter().rev().find(|(key, _)| { return key.to_str() == Some(name); }).map(|(_, value)| { return value.to_string_lossy().into_owned(); });
    };
    let count: usize = value_of("GIT_CONFIG_COUNT").and_then(|text| { return text.parse::<usize>().ok(); }).unwrap_or(0);
    let mut enabled: bool = false;
    for index in 0..count {
        if value_of(format!("GIT_CONFIG_KEY_{index}").as_str()).is_some_and(|key| { return key.eq_ignore_ascii_case("core.lockfilePid"); }) {
            enabled = value_of(format!("GIT_CONFIG_VALUE_{index}").as_str()).is_some_and(|value| { return value == "true"; });
        }
    }
    return enabled;
}

/// Every Git the service starts for a push writes the `~pid.lock` file beside each lock,
/// so a lock left by a stopped push
/// names its owner;
/// without it such a lock cannot be told from another process's.
#[test]
fn the_service_starts_its_push_git_with_lock_pid_files_on() {
    let repo: Repo = repo("service-push-lock-pid");
    let handlers: GitHandlers = GitHandlers::new(crate::test_support::git_context());
    let started: PushGit = handlers.push_git(&item(&repo, WorkKind::Push, OID, BRANCH));
    assert!(lock_pid_files_enabled(started.environment.as_slice()), "the push's Git environment turns on core.lockfilePid");
    remove(repo.root.as_path());
}

/// The listings a drain of `count` pushes of one branch costs,
/// with every item settled.
fn listings_of_a_drain(count: usize) -> usize {
    let repo: Repo = repo(format!("work-linear-{count}").as_str());
    let items: Vec<WorkItem> = (0..count).map(|n| { return item(&repo, WorkKind::Push, format!("{n:040x}").as_str(), BRANCH); }).collect();
    enqueue_all(&repo, items.as_slice());
    // The looks the producers themselves made are no part of the drain.
    let _producers_looks: usize = take_listings();
    drain(repo.common.as_path(), &Recorder::answering(Verdict::Done), &ClaimTime::read());
    assert_eq!(names(&repo, DONE).len(), items.len(), "the drain of {count} items settled every one of them");
    let listings: usize = take_listings();
    remove(repo.root.as_path());
    return listings;
}

/// A drain of many claims walks the work queue once per round and not once per claim (`RG2`):
/// the number of directory listings it costs does not grow with the number of claims it holds.
#[test]
fn a_drain_of_many_claims_lists_the_state_directories_once() {
    let measured: Vec<(usize, usize)> = [4, 24, 96].iter().map(|count| { return (*count, listings_of_a_drain(*count)); }).collect();
    let first: usize = measured[0].1;
    for (_count, listings) in &measured {
        assert_eq!(*listings, first, "the listings of a drain do not grow with its claims: {measured:?}");
    }
}

/// A push the service will try again says so only once the plan that keeps the promise is on disk:
/// a notice written before the plan is a promise a service that restarts cannot keep,
/// and a service killed before the loop applies the worker's changes leaves exactly that.
#[test]
fn the_retry_a_notice_promises_is_on_disk_before_the_notice_exists() {
    let repo: Repo = repo("work-retry-plan-first");
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    let plans: PathBuf = repo.root.join(crate::push_retry::PLAN_DIRECTORY);
    let recorder: Recorder = Recorder::answering(Verdict::Retry(String::from("the network is down")));
    // The worker's changes are never applied, as a kill before the loop applies them leaves them.
    let retrying: Retrying = Retrying { first: Some(FIRST_PAUSE), due: Vec::new(), plans: Some(plans.clone()) };
    let drained: Drained = drain_planned(repo.common.as_path(), &recorder, &ClaimTime::read(), &retrying);
    assert_eq!(drained.changes.len(), 1, "the worker asked for one plan");
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown.len(), 1, "the person is told about the push");
    assert!(shown[0].contains("tries this push again by itself"), "{shown:?}");
    // A service started in its place knows the branch by itself: the promise the notice made is the plan's.
    let restored: PushRetries = PushRetries::restored(FIRST_PAUSE, plans.as_path());
    assert!(restored.next_wait(repo.common.as_path(), Instant::now()).is_some(), "the retry the notice promised is on disk before the notice was written");
    remove(repo.root.as_path());
}

/// What:
///  A plan a round kept is still the file that is on disk after the round's changes are applied,
/// even though an
///  earlier change of the same round forgets that branch.
/// Why:
///   A worker keeps its branch's plan on disk at once,
/// and the loop applies the round's changes afterwards,
/// so
///  the forget of an earlier push of that branch would remove the very file the new plan was published as;
/// a service
///  killed between the two changes would then find no retry where the notice promised one.
/// The file is named here by
///  the inode it was published as:
/// a forget that removed it would leave the plan change to publish another one.
#[test]
fn a_plan_kept_after_an_older_forget_survives_its_replay() {
    use std::os::unix::fs::MetadataExt as _;
    let repo: Repo = repo("work-retry-plan-replayed-forget");
    let plans: PathBuf = repo.root.join(crate::push_retry::PLAN_DIRECTORY);
    let newest: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    // The round asks, in this order, for the branch to be forgotten (the earlier push landed) and for the next try to be planned;
    //  the plan reaches the disk at once, as the worker's keeps do.
    let mut replanning: Replanning = Replanning { first: Some(FIRST_PAUSE), changes: Vec::new(), plans: Some(plans.clone()) };
    replanning.forget(repo.common.as_path(), BRANCH);
    let _said: PlannedRetry = planned_retry(repo.common.as_path(), &mut replanning, FIRST_PAUSE, &newest, 1, "the network is down");
    let published: PathBuf = plans.join(crate::push_retry_store::name_of(&crate::push_retry_store::StoredPlan { git_dir: repo.common.clone(), item: newest.clone(), attempt: 1 }));
    let kept_inode: u64 = std::fs::metadata(published.as_path()).expect("the plan the notice promises is on disk before the loop applies anything").ino();
    let mut retries: PushRetries = PushRetries::restored(FIRST_PAUSE, plans.as_path());
    retries.apply(replanning.changes);
    let still: u64 = std::fs::metadata(published.as_path()).expect("the plan is still there after the replay").ino();
    assert_eq!(still, kept_inode, "the file the worker published was never taken away");
    // A service started in the place of one that stopped right after the forget finds it.
    let restored: PushRetries = PushRetries::restored(FIRST_PAUSE, plans.as_path());
    assert!(restored.next_wait(repo.common.as_path(), Instant::now()).is_some(), "the plan the worker kept is still there after the replay");
    remove(repo.root.as_path());
}

/// What:
///  A round that plans a retry,
/// forgets the branch and plans it again leaves the last plan after a kill at any
///  of its changes.
/// Why:
///   The workers publish their plans at once,
/// so replaying the first plan removes the file the second one wrote,
/// and the forget replayed after that finds nothing to keep;
/// a restarted service would then find no retry where the newer
///  push's notice promised one.
#[test]
fn a_plan_forgotten_and_planned_again_survives_a_kill_after_any_of_its_changes() {
    let repo: Repo = repo("work-retry-plan-forget-plan");
    let plans: PathBuf = repo.root.join(crate::push_retry::PLAN_DIRECTORY);
    let older: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    let newer: WorkItem = item(&repo, WorkKind::Push, OTHER_OID, BRANCH);
    let mut replanning: Replanning = Replanning { first: Some(FIRST_PAUSE), changes: Vec::new(), plans: Some(plans.clone()) };
    let _first: PlannedRetry = planned_retry(repo.common.as_path(), &mut replanning, FIRST_PAUSE, &older, 1, "the network is down");
    replanning.forget(repo.common.as_path(), BRANCH);
    let _second: PlannedRetry = planned_retry(repo.common.as_path(), &mut replanning, FIRST_PAUSE, &newer, 1, "the network is down");
    assert_eq!(replanning.changes.len(), 3, "a plan, a forget and a plan");
    let newest_name: String = crate::push_retry_store::name_of(&crate::push_retry_store::StoredPlan {
        git_dir: repo.common.clone(),
        item: newer.clone(),
        attempt: 1,
    });
    for stopped_after in 0..replanning.changes.len() {
        // The state the workers left: the plan they published,
        // and the earlier one their own saves took away.
        for entry in std::fs::read_dir(plans.as_path()).into_iter().flatten().flatten() {
            std::fs::remove_file(entry.path()).expect("the plans of the round before");
        }
        crate::push_retry_store::save(plans.as_path(), &crate::push_retry_store::StoredPlan {
            git_dir: repo.common.clone(),
            item: newer.clone(),
            attempt: 1,
        })
        .expect("the plan the newer push's worker published");
        let mut retries: PushRetries = PushRetries::restored(FIRST_PAUSE, plans.as_path());
        {
            // The service is killed after that many of the round's changes, as a kill at any instruction leaves it.
            let _stopped = crate::push_retry::stop_after_in_a_control(stopped_after);
            retries.apply(replanning.changes.clone());
        }
        let on_disk: Vec<crate::push_retry_store::StoredPlan> = crate::push_retry_store::load(plans.as_path());
        assert_eq!(
            on_disk.len(),
            1,
            "a kill after {stopped_after} of the round's changes leaves one plan: {on_disk:?}"
        );
        assert_eq!(
            crate::push_retry_store::name_of(&on_disk[0]),
            newest_name,
            "and it is the newer push's, as its notice said"
        );
    }
    remove(repo.root.as_path());
}

/// What:
///  A push whose retry plan cannot be kept is not told that the service keeps trying.
/// Why:
///   The plan is what keeps that promise across a restart;
/// with none on disk the words would outlive the service that
///  could have kept them,
/// and the person would wait for a retry nothing knows about.
/// The words are then Git's own alone,
/// and the
///  service that is running still tries,
/// whose journal says so.
#[test]
fn a_retry_the_plan_cannot_keep_is_not_promised() {
    let repo: Repo = repo("work-retry-plan-unwritable");
    enqueue_all(&repo, &[item(&repo, WorkKind::Push, OID, BRANCH)]);
    let plans: PathBuf = repo.root.join(crate::push_retry::PLAN_DIRECTORY);
    // A file where the plan directory must be makes saving any plan fail.
    std::fs::write(plans.as_path(), "in the way").expect("blocker");
    let recorder: Recorder = Recorder::answering(Verdict::Retry(String::from("the network is down")));
    let retrying: Retrying = Retrying { first: Some(FIRST_PAUSE), due: Vec::new(), plans: Some(plans.clone()) };
    let drained: Drained = drain_planned(repo.common.as_path(), &recorder, &ClaimTime::read(), &retrying);
    let shown: Vec<String> = take_push_failures(repo.common.as_path());
    assert_eq!(shown.len(), 1, "the person is told about the push");
    assert!(!shown[0].contains("tries this push again by itself"), "no promise the plan cannot keep: {shown:?}");
    assert!(shown[0].contains("the network is down"), "Git's own words are kept: {shown:?}");
    assert_eq!(drained.changes.len(), 1, "the service that is running goes on trying the push");
    remove(repo.root.as_path());
}

/// How long the control below lets the record of a refusal appear before it lets the plan's own file answer:
/// long enough for one job of the blocking pool to start and write a file on a contended host,
/// and short enough to keep the control quick.
const BEFORE_THE_PLANS_FILE_ANSWERS: Duration = Duration::from_secs(5);

/// What:
///  Open the pipe at `path` for writing and close it again,
/// which ends a read of it that waited for a writer.
/// Why:
///  A read the control's own code makes waits for a writer that never comes,
/// so the control releases it here rather than holding the suite for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// close(openSync(path, 'w')); // the read that waited for a writer ends
/// ```
fn release_the_plan_pipe(path: &Path) {
    let began: Instant = Instant::now();
    while began.elapsed() < BEFORE_THE_PLANS_FILE_ANSWERS {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(writer) => {
                drop(writer);
                return;
            }
            // No reader is in the pipe yet, so this is the moment before the read starts.
            Err(_no_reader) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// How many records of failed pushes a Git directory holds for the person.
fn notices_of(git_dir: &Path) -> usize {
    return std::fs::read_dir(notice_directory(git_dir)).map_or(0, |entries| { return entries.flatten().count(); });
}

/// What:
///  The record of a push the person must act on is written before the branch's plan file is taken away.
/// Why:
///  The notice an earlier round wrote says another try is coming,
/// and the plan's file is what makes that true.
/// A service killed between the removal of that file and the record of the refusal leaves the item in `failed`,
/// which nothing scans,
/// so the person keeps a promise nothing schedules instead of the refusal that needs them.
/// The control makes the removal wait: the branch's own plan file is a pipe nobody writes to,
/// and the process has one place of its blocking bound,
/// so a record that is written while the removal waits was written first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await save(plans, plan); mkfifo(planPath); const drained = drain(); expect(notices(gitDir)).toHaveLength(1);
/// ```
#[test]
fn the_record_of_a_terminal_failure_is_written_before_the_plans_file_goes() {
    crate::parallel_test_support::room_for("the record of a terminal failure goes before the plan's file", crate::parallel_test_support::ONE_PLACE);
    let repo: Repo = repo("work-terminal-record-before-the-plan-goes");
    let plans: PathBuf = repo.root.join(crate::push_retry::PLAN_DIRECTORY);
    let due: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    let waiting: crate::push_retry_store::StoredPlan =
        crate::push_retry_store::StoredPlan { git_dir: repo.common.clone(), item: due.clone(), attempt: 1 };
    crate::push_retry_store::save(plans.as_path(), &waiting).expect("the plan of the try that failed");
    // The plan's own file becomes a pipe with no writer, which is a record the removal cannot read.
    let reading: PathBuf = plans.join(crate::push_retry_store::name_of(&waiting));
    std::fs::remove_file(reading.as_path()).expect("the plan's file makes way for the pipe");
    let text: std::ffi::CString = std::ffi::CString::new(reading.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe stands where the plan's file was: {}", std::io::Error::last_os_error());
    let recorder: Recorder = Recorder::answering(Verdict::Failed(String::from("the server declined the push")));
    let retrying: Retrying = Retrying { first: Some(FIRST_PAUSE), due: vec![Due { git_dir: repo.common.clone(), item: due.clone(), attempt: 1 }], plans: Some(plans.clone()) };
    let recorded_first: bool = std::thread::scope(|scope| {
        let pipe: &Path = reading.as_path();
        let watcher = scope.spawn(|| {
            let seen: bool = crate::test_support::wait_until_for(crate::test_support::WaitingFor {
                window: BEFORE_THE_PLANS_FILE_ANSWERS,
                done: &|| { return notices_of(repo.common.as_path()) > 0; },
            });
            release_the_plan_pipe(pipe);
            return seen;
        });
        let drained: Drained = drain_planned(repo.common.as_path(), &recorder, &ClaimTime::read(), &retrying);
        assert_eq!(drained.changes.len(), 1, "the worker still asks the plan to drop the branch");
        return watcher.join().expect("the watcher ends");
    });
    assert!(
        recorded_first,
        "the refusal was recorded while the plan's file still waited for a writer, so the record goes first and a kill between the two leaves the plan"
    );
    assert!(reading.exists(), "the plan's file the removal could not read stays, so a later service tries the push the notice promised");
    remove(repo.root.as_path());
}

/// The names in one state directory of the work queue below `common`.
fn state_names(common: &Path, state: &str) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(state_directory(work_directory(common).as_path(), state)).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    found.sort();
    return found;
}

/// What:
///  The after-commit work of a repository whose path is not UTF-8 runs.
/// Why:
///   The wrapper writes a work item's directories into the record as text and the drain reads them back
/// to compare the Git directory with the one it drains,
/// so text that cannot hold the path's bytes makes every item of such a repository unacceptable,
/// and the push and the upkeep never run.
#[test]
fn the_after_commit_work_of_a_repository_whose_path_is_not_utf8_runs() {
    use crate::service_handoff::{HANDED_OVER, ServiceHandoff, hand_over};
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("work-non-utf8-path");
    // No encoding of "wörktree" and "cömmon" spells these two names: their bytes after 0x77 and 0x63 are not text.
    let repository: PathBuf = root.join(<std::ffi::OsStr as OsStrExt>::from_bytes(b"w\xffrktree"));
    let common: PathBuf = root.join(<std::ffi::OsStr as OsStrExt>::from_bytes(b"c\xffmmon"));
    std::fs::create_dir_all(repository.as_path()).expect("worktree");
    std::fs::create_dir_all(common.as_path()).expect("Git directory");
    let handoff: ServiceHandoff = ServiceHandoff {
        config: crate::service_client::ClientConfig {
            address: crate::service_server::Bind::Path(root.join("no-service.sock")),
            activation: crate::service_client::Activation::Unsupported(crate::service_messages::Platform::LinuxWithoutUserServices),
            start_wait: Duration::from_millis(100),
            heartbeat_silence: Duration::from_secs(1),
            start_watch: None,
            keep_fresh: None,
            answer_ceiling: crate::wait_ceilings::ANSWER_CEILING,
            withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING,
            extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT,
            start_ceiling: crate::wait_ceilings::START_CEILING,
            waiting_notice_after: Duration::from_secs(60),
            units_record_dir: None,
        },
        repository: repository.clone(),
        git_dir: common.clone(),
        reference: crate::ref_name::RefName::from_text(BRANCH),
        parent: String::new(),
    };
    let mut notices: Vec<u8> = Vec::new();
    // No service can be started here, so the wake fails; the records are written before it and are the subject.
    let _not_woken = hand_over(&handoff, OID, &mut notices);
    assert_eq!(state_names(common.as_path(), PENDING).len(), HANDED_OVER.len(), "the handoff left one record per kind");
    drain(common.as_path(), &Recorder::answering(Verdict::Done), &ClaimTime::read());
    assert!(state_names(common.as_path(), FAILED).is_empty(), "no work of this repository is refused: {:?}", state_names(common.as_path(), FAILED));
    assert_eq!(state_names(common.as_path(), DONE).len(), HANDED_OVER.len(), "the push and the upkeep ran");
    remove(root.as_path());
}

/// A repository with the tracked shadow log in place and one commit for a review to read,
/// with the `review` work item of that commit.
fn a_repository_to_review(parent: &Path, name: &str) -> (PathBuf, PathBuf, WorkItem) {
    use crate::standards_review_input::TRACKED_LOG_RELATIVE;
    use crate::test_support::{git, git_text, repository};
    let root: PathBuf = repository(parent, name);
    let log: PathBuf = root.join(TRACKED_LOG_RELATIVE);
    std::fs::create_dir_all(log.parent().expect("parent")).expect("directories");
    std::fs::write(log.as_path(), "# Log\n").expect("log");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/a.ts"), format!("export const a = '{name}'\n").as_str()).expect("code");
    git(root.as_path(), &["add", "--", TRACKED_LOG_RELATIVE, "src/a.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=feat(a): add a"]);
    let git_dir: PathBuf = root.join(".git");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Review,
        repository: root.clone(),
        git_dir: git_dir.clone(),
        branch_ref: BRANCH.to_string(),
        oid: git_text(root.as_path(), &["rev-parse", "HEAD"]),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    return (root, git_dir, item);
}

/// The handlers' review answers the same verdict whether a task or a plain thread asks it,
/// and each writes its own repository's entry.
/// The awaited face is the one a round of tasks may ask:
/// the synchronous one waits for its contained job on the
///  caller's thread.
#[test]
fn the_awaited_review_of_the_handlers_answers_what_the_synchronous_one_answers() {
    use crate::standards_review_input::shared_log_path;
    use crate::standards_review_mock::{Answer, Mock, http_reply};
    use crate::test_support::{git_context, repository as repository_at};
    const ANSWER: &str = r#"{"answers":[{"choice":"no","probabilities":[{"value":"no","probability":1}]}]}"#;
    let root: PathBuf = fixture("handlers-awaiting-review");
    std::fs::create_dir_all(root.as_path()).expect("the fixture's folder");
    let (thread_root, _thread_common, thread_item) = a_repository_to_review(root.as_path(), "thread");
    let (task_root, _task_common, task_item) = a_repository_to_review(root.as_path(), "task");
    let thread_mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    let task_mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    // The credential a wake brings is held for the round, as the service holds it: without one the review records that it
    //  had no key and asks nothing.
    let key = |item: &WorkItem| -> std::collections::HashMap<PathBuf, ReviewKey> { return std::collections::HashMap::from([(item.git_dir.clone(), ReviewKey::new("test-key-not-a-secret").expect("a key"))]); };
    let thread_handlers: GitHandlers = GitHandlers::new(git_context()).with_review_endpoint(thread_mock.url.as_str()).for_round(key(&thread_item));
    let task_handlers: GitHandlers = GitHandlers::new(git_context()).with_review_endpoint(task_mock.url.as_str()).for_round(key(&task_item));
    let from_a_thread: Verdict = thread_handlers.review(&thread_item);
    let from_a_task: Verdict = crate::merge_second_pass::tests::await_it(task_handlers.review_awaiting(&task_item));
    thread_mock.finish();
    task_mock.finish();
    assert_eq!(from_a_thread, Verdict::Done, "the review of a plain thread answers");
    assert_eq!(from_a_task, from_a_thread, "the two faces answer one review the same way");
    for (name, root_of) in [("thread", &thread_root), ("task", &task_root)] {
        let written: String = std::fs::read_to_string(shared_log_path(root_of.join(".git").as_path())).expect("the live log");
        assert!(written.contains("cost_verdict: no"), "the review of {name} wrote its entry: {written}");
    }
    // The positive control of the two faces: the review of a repository with no shadow log answers Done and reads nothing,
    //  whichever face asks it.
    let plain: PathBuf = repository_at(root.as_path(), "plain");
    let plain_item: WorkItem = WorkItem { kind: WorkKind::Review, repository: plain.clone(), git_dir: plain.join(".git"), ..task_item.clone() };
    let plain_handlers: GitHandlers = GitHandlers::new(git_context()).with_review_endpoint("http://127.0.0.1:9");
    assert_eq!(crate::merge_second_pass::tests::await_it(plain_handlers.review_awaiting(&plain_item)), Verdict::Done, "no shadow log means no review");
    assert_eq!(plain_handlers.review(&plain_item), Verdict::Done, "the synchronous face answers the same");
    remove(root.as_path());
}

/// What:
///  Every kind of work item a round of the service runs reaches its handler and is settled,
/// asked by a task of the runtime.
/// Why:
///   The drain of a round is a task,
/// and a kind it forgets is a person's backup,
/// a worktree's left-out files or a shadow record
///  that nothing ever runs.
/// A face that is synchronous where the drain is a task settles its items as failed at best:
/// the trash did,
///  because the synchronous disposal enters the runtime and is refused from inside it.
/// This control asks all four kinds at once,
///  requires each handler to be asked once,
/// and every item to be in `done`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPlannedWithAwaiting(dir, handlers, now, lease, TIMED, retrying); expect(asked).toEqual(['push','maintenance','review','trash']); expect(settled).toEqual('done')
/// ```
#[test]
fn the_awaited_drain_runs_every_kind_of_item() {
    let repo: Repo = repo("work-awaited-every-kind");
    let kinds: [WorkKind; 4] = [WorkKind::Push, WorkKind::Maintenance, WorkKind::Review, WorkKind::Trash];
    let items: Vec<WorkItem> = kinds.iter().enumerate().map(|(index, kind)| { return item(&repo, *kind, format!("{index:040x}").as_str(), BRANCH); }).collect();
    enqueue_all(&repo, items.as_slice());
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    // The drain of a round, awaited inside the runtime, which is where the lanes of a round run.
    let drained: Drained = crate::merge_second_pass::tests::await_it(drain_planned_with_awaiting(
        repo.common.as_path(),
        &recorder,
        &ClaimTime::read(),
        crate::service_work::CLAIM_LEASE,
        Renewal::Timed,
        &Retrying::none(),
        &SERVICE_KINDS,
    ));
    let mut asked: Vec<&str> = Vec::new();
    if !recorder.pushes.borrow().is_empty() {
        asked.push("push");
    }
    if !recorder.upkeeps.borrow().is_empty() {
        asked.push("maintenance");
    }
    if !recorder.reviews.borrow().is_empty() {
        asked.push("review");
    }
    if !recorder.trashes.borrow().is_empty() {
        asked.push("trash");
    }
    assert_eq!(asked, vec!["push", "maintenance", "review", "trash"], "every kind reached its handler");
    let settled: Vec<String> = names(&repo, DONE);
    let mut expected: Vec<String> = items.iter().map(|entry| { return entry.id(); }).collect();
    expected.sort();
    assert_eq!(settled, expected, "every item is settled as done, so none was dropped");
    assert_eq!(names(&repo, FAILED), Vec::<String>::new(), "no item was failed");
    assert_eq!(claimed_ids(&repo), Vec::<String>::new(), "no item is left claimed");
    // A push that went through forgets an earlier plan; nothing of this drain keeps a try for later.
    assert!(
        !drained.changes.iter().any(|change| { return matches!(change, crate::push_retry::RetryChange::Plan { .. }); }),
        "a drain that runs every kind plans no retry: {:?}",
        drained.changes
    );
    remove(repo.root.as_path());
}

/// The messages the code under test wrote on this thread while `body` ran,
/// in the order it wrote them.
fn journal_of(body: impl FnOnce()) -> Vec<String> {
    return crate::service_timing::capture::entries_of(body).into_iter().map(|entry| { return entry.message; }).collect();
}

/// What:
///  A round that has spent its share of housekeeping leaves the rest of it pending for a later round,
/// names them in one journal line,
/// and asks to be woken again,
/// while the first housekeeping item it found is started all the same.
/// Why:
///   A trash of one removed worktree is hundreds of thousands of removals,
/// and a service that logged 15 of them spent 1 h 37 min in one round without making the person's
///  push (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
/// A bound makes a round say "not this round" instead of starting another trash,
/// and the records it did not
///  start go back to `pending` so nothing is lost.
/// The first item starts because a look that could spend its whole slice on the claim and the refusal that come before the
///  housekeeping would otherwise leave every item for a later look that would spend its slice the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wait = await drainPending(dir, handlers, lease, renewal, plan, kinds, ZERO); expect(wait).toBe(LOOK_AGAIN); expect(names(DONE)).toHaveLength(1)
/// ```
#[test]
fn a_round_that_has_spent_its_housekeeping_leaves_the_rest_for_a_later_round() {
    let repo: Repo = repo("work-housekeeping-left");
    let items: Vec<WorkItem> = vec![item(&repo, WorkKind::Prewarm, OID, BRANCH), item(&repo, WorkKind::Prewarm, OTHER_OID, BRANCH)];
    enqueue_all(&repo, &items);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    let mut replanning: Replanning = Replanning { first: None, changes: Vec::new(), plans: None };
    // A round whose housekeeping share is already over starts none of it beyond the first item, which is the state a long round
    //  reaches.
    let mut looked_again: Option<Duration> = None;
    let said: Vec<String> = journal_of(|| {
        looked_again = crate::lazy_runtime::run_on_runtime(crate::service_work_drain_pending_claims::drain_pending_awaiting(
            repo.common.as_path(),
            &recorder,
            CLAIM_LEASE,
            Renewal::Timed,
            &mut replanning,
            &SERVICE_KINDS,
            Duration::ZERO,
        ))
        .expect("the process runtime runs the drain");
    });
    assert_eq!(looked_again, Some(crate::housekeeping_of_one_round::LOOK_AGAIN_AFTER_DEFERRING), "a round that left housekeeping for later asks to be woken again");
    assert_eq!(names(&repo, DONE).len(), 1, "the first housekeeping item of the round is started, whatever its slice had spent");
    assert!(names(&repo, FAILED).is_empty(), "the housekeeping the round did not start is not settled");
    assert_eq!(names(&repo, PENDING).len(), items.len() - 1, "the housekeeping the round did not start is pending again");
    assert!(names(&repo, CLAIMED).is_empty(), "nothing the round did not start stays claimed");
    let waiting: String = names(&repo, PENDING)[0].clone();
    let named: Vec<&String> = said.iter().filter(|line| { return line.contains("housekeeping items") && line.contains(waiting.as_str()); }).collect();
    assert_eq!(named.len(), 1, "one line names the housekeeping that waits for a later round: {said:?}");
    remove(repo.root.as_path());
}

/// What:
///  The positive control of the bound:
/// a round with its housekeeping share unspent runs the housekeeping it finds and leaves nothing behind.
/// Why:
///   The bound above must be what leaves work for later,
/// not the drain answering nothing at all:
/// with the shipped bound the same
///  records run and settle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wait = await drainPending(dir, handlers, lease, renewal, plan, HOUSEKEEPING_OF_ONE_ROUND); expect(names(DONE)).toHaveLength(1);
/// ```
#[test]
fn a_round_with_its_housekeeping_share_unspent_runs_what_it_finds() {
    let repo: Repo = repo("work-housekeeping-run");
    let items: Vec<WorkItem> = vec![item(&repo, WorkKind::Prewarm, OID, BRANCH)];
    enqueue_all(&repo, &items);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    let mut replanning: Replanning = Replanning { first: None, changes: Vec::new(), plans: None };
    let looked_again: Option<Duration> = crate::lazy_runtime::run_on_runtime(crate::service_work_drain_pending_claims::drain_pending_awaiting(
        repo.common.as_path(),
        &recorder,
        CLAIM_LEASE,
        Renewal::Timed,
        &mut replanning,
        &SERVICE_KINDS,
        crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND,
    ))
    .expect("the process runtime runs the drain");
    assert_eq!(looked_again, None, "a round that left nothing behind asks for no extra look");
    assert_eq!(names(&repo, DONE).len(), items.len(), "the housekeeping the round had room for ran");
    assert!(names(&repo, PENDING).is_empty() && names(&repo, CLAIMED).is_empty());
    remove(repo.root.as_path());
}

/// A removal records the path it renamed out of the person's way and returns,
/// so the drain is what deletes the subvolume;
/// the step runs it through the handler of its kind and settles it as done.
#[test]
fn the_awaited_drain_deletes_a_removed_worktrees_subvolume() {
    let repo: Repo = repo("work-awaited-deletes-subvolume");
    let deletion: String = String::from("/worktrees/cli-git-deleting-0123456789abcdef");
    let mut entry: WorkItem = item(&repo, WorkKind::SubvolumeDelete, OID, BRANCH);
    entry.parent = deletion.clone();
    enqueue_all(&repo, &[entry.clone()]);
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    let drained: Drained = crate::merge_second_pass::tests::await_it(drain_planned_with_awaiting(
        repo.common.as_path(),
        &recorder,
        &ClaimTime::read(),
        crate::service_work::CLAIM_LEASE,
        Renewal::Timed,
        &Retrying::none(),
        &SERVICE_KINDS,
    ));
    assert_eq!(recorder.deletions.borrow().as_slice(), [deletion.as_str()], "the step asks its handler for the path the record names");
    assert_eq!(names(&repo, DONE), vec![entry.id()], "the deletion is settled as done");
    assert_eq!(names(&repo, FAILED), Vec::<String>::new(), "no deletion is failed");
    assert_eq!(claimed_ids(&repo), Vec::<String>::new(), "no deletion is left claimed");
    assert!(
        !drained.changes.iter().any(|change| { return matches!(change, crate::push_retry::RetryChange::Plan { .. }); }),
        "a drain that deletes a subvolume plans no retry: {:?}",
        drained.changes
    );
    remove(repo.root.as_path());
}

/// What:
///  The snapshot ahead a drain claims runs the handler of its kind and is settled by its answer.
/// Why:
///   It is the one kind of work a person's own add waits for,
/// so a drain that claims the kind and forgets the step leaves the add waiting for a copy nothing makes
///  and the claim taken,
/// which no other control would see.
/// The slow lane claims it beside the pushes,
/// and a drain outside the service claims every kind
/// (`SLOW_KINDS` and `SERVICE_KINDS` in `service_work_handlers_replanning_and_acceptance.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPlannedWithAwaiting(dir, handlers, now, lease, TIMED, retrying, SERVICE_KINDS); expect(asked).toEqual([worktree]); expect(names(DONE)).toEqual([id])
/// ```
#[test]
fn the_awaited_drain_makes_a_snapshot_ahead() {
    let repo: Repo = repo("work-awaited-snapshot-ahead");
    let wanted: WorkItem = item(&repo, WorkKind::SnapshotAhead, OID, BRANCH);
    enqueue_all(&repo, std::slice::from_ref(&wanted));
    let recorder: Recorder = Recorder::answering(Verdict::Done);
    let drained: Drained = crate::merge_second_pass::tests::await_it(drain_planned_with_awaiting(
        repo.common.as_path(),
        &recorder,
        &ClaimTime::read(),
        crate::service_work::CLAIM_LEASE,
        Renewal::Timed,
        &Retrying::none(),
        &SERVICE_KINDS,
    ));
    assert_eq!(recorder.aheads.borrow().as_slice(), [repo.worktree.display().to_string()], "the step asks its handler for the worktree the record names");
    assert_eq!(names(&repo, DONE), vec![wanted.id()], "the snapshot ahead is settled as done");
    assert_eq!(names(&repo, FAILED), Vec::<String>::new(), "no snapshot ahead is failed");
    assert_eq!(claimed_ids(&repo), Vec::<String>::new(), "no snapshot ahead is left claimed");
    assert!(
        !drained.changes.iter().any(|change| { return matches!(change, crate::push_retry::RetryChange::Plan { .. }); }),
        "a drain that makes a snapshot ahead plans no retry: {:?}",
        drained.changes
    );
    remove(repo.root.as_path());
}

/// What:
///  The worker that decided a push needs no new try takes its plan file out of the journal at once,
/// so no round has to replay that forget onto the disk.
/// Why:
///  Finding 3 of the review of `bc3d54f9cd`:
/// the push of a due plan landed and the worker asked the loop to forget the branch,
/// the job that applied the round's changes was given no thread,
/// and the caller then applied the changes without writing:
/// the list forgot the branch,
/// the file stayed,
/// and the next service tried a push that had already landed,
/// with its failure records and a reconcile of a remote that may have moved since.
/// The file goes on the thread that decided it,
/// as the plan a worker promises is already written on that thread (`keep_the_plan`),
/// so the forget reaches the journal before the notice that says the push landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await save(dir, plan); replanning.forget(gitDir, branch); expect(await load(dir)).toEqual([]);
/// ```
#[test]
fn the_worker_that_forgets_a_branch_removes_its_plan_file() {
    let repo: Repo = repo("work-retry-forget-takes-the-file");
    let plans: PathBuf = repo.root.join(crate::push_retry::PLAN_DIRECTORY);
    let landed: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    crate::push_retry_store::save(plans.as_path(), &crate::push_retry_store::StoredPlan { git_dir: repo.common.clone(), item: landed.clone(), attempt: 3 })
        .expect("the due plan the round took");
    let mut replanning: Replanning = Replanning { first: Some(FIRST_PAUSE), changes: Vec::new(), plans: Some(plans.clone()) };
    replanning.forget(repo.common.as_path(), BRANCH);
    assert_eq!(replanning.changes.len(), 1, "the loop still drops the branch from the list it owns");
    assert!(
        crate::push_retry_store::load(plans.as_path()).is_empty(),
        "the plan of the push that landed is out of the journal, so no later service tries it again"
    );
    remove(repo.root.as_path());
}

/// What:
///  The forget that rests on the proof of a gone repository takes the branch's plan file.
/// Why:
///  The file goes on the thread that decided it,
/// and the proof was a reading of the filesystem a moment before,
/// so this is the last place the directory can be asked before the file is gone:
/// a repository that is still missing keeps its promise as no try,
/// and its file goes with the decision.
/// A file left behind is a push a later start tries again for a repository that cannot answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await save(dir, plan); replanning.forgetWhenUnreachable(goneDir, due); expect(await load(dir)).toEqual([]);
/// ```
#[test]
fn a_gone_repository_takes_the_file_of_its_plan() {
    let gone_repository: Repo = repo("work-retry-forget-when-unreachable");
    let plans: PathBuf = gone_repository.root.join(crate::push_retry::PLAN_DIRECTORY);
    // A Git directory the fixture never made, which is what the proof of a deleted repository leaves.
    let gone: PathBuf = gone_repository.root.join("gone").join(".git");
    let due: WorkItem = item(&gone_repository, WorkKind::Push, OID, BRANCH);
    crate::push_retry_store::save(plans.as_path(), &crate::push_retry_store::StoredPlan { git_dir: gone.clone(), item: due.clone(), attempt: 3 })
        .expect("the plan the proof was asked about");
    let mut replanning: Replanning = Replanning { first: Some(FIRST_PAUSE), changes: Vec::new(), plans: Some(plans.clone()) };
    replanning.forget_when_unreachable(gone.as_path(), &Due { git_dir: gone.clone(), item: due.clone(), attempt: 3 });
    assert!(
        crate::push_retry_store::load(plans.as_path()).is_empty(),
        "a repository the filesystem still proves gone ends its plan's file, so no later service tries it again"
    );
    assert!(
        matches!(&replanning.changes[0], crate::push_retry::RetryChange::Forget { only_while_unreachable: true, .. }),
        "the change names the proof it rests on, so the loop asks the filesystem once more: {:?}",
        replanning.changes
    );
    remove(gone_repository.root.as_path());
}

/// What:
///  The forget that rests on the proof of a gone repository keeps the file of a branch whose directory is back.
/// Why:
///  The proof is a reading of the filesystem,
/// and this is the last place the file can be asked about before it goes:
/// a repository renamed back by then is reachable,
///  and its plan is the only scheduled backup of its commits.
/// The change still carries the proof it rests on,
/// so the loop asks the filesystem once more before it drops the branch from the list it owns,
/// and it carries the plan that reading ended,
/// so the list keeps the branch when the directory is there again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await save(dir, plan); replanning.forgetWhenUnreachable(thereDir, due); expect(load(dir)).toHaveLength(1);
/// ```
#[test]
fn a_repository_that_is_back_keeps_the_file_of_its_plan() {
    let back_repository: Repo = repo("work-retry-forget-when-reachable");
    let plans: PathBuf = back_repository.root.join(crate::push_retry::PLAN_DIRECTORY);
    let back: WorkItem = item(&back_repository, WorkKind::Push, OID, BRANCH);
    crate::push_retry_store::save(plans.as_path(), &crate::push_retry_store::StoredPlan { git_dir: back_repository.common.clone(), item: back.clone(), attempt: 3 })
        .expect("the plan the proof was asked about");
    let mut replanning: Replanning = Replanning { first: Some(FIRST_PAUSE), changes: Vec::new(), plans: Some(plans.clone()) };
    // The repository is there, so the stale proof ends nothing.
    replanning.forget_when_unreachable(back_repository.common.as_path(), &Due { git_dir: back_repository.common.clone(), item: back.clone(), attempt: 3 });
    assert_eq!(
        crate::push_retry_store::load(plans.as_path()).len(),
        1,
        "the file of a repository that is back stays, so the next start tries the push the service still holds"
    );
    assert!(
        matches!(&replanning.changes[0], crate::push_retry::RetryChange::Forget { only_while_unreachable: true, .. }),
        "the change names the proof it rests on, so the loop asks the filesystem once more: {:?}",
        replanning.changes
    );
    remove(back_repository.root.as_path());
}

/// What:
///  A retryable answer leaves the record where it is:
/// the round settles nothing for it,
/// so it keeps its lease,
///  and the lease running out is what queues it again with one more try.
/// Why:
///  A copy of the source that cannot be deleted,
/// and a snapshot ahead the service could not make,
/// both answer
///  `Retry` for a condition that can pass:
/// a busy volume,
/// a full disk,
/// an I/O error.
/// Failing the record drops that work for good,
/// and a
///  record in `failed` is never queued again under its id,
/// so a copy that proved stale then stays on the disk for ever,
///  pinning the source's blocks.
///  The recovery is asked twice,
/// because the rule has two sides and only the pair states it:
/// a claim whose lease has not run out is nobody's to take back,
/// and the same claim one lease later is.
/// Both readings are explicit,
/// never "now" with a lease of nothing:
/// a claim's age is measured in whole milliseconds,
/// so a reading taken a moment after the claim and a lease of nothing are one instant,
/// and that pair says nothing about either side of the rule.
#[test]
fn a_retryable_answer_keeps_the_record_for_a_later_round() {
    for kind in [WorkKind::SubvolumeDelete, WorkKind::SnapshotAhead] {
        let repo: Repo = repo(format!("work-retryable-{}", kind.name()).as_str());
        let wanted: WorkItem = item(&repo, kind, OID, BRANCH);
        enqueue_all(&repo, std::slice::from_ref(&wanted));
        let retrying: Recorder = Recorder::answering(Verdict::Retry(String::from("the volume is busy")));
        // The renewal is driven by this control and driven not at all, so no thread of the round is renaming the claim
        //  the pass below takes: the take of a name a renewal moved is the one race the recovery leaves to the next pass.
        let driver: DrivenRenewal = DrivenRenewal::default();
        let wait: Option<Duration> = drain_with_renewal(repo.common.as_path(), &retrying, &ClaimTime::read(), crate::service_work::CLAIM_LEASE, Renewal::Driven(&driver));
        assert_eq!(claimed_ids(&repo), vec![wanted.id()], "{}: the record keeps its claim", kind.name());
        assert_eq!(names(&repo, FAILED), Vec::<String>::new(), "{}: a retryable answer is no failure", kind.name());
        assert!(wait.is_some(), "{}: the round asks to be woken when that claim runs out", kind.name());
        // A reading taken now is a lease short of the claim, so the claim is still the round's and nothing takes it back.
        crate::work_item_recovery::requeue_expired(
            work_directory(repo.common.as_path()).as_path(),
            &ClaimTime::read(),
            crate::service_work::CLAIM_LEASE,
            crate::service_work_handlers_replanning_and_acceptance::MAX_TRIES,
        );
        assert_eq!(claimed_ids(&repo), vec![wanted.id()], "{}: a claim whose lease has not run out is left alone", kind.name());
        // The lease has run out, which is what the wait above asked for: the record is queued again with one more try.
        // The reading is two leases after the claim, not one, because a claim at exactly its lease still holds its record
        //  (`work_item_tests.rs`, "a lease at its limit still holds"), and a claim's age is measured in whole milliseconds:
        //  one lease after the claim is the same instant when nothing measurable has passed, which is how a reading and a
        //  lease of nothing made this control pass or fail with the host's speed (release gate on `fc9f7f5f4b`, 2026-11-11).
        crate::work_item_recovery::requeue_expired(
            work_directory(repo.common.as_path()).as_path(),
            &ClaimTime::read().advanced(crate::service_work::CLAIM_LEASE.saturating_mul(2)),
            crate::service_work::CLAIM_LEASE,
            crate::service_work_handlers_replanning_and_acceptance::MAX_TRIES,
        );
        assert_eq!(names(&repo, PENDING), vec![wanted.id()], "{}: the record waits for the next round", kind.name());
        drain(repo.common.as_path(), &Recorder::answering(Verdict::Done), &ClaimTime::read());
        assert_eq!(names(&repo, DONE), vec![wanted.id()], "{}: the round after it finishes the work", kind.name());
        remove(repo.root.as_path());
    }
}

/// What:
///  The line a planned push retry writes names the repository the push is of.
/// Why:
///  One journal covers every worktree of the machine,
///  and a branch name is the same in all of them:
///  `fix/main`,
///  `main`,
///  so a retry line that names only the branch and the commit cannot say which worktree's push it plans,
///  and the person who reads it after a jam cannot find the commit it speaks of
///  (a Kimi K3.1 Pro review of this branch,
///  its finding that scheduled push-retry lines still omit the repository).
#[test]
fn the_line_a_planned_retry_writes_names_its_repository() {
    let repo: Repo = repo("work-retry-names-its-repository");
    let newest: WorkItem = item(&repo, WorkKind::Push, OID, BRANCH);
    let mut replanning: Replanning = Replanning { first: Some(FIRST_PAUSE), changes: Vec::new(), plans: None };
    let entries: Vec<crate::service_timing::capture::Captured> = crate::service_timing::capture::entries_of(|| {
        let _said: PlannedRetry = planned_retry(repo.common.as_path(), &mut replanning, FIRST_PAUSE, &newest, 1, "the network is down");
    });
    let said: Vec<String> = entries.iter().map(|entry| { return entry.message.clone(); }).collect();
    let line: &String = said.iter().find(|line| { return line.contains("push will be tried again"); }).unwrap_or_else(|| panic!("the retry is said: {said:?}"));
    assert!(
        line.contains(&format!("repository={:?}", repo.worktree)),
        "the line names the worktree whose push it plans, quoted so a path with a space stays one field: {line}"
    );
    assert!(entries.iter().any(|entry| { return entry.fields.iter().any(|(name, value)| { return name == "CLI_GIT_REPOSITORY" && *value == repo.worktree.display().to_string(); }); }), "the journal field names it too: {entries:?}");
    remove(repo.root.as_path());
}
