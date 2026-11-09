//! What:
//!  Await a future and get a panic it raised during one of its polls as an error value,
//! not as an unwind that ends the task.
//! Why:
//!  `std::panic::catch_unwind` accepts only a callable with no arguments,
//! and a future is caught only by polling it inside that call;
//! no crate of this package offers that combinator,
//! so this is the one place that writes it.
//! A task of the runtime that panics is joined as a panic
//! (`tokio-1.54.1/src/runtime/task/error.rs`),
//! and the awaited ordered runner ends its whole run on one
//!  (`OrderedRunError::Panicked`):
//! the items of the Cargo repair are independent,
//! so one workspace whose repair panics must be a failure naming that
//!  workspace while every other workspace keeps its result
//! (`package_repair_cargo_concurrent.rs`),
//! as a scoped thread's panic was before the move.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! try { return { ok: await work }; } catch (panic) { return { panic }; }
//! ```

/// The guarded poll that silences the hook's notice of an expected panic.
use crate::panic_notice::while_panics_are_expected;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// What a panic said:
/// its text when it carried a string,
/// else a plain statement that it carried none.
/// Why:
///   The failures this crate reports carry the words of what went wrong,
/// and a payload that is neither a `&str` nor a `String`
///  has no words to carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const panicWords = (payload: any) => typeof payload === 'string' ? payload : 'no message';
/// ```
pub fn panic_words(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    return String::from("no message");
}

/// One poll of the boxed future,
/// named so the guarded call can carry it and its state.
/// Why:
///   The guard takes a function and the state it reads,
/// never a closure;
/// the state is the future and the task's context,
/// and the answer is that poll's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pollTheBoxed = ([future, context]) => future.poll(context);
/// ```
fn poll_the_boxed<'of, 'context, T>((future, context): (&mut Pin<Box<dyn Future<Output = T> + Send + 'of>>, &mut Context<'context>)) -> Poll<T> {
    return future.as_mut().poll(context);
}

/// What:
///  A future whose panic is its value:
/// `Ok` what it gave,
/// `Err` what its panic said.
/// Why:
///   The state of the future after a panic is never read:
/// the value is dropped as soon as the error is produced,
/// which is why the caller may hold a future that
///  updates state across its awaits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CatchingPanics { constructor(readonly work) {} poll(cx) { try { return this.work.poll(cx) } catch (p) { return { panic: panicWords(p) } } } }
/// ```
pub struct CatchingPanics<'of, T> {
    /// The future whose polls are guarded.
    /// Boxed,
    /// so that taking it by `&mut` in the poll needs no unsafe projection.
    work: Pin<Box<dyn Future<Output = T> + Send + 'of>>,
}

impl<T> Future for CatchingPanics<'_, T> {
    type Output = Result<T, String>;

    /// Poll the guarded future once.
    /// `Pin<Box<..>>` is always `Unpin`,
    /// so the whole value may be taken by `&mut` and the box it holds is never moved.
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let caught: std::thread::Result<Poll<T>> = while_panics_are_expected((&mut self.get_mut().work, context), poll_the_boxed::<T>);
        return match caught {
            Ok(Poll::Ready(value)) => Poll::Ready(Ok(value)),
            Ok(Poll::Pending) => Poll::Pending,
            // The future is dropped after a panic in one of its polls, so nothing reads what it had half updated.
            Err(payload) => Poll::Ready(Err(panic_words(payload.as_ref()))),
        };
    }
}

/// What:
///  `work` with the panic of any one of its polls turned into an error value,
/// and the panic hook's notice of it silenced.
/// Why:
///   See the module comment:
/// this is the guarded call of `call_catching_panics` for work that awaits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const caught = callCatchingPanicsAwaiting(work); // Ok(value) or Err(the panic's words)
/// ```
pub fn call_catching_panics_awaiting<'of, T: 'of, W: Future<Output = T> + Send + 'of>(work: W) -> CatchingPanics<'of, T> {
    return CatchingPanics { work: Box::pin(work) };
}

/// Controls for the guarded await stay out of the release executable.
#[cfg(test)]
#[path = "call_catching_panics_awaiting_tests.rs"]
mod tests;
