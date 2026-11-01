// What:     `package dev.monochromatic.musicplayer` places this test beside the feedback host it checks.
// Why:      Same-package tests call `FeedbackHostState` and `feedbackFolderOpenFailed` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import dev.monochromatic.musicplayer.core.FeedbackKind` and the message types the tests build.
// Why:      The tests build messages of each kind and check the queue that the host keeps.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { FeedbackKind, FeedbackMessage } from "./core/FeedbackQueue";
// ```
import dev.monochromatic.musicplayer.core.FeedbackKind
import dev.monochromatic.musicplayer.core.FeedbackMessage

// What:     JUnit assertions and the test annotation register each host check.
// Why:      Every queue state is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlinx.coroutines.runBlocking

// What:     `class FeedbackHostStateTest` checks that the host shows, dismisses, and expires messages on the clock it
//           is given.
// Why:      The host is the one place where producers, the overlay, and the expiry effect meet, so its state is checked
//           here.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("FeedbackHostState", () => { ... });
// ```
/** Verifies the host's show, dismiss, order, folder message, and expiry behavior with an injected clock. */
class FeedbackHostStateTest {
    // What:     `private var nowMs: Long` is the test clock that the host reads.
    // Why:      Tests move time explicitly, so expiry is checked without real waiting.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let nowMs = 0;
    // ```
    /** Clock value in milliseconds that the host reads for each show and expiry. */
    private var nowMs: Long = 0L

    // What:     `private fun errorMessage(id: String): FeedbackMessage` builds an error message with the given id.
    // Why:      Each test needs an error message, and the id decides which message a dismiss removes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const errorMessage = (id: string): FeedbackMessage => ({ id, text: "Failed", kind: "ERROR", actionLabel: null });
    // ```
    /** Returns an error message with the given id and fixed text. */
    private fun errorMessage(id: String): FeedbackMessage = FeedbackMessage(
        id = id,
        text = "Failed",
        kind = FeedbackKind.ERROR,
        actionLabel = null,
    )

    // What:     `private fun undoMessage(id: String): FeedbackMessage` builds an Undo message with the given id.
    // Why:      The order test needs an Undo message beside the error message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const undoMessage = (id: string): FeedbackMessage => ({ id, text: "Removed", kind: "UNDO", actionLabel: "Undo"
    //           });
    // ```
    /** Returns an Undo message with the given id and an Undo action label. */
    private fun undoMessage(id: String): FeedbackMessage = FeedbackMessage(
        id = id,
        text = "Removed",
        kind = FeedbackKind.UNDO,
        actionLabel = "Undo",
    )

    // What:     `fun showMakesMessageVisibleAtClock` checks that a shown message is visible and expires on its time.
    // Why:      The expiry effect reads the next expiry, so it must be the show time plus the default visible time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("show", () => { host.showFeedback(err); expect(host.queue.visible()).toHaveLength(1); });
    // ```
    /** Confirms that a shown message is visible and that its expiry is the show time plus the default duration. */
    @Test
    fun showMakesMessageVisibleAtClock() {
        nowMs = 1_000L
        val host = FeedbackHostState(clockMs = { nowMs })
        host.showFeedback(errorMessage("error-a"))
        assertEquals(listOf("error-a"), host.queue.visible().map { message -> message.id })
        assertEquals(5_000L, host.queue.nextExpiryMs())
    }

    // What:     `fun dismissRemovesOnlyNamedMessage` checks that dismiss removes only the message with its id.
    // Why:      The user dismisses one message from the overlay, and the other message must stay visible.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("dismiss", () => { host.dismiss("undo-a"); expect(ids).toEqual(["error-a"]); });
    // ```
    /** Confirms that dismissing one id leaves the other visible message in place. */
    @Test
    fun dismissRemovesOnlyNamedMessage() {
        nowMs = 0L
        val host = FeedbackHostState(clockMs = { nowMs })
        host.showFeedback(undoMessage("undo-a"))
        host.showFeedback(errorMessage("error-a"))
        host.dismiss("undo-a")
        assertEquals(listOf("error-a"), host.queue.visible().map { message -> message.id })
    }

    // What:     `fun visibleOrdersUndoAboveError` checks the display order of the two kinds.
    // Why:      The design puts Undo above Error whatever the order of the shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("order", () => expect(visibleIds).toEqual(["undo-a", "error-a"]));
    // ```
    /** Confirms that Undo is drawn above Error even when Error was shown first. */
    @Test
    fun visibleOrdersUndoAboveError() {
        nowMs = 0L
        val host = FeedbackHostState(clockMs = { nowMs })
        host.showFeedback(errorMessage("error-a"))
        host.showFeedback(undoMessage("undo-a"))
        assertEquals(listOf("undo-a", "error-a"), host.queue.visible().map { message -> message.id })
    }

    // What:     `fun expireDropsMessagesPastTheirTime` checks that expiry removes a message once its time has passed.
    // Why:      A message must leave the overlay on its own, and the expiry loop returns at once when the time has
    //           passed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("expire", async () => { await host.expireAt(5000); expect(host.queue.visible()).toEqual([]); });
    // ```
    /** Confirms that a message is removed once the clock has moved past its expiry. */
    @Test
    fun expireDropsMessagesPastTheirTime() {
        nowMs = 1_000L
        val host = FeedbackHostState(clockMs = { nowMs })
        host.showFeedback(errorMessage("error-a"))
        nowMs = 6_000L
        runBlocking {
            host.expireAt(expiryMs = 5_000L)
        }
        assertTrue(host.queue.visible().isEmpty())
        assertNull(host.queue.nextExpiryMs())
    }

    // What:     `fun folderOpenFailedIsErrorWithoutAction` checks the folder error message.
    // Why:      The folder error is a plain error with no button, and its text must be present.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder error", () => expect(feedbackFolderOpenFailed().kind).toBe("ERROR"));
    // ```
    /** Confirms that the folder error is an error message with no action and non-blank text. */
    @Test
    fun folderOpenFailedIsErrorWithoutAction() {
        val message: FeedbackMessage = feedbackFolderOpenFailed()
        assertEquals(FeedbackKind.ERROR, message.kind)
        assertNull(message.actionLabel)
        assertTrue(message.text.isNotBlank())
    }
}
