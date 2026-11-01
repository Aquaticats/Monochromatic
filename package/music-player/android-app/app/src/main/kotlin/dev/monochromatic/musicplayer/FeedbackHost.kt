// What:     `package dev.monochromatic.musicplayer` places the feedback host beside the player host it feeds.
// Why:      The host state is created by the activity and drawn by the player host, both in this package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.os.SystemClock` reads the monotonic clock that times each message.
// Why:      Show and expiry use one clock that does not jump when the wall clock changes.
//
// In TS you'd write (pseudocode):
// ```ts
// import { now } from "performance";
// ```
import android.os.SystemClock

// What:     `import androidx.compose.runtime.getValue` and `setValue` let the delegated queue be read and written.
// Why:      The queue is a Compose state, so the player host redraws the overlay when a message appears or leaves.
//
// In TS you'd write (pseudocode):
// ```ts
// // A delegated property backed by an observable value.
// ```
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue

// What:     `import androidx.compose.runtime.mutableStateOf` creates the observable queue.
// Why:      The queue must notify composition when a message is shown, dismissed, or expired.
//
// In TS you'd write (pseudocode):
// ```ts
// import { observable } from "mobx";
// ```
import androidx.compose.runtime.mutableStateOf

// What:     `import dev.monochromatic.musicplayer.core.FeedbackKind` and the queue types the host stores and shows.
// Why:      The pure queue rules live in core and are shared with the tests.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { FeedbackKind, FeedbackMessage, FeedbackQueue } from "./core/FeedbackQueue";
// ```
import dev.monochromatic.musicplayer.core.FeedbackKind
import dev.monochromatic.musicplayer.core.FeedbackMessage
import dev.monochromatic.musicplayer.core.FeedbackQueue

// What:     `import kotlinx.coroutines.delay` suspends the expiry loop until a message is due.
// Why:      Expiry waits without blocking the main thread, and the loop rechecks the clock after each wait.
//
// In TS you'd write (pseudocode):
// ```ts
// import { setTimeout } from "node:timers/promises";
// ```
import kotlinx.coroutines.delay

// What:     `private const val FEEDBACK_HOST_FOLDER_ERROR_ID: String` is the id of the folder error message.
// Why:      A fixed id lets a later message with the same id replace this one instead of stacking.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_HOST_FOLDER_ERROR_ID = "folder-open-failed";
// ```
/** Id of the message that reports a chosen folder that could not be kept for later launches. */
private const val FEEDBACK_HOST_FOLDER_ERROR_ID: String = "folder-open-failed"

// What:     `interface FeedbackHost` is the producer-facing entry point for transient messages.
// Why:      Producers that run outside composition, such as the activity's folder result, need one method to call
//           without knowing how the queue is drawn or expired.
//
// In TS you'd write (pseudocode):
// ```ts
// interface FeedbackHost { showFeedback(message: FeedbackMessage): void }
// ```
/** Producer-facing entry point: any code holding the host can show one transient message above the player. */
interface FeedbackHost {
    // What:     `fun showFeedback(message: FeedbackMessage)` shows one message above the player.
    // Why:      The queue replaces any visible message of the same kind or id, so producers never stack duplicates.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // showFeedback(message: FeedbackMessage): void;
    // ```
    /** Shows one message above the player, replacing the visible message of its kind or of its id. */
    fun showFeedback(message: FeedbackMessage)
}

// What:     `class FeedbackHostState` owns the visible queue, shows and dismisses messages, and expires them on time.
// Why:      The activity creates it so non-composable producers can reach it, and the player host draws it; the clock
//           is injected so the queue rules can be tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// class FeedbackHostState implements FeedbackHost { queue: FeedbackQueue; ... }
// ```
/**
 * Holds the visible feedback queue. The activity creates it, the player host draws it and schedules expiry,
 * and producers outside composition call [showFeedback] through the [FeedbackHost] interface.
 *
 * @param clockMs monotonic clock in milliseconds that times each message, injected so tests control time
 */
class FeedbackHostState(
    /** Monotonic clock in milliseconds that times each message. */
    private val clockMs: () -> Long = { SystemClock.uptimeMillis() },
) : FeedbackHost {
    // What:     `var queue: FeedbackQueue by mutableStateOf(FeedbackQueue.EMPTY)` is the visible message set.
    // Why:      Composition reads it to draw the overlay and to key the expiry effect, so writes must be observable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // queue: FeedbackQueue = FeedbackQueue.EMPTY;
    // ```
    /** Messages currently visible, in the order they were shown. */
    var queue: FeedbackQueue by mutableStateOf(FeedbackQueue.EMPTY)
        private set

    // What:     `override fun showFeedback(message: FeedbackMessage)` shows one message at the current clock time.
    // Why:      The queue applies the replacement rule, and the clock value fixes when the message expires.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // showFeedback(message) { this.queue = this.queue.show(message, now()); }
    // ```
    /** Shows one message from the current clock time, replacing any visible message of its kind or id. */
    override fun showFeedback(message: FeedbackMessage) {
        queue = queue.show(message = message, nowMs = clockMs())
    }

    // What:     `fun dismiss(id: String)` removes the message with that id.
    // Why:      The user dismisses a message from the overlay, and the queue ignores an unknown id.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // dismiss(id: string): void { this.queue = this.queue.dismiss(id); }
    // ```
    /** Removes the message with the given id, leaving the other visible messages unchanged. */
    fun dismiss(id: String) {
        queue = queue.dismiss(id)
    }

    // What:     `suspend fun expireAt(expiryMs: Long)` waits until the expiry clock value and then drops due messages.
    // Why:      The loop rechecks the clock after each wait, because a delay can end a little early, so no message
    //           stays past its time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async expireAt(expiryMs: number): Promise<void> { /* wait until expiryMs, then expire */ }
    // ```
    /**
     * Suspends until the given clock value, then removes every message whose visible time has elapsed.
     *
     * @param expiryMs clock value at which the earliest visible message expires
     */
    suspend fun expireAt(expiryMs: Long) {
        /** Milliseconds left until the expiry clock value, which is zero or less once the time has passed. */
        var remainingMs: Long = expiryMs - clockMs()
        while (remainingMs > 0L) {
            delay(remainingMs)
            remainingMs = expiryMs - clockMs()
        }
        queue = queue.expire(clockMs())
    }
}

// What:     `fun feedbackFolderOpenFailed(): FeedbackMessage` builds the error for a folder that could not be kept.
// Why:      The folder open path reports one plain two-line error, and the text tells the user the two ways forward.
//
// In TS you'd write (pseudocode):
// ```ts
// function feedbackFolderOpenFailed(): FeedbackMessage { return { id: "folder-open-failed", kind: "ERROR", ... }; }
// ```
/** Returns the error message shown when a chosen folder cannot be kept for later launches. */
fun feedbackFolderOpenFailed(): FeedbackMessage = FeedbackMessage(
    id = FEEDBACK_HOST_FOLDER_ERROR_ID,
    text = "That folder can't be kept open. Choose it again, or pick another folder.",
    kind = FeedbackKind.ERROR,
    actionLabel = null,
)
