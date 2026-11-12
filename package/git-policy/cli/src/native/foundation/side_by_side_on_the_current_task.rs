//! What:
//!  Run the futures a caller lends,
//! beside each other,
//! on the task that awaits them,
//! give each answer back as its item ends,
//! and keep a panic in one item as that item's own answer.
//! Why:
//!  Two of the ways this process runs independent items beside each other need a value that outlives the frame
//! that awaits it:
//! `tokio::task::JoinSet` starts each future as a task of its own,
//! and `side_by_side_on_the_runtime` takes owned state to its jobs (`'static`).
//!  The drain of the service's work lends its handlers to every item it runs,
//! and the items of one work directory are known only when that drain runs,
//! so neither spawning nor `tokio::join!` (which names its futures in the source)
//! can run them.
//!  Nothing here is spawned:
//! one task polls every future in turn,
//! so a future that is waiting for a child process,
//! for a job of the blocking pool or for the network goes on waiting,
//! and the rest of the work runs meanwhile.
//! Polling them in turn is the order `Promise.all` takes,
//! and it needs no lock and no channel:
//! the answers come back in the order the futures were given,
//! whatever order they finish in.
//!  An answer is handed over as its item ends, not when the slowest of the wave does:
//! a push refused at once is on record at once,
//! and with a silent server its sibling holds the wave for the push deadline or the connected push's quiet window
//! (`wait_ceilings.rs`),
//! so a wave that settled only at its slowest left a finished push claimed,
//! to be pushed again after its lease,
//! and told the person of a refusal that long after Git knew it.
//!  A panic in one item is kept as that item's answer
//! (`a_panic_that_ends_one_item.rs`),
//! so the items of its wave finish and are settled and the drain goes on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const [at, answer] of answersAsTheyCome(futures)) settle(at, answer);
//! ```

use crate::a_panic_that_ends_one_item::keep_a_panic_awaiting;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

/// What one item of a wave gave back:
/// its answer,
/// or the words a panic in it left.
/// Why:
///   A wave reports what each item came to,
/// and an item whose work stopped did not answer:
/// a caller that has somewhere to put a failure says so with words of its own
/// (`service_work_items_side_by_side.rs` settles the record as failed with them).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AnAnswer<T> = { answered: T } | { stopped: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnAnswer<T> {
    /// The item's work answered.
    Answered(T),
    /// The item's work stopped unexpectedly, and these are the words it left.
    Stopped(String),
}

/// What one wave still owes:
/// the answer of a future,
/// once it has one.
/// Why:
///   The answers are collected by index,
/// so a slot must be named before any of them arrives,
/// and a named function stands where a closure would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noAnswerYet = () => undefined;
/// ```
fn no_answer_yet<T>() -> Option<AnAnswer<T>> {
    return None;
}

/// What:
///  One wave of futures, whose answers are taken one at a time as their items end.
/// Why:
///  The wave keeps the futures and which of them have answered,
/// so a future that answered is never polled again,
/// and the caller settles each item's record while the rest of the wave still runs.
/// A future that panicked answered with the words the panic left and is not polled again either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wave = AnswersAsTheyCome.starting(futures); while (const [at, answer] = await wave.next()) settle(at, answer);
/// ```
pub struct AnswersAsTheyCome<'wave, T> {
    /// The futures still to answer,
    /// each guarded so a panic in it is that item's answer.
    futures: Vec<Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'wave>>>,
    /// Which of them have answered, by the index the caller gave each.
    answered: Vec<bool>,
}

impl<'wave, T: 'wave> AnswersAsTheyCome<'wave, T> {
    /// What:
    ///  A wave of `futures` that has answered nothing yet.
    /// Why:
    ///   Every future is guarded here once,
    /// so the poll below never has to catch a panic and a future that panicked is dropped with the wave.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const wave = AnswersAsTheyCome.starting(futures);
    /// ```
    pub fn starting(futures: Vec<Pin<Box<dyn Future<Output = T> + Send + 'wave>>>) -> AnswersAsTheyCome<'wave, T> {
        let count: usize = futures.len();
        let answered: Vec<bool> = std::iter::repeat_n(false, count).collect();
        let mut guarded: Vec<Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'wave>>> = Vec::with_capacity(count);
        for future in futures {
            guarded.push(Box::pin(keep_a_panic_awaiting(future)));
        }
        return AnswersAsTheyCome { futures: guarded, answered };
    }

    /// What:
    ///  The next item to answer,
    ///  as its index in the wave and what it answered,
    ///  or nothing when every item has answered.
    /// Why:
    ///   The caller settles one record per answer,
    /// so the fastest item is not held behind the slowest of its wave.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const next = await wave.next();
    /// ```
    pub async fn the_next_answer(&mut self) -> Option<(usize, AnAnswer<T>)> {
        return TheNextAnswer { wave: self }.await;
    }

    /// What:
    ///  Let every item that has not answered finish,
    ///  and drop what they answer.
    /// Why:
    ///   A caller whose claim set went with a refused job cannot settle what is left,
    /// and it must not drop a wave while its items are inside their work:
    /// this waits for them and settles nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await wave.theRest();
    /// ```
    pub async fn the_rest(&mut self) {
        while self.the_next_answer().await.is_some() {}
    }
}

/// What:
///  One wait over a wave,
///  whose polls move the futures that have not answered and stop at the first one that does.
/// Why:
///   A task's own wait is this value rather than a closure handed to the runtime:
/// the wave and its futures outlive one poll,
///  so they are reached through a borrow of the wave and are not captures of a call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheNextAnswer { poll(ctx) { /* poll every unanswered future once, stop at the first answer */ } }
/// ```
struct TheNextAnswer<'wave, 'answers, T> {
    /// The wave being waited over.
    wave: &'answers mut AnswersAsTheyCome<'wave, T>,
}

impl<'wave, T: 'wave> Future for TheNextAnswer<'wave, '_, T> {
    type Output = Option<(usize, AnAnswer<T>)>;

    fn poll(self: Pin<&mut Self>, context: &mut std::task::Context<'_>) -> Poll<Option<(usize, AnAnswer<T>)>> {
        let wave: &mut AnswersAsTheyCome<'_, T> = &mut *self.get_mut().wave;
        let mut waiting: bool = false;
        for index in 0..wave.futures.len() {
            if wave.answered[index] {
                continue;
            }
            match wave.futures[index].as_mut().poll(context) {
                Poll::Ready(Ok(answer)) => {
                    wave.answered[index] = true;
                    return Poll::Ready(Some((index, AnAnswer::Answered(answer))));
                }
                // The guard answered for the item: its work stopped, and this is what it left.
                Poll::Ready(Err(said)) => {
                    wave.answered[index] = true;
                    return Poll::Ready(Some((index, AnAnswer::Stopped(said))));
                }
                Poll::Pending => waiting = true,
            }
        }
        // Nothing was left to answer; a wave with items still waiting asks for the wake that may let one answer.
        return if waiting { Poll::Pending } else { Poll::Ready(None) };
    }
}

/// What:
///  Await every future in `futures` beside each other on this task,
/// and return what each answered in the order they were given.
/// Why:
///   Every future is polled in turn with this task's own waker,
/// so whichever future can make progress when the task is woken does,
/// and no future holds the others back while it waits.
/// This is the shape a caller needs when it wants every answer at the end;
/// a caller that settles each item's record as it ends uses `AnswersAsTheyCome` instead.
/// A panic in one item is that item's own answer (`AnAnswer::Stopped`), never the caller's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = await answersOfWave(futures);
/// ```
pub async fn run_all_side_by_side_on_the_current_task<T>(futures: Vec<Pin<Box<dyn Future<Output = T> + Send + '_>>>) -> Vec<AnAnswer<T>> {
    let count: usize = futures.len();
    let mut answers: Vec<Option<AnAnswer<T>>> = std::iter::repeat_with(no_answer_yet).take(count).collect();
    let mut wave: AnswersAsTheyCome<'_, T> = AnswersAsTheyCome::starting(futures);
    while let Some((index, answer)) = wave.the_next_answer().await {
        answers[index] = Some(answer);
    }
    let mut done: Vec<AnAnswer<T>> = Vec::with_capacity(count);
    for answer in answers {
        done.push(answer.expect("every future of the wave answered before the wait ended"));
    }
    return done;
}

/// Controls for futures run beside each other on one task stay out of the release executable.
#[cfg(test)]
#[path = "side_by_side_on_the_current_task_tests.rs"]
mod tests;
