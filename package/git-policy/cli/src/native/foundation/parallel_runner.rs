//! What:
//!  Runs independent items on a bounded number of threads and reports what they say and what they return in input order,
//!  as if they had run one after another.
//! Why:
//!  Items that share nothing (one worktree each,
//! one directory each) need not wait for each other,
//! but a person reads one
//!  stream of lines.
//! The calling thread is the only consumer:
//! workers hand it lines and results over a channel,
//! it prints the
//!  lines of the first unfinished item as they arrive (so progress stays live),
//! holds the lines of later items until every item
//!  before them is complete,
//! and then prints them in order.
//! No lock is taken:
//! the channel and one atomic counter that hands out
//!  the lanes are the whole of the sharing,
//! and every item has exactly one owner.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runOrdered({ items, lanes, workers, work: new Convert(), onEvent: new Printer() });
//! ```

use crate::line_sink::LineSink;
use crate::scoped_start::{Started, start_or_run_with};
use crate::side_by_side_threads::{ScopeBody, in_thread_scope};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::Scope;

/// What:
///  What the consumer is told,
/// always in input order.
/// Why:
///   An item begins when every item before it is complete,
/// says its lines,
/// and ends with its result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Event<R> = { begin: number } | { line: [number, string] } | { end: [number, R] };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum Event<'line, R> {
    /// The item at this index is the first unfinished one;
    /// its lines follow.
    Begin(usize),
    /// A line the item at this index said.
    Line(usize, &'line str),
    /// The item at this index is complete,
    /// with its result.
    End(usize, R),
}

/// What a worker hands the consumer.
pub(crate) enum Message<R> {
    /// A line an item said.
    Line(usize, String),
    /// An item is complete.
    Done(usize, R),
}

/// What:
///  What one item does.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The state the work needs (the platform,
/// the options,
/// the handlers) is a small struct that implements this and is shared by
///  the worker threads,
/// so it must be `Sync`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface OrderedWork<T, R> { run(index: number, item: T, say: LineSink): R }
/// ```
pub trait OrderedWork<T, R>: Sync {
    /// Do the item at `index`,
    /// saying its lines through `say`,
    /// and return its result.
    fn run(&self, index: usize, item: &T, say: &mut dyn LineSink) -> R;
}

/// What:
///  What the consumer does with each event,
/// on the calling thread only.
/// Why:
///   The state the consumer builds (the printed lines,
/// the counters,
/// the answer) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface OrderedEvents<R> { onEvent(event: Event<R>): void }
/// ```
pub trait OrderedEvents<R> {
    /// Take one event,
    /// in input order.
    fn on_event(&mut self, event: Event<'_, R>);
}

/// What:
///  Everything one ordered run needs.
/// Why:
///   One value names the inputs,
/// so the call reads as a list of facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OrderedRun<T, R> = { items: T[]; lanes: number[][]; workers: number; work: OrderedWork<T, R>; onEvent: OrderedEvents<R> };
/// ```
pub struct OrderedRun<'run, T, R> {
    /// The items,
    /// in the order their output is reported.
    pub items: &'run [T],
    /// The indices of the items,
    /// grouped:
    /// the items of a lane run one after another on one worker,
    /// lanes run side by side.
    ///  Every index appears in exactly one lane.
    /// Items that must not overlap (the same worktree named twice) share a lane.
    pub lanes: &'run [Vec<usize>],
    /// The most threads that run at once;
    /// at least one,
    /// and never more than there are lanes.
    pub workers: usize,
    /// What one item does,
    /// given its index,
    /// itself and a sink that says one line.
    pub work: &'run dyn OrderedWork<T, R>,
    /// The consumer,
    /// called on the calling thread only.
    pub on_event: &'run mut dyn OrderedEvents<R>,
}

/// What the calling thread has reported so far.
pub(crate) struct Cursor<R> {
    /// The first item that is not complete.
    pub(crate) head: usize,
    /// Whether `Begin` was reported for `head`.
    pub(crate) begun: bool,
    /// The lines of items that are not the head yet,
    /// by index.
    pub(crate) held_lines: Vec<Vec<String>>,
    /// The results of items that are complete but not reported,
    /// by index.
    pub(crate) finished: Vec<Option<R>>,
}

impl<R> Cursor<R> {
    /// A cursor with nothing reported yet,
    /// for `count` items.
    pub(crate) fn new(count: usize) -> Self {
        // One empty slot per item: no held lines and no result yet.
        let mut held_lines: Vec<Vec<String>> = Vec::new();
        held_lines.resize_with(count, Vec::new);
        let mut finished: Vec<Option<R>> = Vec::new();
        finished.resize_with(count, Option::default);
        return Cursor { head: 0, begun: false, held_lines, finished };
    }

    /// What:
    ///  Takes one message from a worker and reports whatever is now reportable.
    /// Why:
    ///   Every runner of this folder shares this one place,
    /// so the order the consumer sees cannot drift between them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cursor.tell(message, onEvent);
    /// ```
    pub(crate) fn tell<E: OrderedEvents<R> + ?Sized>(&mut self, message: Message<R>, on_event: &mut E) {
        match message {
            Message::Line(index, text) => {
                if index == self.head && self.begun {
                    on_event.on_event(Event::Line(index, text.as_str()));
                } else {
                    self.held_lines[index].push(text);
                }
            }
            Message::Done(index, result) => self.finished[index] = Some(result),
        }
        self.advance(on_event);
    }

    /// Report everything that is now reportable:
    /// the head begins,
    /// its held lines follow,
    /// and a complete head ends and the
    ///  next item becomes the head.
    pub(crate) fn advance<E: OrderedEvents<R> + ?Sized>(&mut self, on_event: &mut E) {
        loop {
            if self.head >= self.finished.len() {
                return;
            }
            if !self.begun {
                on_event.on_event(Event::Begin(self.head));
                self.begun = true;
                for text in std::mem::take(&mut self.held_lines[self.head]) {
                    on_event.on_event(Event::Line(self.head, text.as_str()));
                }
            }
            match self.finished[self.head].take() {
                Some(result) => {
                    on_event.on_event(Event::End(self.head, result));
                    self.head += 1;
                    self.begun = false;
                }
                None => return,
            }
        }
    }
}

/// The lines of one item,
/// sent to the consumer with the item's index.
struct LinesToConsumer<'sender, R> {
    /// Where the lines and the results go.
    sender: &'sender Sender<Message<R>>,
    /// The item that says the lines.
    index: usize,
}

impl<R> LineSink for LinesToConsumer<'_, R> {
    fn say(&mut self, text: &str) {
        // The consumer is gone only when the calling thread panicked, and that panic is reported by the scope.
        let _sent = self.sender.send(Message::Line(self.index, text.to_string()));
    }
}

/// What one worker reads:
/// the items,
/// the lanes,
/// the counter that hands out lanes,
/// the work and the sender of its results.
type WorkerState<'a, T, R> = (&'a [T], &'a [Vec<usize>], &'a AtomicUsize, &'a dyn OrderedWork<T, R>, Sender<Message<R>>);

/// What a worker does:
/// take lanes until none is left,
/// run the items of each lane in order,
/// and send every line and result.
fn work_lanes<T, R>((items, lanes, next_lane, work, sender): WorkerState<'_, T, R>) {
    loop {
        let lane_index: usize = next_lane.fetch_add(1, Ordering::Relaxed);
        let Some(lane) = lanes.get(lane_index) else { return };
        for &index in lane {
            let Some(item) = items.get(index) else { continue };
            let result: R = work.run(index, item, &mut LinesToConsumer { sender: &sender, index });
            let _sent = sender.send(Message::Done(index, result));
        }
    }
}

/// Everything the scope of one ordered run holds:
/// the workers' inputs,
/// the receiving end and the consumer's cursor.
struct OrderedScope<'env, T, R> {
    /// The items,
    /// in the order their output is reported.
    items: &'env [T],
    /// The lanes the workers take.
    lanes: &'env [Vec<usize>],
    /// The most threads that run at once.
    workers: usize,
    /// What one item does.
    work: &'env dyn OrderedWork<T, R>,
    /// The consumer.
    on_event: &'env mut dyn OrderedEvents<R>,
    /// The counter that hands out the lanes.
    next_lane: &'env AtomicUsize,
    /// The sending end the workers clone;
    /// dropped here once they are started so the channel closes when they are gone.
    sender: Sender<Message<R>>,
    /// Where the workers' lines and results arrive.
    receiver: Receiver<Message<R>>,
    /// What has been reported so far.
    cursor: Cursor<R>,
}

impl<'env, T: Sync + 'env, R: Send + 'env> ScopeBody<'env> for OrderedScope<'env, T, R> {
    type Output = ();

    /// Start the workers,
    /// then read their messages and report them in order until every worker is gone.
    fn run<'scope>(self, scope: &'scope Scope<'scope, 'env>) {
        let OrderedScope { items, lanes, workers, work, on_event, next_lane, sender, receiver, mut cursor } = self;
        // A worker whose thread cannot start runs its lanes here, before the events are read; a panic it had is raised after them.
        let mut inline_panic: Option<Box<dyn std::any::Any + Send>> = None;
        for _worker in 0..workers.max(1).min(lanes.len().max(1)) {
            let worker_sender: Sender<Message<R>> = sender.clone();
            if let Started::Inline(Err(payload)) = start_or_run_with(scope, (items, lanes, next_lane, work, worker_sender), work_lanes::<T, R>) {
                inline_panic.get_or_insert(payload);
            }
        }
        drop(sender);
        cursor.advance(on_event);
        while let Ok(message) = receiver.recv() {
            cursor.tell(message, on_event);
        }
        if let Some(payload) = inline_panic {
            std::panic::resume_unwind(payload);
        }
    }
}

/// What:
///  Refuses lanes that do not name every item exactly once.
/// Why:
///   Both runners of this folder rest on it,
/// and a lane set that leaves an item out would report a run that
///  never ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assertLanesCoverEveryItem(items.length, lanes);
/// ```
pub(crate) fn refuse_lanes_that_do_not_cover_every_item(count: usize, lanes: &[Vec<usize>]) {
    let mut covered: Vec<bool> = vec![false; count];
    for &index in lanes.iter().flatten() {
        let Some(slot) = covered.get_mut(index) else {
            panic!("a lane names item {index}, but there are {count} items");
        };
        assert!(!*slot, "item {index} is in two lanes");
        *slot = true;
    }
    assert!(!covered.contains(&false), "a lane set must cover every item");
}

/// What:
///  Run every item and report in input order.
/// Why:
///   See the module description.
/// A panic in a worker ends the run:
/// the channel closes when every worker is gone,
/// the
///  consumer stops waiting,
/// and the scope raises the worker's panic on the calling thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runOrdered({ items, lanes, workers, work, onEvent });
/// ```
///
/// # Panics
/// When a lane names an index that is not an item,
/// or the lanes do not cover every item exactly once.
pub fn run_ordered<T: Sync, R: Send>(run: OrderedRun<'_, T, R>) {
    let OrderedRun { items, lanes, workers, work, on_event } = run;
    refuse_lanes_that_do_not_cover_every_item(items.len(), lanes);
    let next_lane: AtomicUsize = AtomicUsize::new(0);
    let (sender, receiver) = channel::<Message<R>>();
    let cursor: Cursor<R> = Cursor::new(items.len());
    in_thread_scope(OrderedScope { items, lanes, workers, work, on_event, next_lane: &next_lane, sender, receiver, cursor });
}

/// Ordered run controls stay out of the release executable.
#[cfg(test)]
#[path = "parallel_runner_tests.rs"]
mod tests;
