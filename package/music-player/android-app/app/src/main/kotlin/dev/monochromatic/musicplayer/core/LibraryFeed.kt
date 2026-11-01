// The feed from a library scan to the Search index (D111, D112). A scan reports the tracks it finds in small
// batches, in the order found; the feed appends each batch to the index as soon as the library the player shows
// contains it. It also keeps what the pages show: the paths in the index are the relative display paths of the
// tracks found so far, which lose the segments that all of them share, and that shared run can shrink as the scan
// widens.

// What:     `package dev.monochromatic.musicplayer.core` places the feed beside the service it feeds.
// Why:      The feed has no Android types, so the JVM tests drive it with a real service.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track

// What:     `class LibraryFeed(search: SearchService, generation: Long)` appends scanned tracks to one library.
// Why:      The index keeps a track's number as the order in which it arrived, which never changes while the library is
//           scanned, while the player's list is sorted again at every batch. The feed is where the two meet: it numbers
//           tracks by arrival and gives the index the display paths that the player's pages show.
//
// In TS you'd write (pseudocode):
// ```ts
// const feed = new LibraryFeed(search, generation);
// feed.accept(batch);   // after the player shows the batch
// feed.finish();
// ```
/**
 * Feeds the tracks of one scan to the Search index.
 *
 * Call [accept] with each batch after the player's list contains it, from one thread at a time, and [finish] when the
 * scan ends. The feed holds every track's raw path until [finish], because a shrinking shared prefix needs them.
 */
class LibraryFeed(
    /** The service whose index receives the tracks. */
    private val search: SearchService,
    /** Number of the library in the service; the service refuses a feed of a replaced library. */
    private val generation: Long,
) {
    /** Tracks found so far in arrival order. */
    private val arrival = ArrayList<Track>()

    /** Shared prefix of the raw paths found so far, created with the first track because its URI names the root. */
    private var prefix: CommonPrefix? = null

    /** Number of leading segments that the paths in the index have lost. */
    private var stripped: Int = 0

    /** Number of tracks appended to the index. */
    private var fed: Int = 0

    /** Appends one batch to the index, and rewrites the index's paths first when the shared prefix shrank. */
    @Synchronized
    fun accept(added: List<Track>) {
        if (added.isEmpty()) {
            return
        }
        /** Shared prefix of this scan, cut at the library root named by the first track's source. */
        val shared: CommonPrefix = prefix ?: CommonPrefix(displayRootOf(added.first().uri)).also { prefix = it }
        added.forEach { track -> shared.add(track.displayPath) }
        arrival.addAll(added)
        /** Segments that every path found so far shares, up to the library root. */
        val strip: Int = shared.count
        /** Display paths of the new tracks under the shared prefix. */
        val names: List<String> = added.map { track -> relativeOf(track.displayPath, strip) }
        search.append(generation, names, added.map { track -> track.uri })
        if (strip != stripped && fed > 0) {
            search.replacePaths(generation, arrival.map { track -> relativeOf(track.displayPath, strip) })
        }
        stripped = strip
        fed = arrival.size
    }

    /** Tells the service that the scan ended, so an empty or unmatched answer is final, and drops the raw paths. */
    @Synchronized
    fun finish() {
        arrival.clear()
        arrival.trimToSize()
        search.setComplete(true)
    }
}
