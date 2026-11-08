//! The wait on a queue the runtime owns:
//! an item queued before or during the wait is received,
//! an empty queue times out at its bound,
//! and a queue whose senders are gone is closed,
//! with the queued items still received first.

use super::*;
use std::time::Instant;
use tokio::sync::mpsc::unbounded_channel;

/// A bound far beyond what a control needs.
const PATIENCE: Duration = Duration::from_secs(30);

/// A bound short enough for a control that expects to reach it.
const SHORT_BOUND: Duration = Duration::from_millis(80);

/// An item queued before the wait is received at once,
/// with or without a bound.
#[test]
fn an_item_queued_before_the_wait_is_received() {
    let (sender, mut receiver) = unbounded_channel::<u8>();
    sender.send(1).expect("queued");
    sender.send(2).expect("queued");
    assert_eq!(receive_for_at_most(&mut receiver, Some(PATIENCE)), Received::Item(1));
    assert_eq!(receive_for_at_most(&mut receiver, None), Received::Item(2));
}

/// An item sent from another thread during the wait ends it.
#[test]
fn an_item_sent_during_the_wait_is_received_before_the_bound() {
    let (sender, mut receiver) = unbounded_channel::<u8>();
    let began: Instant = Instant::now();
    let sending = std::thread::spawn(move || {
        std::thread::sleep(SHORT_BOUND);
        sender.send(7).expect("queued");
    });
    assert_eq!(receive_for_at_most(&mut receiver, Some(PATIENCE)), Received::Item(7));
    assert!(began.elapsed() < PATIENCE, "the item ended the wait");
    sending.join().expect("the sender");
}

/// An empty queue times out at its bound and not before.
#[test]
fn an_empty_queue_times_out_at_its_bound() {
    let (_sender, mut receiver) = unbounded_channel::<u8>();
    let began: Instant = Instant::now();
    assert_eq!(receive_for_at_most(&mut receiver, Some(SHORT_BOUND)), Received::Timeout);
    assert!(began.elapsed() >= SHORT_BOUND, "{:?}", began.elapsed());
}

/// A queue whose senders are gone gives what was queued,
/// then says it is closed.
#[test]
fn a_queue_whose_senders_are_gone_gives_its_items_then_closes() {
    let (sender, mut receiver) = unbounded_channel::<u8>();
    sender.send(3).expect("queued");
    drop(sender);
    assert_eq!(receive_for_at_most(&mut receiver, Some(PATIENCE)), Received::Item(3));
    assert_eq!(receive_for_at_most(&mut receiver, Some(PATIENCE)), Received::Closed);
    assert_eq!(receive_for_at_most(&mut receiver, None), Received::Closed);
}
