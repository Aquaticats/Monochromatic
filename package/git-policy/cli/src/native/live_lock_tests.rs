//! Controls for the live-holder wait: its limit from the environment, its notice text,
//!  and the notice a slow wait prints.

use super::*;
use crate::owner_lock::{
    MonotonicClock, OwnerLock, OwnerLockError, SilentWait, WaitClock, acquire_owner_lock, acquire_within_clock,
};
use crate::test_support::{fixture, remove};
use crate::wait_progress::{Cadence, QueuePlace};
use std::ffi::OsString;

/// The limit follows the variable and falls back to the default for anything unusable.
#[test]
fn the_limit_comes_from_the_variable_or_the_default() {
    assert_eq!(live_wait_limit_from(None), DEFAULT_LIVE_WAIT);
    // The default only has to outlast an ordinary wait: a zero one would give up at once.
    assert!(DEFAULT_LIVE_WAIT > Duration::ZERO);
    assert_eq!(
        live_wait_limit_from(Some(OsStr::new("90"))),
        Duration::from_secs(90)
    );
    for value in ["0", "", "-1", "1.5", "soon"] {
        assert_eq!(
            live_wait_limit_from(Some(OsStr::new(value))),
            DEFAULT_LIVE_WAIT,
            "{value:?}"
        );
    }
}

/// A value that is not valid text is unusable.
#[cfg(unix)]
#[test]
fn a_non_text_limit_falls_back_to_the_default() {
    use std::os::unix::ffi::OsStringExt;
    let value: OsString = OsString::from_vec(vec![0xff, 0xfe]);
    assert_eq!(live_wait_limit_from(Some(value.as_os_str())), DEFAULT_LIVE_WAIT);
}

/// The notice names the holder, the lock, the waited time and the limit,
///  and says where to change it.
#[test]
fn the_notice_names_the_holder_the_lock_and_both_durations() {
    let line: String = slow_wait_line(&LockWaitFacts {
        subject: "the hook lock",
        lock_directory: Path::new("/repo/.git/cli-git/hook.lock"),
        holder_pid: Some(4242),
        waited: Duration::from_millis(5900),
        limit: Duration::from_secs(1800),
        place: None,
        repeat: false,
    });
    assert!(
        line.starts_with(
            "cli-git: still waiting for process 4242 to release the hook lock (/repo/.git/cli-git/hook.lock) \
             after 5 seconds; giving up after 1800 seconds (CLI_GIT_LOCK_WAIT_SECONDS changes this), \
             leaving the lock in place."
        ),
        "{line}"
    );
    assert!(line.contains(" Hint: if you started another Git command"), "{line}");
    let anonymous: String = slow_wait_line(&LockWaitFacts {
        subject: "the hook lock",
        lock_directory: Path::new("/x"),
        holder_pid: None,
        waited: Duration::from_secs(6),
        limit: Duration::from_secs(9),
        place: None,
        repeat: false,
    });
    assert!(
        anonymous.starts_with("cli-git: still waiting for another cli-git process to release"),
        "{anonymous}"
    );
}

/// A repeat drops the hint, and a commit in line is told its number.
#[test]
fn a_repeat_drops_the_hint_and_a_queued_commit_learns_its_place() {
    let facts: LockWaitFacts<'_> = LockWaitFacts {
        subject: "the landing reservation",
        lock_directory: Path::new("/repo/.git/cli-git-transactions/reservation.lock"),
        holder_pid: Some(77),
        waited: Duration::from_secs(25),
        limit: Duration::from_secs(1800),
        place: Some(QueuePlace { position: 2, waiting: 3 }),
        repeat: true,
    };
    assert_eq!(
        slow_wait_line(&facts),
        "cli-git: still waiting for process 77 to release the landing reservation \
         (/repo/.git/cli-git-transactions/reservation.lock) after 25 seconds; giving up after 1800 seconds \
         (CLI_GIT_LOCK_WAIT_SECONDS changes this), leaving the lock in place. \
         This is number 2 of 3 commits in line, so 1 will be served first."
    );
}

/// A slow wait prints one line naming the live holder,
///  and the first busy moment prints nothing.
#[test]
fn a_slow_wait_prints_one_line_naming_the_holder() {
    let root: PathBuf = fixture("live-lock-notice");
    let lock_directory: PathBuf = root.join("hook.lock");
    let held: OwnerLock =
        acquire_owner_lock(&lock_directory, DEFAULT_POLL_DELAY, &mut SilentWait).expect("held");
    let mut sink: Vec<u8> = Vec::new();
    let mut notice: HolderNotice<&mut Vec<u8>> = HolderNotice::new(
        lock_directory.as_path(),
        "the hook lock",
        Duration::from_secs(60),
        &mut sink,
    );
    notice.waiting();
    notice.slow(Duration::from_secs(7));
    drop(notice);
    let text: String = String::from_utf8(sink).expect("text");
    assert_eq!(text.matches('\n').count(), 1, "{text}");
    assert!(
        text.contains(&format!("waiting for process {}", std::process::id())),
        "{text}"
    );
    assert!(text.contains("after 7 seconds"), "{text}");
    assert!(text.contains("giving up after 60 seconds"), "{text}");
    drop(held);
    remove(&root);
}

/// A notice that is told again prints a line each time, and only the first carries the hint.
#[test]
fn a_notice_told_again_prints_a_line_each_time_with_the_hint_once() {
    let root: PathBuf = fixture("live-lock-notice-repeat");
    let lock_directory: PathBuf = root.join("hook.lock");
    let held: OwnerLock =
        acquire_owner_lock(&lock_directory, DEFAULT_POLL_DELAY, &mut SilentWait).expect("held");
    let mut sink: Vec<u8> = Vec::new();
    let mut notice: HolderNotice<&mut Vec<u8>> = HolderNotice::new(
        lock_directory.as_path(),
        "the hook lock",
        Duration::from_secs(60),
        &mut sink,
    );
    notice.slow(Duration::from_secs(1));
    notice.slow(Duration::from_secs(11));
    notice.slow(Duration::from_secs(21));
    drop(notice);
    let text: String = String::from_utf8(sink).expect("text");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert!(lines[0].contains("after 1 seconds") && lines[0].contains("Hint:"), "{text}");
    assert!(lines[1].contains("after 11 seconds") && !lines[1].contains("Hint:"), "{text}");
    assert!(lines[2].contains("after 21 seconds") && !lines[2].contains("Hint:"), "{text}");
    drop(held);
    remove(&root);
}

/// The limit the bounded-wait controls give up at.
const BOUND_LIMIT: Duration = Duration::from_millis(900);

/// The silence before the first line and the time between lines in the bounded-wait controls.
const BOUND_CADENCE: Cadence = Cadence {
    grace: Duration::from_millis(100),
    interval: Duration::from_millis(200),
};

/// The pause between two attempts in the bounded-wait controls:
///  it divides the grace, the interval and the limit, so every notice falls on a pause.
const BOUND_POLL: Duration = Duration::from_millis(50);

/// A clock that moves only when the wait pauses, so no control depends on how the host schedules the process.
///  `stretch` is the least a pause moves it,
///  which stands for a wait that was not scheduled for that long.
struct VirtualClock {
    /// How long the wait has lasted.
    now: Duration,
    /// The least a pause moves the clock.
    stretch: Duration,
    /// How many times the wait paused.
    pauses: u32,
}

impl WaitClock for VirtualClock {
    /// The time the pauses added up to.
    fn elapsed(&self) -> Duration {
        return self.now;
    }

    /// Moves the clock by the pause, or by the stretch when that is longer, without sleeping.
    fn pause(&mut self, delay: Duration) {
        self.now += delay.max(self.stretch);
        self.pauses += 1;
    }
}

/// Wait behind a live holder on a virtual clock and report the lines printed, the refusal and the clock.
fn wait_behind_holder(stretch: Duration, name: &str) -> (Vec<String>, OwnerLockError, VirtualClock) {
    let root: PathBuf = fixture(name);
    let lock_directory: PathBuf = root.join("landing.lock");
    let held: OwnerLock =
        acquire_owner_lock(&lock_directory, DEFAULT_POLL_DELAY, &mut SilentWait).expect("held");
    let mut sink: Vec<u8> = Vec::new();
    let mut clock: VirtualClock = VirtualClock { now: Duration::ZERO, stretch, pauses: 0 };
    let refusal: OwnerLockError = {
        let mut notice: HolderNotice<&mut Vec<u8>> =
            HolderNotice::new(lock_directory.as_path(), "the landing lock", BOUND_LIMIT, &mut sink);
        acquire_within_clock(
            lock_directory.as_path(),
            BOUND_POLL,
            &mut notice,
            BOUND_LIMIT,
            BOUND_CADENCE,
            &mut clock,
        )
        .expect_err("a live holder outlasts the limit")
    };
    drop(held);
    remove(&root);
    let text: String = String::from_utf8(sink).expect("text");
    return (text.lines().map(str::to_owned).collect(), refusal, clock);
}

/// A wait behind a live holder prints the holder's line at the grace and at every interval,
///  then gives up at its limit with a refusal that names the holder and every way forward.
///  The clock is virtual, so the lines are exactly the notices the cadence makes due by the limit.
#[test]
fn a_wait_behind_a_live_holder_repeats_its_line_and_ends_at_the_limit() {
    let (lines, refusal, clock) = wait_behind_holder(Duration::ZERO, "live-lock-bound");
    // One notice at the grace, then one at every interval up to and including the limit.
    let due: usize = 1 + usize::try_from((BOUND_LIMIT - BOUND_CADENCE.grace).as_nanos() / BOUND_CADENCE.interval.as_nanos())
        .expect("a small count");
    assert_eq!(lines.len(), due, "{lines:?}");
    let holder: String = format!("waiting for process {} to release the landing lock", std::process::id());
    for line in &lines {
        assert!(line.contains(holder.as_str()), "{line}");
    }
    assert!(lines[0].contains("Hint:"), "{lines:?}");
    for repeat in &lines[1..] {
        assert!(!repeat.contains("Hint:"), "{repeat}");
    }
    assert_eq!(clock.elapsed(), BOUND_LIMIT, "the wait ends at the limit");
    let OwnerLockError::WaitTimedOut { waited, holder_pid, .. } = &refusal else {
        panic!("expected a timed-out wait, got {refusal}");
    };
    assert_eq!(*waited, BOUND_LIMIT);
    assert_eq!(*holder_pid, Some(i64::from(std::process::id())));
    let reason: String = refusal.to_string();
    assert!(reason.contains(format!("process {} still holds", std::process::id()).as_str()), "{reason}");
    assert!(reason.contains("let that process finish and run the command again"), "{reason}");
    assert!(reason.contains("check with `ps`"), "{reason}");
    assert!(reason.contains("CLI_GIT_LOCK_WAIT_SECONDS"), "{reason}");
}

/// A wait that was not scheduled for a long stretch prints one line for all the notices it missed,
///  never one per missed notice, and still ends at its limit.
///  This is what a starved process looks like, and the reason the line count of a real-time wait cannot be asserted.
#[test]
fn a_wait_that_was_not_scheduled_prints_one_line_for_the_notices_it_missed() {
    let (lines, refusal, clock) = wait_behind_holder(BOUND_LIMIT, "live-lock-stalled");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(clock.pauses, 1, "one stalled pause carried the wait past its limit");
    assert!(matches!(refusal, OwnerLockError::WaitTimedOut { .. }), "{refusal}");
}

/// The clock of real waits counts from its start and sleeps for the pause it is given.
#[test]
fn the_real_clock_sleeps_the_pause_and_counts_it() {
    let mut clock: MonotonicClock = MonotonicClock::start();
    let before: Duration = clock.elapsed();
    clock.pause(BOUND_POLL);
    // A sleep lasts at least its duration, so the elapsed time grew by at least the pause.
    assert!(clock.elapsed() >= before + BOUND_POLL, "{:?}", clock.elapsed());
}

/// A notice for a lock nobody holds still prints, without naming a process.
#[test]
fn a_notice_for_an_unreadable_lock_still_prints() {
    let root: PathBuf = fixture("live-lock-notice-absent");
    let mut sink: Vec<u8> = Vec::new();
    let mut notice: HolderNotice<&mut Vec<u8>> = HolderNotice::new(
        root.join("absent.lock").as_path(),
        "a lock",
        Duration::from_secs(60),
        &mut sink,
    );
    notice.slow(Duration::from_secs(5));
    drop(notice);
    let text: String = String::from_utf8(sink).expect("text");
    assert!(text.contains("another cli-git process"), "{text}");
    remove(&root);
}

/// A free lock is taken at once with the process's own limit and no output.
#[test]
fn a_free_lock_is_taken_without_waiting() {
    let root: PathBuf = fixture("live-lock-free");
    let lock: OwnerLock =
        acquire_live_owner_lock(root.join("free.lock").as_path(), "a lock").expect("free");
    lock.release().expect("release");
    remove(&root);
}

/// Variable the helper process is started with.
const HELPER_EXPECTED: &str = "LIVE_LOCK_TEST_EXPECTED_SECONDS";

/// Helper process body,
///  started by the test below with the limit variable set:
///  the limit this process runs under is the variable's.
#[test]
#[ignore = "started as a helper process by the environment test"]
fn helper_reads_the_limit_from_its_environment() {
    let expected: u64 = std::env::var(HELPER_EXPECTED)
        .expect("expected")
        .parse()
        .expect("seconds");
    assert_eq!(live_wait_limit(), Duration::from_secs(expected));
}

/// Run the helper with an expectation and, when given, the limit variable;
///  whether the helper's assertion held.
fn helper_agrees(value: Option<&str>, expected: u64) -> bool {
    let mut command: std::process::Command =
        std::process::Command::new(crate::test_support::this_test_binary().expect("test binary"));
    command
        .args([
            "--exact",
            "live_lock::tests::helper_reads_the_limit_from_its_environment",
            "--ignored",
            "--test-threads=1",
            "--quiet",
        ])
        .env(HELPER_EXPECTED, expected.to_string())
        .env_remove(LIVE_WAIT_VARIABLE);
    if let Some(text) = value {
        command.env(LIVE_WAIT_VARIABLE, text);
    }
    return command.status().expect("start the helper").success();
}

/// The process environment reaches the limit:
///  a child started with the variable runs under it, and one started without it runs under the default.
///  A wrong expectation must fail the helper, which proves the helper really ran its assertion.
#[test]
fn the_process_environment_sets_the_limit() {
    assert!(!helper_agrees(Some("7"), 8), "the helper must fail a wrong expectation");
    assert!(!helper_agrees(None, 7), "the helper must fail a wrong default");
    for (value, expected) in [(Some("7"), 7_u64), (Some("0"), 1800), (Some("junk"), 1800), (None, 1800)] {
        assert!(helper_agrees(value, expected), "{value:?}");
    }
}
