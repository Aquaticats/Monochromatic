//! Controls for the deadline-bound Git query:
//! real children that answer,
//! fail,
//! stall or flood.

use super::*;
use crate::test_support::scratch_environment;

/// A shell command running `script`.
#[cfg(unix)]
fn shell(script: &str) -> Command {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg(script);
    return command;
}

/// The query window is the push's stall window,
/// set by the same variable,
/// with no ceiling of its own.
#[test]
fn the_query_window_is_the_push_stall_window() {
    let set = |value: &str| -> Vec<(OsString, OsString)> {
        return vec![(
            OsString::from(crate::bounded_push::PUSH_DEADLINE_VARIABLE),
            OsString::from(value),
        )];
    };
    assert_eq!(query_window(&[]), crate::bounded_push::DEFAULT_PUSH_DEADLINE);
    // A window above what used to be the query ceiling is followed exactly.
    let high: u64 = crate::bounded_push::DEFAULT_PUSH_DEADLINE.as_secs() * 10;
    assert_eq!(query_window(&set(high.to_string().as_str())), Duration::from_secs(high));
}

/// A finished query keeps its exit code and its two streams apart.
#[cfg(unix)]
#[test]
fn a_finished_query_keeps_code_and_streams_apart() {
    let output: QueryOutput = run_bounded_query(
        shell("echo out; echo err >&2; exit 3"),
        "fixture",
        Duration::from_secs(60),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(
        output,
        QueryOutput {
            code: Some(3),
            stdout: b"out\n".to_vec(),
            stderr: b"err\n".to_vec(),
        }
    );
}

/// A query ended by a signal has no code.
#[cfg(unix)]
#[test]
fn a_query_ended_by_a_signal_has_no_code() {
    let output: QueryOutput = run_bounded_query(
        shell("kill -9 $$"),
        "fixture",
        Duration::from_secs(60),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(output.code, None);
}

/// A silent query is stopped when it has shown no sign of work for the window and reported as a stall naming its arguments.
#[cfg(unix)]
#[test]
fn a_stalled_query_is_stopped_and_reported() {
    let started = std::time::Instant::now();
    let failure: SingleFlightError = run_bounded_query(
        shell("exec sleep 8"),
        "rev-parse HEAD",
        Duration::from_millis(300),
        Duration::from_secs(5),
        scratch_environment().as_slice(),
    )
    .expect_err("stalls");
    assert!(started.elapsed() >= Duration::from_millis(300), "{:?}", started.elapsed());
    match &failure {
        SingleFlightError::TimedOut { arguments, seconds } => {
            assert_eq!(arguments, "rev-parse HEAD");
            assert_eq!(*seconds, 0);
        }
        other => panic!("expected a timeout, got {other}"),
    }
    assert!(
        failure
            .to_string()
            .starts_with("git rev-parse HEAD showed no sign of work for 0 seconds and was stopped; CLI_GIT_PUSH_DEADLINE_SECONDS"),
        "{failure}"
    );
}

/// A query that floods either stream is stopped and reported as a Git failure.
#[cfg(unix)]
#[test]
fn a_flooding_query_is_stopped() {
    for script in [
        "head -c 20000000 /dev/zero; exec sleep 8",
        "head -c 20000000 /dev/zero >&2; exec sleep 8",
    ] {
        let failure: SingleFlightError = run_bounded_query(
            shell(script),
            "log",
            Duration::from_secs(60),
            Duration::from_secs(5),
            scratch_environment().as_slice(),
        )
        .expect_err("floods");
        match &failure {
            SingleFlightError::Git { arguments, code, stderr } => {
                assert_eq!(arguments, "log");
                assert_eq!(*code, None);
                assert!(stderr.contains("bytes, so its output is not taken"), "{stderr}");
            }
            other => panic!("expected a Git failure, got {other}"),
        }
    }
}

/// What:
///  A query that prints past the cap and ends before the next look at its files is a flood,
/// never a success.
/// Why:
///   The second pass reads blob contents from this output,
/// so bytes cut at the cap with exit status zero would reach the merge as Git's whole answer.
/// The fixture writes one byte past the cap and ends at once,
/// which is the run the waiter sees end before it can look.
#[cfg(unix)]
#[test]
fn a_query_that_floods_and_ends_is_not_a_success() {
    let failure: SingleFlightError = run_bounded_query(
        shell("exec head -c 4194305 /dev/zero"),
        "cat-file --batch",
        Duration::from_secs(60),
        Duration::from_secs(5),
        scratch_environment().as_slice(),
    )
    .expect_err("more than the cap was printed, so the answer is not whole");
    match &failure {
        SingleFlightError::Git { arguments, code, stderr } => {
            assert_eq!(arguments, "cat-file --batch", "{failure}");
            assert_eq!(*code, None);
            assert!(stderr.contains("bytes, so its output is not taken as Git's answer"), "{stderr}");
        }
        other => panic!("expected a Git failure, got {other}"),
    }
}

/// What:
///  A query whose input thread cannot start is stopped with the same bounded stop as any stopped query.
/// Why:
///   The command never got its request,
/// and the end of a child the kernel will not release is not waited for:
/// the
///  stop asks it to end,
/// forces the end past the grace,
/// and hands what it cannot collect to the sweep.
/// The
///  fixture takes no polite stop,
/// so the grace is what bounds the answer.
#[cfg(unix)]
#[test]
fn a_query_whose_input_cannot_be_given_is_stopped_within_its_grace() {
    let grace: Duration = Duration::from_millis(400);
    let root: std::path::PathBuf = crate::test_support::fixture("query-input-thread-refused");
    let shim: std::path::PathBuf = root.join("git-that-ignores-the-polite-stop");
    std::fs::write(shim.as_path(), "#!/bin/sh\ntrap '' TERM\nexec sleep 30\n").expect("stand-in");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(shim.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    }
    let _refused = crate::scoped_start::RefusedStarts::here();
    let began: std::time::Instant = std::time::Instant::now();
    let failure: SingleFlightError = run_bounded_git(
        Command::new(shim.as_path()),
        "hash-object --stdin",
        Duration::from_secs(60),
        grace,
        scratch_environment().as_slice(),
        Some(b"x".to_vec()),
    )
    .expect_err("the command never got its request");
    let waited: Duration = began.elapsed();
    assert!(failure.to_string().contains("no thread could be started"), "{failure}");
    assert!(waited >= grace, "the command was asked to end and given its grace before the kill: {waited:?}");
    assert!(waited < Duration::from_secs(10), "the stop is bounded, not a wait for the child: {waited:?}");
    crate::test_support::remove(root.as_path());
}

/// A program that cannot start is a start failure.
#[test]
fn a_program_that_cannot_start_is_a_start_failure() {
    let failure: SingleFlightError = run_bounded_query(
        Command::new("/nonexistent/cli-git-query-fixture"),
        "x",
        Duration::from_secs(5),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect_err("no such program");
    assert!(matches!(failure, SingleFlightError::Start(_)), "{failure}");
}

/// A query that keeps writing is never stopped however long it runs:
/// the window judges quiet time,
/// not wall time.
///  The fixture writes a line every pause for a multiple of the window,
/// with no busy loop.
#[cfg(unix)]
#[test]
fn a_query_that_keeps_working_is_never_stopped_however_long_it_runs() {
    let window: Duration = Duration::from_millis(1200);
    let started = std::time::Instant::now();
    let output: QueryOutput = run_bounded_query(
        shell("n=0; while [ $n -lt 15 ]; do echo line; sleep 0.2; n=$((n + 1)); done"),
        "log",
        window,
        Duration::from_secs(5),
        scratch_environment().as_slice(),
    )
    .expect("a query that keeps working is never stopped");
    assert_eq!(output.code, Some(0));
    assert!(started.elapsed() > window * 2, "the fixture ran only {:?}", started.elapsed());
}

/// A query on a host whose `/proc/<pid>/io` cannot be read still ends:
/// the window it is given is then the whole time it may run,
/// and without that fallback nothing at all would end it.
/// The seam is the clock the shipped query builds,
/// made unmeasured as such a host makes it.
#[test]
fn a_query_where_nothing_is_measured_ends_at_its_window() {
    let window: Duration = Duration::ZERO;
    let mut query: Clock = query_clock(0, window).unmeasured();
    assert!(matches!(query.check(), Some(StopCause::Deadline)), "the query's clock ends the run at its window");
    // The control: the same zero window ends nothing without the fallback, so the window above is live.
    let mut unjudged: Clock = query_clock(0, window).unmeasured().without_wall_fallback();
    assert!(unjudged.check().is_none(), "the same window ends nothing when the run may not be ended by wall time");
}
