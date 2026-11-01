//! Controls for the deadline-bound push runner:
//!  real child processes that finish, fail, stall, ignore the stop request or flood their output,
//!  and the limits derived from the environment.

use super::*;
use crate::test_support::scratch_environment;
use std::ffi::OsStr;

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
    assert_eq!(DEFAULT_PUSH_DEADLINE, Duration::from_secs(600));
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

/// A zero, negative, fractional, empty or non-numeric value never means "no deadline".
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

/// The lock wait outlasts the holder's whole push,
///  its stop grace and the margin,
///  and a huge deadline saturates instead of overflowing.
#[test]
fn the_lock_wait_outlasts_the_holders_whole_push() {
    assert_eq!(lock_wait_limit(&[]), Duration::from_secs(600 + 5 + 60));
    assert_eq!(
        lock_wait_limit(&environment(&[(PUSH_DEADLINE_VARIABLE, "100")])),
        Duration::from_secs(100 + 5 + 60)
    );
    assert_eq!(
        lock_wait_limit(&environment(&[(PUSH_DEADLINE_VARIABLE, "18446744073709551615")])),
        Duration::MAX
    );
}

/// The lock wait variable replaces the derived limit,
///  and an unusable value is ignored.
#[test]
fn the_lock_wait_variable_replaces_the_derived_limit() {
    assert_eq!(
        lock_wait_limit(&environment(&[
            (PUSH_DEADLINE_VARIABLE, "100"),
            (LOCK_WAIT_VARIABLE, "7"),
        ])),
        Duration::from_secs(7)
    );
    for value in ["0", "", "later", "-1"] {
        assert_eq!(
            lock_wait_limit(&environment(&[(LOCK_WAIT_VARIABLE, value)])),
            Duration::from_secs(600 + 5 + 60),
            "{value:?}"
        );
    }
}

/// The low-speed defaults are added when absent and each is left alone when the person set it.
#[test]
fn low_speed_defaults_never_override_the_environment() {
    let mut bare: Command = Command::new("git");
    apply_low_speed_defaults(&mut bare, &[]);
    let added: Vec<(&OsStr, Option<&OsStr>)> = bare.get_envs().collect();
    assert_eq!(
        added,
        vec![
            (
                OsStr::new("GIT_HTTP_LOW_SPEED_LIMIT"),
                Some(OsStr::new("1000"))
            ),
            (
                OsStr::new("GIT_HTTP_LOW_SPEED_TIME"),
                Some(OsStr::new("60"))
            ),
        ]
    );
    let mut limited: Command = Command::new("git");
    apply_low_speed_defaults(
        &mut limited,
        &environment(&[("GIT_HTTP_LOW_SPEED_LIMIT", "1")]),
    );
    let only_time: Vec<(&OsStr, Option<&OsStr>)> = limited.get_envs().collect();
    assert_eq!(
        only_time,
        vec![(
            OsStr::new("GIT_HTTP_LOW_SPEED_TIME"),
            Some(OsStr::new("60"))
        )]
    );
    let mut both: Command = Command::new("git");
    apply_low_speed_defaults(
        &mut both,
        &environment(&[
            ("GIT_HTTP_LOW_SPEED_LIMIT", "1"),
            ("GIT_HTTP_LOW_SPEED_TIME", "2"),
        ]),
    );
    assert_eq!(both.get_envs().count(), 0);
    let mut only_limit: Command = Command::new("git");
    apply_low_speed_defaults(
        &mut only_limit,
        &environment(&[("GIT_HTTP_LOW_SPEED_TIME", "2")]),
    );
    let remaining: Vec<(&OsStr, Option<&OsStr>)> = only_limit.get_envs().collect();
    assert_eq!(
        remaining,
        vec![(
            OsStr::new("GIT_HTTP_LOW_SPEED_LIMIT"),
            Some(OsStr::new("1000"))
        )]
    );
}

/// Exactly one trailing line break goes, of either kind.
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
        Duration::from_secs(60),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(attempt.exit_code, 3);
    assert_eq!(attempt.output, "one\ntwo\nthree");
    let clean: PushAttempt = run_bounded_push(
        shell("echo remote: done"),
        Duration::from_secs(60),
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
        Duration::from_secs(60),
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
    let started: Instant = Instant::now();
    let attempt: PushAttempt = run_bounded_push(
        shell("echo started; exec sleep 8"),
        Duration::from_millis(300),
        Duration::from_secs(10),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    let elapsed: Duration = started.elapsed();
    assert!(elapsed >= Duration::from_millis(300), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(20), "{elapsed:?}");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE);
    assert!(
        attempt.output.starts_with("started\n"),
        "{}",
        attempt.output
    );
    assert!(
        attempt
            .output
            .contains("did not finish within 0 seconds and was stopped"),
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
        Duration::from_millis(200),
        Duration::from_secs(10),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE);
    assert!(
        attempt
            .output
            .starts_with("cli-git: git push did not finish"),
        "{}",
        attempt.output
    );
}

/// A push that ignores the stop request is killed once the grace passes.
#[cfg(unix)]
#[test]
fn a_push_that_ignores_the_stop_request_is_killed_after_the_grace() {
    let started: Instant = Instant::now();
    let attempt: PushAttempt = run_bounded_push(
        shell("trap '' TERM; echo up; exec sleep 8"),
        Duration::from_millis(200),
        Duration::from_millis(400),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    let elapsed: Duration = started.elapsed();
    assert!(elapsed >= Duration::from_millis(600), "{elapsed:?}");
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
        Duration::from_millis(200),
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

/// An output is a flood only when it is larger than the cap.
#[test]
fn an_output_is_a_flood_only_past_the_cap() {
    assert!(!is_flood(MAX_PUSH_OUTPUT - 1));
    assert!(!is_flood(MAX_PUSH_OUTPUT));
    assert!(is_flood(MAX_PUSH_OUTPUT + 1));
}

/// A push that floods its output is stopped,
///  and only the capped part is kept.
#[cfg(unix)]
#[test]
fn a_flooding_push_is_stopped_and_its_output_capped() {
    // The bound below is measured against the constant itself, so the constant's value is pinned here.
    assert_eq!(MAX_PUSH_OUTPUT, 4_194_304);
    let attempt: PushAttempt = run_bounded_push(
        shell("head -c 20000000 /dev/zero; exec sleep 8"),
        Duration::from_secs(60),
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

/// A program that cannot start is a start failure, not a push result.
#[test]
fn a_program_that_cannot_start_is_a_start_failure() {
    let failure: SingleFlightError = run_bounded_push(
        Command::new("/nonexistent/cli-git-push-fixture"),
        Duration::from_secs(5),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect_err("no such program");
    assert!(matches!(failure, SingleFlightError::Start(_)), "{failure}");
}
