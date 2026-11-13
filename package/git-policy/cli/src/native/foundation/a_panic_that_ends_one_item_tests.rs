//! Controls for a panic kept as one item's answer:
//! an answer comes back as it is,
//! a panic comes back as its words,
//! and the task that awaited the item goes on.

use super::*;
use std::time::Duration;

/// What:
///  A future that answers gives its answer back unchanged,
///  so guarding an item changes nothing about an item that does not panic.
/// Why:
///   Every item of a wave is guarded, so the answer of the ordinary one must be exactly what its work gave,
///  in value and in the moment it arrives.
#[test]
fn an_answer_passes_through_unchanged() {
    let answered: Result<u32, String> = crate::merge_second_pass::tests::await_it(keep_a_panic_awaiting(async { return 7; }));
    assert_eq!(answered, Ok(7), "the answer of an item that did not panic is its own");
}

/// What:
///  A future that waits and then answers is awaited to its answer,
///  so a slow item is not cut short.
/// Why:
///   The guard must not turn a wait into an answer:
/// a clone or a push that takes a second is awaited for that second.
#[test]
fn an_item_that_waited_answers_after_its_wait() {
    let answered: Result<u32, String> = crate::merge_second_pass::tests::await_it(keep_a_panic_awaiting(async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        return 9;
    }));
    assert_eq!(answered, Ok(9));
}

/// What:
///  A panic comes back as the words it left,
///  and the task that awaited it is not unwound.
/// Why:
///   This is the whole guarantee:
/// one item's failure ends that item,
/// and the line that settles its record says what happened.
#[test]
fn a_panic_comes_back_as_the_words_it_left() {
    let answered: Result<u32, String> = crate::merge_second_pass::tests::await_it(keep_a_panic_awaiting(async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        panic!("a handler stopped unexpectedly");
    }));
    let said: String = answered.expect_err("a panic is not an answer");
    assert!(said.contains("stopped unexpectedly"), "the words say what happened: {said}");
    assert!(said.contains("a handler stopped unexpectedly"), "the words carry the panic's own text: {said}");
}

/// What:
///  A panic whose payload is not text still says that the work stopped,
///  so the record is never left with no words at all.
/// Why:
///   A panic payload can be any value,
/// and a record settled with no reason would tell the person nothing.
#[test]
fn a_panic_whose_payload_is_not_text_still_says_the_work_stopped() {
    let answered: Result<u32, String> = crate::merge_second_pass::tests::await_it(keep_a_panic_awaiting(async {
        std::panic::panic_any(404_u32);
    }));
    let said: String = answered.expect_err("a panic is not an answer");
    assert!(said.contains("stopped unexpectedly"), "{said}");
    assert!(said.contains("left no words"), "the words say that the panic carried none: {said}");
}
