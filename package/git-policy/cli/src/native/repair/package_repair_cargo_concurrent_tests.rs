//! Controls for the runner of one job per item:
//! every item has its result in item order,
//! a panic costs only the item it happened in,
//! and no more items are inside their work at once than the run asks for.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// A runtime of the control's own,
/// so no other test's tasks run on it.
fn runtime() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
}

/// The words that name an item,
/// as `item 3`.
fn item_label(item: &usize) -> String {
    return format!("item {item}");
}

/// A job that answers with ten times its item,
/// and one that breaks on `at`.
struct TenTimes {
    /// The item that breaks.
    at: Option<usize>,
}

impl ConcurrentAwaitingWork<usize, usize> for TenTimes {
    fn run<'a>(&'a self, item: &'a usize) -> Pin<Box<dyn Future<Output = usize> + Send + 'a>> {
        return Box::pin(async move {
            // A wait before the answer, so the panic happens inside a poll after the first.
            tokio::time::sleep(Duration::from_millis(1)).await;
            let Some(broken) = self.at else { return *item * 10 };
            assert_ne!(*item, broken, "this item breaks");
            return *item * 10;
        });
    }
}

/// A job that counts how many items are inside their work at once.
struct Counting {
    /// How many are inside now.
    inside: Arc<AtomicUsize>,
    /// The most that were inside at once.
    most: Arc<AtomicUsize>,
}

impl ConcurrentAwaitingWork<usize, usize> for Counting {
    fn run<'a>(&'a self, item: &'a usize) -> Pin<Box<dyn Future<Output = usize> + Send + 'a>> {
        return Box::pin(async move {
            let now: usize = self.inside.fetch_add(1, Ordering::SeqCst) + 1;
            self.most.fetch_max(now, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _left: usize = self.inside.fetch_sub(1, Ordering::SeqCst);
            return *item + 1;
        });
    }
}

/// Without a panic every item has its result in item order,
/// for any number of workers.
#[test]
fn every_item_has_its_result_in_item_order() {
    let items: Vec<usize> = (0..9).collect();
    for workers in [0, 1, 4, 64] {
        let shared: Arc<[usize]> = Arc::from(items.clone());
        let results: Vec<Result<usize, String>> =
            runtime().block_on(map_concurrent_awaiting(shared, workers, Arc::new(item_label), Arc::new(TenTimes { at: None })));
        assert_eq!(results, items.iter().map(|item: &usize| { return Ok(*item * 10); }).collect::<Vec<Result<usize, String>>>(), "workers {workers}");
    }
}

/// A job that panics on one item gives that item a failure naming it and saying what the panic said;
/// every other item still has its result,
/// in item order.
#[test]
fn a_panicking_item_is_a_failure_naming_it_and_the_others_keep_their_results() {
    let items: Vec<usize> = (0..6).collect();
    let shared: Arc<[usize]> = Arc::from(items.clone());
    let results: Vec<Result<usize, String>> = runtime().block_on(map_concurrent_awaiting(shared, 2, Arc::new(item_label), Arc::new(TenTimes { at: Some(3) })));
    assert_eq!(results.len(), items.len(), "{results:?}");
    for (item, outcome) in items.iter().zip(&results) {
        if *item == 3 {
            let reason: &String = outcome.as_ref().expect_err("the broken item fails");
            assert!(reason.contains("item 3") && reason.contains("this item breaks"), "{reason}");
        } else {
            assert_eq!(outcome.as_ref().ok(), Some(&(*item * 10)), "item {item}");
        }
    }
}

/// The items inside their work at once are never more than the run asks for,
/// and a run that asks for two really starts two.
#[test]
fn no_more_items_are_inside_their_work_at_once_than_the_run_asks_for() {
    const ITEMS: usize = 6;
    const WORKERS: usize = 2;
    let inside: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let most: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let items: Arc<[usize]> = Arc::from((0..ITEMS).collect::<Vec<usize>>());
    let results: Vec<Result<usize, String>> = runtime().block_on(map_concurrent_awaiting(
        items,
        WORKERS,
        Arc::new(item_label),
        Arc::new(Counting { inside: Arc::clone(&inside), most: Arc::clone(&most) }),
    ));
    assert!(results.iter().all(Result::is_ok), "{results:?}");
    let reached: usize = most.load(Ordering::SeqCst);
    assert_eq!(reached, WORKERS, "the run started {reached} items at once, and it was asked for {WORKERS}");
}
