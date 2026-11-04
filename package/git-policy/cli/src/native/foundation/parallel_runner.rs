//! What:
//!  Runs independent items on a bounded number of threads and reports what they say and what they return in input order,
//!  as if they had run one after another.
//! Why:
//!  Items that share nothing (one worktree each, one directory each) need not wait for each other, but a person reads one
//!  stream of lines. The calling thread is the only consumer: workers hand it lines and results over a channel, it prints the
//!  lines of the first unfinished item as they arrive (so progress stays live), holds the lines of later items until every item
//!  before them is complete, and then prints them in order. No lock is taken: the channel and one atomic counter that hands out
//!  the lanes are the whole of the sharing, and every item has exactly one owner.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runOrdered({ items, lanes, workers, work: (index, item, say) => convert(item, say), onEvent });
//! ```

use crate::scoped_start::{Started, start_or_run};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};

/// What:
///  What the consumer is told, always in input order.
/// Why:
///   An item begins when every item before it is complete, says its lines, and ends with its result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Event<R> = { begin: number } | { line: [number, string] } | { end: [number, R] };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum Event<'line, R> {
    /// The item at this index is the first unfinished one; its lines follow.
    Begin(usize),
    /// A line the item at this index said.
    Line(usize, &'line str),
    /// The item at this index is complete, with its result.
    End(usize, R),
}

/// What a worker hands the consumer.
enum Message<R> {
    /// A line an item said.
    Line(usize, String),
    /// An item is complete.
    Done(usize, R),
}

/// What:
///  Everything one ordered run needs.
/// Why:
///   One value names the inputs, so the call reads as a list of facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OrderedRun<T, R> = { items: T[]; lanes: number[][]; workers: number; work: (index, item, say) => R; onEvent: (event) => void };
/// ```
pub struct OrderedRun<'run, T, R> {
    /// The items, in the order their output is reported.
    pub items: &'run [T],
    /// The indices of the items, grouped: the items of a lane run one after another on one worker, lanes run side by side.
    ///  Every index appears in exactly one lane. Items that must not overlap (the same worktree named twice) share a lane.
    pub lanes: &'run [Vec<usize>],
    /// The most threads that run at once; at least one, and never more than there are lanes.
    pub workers: usize,
    /// What one item does, given its index, itself and a function that says one line.
    pub work: &'run (dyn Fn(usize, &T, &mut dyn FnMut(&str)) -> R + Sync),
    /// The consumer, called on the calling thread only.
    pub on_event: &'run mut dyn FnMut(Event<'_, R>),
}

/// What the calling thread has reported so far.
struct Cursor<R> {
    /// The first item that is not complete.
    head: usize,
    /// Whether `Begin` was reported for `head`.
    begun: bool,
    /// The lines of items that are not the head yet, by index.
    held_lines: Vec<Vec<String>>,
    /// The results of items that are complete but not reported, by index.
    finished: Vec<Option<R>>,
}

impl<R> Cursor<R> {
    /// Report everything that is now reportable: the head begins, its held lines follow, and a complete head ends and the
    ///  next item becomes the head.
    fn advance(&mut self, on_event: &mut dyn FnMut(Event<'_, R>)) {
        loop {
            if self.head >= self.finished.len() {
                return;
            }
            if !self.begun {
                on_event(Event::Begin(self.head));
                self.begun = true;
                for text in std::mem::take(&mut self.held_lines[self.head]) {
                    on_event(Event::Line(self.head, text.as_str()));
                }
            }
            match self.finished[self.head].take() {
                Some(result) => {
                    on_event(Event::End(self.head, result));
                    self.head += 1;
                    self.begun = false;
                }
                None => return,
            }
        }
    }
}

/// What a worker does: take lanes until none is left, run the items of each lane in order, and send every line and result.
fn work_lanes<T, R>(run: (&[T], &[Vec<usize>], &AtomicUsize, &(dyn Fn(usize, &T, &mut dyn FnMut(&str)) -> R + Sync)), sender: &Sender<Message<R>>) {
    let (items, lanes, next_lane, work) = run;
    loop {
        let lane_index: usize = next_lane.fetch_add(1, Ordering::Relaxed);
        let Some(lane) = lanes.get(lane_index) else { return };
        for &index in lane {
            let Some(item) = items.get(index) else { continue };
            let result: R = work(index, item, &mut |text: &str| {
                // The consumer is gone only when the calling thread panicked, and that panic is reported by the scope.
                let _sent = sender.send(Message::Line(index, text.to_string()));
            });
            let _sent = sender.send(Message::Done(index, result));
        }
    }
}

/// What:
///  Run every item and report in input order.
/// Why:
///   See the module description. A panic in a worker ends the run: the channel closes when every worker is gone, the
///  consumer stops waiting, and the scope raises the worker's panic on the calling thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runOrdered({ items, lanes, workers, work, onEvent });
/// ```
///
/// # Panics
/// When a lane names an index that is not an item, or the lanes do not cover every item exactly once.
pub fn run_ordered<T: Sync, R: Send>(run: OrderedRun<'_, T, R>) {
    let OrderedRun { items, lanes, workers, work, on_event } = run;
    let mut covered: Vec<bool> = vec![false; items.len()];
    for &index in lanes.iter().flatten() {
        let Some(slot) = covered.get_mut(index) else {
            panic!("a lane names item {index}, but there are {} items", items.len());
        };
        assert!(!*slot, "item {index} is in two lanes");
        *slot = true;
    }
    assert!(!covered.contains(&false), "a lane set must cover every item");
    let next_lane: AtomicUsize = AtomicUsize::new(0);
    let (sender, receiver) = channel::<Message<R>>();
    // One empty slot per item: no held lines and no result yet.
    let mut held_lines: Vec<Vec<String>> = Vec::new();
    held_lines.resize_with(items.len(), Vec::new);
    let mut finished: Vec<Option<R>> = Vec::new();
    finished.resize_with(items.len(), Option::default);
    let mut cursor: Cursor<R> = Cursor { head: 0, begun: false, held_lines, finished };
    std::thread::scope(|scope| {
        // A worker whose thread cannot start runs its lanes here, before the events are read; a panic it had is raised after them.
        let mut inline_panic: Option<Box<dyn std::any::Any + Send>> = None;
        for _worker in 0..workers.max(1).min(lanes.len().max(1)) {
            let worker_sender: Sender<Message<R>> = sender.clone();
            let next: &AtomicUsize = &next_lane;
            if let Started::Inline(Err(payload)) = start_or_run(scope, move || { work_lanes((items, lanes, next, work), &worker_sender); }) {
                inline_panic.get_or_insert(payload);
            }
        }
        drop(sender);
        cursor.advance(on_event);
        while let Ok(message) = receiver.recv() {
            match message {
                Message::Line(index, text) => {
                    if index == cursor.head && cursor.begun {
                        on_event(Event::Line(index, text.as_str()));
                    } else {
                        cursor.held_lines[index].push(text);
                    }
                }
                Message::Done(index, result) => cursor.finished[index] = Some(result),
            }
            cursor.advance(on_event);
        }
        if let Some(payload) = inline_panic {
            std::panic::resume_unwind(payload);
        }
    });
}

/// Ordered run controls stay out of the release executable.
#[cfg(test)]
#[path = "parallel_runner_tests.rs"]
mod tests;
