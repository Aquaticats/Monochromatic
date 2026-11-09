//! Controls for the read of a step's output that goes whole through two pipes:
//!  an end of file that is ready at the same poll as the drain after the step ended is the answer,
//! not a stream
//!  a program was said to hold open.
#![cfg(unix)]

use super::*;
use std::io::Write as _;

/// What:
///  An end of file that is ready at the same poll as the drain after the step ended is the answer.
/// Why:
///  `tokio::select!` polls its branches in a random order unless it is told which comes first,
/// so the drain could win a
///  poll at which every writer had closed and the whole answer was there,
/// and the step was reported as one whose output a
///  program still held open
///  (the fifth review's finding 5).
///  Both branches are ready at their first poll here:
/// the drain's moment is a second in the past,
///  and a pipe whose
///  write end is closed answers a read to its end at once.
///  The choice is repeated, so a random order has to show.
#[cfg(target_os = "linux")]
#[test]
fn end_of_file_wins_over_a_drain_that_is_ready_at_the_same_poll() {
    let answers: Vec<std::io::Result<Vec<u8>>> = crate::lazy_runtime::run_on_runtime(async {
        let mut answers: Vec<std::io::Result<Vec<u8>>> = Vec::new();
        for _attempt in 0..64 {
            let (reader, mut writer) = std::io::pipe().expect("a pipe");
            writer.write_all(b"answer").expect("the answer goes into the pipe");
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
    assert_eq!(read.len(), 6 * 64, "every answer of every read came back whole");
}
