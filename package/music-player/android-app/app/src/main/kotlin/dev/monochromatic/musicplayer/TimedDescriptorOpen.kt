// Opening a track's file descriptor asks the media provider, which can be busy for seconds, so the open must never run
// on the main thread (M6 of the polish review: an ANR with the main thread inside
// `ContentResolver.openFileDescriptor`).

// What:     `package dev.monochromatic.musicplayer` places this helper beside the engine that uses it.
// Why:      It has no Android types, so a JVM unit test drives it with a fake opener.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import java.io.Closeable
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import java.util.concurrent.atomic.AtomicBoolean
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

// What:     `sealed interface DescriptorOutcome<out T : Closeable>` is the closed set of results of one timed open.
// Why:      The caller must handle a successful open, a refused open and a stalled open differently: the last one
//           gets its own message, and a late descriptor from it must still be closed.
//
// In TS you'd write (pseudocode):
// ```ts
// type DescriptorOutcome<T> = { kind: "opened"; descriptor: T } | { kind: "unavailable" } | { kind: "timedOut" };
// ```
/** What one timed descriptor open produced. */
sealed interface DescriptorOutcome<out T : Closeable> {
    /** The provider handed over a descriptor, which the receiver must close. */
    data class Opened<out T : Closeable>(
        /** Open descriptor. */
        val descriptor: T,
    ) : DescriptorOutcome<T>

    /** The provider refused or the file is gone, so no descriptor exists. */
    data object Unavailable : DescriptorOutcome<Nothing>

    /** The provider did not answer within the time limit; a late descriptor is closed by the opener. */
    data object TimedOut : DescriptorOutcome<Nothing>
}

// What:     `class TimedDescriptorOpen<T : Closeable>(...)` runs a blocking open on a background dispatcher and reports
//           its outcome through a callback, giving up after `timeoutMs`.
// Why:      The player's tap handler runs on the main thread; it starts an open here and continues when the outcome
//           arrives. A provider that stalls costs a message after the time limit and not a frozen screen.
//
// In TS you'd write (pseudocode):
// ```ts
// class TimedDescriptorOpen<T extends Closeable> {
//   open(uri: string, onOutcome: (o: DescriptorOutcome<T>) => void): void {
//     let settled = false;
//     setTimeout(() => { if (!settled) { settled = true; onOutcome({ kind: "timedOut" }); } }, this.timeoutMs);
//     const found = this.opener(uri);                          // blocks a worker, not the caller
//     if (!settled) { settled = true; onOutcome(found ? { kind: "opened", descriptor: found } : unavailable); }
//     else found?.close();                                     // too late: nobody will receive it
//   }
// }
// ```
/**
 * Opens descriptors off the calling thread with a time limit.
 *
 * @param dispatcher where the blocking open runs; never the main thread
 * @param timeoutMs how long to wait for the provider before reporting [DescriptorOutcome.TimedOut]
 * @param opener the blocking open, returning null when the provider cannot open the file
 * @param report receives an exception the opener threw, which is then treated as an unavailable file
 */
class TimedDescriptorOpen<T : Closeable>(
    dispatcher: CoroutineDispatcher,
    private val timeoutMs: Long,
    private val opener: (String) -> T?,
    private val report: (uri: String, failure: Exception) -> Unit,
) {
    /** Scope that owns every pending open; independent children so one stalled open never cancels another. */
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + dispatcher)

    /**
     * Starts opening [uri] and returns at once; [onOutcome] runs later on a background thread.
     *
     * @param uri track location to open
     * @param onOutcome receives the result exactly once; the receiver closes an opened descriptor
     */
    fun open(uri: String, onOutcome: (DescriptorOutcome<T>) -> Unit) {
        /** Set by whichever of the open and the watchdog finishes first, so only one of them reports. */
        val settled = AtomicBoolean(false)
        /** Reports the stall after the time limit unless the open finished first. */
        val watchdog: Job = scope.launch {
            delay(timeoutMs)
            if (settled.compareAndSet(false, true)) {
                onOutcome(DescriptorOutcome.TimedOut)
            }
        }
        scope.launch {
            /** What the blocking open returned, or null when the provider refused or failed. */
            val found: T? = try {
                opener(uri)
            } catch (expectedFailure: Exception) {
                report(uri, expectedFailure)
                null
            }
            if (!settled.compareAndSet(false, true)) {
                // The watchdog already reported the stall; nobody else will ever receive this descriptor.
                found?.close()
                return@launch
            }
            watchdog.cancel()
            onOutcome(if (found == null) DescriptorOutcome.Unavailable else DescriptorOutcome.Opened(found))
        }
    }
}
