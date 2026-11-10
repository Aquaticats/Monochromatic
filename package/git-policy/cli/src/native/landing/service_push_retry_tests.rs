//! Controls for the service's own retry of a push that was stopped or failed:
//! a local bare remote whose `pre-receive` hook
//! stalls the first push and accepts the next,
//! real Git pushing,
//! and the service's work loop driven by hand.
//! Why:
//!  At 05:40 on 2026-11-03 a stopped auto-push of `main` was never tried again until a person pushed by hand 15 minutes
//!  later,
//! which also blocked a release (the owner's rule,
//! `SPEC.md` "Merges and commits never fail":
//! auto-push must
//!  land).
//! The retry needs no new commit:
//! the work loop looks at the repository again when the pause is over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const wait = await round(gitDir, retries); await sleep(wait); await round(gitDir, retries); expect(remoteHas(commit)).toBe(true);
//! ```
#![cfg(unix)]

use crate::work_item::WorkKind;
use crate::push_retry::{Due, PushRetries, RetryChange};
use crate::parallel_budget::HostResources;
use crate::service_work_run_round::run_planned_round;
use super::*;
use crate::push_failure_notice::notice_directory;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository, wait_until};
use crate::work_item::enqueue;
use std::os::unix::fs::PermissionsExt;
use crate::push_retry::PLAN_DIRECTORY;
use std::time::Instant;
use crate::service_claim_watch::{ClaimWatch, WATCH_DIRECTORY};
use crate::service_retry::RETRY_SLACK;
use crate::service_server::work_loop;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// One round over a single directory,
/// the way the service's loop runs it:
/// what is due comes out of `retries`,
/// the worker's changes
/// go back into it,
/// and the wait is when to look at the directory again.
fn round_of<H: WorkHandlers + Sync>(git_dir: &Path, handlers: &H, retries: &mut PushRetries) -> Option<Duration> {
    let asked: Vec<(PathBuf, Duration)> = run_planned_round(&HostResources::read(), &[git_dir.to_path_buf()], handlers, retries, &ClaimTime::read());
    return asked.into_iter().find(|(directory, _)| { return directory == git_dir; }).map(|(_, wait)| { return wait; });
}

/// The stall window of the controls as a number.
const STALL_SECONDS_NUMBER: u64 = 2;

/// The stall window of the controls,
/// short enough to wait for and longer than a healthy local push.
const STALL_SECONDS: &str = "2";

/// The first pause before a push is tried again,
/// in the controls.
const PAUSE: Duration = Duration::from_millis(800);

/// A repository tracking a bare remote whose hook stalls the first push it sees after the setup.
struct Stage {
    /// The fixture directory,
    /// removed at the end.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// The bare remote.
    remote: PathBuf,
    /// The commit to deliver.
    landed: String,
}

/// Build the stage with a `pre-receive` hook that stalls the first push,
/// the way a remote that checks the pack before writing anything behaves.
fn stage(name: &str) -> Stage {
    return stage_with(name, "pre-receive");
}

/// Build the stage:
/// an initial commit pushed with no hook,
/// the hook called `hook_name` installed,
/// a new commit landed,
/// its work item written.
fn stage_with(name: &str, hook_name: &str) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let marker: PathBuf = root.join("stalled-once");
    let hook: PathBuf = remote.join("hooks").join(hook_name);
    std::fs::write(hook.as_path(), format!("#!/bin/sh\nif [ ! -e '{}' ]; then : > '{}'; exec sleep 600; fi\nexit 0\n", marker.display(), marker.display())).expect("hook");
    std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("hook is executable");
    std::fs::write(repo.join("a.txt"), b"two\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let work_item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.to_path_buf(),
        git_dir: repo.join(".git").to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(repo.join(".git").as_path()).as_path(), &work_item).expect("enqueue");
    let staged: Stage = Stage { root, repo, remote, landed };
    // The reading of the remote's ref file is the one every control waits on, so it is shown to see a push this stage made.
    assert_eq!(remote_tip(&staged), first, "the control's reading of the remote's ref shows the push the stage made");
    return staged;
}

/// What:
///  The branch tip the remote holds,
///  read from the bare remote's own ref file.
/// Why:
///   A control waits for the push by asking this every `WAIT_LOOK_EVERY`,
/// and starting Git for it costs the contended disk a process a second,
/// which is work that keeps a wait for the push alive and delays the very push it watches.
/// The stage's own first push,
/// made by real Git before any control runs,
/// is the positive control that this reading sees a push.
fn remote_tip(stage: &Stage) -> String {
    return std::fs::read_to_string(stage.remote.join("refs").join("heads").join("main")).map(|text| { return text.trim().to_owned(); }).unwrap_or_default();
}

/// How many records of failed pushes the repository holds for the person.
fn notices(stage: &Stage) -> usize {
    return std::fs::read_dir(notice_directory(stage.repo.join(".git").as_path())).map_or(0, |entries| { return entries.flatten().count(); });
}

/// A push the service stopped is tried again by the service alone,
/// after a pause,
/// with no new commit:
/// the first drain stops
///  the stalled push and says when it will try again,
/// the second (the work loop's wake after the pause) delivers,
/// and the
///  person's failure record is gone.
#[test]
fn a_stopped_push_is_tried_again_without_a_new_commit_and_lands() {
    let stage: Stage = stage("push-retry-stall-once");
    let git_dir: PathBuf = stage.repo.join(".git");
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&[("CLI_GIT_PUSH_DEADLINE_SECONDS", STALL_SECONDS)]);
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let first: Option<Duration> = round_of(git_dir.as_path(), &handlers, &mut retries);
    assert_ne!(remote_tip(&stage), stage.landed, "the first push stalled and was stopped");
    assert_eq!(notices(&stage), 1, "the person is told the push failed");
    let wait: Duration = first.expect("the service says when to look at the repository again");
    assert!(wait <= PAUSE + Duration::from_secs(5), "the wake is for the retry: {wait:?}");
    std::thread::sleep(wait);
    let second: Option<Duration> = round_of(git_dir.as_path(), &handlers, &mut retries);
    assert_eq!(remote_tip(&stage), stage.landed, "the retry delivered the commit");
    assert_eq!(notices(&stage), 0, "the record of the failure is gone once the push landed");
    assert_eq!(second, None, "nothing is left to try again");
    remove(stage.root.as_path());
}

/// Handlers whose pushes answer from a script,
/// one verdict per push.
struct Scripted {
    /// The verdicts still to give,
    /// the next last.
    /// A test double:
    /// the round's workers share the handlers,
    /// and a script is read in order.
    verdicts: Mutex<Vec<Verdict>>,
    /// How many pushes ran.
    pushes: AtomicUsize,
}

impl WorkHandlers for Scripted {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        self.pushes.fetch_add(1, Ordering::SeqCst);
        return self.verdicts.lock().expect("the script").pop().unwrap_or(Verdict::Done);
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// The first pause of the scripted controls.
const SCRIPTED_PAUSE: Duration = Duration::from_millis(300);

/// A directory standing for a repository,
/// with a push item written for it.
fn scripted_world(name: &str, verdicts: Vec<Verdict>) -> (PathBuf, PathBuf, Scripted, PushRetries) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir_all(&git_dir).expect("common directory");
    std::fs::create_dir_all(&worktree).expect("worktree directory");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: worktree.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(git_dir.as_path()).as_path(), &item).expect("enqueue");
    let handlers: Scripted = Scripted { verdicts: Mutex::new(verdicts), pushes: AtomicUsize::new(0) };
    return (root, git_dir, handlers, PushRetries::with_first_pause(SCRIPTED_PAUSE));
}

/// Every failure that can pass is tried again,
/// each time after a longer pause,
/// until the push lands;
/// the record of the failure
/// says the service keeps trying,
/// and goes when the push lands.
#[test]
fn a_push_that_keeps_failing_is_tried_again_with_longer_pauses_until_it_lands() {
    // The script is read from the end: two failures, then success.
    let (root, git_dir, handlers, mut retries): (PathBuf, PathBuf, Scripted, PushRetries) =
        scripted_world("push-retry-grows", vec![Verdict::Done, Verdict::Retry(String::from("Could not resolve host")), Verdict::Retry(String::from("Could not resolve host"))]);
    let first: Duration = round_of(git_dir.as_path(), &handlers, &mut retries).expect("a retry is planned");
    // That each pause is twice the one before is controlled in `push_retry_tests.rs`. Here only the pause's own figure is asked
    //  for, as a lower bound: what a round returns also holds the time the round took, which a loaded host stretches (measured
    //  2026-11-09: a round that outlasted its own pause returned nothing to wait, and the growth the control asked for read
    //  "the pause grew: 298.945694ms then 0ns").
    assert!(first <= SCRIPTED_PAUSE + RETRY_SLACK, "the plan asks for no more than its pause and the slack: {first:?}");
    let record: String = std::fs::read_dir(notice_directory(git_dir.as_path())).expect("the notices").flatten().map(|entry| { return std::fs::read_to_string(entry.path()).expect("notice"); }).collect();
    assert!(record.contains("tries this push again by itself") && record.contains("Could not resolve host"), "{record}");
    let planned: std::cell::RefCell<PushRetries> = std::cell::RefCell::new(retries);
    assert!(
        wait_until(&|| {
            // Each look runs a round, which runs the retry once its pause is over; the plan is gone once the push landed.
            let mut held = planned.borrow_mut();
            let _ran = round_of(git_dir.as_path(), &handlers, &mut held);
            return held.directories().is_empty();
        }),
        "nothing is left to try once every pause the plan asked for has passed"
    );
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 3);
    assert_eq!(std::fs::read_dir(notice_directory(git_dir.as_path())).map_or(0, |entries| { return entries.flatten().count(); }), 0, "the record went when the push landed");
    remove(root.as_path());
}

/// A conflict is final:
/// the push is not tried again,
/// and the person's record stays.
#[test]
fn a_conflict_is_not_tried_again() {
    let (root, git_dir, handlers, mut retries): (PathBuf, PathBuf, Scripted, PushRetries) =
        scripted_world("push-retry-conflict", vec![Verdict::Failed(String::from("merging origin/main into main conflicts in: a.txt")), Verdict::Retry(String::from("stalled"))]);
    let first: Duration = round_of(git_dir.as_path(), &handlers, &mut retries).expect("a retry is planned");
    std::thread::sleep(first);
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None, "a conflict plans no retry");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 2);
    let record: String = std::fs::read_dir(notice_directory(git_dir.as_path())).expect("the notices").flatten().map(|entry| { return std::fs::read_to_string(entry.path()).expect("notice"); }).collect();
    assert!(record.contains("conflicts in: a.txt") && !record.contains("tries this push again"), "{record}");
    remove(root.as_path());
}

/// A new commit's push replaces the plan of its branch:
/// a push that lands needs no retry.
#[test]
fn a_push_that_lands_cancels_the_plan_of_its_branch() {
    let (root, git_dir, handlers, mut retries): (PathBuf, PathBuf, Scripted, PushRetries) = scripted_world("push-retry-cancelled", vec![Verdict::Retry(String::from("stalled"))]);
    assert!(round_of(git_dir.as_path(), &handlers, &mut retries).is_some());
    retries.forget(git_dir.as_path(), "refs/heads/main");
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None);
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "a forgotten plan is not pushed");
    remove(root.as_path());
}

/// The longest a push the service made took while the whole suite ran at once,
/// measured on 2026-11-09:
/// `git push` took 23.1 s while `git-receive-pack` sat in uninterruptible disk I/O,
/// and the push's own wait loop looked at its child every 10 ms with no gap over 44 ms,
/// so nothing in the wait was starved and the child itself was slow.
/// The journal of one of these controls kept `push_ms=23058` in a run of 85 of them,
/// against 1.2 to 3 s for the same push alone.
/// Main answered that figure with a patience above it,
/// and that patience is kept here as the figure a control's wait has to cover;
/// the wait itself is
///  this branch's (`test_support::STILL_WORKING_WINDOW`),
/// so the guarantee main chose survives the mechanism that replaced it.
const LONGEST_CONTENDED_PUSH: Duration = Duration::from_millis(23_100);

/// Half of the longest contended push,
/// so that the patience allows that figure and half again.
const HALF_A_CONTENDED_PUSH: Duration = Duration::from_millis(11_550);

/// How long a control's wait allowed for a service to deliver while this was a wall clock:
/// the longest contended push and half of it again,
/// its first pause,
/// and the slack the schedule adds.
///  Kept as a figure,
/// not as a bound:
/// a wait that ended on it failed while the push went on to land.
const RESTART_PATIENCE: Duration = LONGEST_CONTENDED_PUSH
    .saturating_add(HALF_A_CONTENDED_PUSH)
    .saturating_add(PAUSE)
    .saturating_add(RETRY_SLACK);

/// What:
///  The window a control's wait allows is above the longest push measured while the whole suite ran,
///  and above the patience main chose for it,
///  so a delivery that was waited for under that patience is waited for here,
///  and a lost plan is still found out inside the window.
/// Why:
///   A push held by the host is waited for while the run works,
/// so the window has to be the longest such push or longer;
///  a plan that was lost shows nothing and ends the wait at the window.
/// Main measured no gap over 44 ms in a push's own wait loop while `git-receive-pack` sat in
///  uninterruptible I/O,
/// so a window of the run's silence covers the delivery that patience covered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(STILL_WORKING_WINDOW).toBeGreaterThan(RESTART_PATIENCE);
/// ```
#[test]
fn the_controls_window_covers_the_longest_push_measured_under_the_suite() {
    assert!(
        crate::test_support::STILL_WORKING_WINDOW > LONGEST_CONTENDED_PUSH,
        "{:?} is not above the {LONGEST_CONTENDED_PUSH:?} a contended push measured",
        crate::test_support::STILL_WORKING_WINDOW
    );
    assert!(
        crate::test_support::STILL_WORKING_WINDOW > RESTART_PATIENCE,
        "{:?} is not above the {RESTART_PATIENCE:?} main allowed a delivery",
        crate::test_support::STILL_WORKING_WINDOW
    );
}

/// Run `work_loop` on a thread,
/// as the service does,
/// and give the way to wake it;
/// dropping the sender stops it.
fn start_service<'scope>(scope: &'scope std::thread::Scope<'scope, '_>, handlers: &'scope GitHandlers, plans: &'scope Path, busy: &'scope AtomicUsize) -> (UnboundedSender<Wake>, std::thread::ScopedJoinHandle<'scope, ()>) {
    let (sender, mut receiver) = unbounded_channel::<Wake>();
    let running = scope.spawn(move || { work_loop(&mut receiver, handlers, busy, PushRetries::restored(PAUSE, plans), ClaimWatch::in_memory()); });
    return (sender, running);
}

/// A service that is stopped while a push waits for a new try does not take the plan with it:
/// the service started in its place
///  delivers the commit by itself,
/// with no new commit and no wake from a wrapper command.
/// The plan is not the person's failure
///  record,
/// which the next wrapper command shows and removes.
/// The wait is for the delivery,
/// never for a number of seconds:
/// measured on 2026-11-09 while the host ran at a load average of 70 to 100,
/// the restarted service delivered the commit
///  2.2 s,
/// 10.7 s and 12.3 s after it started,
/// and every round of its work is real Git whose starts took 0.9 to 2.8 s
/// there.
/// A control that allowed a stated number of seconds failed at whichever delivery was slower than it,
/// though the service
///  retried every time.
#[test]
fn a_stopped_push_is_still_tried_again_after_the_service_restarts() {
    assert_eq!(STALL_SECONDS, STALL_SECONDS_NUMBER.to_string(), "the two spellings of the stall window agree");
    let stage: Stage = stage("push-retry-restart");
    let git_dir: PathBuf = stage.repo.join(".git");
    let plans: PathBuf = stage.root.join("journal").join(PLAN_DIRECTORY);
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&[("CLI_GIT_PUSH_DEADLINE_SECONDS", STALL_SECONDS)]);
    let commits_before: String = git_text(stage.repo.as_path(), &["rev-list", "--count", "--all"]);
    let busy: AtomicUsize = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let (wake, first) = start_service(scope, &handlers, plans.as_path(), &busy);
        wake.send((git_dir.clone(), None)).expect("the first service takes the wake");
        assert!(wait_until(&|| { return notices(&stage) == 1; }), "the first service stopped the stalled push and told the person");
        // The service is stopped while its plan is pending.
        drop(wake);
        first.join().expect("the first service ends");
        assert_ne!(remote_tip(&stage), stage.landed, "nothing was delivered before the restart");
        // The person's next command shows and removes the failure record, as it does in a day of work.
        let _shown = crate::test_support::shown_notices(crate::push_failure_notice::push_failures(git_dir.as_path()));
        let (nobody, second) = start_service(scope, &handlers, plans.as_path(), &busy);
        assert!(wait_until(&|| { return remote_tip(&stage) == stage.landed; }), "the restarted service delivered the commit by itself");
        drop(nobody);
        second.join().expect("the second service ends");
    });
    assert_eq!(notices(&stage), 0, "the record of the failure is gone once the push landed");
    assert_eq!(git_text(stage.repo.as_path(), &["rev-list", "--count", "--all"]), commits_before, "no commit was made");
    remove(stage.root.as_path());
}

/// How long the control of the wait holds its work:
/// five stall windows and two seconds more,
/// which is longer than the wait
///  used to allow,
/// and no Git at all,
/// so only the wait's own figure is measured.
const HELD_PUSH: Duration = Duration::from_secs(STALL_SECONDS_NUMBER * 5 + 2);

/// What:
///  The wait for work the service does holds while that work is unfinished for longer than five stall windows.
/// Why:
///   The controls of this module failed in whole-suite runs on 2026-11-09 with a push that took 23.1 s,
/// while the wait
///  allowed only five stall windows:
/// the stall window says when a push is silent for too long,
/// never how long a
///  push may take,
/// because a push that works is never stopped (owner,
/// 2026-11-03:
/// "Stop only on no progress").
/// The work here is a hold and nothing else,
/// so the figure measured is the wait's own.
#[test]
fn the_wait_for_a_service_push_holds_past_five_stall_windows() {
    let began: Instant = Instant::now();
    let finished: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            std::thread::sleep(HELD_PUSH);
            finished.store(true, Ordering::SeqCst);
        });
        assert!(wait_until(&|| return finished.load(Ordering::SeqCst)), "the wait held for work that took {HELD_PUSH:?}");
    });
    assert!(began.elapsed() >= HELD_PUSH, "the work really took that long: {:?}", began.elapsed());
}

/// What:
///  A push of a branch the person deleted settles with one notice and is not planned again,
/// and the notice names the
///  commit and the ref that still holds it.
/// Why:
///   Nothing was sent and every later attempt would fail the same way before anything is,
/// so the item has a terminal
///  state rather than an endless retry;
/// the commit is not on that branch any more because the person removed the branch,
///  and the words say where it still lives.
#[test]
fn a_push_of_a_branch_the_person_deleted_settles_with_one_notice() {
    let (root, git_dir, work, item, gone): (PathBuf, PathBuf, PathBuf, WorkItem, String) = deleted_branch_world("push-deleted-branch", false);
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None, "nothing is planned for a branch that is gone");
    assert_eq!(settled_names(work.as_path()), vec![item.id()], "the push settles as done, not as a failure to retry");
    let record: String = std::fs::read_dir(notice_directory(git_dir.as_path())).expect("the notices").flatten().map(|entry| { return std::fs::read_to_string(entry.path()).expect("notice"); }).collect();
    assert!(record.contains("no longer exists"), "the person is told the branch is gone: {record}");
    assert!(record.contains(gone.as_str()), "with the commit named: {record}");
    assert!(record.contains("refs/remotes/origin/feature"), "and the ref that still holds it: {record}");
    // Nothing is tried again: a second round plans nothing and writes no second record.
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None, "a second round plans nothing either");
    assert_eq!(std::fs::read_dir(notice_directory(git_dir.as_path())).expect("the notices").count(), 1, "one notice, not one per attempt");
    remove(root.as_path());
}

/// What:
///  A commit no ref of the repository holds any more is said to be in the object store alone,
/// and is named.
/// Why:
///   The person deleted the branch and the commit went with it,
/// so the only remaining word about that commit must say
///  that,
/// or they would look for it on a branch that does not exist.
#[test]
fn a_push_of_a_commit_no_ref_holds_says_so() {
    let (root, git_dir, work, item, gone): (PathBuf, PathBuf, PathBuf, WorkItem, String) = deleted_branch_world("push-deleted-branch-everywhere", true);
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None, "nothing is planned for a branch that is gone");
    assert_eq!(settled_names(work.as_path()), vec![item.id()], "the push settles as done");
    let record: String = std::fs::read_dir(notice_directory(git_dir.as_path())).expect("the notices").flatten().map(|entry| { return std::fs::read_to_string(entry.path()).expect("notice"); }).collect();
    assert!(record.contains("no ref of this repository"), "the person is told no ref holds the commit: {record}");
    assert!(record.contains(gone.as_str()), "and it is named: {record}");
    remove(root.as_path());
}

/// What:
///  A failed push of a branch that still exists is retried,
/// even when the program the push ran printed a line about a missing branch.
/// Why:
///   The words of a push's output can be any program's,
/// and only the repository says whether this branch is there;
/// settling a
///  live branch as deleted would stop its retry for good and tell the person it was deleted.
#[test]
fn a_failed_push_of_a_branch_that_exists_is_not_settled_as_a_gone_branch() {
    let root: PathBuf = fixture("push-live-branch-gone-words");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    // The program the push runs prints what a nested push of a missing branch prints, and refuses the push.
    let refusing: PathBuf = root.join("receive-pack-that-names-another-branch");
    std::fs::write(refusing.as_path(), "#!/bin/sh\necho 'error: src refspec refs/heads/helper does not match any' >&2\nexit 1\n").expect("the refusing program");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(refusing.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    }
    git(repo.as_path(), &["config", "remote.origin.receivepack", refusing.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"two\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let git_dir: PathBuf = repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-09T10:00:00.000Z"),
        tries: 0,
    };
    enqueue(work.as_path(), &item).expect("enqueue");
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    assert!(round_of(git_dir.as_path(), &handlers, &mut retries).is_some(), "the push is planned again, because main is still there");
    assert!(names_in(work.as_path(), crate::work_item::DONE).is_empty(), "and it is not settled as done");
    let record: String = std::fs::read_dir(notice_directory(git_dir.as_path())).map(|entries| { return entries.flatten().map(|entry| { return std::fs::read_to_string(entry.path()).unwrap_or_default(); }).collect::<String>(); }).unwrap_or_default();
    assert!(!record.contains("no longer exists"), "the person is not told the branch was deleted: {record}");
    remove(root.as_path());
}

/// What:
///  A notice about a deleted branch whose refs could not be read says that it could not tell,
/// never that no ref holds the commit.
/// Why:
///   A failed query is not an empty answer,
/// and the person would stop looking for a commit the repository still holds.
#[test]
fn a_notice_of_a_gone_branch_says_when_the_refs_could_not_be_read() {
    let root: PathBuf = fixture("push-gone-branch-unreadable-refs");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let git_dir: PathBuf = repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    // The branch is gone, and the commit the item names is no object of this repository, so the containment query fails.
    let absent: String = String::from("1234567890abcdef1234567890abcdef12345678");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/feature"),
        oid: absent.clone(),
        parent: String::new(),
        created: String::from("2026-11-09T10:00:00.000Z"),
        tries: 0,
    };
    enqueue(work.as_path(), &item).expect("enqueue");
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None, "the branch is gone, so nothing is planned");
    let record: String = std::fs::read_dir(notice_directory(git_dir.as_path())).map(|entries| { return entries.flatten().map(|entry| { return std::fs::read_to_string(entry.path()).unwrap_or_default(); }).collect::<String>(); }).unwrap_or_default();
    assert!(record.contains("no longer exists"), "the branch is said to be gone: {record}");
    assert!(record.contains("could not be told"), "and the notice says the refs could not be read: {record}");
    assert!(!record.contains("no ref of this repository"), "it never says no ref holds it: {record}");
    assert!(record.contains(absent.as_str()), "the commit is still named: {record}");
    remove(root.as_path());
}

/// The names a state directory of a work directory holds,
/// sorted.
fn names_in(work: &Path, state: &str) -> Vec<String> {
    let directory: PathBuf = crate::work_item::state_directory(work, state);
    let mut names: Vec<String> = std::fs::read_dir(directory.as_path())
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .collect();
    names.sort();
    return names;
}

/// What:
///  A repository with a commit on `feature`,
/// pushed once,
/// whose branch is then deleted.
/// Why:
///   The push's own failure is what the service must read,
/// so the fixture is a real repository and a real remote;
/// `everywhere` also deletes the branch on the remote and its tracking ref,
/// so no ref of the repository holds
///  the commit any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, remote } = await repositoryWithBranch('feature');
/// ```
fn deleted_branch_world(name: &str, everywhere: bool) -> (PathBuf, PathBuf, PathBuf, WorkItem, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("b.txt"), b"gone\n").expect("file");
    git(repo.as_path(), &["add", "--", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=two"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "feature"]);
    let gone: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    git(repo.as_path(), &["branch", "--delete", "--force", "feature"]);
    if everywhere {
        git(repo.as_path(), &["push", "--quiet", "origin", "--delete", "feature"]);
        git(repo.as_path(), &["update-ref", "-d", "refs/remotes/origin/feature"]);
    }
    let git_dir: PathBuf = repo.join(".git");
    let work: PathBuf = work_directory(git_dir.as_path());
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/feature"),
        oid: gone.clone(),
        parent: String::new(),
        created: String::from("2026-11-09T08:00:00.000Z"),
        tries: 0,
    };
    enqueue(work.as_path(), &item).expect("enqueue");
    return (root, git_dir, work, item, gone);
}

/// The names a work directory holds as done,
/// sorted.
fn settled_names(work: &Path) -> Vec<String> {
    let done: PathBuf = crate::work_item::state_directory(work, crate::work_item::DONE);
    let mut names: Vec<String> = std::fs::read_dir(done.as_path())
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .collect();
    names.sort();
    return names;
}

/// What a hook that declines every push says;
/// the person is shown it.
const DECLINE_MESSAGE: &str = "protected: pushes to main need a review";

/// The variables of the controls that go with a remote the service has no login for:
/// no prompt,
/// and no program that asks a
/// person (the desktop's askpass helper,
/// which the host's own settings may name and which waits on a window),
/// may be waited for.
const NO_PROMPT: [(&str, &str); 2] = [("GIT_TERMINAL_PROMPT", "0"), ("GIT_ASKPASS", "/bin/true")];

/// The text of every record of a failed push the person has not been shown yet.
fn notice_text(stage: &Stage) -> String {
    return std::fs::read_dir(notice_directory(stage.repo.join(".git").as_path()))
        .map(|entries| { return entries.flatten().map(|entry| { return std::fs::read_to_string(entry.path()).expect("notice"); }).collect::<String>(); })
        .unwrap_or_default();
}

/// Point the stage's remote at another address.
fn point_remote_at(stage: &Stage, url: &str) {
    git(stage.repo.as_path(), &["remote", "set-url", "origin", url]);
}

/// One round of the service over the stage's repository with real Git,
/// the way the work loop runs it.
fn round_with_real_git(stage: &Stage, retries: &mut PushRetries) -> Option<Duration> {
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&NO_PROMPT);
    return round_of(stage.repo.join(".git").as_path(), &handlers, retries);
}

/// A server that answers every request with a 401 and the request for a login,
/// until it is dropped.
struct LoginServer {
    /// The address it listens on.
    address: std::net::SocketAddr,
    /// Set when the server is to stop.
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// The thread that answers.
    thread: Option<std::thread::JoinHandle<()>>,
}

impl LoginServer {
    /// Start on a free loopback port.
    fn start() -> LoginServer {
        let listener: std::net::TcpListener = std::net::TcpListener::bind("127.0.0.1:0").expect("a free port");
        let address: std::net::SocketAddr = listener.local_addr().expect("its address");
        let stop: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stopping: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&stop);
        let thread = std::thread::spawn(move || {
            for connection in listener.incoming().flatten() {
                if stopping.load(Ordering::SeqCst) {
                    return;
                }
                let mut stream: std::net::TcpStream = connection;
                let mut request: [u8; 4096] = [0; 4096];
                let _read = std::io::Read::read(&mut stream, &mut request);
                let reply: &str = "HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"fixture\"\r\nContent-Length: 13\r\nConnection: close\r\n\r\nlogin needed\n";
                let _written = std::io::Write::write_all(&mut stream, reply.as_bytes());
            }
        });
        return LoginServer { address, stop, thread: Some(thread) };
    }
}

impl Drop for LoginServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _woken = std::net::TcpStream::connect(self.address);
        if let Some(thread) = self.thread.take() {
            let _joined = thread.join();
        }
    }
}

/// A push the server's hook declines is not tried again:
/// the person is told what the server said and that the service stops until
///  their next commit,
/// and the plan stays empty.
#[test]
fn a_push_a_server_hook_declines_is_not_tried_again() {
    let stage: Stage = stage("push-retry-declined");
    let hook: PathBuf = stage.remote.join("hooks").join("pre-receive");
    std::fs::write(hook.as_path(), format!("#!/bin/sh\necho '{DECLINE_MESSAGE}' >&2\nexit 1\n")).expect("hook");
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    assert_eq!(round_with_real_git(&stage, &mut retries), None, "no wake is asked for a decline");
    assert!(retries.directories().is_empty(), "no plan is made for a decline");
    let record: String = notice_text(&stage);
    assert!(record.contains(DECLINE_MESSAGE) && record.contains("hook declined"), "the person is told what the server said: {record}");
    assert!(record.contains("until your next commit") && !record.contains("tries this push again"), "{record}");
    remove(stage.root.as_path());
}

/// A push that needs a login the service does not have is not tried again,
/// and the person is told to log in.
#[test]
fn a_push_that_needs_a_login_is_not_tried_again() {
    let stage: Stage = stage("push-retry-login");
    let server: LoginServer = LoginServer::start();
    point_remote_at(&stage, format!("http://{}/r.git", server.address).as_str());
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    assert_eq!(round_with_real_git(&stage, &mut retries), None, "no wake is asked for a login failure");
    assert!(retries.directories().is_empty(), "no plan is made for a login failure");
    let record: String = notice_text(&stage);
    assert!(record.contains("Log in") && record.contains("until your next commit") && !record.contains("tries this push again"), "{record}");
    remove(stage.root.as_path());
}

/// The program that stands in for the desktop's password dialog (`SSH_ASKPASS`):
/// it writes the prompt it was given to `log` and
///  answers a user name,
/// so no window ever opens,
/// and the log shows whether anything asked.
fn recorder(root: &Path, log: &Path) -> PathBuf {
    let program: PathBuf = root.join("recorded-askpass");
    crate::test_support::executable(program.as_path(), format!("#!/bin/sh\nprintf '%s\\n' \"$1\" >> '{}'\nprintf 'probe-user\\n'\n", log.display()).as_bytes());
    return program;
}

/// What the recorder was asked,
/// one prompt per line;
/// nothing when it was never started.
fn prompts_recorded(log: &Path) -> String {
    return std::fs::read_to_string(log).unwrap_or_default();
}

/// The controls that show the dialog need `GIT_ASKPASS` unset in the process whose environment the service's Git inherits.
fn assert_askpass_is_unset_here() {
    assert!(std::env::var_os("GIT_ASKPASS").is_none(), "this control shows what an unset GIT_ASKPASS does; run it without GIT_ASKPASS in the environment");
}

/// A push to a server that answers 401 never starts the program the host names as its askpass helper:
/// Git tries `GIT_ASKPASS`,
///  then `core.askPass`,
/// then `SSH_ASKPASS` before it reads `GIT_TERMINAL_PROMPT` (Git 2.56.0 `prompt.c`),
/// so the service sets
///  `GIT_ASKPASS` to the empty string.
/// The failure is the login failure the person is told about,
/// reached without a wait.
#[test]
fn a_background_push_that_needs_a_login_never_starts_the_hosts_askpass_program() {
    assert_askpass_is_unset_here();
    let stage: Stage = stage("push-never-prompts");
    let server: LoginServer = LoginServer::start();
    point_remote_at(&stage, format!("http://{}/r.git", server.address).as_str());
    let log: PathBuf = stage.root.join("askpass.log");
    let program: PathBuf = recorder(stage.root.as_path(), log.as_path());
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&[("SSH_ASKPASS", program.to_str().expect("utf-8"))]);
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let wake: Option<Duration> = round_of(stage.repo.join(".git").as_path(), &handlers, &mut retries);
    assert_eq!(prompts_recorded(log.as_path()), "", "nothing asked a person for a name or a password");
    assert_eq!(wake, None, "a login failure is final and the push did not wait for an answer, so no wake is asked for");
    assert!(retries.directories().is_empty(), "no plan is made for a login failure");
    let record: String = notice_text(&stage);
    assert!(record.contains("Log in") && record.contains("until your next commit") && !record.contains("tries this push again"), "{record}");
    remove(stage.root.as_path());
}

/// The Git LFS upload of a background push gets the same environment:
/// `git-lfs` reads `GIT_ASKPASS`,
/// then `core.askpass`,
/// then
///  `SSH_ASKPASS` itself (`creds/creds.go`,
/// v3.8.1),
/// so the stand-in below follows that order and asks the recorder when it may,
///  and it keeps the `GIT_TERMINAL_PROMPT` it was given.
/// The real `git-lfs` is not in the gate's image;
/// the troubleshooting note
///  holds the run of the real one.
#[test]
fn a_background_lfs_upload_never_starts_the_hosts_askpass_program() {
    assert_askpass_is_unset_here();
    let stage: Stage = stage("lfs-never-prompts");
    let log: PathBuf = stage.root.join("askpass.log");
    let program: PathBuf = recorder(stage.root.as_path(), log.as_path());
    let seen: PathBuf = stage.root.join("lfs-terminal-prompt");
    let bin: PathBuf = stage.root.join("bin");
    std::fs::create_dir_all(bin.as_path()).expect("bin directory");
    crate::test_support::executable(
        bin.join("git-lfs").as_path(),
        format!("#!/bin/sh\nprintf '%s' \"$GIT_TERMINAL_PROMPT\" > '{}'\nasker=\"${{GIT_ASKPASS-$SSH_ASKPASS}}\"\nif [ -n \"$asker\" ]; then \"$asker\" 'Username for \"http://127.0.0.1\"' > /dev/null; fi\nexit 2\n", seen.display()).as_bytes(),
    );
    let path: String = format!("{}:/usr/bin:/bin", bin.display());
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&[("SSH_ASKPASS", program.to_str().expect("utf-8"))]);
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: stage.repo.to_path_buf(),
        git_dir: stage.repo.join(".git").to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: stage.landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-07T07:00:00.000Z"),
        tries: 0,
    };
    let plan: crate::lfs_upload_plan::UploadPlan = crate::lfs_upload_plan::UploadPlan { remotes: vec![std::ffi::OsString::from("origin")], refs: vec![std::ffi::OsString::from("main")], everything: false, submodules: false, full_scan: false, objects: None };
    // The lookup of `git-lfs` takes the first `PATH` of the environment, so the stand-in's directory replaces the inherited one.
    let mut git: crate::auto_push_single_flight::PushGit = handlers.push_git(&item);
    git.environment.retain(|(name, _)| { return name != "PATH"; });
    git.environment.push((std::ffi::OsString::from("PATH"), std::ffi::OsString::from(path.as_str())));
    let outcome = crate::lfs_upload::run_upload_bounded(&git, &plan);
    let failed = outcome.expect("the upload ran").expect("the stand-in failed the upload");
    assert_eq!(failed.exit_code, 2, "{}", failed.output);
    assert_eq!(prompts_recorded(log.as_path()), "", "git-lfs asked a person for a name");
    assert_eq!(std::fs::read_to_string(seen.as_path()).expect("the stand-in ran"), "0", "git-lfs was told not to prompt on a terminal");
    remove(stage.root.as_path());
}

/// The user and password of a URL a server's hook prints in the push controls that look for leaked credentials.
const SENTINEL_URL_USERINFO: &str = "sentinel-user:sentinel-password-4f9c";

/// Every file below `root` that holds `needle`,
/// skipping the directories named in `skip`.
fn files_holding(root: &Path, needle: &str, skip: &[PathBuf]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(root).expect("a readable directory").flatten() {
        let path: PathBuf = entry.path();
        if skip.contains(&path) {
            continue;
        }
        let kind: std::fs::FileType = entry.file_type().expect("a file type");
        if kind.is_dir() {
            found.extend(files_holding(path.as_path(), needle, skip));
        } else if kind.is_file() && std::fs::read(path.as_path()).is_ok_and(|bytes| { return String::from_utf8_lossy(bytes.as_slice()).contains(needle); }) {
            found.push(path);
        }
    }
    return found;
}

/// A server's words can carry a URL with a user and a password (a hook that prints where to log in is one way;
/// an older Git printed
///  the remote's own URL that way),
/// and the push output is stored for the person in the failure record,
/// the work records and the
///  journal.
/// None of them keeps the password or the user part:
/// the scan covers every file of the fixture except the hook that
///  prints it,
/// with a planted file first to show the scan finds a leak,
/// and the Git run of the push to show the words do carry it.
#[test]
fn a_failed_push_whose_output_names_a_url_with_a_password_leaves_it_in_no_file_the_wrapper_writes() {
    let stage: Stage = stage("push-output-keeps-no-password");
    let hooks: PathBuf = stage.remote.join("hooks");
    std::fs::write(hooks.join("pre-receive"), format!("#!/bin/sh\necho 'log in at https://{SENTINEL_URL_USERINFO}@example.test/login first' >&2\nexit 1\n")).expect("hook");
    let mut leaks: Vec<PathBuf> = Vec::new();
    let planted: PathBuf = stage.root.join("planted.txt");
    std::fs::write(planted.as_path(), format!("https://{SENTINEL_URL_USERINFO}@example.test/")).expect("planted file");
    assert_eq!(files_holding(stage.root.as_path(), SENTINEL_URL_USERINFO, std::slice::from_ref(&hooks)), vec![planted.clone()], "the scan finds a file that holds the credentials");
    std::fs::remove_file(planted.as_path()).expect("the planted file goes");
    let direct = crate::test_support::git_output(stage.repo.as_path(), &["push", "origin", "main"]);
    assert!(String::from_utf8_lossy(direct.stderr.as_slice()).contains(SENTINEL_URL_USERINFO), "Git's own words carry the credentials, so the scan below is not vacuous");
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&NO_PROMPT);
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let entries = crate::service_timing::capture::entries_of(|| { let _wake = round_of(stage.repo.join(".git").as_path(), &handlers, &mut retries); });
    let record: String = notice_text(&stage);
    assert!(record.contains("log in at https://") && record.contains("<credentials>"), "the person is shown the words with the credentials masked: {record}");
    leaks.extend(files_holding(stage.root.as_path(), SENTINEL_URL_USERINFO, std::slice::from_ref(&hooks)));
    for part in SENTINEL_URL_USERINFO.split(':') {
        leaks.extend(files_holding(stage.root.as_path(), part, std::slice::from_ref(&hooks)));
        assert!(entries.iter().all(|entry| { return !entry.message.contains(part); }), "the journal keeps the credentials of a URL: {part}");
    }
    assert!(leaks.is_empty(), "files that hold the credentials: {leaks:?}");
    remove(stage.root.as_path());
}

/// A push that cannot reach the remote at all is still tried again by the service.
#[test]
fn an_offline_push_is_still_tried_again() {
    let stage: Stage = stage("push-retry-offline");
    let closed: std::net::TcpListener = std::net::TcpListener::bind("127.0.0.1:0").expect("a free port");
    let address: std::net::SocketAddr = closed.local_addr().expect("its address");
    drop(closed);
    point_remote_at(&stage, format!("http://{address}/r.git").as_str());
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let wait: Duration = round_with_real_git(&stage, &mut retries).expect("a wake is asked for the retry");
    assert!(wait <= PAUSE + RETRY_SLACK, "{wait:?}");
    assert_eq!(retries.directories().len(), 1, "the push is planned again");
    assert!(notice_text(&stage).contains("tries this push again by itself"));
    remove(stage.root.as_path());
}

/// A rejection by the server for a reason this release does not know is tried again,
/// and the journal names the line it did not
///  recognise.
/// Git 2.56.0 refuses a reference that `receive.hideRefs` hides with `deny updating a hidden ref`
///  (`builtin/receive-pack.c`),
/// which is no hook and no login,
/// so it stands for a reason nobody has listed.
#[test]
fn a_rejection_the_release_does_not_know_is_tried_again_and_named_in_the_journal() {
    use crate::journald_native::Priority;
    let stage: Stage = stage("push-retry-unknown-rejection");
    // The stage's hook stalls the first push; this control is about the answer of the server, so the hook goes.
    std::fs::remove_file(stage.remote.join("hooks").join("pre-receive")).expect("the stalling hook");
    git(stage.remote.as_path(), &["config", "receive.hideRefs", "refs/heads/main"]);
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: stage.repo.to_path_buf(),
        git_dir: stage.repo.join(".git").to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: stage.landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&NO_PROMPT);
    let mut verdict: Option<Verdict> = None;
    let entries = crate::service_timing::capture::entries_of(|| { verdict = Some(handlers.push(std::slice::from_ref(&item))); });
    assert!(matches!(verdict, Some(Verdict::Retry(_))), "{verdict:?}");
    let named = entries.iter().find(|entry| { return entry.message.contains("a reason this release does not know"); }).expect("the journal names the rejection");
    assert_eq!(named.priority, Priority::Warning);
    assert!(named.message.contains("[remote rejected] main -> main (deny updating a hidden ref)"), "{}", named.message);
    remove(stage.root.as_path());
}

/// The directories a service of this instance was woken for,
/// as the journal of a service that ended holds them.
fn watched_by_an_earlier_service(stage: &Stage) -> PathBuf {
    let store: PathBuf = stage.root.join("journal").join(WATCH_DIRECTORY);
    ClaimWatch::restored(store.as_path(), None).remember(stage.repo.join(".git").as_path());
    return store;
}

/// Lets the stage's hook accept pushes,
/// as it does after its first stall.
fn let_pushes_through(stage: &Stage) {
    std::fs::write(stage.root.join("stalled-once"), b"").expect("the marker that makes the hook accept");
}

/// The file name of the push record of `commit` on the stage's branch,
/// which is `main`:
/// the record's own id,
/// which carries the branch as well as the commit.
fn push_record_id(commit: &str) -> String {
    return WorkItem {
        kind: WorkKind::Push,
        repository: PathBuf::new(),
        git_dir: PathBuf::new(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from(commit),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    }
    .id();
}

/// Moves the item of `commit` to `claimed` as a service would have claimed it `age` ago,
/// and gives the claim's path.
fn claim_as_a_dead_service_did(stage: &Stage, commit: &str, age: Duration) -> PathBuf {
    let work: PathBuf = work_directory(stage.repo.join(".git").as_path());
    let id: String = push_record_id(commit);
    let last_renewal: ClaimTime = ClaimTime::read().rewound(age);
    let claimed: PathBuf = work.join("claimed").join(crate::work_item::claim_file_name(id.as_str(), "0123456789abcdef0123456789abcdef", &last_renewal));
    std::fs::create_dir_all(work.join("claimed")).expect("the claimed directory");
    std::fs::rename(work.join("pending").join(id), claimed.as_path()).expect("claimed by the service that died");
    return claimed;
}

/// Run `work_loop` on a thread as a service that has just started:
/// the plan and the watch are read from the journal,
/// and no wake is sent.
fn start_service_watching<'scope>(scope: &'scope std::thread::Scope<'scope, '_>, handlers: &'scope GitHandlers, plans: &'scope Path, watched: &'scope Path, busy: &'scope AtomicUsize) -> (UnboundedSender<Wake>, std::thread::ScopedJoinHandle<'scope, ()>) {
    return start_service_watching_with(scope, handlers, PAUSE, plans, watched, busy);
}

/// The same with the first pause of a push that is tried again given.
fn start_service_watching_with<'scope>(scope: &'scope std::thread::Scope<'scope, '_>, handlers: &'scope GitHandlers, first: Duration, plans: &'scope Path, watched: &'scope Path, busy: &'scope AtomicUsize) -> (UnboundedSender<Wake>, std::thread::ScopedJoinHandle<'scope, ()>) {
    let (sender, mut receiver) = unbounded_channel::<Wake>();
    let running = scope.spawn(move || { work_loop(&mut receiver, handlers, busy, PushRetries::restored(first, plans), ClaimWatch::restored(watched, None)); });
    return (sender, running);
}

/// A service killed while it holds a claim is followed by one that runs the claim without any wake:
/// nobody commits after the kill,
///  no wrapper command runs,
/// and the service started in its place looks at the repository an earlier service was woken for,
/// finds
///  the claim held by a holder that does not renew it,
/// and pushes once the lease is over.
#[test]
fn a_claim_left_by_a_killed_service_is_run_by_its_successor_with_no_wake() {
    let stage: Stage = stage("push-claim-no-wake");
    let_pushes_through(&stage);
    let store: PathBuf = watched_by_an_earlier_service(&stage);
    let claimed: PathBuf = claim_as_a_dead_service_did(&stage, stage.landed.as_str(), crate::service_work::CLAIM_LEASE - Duration::from_secs(2));
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let plans: PathBuf = stage.root.join("journal").join(PLAN_DIRECTORY);
    let busy: AtomicUsize = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let (sender, running) = start_service_watching(scope, &handlers, plans.as_path(), store.as_path(), &busy);
        assert!(wait_until(&|| { return remote_tip(&stage) == stage.landed; }), "the successor pushed the claimed commit although nothing woke it");
        assert!(wait_until(&|| { return !claimed.exists(); }), "the claim was settled");
        drop(sender);
        running.join().expect("the service ends");
    });
    assert!(work_directory(stage.repo.join(".git").as_path()).join("done").join(push_record_id(stage.landed.as_str())).exists(), "settled as done");
    remove(stage.root.as_path());
}

/// The wake of a repository is written down by the worker before it drains,
/// so that a service killed in the middle of the first
///  claim is followed by one that knows where to look;
/// a service that restarts finds the file.
#[test]
fn a_wake_writes_the_repository_down_for_the_next_service() {
    let stage: Stage = stage("push-claim-wake-written");
    let_pushes_through(&stage);
    let git_dir: PathBuf = stage.repo.join(".git");
    let store: PathBuf = stage.root.join("journal").join(WATCH_DIRECTORY);
    let plans: PathBuf = stage.root.join("journal").join(PLAN_DIRECTORY);
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let busy: AtomicUsize = AtomicUsize::new(0);
    assert!(ClaimWatch::restored(store.as_path(), None).directories().is_empty(), "a new journal watches nothing");
    std::thread::scope(|scope| {
        let (sender, running) = start_service_watching(scope, &handlers, plans.as_path(), store.as_path(), &busy);
        sender.send((git_dir.clone(), None)).expect("the service takes the wake");
        assert!(wait_until(&|| { return remote_tip(&stage) == stage.landed; }), "the wake was run");
        drop(sender);
        running.join().expect("the service ends");
    });
    assert_eq!(ClaimWatch::restored(store.as_path(), None).directories(), vec![git_dir], "the next service looks at the repository by itself");
    remove(stage.root.as_path());
}

/// The service that starts in place of another runs what is pending at once but leaves a claim whose holder renewed it a moment
///  ago alone:
/// the lease decides that a holder is dead,
/// never the start of a successor,
/// so the work is not run twice.
#[test]
fn a_claim_whose_holder_renewed_it_is_left_alone_while_pending_work_is_run() {
    let stage: Stage = stage("push-claim-live-holder");
    let_pushes_through(&stage);
    let git_dir: PathBuf = stage.repo.join(".git");
    let store: PathBuf = watched_by_an_earlier_service(&stage);
    // The landed commit is claimed by a holder that renewed a moment ago; a newer commit is pending.
    std::fs::write(stage.repo.join("a.txt"), b"three\n").expect("file");
    git(stage.repo.as_path(), &["commit", "--quiet", "--all", "--message=three"]);
    let newer: String = git_text(stage.repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: stage.repo.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: newer.clone(),
        parent: stage.landed.clone(),
        created: String::from("2026-11-04T05:40:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(git_dir.as_path()).as_path(), &item).expect("enqueue");
    let claimed: PathBuf = claim_as_a_dead_service_did(&stage, stage.landed.as_str(), Duration::ZERO);
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let plans: PathBuf = stage.root.join("journal").join(PLAN_DIRECTORY);
    let busy: AtomicUsize = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let (sender, running) = start_service_watching(scope, &handlers, plans.as_path(), store.as_path(), &busy);
        assert!(wait_until(&|| { return remote_tip(&stage) == newer; }), "the pending push was run at the start");
        assert!(wait_until(&|| { return work_directory(git_dir.as_path()).join("done").join(push_record_id(newer.as_str())).exists(); }), "the pending push was settled");
        assert!(claimed.exists(), "the claim of a holder that renewed a moment ago was not taken");
        drop(sender);
        running.join().expect("the service ends");
    });
    remove(stage.root.as_path());
}

/// How far off the first try of a push plan is in the control that has one:
/// far past what the control waits for.
const FAR_PLAN_PAUSE: Duration = Duration::from_secs(600);

/// A repository that has a push plan due much later is still looked at at once when the service starts:
/// the plan is the push's
///  own schedule and must not hide the claim of a dead holder,
/// which runs out its lease long before.
/// The claim has two seconds of its lease left,
/// and the control waits for the delivery on the run's work:
/// measured on 2026-11-09,
/// the claim landed 3.2 to 3.9 s after the service started,
/// while the control's own fixture took 3.0 to 7.2 s
/// of real Git on the same host,
/// so a fixed patience shorter than the delivery would fail a service that behaved.
#[test]
fn a_push_plan_due_later_does_not_hide_the_startup_look() {
    let stage: Stage = stage("push-claim-plan-later");
    let_pushes_through(&stage);
    let git_dir: PathBuf = stage.repo.join(".git");
    let store: PathBuf = watched_by_an_earlier_service(&stage);
    let claimed: PathBuf = claim_as_a_dead_service_did(&stage, stage.landed.as_str(), crate::service_work::CLAIM_LEASE - Duration::from_secs(2));
    let plans: PathBuf = stage.root.join("journal").join(PLAN_DIRECTORY);
    let planned: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: stage.repo.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: stage.landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-04T05:40:00.000Z"),
        tries: 0,
    };
    crate::push_retry_store::save(plans.as_path(), &crate::push_retry_store::StoredPlan { git_dir: git_dir.clone(), item: planned, attempt: 1 }).expect("the plan is kept");
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let busy: AtomicUsize = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let (sender, running) = start_service_watching_with(scope, &handlers, FAR_PLAN_PAUSE, plans.as_path(), store.as_path(), &busy);
        assert!(wait_until(&|| { return remote_tip(&stage) == stage.landed; }), "the claim was run although the push plan is due only after {FAR_PLAN_PAUSE:?}");
        assert!(wait_until(&|| { return !claimed.exists(); }), "the claim was settled");
        drop(sender);
        running.join().expect("the service ends");
    });
    remove(stage.root.as_path());
}

/// Puts a file where the notes' directory should be,
/// so that nothing can be written or listed under it,
/// and gives the store's path.
fn block_the_notes(stage: &Stage) -> (PathBuf, PathBuf) {
    let blocker: PathBuf = stage.root.join("note-blocker");
    std::fs::write(&blocker, b"not a directory\n").expect("the blocker");
    return (blocker.clone(), blocker.join(WATCH_DIRECTORY));
}

/// A note that cannot be written when the service first drains a repository does not stop the work,
/// and is written once the fault
/// is mended,
/// without another wake:
/// a service killed after that finds the repository,
/// so a full disk at the first commit does not
/// leave the last commit's push to wait for a commit that may never come.
#[test]
fn a_note_that_could_not_be_written_is_written_after_the_fault_is_mended() {
    let stage: Stage = stage("push-claim-note-unwritable");
    let_pushes_through(&stage);
    let git_dir: PathBuf = stage.repo.join(".git");
    let (blocker, store): (PathBuf, PathBuf) = block_the_notes(&stage);
    let plans: PathBuf = stage.root.join("journal").join(PLAN_DIRECTORY);
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let busy: AtomicUsize = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let (sender, running) = start_service_watching(scope, &handlers, plans.as_path(), store.as_path(), &busy);
        sender.send((git_dir.clone(), None)).expect("the service takes the wake");
        assert!(wait_until(&|| { return remote_tip(&stage) == stage.landed; }), "the push was made although the note could not be written");
        assert!(wait_until(&|| { return busy.load(Ordering::SeqCst) == 1; }), "the service stays up for the note it has to write");
        std::fs::remove_file(&blocker).expect("the fault is mended");
        assert!(wait_until(&|| { return !ClaimWatch::restored(store.as_path(), None).directories().is_empty(); }), "the note was written by the retry");
        drop(sender);
        running.join().expect("the service ends");
    });
    remove(stage.root.as_path());
}

/// A store of notes that cannot be listed when the service starts does not end the look:
/// the service stays up,
/// lists it again
/// after a pause,
/// and runs the claim of the repository the note names.
#[test]
fn a_store_that_could_not_be_listed_at_the_start_is_listed_again_and_its_claim_is_run() {
    let stage: Stage = stage("push-claim-store-unlistable");
    let_pushes_through(&stage);
    let (blocker, store): (PathBuf, PathBuf) = block_the_notes(&stage);
    let claimed: PathBuf = claim_as_a_dead_service_did(&stage, stage.landed.as_str(), crate::service_work::CLAIM_LEASE - Duration::from_secs(2));
    let plans: PathBuf = stage.root.join("journal").join(PLAN_DIRECTORY);
    let handlers: GitHandlers = GitHandlers::new(git_context());
    let busy: AtomicUsize = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let (sender, running) = start_service_watching(scope, &handlers, plans.as_path(), store.as_path(), &busy);
        assert!(wait_until(&|| { return busy.load(Ordering::SeqCst) == 1; }), "the service stays up for the store it has to list again");
        // The fault is mended and the note an earlier service wrote is there.
        std::fs::remove_file(&blocker).expect("the fault is mended");
        ClaimWatch::restored(store.as_path(), None).remember(stage.repo.join(".git").as_path());
        assert!(wait_until(&|| { return remote_tip(&stage) == stage.landed; }), "the claim was run once the store could be listed");
        assert!(wait_until(&|| { return !claimed.exists(); }), "the claim was settled");
        drop(sender);
        running.join().expect("the service ends");
    });
    remove(stage.root.as_path());
}

/// Moves a Git directory's parent aside,
/// the way an unmounted drive takes a repository away,
/// and back.
fn away(from: &Path, to: &Path) {
    std::fs::rename(from, to).expect("the directory goes away or comes back");
}

/// A push plan whose directory is away keeps its plan and doubles its pause at every look,
/// runs nothing while the directory is away,
/// and delivers once it is back:
/// the push is not forgotten and never runs anywhere else.
#[test]
fn a_push_plan_whose_directory_is_away_keeps_its_doubling_pause_and_runs_nothing_until_it_is_back() {
    let (root, git_dir, handlers, mut retries): (PathBuf, PathBuf, Scripted, PushRetries) =
        scripted_world("push-retry-away", vec![Verdict::Done, Verdict::Retry(String::from("Could not resolve host"))]);
    let first: Duration = round_of(git_dir.as_path(), &handlers, &mut retries).expect("a retry is planned");
    let aside: PathBuf = root.join("common-away");
    away(git_dir.as_path(), aside.as_path());
    std::thread::sleep(first);
    let second: Duration = round_of(git_dir.as_path(), &handlers, &mut retries).expect("the plan is kept while the directory is away");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "nothing ran while the directory was away");
    assert!(second > first, "the pause doubled: {first:?} then {second:?}");
    std::thread::sleep(second);
    let third: Duration = round_of(git_dir.as_path(), &handlers, &mut retries).expect("the plan is still kept");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "still nothing ran");
    assert!(third > second, "the pause doubled again: {second:?} then {third:?}");
    away(aside.as_path(), git_dir.as_path());
    std::thread::sleep(third);
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None, "the push landed and nothing is left to try");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 2, "it ran once the directory was back");
    remove(root.as_path());
}

/// A wake for a directory that is away runs nothing and touches nothing:
/// its work stays where it is for the next command,
/// and runs once the directory is back.
#[test]
fn a_wake_for_a_directory_that_is_away_leaves_its_work_for_the_next_command() {
    let (root, git_dir, handlers, _retries): (PathBuf, PathBuf, Scripted, PushRetries) = scripted_world("push-retry-away-wake", vec![Verdict::Done]);
    let aside: PathBuf = root.join("common-away");
    away(git_dir.as_path(), aside.as_path());
    let drained: Drained = drain_planned(git_dir.as_path(), &handlers, &ClaimTime::read(), &Retrying::none());
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 0, "nothing ran while the directory was away");
    assert_eq!(drained.wait, None);
    assert!(drained.changes.is_empty());
    assert!(aside.exists(), "the directory was not recreated at its old path: {}", git_dir.exists());
    assert!(!git_dir.exists(), "no directory was made at the old path");
    away(aside.as_path(), git_dir.as_path());
    let _wait: Drained = drain_planned(git_dir.as_path(), &handlers, &ClaimTime::read(), &Retrying::none());
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "the work ran once the directory was back");
    remove(root.as_path());
}

/// What:
///  A push the service stopped after the remote had already written the reference is settled by its own retry,
///  and the notice it left claims nothing about the remote.
/// Why:
///   The wrapper cannot know which object Git really sent,
/// which of a remote's push URLs went first or which
///  configuration shaped the attempt,
/// so a stopped push says only what is known:
/// that it was stopped,
/// that the remote may
///  or may not hold the commit,
/// and whether a retry is planned.
/// The retry settles the question,
/// because a push of something the
///  remote already has succeeds and clears the record.
#[test]
fn a_stopped_push_whose_reference_landed_is_settled_by_its_retry_and_claims_nothing() {
    let stage: Stage = stage_with("push-retry-post-receive-once", "post-receive");
    let git_dir: PathBuf = stage.repo.join(".git");
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&[("CLI_GIT_PUSH_DEADLINE_SECONDS", STALL_SECONDS)]);
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let first: Option<Duration> = round_of(git_dir.as_path(), &handlers, &mut retries);
    assert_eq!(remote_tip(&stage), stage.landed, "the remote wrote the reference before its hook stalled");
    assert_eq!(notices(&stage), 1, "the person is told the push was stopped");
    let mut stored: String = String::new();
    for entry in std::fs::read_dir(notice_directory(git_dir.as_path())).expect("the notice directory").flatten() {
        stored.push_str(std::fs::read_to_string(entry.path()).expect("the record").as_str());
    }
    assert!(
        stored.contains("may or may not have taken"),
        "the notice says what is known about the remote: {stored}"
    );
    for absent in ["delivered every reference", "nothing is left to deliver", "does not restart that work"] {
        assert!(!stored.contains(absent), "{absent} in {stored}");
    }
    let wait: Duration = first.expect("the service says when to look at the repository again");
    assert!(wait <= PAUSE + Duration::from_secs(5), "the wake is for the retry: {wait:?}");
    std::thread::sleep(wait);
    let second: Option<Duration> = round_of(git_dir.as_path(), &handlers, &mut retries);
    assert_eq!(remote_tip(&stage), stage.landed, "the retry finds the reference already there and succeeds");
    assert_eq!(notices(&stage), 0, "the retry cleared the record of the stopped push");
    assert_eq!(second, None, "nothing is left to try again");
    remove(stage.root.as_path());
}

/// What:
///  A push that was due again and did not finish where nothing keeps a retry plan is worded as one nothing will try
///  again,
/// and the branch's plan is dropped as it is.
/// Why:
///   `run_due_push_retries` forgets the branch's plan when the round keeps no plan (`replanning.first` is nothing),
/// so a
///  notice promising that the service keeps trying would send the person to wait for a try that never comes:
/// the
///  wording has to follow whether a plan still exists.
#[test]
fn a_due_push_with_no_retry_plan_left_is_worded_as_one_nothing_tries_again() {
    let root: PathBuf = fixture("push-retry-no-plan");
    let git_dir: PathBuf = root.join("common");
    std::fs::create_dir_all(git_dir.as_path()).expect("common directory");
    std::fs::create_dir_all(root.join("worktree").as_path()).expect("worktree directory");
    let handlers: Scripted = Scripted { verdicts: Mutex::new(vec![Verdict::Retry(String::from("Could not resolve host"))]), pushes: AtomicUsize::new(0) };
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: root.join("worktree"),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
    let due: Due = Due { git_dir: git_dir.clone(), item, attempt: 1 };
    // A round that keeps no plan: no first pause, and this push due again.
    let round: Retrying = Retrying { first: None, due: vec![due], plans: None };
    let drained: Drained = drain_planned_with(git_dir.as_path(), &handlers, &ClaimTime::read(), crate::service_work::CLAIM_LEASE, Renewal::Timed, &round);
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "the due push ran");
    assert!(
        drained.changes.iter().any(|change| { return matches!(change, RetryChange::Forget { .. }); }),
        "the branch's plan was dropped"
    );
    // The wording is applied when the notice is shown, so that is what the person reads.
    let shown: Vec<String> = crate::test_support::shown_notices(crate::push_failure_notice::push_failures(git_dir.as_path()));
    assert_eq!(shown.len(), 1, "one record for the push: {shown:?}");
    let notice: &str = shown[0].as_str();
    assert!(notice.contains("Could not resolve host"), "the person is told what happened: {notice}");
    assert!(
        !notice.contains("keeps the push in its retry plan"),
        "the notice promises no retry the service does not plan: {notice}"
    );
    assert!(
        notice.contains("not trying this push again"),
        "the notice says the push is not tried again: {notice}"
    );
    remove(root.as_path());
}

/// What:
///  A push the round planned again is worded as one the service keeps trying.
/// Why:
///   This is the other side of the plan:
/// the person is told an automatic retry is coming only when one is planned,
/// so a wording that never promises one would be as wrong as one that always does.
#[test]
fn a_due_push_with_a_kept_plan_is_worded_as_one_the_service_keeps_trying() {
    let root: PathBuf = fixture("push-retry-plan-kept");
    let git_dir: PathBuf = root.join("common");
    std::fs::create_dir_all(git_dir.as_path()).expect("common directory");
    std::fs::create_dir_all(root.join("worktree").as_path()).expect("worktree directory");
    let handlers: Scripted = Scripted { verdicts: Mutex::new(vec![Verdict::Retry(String::from("Could not resolve host"))]), pushes: AtomicUsize::new(0) };
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: root.join("worktree"),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
    let due: Due = Due { git_dir: git_dir.clone(), item, attempt: 1 };
    // A round that keeps a plan: this push due again, and the first pause to plan the next try with.
    let round: Retrying = Retrying { first: Some(PAUSE), due: vec![due], plans: None };
    let drained: Drained = drain_planned_with(git_dir.as_path(), &handlers, &ClaimTime::read(), crate::service_work::CLAIM_LEASE, Renewal::Timed, &round);
    assert!(
        drained.changes.iter().any(|change| { return matches!(change, RetryChange::Plan { .. }); }),
        "a new try was planned"
    );
    let shown: Vec<String> = crate::test_support::shown_notices(crate::push_failure_notice::push_failures(git_dir.as_path()));
    assert_eq!(shown.len(), 1, "one record for the push: {shown:?}");
    let notice: &str = shown[0].as_str();
    assert!(notice.contains("keeps the push in its retry plan"), "the notice promises the planned retry: {notice}");
    assert!(!notice.contains("not trying this push again"), "the notice does not deny the plan it kept: {notice}");
    remove(root.as_path());
}

/// A repository of the handler controls:
/// a bare remote,
/// a clone that pushed its first commit,
/// and a second commit the push handler delivers.
fn repository_with_a_landed_commit(root: &Path, name: &str) -> (PathBuf, PathBuf, String) {
    let repo: PathBuf = repository(root, name);
    let remote: PathBuf = root.join(format!("{name}.git"));
    git(root, &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    std::fs::write(repo.join("a.txt"), b"two\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    return (repo, remote, landed);
}

/// The branch tip a bare remote holds,
/// read from its own ref file.
fn tip_of(remote: &Path) -> String {
    return std::fs::read_to_string(remote.join("refs").join("heads").join("main")).map(|text| { return text.trim().to_owned(); }).unwrap_or_default();
}

/// The work item of a landed commit of `repo`.
fn push_item(repo: &Path, landed: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: repo.to_path_buf(),
        git_dir: repo.join(".git"),
        branch_ref: String::from("refs/heads/main"),
        oid: landed.to_owned(),
        parent: String::new(),
        created: String::from("2026-11-09T00:00:00.000Z"),
        tries: 0,
    };
}

/// The handlers' push answers the same verdict whether a task or a plain thread asks it,
/// and both deliver the commit.
#[test]
fn the_awaited_push_of_the_handlers_answers_what_the_synchronous_one_answers() {
    let root: PathBuf = fixture("handlers-awaiting-push");
    std::fs::create_dir_all(root.as_path()).expect("the fixture's folder");
    let (thread_repo, thread_remote, thread_landed) = repository_with_a_landed_commit(root.as_path(), "thread");
    let (task_repo, task_remote, task_landed) = repository_with_a_landed_commit(root.as_path(), "task");
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&NO_PROMPT);
    let from_a_thread: Verdict = handlers.push(std::slice::from_ref(&push_item(thread_repo.as_path(), thread_landed.as_str())));
    let from_a_task: Verdict = crate::merge_second_pass::tests::await_it(
        handlers.push_awaiting(std::slice::from_ref(&push_item(task_repo.as_path(), task_landed.as_str()))),
    );
    assert_eq!(from_a_task, from_a_thread, "the two faces answer one push the same way");
    assert_eq!(from_a_task, Verdict::Done, "the push landed");
    assert_eq!(tip_of(thread_remote.as_path()), thread_landed, "the commit reached the remote");
    assert_eq!(tip_of(task_remote.as_path()), task_landed, "the task's push reached its remote");
    remove(root.as_path());
}

/// The handlers' upkeep answers the same verdict whether a task or a plain thread asks it.
#[test]
fn the_awaited_maintenance_of_the_handlers_answers_what_the_synchronous_one_answers() {
    let root: PathBuf = fixture("handlers-awaiting-maintenance");
    std::fs::create_dir_all(root.as_path()).expect("the fixture's folder");
    let (thread_repo, _thread_remote, thread_landed) = repository_with_a_landed_commit(root.as_path(), "thread");
    let (task_repo, _task_remote, task_landed) = repository_with_a_landed_commit(root.as_path(), "task");
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&NO_PROMPT);
    let from_a_thread: Verdict = handlers.maintenance(&push_item(thread_repo.as_path(), thread_landed.as_str()));
    let from_a_task: Verdict = crate::merge_second_pass::tests::await_it(
        handlers.maintenance_awaiting(&push_item(task_repo.as_path(), task_landed.as_str())),
    );
    assert_eq!(from_a_task, from_a_thread, "the two faces answer one upkeep the same way");
    assert_eq!(from_a_task, Verdict::Done, "the upkeep ran");
    remove(root.as_path());
}
