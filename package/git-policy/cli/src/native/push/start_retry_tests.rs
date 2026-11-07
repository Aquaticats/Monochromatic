//! Controls for the bounded retry of a child start that the operating system refused for lack of processes.

use super::*;

/// A function-like value with the right shape is a start in these controls:
/// the shipped callers write a small struct that holds the state.
impl<T, Start: FnMut() -> Result<T>> RefusableStart<T> for Start {
    fn start(&mut self) -> Result<T> {
        return self();
    }
}

/// The error a refused start returns:
/// `EAGAIN`,
/// which Rust reports as `WouldBlock`.
fn refusal() -> Error {
    return Error::from_raw_os_error(libc::EAGAIN);
}

/// Run the retry with a recording pause:
/// the result,
/// the pauses it asked for,
/// and how often it tried.
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

/// The pauses in milliseconds,
/// for readable assertions.
fn millis(pauses: &[Duration]) -> Vec<u128> {
    return pauses.iter().map(Duration::as_millis).collect();
}

/// Check the retry's pause schedule:
/// one pause per refusal,
/// doubling from the first one and
///  never longer than the longest.
fn assert_growing_pauses(pauses: &[Duration]) {
    let mut expected: Duration = FIRST_PAUSE;
    for (index, pause) in pauses.iter().enumerate() {
        assert_eq!(*pause, expected, "pause {index}: {pauses:?}");
        expected = (expected * 2).min(LONGEST_PAUSE);
    }
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
    assert_eq!(pauses.len(), 3, "{:?}", millis(&pauses));
    assert_growing_pauses(&pauses);
    assert_eq!(tries, 4);
}

/// The pauses double until they reach the longest one and stay there.
#[test]
fn pauses_double_up_to_the_longest_one() {
    let (result, pauses, _) = drive(Duration::from_secs(600), |attempt| {
        return if attempt <= 12 { Err(refusal()) } else { Ok(1) };
    });
    assert!(result.is_ok());
    assert_eq!(pauses.len(), 12, "{:?}", millis(&pauses));
    assert_growing_pauses(&pauses);
    // Twelve pauses end on the longest one: the doubling reached it and stayed.
    assert_eq!(pauses.last(), Some(&LONGEST_PAUSE));
}

/// A start refused for longer than the budget fails with the same kind and says how often it was refused.
#[test]
fn a_start_refused_past_the_budget_fails_and_counts_the_refusals() {
    let (result, pauses, tries) = drive(START_RETRY_BUDGET, |_| { return Err(refusal()); });
    let error: Error = result.expect_err("the budget ran out");
    assert_eq!(error.kind(), ErrorKind::WouldBlock);
    let text: String = error.to_string();
    assert!(text.contains("Resource temporarily unavailable"), "{text}");
    let waited: Duration = pauses.iter().sum();
    // The budget is the sum of the pauses that fit: each doubled pause is added only while the
    //  budget still holds it.
    let mut budgeted: Duration = Duration::ZERO;
    let mut delay: Duration = FIRST_PAUSE;
    while budgeted + delay <= START_RETRY_BUDGET {
        budgeted += delay;
        delay = (delay * 2).min(LONGEST_PAUSE);
    }
    assert_eq!(waited, budgeted, "the wait is the pauses the budget holds");
    // Every pause follows one refusal, and the refused try that finds the budget spent counts too.
    let refusals: u32 = u32::try_from(pauses.len()).expect("pause count") + 1;
    assert_eq!(tries, refusals);
    assert!(
        text.contains(format!("refused {refusals} times over {} ms", waited.as_millis()).as_str()),
        "{text}"
    );
    assert!(waited <= START_RETRY_BUDGET, "the pauses never pass the budget");
}

/// An error that is not a shortage of processes is the caller's at once,
/// without a pause.
#[test]
fn another_error_is_returned_at_once() {
    let (result, pauses, tries) = drive(START_RETRY_BUDGET, |_| { return Err(Error::from(ErrorKind::NotFound)); });
    assert_eq!(result.expect_err("not retried").kind(), ErrorKind::NotFound);
    assert_eq!(pauses, Vec::<Duration>::new());
    assert_eq!(tries, 1);
}

/// A real command starts through the retrying calls,
/// and a program that does not exist fails at once.
#[test]
fn real_commands_start_through_the_retrying_calls() {
    assert!(Command::new("true").status_retrying().expect("true starts").success());
    let output: Output = Command::new("echo").arg("kept").output_retrying().expect("echo starts");
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()), "kept\n");
    let mut child: Child = Command::new("true").spawn_retrying().expect("true spawns");
    assert!(child.wait().expect("true ends").success());
    let error: Error = Command::new("/nonexistent-cli-git-program").status_retrying().expect_err("no such program");
    assert_eq!(error.kind(), ErrorKind::NotFound);
}

/// A failed replacement of the process returns its error,
/// and a missing program is not retried.
#[cfg(unix)]
#[test]
fn a_failed_exec_returns_its_error_at_once() {
    let error: Error = exec_retrying(&mut Command::new("/nonexistent-cli-git-program"));
    assert_eq!(error.kind(), ErrorKind::NotFound);
}

/// The retry run on a current-thread runtime next to a ticking task:
/// how often the ticker ran,
/// how often the start was tried,
/// and the
///  result.
/// A pause that blocks the thread leaves the ticker no turn.
fn drive_awaiting_beside_a_ticker(budget: Duration) -> (Result<i32>, u32, u32) {
    let runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_current_thread().enable_time().build().expect("a current-thread runtime");
    let ticks: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let mut tries: u32 = 0;
    let result: Result<i32> = runtime.block_on(async {
        let ticker = async {
            loop {
                tokio::time::sleep(Duration::from_millis(1)).await;
                ticks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        };
        let mut refused = || {
            tries += 1;
            return Err::<i32, Error>(refusal());
        };
        let retry = retry_refused_start_awaiting_within("a test child", budget, &mut refused);
        let outcome: Result<i32> = tokio::select! {
            result = retry => result,
            () = ticker => unreachable!("the ticker never ends"),
        };
        return outcome;
    });
    return (result, ticks.load(std::sync::atomic::Ordering::SeqCst), tries);
}

/// The pause between two refused starts is the runtime's timer:
/// a task beside the retry runs during every pause,
/// which a pause that
///  sleeps the thread never allows.
#[test]
fn a_pause_between_refused_starts_lets_other_tasks_run() {
    let (result, ticks, tries) = drive_awaiting_beside_a_ticker(Duration::from_millis(100));
    assert_eq!(result.expect_err("the budget ran out").kind(), ErrorKind::WouldBlock);
    // Every pause but none after the last refusal: one fewer pause than tries.
    let pauses: u32 = tries - 1;
    assert!(pauses > 0, "the budget must hold at least one pause for this control to mean anything");
    assert!(ticks >= pauses, "the ticker ran {ticks} times over {pauses} pauses: the pauses held the thread");
}

/// The awaiting retry has the schedule and the final error of the sleeping one for the same refusals.
#[test]
fn the_awaiting_retry_refuses_in_the_same_words_as_the_sleeping_one() {
    let budget: Duration = Duration::from_millis(100);
    let (sleeping, _, _) = drive(budget, |_| { return Err(refusal()); });
    let (awaiting, _, _) = drive_awaiting_beside_a_ticker(budget);
    assert_eq!(awaiting.expect_err("refused").to_string(), sleeping.expect_err("refused").to_string());
}

/// A real command starts through the awaiting call,
/// and a program that does not exist fails at once.
#[test]
fn a_real_command_starts_through_the_awaiting_call() {
    let runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_current_thread().enable_time().build().expect("a current-thread runtime");
    let mut child: Child = runtime.block_on(spawn_retrying_without_blocking(&mut Command::new("true"))).expect("true spawns");
    assert!(child.wait().expect("true ends").success());
    let error: Error = runtime
        .block_on(spawn_retrying_without_blocking(&mut Command::new("/nonexistent-cli-git-program")))
        .expect_err("no such program");
    assert_eq!(error.kind(), ErrorKind::NotFound);
}
