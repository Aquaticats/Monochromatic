// Which queued track loads the native engine could not open for playback, and which of those failures to tell the
// user about. Plain code with no Android types, so the host JVM checks every case without a device.

// What:     `package dev.monochromatic.musicplayer.core` places the ledger beside the other rules of the player.
// Why:      The ledger has no Android type in it, so the core layer owns it and the tests read it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `class QueuedLoadLedger` counts the loads handed to the native engine and decides which failure it
//           reports.
// Why:      The native `load` call returns once the load is queued, and the engine worker opens and decodes the file
//           later. A file that opens but does not decode is announced only by a failure number that the worker
//           publishes, and the worker handles loads in the order they were queued. A failure belongs to the load
//           with that number, so only the failure of the latest queued load is told to the user: an older one has
//           been replaced by a newer request.
//
// In TS you'd write (pseudocode):
// ```ts
// class QueuedLoadLedger {
//   private queuedCount = 0; private reportedFailure = 0; private latestUri: string | null = null;
//   queued(uri: string): void { ... }
//   failureToReport(failedLoad: number): string | null { ... }
// }
// ```
/** Counts queued native loads and picks the failure of the latest one to report, once. */
class QueuedLoadLedger {
    // What:     `private var queuedCount: Long` is how many loads were queued so far, counting from one.
    // Why:      The worker numbers the loads it handles the same way, so a failure number matches a load.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private queuedCount = 0;
    // ```
    /** Number of loads queued with the native engine so far. */
    private var queuedCount: Long = 0L

    // What:     `private var reportedFailure: Long` is the highest failure number already looked at.
    // Why:      The worker publishes the number of its last failed load and keeps it, so the same number is read on
    //           every poll; it must lead to at most one report.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private reportedFailure = 0;
    // ```
    /** Highest failure number seen so far, whether it was reported or dropped as stale. */
    private var reportedFailure: Long = 0L

    // What:     `private var latestUri: String?` is the URI of the load queued last.
    // Why:      The failure report names the file, and only the latest load can be reported.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private latestUri: string | null = null;
    // ```
    /** URI of the most recently queued load, or null before the first. */
    private var latestUri: String? = null

    // What:     `private var latestRequestQueued: Boolean` says whether the newest load request reached the native
    //           engine.
    // Why:      A request can end before the native call, so it is never queued; the failure of the load queued
    //           before it then still equals the queued count, although the user has asked for something newer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private latestRequestQueued = true;
    // ```
    /** Whether the newest requested load was queued; false between [requested] and the [queued] that follows it. */
    private var latestRequestQueued: Boolean = true

    // What:     `fun queued(uri: String)` records one more load handed to the native engine.
    // Why:      The count keeps pace with the worker's own count of the loads it handles.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // queued(uri: string): void { this.queuedCount += 1; this.latestUri = uri; this.latestRequestQueued = true; }
    // ```
    /** Records that the load of [uri] was queued with the native engine. */
    fun queued(uri: String) {
        queuedCount += 1L
        latestUri = uri
        latestRequestQueued = true
    }

    // What:     `fun requested()` records that a newer load was asked for and has not reached the native engine yet.
    // Why:      A load request can end before the native call (the descriptor cannot be opened, the native load
    //           refuses it), and then it is never queued, yet it has replaced the load that failed before it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // requested(): void { this.latestRequestQueued = false; }
    // ```
    /** Records that a load was requested; the next [queued] call says it reached the native engine. */
    fun requested() {
        latestRequestQueued = false
    }

    // What:     `fun failureToReport(failedLoad: Long): String?` returns the URI to report, or null.
    // Why:      A failure number of zero means none failed, a number already seen was handled, a number below
    //           the count of queued loads names a load that a newer request has replaced, and so does the count
    //           itself while the newest request has not been queued.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // failureToReport(failedLoad: number): string | null {
    //   if (failedLoad <= this.reportedFailure) return null;
    //   this.reportedFailure = failedLoad;
    //   return failedLoad === this.queuedCount && this.latestRequestQueued ? this.latestUri : null;
    // }
    // ```
    /**
     * URI of the latest queued load when the worker has newly failed it, else null.
     *
     * @param failedLoad failure number the worker published: the sequence number of its last failed load, or zero
     */
    fun failureToReport(failedLoad: Long): String? {
        if (failedLoad <= reportedFailure) {
            return null
        }
        reportedFailure = failedLoad
        return if (failedLoad == queuedCount && latestRequestQueued) latestUri else null
    }
}
