//! The ordered run on the runtime:
//! every item runs,
//! the report keeps input order however the jobs finish,
//! the jobs really overlap up to what the runner promises (its workers,
//! its lanes and the process's bound on blocking jobs),
//! the process's bound on blocking jobs holds them,
//! a system that refuses every thread is a plain error,
//! and a lane set that leaves an item out is refused.

use super::*;
use crate::blocking_jobs_within_the_budget::{budgeted_blocking_jobs, hold_blocking_jobs_to};
use crate::no_threads_child_test_support::ran_in_child;
use crate::parallel_runner::Event;
use crate::parallel_test_support::{PATIENCE, Rendezvous};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Milliseconds one item of the timing controls keeps its job busy.
const BUSY_MILLIS: u64 = 25;

/// How many items the timing controls run.
const ITEMS: usize = 4;

/// How many jobs the controls let run at once when the bound is what they measure.
const ONE_PLACE: usize = 1;

/// How many workers the overlap control asks one run for:
/// more than the two a two-CPU allowance leaves,
/// so the run is asked for more jobs at once than the process's bound grants.
const WORKERS_ASKED: usize = 4;

/// One lane per item,
/// so items may finish in any order.
fn single_lanes(count: usize) -> Arc<[Vec<usize>]> {
    let lanes: Vec<Vec<usize>> = (0..count).map(lane_of_one_item).collect();
    return Arc::from(lanes);
}

/// The lane of an item that must not wait for another:
/// itself alone.
fn lane_of_one_item(index: usize) -> Vec<usize> {
    return vec![index];
}

/// What:
///  The most jobs of one run that can be inside their work at once,
/// from what the runner promises its callers,
/// for positive worker and lane counts.
/// Why:
///  The runner runs its items under the process's bound on blocking jobs
/// (`parallel_runner_on_the_runtime.rs`),
/// and every job takes a place of that bound before it starts,
/// so a run never has more than the fewest of the workers it is asked for,
/// its lanes and `budgeted_blocking_jobs()` in flight;
/// a control that expects a typed four fails wherever the bound is two.
fn jobs_that_may_overlap(workers: usize, lanes: usize) -> usize {
    return workers.min(lanes).min(budgeted_blocking_jobs());
}

/// A run whose items all end with their index times ten.
fn run_ending(items: Vec<u64>, workers: usize, work: Arc<dyn OrderedWork<u64, u64> + Send + Sync>, on_event: &mut dyn OrderedEvents<u64>) -> Result<(), OrderedRunError> {
    let count: usize = items.len();
    return run_ordered_and_wait_here(OrderedRunOfOwned { items: Arc::from(items), lanes: single_lanes(count), workers, work, on_event });
}

/// Items that sleep for the time they are given say one line and return ten times their index.
struct SleepAndSay;

impl OrderedWork<u64, u64> for SleepAndSay {
    fn run(&self, index: usize, item: &u64, say: &mut dyn LineSink) -> u64 {
        std::thread::sleep(Duration::from_millis(*item));
        say.say(format!("item {index}").as_str());
        return index as u64 * 10;
    }
}

/// Every event,
/// as text,
/// in the order the consumer saw them.
#[derive(Default)]
struct SeenText {
    /// The events as text.
    seen: Vec<String>,
}

impl OrderedEvents<u64> for SeenText {
    fn on_event(&mut self, event: Event<'_, u64>) {
        self.seen.push(match event {
            Event::Begin(index) => format!("begin {index}"),
            Event::Line(index, text) => format!("line {index} {text}"),
            Event::End(index, result) => format!("end {index} {result}"),
        });
    }
}

/// Every item runs and the report keeps input order although the first item ends last.
#[test]
fn every_item_runs_and_the_report_keeps_input_order() {
    let mut seen: SeenText = SeenText::default();
    run_ending(vec![40, 1, 20, 2], ITEMS, Arc::new(SleepAndSay), &mut seen).expect("the run finished");
    let expected: Vec<String> = vec![
        String::from("begin 0"),
        String::from("line 0 item 0"),
        String::from("end 0 0"),
        String::from("begin 1"),
        String::from("line 1 item 1"),
        String::from("end 1 10"),
        String::from("begin 2"),
        String::from("line 2 item 2"),
        String::from("end 2 20"),
        String::from("begin 3"),
        String::from("line 3 item 3"),
        String::from("end 3 30"),
    ];
    assert_eq!(seen.seen, expected, "the consumer saw the events out of input order");
}

/// An item that waits for every other item at a point a serial run can never pass.
struct Arrive {
    /// The point all items must reach.
    point: Arc<Rendezvous>,
}

impl OrderedWork<u64, bool> for Arrive {
    fn run(&self, _index: usize, _item: &u64, _say: &mut dyn LineSink) -> bool {
        return self.point.arrive();
    }
}

/// How many items met the others.
#[derive(Default)]
struct CountArrived {
    /// The items that met the others.
    arrived: usize,
}

impl OrderedEvents<bool> for CountArrived {
    fn on_event(&mut self, event: Event<'_, bool>) {
        if let Event::End(_index, came) = event
            && came
        {
            self.arrived += 1;
        }
    }
}

/// What:
///  The items run at the same time,
/// as many at once as the runner promises,
///  and a run that keeps them apart cannot bring them together.
/// Why:
///  The count is derived from the runner's promise and not typed in,
///  because a fixed count fails wherever the process may run fewer blocking jobs:
///  a CPU quota lowers `available_parallelism`,
///  which lowers the runtime's workers and the process's bound with them,
///  and the gate container is allowed two CPUs.
///  Where that bound is one,
/// no two jobs of any run can be inside their work together,
///  so the control says so and returns
/// instead of passing a serial run off as overlap.
#[test]
fn items_run_at_the_same_time() {
    let at_once: usize = jobs_that_may_overlap(WORKERS_ASKED, WORKERS_ASKED);
    // One job at a time can never meet another, so a bound of two is the least this control can measure.
    if at_once < 2 {
        eprintln!(
            "concurrency not exercised here: this process's bound on blocking jobs is {at_once}, so no two items of one run can \
             be at one point; this control shows overlap only where that bound is two or more"
        );
        return;
    }
    let mut seen: CountArrived = CountArrived::default();
    let work: Arc<Arrive> = Arc::new(Arrive { point: Arc::new(Rendezvous::impatient(at_once, PATIENCE)) });
    let items: Vec<u64> = vec![0_u64; at_once];
    let run: OrderedRunOfOwned<'_, u64, bool> = OrderedRunOfOwned {
        items: Arc::from(items),
        lanes: single_lanes(at_once),
        workers: WORKERS_ASKED,
        work,
        on_event: &mut seen,
    };
    run_ordered_and_wait_here(run).expect("the run finished");
    assert_eq!(
        seen.arrived,
        at_once,
        "the fewest of the workers asked for, the lanes and the process's bound on blocking jobs is how many items must meet, \
         and these reached the point"
    );
}

/// How many items are inside their job right now,
/// and the most that ever were.
struct Concurrency {
    /// How many are inside now.
    live: AtomicUsize,
    /// The most that were inside together.
    most: AtomicUsize,
}

/// An item that is inside for `BUSY_MILLIS` and keeps the most.
struct StayAndCount {
    /// What counts the items inside.
    shared: Arc<Concurrency>,
}

impl OrderedWork<u64, u64> for StayAndCount {
    fn run(&self, _index: usize, _item: &u64, _say: &mut dyn LineSink) -> u64 {
        let now: usize = self.shared.live.fetch_add(1, Ordering::SeqCst) + 1;
        let _kept: usize = self.shared.most.fetch_max(now, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(BUSY_MILLIS));
        let _left: usize = self.shared.live.fetch_sub(1, Ordering::SeqCst);
        return 0;
    }
}

/// How many items ended.
#[derive(Default)]
struct CountEnds {
    /// The items that ended.
    ended: usize,
}

impl OrderedEvents<u64> for CountEnds {
    fn on_event(&mut self, event: Event<'_, u64>) {
        if let Event::End(_index, _result) = event {
            self.ended += 1;
        }
    }
}

/// The process's bound on blocking jobs holds the items:
/// one place means one item at a time and a run as long as the items one after another.
#[test]
fn the_process_bound_holds_the_items_to_one_at_a_time() {
    hold_blocking_jobs_to(ONE_PLACE);
    let shared: Arc<Concurrency> = Arc::new(Concurrency { live: AtomicUsize::new(0), most: AtomicUsize::new(0) });
    let mut ends: CountEnds = CountEnds::default();
    let began: Instant = Instant::now();
    run_ending(vec![0_u64; ITEMS], ITEMS, Arc::new(StayAndCount { shared: Arc::clone(&shared) }), &mut ends).expect("the run finished");
    let took: Duration = began.elapsed();
    assert_eq!(ends.ended, ITEMS, "every item ended");
    assert_eq!(shared.most.load(Ordering::SeqCst), ONE_PLACE, "more items were inside their job at once than the process's bound allows");
    assert!(
        took >= Duration::from_millis(BUSY_MILLIS * ITEMS as u64 / 2),
        "the run took {took:?}, which is less than half the time of one item after another, so something other than the bound held it"
    );
}

/// An item that counts itself and does nothing else.
struct CountRuns {
    /// How many items ran.
    ran: Arc<AtomicUsize>,
}

impl OrderedWork<u64, u64> for CountRuns {
    fn run(&self, _index: usize, _item: &u64, _say: &mut dyn LineSink) -> u64 {
        return self.ran.fetch_add(1, Ordering::SeqCst) as u64;
    }
}

/// A consumer that keeps nothing,
/// for a run whose answer is the error it returns.
struct Nothing;

impl OrderedEvents<u64> for Nothing {
    fn on_event(&mut self, _event: Event<'_, u64>) {}
}

/// Where the system refuses every thread,
/// the run says plainly that the items were not run.
#[test]
fn where_no_thread_can_start_the_run_is_a_plain_error() {
    if ran_in_child(module_path!(), "where_no_thread_can_start_the_run_is_a_plain_error") {
        return;
    }
    let ran: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let outcome = run_ending(vec![1_u64, 2, 3], 3, Arc::new(CountRuns { ran: Arc::clone(&ran) }), &mut Nothing);
    assert!(matches!(outcome, Err(OrderedRunError::NoThreadToStart)), "the run answered {outcome:?}");
    assert_eq!(ran.load(Ordering::SeqCst), 0, "a job ran although no thread could start");
}

/// An item that does nothing.
struct Quiet;

impl OrderedWork<u64, u64> for Quiet {
    fn run(&self, _index: usize, _item: &u64, _say: &mut dyn LineSink) -> u64 {
        return 0;
    }
}

/// A lane set that leaves an item out is refused before any job starts.
#[test]
#[should_panic(expected = "a lane set must cover every item")]
fn a_lane_set_that_leaves_an_item_out_is_refused() {
    let run: OrderedRunOfOwned<'_, u64, u64> =
        OrderedRunOfOwned { items: Arc::from(vec![1_u64, 2]), lanes: Arc::from(vec![vec![0_usize]]), workers: 1, work: Arc::new(Quiet), on_event: &mut Nothing };
    run_ordered_and_wait_here(run).expect("the run finished");
}
