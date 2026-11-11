//! Controls for futures run beside each other on the task that awaits them:
//! every future runs,
//! all of them are in flight at once,
//! their answers keep the order they were given in,
//! and a future that answered is not asked again.

use super::*;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// How many futures these controls run.
const FUTURES: usize = 4;

/// How long a control waits for a wave that never overlaps,
/// so a run that runs the futures one after another fails rather than hangs.
const PATIENCE: Duration = Duration::from_secs(20);

/// A runtime of this thread, which polls every future of a wave.
fn runtime() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
}

/// One future of a wave:
/// it joins the others at a barrier of all of them,
/// so it answers only when every future is in flight at the same time.
async fn joins_the_wave((barrier, amount): (Arc<tokio::sync::Barrier>, usize)) -> usize {
    let _everyone_here: tokio::sync::BarrierWaitResult = barrier.wait().await;
    return amount * 10;
}

/// One future that answers after its own pause,
/// so the answers arrive in an order the wave does not start in.
async fn answers_after((pause, amount): (Duration, usize)) -> usize {
    tokio::time::sleep(pause).await;
    return amount;
}

/// A future that counts how many polled it,
/// and answers with that count once it is polled more than once.
struct Polled {
    /// How many times this future was polled.
    polls: Arc<AtomicUsize>,
}

impl Future for Polled {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, context: &mut std::task::Context<'_>) -> Poll<usize> {
        let polls: usize = self.polls.fetch_add(1, Ordering::SeqCst) + 1;
        if polls == 1 {
            context.waker().wake_by_ref();
            return Poll::Pending;
        }
        return Poll::Ready(polls);
    }
}

/// What:
///  Every future of the wave is in flight at the same time,
/// which is what running them beside each other means.
/// Why:
///   A wave run one future after another reaches the first future's barrier alone and never leaves it,
/// so the control's own patience is what fails it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await runAllSideBySide(futures)).toEqual([0, 10, 20, 30])
/// ```
#[test]
fn every_future_of_the_wave_is_in_flight_at_once() {
    let barrier: Arc<tokio::sync::Barrier> = Arc::new(tokio::sync::Barrier::new(FUTURES));
    let amounts: Vec<Pin<Box<dyn Future<Output = usize> + Send + '_>>> = (0..FUTURES).map(|amount| { return Box::pin(joins_the_wave((Arc::clone(&barrier), amount))) as Pin<Box<dyn Future<Output = usize> + Send + '_>>; }).collect();
    let waited = runtime().block_on(async { return tokio::time::timeout(PATIENCE, run_all_side_by_side_on_the_current_task(amounts)).await; });
    let answers: Vec<usize> = waited.expect("every future of the wave was in flight at once");
    assert_eq!(answers, vec![0, 10, 20, 30], "the answers keep the order the futures were given in");
}

/// What:
///  The answers keep the order the futures were given in,
/// whichever future answers first.
/// Why:
///   A wave's answers are read beside the items that were given,
/// so an answer at another item's place would settle the wrong record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await runAllSideBySide([slow(30), quick(10)])).toEqual([30, 10])
/// ```
#[test]
fn the_answers_keep_the_order_the_futures_were_given_in() {
    let pauses: Vec<Pin<Box<dyn Future<Output = usize> + Send + '_>>> = Vec::from([
        Box::pin(answers_after((Duration::from_millis(60), 1))) as Pin<Box<dyn Future<Output = usize> + Send + '_>>,
        Box::pin(answers_after((Duration::from_millis(1), 2))) as Pin<Box<dyn Future<Output = usize> + Send + '_>>,
        Box::pin(answers_after((Duration::from_millis(30), 3))) as Pin<Box<dyn Future<Output = usize> + Send + '_>>,
    ]);
    let answers: Vec<usize> = runtime().block_on(run_all_side_by_side_on_the_current_task(pauses));
    assert_eq!(answers, vec![1, 2, 3], "the answers keep the order the futures were given in");
}

/// What:
///  A future that answered is not polled again,
/// so an answer that a future produces once is never asked for a second one.
/// Why:
///   The wait polls every slot it has no answer for,
/// and a slot that is filled must leave the loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await runAllSideBySide([polled()])).toEqual([2])
/// ```
#[test]
fn a_future_that_answered_is_not_asked_again() {
    let polls: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let one: Vec<Pin<Box<dyn Future<Output = usize> + Send + '_>>> = Vec::from([
        Box::pin(Polled { polls: Arc::clone(&polls) }) as Pin<Box<dyn Future<Output = usize> + Send + '_>>,
    ]);
    let answers: Vec<usize> = runtime().block_on(run_all_side_by_side_on_the_current_task(one));
    assert_eq!(answers, vec![2], "the future answered on its second poll");
    assert_eq!(polls.load(Ordering::SeqCst), 2, "the future was polled exactly until it answered");
}

/// What:
///  A wave with no future in it answers nothing and waits for nothing.
/// Why:
///   A directory whose batch of one kind is empty asks for no wave,
/// and a wait that never ended would hold the round.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await runAllSideBySide([])).toEqual([])
/// ```
#[test]
fn a_wave_with_no_future_answers_nothing() {
    let answers: Vec<usize> = runtime().block_on(run_all_side_by_side_on_the_current_task(Vec::new()));
    assert_eq!(answers, Vec::<usize>::new(), "an empty wave answers nothing");
}
