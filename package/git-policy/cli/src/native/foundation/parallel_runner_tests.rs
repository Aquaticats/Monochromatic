//! The ordered parallel run: items really overlap, the report keeps input order, progress is live, the worker bound holds, lanes
//!  never overlap inside themselves, and a worker's panic ends the run instead of hanging it.

use super::*;
use crate::parallel_test_support::{PATIENCE, Rendezvous, SERIAL_PATIENCE};
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

/// One lane per item.
fn single_lanes(count: usize) -> Vec<Vec<usize>> {
    return (0..count).map(|index: usize| { return vec![index]; }).collect();
}

/// Run `work` over `count` items on `workers` threads and return every event as text, in the order the consumer saw them.
fn events_of(count: usize, workers: usize, work: &(dyn Fn(usize, &usize, &mut dyn FnMut(&str)) -> String + Sync)) -> Vec<String> {
    let items: Vec<usize> = (0..count).collect();
    let lanes: Vec<Vec<usize>> = single_lanes(count);
    let mut seen: Vec<String> = Vec::new();
    run_ordered(OrderedRun {
        items: items.as_slice(),
        lanes: lanes.as_slice(),
        workers,
        work,
        on_event: &mut |event: Event<'_, String>| {
            seen.push(match event {
                Event::Begin(index) => format!("begin {index}"),
                Event::Line(index, text) => format!("line {index} {text}"),
                Event::End(index, result) => format!("end {index} {result}"),
            });
        },
    });
    return seen;
}

/// Items that wait for each other all finish when they run side by side: a serial run would time out at the rendezvous.
#[test]
fn items_overlap_in_time() {
    let count: usize = 4;
    let point: Rendezvous = Rendezvous::new(count);
    let events: Vec<String> = events_of(count, count, &|_index: usize, _item: &usize, _say: &mut dyn FnMut(&str)| { return point.arrive().to_string(); });
    let ends: Vec<&String> = events.iter().filter(|event: &&String| { return event.starts_with("end"); }).collect();
    assert_eq!(ends.len(), count, "{events:?}");
    assert!(ends.iter().all(|end: &&String| { return end.ends_with("true"); }), "every item met the others: {events:?}");
}

/// The control for the control: one worker cannot pass a rendezvous that needs two, so the overlap test would fail for a serial run.
#[test]
fn one_worker_cannot_pass_a_rendezvous_of_two() {
    let point: Rendezvous = Rendezvous::impatient(2, SERIAL_PATIENCE);
    let events: Vec<String> = events_of(2, 1, &|_index: usize, _item: &usize, _say: &mut dyn FnMut(&str)| { return point.arrive().to_string(); });
    assert!(events.iter().any(|event: &String| { return event.ends_with("false"); }), "{events:?}");
}

/// The first item finishes last, yet the report keeps input order and holds the later items' lines until it is complete.
#[test]
fn the_report_keeps_input_order_when_completion_order_inverts() {
    let count: usize = 3;
    let last_done: Rendezvous = Rendezvous::new(count - 1);
    let events: Vec<String> = events_of(count, count, &|index: usize, _item: &usize, say: &mut dyn FnMut(&str)| {
        say("working");
        if index == 0 {
            assert!(last_done.wait_for_target(), "the later items finished while the first waited");
        } else {
            last_done.arrive_without_waiting();
        }
        return String::from("ok");
    });
    let expected: Vec<String> = (0..count)
        .flat_map(|index: usize| { return [format!("begin {index}"), format!("line {index} working"), format!("end {index} ok")]; })
        .collect();
    assert_eq!(events, expected);
}

/// A line of the first unfinished item reaches the consumer while that item still runs: the item waits for the consumer to have seen it.
#[test]
fn the_first_items_lines_are_live() {
    let seen_by_consumer: AtomicBool = AtomicBool::new(false);
    let items: Vec<usize> = vec![0];
    let lanes: Vec<Vec<usize>> = single_lanes(1);
    let mut observed_live: bool = false;
    run_ordered(OrderedRun {
        items: items.as_slice(),
        lanes: lanes.as_slice(),
        workers: 1,
        work: &|_index: usize, _item: &usize, say: &mut dyn FnMut(&str)| {
            say("early");
            let deadline: std::time::Instant = std::time::Instant::now() + PATIENCE;
            while !seen_by_consumer.load(Ordering::SeqCst) && std::time::Instant::now() < deadline {
                std::thread::yield_now();
            }
            return seen_by_consumer.load(Ordering::SeqCst);
        },
        on_event: &mut |event: Event<'_, bool>| {
            match event {
                Event::Line(_, _) => seen_by_consumer.store(true, Ordering::SeqCst),
                Event::End(_, still_running_when_seen) => observed_live = still_running_when_seen,
                Event::Begin(_) => {}
            }
        },
    });
    assert!(observed_live, "the consumer saw the line before the item ended");
}

/// No more items run at once than the workers allow, and the bound is reached.
#[test]
fn the_worker_bound_holds_and_is_reached() {
    let workers: usize = 3;
    let running: AtomicUsize = AtomicUsize::new(0);
    let high: AtomicUsize = AtomicUsize::new(0);
    let point: Rendezvous = Rendezvous::new(workers);
    let _events = events_of(workers * 3, workers, &|index: usize, _item: &usize, _say: &mut dyn FnMut(&str)| {
        let now: usize = running.fetch_add(1, Ordering::SeqCst) + 1;
        high.fetch_max(now, Ordering::SeqCst);
        if index < workers {
            assert!(point.arrive(), "the first items overlap");
        }
        running.fetch_sub(1, Ordering::SeqCst);
        return String::new();
    });
    assert_eq!(high.load(Ordering::SeqCst), workers);
}

/// The items of one lane run one after another in lane order, while lanes overlap.
#[test]
fn a_lane_runs_its_items_in_order_and_never_overlaps_itself() {
    let items: Vec<usize> = (0..4).collect();
    let lanes: Vec<Vec<usize>> = vec![vec![0, 2], vec![1, 3]];
    let in_lane: [AtomicUsize; 2] = [AtomicUsize::new(0), AtomicUsize::new(0)];
    let violations: AtomicUsize = AtomicUsize::new(0);
    let order: std::sync::Mutex<Vec<usize>> = Mutex::new(Vec::new());
    let point: Rendezvous = Rendezvous::new(2);
    run_ordered(OrderedRun {
        items: items.as_slice(),
        lanes: lanes.as_slice(),
        workers: 2,
        work: &|index: usize, _item: &usize, _say: &mut dyn FnMut(&str)| {
            let lane: &AtomicUsize = &in_lane[index % 2];
            if lane.fetch_add(1, Ordering::SeqCst) != 0 {
                violations.fetch_add(1, Ordering::SeqCst);
            }
            if index < 2 {
                assert!(point.arrive(), "the lanes overlap");
            }
            order.lock().expect("lock").push(index);
            lane.fetch_sub(1, Ordering::SeqCst);
        },
        on_event: &mut |_event: Event<'_, ()>| {},
    });
    assert_eq!(violations.load(Ordering::SeqCst), 0);
    let ran: Vec<usize> = order.lock().expect("lock").clone();
    let position = |index: usize| -> usize { return ran.iter().position(|seen: &usize| { return *seen == index; }).expect("ran"); };
    assert!(position(0) < position(2) && position(1) < position(3), "{ran:?}");
}

/// A panic in a worker reaches the caller and does not leave the consumer waiting.
#[test]
fn a_worker_panic_ends_the_run() {
    let outcome = std::panic::catch_unwind(|| {
        let _events = events_of(3, 3, &|index: usize, _item: &usize, _say: &mut dyn FnMut(&str)| {
            assert_ne!(index, 1, "this item fails");
            return String::new();
        });
    });
    assert!(outcome.is_err());
}

/// A lane set that leaves an item out, repeats one, or names one that does not exist is refused before anything runs.
#[test]
fn lanes_must_cover_every_item_exactly_once() {
    let items: Vec<usize> = (0..2).collect();
    for lanes in [vec![vec![0]], vec![vec![0], vec![0, 1]], vec![vec![0, 1, 2]]] {
        let ran: AtomicBool = AtomicBool::new(false);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_ordered(OrderedRun {
                items: items.as_slice(),
                lanes: lanes.as_slice(),
                workers: 1,
                work: &|_index: usize, _item: &usize, _say: &mut dyn FnMut(&str)| { ran.store(true, Ordering::SeqCst); },
                on_event: &mut |_event: Event<'_, ()>| {},
            });
        }));
        assert!(outcome.is_err(), "{lanes:?}");
        assert!(!ran.load(Ordering::SeqCst), "nothing ran for {lanes:?}");
    }
}

/// An empty run reports nothing and returns.
#[test]
fn an_empty_run_reports_nothing() {
    assert!(events_of(0, 4, &|_index: usize, _item: &usize, _say: &mut dyn FnMut(&str)| { return String::new(); }).is_empty());
}

/// Where the system starts no thread the workers' lanes run on the calling thread and the report is whole and in order (shipped
/// `Scope::spawn` panicked there). The body runs in a child that is refused every thread.
#[test]
fn an_ordered_run_where_no_thread_can_start_reports_every_item_in_order() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "an_ordered_run_where_no_thread_can_start_reports_every_item_in_order") {
        return;
    }
    let seen: Vec<String> = events_of(5, 3, &|index: usize, _item: &usize, _line: &mut dyn FnMut(&str)| { return format!("r{index}"); });
    let ends: Vec<&String> = seen.iter().filter(|text| { return text.starts_with("end "); }).collect();
    assert_eq!(ends, vec!["end 0 r0", "end 1 r1", "end 2 r2", "end 3 r3", "end 4 r4"], "{seen:?}");
}
