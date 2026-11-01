// The library follows the disk by itself (D113): a change signal from the source or from the app returning to the front
// asks for a rescan, and a signal that arrives while a load is running must not be lost.

// What:     `package ...core` places the plan beside the other library rules.
// Why:      The plan has no Android types, so unit tests reach it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `class RescanPlan` decides when a change signal starts a rescan and when it waits for the running load.
// Why:      A rescan that is skipped because a load is running would leave the library stale until the next signal, so
//           the signal is remembered and the rescan runs once when the load finishes, however many signals came.
//
// In TS you'd write (pseudocode):
// ```ts
// class RescanPlan {
//   signal(): boolean { if (this.loading) { this.queued = true; return false; } return true; }
//   loadStarted(): void { this.loading = true; }
//   loadFinished(): boolean { this.loading = false; const again = this.queued; this.queued = false; return again; }
// }
// ```
/** Remembers change signals that arrive while a library load runs, so the rescan they ask for is not lost. */
class RescanPlan {
    /** Whether a library load or rescan is running now. */
    private var loading: Boolean = false

    /** Whether a signal arrived during the running load and a rescan is owed. */
    private var queued: Boolean = false

    /**
     * Reports a debounced change signal.
     *
     * @return true when a rescan should start now; false when a load is running and the rescan waits for it
     */
    fun signal(): Boolean {
        if (loading) {
            queued = true
            return false
        }
        return true
    }

    /**
     * Reports that the app came back to the front, or that the activity bound to the running service.
     *
     * Before the first load nothing is asked, because that load reads the disk anyway; afterwards every return asks
     * for a rescan, so files added while the app was away appear with no user action (D113).
     *
     * @param libraryLoaded whether the first library load has started
     * @return true when a debounced rescan should be requested
     */
    fun foreground(libraryLoaded: Boolean): Boolean = libraryLoaded

    /** Records that a library load or rescan began. */
    fun loadStarted() {
        loading = true
    }

    /**
     * Records that a library load or rescan ended.
     *
     * @return true once when a signal arrived during the load, so the caller rescans again
     */
    fun loadFinished(): Boolean {
        loading = false
        /** Whether a rescan is owed because of a signal during the load. */
        val again: Boolean = queued
        queued = false
        return again
    }
}
