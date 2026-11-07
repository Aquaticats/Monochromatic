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

use std::time::Instant;
use crate::work_item::WorkKind;
use crate::push_retry::PushRetries;
use crate::parallel_budget::HostResources;
use crate::service_work_run_round::run_planned_round;
use super::*;
use crate::push_failure_notice::notice_directory;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::work_item::enqueue;
use std::os::unix::fs::PermissionsExt;
use crate::push_retry::PLAN_DIRECTORY;
use crate::service_claim_watch::{ClaimWatch, WATCH_DIRECTORY};
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
    // That each pause is twice the one before is controlled in `push_retry_tests.rs`; a wait measured here also holds the time the
    //  round took, which a loaded host stretches, so only its growth is asked.
    assert!(second > first, "the pause grew: {first:?} then {second:?}");
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

/// The margin the restart control adds for a loaded host: many times the stall window of the controls, which is itself longer than
/// a healthy local push, so a slow start is waited for and a plan that was lost is still found out within a fixed time.
const RESTART_MARGIN: Duration = Duration::from_secs(STALL_SECONDS_NUMBER * 5);

/// The stall window of the controls as a number.
const STALL_SECONDS_NUMBER: u64 = 2;

/// How long the restart control gives a restarted service to deliver: its first pause, the slack the schedule adds, and the margin.
const RESTART_PATIENCE: Duration = RESTART_MARGIN.saturating_add(PAUSE).saturating_add(RETRY_SLACK);

/// Run `work_loop` on a thread, as the service does, and give the way to wake it; dropping the sender stops it.
fn start_service<'scope>(scope: &'scope std::thread::Scope<'scope, '_>, handlers: &'scope GitHandlers, plans: &'scope Path, busy: &'scope AtomicUsize) -> (Sender<Wake>, std::thread::ScopedJoinHandle<'scope, ()>) {
    let (sender, receiver) = channel::<Wake>();
    let running = scope.spawn(move || { work_loop(&receiver, handlers, busy, PushRetries::restored(PAUSE, plans), ClaimWatch::in_memory()); });
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

/// What a hook that declines every push says; the person is shown it.
const DECLINE_MESSAGE: &str = "protected: pushes to main need a review";

/// The variables of the controls that go with a remote the service has no login for: no prompt, and no program that asks a
/// person (the desktop's askpass helper, which the host's own settings may name and which waits on a window), may be waited for.
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

/// One round of the service over the stage's repository with real Git, the way the work loop runs it.
fn round_with_real_git(stage: &Stage, retries: &mut PushRetries) -> Option<Duration> {
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&NO_PROMPT);
    return round_of(stage.repo.join(".git").as_path(), &handlers, retries);
}

/// A server that answers every request with a 401 and the request for a login, until it is dropped.
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

/// A push the server's hook declines is not tried again: the person is told what the server said and that the service stops until
///  their next commit, and the plan stays empty.
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

/// A push that needs a login the service does not have is not tried again, and the person is told to log in.
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

/// The program that stands in for the desktop's password dialog (`SSH_ASKPASS`): it writes the prompt it was given to `log` and
///  answers a user name, so no window ever opens, and the log shows whether anything asked.
fn recorder(root: &Path, log: &Path) -> PathBuf {
    let program: PathBuf = root.join("recorded-askpass");
    crate::test_support::executable(program.as_path(), format!("#!/bin/sh\nprintf '%s\\n' \"$1\" >> '{}'\nprintf 'probe-user\\n'\n", log.display()).as_bytes());
    return program;
}

/// What the recorder was asked, one prompt per line; nothing when it was never started.
fn prompts_recorded(log: &Path) -> String {
    return std::fs::read_to_string(log).unwrap_or_default();
}

/// The controls that show the dialog need `GIT_ASKPASS` unset in the process whose environment the service's Git inherits.
fn assert_askpass_is_unset_here() {
    assert!(std::env::var_os("GIT_ASKPASS").is_none(), "this control shows what an unset GIT_ASKPASS does; run it without GIT_ASKPASS in the environment");
}

/// A push to a server that answers 401 never starts the program the host names as its askpass helper: Git tries `GIT_ASKPASS`,
///  then `core.askPass`, then `SSH_ASKPASS` before it reads `GIT_TERMINAL_PROMPT` (Git 2.56.0 `prompt.c`), so the service sets
///  `GIT_ASKPASS` to the empty string. The failure is the login failure the person is told about, reached without a wait.
#[test]
fn a_background_push_that_needs_a_login_never_starts_the_hosts_askpass_program() {
    assert_askpass_is_unset_here();
    let stage: Stage = stage("push-never-prompts");
    let server: LoginServer = LoginServer::start();
    point_remote_at(&stage, format!("http://{}/r.git", server.address).as_str());
    let log: PathBuf = stage.root.join("askpass.log");
    let program: PathBuf = recorder(stage.root.as_path(), log.as_path());
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&[("SSH_ASKPASS", program.to_str().expect("utf-8")), ("CLI_GIT_PUSH_DEADLINE_SECONDS", STALL_SECONDS)]);
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let began: Instant = Instant::now();
    let wake: Option<Duration> = round_of(stage.repo.join(".git").as_path(), &handlers, &mut retries);
    let waited: Duration = began.elapsed();
    assert_eq!(prompts_recorded(log.as_path()), "", "nothing asked a person for a name or a password");
    assert!(waited < Duration::from_secs(STALL_SECONDS.parse().expect("seconds")), "the push ended at once instead of waiting for an answer: {waited:?}");
    assert_eq!(wake, None, "a login failure is final, so no wake is asked for");
    assert!(retries.directories().is_empty(), "no plan is made for a login failure");
    let record: String = notice_text(&stage);
    assert!(record.contains("Log in") && record.contains("until your next commit") && !record.contains("tries this push again"), "{record}");
    remove(stage.root.as_path());
}

/// The Git LFS upload of a background push gets the same environment: `git-lfs` reads `GIT_ASKPASS`, then `core.askpass`, then
///  `SSH_ASKPASS` itself (`creds/creds.go`, v3.8.1), so the stand-in below follows that order and asks the recorder when it may,
///  and it keeps the `GIT_TERMINAL_PROMPT` it was given. The real `git-lfs` is not in the gate's image; the troubleshooting note
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
        repository: stage.repo.display().to_string(),
        git_dir: stage.repo.join(".git").display().to_string(),
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

/// A rejection by the server for a reason this release does not know is tried again, and the journal names the line it did not
///  recognise. Git 2.56.0 refuses a reference that `receive.hideRefs` hides with `deny updating a hidden ref`
///  (`builtin/receive-pack.c`), which is no hook and no login, so it stands for a reason nobody has listed.
#[test]
fn a_rejection_the_release_does_not_know_is_tried_again_and_named_in_the_journal() {
    use crate::journald_native::Priority;
    let stage: Stage = stage("push-retry-unknown-rejection");
    // The stage's hook stalls the first push; this control is about the answer of the server, so the hook goes.
    std::fs::remove_file(stage.remote.join("hooks").join("pre-receive")).expect("the stalling hook");
    git(stage.remote.as_path(), &["config", "receive.hideRefs", "refs/heads/main"]);
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: stage.repo.display().to_string(),
        git_dir: stage.repo.join(".git").display().to_string(),
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

/// The directories a service of this instance was woken for, as the journal of a service that ended holds them.
fn watched_by_an_earlier_service(stage: &Stage) -> PathBuf {
    let store: PathBuf = stage.root.join("journal").join(WATCH_DIRECTORY);
    ClaimWatch::restored(store.as_path(), None).remember(stage.repo.join(".git").as_path());
    return store;
}

/// Lets the stage's hook accept pushes, as it does after its first stall.
fn let_pushes_through(stage: &Stage) {
    std::fs::write(stage.root.join("stalled-once"), b"").expect("the marker that makes the hook accept");
}

/// Moves the item of `commit` to `claimed` as a service would have claimed it `age` ago, and gives the claim's path.
fn claim_as_a_dead_service_did(stage: &Stage, commit: &str, age: Duration) -> PathBuf {
    let work: PathBuf = work_directory(stage.repo.join(".git").as_path());
    let id: String = format!("push-{commit}");
    let last_renewal: ClaimTime = ClaimTime::read().rewound(age);
    let claimed: PathBuf = work.join("claimed").join(crate::work_item::claim_file_name(id.as_str(), "0123456789abcdef0123456789abcdef", &last_renewal));
    std::fs::create_dir_all(work.join("claimed")).expect("the claimed directory");
    std::fs::rename(work.join("pending").join(id), claimed.as_path()).expect("claimed by the service that died");
    return claimed;
}

/// Run `work_loop` on a thread as a service that has just started: the plan and the watch are read from the journal, and no wake is sent.
fn start_service_watching<'scope>(scope: &'scope std::thread::Scope<'scope, '_>, handlers: &'scope GitHandlers, plans: &'scope Path, watched: &'scope Path, busy: &'scope AtomicUsize) -> (Sender<Wake>, std::thread::ScopedJoinHandle<'scope, ()>) {
    return start_service_watching_with(scope, handlers, PAUSE, plans, watched, busy);
}

/// The same with the first pause of a push that is tried again given.
fn start_service_watching_with<'scope>(scope: &'scope std::thread::Scope<'scope, '_>, handlers: &'scope GitHandlers, first: Duration, plans: &'scope Path, watched: &'scope Path, busy: &'scope AtomicUsize) -> (Sender<Wake>, std::thread::ScopedJoinHandle<'scope, ()>) {
    let (sender, receiver) = channel::<Wake>();
    let running = scope.spawn(move || { work_loop(&receiver, handlers, busy, PushRetries::restored(first, plans), ClaimWatch::restored(watched, None)); });
    return (sender, running);
}

/// A service killed while it holds a claim is followed by one that runs the claim without any wake: nobody commits after the kill,
///  no wrapper command runs, and the service started in its place looks at the repository an earlier service was woken for, finds
///  the claim held by a holder that does not renew it, and pushes once the lease is over.
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
    assert!(work_directory(stage.repo.join(".git").as_path()).join("done").join(format!("push-{}", stage.landed)).exists(), "settled as done");
    remove(stage.root.as_path());
}

/// The wake of a repository is written down by the worker before it drains, so that a service killed in the middle of the first
///  claim is followed by one that knows where to look; a service that restarts finds the file.
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
///  ago alone: the lease decides that a holder is dead, never the start of a successor, so the work is not run twice.
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
        repository: stage.repo.display().to_string(),
        git_dir: git_dir.display().to_string(),
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
        assert!(wait_until(&|| { return work_directory(git_dir.as_path()).join("done").join(format!("push-{newer}")).exists(); }), "the pending push was settled");
        assert!(claimed.exists(), "the claim of a holder that renewed a moment ago was not taken");
        drop(sender);
        running.join().expect("the service ends");
    });
    remove(stage.root.as_path());
}

/// How far off the first try of a push plan is in the control that has one: far past what the control waits for.
const FAR_PLAN_PAUSE: Duration = Duration::from_secs(600);

/// A repository that has a push plan due much later is still looked at at once when the service starts: the plan is the push's
///  own schedule and must not hide the claim of a dead holder, which runs out its lease long before.
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
        repository: stage.repo.display().to_string(),
        git_dir: git_dir.display().to_string(),
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

/// How long the controls of a fault in the notes wait for the service to try again: the pause, the slack and a margin for a loaded host.
const NOTE_RETRY_PATIENCE: Duration = RESTART_MARGIN.saturating_add(PAUSE).saturating_add(RETRY_SLACK);

/// Puts a file where the notes' directory should be, so that nothing can be written or listed under it, and gives the store's path.
fn block_the_notes(stage: &Stage) -> (PathBuf, PathBuf) {
    let blocker: PathBuf = stage.root.join("note-blocker");
    std::fs::write(&blocker, b"not a directory\n").expect("the blocker");
    return (blocker.clone(), blocker.join(WATCH_DIRECTORY));
}

/// A note that cannot be written when the service first drains a repository does not stop the work, and is written once the fault
/// is mended, without another wake: a service killed after that finds the repository, so a full disk at the first commit does not
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
        let began: Instant = Instant::now();
        while ClaimWatch::restored(store.as_path(), None).directories().is_empty() && began.elapsed() < NOTE_RETRY_PATIENCE {
            std::thread::sleep(LOOK_EVERY);
        }
        assert_eq!(ClaimWatch::restored(store.as_path(), None).directories(), vec![git_dir.clone()], "the note was written by the retry");
        drop(sender);
        running.join().expect("the service ends");
    });
    remove(stage.root.as_path());
}

/// A store of notes that cannot be listed when the service starts does not end the look: the service stays up, lists it again
/// after a pause, and runs the claim of the repository the note names.
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
