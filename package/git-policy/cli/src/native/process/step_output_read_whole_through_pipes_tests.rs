//! Controls for the read of a step's output that goes whole through two pipes:
//!  an end of file that is ready at the same poll as the drain after the step ended is the answer,
//! not a stream
//!  a program was said to hold open.
#![cfg(unix)]

use super::*;
use std::io::Write as _;

/// The answer every read of this file's first control is given,
/// so a whole answer is one the stream closed after writing this.
const THE_ANSWER: &[u8] = b"answer";

/// How many pipes the first control reads:
/// one choice of the select per pipe,
/// so a poll that answered differently would show.
const READS: usize = 64;

/// What:
///  An end of file that is ready at the same poll as the drain after the step ended is the answer.
/// Why:
///  `tokio::select!` polls its branches in a random order unless it is told which comes first,
/// so the drain could win a
///  poll at which every writer had closed and the whole answer was there,
/// and the step was reported as one whose output a
///  program still held open
///  (the fifth review's finding 5).
///  The drain is ready at its first poll here,
/// because its moment is a whole drain and a second in the past,
/// and the pipe holds
///  the whole answer with every write end closed.
///  A read the reactor has not yet woken for answers nothing at that poll,
/// which is why telling the select which branch
///  comes first is not what answers:
/// the answer is the read of the descriptor itself.
///  The read is made of sixty-four pipes,
/// so one that
///  answered nothing by luck would show.
#[cfg(target_os = "linux")]
#[test]
fn end_of_file_wins_over_a_drain_that_is_ready_at_the_same_poll() {
    let answers: Vec<std::io::Result<Vec<u8>>> = crate::lazy_runtime::run_on_runtime(async {
        let mut answers: Vec<std::io::Result<Vec<u8>>> = Vec::new();
        for _read in 0..READS {
            let (reader, mut writer) = std::io::pipe().expect("a pipe");
            writer.write_all(THE_ANSWER).expect("the answer goes into the pipe");
            let pipe: Receiver = Receiver::from_owned_fd(OwnedFd::from(reader)).expect("the read end");
            // The write end closes here, so every writer has closed and the pipe answers a read to its end.
            drop(writer);
            // The moment the step ended is a whole drain and a second in the past, so the drain's own timer is behind this
            //  instant rather than inside the tick it is polled in, and it answers ready at its first poll.
            let ended_at: Instant =
                Instant::now().checked_sub(OUTPUT_DRAIN_AFTER_EXIT + Duration::from_secs(1)).expect("this process started two seconds ago");
            let (_told, ends) = tokio::sync::watch::channel::<Option<Instant>>(Some(ended_at));
            answers.push(read_until_the_step_has_ended(pipe, ends, "a step", None).await);
        }
        return answers;
    })
    .expect("the runtime");
    let mut read: Vec<u8> = Vec::new();
    let mut failures: usize = 0;
    for answer in answers {
        match answer {
            Ok(bytes) => read.extend(bytes),
            Err(error) => {
                failures = failures.saturating_add(1);
                crate::diagnostic_log::warn("stepOutput", format!("a read whose end of file was ready failed: {error}").as_str());
            }
        }
    }
    assert_eq!(failures, 0, "a read whose end of file was ready reported the output as held open {failures} times");
    assert_eq!(read.len(), THE_ANSWER.len() * READS, "every answer of every read came back whole");
}

/// What:
///  A query whose ceiling came,
/// and whose child the kernel would not let end,
/// is reported as left running.
/// Why:
///  The sentence of that ending was built from the text the wait carried,
/// so the structured answer never reached the caller:
/// the
///  wrapper said the query was ended though it had been left running
///  (the fifth review's finding 6).
///  The kernel cannot be made to hold a process here,
/// so the signal of the ceiling is what a control refuses:
/// a child that is
///  not signalled and does not end is the same state from this wait's side.
#[cfg(target_os = "linux")]
#[test]
fn the_ceiling_of_a_query_that_could_not_be_ended_says_it_was_left_running() {
    let runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this control");
    let (failure, pid): (std::io::Error, u32) = runtime.block_on(async {
        let mut command: Command = Command::new("sleep");
        command.arg("600").stdin(Stdio::null());
        let pipes: StepPipes = attach_pipes_awaiting(&mut command).await.expect("the pipes of the query");
        let child: Child = command.spawn().expect("the query starts");
        release_write_ends(&mut command);
        let pid: u32 = child.id();
        // The signal of the ceiling is refused, and the child ends by itself after this control has read the answer.
        crate::child_waited_for_within_a_ceiling::a_control_refuses_the_next_kill();
        let carried: CarriedCeiling = carried_by(Duration::from_millis(100), CeilingWhy::TheQueryIsBounded);
        let failure: std::io::Error = read_until_the_child_ends_awaiting(pipes, child, "a query", Some(carried))
            .await
            .expect_err("the ceiling ended the wait for the query");
        return (failure, pid);
    });
    // The child could not be ended, so this control ends it; the thread that watches it collects its end.
    if crate::child_waited_for_within_a_ceiling::left_running_when_the_ceiling_was_reached(&failure) {
        // SAFETY: `kill` takes two integers and touches no memory; the number is the query this control started.
        let _ended = unsafe { libc::kill(i32::try_from(pid).expect("a process number"), libc::SIGKILL) };
    }
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    assert!(failure.to_string().contains("the longest the wrapper lets one step run"), "the ceiling is what ended the wait: {failure}");
    assert!(
        crate::child_waited_for_within_a_ceiling::left_running_when_the_ceiling_was_reached(&failure),
        "a query the kernel would not let end was reported as one that was ended: {failure}"
    );
}

/// What:
///  A wait that only backs up a watch whose pin proves the process fires after that watch's own ceiling,
/// and a wait that ends the step itself fires at the ceiling.
/// Why:
///   Both deadlines were the step's ceiling,
/// so a watch that was scheduled a moment late lost to its own backstop:
/// the
///  wait then ended the step,
/// said its watch could not end it,
/// and answered with an error that was not the ceiling's
///  (`push/child_watch_tests.rs`,
/// `a_step_whose_watch_is_held_back_is_still_ended_at_its_ceiling`,
/// which shows it).
/// The watch
///  that can end the step needs its ending before the wait takes it,
/// and that is the only watch whose backstop moves:
/// one that
///  can signal nothing and one that never started both leave the whole ending to this wait.
#[test]
fn only_a_watch_that_can_end_the_step_puts_the_backstop_behind_its_ceiling() {
    let ceiling: Duration = Duration::from_secs(7);
    let proven: CarriedCeiling = carried_for(Some(ceiling), TheWatchOfAStep::WatchedAndProven).expect("a step with a ceiling is carried");
    let unproven: CarriedCeiling = carried_for(Some(ceiling), TheWatchOfAStep::WatchedAndUnproven).expect("a step with a ceiling is carried");
    let unwatched: CarriedCeiling = carried_for(Some(ceiling), TheWatchOfAStep::NotWatched).expect("a step with a ceiling is carried");
    assert!(proven.backstop > proven.ceiling, "the wait then reaches its own watch's ceiling at {:?}", proven.backstop);
    assert_eq!(proven.backstop, ceiling.saturating_add(crate::child_watch_running_task::THE_WATCH_HAS_ITS_ENDING));
    assert_eq!(proven.why, None, "a watch that can end the step is what ends it");
    assert_eq!(
        (unproven.backstop, unproven.why),
        (ceiling, Some(CeilingWhy::TheWatchCouldNotEndIt)),
        "a watch that can signal nothing leaves the ending to the wait, at the ceiling itself"
    );
    assert_eq!(
        (unwatched.backstop, unwatched.why),
        (ceiling, Some(CeilingWhy::NoWatcher)),
        "no watch leaves the ending to the wait, at the ceiling itself"
    );
    assert!(carried_for(None, TheWatchOfAStep::WatchedAndProven).is_none(), "a step with no ceiling carries none");
}
