//! What:
//!  Await one item's work and keep a panic in it as that item's answer,
//!  so one item's failure ends that item and not the task that ran it.
//! Why:
//!  The service's work runs the independent items of one work directory in waves,
//! and those waves are polled on one task
//! (`side_by_side_on_the_current_task.rs`):
//! a panic in one item of a wave unwound the whole wave,
//! so the records of the wave stayed claimed,
//! including the one whose work had finished and which had the answer ready,
//! and a panic in the slow lane's look ended that lane,
//! which left the service with no slow work while it lived
//! (an audit of `0.6.1-alpha.12`).
//! What an item that panicked answers is the words the panic left,
//! so its record is settled as failed with what happened and the person is told,
//! while its siblings finish and are settled.
//! Nothing is hidden:
//! the process's own panic hook still prints the panic where it happened,
//! so the journal holds it as well.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = await keepAPanicAwaiting(work); // { ok: true, value } | { ok: false, said }
//! ```

use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

/// What:
///  Await `work`, and answer its answer,
///  or the words a panic in it left.
/// Why:
///   The panic is caught at the poll boundary, so it never reaches the task that awaited the item:
/// the wave, the drain and the lane that ran it go on with their other items.
/// A future that panicked is never polled again,
/// and it is dropped when this call returns.
/// The item's own future must be `Send`,
/// because the wave and the lane that await it are tasks of the runtime.
/// The payload is read by its own type, so the text `panic!` was given is the text the record carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await keepAPanicAwaiting(work);
/// ```
pub(crate) async fn keep_a_panic_awaiting<T, F: Future<Output = T> + Send>(work: F) -> Result<T, String> {
    let mut the_work: Pin<Box<F>> = Box::pin(work);
    return std::future::poll_fn(move |context| {
        return match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { return the_work.as_mut().poll(context); })) {
            Ok(Poll::Ready(answer)) => Poll::Ready(Ok(answer)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(payload) => Poll::Ready(Err(what_a_panic_said(payload))),
        };
    })
    .await;
}

/// What:
///  The words a panic left,
///  as the line that reports one item's failure says them.
/// Why:
///   A panic payload is the text `panic!` was given,
/// which a control and the shipped code both write as text;
/// a payload that is not text says only that the work stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = whatAPanicSaid(payload);
/// ```
fn what_a_panic_said(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        return format!("its work stopped unexpectedly: {text}");
    }
    if let Some(text) = payload.downcast_ref::<String>() {
        return format!("its work stopped unexpectedly: {text}");
    }
    return String::from("its work stopped unexpectedly, and it left no words");
}

/// Controls for a panic kept as one item's answer stay out of the release executable.
#[cfg(test)]
#[path = "a_panic_that_ends_one_item_tests.rs"]
mod tests;
