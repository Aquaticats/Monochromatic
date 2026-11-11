//! Controls for the lock files a stopped Git child must not leave behind.
//! Why:
//!  A Git child stopped at its deadline while it holds a ref lock leaves `<ref>.lock` behind when it cannot clean up
//!  after itself (it was killed,
//! or never ran its signal handler),
//! and every later Git command that updates the same ref
//!  then fails with "Unable to create ... File exists".
//!  Each control holds a real lock with real Git in a disposable repository,
//! stops the holder at its deadline,
//! and
//!  then runs the Git command that failed in practice (`git fetch`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runBoundedPush(holderOfRefLock, { deadlineMs: 1500 });
//! expect(lockFilesUnder(gitDir)).toEqual([]); await git(['fetch', 'origin', 'main']);
//! ```
#![cfg(unix)]

use crate::bounded_push::{run_bounded_push, Patience, STOPPED_EXIT_CODE};
use crate::child_watch::{output_watched, WatchTerms};
use crate::process_tree::TreeMember;
use crate::push_limit_held_for_an_ordering::{LimitHeldForAnOrdering, LimitHoldGuard, hold_limit_for_an_ordering};
use crate::stop_waits_while_a_member_cannot_be_interrupted::{ControlRules, RulesGuard, STOP_CEILING, rules_for_this_thread};
use crate::test_support::{fixture, git, git_text, remove, repository, scratch_environment, while_test_lives};
use std::cell::Cell;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The deadline of the held lock's command:
/// long enough for Git to start and take the lock,
/// short enough to wait for.
/// It is held until a look of the control sees Git's lock and the pid file beside it
/// (`the_holder_holds_its_lock`),
/// so it says how long the run goes on after the lock is held,
/// never when the lock must be held.
const HOLD_DEADLINE: Duration = Duration::from_millis(2500);

/// The grace after the polite stop request:
/// the unresponsive holder never answers it,
/// and the responsive one needs the
///  host to run its handler,
/// which a loaded gate does not do within a few hundred milliseconds.
const SHORT_GRACE: Duration = Duration::from_secs(2);

/// The grace of a stop whose holder answers the request:
/// Git's handler removes the pid file of the lock and then the lock,
/// 130
///  microseconds apart on a quiet disk (`strace` of `git update-ref --stdin` told to prepare,
/// then sent `SIGTERM`,
/// 2026-11-07),
/// and
///  a kill between the two leaves the lock with no pid file,
/// which the sweep rightly leaves alone as unproven.
/// A check and a rename
///  on this host once took 6.03 s of wall time under load (whether the disk or the scheduler held the thread was not measured) (`doc/planning/git-policy-cli-full-tokio.md`,
/// "The coordinator's three settlements",
///  item c),
/// so the handler of a responsive holder is given twice that;
/// a holder that answers ends at once and never waits it out.
///  The gate of `85e6d2ab99` left such a lock after the 2 s grace of `SHORT_GRACE` while the host load was about 70.
const RESPONSIVE_GRACE: Duration = Duration::from_micros(6_031_986).saturating_mul(2);

/// The tracking ref the incident left locked.
const HELD_REF: &str = "refs/remotes/origin/main";

/// Whether the holder can act on `SIGTERM`.
#[derive(Clone, Copy, Debug)]
enum Holder {
    /// Git runs its own signal handler and removes its lock files.
    Responsive,
    /// `SIGTERM` stays blocked in every process of the command,
    /// as for a Git that sits in an uninterruptible call.
    Unresponsive,
    /// Git answers `SIGTERM`,
    /// but its handler is held for `CLEANUP_DELAY_MICROSECONDS` after its first `unlink` (the lock's pid file),
    ///  as a handler is held by a stalled disk or a processor that is not given to it:
    /// a `SIGKILL` at the end of the grace then
    ///  falls between the removal of the pid file and the removal of the lock.
    SlowToFinishCleanup,
    /// As `SlowToFinishCleanup`,
    /// held for twice `STOP_GRACE`:
    /// the watched stop's grace is that constant,
    /// and the kill at its end falls
    ///  inside the hold.
    HeldPastTheStopGrace,
}

/// How long the handler of the slow holder is held after its first `unlink`,
/// in microseconds:
/// more than twice `SHORT_GRACE`,
/// so the
///  kill at the end of the grace always falls inside it.
const CLEANUP_DELAY_MICROSECONDS: u64 = 5_000_000;

/// How long a control holds the run's limit back for the holder's announcement of its lock,
/// before it lets the limit pass and its own assertion reports that the announcement never came.
/// Why:
///  A control that held the limit back forever would wait for the run's watchdog,
/// and one that held it not at all
///  would bet that the holder is quick (`doc/troubleshooting/git-policy-cli-stop-lock-controls-wait-on-order-not-time.md`).
///  The owner's "3x worst case" is what this package waits for anything that is only slow (`STOP_CEILING`):
/// it lies above
///  the longest announcement measured here (2.9 s for a `strace`-wrapped Git under eleven controls sharing a tenth of a
/// processor,
/// 2026-11-08)
/// and below the watchdog's 120 s.
const ANNOUNCEMENT_HOLD: Duration = STOP_CEILING;

/// What one control watches on disk,
/// and what its looks saw.
/// Why:
///  The stop's request is decided by the run's limit and its kill by the stop's grace,
/// and both are elapsed
///  time;
/// the holder takes its lock and Git's handler walks between its two unlinks each after a delay the host
///  sets,
/// never one the control can bound.
/// Measured here on 2026-11-08:
/// a `strace`-wrapped Git took
///  2.9 s to take its lock against the 2.5 s limit,
/// so the run was stopped before the lock the control was about
///  to check ever existed.
///  The control therefore reads Git's own two files,
/// the lock and the pid file beside it,
/// never the word of
///  a shell the stop kills:
/// the pid file is what the stop's own claim reads before it signals (`LockLedger::claiming`).
struct Watched {
    /// The lock of `HELD_REF`.
    lock: PathBuf,
    /// The pid file Git writes beside it under `core.lockfilePid`.
    pid_file: PathBuf,
    /// Set by a look that saw the lock and its pid file:
    /// the holder really held the lock.
    held_seen: Cell<bool>,
    /// Set by a look that saw the lock with its pid file gone:
    /// Git's handler has removed the first and not yet the second.
    cut_seen: Cell<bool>,
}

impl Watched {
    fn of(stage: &Stage) -> Self {
        let git_directory: PathBuf = stage.repo.join(".git");
        return Self {
            lock: git_directory.join(format!("{HELD_REF}.lock")),
            pid_file: git_directory.join(format!("{HELD_REF}~pid.lock")),
            held_seen: Cell::new(false),
            cut_seen: Cell::new(false),
        };
    }

    /// What the two files look like now,
    /// for the words of a failure.
    fn describe(&self) -> String {
        return format!(
            "lock {} present={}, pid file {} present={}",
            self.lock.display(),
            self.lock.exists(),
            self.pid_file.display(),
            self.pid_file.exists()
        );
    }
}

thread_local! {
    /// What the control of this thread watches;
    /// nothing outside a control.
    static WATCHED: std::cell::RefCell<Option<Watched>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Whether Git holds the ref's lock now:
/// the lock and the pid file beside it are both there.
/// Why:
///   This is the ordering the stop's own claim reads before it signals,
/// so a stop asked before it can prove nothing
///  about a lock that is not there yet.
/// The look is recorded,
/// so a control can report afterwards that the ordering it
///  needed happened.
fn the_holder_holds_its_lock() -> bool {
    return WATCHED.with(|slot| {
        let watching = slot.borrow();
        let Some(watched) = watching.as_ref() else {
            return false;
        };
        let held: bool = watched.lock.exists() && watched.pid_file.exists();
        if held {
            watched.held_seen.set(true);
        }
        return held;
    });
}

/// Whether a look of this control saw the holder holding its lock.
fn saw_the_holder_hold_the_lock() -> bool {
    return WATCHED.with(|slot| { return slot.borrow().as_ref().is_some_and(|watched| { return watched.held_seen.get(); }); });
}

/// What the watched files look like now,
/// for the words of a failure.
fn describe_the_holder() -> String {
    return WATCHED.with(|slot| { return slot.borrow().as_ref().map(Watched::describe).unwrap_or_else(|| { return String::from("nothing is watched on this thread"); }); });
}

/// Whether a look of this control saw Git between its two unlinks,
/// the ordering both slow-cleanup controls are about.
fn saw_the_cut() -> bool {
    return WATCHED.with(|slot| { return slot.borrow().as_ref().is_some_and(|watched| { return watched.cut_seen.get(); }); });
}

/// Keep the stop waiting while Git has not begun its cleanup,
/// so the kill that ends the wait falls at the cut.
/// Records the cut,
/// because a kill there is the whole arrangement of the control that uses this rule.
fn wait_while_git_has_not_begun_its_cleanup(_member: &TreeMember) -> bool {
    return WATCHED.with(|slot| {
        let watching = slot.borrow();
        let Some(watched) = watching.as_ref() else {
            return false;
        };
        let between: bool = watched.lock.exists() && !watched.pid_file.exists();
        if between {
            watched.cut_seen.set(true);
        }
        // The pid file's removal is what the wait is for, so it also ends the wait.
        return watched.pid_file.exists() && watched.lock.exists();
    });
}

/// Keep the stop waiting while Git has not finished its cleanup,
/// so the lock goes with its pid file and no kill lands inside the cleanup.
/// Records the cut while it lasts.
fn wait_while_git_has_not_finished_its_cleanup(_member: &TreeMember) -> bool {
    return WATCHED.with(|slot| {
        let watching = slot.borrow();
        let Some(watched) = watching.as_ref() else {
            return false;
        };
        let between: bool = watched.lock.exists() && !watched.pid_file.exists();
        if between {
            watched.cut_seen.set(true);
        }
        return watched.lock.exists();
    });
}

/// What a control expects the stop to wait for past its grace.
/// Why:
///  The production rule waits past the grace only while a member is in uninterruptible sleep (`D`),
/// and a control
///  cannot make a `D`:
/// the fixture's `strace` holds the handler in the traced stop (`t`) while it delays the first
///  `unlink`,
/// which is what keeps Git's cleanup between its two unlinks (130 µs apart on a quiet disk).
/// A
///  control cannot make the production rule see a state it does not look for,
/// so the slow-cleanup controls wait by
///  the cut on disk instead,
/// which is the same ordering named by two files rather than by a letter.
#[derive(Clone, Copy)]
enum StopRule {
    /// The production rule alone,
    /// which proves nothing about the cut here.
    TheProductionRule,
    /// Wait while Git has not begun its cleanup,
    /// so the kill that ends the wait lands at the cut.
    UntilTheCutIsReached,
    /// Wait while Git has not finished its cleanup,
    /// so no kill lands inside it.
    UntilTheCleanupIsFinished,
}

impl StopRule {
    /// The rule the stop consults past its grace,
    /// or nothing where the production one stays.
    fn blocked(self) -> Option<fn(&TreeMember) -> bool> {
        return match self {
            StopRule::TheProductionRule => None,
            StopRule::UntilTheCutIsReached => Some(wait_while_git_has_not_begun_its_cleanup),
            StopRule::UntilTheCleanupIsFinished => Some(wait_while_git_has_not_finished_its_cleanup),
        };
    }

    /// Whether the control arranged the run so that the cut happens,
    /// and must therefore report whether it saw it.
    fn expects_the_cut(self) -> bool {
        return !matches!(self, StopRule::TheProductionRule);
    }
}

/// What one bounded control puts in force for its run:
/// the look at the holder's files,
/// the run's limit held until the holder announced its lock,
/// and the stop's rule where the control replaces the production one.
/// Every part ends when this value drops.
struct Watching {
    /// Ends the look at the files of this thread;
    /// held for its `Drop` and read by nothing.
    _watched: WatchedClear,
    /// Ends the hold of the run's limit;
    /// held for its `Drop` and read by nothing.
    _limit: LimitHoldGuard,
    /// Ends the stop's own rule where the control gave one;
    /// held for its `Drop` and read by nothing.
    _rules: Option<RulesGuard>,
}

impl Watching {
    fn of(stage: &Stage, rule: StopRule) -> Self {
        let watched: Watched = Watched::of(stage);
        let clear: WatchedClear = WATCHED.with(|slot| {
            *slot.borrow_mut() = Some(watched);
            return WatchedClear;
        });
        let limit: LimitHoldGuard = hold_limit_for_an_ordering(LimitHeldForAnOrdering { reached: the_holder_holds_its_lock, ceiling: ANNOUNCEMENT_HOLD });
        let rules: Option<RulesGuard> = rule.blocked().map(|blocked| { return rules_for_this_thread(ControlRules { blocked, ceiling: STOP_CEILING }); });
        return Self { _watched: clear, _limit: limit, _rules: rules };
    }
}

/// Ends the look at the files when dropped.
struct WatchedClear;

impl Drop for WatchedClear {
    fn drop(&mut self) {
        WATCHED.with(|slot| { *slot.borrow_mut() = None; });
    }
}

/// A repository whose `origin/main` is behind a bare remote's `main`,
/// so a later fetch has a tracking ref to update.
struct Stage {
    /// The fixture directory,
    /// removed at the end.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// The ready marker the holder writes once the lock exists.
    ready: PathBuf,
}

/// Every member is held where a signal cannot act,
/// for ever.
fn held_for_ever(_member: &TreeMember) -> bool {
    return true;
}

/// Build the stage.
fn stage(name: &str) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["push", "--quiet", "origin", "main"]);
    let behind: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message", "second"]);
    git(repo.as_path(), &["push", "--quiet", "origin", "main"]);
    git(repo.as_path(), &["update-ref", HELD_REF, behind.as_str()]);
    let ready: PathBuf = root.join("ready");
    return Stage { root, repo, ready };
}

/// The command that holds the lock of `HELD_REF`:
/// `git update-ref --stdin` told to prepare,
/// then kept waiting.
fn holder(stage: &Stage, kind: Holder) -> Command {
    let lock: PathBuf = stage.repo.join(".git").join(format!("{HELD_REF}.lock"));
    let target: String = git_text(stage.repo.as_path(), &["rev-parse", "HEAD"]);
    // The wait for the lock ends with the test, and the hold after it must keep the pipe open from a session the stop under
    //  test cannot reach: `git update-ref --stdin` aborts its prepared transaction and unlinks its own lock when its input
    //  ends, so a writer that dies with the holder's tree would let Git clean up the lock the control looks for
    //  (`a_stall_stop_of_an_unresponsive_holder_leaves_the_lock_it_cannot_prove`, measured 2026-11-11).
    let lives: String = while_test_lives(stage.root.as_path());
    let watcher: PathBuf = stage.root.join("hold-the-pipe-open.sh");
    crate::test_support::executable(watcher.as_path(), format!("#!/bin/sh\nwhile {lives}; do sleep 0.1; done\n").as_bytes());
    // `strace` holds the first `unlink` of Git after it ran: the handler removes the pid file first and the lock second.
    let git: String = match kind {
        Holder::SlowToFinishCleanup => format!("strace -f -o /dev/null -e trace=unlink -e inject=unlink:delay_exit={CLEANUP_DELAY_MICROSECONDS}:when=1 /usr/bin/git"),
        Holder::HeldPastTheStopGrace => format!("strace -f -o /dev/null -e trace=unlink -e inject=unlink:delay_exit={}:when=1 /usr/bin/git", 2 * crate::bounded_push::STOP_GRACE.as_micros()),
        Holder::Responsive | Holder::Unresponsive => String::from("/usr/bin/git"),
    };
    // The watcher keeps the pipe to Git's input on its standard output, which is what holds the transaction open, and it is
    //  started in a session of its own and left behind by the subshell: the stop under test ends the tree it can see, so a
    //  writer in that tree would be ended with Git, and Git reads its end of input, aborts its prepared transaction and unlinks
    //  the very lock the control looks for (measured 2026-11-11). Every other stream is closed, since a reader of this command's
    //  output waits for its pipe's end.
    let script: String = format!(
        "( printf 'start\\nupdate %s %s\\nprepare\\n' \"$HELD_REF\" \"$TARGET\"; \
         while [ ! -e \"$LOCK\" ] && {lives}; do sleep 0.05; done; : > \"$READY\"; setsid \"$WATCHER\" 2>/dev/null </dev/null & ) \
         | {git} update-ref --stdin"
    );
    let mut command: Command = Command::new("sh");
    command
        .arg("-c")
        .arg(script)
        .current_dir(stage.repo.as_path())
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.lockfilePid")
        .env("GIT_CONFIG_VALUE_0", "true")
        .env("HELD_REF", HELD_REF)
        .env("TARGET", target)
        .env("LOCK", lock.as_os_str())
        .env("WATCHER", watcher.as_os_str())
        .env("READY", stage.ready.as_os_str());
    if matches!(kind, Holder::Unresponsive) {
        // SAFETY: only async-signal-safe calls (`sigemptyset`, `sigaddset`, `sigprocmask`) run between fork and exec.
        unsafe {
            command.pre_exec(|| {
                let mut set: libc::sigset_t = std::mem::zeroed();
                libc::sigemptyset(&mut set);
                libc::sigaddset(&mut set, libc::SIGTERM);
                libc::sigprocmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
                return Ok(());
            });
        }
    }
    return command;
}

/// Every file under `directory` whose name ends in `.lock`.
fn lock_files(directory: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![directory.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(next.as_path()).expect("read the directory").flatten() {
            let path: PathBuf = entry.path();
            if entry.file_type().expect("file type").is_dir() {
                pending.push(path);
            } else if path.to_string_lossy().ends_with(".lock") {
                found.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// How long a test lets the thread that sweeps for a tree the host was slow to end take,
/// instead of looking once.
const SWEEP_WAIT: Duration = Duration::from_secs(20);

/// Each leftover file with its size,
/// birth and whether a pid file sits beside it,
/// so a failure on a loaded host says what
/// was left.
fn describe_left(files: &[PathBuf]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for file in files {
        let meta = std::fs::symlink_metadata(file);
        let size: String = meta.as_ref().map_or(String::from("gone"), |metadata| { return metadata.len().to_string(); });
        let born: String = meta.as_ref().ok().and_then(|metadata| { return metadata.created().ok(); }).map_or(String::from("unknown"), |time| { return format!("{time:?}"); });
        let name: String = file.to_string_lossy().into_owned();
        let sibling: String = format!("{}~pid.lock", name.trim_end_matches(".lock"));
        lines.push(format!("{name} size={size} born={born} pid_file_beside={}", Path::new(sibling.as_str()).exists()));
    }
    return lines.join("; ");
}

/// What a stop leaves on disk.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Leftover {
    /// Git's own handler removed the lock and its pid file.
    TheHolderCleansItUp,
    /// The holder was killed before it could remove them,
    /// and nothing the wrapper sees proves the lock's holder gone,
    /// so the lock stays and the next command prints the command that clears it.
    NothingProvesIt,
}

/// The checks that follow a stop,
/// according to what the stop leaves:
/// a holder that cleaned up leaves nothing,
/// and one that was killed leaves the lock the wrapper will not remove.
fn assert_after_stop(stage: &Stage, kind: Holder, how: &str, leftover: Leftover) {
    if leftover == Leftover::NothingProvesIt {
        // The sweep runs before the stop returns, and it removes nothing it cannot prove:
        // the lock the killed holder left is still there for the next command to name.
        let left: Vec<PathBuf> = lock_files(stage.repo.join(".git").as_path());
        assert!(!left.is_empty(), "{how} {kind:?}: no lock was left, so this control no longer tests a lock nothing proves gone");
        return;
    }
    let waited: Instant = Instant::now();
    let mut left: Vec<PathBuf> = lock_files(stage.repo.join(".git").as_path());
    while !left.is_empty() && waited.elapsed() < SWEEP_WAIT {
        std::thread::sleep(Duration::from_millis(50));
        left = lock_files(stage.repo.join(".git").as_path());
    }
    assert!(left.is_empty(), "{how} {kind:?}: a stopped Git child left lock files: {}", describe_left(left.as_slice()));
    let fetched = std::process::Command::new("/usr/bin/git")
        .current_dir(stage.repo.as_path())
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["fetch", "--quiet", "origin", "main"])
        .output()
        .expect("start git fetch");
    assert!(fetched.status.success(), "{how} {kind:?}: the following fetch failed: {}", String::from_utf8_lossy(&fetched.stderr));
}

/// A push-style bounded run stops the holder at its run's limit.
fn bounded_run(kind: Holder, name: &str, leftover: Leftover) {
    bounded_run_with(kind, name, StopRule::TheProductionRule, leftover);
}

/// What:
///  A push-style bounded run stops the holder at its run's limit and leaves no lock,
/// while `rule` says what the stop
///  waits for past its grace.
/// Why:
///   The run's limit is held until a look sees the lock and its pid file,
/// `the_holder_holds_its_lock`,
/// so the
///  stop is asked after the lock is held however slow the host is;
/// the report then names the ordering the control
///  needed and did not see,
/// never a watchdog.
fn bounded_run_with(kind: Holder, name: &str, rule: StopRule, leftover: Leftover) {
    let stage: Stage = stage(name);
    let grace: Duration = match kind {
        Holder::Responsive => RESPONSIVE_GRACE,
        Holder::Unresponsive | Holder::SlowToFinishCleanup | Holder::HeldPastTheStopGrace => SHORT_GRACE,
    };
    let watching: Watching = Watching::of(&stage, rule);
    let attempt = run_bounded_push(holder(&stage, kind), Patience::Fixed(HOLD_DEADLINE), grace, scratch_environment().as_slice()).expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "the holder is stopped at its limit");
    assert_the_holder_held_its_lock(kind, "bounded run");
    if rule.expects_the_cut() {
        assert!(saw_the_cut(), "{kind:?}: the stop ended without a look seeing Git between its two unlinks, so the ordering this control is about never happened");
    }
    drop(watching);
    assert_after_stop(&stage, kind, "bounded run", leftover);
    remove(stage.root.as_path());
}

/// What:
///  Assert that a look of this control saw Git holding the ref's lock before the stop was asked,
/// which is what makes
///  the stop and the cut it makes worth judging.
/// Why:
///   The limit that asks for the stop is held for that very ordering (`ANNOUNCEMENT_HOLD`),
/// so a failure here says
///  the hold ran out and names what was on disk,
/// instead of the misleading "the holder took the lock before it was
///  stopped" of a control that bet the holder is quick.
fn assert_the_holder_held_its_lock(kind: Holder, how: &str) {
    assert!(
        saw_the_holder_hold_the_lock(),
        "{how} {kind:?}: no look saw the lock and its pid file within the hold of {ANNOUNCEMENT_HOLD:?}, so the stop was asked before the holder held the lock: {}",
        describe_the_holder()
    );
}

/// What:
///  Assert the fixture's own word that it held the lock:
/// the holder writes `ready` once it sees its lock.
/// Why:
///   The steps whose stop is decided by signs of work,
/// a watched step and a push stopped on no progress,
///  cannot be stopped while the step is still working,
/// so their stop is after the lock whatever the host does,
/// and the word
///  is enough.
/// A step stopped by a fixed limit is not:
/// its limit can pass before the holder reaches its lock,
///  which is why those controls read Git's own files instead (`the_holder_holds_its_lock`).
fn assert_the_holder_announced_its_lock(stage: &Stage, kind: Holder, how: &str) {
    assert!(stage.ready.exists(), "{how} {kind:?}: the holder never announced that it held the lock, so the stop was asked before the lock existed");
}

/// A watched step stops the holder at its bound.
fn watched_step(kind: Holder, name: &str, leftover: Leftover) {
    let stage: Stage = stage(name);
    let mut command: Command = holder(&stage, kind);
    let terms: WatchTerms = WatchTerms {
        limit: HOLD_DEADLINE,
        cadence: crate::wait_progress::Cadence { grace: HOLD_DEADLINE, interval: HOLD_DEADLINE },
        notices: Box::new(std::io::sink()),
    };
    let ended = output_watched(&mut command, "git update-ref --stdin", terms);
    assert_eq!(ended.expect_err("the step is stopped at its bound").kind(), std::io::ErrorKind::TimedOut);
    assert_the_holder_announced_its_lock(&stage, kind, "watched step");
    assert_after_stop(&stage, kind, "watched step", leftover);
    remove(stage.root.as_path());
}

/// A push stopped on no progress kills a holder that cannot answer,
/// and the lock it held stays:
/// nothing the wrapper sees proves whose it was.
#[test]
fn a_stall_stop_of_an_unresponsive_holder_leaves_the_lock_it_cannot_prove() {
    let stage: Stage = stage("stop-lock-push-stall");
    let patience: Patience = Patience::Progress { stall: HOLD_DEADLINE };
    let attempt = run_bounded_push(holder(&stage, Holder::Unresponsive), patience, SHORT_GRACE, scratch_environment().as_slice()).expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "the holder is stopped on no progress");
    assert_the_holder_announced_its_lock(&stage, Holder::Unresponsive, "stalled run");
    assert_after_stop(&stage, Holder::Unresponsive, "stalled run", Leftover::NothingProvesIt);
    remove(stage.root.as_path());
}

/// A holder that answers `SIGTERM` is cleaned up by Git itself in the push path.
#[test]
fn a_push_stop_of_a_responsive_holder_leaves_no_lock() {
    bounded_run(Holder::Responsive, "stop-lock-push-responsive", Leftover::TheHolderCleansItUp);
}

/// A holder whose handler is slow to finish is not killed between Git's two unlinks:
/// the stop waits,
/// up to its ceiling,
/// while Git's cleanup has not finished,
/// so the lock goes with its pid file.
/// Before the wait the kill at the end of the grace left the
///  lock without its pid file and the sweep left it alone,
/// because nothing proves its holder gone (the gate of `85e6d2ab99`;
/// plan,
///  "The two gate failures of main at `85e6d2ab99`").
/// The wait is decided by the lock on disk and not by the grace:
/// a grace shorter than the
///  host's delay in running Git's handler would kill before the handler began and leave both files
///  (`doc/troubleshooting/git-policy-cli-stop-lock-controls-wait-on-order-not-time.md`).
#[test]
fn a_push_stop_of_a_holder_slow_to_finish_its_cleanup_leaves_no_lock() {
    bounded_run_with(Holder::SlowToFinishCleanup, "stop-lock-push-slow-cleanup", StopRule::UntilTheCleanupIsFinished, Leftover::TheHolderCleansItUp);
}

/// A kill that falls between Git's two unlinks leaves the lock,
/// whatever the load,
/// because the pid file it took with it was the only thing that named the holder.
/// Why:
///  Git's handler removes the pid file of a lock and then the lock;
/// a stop whose kill arrives between the two leaves the lock with
///  no pid file,
/// and the sweep,
/// which had only that pid file to name the holder,
/// left it as unproven (the container gate of 2026-11-07
///  left `refs/remotes/origin/main.lock` after 29.9 s while the host was loaded).
/// The stop that waits while a member cannot be
///  interrupted only makes the cut rarer.
/// The holder here is held after its first unlink by a traced stop,
/// which the production rule does not
///  count as a wait,
/// so the production stop kills at its grace;
/// the control's own rule keeps that stop waiting until
///  the cut on disk begins,
/// so the kill falls between the two unlinks however long the host takes to run
///  Git's handler.
/// A number in a pid file is not the wrapper's own observation,
/// so the lock stays and is named.
#[test]
fn a_kill_between_the_pid_file_and_the_lock_leaves_the_lock_it_cannot_prove() {
    bounded_run_with(Holder::SlowToFinishCleanup, "stop-lock-push-cut-cleanup", StopRule::UntilTheCutIsReached, Leftover::NothingProvesIt);
}

/// The same cut in the watched path leaves the lock too.
#[test]
fn a_watched_kill_between_the_pid_file_and_the_lock_leaves_the_lock_it_cannot_prove() {
    watched_step(Holder::HeldPastTheStopGrace, "stop-lock-watched-cut-cleanup", Leftover::NothingProvesIt);
}

/// A member that never leaves the state where a signal cannot act is still ended at the stop's ceiling:
/// the stop returns about one
///  ceiling after the request,
/// the holder is gone,
/// and the lock it left stays for the person,
/// so nothing waits for good.
#[test]
fn a_member_held_for_ever_is_ended_at_the_ceiling_and_its_lock_is_left() {
    const CEILING: Duration = Duration::from_millis(1500);
    const GRACE: Duration = Duration::from_millis(300);
    // The hold the run's limit is kept back for is part of what the stop's whole time may hold, and nothing else is.
    const GENEROUS_SLACK: Duration = ANNOUNCEMENT_HOLD;
    let stage: Stage = stage("stop-lock-push-held-for-ever");
    let watching: Watching = Watching::of(&stage, StopRule::TheProductionRule);
    let _rules = rules_for_this_thread(ControlRules { blocked: held_for_ever, ceiling: CEILING });
    let began: Instant = Instant::now();
    let attempt = run_bounded_push(holder(&stage, Holder::Unresponsive), Patience::Fixed(HOLD_DEADLINE), GRACE, scratch_environment().as_slice()).expect("runs");
    let took: Duration = began.elapsed();
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "the holder is stopped at its limit");
    assert!(took >= HOLD_DEADLINE + CEILING, "the stop forced the end after {took:?}, before the ceiling of {CEILING:?} that followed the request");
    assert!(took < HOLD_DEADLINE + CEILING + GENEROUS_SLACK, "the stop took {took:?}: it waited past its ceiling");
    assert_the_holder_held_its_lock(Holder::Unresponsive, "stalled run held for ever");
    drop(watching);
    assert_after_stop(&stage, Holder::Unresponsive, "bounded run held for ever", Leftover::NothingProvesIt);
    remove(stage.root.as_path());
}

/// A holder that cannot answer `SIGTERM` is killed,
/// and the lock it held stays.
#[test]
fn a_push_stop_of_an_unresponsive_holder_leaves_the_lock_it_cannot_prove() {
    bounded_run(Holder::Unresponsive, "stop-lock-push-unresponsive", Leftover::NothingProvesIt);
}

/// A holder that answers `SIGTERM` gets the chance in the watched path.
#[test]
fn a_watched_stop_of_a_responsive_holder_leaves_no_lock() {
    watched_step(Holder::Responsive, "stop-lock-watched-responsive", Leftover::TheHolderCleansItUp);
}

/// A holder that cannot answer `SIGTERM` is killed in the watched path,
/// and the lock it held stays.
#[test]
fn a_watched_stop_of_an_unresponsive_holder_leaves_the_lock_it_cannot_prove() {
    watched_step(Holder::Unresponsive, "stop-lock-watched-unresponsive", Leftover::NothingProvesIt);
}

/// The watched path asks with a catchable signal before it kills:
/// a shell with a `TERM` trap runs the trap.
///  The shell sits silent in a bounded `sleep` it waits for (a loop that starts a process every pause would show work at every
///  look,
/// and a step that works is never stopped),
/// so the quiet window is what stops it.
#[test]
fn a_watched_stop_gives_a_catchable_signal_first() {
    let root: PathBuf = fixture("stop-lock-watched-catchable");
    let marker: PathBuf = root.join("trapped");
    let mut command: Command = Command::new("sh");
    command
        .arg("-c")
        .arg("trap ': > \"$MARKER\"; exit 0' TERM; sleep 30 & wait")
        .env("MARKER", marker.as_os_str());
    let terms: WatchTerms = WatchTerms {
        limit: Duration::from_millis(500),
        cadence: crate::wait_progress::Cadence { grace: Duration::from_millis(500), interval: Duration::from_millis(500) },
        notices: Box::new(std::io::sink()),
    };
    let ended = output_watched(&mut command, "a trapping shell", terms);
    assert_eq!(ended.expect_err("stopped for being quiet").kind(), std::io::ErrorKind::TimedOut);
    assert!(marker.exists(), "the stop reached the TERM trap before any kill");
    remove(root.as_path());
}

/// A stop with no grace kills at once and hands the child to a thread,
/// which sweeps its locks when it has ended;
/// the lock stays,
/// because nothing proves its holder gone.
#[test]
fn a_stop_without_grace_sweeps_on_a_thread_of_its_own() {
    let stage: Stage = stage("stop-lock-push-no-grace");
    let _watching: Watching = Watching::of(&stage, StopRule::TheProductionRule);
    let attempt = run_bounded_push(holder(&stage, Holder::Unresponsive), Patience::Fixed(HOLD_DEADLINE), Duration::ZERO, scratch_environment().as_slice()).expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "the holder is stopped at its limit");
    assert_the_holder_held_its_lock(Holder::Unresponsive, "bounded run without grace");
    assert_after_stop(&stage, Holder::Unresponsive, "bounded run without grace", Leftover::NothingProvesIt);
    remove(stage.root.as_path());
}
