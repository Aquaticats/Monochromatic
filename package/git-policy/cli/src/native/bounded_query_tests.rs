//! Controls for the deadline-bound Git query: real children that answer, fail, stall or flood.

use super::*;
use crate::test_support::scratch_environment;

/// A shell command running `script`.
#[cfg(unix)]
fn shell(script: &str) -> Command {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg(script);
    return command;
}

/// The query deadline follows the push deadline and never passes the ceiling.
#[test]
fn the_query_deadline_follows_the_push_deadline_up_to_the_ceiling() {
    let set = |value: &str| -> Vec<(OsString, OsString)> {
        return vec![(
            OsString::from(crate::bounded_push::PUSH_DEADLINE_VARIABLE),
            OsString::from(value),
        )];
    };
    assert_eq!(query_deadline(&[]), QUERY_CEILING);
    assert_eq!(query_deadline(&set("7")), Duration::from_secs(7));
    assert_eq!(query_deadline(&set("120")), Duration::from_secs(120));
    assert_eq!(query_deadline(&set("3600")), QUERY_CEILING);
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

/// A stalled query is stopped at its deadline and reported as a timeout naming its arguments.
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
    assert!(started.elapsed() < Duration::from_secs(7), "{:?}", started.elapsed());
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
            .starts_with("git rev-parse HEAD did not finish within 0 seconds and was stopped; CLI_GIT_PUSH_DEADLINE_SECONDS"),
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
                assert!(stderr.contains("bytes and was stopped"), "{stderr}");
            }
            other => panic!("expected a Git failure, got {other}"),
        }
    }
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
