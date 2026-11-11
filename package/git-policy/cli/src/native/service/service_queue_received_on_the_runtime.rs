//! What:
//!  The wait of a thread on a queue the runtime owns:
//! one item,
//! the end of the wait at its bound,
//! or the end of the queue.
//! Why:
//!  The service's two queues (the requests of the landing and the wakes of the after-commit work) are tokio channels,
//! so tasks of the runtime send into them without a thread and the loop that is a task awaits them directly;
//! the loop that is still a thread (its Git steps are synchronous until its callers are `async`) waits on the
//! runtime's own timer and wake instead of a condition variable,
//! and enters the runtime once for that wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const received = await Promise.race([queue.next(), timeout(wait)]); // 'item' | 'timeout' | 'closed'
//! ```

use crate::lazy_runtime::run_on_runtime;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

/// How a wait on a queue ended.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Received<Item> {
    /// The next item.
    Item(Item),
    /// The bound passed with nothing queued.
    Timeout,
    /// Every sender is gone and nothing is queued,
    /// or the runtime could not run the wait.
    Closed,
}

/// An item,
/// or the end of the queue when there is none.
fn from_next<Item>(next: Option<Item>) -> Received<Item> {
    return match next {
        Some(item) => Received::Item(item),
        None => Received::Closed,
    };
}

/// The next item of `receiver`,
/// within `wait` when there is a bound.
async fn next_item<Item>(receiver: &mut UnboundedReceiver<Item>, wait: Option<Duration>) -> Received<Item> {
    let Some(limit) = wait else {
        return from_next(receiver.recv().await);
    };
    return match tokio::time::timeout(limit, receiver.recv()).await {
        Ok(next) => from_next(next),
        Err(_elapsed) => Received::Timeout,
    };
}

/// What:
///  Waits for the next item of `receiver`,
/// for at most `wait` when there is one,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The loop of the after-commit work is a task of the runtime,
/// and a task that asked the bridge a thread asks would be refused there;
///  this face awaits the queue's own wake and the runtime's own timer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const received = wait === undefined ? await next(queue) : await nextWithin(queue, wait);
/// ```
pub(crate) async fn receive_for_at_most_awaiting<Item>(receiver: &mut UnboundedReceiver<Item>, wait: Option<Duration>) -> Received<Item> {
    return next_item(receiver, wait).await;
}

/// What:
///  `receive_for_at_most_awaiting` for a caller outside the runtime,
/// entering it once.
/// Why:
///  The caller is a thread that is not in the runtime;
/// a call from inside it is refused by the bridge and counts as the end of the queue,
/// which ends the loop that asked rather than blocking a worker.
/// The body is the awaited one,
/// so a queue cannot be waited on differently depending on which face asked it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const received = wait === undefined ? await next(queue) : await nextWithin(queue, wait);
/// ```
pub(crate) fn receive_for_at_most<Item>(receiver: &mut UnboundedReceiver<Item>, wait: Option<Duration>) -> Received<Item> {
    return match run_on_runtime(receive_for_at_most_awaiting(receiver, wait)) {
        Ok(received) => received,
        Err(refusal) => {
            crate::service_error!("a queue of the service could not be waited on: {refusal}");
            Received::Closed
        }
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_queue_received_on_the_runtime_tests.rs"]
mod tests;
