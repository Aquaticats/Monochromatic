// The gate that cuts a library scan into small batches of newly found tracks (D111). The first gate, `BatchEmitGate`,
// handed over the whole sorted-so-far list at every threshold, which sorted the list again each time and made a scan
// of a million tracks quadratic. This one hands over only the tracks found since the last batch, in the order found,
// so a batch costs time proportional to its own size, and the receiver decides when to sort and publish.

// What:     `package dev.monochromatic.musicplayer.core` places the gate beside the other pure scan logic.
// Why:      The gate has no Android types, so the JVM tests drive it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `class LibraryDeltaGate<T>(private val threshold: Int)` decides when enough new items have piled up.
// Why:      Each scan creates its own gate, because two scans can run at once (playback and the background sweep both
//           use the shared sources), and a shared running total would corrupt across them.
//
// In TS you'd write (pseudocode):
// ```ts
// class LibraryDeltaGate<T> { nextDelta(accumulated: T[]): T[] | null; tail(accumulated: T[]): T[] | null; }
// ```
/** Cuts a growing list into batches of at least [threshold] new items, plus a final batch for the remainder. */
class LibraryDeltaGate<T>(
    /** Number of new items that must pile up before a batch is handed over. */
    private val threshold: Int,
) {
    /** Number of leading items of the accumulated list that earlier batches already carried. */
    private var handedOver: Int = 0

    /** Returns the items found since the last batch once [threshold] of them exist, else null. */
    fun nextDelta(accumulated: List<T>): List<T>? {
        if (accumulated.size - handedOver < threshold) {
            return null
        }
        return takeRest(accumulated)
    }

    /** Returns every item not yet handed over, however few, or null when none remain; called once at the end. */
    fun tail(accumulated: List<T>): List<T>? {
        if (accumulated.size == handedOver) {
            return null
        }
        return takeRest(accumulated)
    }

    /** Copies the items after the handed-over ones and counts them as handed over. */
    private fun takeRest(accumulated: List<T>): List<T> {
        /** Copy of the items after the handed-over ones. */
        val rest: List<T> = accumulated.subList(handedOver, accumulated.size).toList()
        handedOver = accumulated.size
        return rest
    }
}
