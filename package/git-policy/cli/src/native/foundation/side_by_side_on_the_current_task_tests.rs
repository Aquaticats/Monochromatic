//! Controls for futures run beside each other on the task that awaits them:
//! every future runs,
//! all of them are in flight at once,
//! each answer comes back as its item ends,
//! a panic in one item is that item's answer,
//! their answers keep the order they were given in,
//! and a future that answered is not asked again.

use super::*;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// How many futures these controls run.
const FUTURES: usize = 4;

/// How long a control waits for a wave that never overlaps,
/// so a run that runs the futures one after another fails rather than hangs.
const PATIENCE: Duration = Duration::from_secs(20);

/// A runtime of this thread,
/// which polls every future of a wave.
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

/// The answers of a wave every item of which answered,
/// as the values they gave.
/// Why:
///   The order and the count of a wave's answers are what these controls are about,
/// and a control that panicked is not one of those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const values = answeredAll(answers);
/// ```
fn answered(answers: Vec<AnAnswer<usize>>) -> Vec<usize> {
    let mut values: Vec<usize> = Vec::with_capacity(answers.len());
    for answer in answers {
        match answer {
            AnAnswer::Answered(value) => values.push(value),
            AnAnswer::Stopped(said) => panic!("an item of this control stopped unexpectedly: {said}"),
        }
    }
    return values;
}

/// A future whose work stops unexpectedly,
/// and which an item of a control hands in where an answer of `T` is expected.
/// Why:
///   An `async` block whose only statement panics answers `!`,
/// which does not coerce to the answer type a wave is asked for,
///  so a control names that type here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function panics<T>(said: string): Promise<T> { throw new Error(said) }
/// ```
async fn panics<T>(said: &'static str) -> T {
    panic!("{said}");
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
    let answers: Vec<usize> = answered(waited.expect("every future of the wave was in flight at once"));
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
    let answers: Vec<usize> = answered(runtime().block_on(run_all_side_by_side_on_the_current_task(pauses)));
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
    let answers: Vec<usize> = answered(runtime().block_on(run_all_side_by_side_on_the_current_task(one)));
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
    let answers: Vec<AnAnswer<usize>> = runtime().block_on(run_all_side_by_side_on_the_current_task(Vec::new()));
    assert_eq!(answers, Vec::<AnAnswer<usize>>::new(), "an empty wave answers nothing");
}

/// What:
///  The first item's answer comes back before the last item of its wave has answered,
///  so a caller settles one record while its slow sibling still runs.
/// Why:
///   A wave that answered only at its slowest left a push refused at once recorded, and its notice written,
/// only when the sibling a silent server held for minutes ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [at, answer] = await quickThenSlow.next(); expect(at).toBe(0); expect(waited).toBeLessThan(500)
/// ```
#[test]
fn an_answer_comes_back_as_its_item_ends() {
    let futures: Vec<Pin<Box<dyn Future<Output = usize> + Send + '_>>> = Vec::from([
        Box::pin(answers_after((Duration::from_millis(1), 1))) as Pin<Box<dyn Future<Output = usize> + Send + '_>>,
        Box::pin(answers_after((Duration::from_millis(300), 2))) as Pin<Box<dyn Future<Output = usize> + Send + '_>>,
    ]);
    let (at, answer, waited) = runtime().block_on(async move {
        let mut wave: AnswersAsTheyCome<'_, usize> = AnswersAsTheyCome::starting(futures);
        let began: Instant = Instant::now();
        let (at, answer): (usize, AnAnswer<usize>) = wave.the_next_answer().await.expect("the quick item answers");
        let waited: Duration = began.elapsed();
        wave.the_rest().await;
        return (at, answer, waited);
    });
    assert_eq!(at, 0, "the item that answered is the quick one");
    assert_eq!(answered(vec![answer]), vec![1]);
    assert!(waited < Duration::from_millis(150), "the quick item's answer came back before its slow sibling ended (waited {waited:?})");
}

/// What:
///  A panic in one item is that item's own answer,
///  and the items beside it still answer.
/// Why:
///   A wave polls every item on one task,
/// so a panic used to unwind the whole wave and take the answers its siblings had ready with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = await answersOfWave([panicking(), quick(10)]); expect(answers[0]).toEqual({ stopped: /stopped unexpectedly/ }); expect(answers[1]).toEqual({ answered: 10 })
/// ```
#[test]
fn a_panic_is_one_items_answer_and_its_siblings_still_answer() {
    let futures: Vec<Pin<Box<dyn Future<Output = usize> + Send + '_>>> = Vec::from([
        Box::pin(panics::<usize>("an item of the wave stopped unexpectedly")) as Pin<Box<dyn Future<Output = usize> + Send + '_>>,
        Box::pin(answers_after((Duration::from_millis(1), 10))) as Pin<Box<dyn Future<Output = usize> + Send + '_>>,
    ]);
    let answers: Vec<AnAnswer<usize>> = runtime().block_on(run_all_side_by_side_on_the_current_task(futures));
    assert_eq!(answers.len(), 2);
    match &answers[0] {
        AnAnswer::Stopped(said) => assert!(said.contains("stopped unexpectedly") && said.contains("an item of the wave stopped unexpectedly"), "{said}"),
        AnAnswer::Answered(value) => panic!("the item that panicked answered {value}"),
    }
    match &answers[1] {
        AnAnswer::Answered(value) => assert_eq!(*value, 10, "the sibling's answer is its own"),
        AnAnswer::Stopped(said) => panic!("the sibling that answered was read as stopped: {said}"),
    }
}
