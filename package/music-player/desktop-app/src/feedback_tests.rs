// What:     Unit tests for `feedback.rs`, pulled in by
//           `#[cfg(test)] #[path = "feedback_tests.rs"] mod tests;` at the bottom of `feedback.rs`.
//           Mirrors every case of the Kotlin `FeedbackQueueTest` with the same clock values and expected messages.
// Why:      Keep the tests beside the code. Expiry, replacement, and ordering are checked on exact clock values.

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need the queue, the message and kind types, the duration constants, and `FeedbackError`.
use super::*;

// What:     `const START_MS` is the clock value at which the test messages appear.
// Why:      A non-zero start proves that expiry uses elapsed time rather than the absolute clock value.
const START_MS: i64 = 10_000;

// What:     `const HALF_SECOND_MS` is a short offset after the first show.
// Why:      A second message shown part way through the first one's lifetime exercises replacement timing.
const HALF_SECOND_MS: i64 = 500;

// What:     `const ONE_SECOND_MS` is a short custom visible time.
// Why:      A duration shorter than the default proves that each entry keeps its own timer.
const ONE_SECOND_MS: i64 = 1_000;

// What:     `const THREE_SECONDS_MS` is an offset that leaves under the default time.
// Why:      A re-show at this offset must restart the timer rather than inherit the earlier show time.
const THREE_SECONDS_MS: i64 = 3_000;

// What:     `const FOUR_SECONDS_MS` is the expected default visible time.
// Why:      The default-duration tests compare against this literal, so the accepted value is stated once.
const FOUR_SECONDS_MS: i64 = 4_000;

// What:     `fn error_message(id)` builds one error message with no action.
// Why:      Each test names only the id it cares about, so the other fields stay identical across tests.
//
// In TS you'd write (pseudocode):
// ```ts
// function errorMessage(id: string): FeedbackMessage { ... }
// ```
fn error_message(id: &str) -> FeedbackMessage {
    return FeedbackMessage::new(id, &format!("Error {id}"), FeedbackKind::Error, None)
        .expect("error message fixture has non-blank fields");
}

// What:     `fn undo_message(id)` builds one Undo message with an Undo action.
// Why:      Undo tests need an action label so they match the production shape of the message.
//
// In TS you'd write (pseudocode):
// ```ts
// function undoMessage(id: string): FeedbackMessage { ... }
// ```
fn undo_message(id: &str) -> FeedbackMessage {
    return FeedbackMessage::new(
        id,
        &format!("Moved {id} to trash"),
        FeedbackKind::Undo,
        Some("Undo"),
    )
    .expect("undo message fixture has non-blank fields");
}

// What:     `fn show_timed(queue, message, now_ms, duration_ms)` shows a message with an in-range visible time.
// Why:      Every timed fixture uses a duration inside the bound, so the expect message stays the same.
fn show_timed(
    queue: &FeedbackQueue,
    message: FeedbackMessage,
    now_ms: i64,
    duration_ms: i64,
) -> FeedbackQueue {
    return queue
        .show_for(message, now_ms, duration_ms)
        .expect("fixture duration is within the bound");
}

// What:     `#[test] fn show_adds_message()` checks that one shown message becomes visible.
// Why:      Every other transition builds on a queue that holds the message after show.
#[test]
fn show_adds_message() {
    let queue = FeedbackQueue::EMPTY.show(error_message("a"), START_MS);
    assert_eq!(queue.visible(), [error_message("a")]);
}

// What:     `#[test] fn show_replaces_same_kind()` checks that a second error replaces the first error.
// Why:      Only one error may be visible, so the newest failure is the one the user reads.
#[test]
fn show_replaces_same_kind() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("a"), START_MS)
        .show(error_message("b"), START_MS);
    assert_eq!(queue.visible(), [error_message("b")]);
}

// What:     `#[test] fn replacement_restarts_timer()` checks that the replacement expires on its own schedule.
// Why:      A replaced message must not keep its old expiry, or the newer message would vanish early.
#[test]
fn replacement_restarts_timer() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("a"), START_MS)
        .show(error_message("b"), START_MS + HALF_SECOND_MS)
        .expire(START_MS + FOUR_SECONDS_MS);
    assert_eq!(queue.visible(), [error_message("b")]);
}

// What:     `#[test] fn undo_and_error_both_visible()` checks that one message of each kind can be visible together.
// Why:      The Undo message and the error message are independent, so both may be visible at once.
#[test]
fn undo_and_error_both_visible() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("e"), START_MS)
        .show(undo_message("u"), START_MS);
    assert_eq!(queue.visible(), [undo_message("u"), error_message("e")]);
}

// What:     `#[test] fn visible_orders_undo_above_error()` checks display order when the error is shown first.
// Why:      The Undo message is drawn above the error message regardless of show order.
#[test]
fn visible_orders_undo_above_error() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("e"), START_MS)
        .show(undo_message("u"), START_MS);
    let visible = queue.visible();
    assert_eq!(visible.first().map(FeedbackMessage::id), Some("u"));
}

// What:     `#[test] fn dismiss_removes_by_id()` checks that dismissal removes only the named message.
// Why:      The other visible message keeps its place and its timer after one is dismissed.
#[test]
fn dismiss_removes_by_id() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("e"), START_MS)
        .show(undo_message("u"), START_MS)
        .dismiss("e");
    assert_eq!(queue.visible(), [undo_message("u")]);
}

// What:     `#[test] fn dismiss_unknown_id_keeps_queue()` checks that an unknown id changes nothing.
// Why:      A dismiss that arrives after expiry must not remove a message that is still visible.
#[test]
fn dismiss_unknown_id_keeps_queue() {
    let queue = FeedbackQueue::EMPTY.show(error_message("e"), START_MS);
    assert_eq!(queue.dismiss("missing").visible(), [error_message("e")]);
}

// What:     `#[test] fn stale_dismiss_keeps_replacement()` checks a dismiss aimed at a replaced message.
// Why:      A late dismiss for the old error must not hide the newer error the user has not read.
#[test]
fn stale_dismiss_keeps_replacement() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("old"), START_MS)
        .show(error_message("new"), START_MS)
        .dismiss("old");
    assert_eq!(queue.visible(), [error_message("new")]);
}

// What:     `#[test] fn expire_keeps_message_one_ms_before_duration()` checks the last millisecond of visibility.
// Why:      A message is visible for exactly its duration, so one millisecond before the boundary it must remain.
#[test]
fn expire_keeps_message_one_ms_before_duration() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("e"), START_MS)
        .expire(START_MS + FOUR_SECONDS_MS - 1);
    assert_eq!(queue.visible(), [error_message("e")]);
}

// What:     `#[test] fn expire_removes_message_at_duration()` checks the first millisecond after the boundary.
// Why:      At exactly the full visible time the message is gone, so the expiry boundary is inclusive.
#[test]
fn expire_removes_message_at_duration() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("e"), START_MS)
        .expire(START_MS + FOUR_SECONDS_MS);
    assert!(queue.visible().is_empty());
}

// What:     `#[test] fn expire_keeps_message_when_clock_is_before_show()` checks a clock value earlier than the show time.
// Why:      A caller clock that reads before the show time must not expire the message immediately.
#[test]
fn expire_keeps_message_when_clock_is_before_show() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("e"), START_MS)
        .expire(START_MS - 1);
    assert_eq!(queue.visible(), [error_message("e")]);
}

// What:     `#[test] fn expire_uses_per_message_duration()` checks that a custom duration expires only its own message.
// Why:      Each entry carries its own timer, so a short-lived message does not drag the others with it.
#[test]
fn expire_uses_per_message_duration() {
    let base = FeedbackQueue::EMPTY.show(error_message("e"), START_MS);
    let queue = show_timed(&base, undo_message("u"), START_MS, ONE_SECOND_MS)
        .expire(START_MS + ONE_SECOND_MS);
    assert_eq!(queue.visible(), [error_message("e")]);
}

// What:     `#[test] fn repeated_id_resets_timer()` checks that showing the same id again restarts its timer.
// Why:      A repeated show with the same id is a refresh, so the message must not expire on the first timer.
#[test]
fn repeated_id_resets_timer() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("e"), START_MS)
        .show(error_message("e"), START_MS + THREE_SECONDS_MS)
        .expire(START_MS + FOUR_SECONDS_MS);
    assert_eq!(queue.visible(), [error_message("e")]);
}

// What:     `#[test] fn repeated_id_across_kinds_keeps_one_entry()` checks that one id never appears twice.
// Why:      Dismissal and actions name a message by id, so two visible messages with one id would be ambiguous.
#[test]
fn repeated_id_across_kinds_keeps_one_entry() {
    let queue = FeedbackQueue::EMPTY
        .show(error_message("x"), START_MS)
        .show(undo_message("x"), START_MS);
    assert_eq!(queue.visible(), [undo_message("x")]);
}

// What:     `#[test] fn next_expiry_null_when_empty()` checks that an empty queue asks for no timer.
// Why:      The wiring step must not schedule a wake-up when nothing is visible.
#[test]
fn next_expiry_null_when_empty() {
    assert_eq!(FeedbackQueue::EMPTY.next_expiry_ms(), None);
}

// What:     `#[test] fn next_expiry_is_earliest_entry()` checks that the reported expiry is the soonest one.
// Why:      A single timer set to the earliest expiry wakes the caller in time for every message.
#[test]
fn next_expiry_is_earliest_entry() {
    let base = FeedbackQueue::EMPTY.show(error_message("e"), START_MS);
    let queue = show_timed(&base, undo_message("u"), START_MS + HALF_SECOND_MS, ONE_SECOND_MS);
    assert_eq!(queue.next_expiry_ms(), Some(START_MS + HALF_SECOND_MS + ONE_SECOND_MS));
}

// What:     `#[test] fn error_default_duration_is_four_seconds()` checks the default error duration.
// Why:      The accepted study used the Short duration, which is 4000 ms.
#[test]
fn error_default_duration_is_four_seconds() {
    assert_eq!(FeedbackKind::Error.default_duration_ms(), FOUR_SECONDS_MS);
}

// What:     `#[test] fn undo_default_duration_is_four_seconds()` checks the default Undo duration.
// Why:      The study showed the Undo message with the same Short duration as the error message.
#[test]
fn undo_default_duration_is_four_seconds() {
    assert_eq!(FeedbackKind::Undo.default_duration_ms(), FOUR_SECONDS_MS);
}

// What:     `#[test] fn blank_id_rejected()` checks that a message id cannot be blank.
// Why:      A blank id cannot be dismissed reliably, so the message is refused at construction.
#[test]
fn blank_id_rejected() {
    let error = FeedbackMessage::new(" ", "Error", FeedbackKind::Error, None)
        .expect_err("a blank id must be rejected");
    assert_eq!(error.to_string(), "Feedback message id must not be blank.");
}

// What:     `#[test] fn blank_text_rejected()` checks that a message text cannot be blank.
// Why:      An empty message would show a blank bar, which the user cannot read.
#[test]
fn blank_text_rejected() {
    let error = FeedbackMessage::new("e", "", FeedbackKind::Error, None)
        .expect_err("a blank text must be rejected");
    assert_eq!(error.to_string(), "Feedback message text must not be blank.");
}

// What:     `#[test] fn blank_action_label_rejected()` checks that a present action label cannot be blank.
// Why:      A blank button would be unlabeled, so the caller must pass None for no action.
#[test]
fn blank_action_label_rejected() {
    let error = FeedbackMessage::new("u", "Moved", FeedbackKind::Undo, Some(" "))
        .expect_err("a blank action label must be rejected");
    assert_eq!(
        error.to_string(),
        "Feedback action label must not be blank when present."
    );
}

// What:     `#[test] fn zero_duration_rejected()` checks that a show call cannot pass a zero visible time.
// Why:      A zero duration would expire the message at the moment it appears.
#[test]
fn zero_duration_rejected() {
    let error = FeedbackQueue::EMPTY
        .show_for(error_message("e"), START_MS, 0)
        .expect_err("a zero duration must be rejected");
    assert_eq!(
        error.to_string(),
        "Feedback duration must be between 1 and 60000 ms, was 0."
    );
}

// What:     `#[test] fn duration_above_bound_rejected()` checks that a show call cannot exceed the transient bound.
// Why:      Durations past the bound would leave a message on screen for an unbounded time.
#[test]
fn duration_above_bound_rejected() {
    let error = FeedbackQueue::EMPTY
        .show_for(error_message("e"), START_MS, FEEDBACK_MAX_DURATION_MS + 1)
        .expect_err("a duration above the bound must be rejected");
    assert_eq!(
        error.to_string(),
        "Feedback duration must be between 1 and 60000 ms, was 60001."
    );
}

// What:     `#[test] fn transitions_leave_older_queue_unchanged()` checks that every transition returns a new value.
// Why:      Immutable queue values let the player keep the previous state for comparison and rollback.
#[test]
fn transitions_leave_older_queue_unchanged() {
    let before = FeedbackQueue::EMPTY.show(error_message("e"), START_MS);
    let _after_show = before.show(undo_message("u"), START_MS);
    let _after_dismiss = before.dismiss("e");
    let _after_expire = before.expire(START_MS + FOUR_SECONDS_MS);
    assert_eq!(before.visible(), [error_message("e")]);
}
