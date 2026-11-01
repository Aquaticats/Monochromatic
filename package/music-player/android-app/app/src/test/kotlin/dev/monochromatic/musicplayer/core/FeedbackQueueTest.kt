// What:     `package dev.monochromatic.musicplayer.core` places this test beside the feedback queue it verifies.
// Why:      The test reaches the public queue API directly and needs no Compose runtime.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each queue transition is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test

// What:     `private const val START_MS: Long = 10_000L` is the clock value at which the test messages appear.
// Why:      A non-zero start proves that expiry uses elapsed time rather than the absolute clock value.
//
// In TS you'd write (pseudocode):
// ```ts
// const START_MS = 10000;
// ```
/** Clock value in milliseconds at which the first test message is shown. */
private const val START_MS: Long = 10_000L

// What:     `private const val HALF_SECOND_MS: Long = 500L` is a short offset after the first show.
// Why:      A second message shown part way through the first one's lifetime exercises replacement timing.
//
// In TS you'd write (pseudocode):
// ```ts
// const HALF_SECOND_MS = 500;
// ```
/** Offset in milliseconds after the first show at which a second message appears. */
private const val HALF_SECOND_MS: Long = 500L

// What:     `private const val ONE_SECOND_MS: Long = 1_000L` is a short custom visible time.
// Why:      A duration shorter than the default proves that each entry keeps its own timer.
//
// In TS you'd write (pseudocode):
// ```ts
// const ONE_SECOND_MS = 1000;
// ```
/** Custom visible time in milliseconds used for a message that expires before the default. */
private const val ONE_SECOND_MS: Long = 1_000L

// What:     `private const val THREE_SECONDS_MS: Long = 3_000L` is an offset that leaves under the default time.
// Why:      A re-show at this offset must restart the timer rather than inherit the earlier show time.
//
// In TS you'd write (pseudocode):
// ```ts
// const THREE_SECONDS_MS = 3000;
// ```
/** Offset in milliseconds after the first show at which the same message is shown again. */
private const val THREE_SECONDS_MS: Long = 3_000L

// What:     `private const val FOUR_SECONDS_MS: Long = 4_000L` is the expected default visible time.
// Why:      The default-duration tests compare against this literal, so the accepted value is stated once.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOUR_SECONDS_MS = 4000;
// ```
/** Expected default visible time in milliseconds, matching the study's Short duration. */
private const val FOUR_SECONDS_MS: Long = 4_000L

// What:     `private fun errorMessage(id: String): FeedbackMessage` builds one error message without an action.
// Why:      Each test names only the id it cares about, so the other fields stay identical across tests.
//
// In TS you'd write (pseudocode):
// ```ts
// function errorMessage(id: string): FeedbackMessage { ... }
// ```
/** Builds an error message with the given id and no action. */
private fun errorMessage(id: String): FeedbackMessage =
    FeedbackMessage(id = id, text = "Error $id", kind = FeedbackKind.ERROR, actionLabel = null)

// What:     `private fun undoMessage(id: String): FeedbackMessage` builds one Undo message with an Undo action.
// Why:      Undo tests need an action label so they match the production shape of the message.
//
// In TS you'd write (pseudocode):
// ```ts
// function undoMessage(id: string): FeedbackMessage { ... }
// ```
/** Builds an Undo message with the given id and an Undo action label. */
private fun undoMessage(id: String): FeedbackMessage =
    FeedbackMessage(id = id, text = "Moved $id to trash", kind = FeedbackKind.UNDO, actionLabel = "Undo")

/** Builds a notice message with the given id and no action. */
private fun noticeMessage(id: String): FeedbackMessage =
    FeedbackMessage(id = id, text = "Notice $id", kind = FeedbackKind.NOTICE, actionLabel = null)

/** Every order in which the three invented messages (one per kind) can be shown. */
private fun showOrdersOfThree(): List<List<FeedbackMessage>> {
    /** One message of each declared kind, in declaration order. */
    val messages: List<FeedbackMessage> = listOf(errorMessage("e"), undoMessage("u"), noticeMessage("n"))
    return messages.flatMap { first ->
        messages.filter { second -> second != first }.map { second ->
            listOf(first, second, messages.first { third -> third != first && third != second })
        }
    }
}

// What:     `class FeedbackQueueTest` groups the queue transition tests.
// Why:      Replacement, dismissal, expiry, ordering and validation are each checked on the host JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("FeedbackQueue", () => { ... });
// ```
/** Verifies the transient feedback queue without a Compose runtime or a real clock. */
class FeedbackQueueTest {
    // What:     `showAddsMessage` checks that one shown message becomes visible.
    // Why:      Every other transition builds on a queue that holds the message after show.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("show adds a message", () => { ... });
    // ```
    /** Confirms a single shown message is the only visible message. */
    @Test
    fun showAddsMessage() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY.show(errorMessage("a"), nowMs = START_MS)
        assertEquals(listOf(errorMessage("a")), queue.visible())
    }

    // What:     `showReplacesSameKind` checks that a second error replaces the first error.
    // Why:      Only one error may be visible, so the newest failure is the one the user reads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a new error replaces the visible error", () => { ... });
    // ```
    /** Confirms that showing a second error removes the first one. */
    @Test
    fun showReplacesSameKind() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("a"), nowMs = START_MS)
            .show(errorMessage("b"), nowMs = START_MS)
        assertEquals(listOf(errorMessage("b")), queue.visible())
    }

    // What:     `replacementRestartsTimer` checks that the replacing message expires on its own schedule.
    // Why:      A replaced message must not keep its old expiry, or the newer message would vanish early.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("replacement restarts the timer", () => { ... });
    // ```
    /** Confirms a replacement shown later stays visible after the first message's time would have elapsed. */
    @Test
    fun replacementRestartsTimer() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("a"), nowMs = START_MS)
            .show(errorMessage("b"), nowMs = START_MS + HALF_SECOND_MS)
            .expire(nowMs = START_MS + FEEDBACK_ERROR_DURATION_MS)
        assertEquals(listOf(errorMessage("b")), queue.visible())
    }

    // What:     `undoAndErrorBothVisible` checks that one message of each kind can be visible together.
    // Why:      D29 and D83 keep the Undo message and the error message as independent owners.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("undo and error are both visible", () => { ... });
    // ```
    /** Confirms an Undo message and an error message remain visible at the same time. */
    @Test
    fun undoAndErrorBothVisible() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .show(undoMessage("u"), nowMs = START_MS)
        assertEquals(listOf(undoMessage("u"), errorMessage("e")), queue.visible())
    }

    // What:     `visibleOrdersUndoAboveError` checks display order when the error is shown first.
    // Why:      D29 places the Undo message above the error bar regardless of show order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("undo is listed before error", () => { ... });
    // ```
    /** Confirms the Undo message is listed first even though the error was shown first. */
    @Test
    fun visibleOrdersUndoAboveError() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .show(undoMessage("u"), nowMs = START_MS)
        assertEquals("u", queue.visible().first().id)
    }

    // What:     `dismissRemovesById` checks that dismissal removes only the named message.
    // Why:      The other visible message keeps its place and its timer after one is dismissed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("dismiss removes one message by id", () => { ... });
    // ```
    /** Confirms dismissing one id leaves the other visible message in place. */
    @Test
    fun dismissRemovesById() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .show(undoMessage("u"), nowMs = START_MS)
            .dismiss("e")
        assertEquals(listOf(undoMessage("u")), queue.visible())
    }

    // What:     `dismissUnknownIdKeepsQueue` checks that an unknown id changes nothing.
    // Why:      A dismiss that arrives after expiry must not remove a message that is still visible.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("dismiss of an unknown id is a no-op", () => { ... });
    // ```
    /** Confirms dismissing an id that is not visible leaves the visible messages unchanged. */
    @Test
    fun dismissUnknownIdKeepsQueue() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY.show(errorMessage("e"), nowMs = START_MS)
        assertEquals(listOf(errorMessage("e")), queue.dismiss("missing").visible())
    }

    // What:     `staleDismissKeepsReplacement` checks a dismiss aimed at a replaced message.
    // Why:      A late dismiss for the old error must not hide the newer error the user has not read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("stale dismiss does not remove the replacement", () => { ... });
    // ```
    /** Confirms dismissing the id of a replaced message keeps its replacement visible. */
    @Test
    fun staleDismissKeepsReplacement() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("old"), nowMs = START_MS)
            .show(errorMessage("new"), nowMs = START_MS)
            .dismiss("old")
        assertEquals(listOf(errorMessage("new")), queue.visible())
    }

    // What:     `expireKeepsMessageOneMsBeforeDuration` checks the last millisecond of visibility.
    // Why:      A message is visible for exactly its duration, so one millisecond before the boundary it must remain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("message is visible one millisecond before its duration", () => { ... });
    // ```
    /** Confirms a message still shows one millisecond before its full visible time has elapsed. */
    @Test
    fun expireKeepsMessageOneMsBeforeDuration() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .expire(nowMs = START_MS + FEEDBACK_ERROR_DURATION_MS - 1L)
        assertEquals(listOf(errorMessage("e")), queue.visible())
    }

    // What:     `expireRemovesMessageAtDuration` checks the first millisecond after the boundary.
    // Why:      At exactly the full visible time the message must be gone, so the boundary is inclusive for expiry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("message expires at its duration", () => { ... });
    // ```
    /** Confirms a message is removed once its full visible time has elapsed. */
    @Test
    fun expireRemovesMessageAtDuration() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .expire(nowMs = START_MS + FEEDBACK_ERROR_DURATION_MS)
        assertEquals(emptyList<FeedbackMessage>(), queue.visible())
    }

    // What:     `expireKeepsMessageWhenClockIsBeforeShow` checks a clock value earlier than the show time.
    // Why:      A caller clock that reads before the show time must not expire the message immediately.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("earlier clock does not expire", () => { ... });
    // ```
    /** Confirms a clock value before the show time keeps the message visible. */
    @Test
    fun expireKeepsMessageWhenClockIsBeforeShow() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .expire(nowMs = START_MS - 1L)
        assertEquals(listOf(errorMessage("e")), queue.visible())
    }

    // What:     `expireUsesPerMessageDuration` checks that a custom duration expires only its own message.
    // Why:      The wiring step may pass adjusted durations, so each entry must carry its own timer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("each message expires on its own duration", () => { ... });
    // ```
    /** Confirms a short-lived Undo message expires while a default-length error message stays visible. */
    @Test
    fun expireUsesPerMessageDuration() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .show(undoMessage("u"), nowMs = START_MS, durationMs = ONE_SECOND_MS)
            .expire(nowMs = START_MS + ONE_SECOND_MS)
        assertEquals(listOf(errorMessage("e")), queue.visible())
    }

    // What:     `repeatedIdResetsTimer` checks that showing the same id again restarts its timer.
    // Why:      A repeated show with the same id is a refresh, so the message must not expire on the first timer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("re-showing an id resets its timer", () => { ... });
    // ```
    /** Confirms a message re-shown under the same id stays visible past its first expiry. */
    @Test
    fun repeatedIdResetsTimer() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .show(errorMessage("e"), nowMs = START_MS + THREE_SECONDS_MS)
            .expire(nowMs = START_MS + FEEDBACK_ERROR_DURATION_MS)
        assertEquals(listOf(errorMessage("e")), queue.visible())
    }

    // What:     `repeatedIdAcrossKindsKeepsOneEntry` checks that one id never appears twice.
    // Why:      Dismissal and actions name a message by id, so two visible messages with one id would be ambiguous.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one id, one visible message", () => { ... });
    // ```
    /** Confirms showing an error and then an Undo message with the same id leaves only the Undo message. */
    @Test
    fun repeatedIdAcrossKindsKeepsOneEntry() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("x"), nowMs = START_MS)
            .show(undoMessage("x"), nowMs = START_MS)
        assertEquals(listOf(undoMessage("x")), queue.visible())
    }

    // What:     `nextExpiryNullWhenEmpty` checks that an empty queue asks for no timer.
    // Why:      The wiring step must not schedule a wake-up when nothing is visible.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no expiry for an empty queue", () => { ... });
    // ```
    /** Confirms the next expiry of an empty queue is null. */
    @Test
    fun nextExpiryNullWhenEmpty() {
        assertNull(FeedbackQueue.EMPTY.nextExpiryMs())
    }

    // What:     `nextExpiryIsEarliestEntry` checks that the reported expiry is the soonest one.
    // Why:      A single timer set to the earliest expiry wakes the caller in time for every message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("next expiry is the earliest entry", () => { ... });
    // ```
    /** Confirms the next expiry is the earliest of the visible entries' expiry times. */
    @Test
    fun nextExpiryIsEarliestEntry() {
        /** Queue value produced by the chained transitions under test. */
        val queue = FeedbackQueue.EMPTY
            .show(errorMessage("e"), nowMs = START_MS)
            .show(undoMessage("u"), nowMs = START_MS + HALF_SECOND_MS, durationMs = ONE_SECOND_MS)
        assertEquals(START_MS + HALF_SECOND_MS + ONE_SECOND_MS, queue.nextExpiryMs())
    }

    // What:     `errorDefaultDurationIsFourSeconds` checks the default error duration.
    // Why:      The accepted study used the Short duration, which is 4000 ms in the shipped material3 version.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("error default is 4000 ms", () => { ... });
    // ```
    /** Confirms the default error visible time equals the study's Short duration. */
    @Test
    fun errorDefaultDurationIsFourSeconds() {
        assertEquals(FOUR_SECONDS_MS, FeedbackKind.ERROR.defaultDurationMs)
    }

    // What:     `undoDefaultDurationIsFourSeconds` checks the default Undo duration.
    // Why:      The study showed the Undo message with the same Short duration as the error message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("undo default is 4000 ms", () => { ... });
    // ```
    /** Confirms the default Undo visible time equals the study's Short duration. */
    @Test
    fun undoDefaultDurationIsFourSeconds() {
        assertEquals(FOUR_SECONDS_MS, FeedbackKind.UNDO.defaultDurationMs)
    }

    // What:     `blankIdRejected` checks that a message id cannot be blank.
    // Why:      A blank id cannot be dismissed reliably, so the message is refused at construction.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank id throws", () => { ... });
    // ```
    /** Confirms constructing a message with a blank id throws. */
    @Test
    fun blankIdRejected() {
        assertThrows(IllegalArgumentException::class.java) {
            FeedbackMessage(id = " ", text = "Error", kind = FeedbackKind.ERROR, actionLabel = null)
        }
    }

    // What:     `blankTextRejected` checks that a message text cannot be blank.
    // Why:      An empty message would show a blank bar, which the user cannot read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank text throws", () => { ... });
    // ```
    /** Confirms constructing a message with blank text throws. */
    @Test
    fun blankTextRejected() {
        assertThrows(IllegalArgumentException::class.java) {
            FeedbackMessage(id = "e", text = "", kind = FeedbackKind.ERROR, actionLabel = null)
        }
    }

    // What:     `blankActionLabelRejected` checks that a present action label cannot be blank.
    // Why:      A blank button would be unlabeled, so the caller must pass null for no action.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank action label throws", () => { ... });
    // ```
    /** Confirms constructing a message with a blank action label throws. */
    @Test
    fun blankActionLabelRejected() {
        assertThrows(IllegalArgumentException::class.java) {
            FeedbackMessage(id = "u", text = "Moved", kind = FeedbackKind.UNDO, actionLabel = " ")
        }
    }

    // What:     `zeroDurationRejected` checks that a show call cannot pass a zero visible time.
    // Why:      A zero duration would expire the message at the moment it appears.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("zero duration throws", () => { ... });
    // ```
    /** Confirms showing a message with a zero visible time throws. */
    @Test
    fun zeroDurationRejected() {
        assertThrows(IllegalArgumentException::class.java) {
            FeedbackQueue.EMPTY.show(errorMessage("e"), nowMs = START_MS, durationMs = 0L)
        }
    }

    // What:     `durationAboveBoundRejected` checks that a show call cannot exceed the transient bound.
    // Why:      Durations past the bound would leave a message on screen for an unbounded time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("duration above the bound throws", () => { ... });
    // ```
    /** Confirms showing a message with a visible time above the bound throws. */
    @Test
    fun durationAboveBoundRejected() {
        assertThrows(IllegalArgumentException::class.java) {
            FeedbackQueue.EMPTY.show(errorMessage("e"), nowMs = START_MS, durationMs = FEEDBACK_MAX_DURATION_MS + 1L)
        }
    }

    // What:     `transitionsLeaveOlderQueueUnchanged` checks that every transition returns a new value.
    // Why:      Immutable queue values let the player keep the previous state for comparison and rollback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("older queue is unchanged", () => { ... });
    // ```
    /** Confirms show, dismiss and expire leave the queue they were called on unchanged. */
    @Test
    fun transitionsLeaveOlderQueueUnchanged() {
        /** Queue value that the later transitions in this test must leave unchanged. */
        val before = FeedbackQueue.EMPTY.show(errorMessage("e"), nowMs = START_MS)
        before.show(undoMessage("u"), nowMs = START_MS)
        before.dismiss("e")
        before.expire(nowMs = START_MS + FEEDBACK_ERROR_DURATION_MS)
        assertEquals(listOf(errorMessage("e")), before.visible())
    }

    // What:     `visibleOrdersEveryKindWhateverTheShowOrder` checks the display order of one message of each kind.
    // Why:      The order table once left out the notice kind, so a notice sat above the Undo message by accident;
    //           the documented order is Undo, then notice, then error, whichever message was shown first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("every kind has one display order", () => { ... });
    // ```
    /** Confirms the three kinds are listed Undo, notice, error for each of the six orders they can be shown in. */
    @Test
    fun visibleOrdersEveryKindWhateverTheShowOrder() {
        for (shown in showOrdersOfThree()) {
            /** Queue holding the three messages, shown in the order under test. */
            val queue: FeedbackQueue = shown.fold(FeedbackQueue.EMPTY) { acc, message -> acc.show(message, nowMs = START_MS) }
            assertEquals(
                "shown as ${shown.map { message -> message.id }}",
                listOf("u", "n", "e"),
                queue.visible().map { message -> message.id },
            )
        }
    }

    // What:     `displayOrderNamesEveryDeclaredKindOnce` checks that the order table and the enum agree.
    // Why:      A kind added to the enum without a place in the table would get an accidental position again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("display order lists every kind once", () => { ... });
    // ```
    /** Confirms the display order holds each declared feedback kind exactly once. */
    @Test
    fun displayOrderNamesEveryDeclaredKindOnce() {
        assertEquals(FeedbackKind.entries.toSet(), FEEDBACK_DISPLAY_ORDER.toSet())
        assertEquals(FeedbackKind.entries.size, FEEDBACK_DISPLAY_ORDER.size)
    }
}
