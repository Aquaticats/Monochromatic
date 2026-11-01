// Unit tests for the feedback text of the track file actions: the Undo message, the error messages, the line limit,
// and the naming of the affected file. These run on the host JVM with no device.

// What:     `package dev.monochromatic.musicplayer.core` places this test beside the messages it checks.
// Why:      The test reaches the public message builders and the failure enum in the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each message property is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `private const val SAMPLE_NAME: String` is the file name the message tests use.
// Why:      A fixed short name lets each test check that the name appears unchanged.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_NAME = "Song.mp3";
// ```
/** File name used by the message tests. */
private const val SAMPLE_NAME: String = "Song.mp3"

// What:     `private const val MAX_MESSAGE_LINES: Int` is the most visible lines a feedback message may use.
// Why:      The overlay is two lines by design (D83), and a longer message would grow the overlay.
//
// In TS you'd write (pseudocode):
// ```ts
// const MAX_MESSAGE_LINES = 2;
// ```
/** Largest number of visible lines a feedback message may use. */
private const val MAX_MESSAGE_LINES: Int = 2

// What:     `class TrackFileMessagesTest` groups the tests for the feedback text of the track actions.
// Why:      The wording, kind, action, and line count are part of the user-visible contract.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("track file messages", () => { ... });
// ```
/** Verifies the Undo message, the error messages, and the two-line limit for every failure. */
class TrackFileMessagesTest {
    // What:     `undoMessageNamesFileAndOffersUndo` checks the Undo message of a trash.
    // Why:      D8 and the brief fix this wording and action, so the test pins both.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("undo", () => expect(trashUndoMessage(record).text).toBe("Moved Song.mp3 to trash"));
    // ```
    /** Confirms the Undo message reads Moved name to trash and carries the Undo action. */
    @Test
    fun undoMessageNamesFileAndOffersUndo() {
        val message: FeedbackMessage = trashUndoMessage(sampleRecord())
        assertEquals("Moved Song.mp3 to trash", message.text)
        assertEquals("Undo", message.actionLabel)
        assertEquals(FeedbackKind.UNDO, message.kind)
    }

    // What:     `undoMessageIdIsRecordScoped` checks that the Undo message id belongs to one trash.
    // Why:      Dismissal and the Undo action find the record by this id, so it must not be shared.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("undo id", () => expect(trashUndoMessage(record("r9")).id).toBe("trash-undo-r9"));
    // ```
    /** Confirms the Undo message id is the record's message id. */
    @Test
    fun undoMessageIdIsRecordScoped() {
        assertEquals("trash-undo-r9", trashUndoMessage(sampleRecord(id = "r9")).id)
    }

    // What:     `undoMessageShortensLongName` checks an Undo message for a long file name.
    // Why:      A long name must not push the message past two visible lines.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("long undo", () => expect(trashUndoMessage(longRecord).text.length).toBeLessThan(80));
    // ```
    /** Confirms a long name is shortened inside the Undo text. */
    @Test
    fun undoMessageShortensLongName() {
        val longName: String = "x".repeat(200) + ".mp3"
        val message: FeedbackMessage = trashUndoMessage(sampleRecord(displayName = longName))
        assertTrue(message.text.length < longName.length)
        assertTrue(message.text.endsWith(" to trash"))
    }

    // What:     `errorMessageUsesFixedErrorId` checks that every track action error shares one id.
    // Why:      A fixed id lets the next error replace the visible one rather than stack a second.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("error id", () => expect(trackActionFailureMessage("COPY_FAILED", "a").id).toBe("track-action-error"));
    // ```
    /** Confirms every error message uses the shared error id. */
    @Test
    fun errorMessageUsesFixedErrorId() {
        for (failure in TrackActionFailure.entries) {
            assertEquals(TRASH_ERROR_ID, trackActionFailureMessage(failure, SAMPLE_NAME).id)
        }
    }

    // What:     `errorMessageIsErrorKindWithoutAction` checks the kind and action of every error message.
    // Why:      An error offers no action, and a wrong kind would route it to the Undo layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("error kind", () => expect(message.kind).toBe("ERROR"));
    // ```
    /** Confirms every error message is of the error kind with no action label. */
    @Test
    fun errorMessageIsErrorKindWithoutAction() {
        for (failure in TrackActionFailure.entries) {
            val message: FeedbackMessage = trackActionFailureMessage(failure, SAMPLE_NAME)
            assertEquals(FeedbackKind.ERROR, message.kind)
            assertNull(message.actionLabel)
        }
    }

    // What:     `everyErrorFitsTwoLinesAndNamesFile` checks the line count and the file name of every failure.
    // Why:      D83 limits feedback to two visible lines, and the user must see which file failed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("lines", () => for (const f of failures) { expect(text(f).split("\n").length).toBeLessThanOrEqual(2); });
    // ```
    /** Confirms every failure text has at most two lines and contains the file name. */
    @Test
    fun everyErrorFitsTwoLinesAndNamesFile() {
        for (failure in TrackActionFailure.entries) {
            val text: String = trackActionFailureMessage(failure, SAMPLE_NAME).text
            assertTrue("too many lines for $failure", text.lines().size <= MAX_MESSAGE_LINES)
            assertTrue("name missing for $failure", text.contains(SAMPLE_NAME))
        }
    }

    // What:     `errorNameIsShortenedWhenLong` checks an error for a very long file name.
    // Why:      A long name must still leave the message within two lines.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("long error", () => expect(trackActionFailureMessage("COPY_FAILED", "x".repeat(300)).text).toContain("…"));
    // ```
    /** Confirms a long name is shortened inside the error text. */
    @Test
    fun errorNameIsShortenedWhenLong() {
        val longName: String = "y".repeat(300)
        for (failure in TrackActionFailure.entries) {
            val text: String = trackActionFailureMessage(failure, longName).text
            assertTrue("name not shortened for $failure", text.contains(TRASH_ELLIPSIS))
            assertTrue("too many lines for $failure", text.lines().size <= MAX_MESSAGE_LINES)
        }
    }

    // What:     `copyFailureSaysNothingWasMoved` checks the wording that protects the user after a failed copy.
    // Why:      The user must learn that the file was not deleted, since the copy step runs first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("copy wording", () => expect(text("COPY_FAILED")).toContain("Nothing was moved to trash."));
    // ```
    /** Confirms the copy failure text states that nothing was moved to trash. */
    @Test
    fun copyFailureSaysNothingWasMoved() {
        assertTrue(trackActionFailureMessage(TrackActionFailure.COPY_FAILED, SAMPLE_NAME).text.contains("Nothing was moved"))
    }

    // What:     `sampleRecord(...)` builds a pending record for the message tests.
    // Why:      Each message test changes one name or id, so one helper keeps the baseline in one place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function sampleRecord(overrides?: Partial<TrashRecord>): TrashRecord { ... }
    // ```
    /** Builds a pending record with the given id and display name. */
    private fun sampleRecord(id: String = "r1", displayName: String = SAMPLE_NAME): TrashRecord = TrashRecord(
        id = id,
        displayName = displayName,
        route = TrashRoute.SAF_DELETE_WITH_COPY_BACK,
        trackUri = "content://a/tree/t/document/d",
        treeUri = "content://a/tree/t",
        parentDocumentId = "t:Music",
        mimeType = "audio/mpeg",
        copyPath = "/cache/trash/r1/Song.mp3",
        sizeBytes = 1L,
    )
}
