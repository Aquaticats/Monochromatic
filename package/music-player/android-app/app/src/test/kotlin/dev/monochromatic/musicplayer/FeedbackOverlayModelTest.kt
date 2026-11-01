// What:     `package dev.monochromatic.musicplayer` places this test beside the feedback overlay it verifies.
// Why:      The test reaches the module-internal pure helpers without exposing them publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the feedback kind type.
// Why:      The dismiss label tests enumerate the kinds the overlay draws.
//
// In TS you'd write (pseudocode):
// ```ts
// import { FeedbackKind } from "core/FeedbackQueue";
// ```
import dev.monochromatic.musicplayer.core.FeedbackKind

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each helper outcome is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class FeedbackOverlayModelTest` groups the pure helper tests for the feedback overlay.
// Why:      The dismiss label and the two-line rule can be verified on the host JVM without a Compose runtime.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("FeedbackOverlay helpers", () => { ... });
// ```
/** Verifies the pure helpers behind the feedback overlay without a Compose runtime. */
class FeedbackOverlayModelTest {
    // What:     `dismissDescriptionForError` checks the label of the error dismiss control.
    // Why:      Screen readers must name the message that the control closes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("error dismiss label", () => { ... });
    // ```
    /** Confirms the dismiss control for an error message is labeled as a message dismissal. */
    @Test
    fun dismissDescriptionForError() {
        assertEquals("Dismiss message", feedbackDismissDescription(FeedbackKind.ERROR))
    }

    // What:     `dismissDescriptionForUndo` checks the label of the Undo dismiss control.
    // Why:      The Undo dismiss label names the Undo message so it is not confused with the error control.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("undo dismiss label", () => { ... });
    // ```
    /** Confirms the dismiss control for an Undo message names the Undo message. */
    @Test
    fun dismissDescriptionForUndo() {
        assertEquals("Dismiss Undo message", feedbackDismissDescription(FeedbackKind.UNDO))
    }

    // What:     `oneLineIsWithinTextLimit` checks that a single measured line needs no log direction.
    // Why:      Short messages must keep their text rather than redirecting the user to logs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one line is within the limit", () => { ... });
    // ```
    /** Confirms a one-line message without overflow stays within the text limit. */
    @Test
    fun oneLineIsWithinTextLimit() {
        assertFalse(feedbackExceedsTextLimit(lineCount = 1, hasVisualOverflow = false))
    }

    // What:     `twoLinesAreWithinTextLimit` checks that exactly two lines is still allowed.
    // Why:      D83 says messages target two visible lines, so the limit is inclusive at two.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("two lines are within the limit", () => { ... });
    // ```
    /** Confirms a two-line message without overflow stays within the text limit. */
    @Test
    fun twoLinesAreWithinTextLimit() {
        assertFalse(feedbackExceedsTextLimit(lineCount = 2, hasVisualOverflow = false))
    }

    // What:     `threeLinesExceedTextLimit` checks that a third line triggers the log direction.
    // Why:      A message that needs a third line must be redirected to Android logs, not grown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("three lines exceed the limit", () => { ... });
    // ```
    /** Confirms a three-line message exceeds the text limit. */
    @Test
    fun threeLinesExceedTextLimit() {
        assertTrue(feedbackExceedsTextLimit(lineCount = 3, hasVisualOverflow = false))
    }

    // What:     `overflowWithTwoLinesExceedsTextLimit` checks that clipped text counts as exceeding the limit.
    // Why:      A two-line layout that still clips its text hides detail, so it must not pass as fitting.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("overflow exceeds the limit", () => { ... });
    // ```
    /** Confirms a two-line message with visual overflow exceeds the text limit. */
    @Test
    fun overflowWithTwoLinesExceedsTextLimit() {
        assertTrue(feedbackExceedsTextLimit(lineCount = 2, hasVisualOverflow = true))
    }

    // What:     `zeroLinesRejected` checks that an unmeasured layout is refused.
    // Why:      A zero line count is not evidence that the text fits, so the rule must not answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("zero lines throw", () => { ... });
    // ```
    /** Confirms a line count below one throws rather than reporting that the text fits. */
    @Test
    fun zeroLinesRejected() {
        assertThrows(IllegalArgumentException::class.java) {
            feedbackExceedsTextLimit(lineCount = 0, hasVisualOverflow = false)
        }
    }
}
