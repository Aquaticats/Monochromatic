//! Controls for the bounded retry of a child start that the operating system refused for lack of processes.

use super::*;
use std::time::Instant;

/// The error a refused start returns: `EAGAIN`, which Rust reports as `WouldBlock`.
fn refusal() -> Error {
    return Error::from_raw_os_error(libc::EAGAIN);
}

/// Run the retry with a recording pause: the result, the pauses it asked for, and how often it tried.
fn drive(
    budget: Duration,
    mut outcome: impl FnMut(u32) -> Result<i32>,
) -> (Result<i32>, Vec<Duration>, u32) {
    let mut pauses: Vec<Duration> = Vec::new();
    let mut tries: u32 = 0;
    let result: Result<i32> = retry_refused_start_with(
        "a test child",
        budget,
        &mut |wait: Duration| { pauses.push(wait); },
        &mut || {
            tries += 1;
            return outcome(tries);
        },
    );
    return (result, pauses, tries);
}

/// The pauses in milliseconds, for readable assertions.
fn millis(pauses: &[Duration]) -> Vec<u128> {
    return pauses.iter().map(Duration::as_millis).collect();
}

/// A start that works the first time never pauses.
#[test]
fn a_start_that_succeeds_at_once_never_pauses() {
    let (result, pauses, tries) = drive(START_RETRY_BUDGET, |_| { return Ok(7); });
    assert_eq!(result.expect("started"), 7);
    assert_eq!(pauses, Vec::<Duration>::new());
    assert_eq!(tries, 1);
}

/// A refused start is tried again after pauses that double from the first one.
#[test]
fn a_refused_start_is_tried_again_after_growing_pauses() {
    let (result, pauses, tries) = drive(START_RETRY_BUDGET, |attempt| {
        return if attempt <= 3 { Err(refusal()) } else { Ok(9) };
    });
    assert_eq!(result.expect("started on the fourth try"), 9);
    assert_eq!(millis(&pauses), vec![10, 20, 40]);
    assert_eq!(tries, 4);
}

/// The pauses double until they reach the longest one and stay there.
#[test]
fn pauses_double_up_to_the_longest_one() {
    let (result, pauses, _) = drive(Duration::from_secs(600), |attempt| {
        return if attempt <= 12 { Err(refusal()) } else { Ok(1) };
    });
    assert!(result.is_ok());
    assert_eq!(millis(&pauses), vec![10, 20, 40, 80, 160, 320, 500, 500, 500, 500, 500, 500]);
}

/// A start refused for longer than the budget fails with the same kind and says how often it was refused.
#[test]
fn a_start_refused_past_the_budget_fails_and_counts_the_refusals() {
    let (result, pauses, tries) = drive(START_RETRY_BUDGET, |_| { return Err(refusal()); });
    let error: Error = result.expect_err("the budget ran out");
    assert_eq!(error.kind(), ErrorKind::WouldBlock);
    let text: String = error.to_string();
    assert!(text.contains("Resource temporarily unavailable"), "{text}");
    assert!(text.contains("refused 15 times over 4630 ms"), "{text}");
    assert_eq!(tries, 15);
    let waited: Duration = pauses.iter().sum();
    assert_eq!(waited, Duration::from_millis(4630));
    assert!(waited <= START_RETRY_BUDGET, "the pauses never pass the budget");
}

/// An error that is not a shortage of processes is the caller's at once, without a pause.
#[test]
fn another_error_is_returned_at_once() {
    let (result, pauses, tries) = drive(START_RETRY_BUDGET, |_| { return Err(Error::from(ErrorKind::NotFound)); });
    assert_eq!(result.expect_err("not retried").kind(), ErrorKind::NotFound);
    assert_eq!(pauses, Vec::<Duration>::new());
    assert_eq!(tries, 1);
}

/// A real command starts through the retrying calls, and a program that does not exist fails at once.
#[test]
fn real_commands_start_through_the_retrying_calls() {
    assert!(Command::new("true").status_retrying().expect("true starts").success());
    let output: Output = Command::new("echo").arg("kept").output_retrying().expect("echo starts");
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()), "kept\n");
    let mut child: Child = Command::new("true").spawn_retrying().expect("true spawns");
    assert!(child.wait().expect("true ends").success());
    let started: Instant = Instant::now();
    let error: Error = Command::new("/nonexistent-cli-git-program").status_retrying().expect_err("no such program");
    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert!(started.elapsed() < Duration::from_secs(1), "a missing program is not retried");
}

/// A failed replacement of the process returns its error, and a missing program is not retried.
#[cfg(unix)]
#[test]
fn a_failed_exec_returns_its_error_at_once() {
    let started: Instant = Instant::now();
    let error: Error = exec_retrying(&mut Command::new("/nonexistent-cli-git-program"));
    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert!(started.elapsed() < Duration::from_secs(1));
}
