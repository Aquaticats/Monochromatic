//! Controls for the service's own retry of a push that was stopped or failed: a local bare remote whose `pre-receive` hook
//! stalls the first push and accepts the next, real Git pushing, and the service's work loop driven by hand.
//! Why:
//!  At 05:40 on 2026-11-03 a stopped auto-push of `main` was never tried again until a person pushed by hand 15 minutes
//!  later, which also blocked a release (the owner's rule, `SPEC.md` "Merges and commits never fail": auto-push must
//!  land). The retry needs no new commit: the work loop looks at the repository again when the pause is over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const wait = await round(gitDir, retries); await sleep(wait); await round(gitDir, retries); expect(remoteHas(commit)).toBe(true);
//! ```
#![cfg(unix)]

use super::*;
use crate::push_failure_notice::notice_directory;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::work_item::enqueue;
use std::os::unix::fs::PermissionsExt;
use crate::push_retry::PLAN_DIRECTORY;
use crate::service_retry::RETRY_SLACK;
use crate::service_server::work_loop;
use std::sync::mpsc::{Sender, channel};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// One round over a single directory, the way the service's loop runs it: what is due comes out of `retries`, the worker's changes
/// go back into it, and the wait is when to look at the directory again.
fn round_of<H: WorkHandlers + Sync>(git_dir: &Path, handlers: &H, retries: &mut PushRetries) -> Option<Duration> {
    let asked: Vec<(PathBuf, Duration)> = run_planned_round(&HostResources::read(), &[git_dir.to_path_buf()], handlers, retries, &ClaimTime::read());
    return asked.into_iter().find(|(directory, _)| { return directory == git_dir; }).map(|(_, wait)| { return wait; });
}

/// The stall window of the controls, short enough to wait for and longer than a healthy local push.
const STALL_SECONDS: &str = "2";

/// The first pause before a push is tried again, in the controls.
const PAUSE: Duration = Duration::from_millis(800);

/// A repository tracking a bare remote whose hook stalls the first push it sees after the setup.
struct Stage {
    /// The fixture directory, removed at the end.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// The bare remote.
    remote: PathBuf,
    /// The commit to deliver.
    landed: String,
}

/// Build the stage: an initial commit pushed with no hook, the hook installed, a new commit landed, its work item written.
fn stage(name: &str) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let marker: PathBuf = root.join("stalled-once");
    let hook: PathBuf = remote.join("hooks").join("pre-receive");
    std::fs::write(hook.as_path(), format!("#!/bin/sh\nif [ ! -e '{}' ]; then : > '{}'; exec sleep 600; fi\nexit 0\n", marker.display(), marker.display())).expect("hook");
    std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("hook is executable");
    std::fs::write(repo.join("a.txt"), b"two\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let work_item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.display().to_string(),
        git_dir: repo.join(".git").display().to_string(),
        branch_ref: String::from("refs/heads/main"),
        oid: landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(repo.join(".git").as_path()).as_path(), &work_item).expect("enqueue");
    return Stage { root, repo, remote, landed };
}

/// The branch tip the remote holds.
fn remote_tip(stage: &Stage) -> String {
    return git_text(stage.remote.as_path(), &["rev-parse", "refs/heads/main"]).trim().to_owned();
}

/// How many records of failed pushes the repository holds for the person.
fn notices(stage: &Stage) -> usize {
    return std::fs::read_dir(notice_directory(stage.repo.join(".git").as_path())).map_or(0, |entries| { return entries.flatten().count(); });
}

/// A push the service stopped is tried again by the service alone, after a pause, with no new commit: the first drain stops
///  the stalled push and says when it will try again, the second (the work loop's wake after the pause) delivers, and the
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

/// Handlers whose pushes answer from a script, one verdict per push.
struct Scripted {
    /// The verdicts still to give, the next last. A test double: the round's workers share the handlers, and a script is read in order.
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

/// A directory standing for a repository, with a push item written for it.
fn scripted_world(name: &str, verdicts: Vec<Verdict>) -> (PathBuf, PathBuf, Scripted, PushRetries) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir_all(&git_dir).expect("common directory");
    std::fs::create_dir_all(&worktree).expect("worktree directory");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: worktree.display().to_string(),
        git_dir: git_dir.display().to_string(),
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

/// Every failure that can pass is tried again, each time after a longer pause, until the push lands; the record of the failure
/// says the service keeps trying, and goes when the push lands.
#[test]
fn a_push_that_keeps_failing_is_tried_again_with_longer_pauses_until_it_lands() {
    // The script is read from the end: two failures, then success.
    let (root, git_dir, handlers, mut retries): (PathBuf, PathBuf, Scripted, PushRetries) =
        scripted_world("push-retry-grows", vec![Verdict::Done, Verdict::Retry(String::from("Could not resolve host")), Verdict::Retry(String::from("Could not resolve host"))]);
    let first: Duration = round_of(git_dir.as_path(), &handlers, &mut retries).expect("a retry is planned");
    std::thread::sleep(first);
    let second: Duration = round_of(git_dir.as_path(), &handlers, &mut retries).expect("a second retry is planned");
    assert!(second > first + Duration::from_millis(150), "the pause grew: {first:?} then {second:?}");
    let record: String = std::fs::read_dir(notice_directory(git_dir.as_path())).expect("the notices").flatten().map(|entry| { return std::fs::read_to_string(entry.path()).expect("notice"); }).collect();
    assert!(record.contains("tries this push again by itself") && record.contains("Could not resolve host"), "{record}");
    std::thread::sleep(second);
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None, "nothing is left to try");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 3);
    assert_eq!(std::fs::read_dir(notice_directory(git_dir.as_path())).map_or(0, |entries| { return entries.flatten().count(); }), 0, "the record went when the push landed");
    remove(root.as_path());
}

/// A conflict is final: the push is not tried again, and the person's record stays.
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

/// A new commit's push replaces the plan of its branch: a push that lands needs no retry.
#[test]
fn a_push_that_lands_cancels_the_plan_of_its_branch() {
    let (root, git_dir, handlers, mut retries): (PathBuf, PathBuf, Scripted, PushRetries) = scripted_world("push-retry-cancelled", vec![Verdict::Retry(String::from("stalled"))]);
    assert!(round_of(git_dir.as_path(), &handlers, &mut retries).is_some());
    retries.forget(git_dir.as_path(), "refs/heads/main");
    assert_eq!(round_of(git_dir.as_path(), &handlers, &mut retries), None);
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "a forgotten plan is not pushed");
    remove(root.as_path());
}

/// How often the restart control looks at what the services have done.
const LOOK_EVERY: Duration = Duration::from_millis(50);

/// How long the restart control gives a restarted service to deliver: its first pause, the slack the schedule adds, and a margin
/// far longer than a healthy local push.
const RESTART_PATIENCE: Duration = Duration::from_secs(10).saturating_add(PAUSE).saturating_add(RETRY_SLACK);

/// Run `work_loop` on a thread, as the service does, and give the way to wake it; dropping the sender stops it.
fn start_service<'scope>(scope: &'scope std::thread::Scope<'scope, '_>, handlers: &'scope GitHandlers, plans: &'scope Path, busy: &'scope AtomicUsize) -> (Sender<Wake>, std::thread::ScopedJoinHandle<'scope, ()>) {
    let (sender, receiver) = channel::<Wake>();
    let running = scope.spawn(move || { work_loop(&receiver, handlers, busy, PushRetries::restored(PAUSE, plans)); });
    return (sender, running);
}

/// Wait until `done` holds, looking every `LOOK_EVERY` for at most `RESTART_PATIENCE`; gives whether it held.
fn wait_until(done: &dyn Fn() -> bool) -> bool {
    let began: Instant = Instant::now();
    while began.elapsed() < RESTART_PATIENCE {
        if done() {
            return true;
        }
        std::thread::sleep(LOOK_EVERY);
    }
    return done();
}

/// A service that is stopped while a push waits for a new try does not take the plan with it: the service started in its place
///  delivers the commit by itself, with no new commit and no wake from a wrapper command. The plan is not the person's failure
///  record, which the next wrapper command shows and removes.
#[test]
fn a_stopped_push_is_still_tried_again_after_the_service_restarts() {
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
        let _shown = crate::push_failure_notice::take_push_failures(git_dir.as_path());
        let (nobody, second) = start_service(scope, &handlers, plans.as_path(), &busy);
        assert!(wait_until(&|| { return remote_tip(&stage) == stage.landed; }), "the restarted service delivered the commit by itself");
        drop(nobody);
        second.join().expect("the second service ends");
    });
    assert_eq!(notices(&stage), 0, "the record of the failure is gone once the push landed");
    assert_eq!(git_text(stage.repo.as_path(), &["rev-list", "--count", "--all"]), commits_before, "no commit was made");
    remove(stage.root.as_path());
}
