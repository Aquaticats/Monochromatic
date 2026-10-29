//! Ordering and eviction of the bounded event ring.

/// Brings the ring and its bound into scope.
use super::*;

/// Events come back oldest first,
///  and the window keeps the newest ones.
#[test]
fn last_n_is_a_window_over_the_newest_events() {
    let mut ring = EventRing::default();
    for number in 0..5_u32 {
        ring.push(u64::from(number) * 10, number);
    }
    let all: Vec<u32> = ring.last_n(64).iter().map(|timed| return timed.event).collect();
    assert_eq!(all, [0, 1, 2, 3, 4]);
    let newest: Vec<u64> = ring.last_n(2).iter().map(|timed| return timed.at_ms).collect();
    assert_eq!(newest, [30, 40]);
}

/// Past the capacity the oldest event is dropped first.
#[test]
fn the_oldest_event_is_evicted_at_capacity() {
    let mut ring = EventRing::default();
    for number in 0..EVENT_RING_CAPACITY + 3 {
        ring.push(0, number);
    }
    assert_eq!(ring.len(), EVENT_RING_CAPACITY);
    assert_eq!(ring.last_n(usize::MAX).first().map(|timed| return timed.event), Some(3));
}

/// Clearing empties the ring and a new event starts a fresh sequence.
#[test]
fn clear_empties_the_ring() {
    let mut ring = EventRing::default();
    ring.push(1, "a");
    ring.clear();
    assert!(ring.is_empty());
    ring.push(2, "b");
    assert_eq!(ring.len(), 1);
}
