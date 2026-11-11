//! Controls for the deadline-bound push runner:
//!  real child processes that finish,
//! fail,
//! stall,
//! ignore the stop request or flood their output,
//!  and the limits derived from the environment.

use super::*;
use crate::process_tree::{TreeMember, member_of};
use std::time::Instant;
use crate::stop_of_a_push_and_sweep_of_its_locks::grace_remains;
use crate::test_support::scratch_environment;
// The window a connected run is stopped by, which the wait line names.
use crate::wait_ceilings::CONNECTED_PUSH_QUIET_WINDOW;

/// An environment holding the given pairs.
fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    let mut built: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in pairs {
        built.push((OsString::from(name), OsString::from(value)));
    }
    return built;
}

/// A shell command running `script`.
#[cfg(unix)]
fn shell(script: &str) -> Command {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg(script);
    return command;
}

/// The deadline is the default without the variable and the variable's whole seconds with it,
///  and the last of repeated entries wins.
#[test]
fn the_deadline_comes_from_the_variable_or_the_default() {
    assert_eq!(push_deadline(&[]), DEFAULT_PUSH_DEADLINE);
    // The default only has to bound the push: a zero default would stop every push at once.
    assert!(DEFAULT_PUSH_DEADLINE > Duration::ZERO);
    assert_eq!(
        push_deadline(&environment(&[(PUSH_DEADLINE_VARIABLE, "45")])),
        Duration::from_secs(45)
    );
    assert_eq!(
        push_deadline(&environment(&[(PUSH_DEADLINE_VARIABLE, " 7 ")])),
        Duration::from_secs(7)
    );
    assert_eq!(
        push_deadline(&environment(&[
            (PUSH_DEADLINE_VARIABLE, "10"),
            ("OTHER", "99"),
            (PUSH_DEADLINE_VARIABLE, "20"),
        ])),
        Duration::from_secs(20)
    );
}

/// A zero,
/// negative,
/// fractional,
/// empty or non-numeric value never means "no deadline".
#[test]
fn an_unusable_deadline_falls_back_to_the_default() {
    for value in ["0", "-5", "1.5", "", "soon", "10s"] {
        assert_eq!(
            push_deadline(&environment(&[(PUSH_DEADLINE_VARIABLE, value)])),
            DEFAULT_PUSH_DEADLINE,
            "{value:?}"
        );
    }
}

/// A value that is not valid text is unusable too.
#[cfg(unix)]
#[test]
fn a_non_text_deadline_falls_back_to_the_default() {
    use std::os::unix::ffi::OsStringExt;
    let pairs: Vec<(OsString, OsString)> = vec![(
        OsString::from(PUSH_DEADLINE_VARIABLE),
        OsString::from_vec(vec![0xff, 0xfe]),
    )];
    assert_eq!(push_deadline(&pairs), DEFAULT_PUSH_DEADLINE);
}

/// A run goes by the file name of its program and its first word that is not an option,
/// with the values of `-c` and `-C`
///  skipped;
/// a program with no such word goes by its name alone.
#[test]
fn a_run_goes_by_its_program_and_its_first_subcommand() {
    let mut push: Command = Command::new("/nonexistent/bin/git");
    push.args(["-c", "core.example=1", "-C", "/nonexistent/repository", "--no-pager", "push", "origin", "main"]);
    assert_eq!(command_label(&push), "git push");
    let mut upload: Command = Command::new("git-lfs");
    upload.args(["push", "--all", "origin"]);
    assert_eq!(command_label(&upload), "git-lfs push");
    let mut bare: Command = Command::new("git");
    bare.args(["--version"]);
    assert_eq!(command_label(&bare), "git");
}

/// Exactly one trailing line break goes,
/// of either kind.
#[test]
fn one_trailing_line_break_is_removed() {
    assert_eq!(strip_one_line_break("a\n"), "a");
    assert_eq!(strip_one_line_break("a\r\n"), "a");
    assert_eq!(strip_one_line_break("a\n\n"), "a\n");
    assert_eq!(strip_one_line_break("a"), "a");
    assert_eq!(strip_one_line_break("a\r"), "a\r");
    assert_eq!(strip_one_line_break(""), "");
}

/// A push that ends in time reports its exit code and both streams in the order written.
#[cfg(unix)]
#[test]
fn a_finished_push_keeps_the_interleaved_output_and_exit_code() {
    let attempt: PushAttempt = run_bounded_push(
        shell("echo one; echo two >&2; echo three; exit 3"),
        Patience::Fixed(Duration::from_secs(60)),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(attempt.exit_code, 3);
    assert_eq!(attempt.output, "one\ntwo\nthree");
    let clean: PushAttempt = run_bounded_push(
        shell("echo remote: done"),
        Patience::Fixed(Duration::from_secs(60)),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(clean.exit_code, 0);
    assert_eq!(clean.output, "remote: done");
}

/// A push ended by a signal has no exit code and reports one,
///  and it is not mistaken for a stopped push.
#[cfg(unix)]
#[test]
fn a_push_ended_by_a_signal_reports_one() {
    let attempt: PushAttempt = run_bounded_push(
        shell("echo before; kill -9 $$"),
        Patience::Fixed(Duration::from_secs(60)),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(attempt.exit_code, 1);
    assert_eq!(attempt.output, "before");
}

/// A stalled push is stopped at its deadline and becomes a failed attempt that says why,
///  keeping what it printed first.
#[cfg(unix)]
#[test]
fn a_stalled_push_is_stopped_at_its_deadline() {
    let deadline: Duration = Duration::from_millis(300);
    let started: Instant = Instant::now();
    let attempt: PushAttempt = run_bounded_push(
        shell("echo started; exec sleep 8"),
        Patience::Fixed(deadline),
        Duration::from_secs(10),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    let elapsed: Duration = started.elapsed();
    assert!(elapsed >= deadline, "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(20), "{elapsed:?}");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE);
    assert!(
        attempt.output.starts_with("started\n"),
        "{}",
        attempt.output
    );
    assert!(
        attempt.output.contains(
            format!(
                "did not finish within {} seconds and was stopped",
                deadline.as_secs()
            )
            .as_str()
        ),
        "{}",
        attempt.output
    );
    assert!(
        attempt.output.contains(PUSH_DEADLINE_VARIABLE),
        "{}",
        attempt.output
    );
}

/// A stalled push that printed nothing reports only the reason.
#[cfg(unix)]
#[test]
fn a_silent_stalled_push_reports_only_the_reason() {
    let attempt: PushAttempt = run_bounded_push(
        shell("exec sleep 8"),
        Patience::Fixed(Duration::from_millis(200)),
        Duration::from_secs(10),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE);
    assert!(
        attempt
            .output
            .starts_with("monochromatic-git-policy-cli: git push did not finish"),
        "{}",
        attempt.output
    );
}

/// A push that ignores the stop request is killed once the grace passes.
#[cfg(unix)]
#[test]
fn a_push_that_ignores_the_stop_request_is_killed_after_the_grace() {
    let deadline: Duration = Duration::from_millis(200);
    let grace: Duration = Duration::from_millis(400);
    let started: Instant = Instant::now();
    let attempt: PushAttempt = run_bounded_push(
        shell("trap '' TERM; echo up; exec sleep 8"),
        Patience::Fixed(deadline),
        grace,
        scratch_environment().as_slice(),
    )
    .expect("runs");
    let elapsed: Duration = started.elapsed();
    // The claim is the lower bound: the stop request is asked, and the kill comes only once the grace it was given has passed.
    assert!(elapsed >= deadline + grace, "{elapsed:?}");
    // The ceiling is a liveness guard and not a claim about a host: nothing the child does after it stops answering the request
    //  is a sign of progress this control could read. Measured on this host at a load average of 47: the attempt took 0.66 s for
    //  a deadline and grace of 0.6 s, so the 20 s ceiling carries 30 times the figure it was measured at.
    assert!(elapsed < Duration::from_secs(20), "{elapsed:?}");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE);
    assert!(attempt.output.starts_with("up\n"), "{}", attempt.output);
}

/// A push that honors the stop request is asked first:
///  it sees the request and ends by itself before the grace is used up.
#[cfg(unix)]
#[test]
fn a_push_that_honors_the_stop_request_ends_within_the_grace() {
    let attempt: PushAttempt = run_bounded_push(
        shell("trap 'echo got-the-request; exit 0' TERM; echo up; while :; do sleep 0.05; done"),
        Patience::Fixed(Duration::from_millis(200)),
        Duration::from_secs(3),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE);
    assert!(
        attempt.output.contains("got-the-request\n"),
        "{}",
        attempt.output
    );
}

/// The grace is over exactly when the time since the request reaches it.
#[test]
fn the_grace_is_over_at_its_length_exactly() {
    let grace: Duration = Duration::from_millis(400);
    assert!(grace_remains(Duration::from_millis(399), grace));
    assert!(!grace_remains(grace, grace));
    assert!(!grace_remains(Duration::from_millis(401), grace));
}

/// An output is a flood only when it is larger than the cap it was given,
/// so a caller that reads a larger answer gets it.
#[test]
fn an_output_is_a_flood_only_past_the_cap() {
    assert!(!is_flood(MAX_PUSH_OUTPUT - 1, MAX_PUSH_OUTPUT));
    assert!(!is_flood(MAX_PUSH_OUTPUT, MAX_PUSH_OUTPUT));
    assert!(is_flood(MAX_PUSH_OUTPUT + 1, MAX_PUSH_OUTPUT));
    let larger: u64 = MAX_PUSH_OUTPUT * 4;
    assert!(!is_flood(MAX_PUSH_OUTPUT + 1, larger), "a ceiling the caller raised is the one that stops the output");
    assert!(is_flood(larger + 1, larger));
}

/// A push that floods its output is stopped,
///  and only the capped part is kept.
#[cfg(unix)]
#[test]
fn a_flooding_push_is_stopped_and_its_output_capped() {
    // The producer writes twice the cap, so the case holds for any cap the constant takes.
    let flood: String = format!("head -c {} /dev/zero; exec sleep 8", MAX_PUSH_OUTPUT * 2);
    let attempt: PushAttempt = run_bounded_push(
        shell(flood.as_str()),
        Patience::Fixed(Duration::from_secs(60)),
        Duration::from_secs(10),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE);
    assert!(
        attempt.output.contains("bytes of output and was stopped. Hint: if a hook or the remote prints this much"),
        "{}",
        &attempt.output[attempt.output.len().saturating_sub(120)..]
    );
    assert!(
        attempt.output.len() < 2 * MAX_PUSH_OUTPUT as usize,
        "{}",
        attempt.output.len()
    );
}

/// A program that cannot start is a start failure,
/// not a push result.
#[test]
fn a_program_that_cannot_start_is_a_start_failure() {
    let failure: SingleFlightError = run_bounded_push(
        Command::new("/nonexistent/cli-git-push-fixture"),
        Patience::Fixed(Duration::from_secs(5)),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect_err("no such program");
    assert!(matches!(failure, SingleFlightError::Start(_)), "{failure}");
}

/// A child the kernel releases only after the push was reported is swept on a thread of its own when it ends,
/// and the lock it left stays,
/// because the pid file beside it names a number and neither that number nor the birth order of the two files proves
/// whose writer made it
/// (cross-vendor review of 2026-11-09,
/// finding 3;
/// `Proof::PidGone` is removed).
/// What replaced the removal is the notice:
/// the sweep names the lock,
/// its size and age,
/// and the `rm --` command that clears it with its pid file,
/// and the next push that meets it prints that line before Git refuses.
#[cfg(unix)]
#[test]
fn a_child_released_late_is_swept_and_its_lock_stays_named() {
    let root: std::path::PathBuf = crate::test_support::fixture("sweep-after-exit");
    let heads: std::path::PathBuf = root.join(".git").join("refs").join("heads");
    std::fs::create_dir_all(heads.as_path()).expect("ref directory");
    let lock: std::path::PathBuf = heads.join("late.lock");
    // The child takes its lock and writes the pid file Git writes beside it, then keeps running as the same process.
    let mut command: Command = shell(": > \"$LOCK\"; echo \"pid $$\" > \"$PIDFILE\"; exec sleep 1");
    command.current_dir(root.as_path()).env("LOCK", lock.as_os_str()).env("PIDFILE", heads.join("late~pid.lock").as_os_str());
    let locks: LockLedger = LockLedger::before_start(&command).with_judge(crate::stale_ref_lock_killed_git::judge());
    let child: Child = command.spawn().expect("start the child");
    let member: TreeMember = member_of(i32::try_from(child.id()).expect("pid")).expect("a running child");
    let sweeper = sweep_after_exit(child, vec![member], locks).expect("the sweeping task starts");
    crate::lazy_runtime::run_on_runtime(sweeper).expect("the runtime runs the wait").expect("the sweeping task ends");
    assert!(lock.exists(), "the lock the child left stays: no number in a pid file proves its writer gone");
    assert!(heads.join("late~pid.lock").exists(), "its pid file stays with it, so the pre-removal words can quote it");
    // The judgement that this sweep ran is the one the earlier rounds pinned; here the outcome is what the person meets:
    // the lock is still on disk, and the notice of the next command names it with the command that clears it.
    let git_dir: std::path::PathBuf = root.join(".git");
    let judged = crate::stale_ref_locks::clear_stale_ref_locks(std::slice::from_ref(&git_dir), &crate::stale_ref_lock_killed_git::judge());
    assert!(judged.notice().contains("rm -- ") && judged.notice().contains("late.lock"), "the notice names the file and the command: {}", judged.notice());
    crate::test_support::remove(root.as_path());
}

/// How long the control below holds its own thread before it settles the locks,
/// far longer than the grace it gives:
/// a member that
///  lived for a fixed time would have ended by then (the gate of the branch merged with main at `944bdd30b5` read 81.9 s for a control
///  that needs a few milliseconds,
/// and its one-second member had ended),
/// so the member lives until the control lets go of its input.
const HELD_UP_TEST_THREAD: Duration = Duration::from_millis(1500);

/// A tree that has not ended when the grace is over is not swept then,
/// which would leave its lock and never try again:
/// the sweep waits for it on a thread of its own,
/// runs once it has ended,
/// and leaves the lock it left,
/// because the pid file beside it proves nothing about its writer
/// (cross-vendor review of 2026-11-09,
/// finding 3;
/// `Proof::PidGone` is removed).
/// What replaced the removal is the notice that names the lock and the command that clears it.
#[cfg(unix)]
#[test]
fn a_tree_still_running_after_the_grace_is_swept_when_it_ends_and_its_lock_stays() {
    let root: std::path::PathBuf = crate::test_support::fixture("sweep-lingering-member");
    let heads: std::path::PathBuf = root.join(".git").join("refs").join("heads");
    std::fs::create_dir_all(heads.as_path()).expect("ref directory");
    let lock: std::path::PathBuf = heads.join("lingering.lock");
    let mut command: Command = Command::new("true");
    command.current_dir(root.as_path());
    let locks: LockLedger = LockLedger::before_start(&command).with_judge(crate::stale_ref_lock_killed_git::judge());
    let mut ended: Child = command.spawn().expect("start the collected child");
    // `cat` lives until its input is closed, which the control does when it wants the member to end.
    let mut lingering: Child = Command::new("cat").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::null()).spawn().expect("start the member");
    let member: TreeMember = member_of(i32::try_from(lingering.id()).expect("pid")).expect("a running member");
    std::fs::write(lock.as_path(), "").expect("an empty lock the tree left");
    std::fs::write(heads.join("lingering~pid.lock"), format!("pid {}\n", member.pid)).expect("the pid file of the member");
    ended.wait().expect("the child is collected");
    std::thread::sleep(HELD_UP_TEST_THREAD);
    let started = crate::lazy_runtime::run_on_runtime(settle_locks(None, vec![member], locks, Duration::from_millis(100))).expect("the runtime runs the settling");
    assert!(lock.exists(), "a tree that still runs is not swept");
    let sweeper = started.expect("the sweep waits for the tree in a task of its own");
    drop(lingering.stdin.take());
    lingering.wait().expect("the member ends");
    crate::lazy_runtime::run_on_runtime(sweeper).expect("the runtime runs the wait").expect("the sweeping task ends");
    assert!(lock.exists(), "the sweep ran once the tree ended and left the lock: no number in a pid file proves its writer gone");
    assert!(heads.join("lingering~pid.lock").exists(), "its pid file stays with it");
    let git_dir: std::path::PathBuf = root.join(".git");
    let judged = crate::stale_ref_locks::clear_stale_ref_locks(std::slice::from_ref(&git_dir), &crate::stale_ref_lock_killed_git::judge());
    assert!(judged.notice().contains("rm -- ") && judged.notice().contains("lingering.lock"), "the notice names the file and the command: {}", judged.notice());
    crate::test_support::remove(root.as_path());
}

/// What:
///  A push that prints past the cap and ends between two looks keeps its own status,
/// and the attempt says that its
///  output was cut.
/// Why:
///   The output is the log a person reads,
/// not Git's answer,
/// so the attempt is not turned into a stopped push;
/// but the
///  text kept is the head of what was printed,
/// and passing that for all of it is the silence the query caller had.
#[cfg(unix)]
#[test]
fn a_push_that_floods_and_ends_keeps_its_status_and_says_its_output_was_cut() {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg("exec head -c 4194305 /dev/zero");
    let attempt: PushAttempt = run_bounded_push_once(
        command,
        Patience::Progress { stall: Duration::from_secs(60) },
        Duration::from_secs(1),
        crate::test_support::scratch_environment().as_slice(),
    )
    .expect("a push attempt");
    assert_eq!(attempt.exit_code, 0, "the push's own status is kept: {}", attempt.output);
    assert!(attempt.output.contains("only the first"), "the attempt says its output was cut: {}", attempt.output);
    assert!(attempt.output.len() < usize::try_from(MAX_PUSH_OUTPUT).unwrap_or(usize::MAX) + 2048, "the kept output stays bounded");
}

//region The words of a stop name the window that fired

/// What:
///  The line that says a connected run is waiting names the window that will end it.
/// Why:
///   A run whose connection stays open is stopped at the connected window,
///  so a line that says no time limit applies promises a wait the wrapper does not keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(waitLine(label, report)).toContain(`${CONNECTED_PUSH_QUIET_WINDOW}s is stopped as a stall`)
/// ```
#[test]
fn the_wait_line_names_the_connected_window_that_ends_the_wait() {
    let report = WaitReport {
        silent: CONNECTED_PUSH_QUIET_WINDOW,
        elapsed: CONNECTED_PUSH_QUIET_WINDOW * 2,
        window: CONNECTED_PUSH_QUIET_WINDOW,
    };
    let line: String = wait_line("git push", &report, None);
    assert!(line.contains(WAIT_NOTE_PHRASE), "the line names the wait: {line}");
    assert!(!line.contains("no time limit"), "no promise of an unbounded wait: {line}");
    assert!(
        line.contains(&format!("{} seconds is stopped as a stall", CONNECTED_PUSH_QUIET_WINDOW.as_secs())),
        "the connected window that ends it is named: {line}"
    );
}

/// What:
///  A stop by the connected window names that window and does not offer the stall setting;
/// a stop by the stall window keeps the setting as its remedy.
/// Why:
///   `CLI_GIT_PUSH_DEADLINE_SECONDS` sets the stall window only,
///  so offering it after a connected stop sends its reader to a limit that cannot move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(connected).not.toContain('to allow longer'); expect(stalled).toContain('to allow longer');
/// ```
#[test]
fn a_connected_stop_says_its_window_is_fixed_and_a_stall_stop_offers_the_setting() {
    let window: Duration = CONNECTED_PUSH_QUIET_WINDOW;
    let connected: String = stalled_reason(&StopReport { window: StopWindow::Connected, limit: window, moved: 12, elapsed: window }, None);
    assert!(connected.contains("limit=connected"), "the window that fired is named: {connected}");
    assert!(
        connected.contains(&format!("{PUSH_DEADLINE_VARIABLE} does not change this window")),
        "the setting is named as one that does not move this window: {connected}"
    );
    assert!(!connected.contains("to allow longer"), "no remedy that cannot work is offered: {connected}");
    assert!(connected.contains(PUSH_QUEUED_LINE), "the retry that does settle it is named: {connected}");
    let stalled: String = stalled_reason(&StopReport { window: StopWindow::Stall, limit: window, moved: 12, elapsed: window }, None);
    assert!(stalled.contains("limit=stall"), "the window that fired is named: {stalled}");
    assert!(stalled.contains("to allow longer"), "the setting that moves this window is offered: {stalled}");
}

//endregion
