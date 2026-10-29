//! A bounded,
//!  oldest-first record of observed events.
//!
//! The notification and job stores keep their logs here.
//! The ring holds plain values only,
//!  so ordering and eviction are unit-tested without a bus.

/// Imports the double-ended queue that holds the events in arrival order.
use std::collections::VecDeque;

/// Most events kept before the oldest is dropped.
pub const EVENT_RING_CAPACITY: usize = 4096;

/// One event with the time it was observed.
///
/// What:
///      `pub struct Timed<T> { at_ms: u64, event: T }`.
/// Why:
///       Every log line starts with the milliseconds since the compositor started.
#[derive(Clone, Debug, PartialEq)]
pub struct Timed<T> {
    /// Milliseconds since the compositor started,
    ///  when the event was observed.
    pub at_ms: u64,
    /// The event itself.
    pub event: T,
}

/// Oldest-first ring of the most recent events.
#[derive(Debug)]
pub struct EventRing<T> {
    /// Events in arrival order,
    ///  oldest at the front.
    events: VecDeque<Timed<T>>,
}

/// An empty ring is the starting state of every log.
impl<T> Default for EventRing<T> {
    /// Creates an empty ring.
    fn default() -> Self {
        return Self { events: VecDeque::new() };
    }
}

/// Appending,
///  reading the newest events,
///  and clearing.
impl<T> EventRing<T> {
    /// Appends an event and evicts the oldest once the capacity is reached.
    pub fn push(&mut self, at_ms: u64, event: T) {
        if self.events.len() == EVENT_RING_CAPACITY {
            self.events.pop_front();
        }
        self.events.push_back(Timed { at_ms, event });
    }

    /// Returns up to `count` of the newest events,
    ///  oldest first.
    #[must_use]
    pub fn last_n(&self, count: usize) -> Vec<&Timed<T>> {
        let skip = self.events.len().saturating_sub(count);
        return self.events.iter().skip(skip).collect();
    }

    /// Removes every stored event.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Returns how many events are stored.
    #[must_use]
    pub fn len(&self) -> usize {
        return self.events.len();
    }

    /// Tells whether no event is stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        return self.events.is_empty();
    }
}

/// Unit tests for ordering and eviction.
#[cfg(test)]
#[path = "event_ring_tests.rs"]
mod tests;
