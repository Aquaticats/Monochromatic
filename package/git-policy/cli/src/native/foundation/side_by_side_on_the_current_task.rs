//! What:
//!  Run the futures a caller lends, beside each other, on the task that awaits them,
//! and give their answers back in the order they were given.
//! Why:
//!  Two of the ways this process runs independent items beside each other need a value that outlives the frame
//! that awaits it: `tokio::task::JoinSet` starts each future as a task of its own,
//! and `side_by_side_on_the_runtime` takes owned state to its jobs (`'static`).
//!  The drain of the service's work lends its handlers to every item it runs,
//! and the items of one work directory are known only when that drain runs,
//! so neither spawning nor `tokio::join!` (which names its futures in the source)
//! can run them.
//!  Nothing here is spawned:
//! one task polls every future in turn,
//! so a future that is waiting for a child process, for a job of the blocking pool or for the network goes on waiting,
//! and the rest of the work runs meanwhile.
//! Polling them in turn is the order `Promise.all` takes,
//! and it needs no lock and no channel:
//! the answers come back in the order the futures were given,
//! whatever order they finish in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answers = await Promise.all(futures);
//! ```

use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

/// What one wave still owes:
/// the answer of a future, once it has one.
/// Why:
///   The answers are collected by index,
/// so a slot must be named before any of them arrives,
/// and a named function stands where a closure would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noAnswerYet = () => undefined;
/// ```
fn no_answer_yet<T>() -> Option<T> {
    return None;
}

/// What:
///  Await every future in `futures` beside each other on this task,
/// and return their answers in the order they were given.
/// Why:
///   Every future is polled in turn with this task's own waker,
/// so whichever future can make progress when the task is woken does,
/// and no future holds the others back while it waits.
/// A future that answered is not polled again,
/// so a future whose answer is a value it produced once cannot be asked for a second one.
/// The call returns when the last future has answered,
/// so the caller waits for the slowest of them,
/// as it does for the slowest of a wave of jobs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = await Promise.all(futures);
/// ```
pub async fn run_all_side_by_side_on_the_current_task<T>(mut futures: Vec<Pin<Box<dyn Future<Output = T> + Send + '_>>>) -> Vec<T> {
    let mut answers: Vec<Option<T>> = std::iter::repeat_with(no_answer_yet).take(futures.len()).collect();
    std::future::poll_fn(|context| {
        let mut waiting: bool = false;
        for (index, future) in futures.iter_mut().enumerate() {
            if answers[index].is_some() {
                continue;
            }
            match future.as_mut().poll(context) {
                Poll::Ready(answer) => answers[index] = Some(answer),
                Poll::Pending => waiting = true,
            }
        }
        // The last future answered on this poll, so there is nothing left to wait for; a poll that leaves one waiting asks for
        //  the wake that may let it answer.
        return if waiting { Poll::Pending } else { Poll::Ready(()) };
    })
    .await;
    let mut done: Vec<T> = Vec::with_capacity(answers.len());
    for answer in answers {
        done.push(answer.expect("every future answered before the wait ended"));
    }
    return done;
}

/// Controls for futures run beside each other on one task stay out of the release executable.
#[cfg(test)]
#[path = "side_by_side_on_the_current_task_tests.rs"]
mod tests;
