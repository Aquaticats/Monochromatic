//! Controls for the in-memory claim on a name that is being published: one holder at a time, a bounded wait, a claim given back
//! on every way out, and independence of names in different slots.

use super::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

/// A wait no control reaches: a held name is given back long before it.
const FAR_LONGER_THAN_A_CONTROL: Duration = Duration::from_secs(60);

/// Two paths whose claims live in different slots, found by trying names until the slots differ.
fn two_paths_in_different_slots() -> (PathBuf, PathBuf) {
    let first: PathBuf = PathBuf::from("claim-control-first");
    for number in 0.. {
        let other: PathBuf = PathBuf::from(format!("claim-control-other-{number}"));
        if slot_of(first.as_path()) != slot_of(other.as_path()) {
            return (first, other);
        }
    }
    unreachable!("the range has no end");
}

/// A held name refuses a second hold of the same name until the first ends, and then gives it; the refusal is `TimedOut`, and the
///  control's own wait is the length of a zero deadline, so nothing sleeps.
#[test]
fn a_name_that_is_held_is_refused_until_the_holder_drops() {
    let path: PathBuf = PathBuf::from("claim-control-held");
    let first: HeldName = hold_name_while_publishing(path.as_path(), FAR_LONGER_THAN_A_CONTROL).expect("a free name is held");
    let refused: std::io::Error = hold_name_while_publishing(path.as_path(), Duration::ZERO).err().expect("the name is held");
    assert_eq!(refused.kind(), std::io::ErrorKind::TimedOut);
    assert!(refused.to_string().contains("claim-control-held"), "{refused}");
    drop(first);
    hold_name_while_publishing(path.as_path(), Duration::ZERO).expect("the name is free once the holder dropped");
}

/// A thread that waits for a held name gets it as soon as the holder lets go, long before its deadline.
#[test]
fn a_waiting_thread_gets_the_name_when_the_holder_lets_go() {
    let path: PathBuf = PathBuf::from("claim-control-waiting");
    let first: HeldName = hold_name_while_publishing(path.as_path(), FAR_LONGER_THAN_A_CONTROL).expect("held");
    let waiter_path: PathBuf = path.clone();
    let waiter = std::thread::spawn(move || {
        return hold_name_while_publishing(waiter_path.as_path(), FAR_LONGER_THAN_A_CONTROL).map(drop);
    });
    // The waiter is waiting while the holder holds: a thread that finished could not have been waiting.
    std::thread::sleep(Duration::from_millis(50));
    assert!(!waiter.is_finished(), "the waiter must not hold a name that is held");
    drop(first);
    waiter.join().expect("the waiter").expect("the waiter got the name");
}

/// A claim is given back when the publish panics, so a panic of one thread never keeps a name from the others.
#[test]
fn a_panic_while_holding_gives_the_name_back() {
    let path: PathBuf = PathBuf::from("claim-control-panic");
    let panicking_path: PathBuf = path.clone();
    let outcome = std::thread::spawn(move || {
        let _held: HeldName = hold_name_while_publishing(panicking_path.as_path(), FAR_LONGER_THAN_A_CONTROL).expect("held");
        panic!("the publish panicked");
    })
    .join();
    assert!(outcome.is_err(), "the control's thread panicked");
    hold_name_while_publishing(path.as_path(), Duration::ZERO).expect("the claim was given back by the unwind");
}

/// Names in different slots are held at once; only names that share a slot wait for each other.
#[test]
fn names_in_different_slots_are_held_at_the_same_time() {
    let (first, second) = two_paths_in_different_slots();
    let _a: HeldName = hold_name_while_publishing(first.as_path(), Duration::ZERO).expect("first");
    let _b: HeldName = hold_name_while_publishing(second.as_path(), Duration::ZERO).expect("second, in another slot");
}

/// Many threads that each hold one name, read a counter, let others run and write the counter plus one never lose an increment:
///  the claim makes the read and the write one step to the others (a read-modify-write that is not atomic, on purpose).
#[test]
fn the_claim_makes_a_read_and_a_write_one_step() {
    const THREADS: usize = 8;
    const ROUNDS: usize = 200;
    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let workers: Vec<_> = (0..THREADS)
        .map(|_| {
            let shared: Arc<AtomicUsize> = Arc::clone(&counter);
            return std::thread::spawn(move || {
                for _ in 0..ROUNDS {
                    let _held: HeldName = hold_name_while_publishing(Path::new("claim-control-counter"), FAR_LONGER_THAN_A_CONTROL).expect("held");
                    let seen: usize = shared.load(Ordering::SeqCst);
                    std::thread::yield_now();
                    shared.store(seen + 1, Ordering::SeqCst);
                }
            });
        })
        .collect();
    for worker in workers {
        worker.join().expect("a worker");
    }
    assert_eq!(counter.load(Ordering::SeqCst), THREADS * ROUNDS);
}
