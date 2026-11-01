// The ingest of a library scan into the player (D111). A scan reports the tracks it finds in small batches in the
// order found. The player shows the library sorted, and sorting and deriving the pages of a million tracks takes
// seconds, so the ingest does that work on its own thread, at most as often as it can afford, and hands the main
// thread a finished library to swap in. The screen never waits for a scan batch, and Search receives every track
// once the player shows it.

// What:     `package dev.monochromatic.musicplayer.core` places the ingest beside the feed and the build it drives.
// Why:      It has no Android types: the caller supplies how a built library reaches the main thread.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import java.util.concurrent.ScheduledExecutorService
import java.util.concurrent.ScheduledFuture
import java.util.concurrent.TimeUnit

/** The fewest time between two publications of a growing library: a quarter of a second. */
const val INGEST_MIN_INTERVAL_NANOS: Long = 250_000_000L

/** How many times the cost of the last publication the ingest waits before the next, so it never runs flat out. */
const val INGEST_COST_FACTOR: Long = 4L

// What:     `class FinalLibrary(...)` is the whole library of a finished scan, built and ready to install.
// Why:      The last install happens on the main thread with the session restore, so it is prepared here, and the
//           tracks that no install carried yet are kept for the Search feed.
//
// In TS you'd write (pseudocode):
// ```ts
// type FinalLibrary = { built: BuiltLibrary; unfed: Track[] };
// ```
/** The complete sorted library of a finished scan and the tracks that Search has not been given. */
class FinalLibrary(
    /** The whole library, built. */
    val built: BuiltLibrary,
    /** Tracks found since the last publication, in the order found. */
    val unfed: List<Track>,
)

// What:     `class LibraryIngest(...)` accumulates scanned tracks and publishes sorted snapshots of them.
// Why:      Sorting everything at every batch made a scan quadratic. Here a batch costs a copy, a publication costs one
//           merge of the new tracks into the sorted list plus one build, and publications are spaced by their own cost.
//
// In TS you'd write (pseudocode):
// ```ts
// const ingest = new LibraryIngest(feed, install, executor);
// ingest.accept(batch);              // scan thread, per batch
// const final = ingest.finish();     // after the scan, off the main thread
// ```
/**
 * Turns the batches of one scan into published libraries.
 *
 * [accept] may be called from any thread. Publications run one at a time on [executor], which must be a single
 * thread; [install] is called on that thread and returns once the main thread holds the library, or false when the
 * library was dropped because the scan was superseded. [finish] and [cancel] end the ingest.
 */
class LibraryIngest(
    /** Receives each published batch of tracks after the player shows it. */
    private val feed: LibraryFeed,
    /** Hands a built library to the main thread and waits for it; returns false when the scan was superseded. */
    private val install: (BuiltLibrary) -> Boolean,
    /** The single thread that publishes. */
    private val executor: ScheduledExecutorService,
    /** The URI of the playing track, so the build can locate it instead of the main thread scanning for it. */
    private val playingUri: () -> String? = { null },
    /** The fewest nanoseconds between two publications. */
    private val minIntervalNanos: Long = INGEST_MIN_INTERVAL_NANOS,
    /** Source of the time, replaced by the tests. */
    private val clock: () -> Long = System::nanoTime,
    /** Receives one line per publication. */
    private val logger: (String) -> Unit = {},
) {
    /** Guards the fields below that the scan thread and the publishing thread share. */
    private val lock = Any()

    /** Every track found, in the order found. */
    private val arrival = ArrayList<Track>()

    /** Tracks of the last publication, sorted by display path; only the publishing thread and [finish] touch it. */
    private var sorted: List<Track> = emptyList()

    /** Number of arrival tracks the last publication carried. */
    private var published: Int = 0

    /** The publication that is scheduled and has not started, or null. */
    private var pending: ScheduledFuture<*>? = null

    /** When the last publication ended, in clock nanoseconds, or null before the first. */
    private var lastEnd: Long? = null

    /** How long the last publication took, in nanoseconds. */
    private var lastCost: Long = 0L

    /** Whether the ingest ended, after which batches are ignored. */
    private var closed: Boolean = false

    /** Number of tracks found so far. */
    val found: Int get() = synchronized(lock) { arrival.size }

    /** Number of tracks the player has been given. */
    val shown: Int get() = synchronized(lock) { published }

    /** Records one batch of newly found tracks and makes sure a publication is on the way. */
    fun accept(added: List<Track>) {
        synchronized(lock) {
            if (closed || added.isEmpty()) {
                return
            }
            arrival.addAll(added)
            scheduleLocked()
        }
    }

    /** Schedules a publication unless one is waiting, for after the pause that its predecessor's cost earned. */
    private fun scheduleLocked() {
        if (pending != null || closed) {
            return
        }
        /** When the next publication may start, or null before the first. */
        val earliest: Long? = lastEnd?.let { end -> end + maxOf(minIntervalNanos, lastCost * INGEST_COST_FACTOR) }
        /** Nanoseconds to wait before the publication. */
        val wait: Long = if (earliest == null) 0L else maxOf(0L, earliest - clock())
        pending = executor.schedule({ publish() }, wait, TimeUnit.NANOSECONDS)
    }

    /** Publishes the tracks found since the last publication. */
    private fun publish() {
        /** When this publication started. */
        val started: Long = clock()
        /** Tracks found since the last publication, in the order found. */
        val delta: List<Track>
        /** Number of tracks found when this publication took its snapshot. */
        val count: Int
        synchronized(lock) {
            pending = null
            if (closed || arrival.size == published) {
                return
            }
            count = arrival.size
            delta = arrival.subList(published, count).toList()
        }
        /** The published tracks with the new ones inserted in order. */
        val merged: List<Track> = mergeSortedTracks(sorted, delta)
        /** The merged tracks with every derived list. */
        val built: BuiltLibrary = buildLibrary(merged, playingUri())
        /** Whether the main thread applied the library. */
        val installed: Boolean = install(built)
        if (installed) {
            sorted = merged
            feed.accept(delta)
        }
        /** When this publication ended. */
        val ended: Long = clock()
        synchronized(lock) {
            if (installed) {
                published = count
            }
            lastCost = ended - started
            lastEnd = ended
            if (installed && arrival.size > published) {
                scheduleLocked()
            }
        }
        logger("library published tracks=$count installed=$installed ms=${(ended - started) / NANOS_PER_MILLISECOND}")
    }

    // What:     `fun finish(): FinalLibrary` ends the ingest and builds the whole library.
    // Why:      The scan returned, so every track was reported. It waits for a publication in progress, then merges
    //           what remains and builds, all off the main thread; the caller installs the result with the session.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const final = ingest.finish(); // then: controller.finishBuilt(final.built, session); ingest.complete(final);
    // ```
    /** Ends the ingest, waits for a publication in progress, and returns the complete sorted library. Off-main only. */
    fun finish(): FinalLibrary {
        synchronized(lock) {
            closed = true
            pending?.cancel(false)
            pending = null
        }
        // A publication that already started runs to its end first, because the executor is one thread.
        executor.submit {}.get()
        /** Tracks found since the last publication, in the order found. */
        val delta: List<Track> = synchronized(lock) { arrival.subList(published, arrival.size).toList() }
        /** All tracks, sorted. */
        val merged: List<Track> = mergeSortedTracks(sorted, delta)
        return FinalLibrary(buildLibrary(merged, playingUri()), delta)
    }

    /** Gives Search the tracks that no publication carried, and tells it the scan is over. Call it after installing. */
    fun complete(final: FinalLibrary) {
        feed.accept(final.unfed)
        feed.finish()
        executor.shutdown()
    }

    /** Ends the ingest without publishing again, because the scan was superseded or failed. */
    fun cancel() {
        synchronized(lock) {
            closed = true
            pending?.cancel(false)
            pending = null
        }
        executor.shutdownNow()
    }
}

/** Nanoseconds in a millisecond, for the log line. */
private const val NANOS_PER_MILLISECOND: Double = 1_000_000.0

// What:     `internal fun mergeSortedTracks(sorted: List<Track>, added: List<Track>): List<Track>` inserts tracks.
// Why:      Sorting the whole list again for every batch is what made a scan quadratic. The new tracks are sorted on
//           their own and each is placed by a binary search, so the cost is the size of the batch times a logarithm
//           plus one copy. Equal paths keep the order found, as a stable sort of everything would.
//
// In TS you'd write (pseudocode):
// ```ts
// const mergeSortedTracks = (sorted: Track[], added: Track[]) => [...sorted, ...added].sort(byCodePoint);
// ```
/** Returns [sorted] with [added] inserted, ordered by display path in code-point order, ties in the order found. */
internal fun mergeSortedTracks(sorted: List<Track>, added: List<Track>): List<Track> {
    /** Order of display paths in code-point order. */
    val order = Comparator<Track> { left, right -> compareByCodePoint(left.displayPath, right.displayPath) }
    /** The new tracks, sorted on their own; equal paths keep the order found. */
    val extra: List<Track> = added.sortedWith(order)
    /** The result, filled in order. */
    val merged = ArrayList<Track>(sorted.size + extra.size)
    /** Number of published tracks copied to the result so far. */
    var taken = 0
    for (item in extra) {
        // First position after every track that is not after the item, which keeps older equals before it.
        /** Lowest position the item may take. */
        var low: Int = taken
        /** One past the highest position the item may take. */
        var high: Int = sorted.size
        while (low < high) {
            /** Middle of the remaining range. */
            val middle: Int = (low + high) ushr 1
            if (order.compare(sorted[middle], item) <= 0) {
                low = middle + 1
            } else {
                high = middle
            }
        }
        merged.addAll(sorted.subList(taken, low))
        taken = low
        merged.add(item)
    }
    merged.addAll(sorted.subList(taken, sorted.size))
    return merged
}
