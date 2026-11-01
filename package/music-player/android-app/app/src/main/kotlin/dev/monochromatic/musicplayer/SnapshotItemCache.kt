// What:     `package dev.monochromatic.musicplayer` puts this class beside `PlayerController`, which owns one.
// Why:      The rule is plain Kotlin with no Android types, so unit tests reach it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import dev.monochromatic.musicplayer.core.playbackEntryIds
import dev.monochromatic.musicplayer.core.trackTitleFor

// What:     `class SnapshotItemCache` remembers the queue rows of the last playback snapshot together with the three
//           lists they came from (playback order, display paths, track URIs) and the artist tags they carry.
// Why:      Every media session refresh used to rebuild one row per queue position (title, ids, artist), so a track
//           change cost time proportional to the queue size. The lists are replaced, never edited in place, so when
//           the same list instances come back the rows are still right; a changed artist patches only the rows of
//           that track.
// Gotcha:   Identity (`===`) is compared on purpose, not structural equality: comparing 10 000 element lists
//           would cost as much as rebuilding them.
//
// In TS you'd write (pseudocode):
// ```ts
// class SnapshotItemCache {
//   private order?: readonly number[];
//   private display?: readonly string[];
//   private uris?: readonly string[];
//   private items: readonly SnapshotItem[] = [];
//   private appliedArtists = new Map<string, string>();
//   itemsFor(args: {...}): readonly SnapshotItem[] { /* ... */ }
// }
// ```
/** Keeps the rows of the playback snapshot between calls and rebuilds one only when its track or tag changed. */
class SnapshotItemCache {
    /** Playback order the cached rows were built from, compared by identity. */
    private var cachedOrder: List<Int>? = null

    /** Display paths the cached rows were built from, compared by identity. */
    private var cachedDisplay: List<String>? = null

    /** Track URIs the cached rows were built from, compared by identity. */
    private var cachedUris: List<String>? = null

    /** Rows built for [cachedOrder]; a new list instance whenever any row differs, the same instance otherwise. */
    private var cachedItems: List<SnapshotItem> = emptyList()

    /** Copy of the artist tags the cached rows carry, to find the tags that changed since. */
    private var appliedArtists: Map<String, String> = emptyMap()

    /**
     * Returns one row per queue position, reusing every row of the previous call that is still right.
     *
     * @param order load-order indices in playback order, repeats allowed
     * @param display display path of each load-order index
     * @param uris playback URI of each load-order index
     * @param artists artist tag by track URI; the map may be edited after the call
     * @return the rows; the same list instance as the previous call when nothing changed
     */
    fun itemsFor(
        order: List<Int>,
        display: List<String>,
        uris: List<String>,
        artists: Map<String, String>,
    ): List<SnapshotItem> {
        if (order !== cachedOrder || display !== cachedDisplay || uris !== cachedUris) {
            cachedItems = buildAll(order = order, display = display, uris = uris, artists = artists)
            cachedOrder = order
            cachedDisplay = display
            cachedUris = uris
            appliedArtists = HashMap(artists)
            return cachedItems
        }
        /** URIs whose tag was added, replaced or evicted since the cached rows were built. */
        val changed: Set<String> = changedTagUris(applied = appliedArtists, current = artists)
        if (changed.isEmpty()) {
            return cachedItems
        }
        cachedItems = patched(items = cachedItems, changed = changed, artists = artists)
        appliedArtists = HashMap(artists)
        return cachedItems
    }

    /** Builds every row from scratch. */
    private fun buildAll(
        order: List<Int>,
        display: List<String>,
        uris: List<String>,
        artists: Map<String, String>,
    ): List<SnapshotItem> {
        /** One timeline id per playback position; repeats of a track get distinct ids. */
        val entryIds: List<Long> = playbackEntryIds(order)
        return order.mapIndexed { position, loadIndex ->
            SnapshotItem(
                uri = uris[loadIndex],
                title = trackTitleFor(display[loadIndex]),
                loadIndex = loadIndex,
                entryId = entryIds[position],
                artist = artists[uris[loadIndex]],
            )
        }
    }

    /** Returns [items] with the artist of each row whose URI is in [changed] set from [artists]; others are shared. */
    private fun patched(
        items: List<SnapshotItem>,
        changed: Set<String>,
        artists: Map<String, String>,
    ): List<SnapshotItem> {
        /** Copy that receives the new rows, made on the first row that needs one. */
        var copy: MutableList<SnapshotItem>? = null
        for (position in items.indices) {
            /** Row at this position. */
            val item: SnapshotItem = items[position]
            if (item.uri in changed && artists[item.uri] != item.artist) {
                /** Rows being patched: the copy made for an earlier row, or a new copy for this first one. */
                val target: MutableList<SnapshotItem> = copy ?: ArrayList(items)
                target[position] = item.copy(artist = artists[item.uri])
                copy = target
            }
        }
        return copy ?: items
    }
}

/**
 * Finds the URIs whose artist tag differs between two tag maps.
 *
 * @param applied tags the cached rows carry
 * @param current tags on record now
 * @return URIs added, replaced or removed
 */
internal fun changedTagUris(applied: Map<String, String>, current: Map<String, String>): Set<String> {
    /** URIs found different so far. */
    val changed: MutableSet<String> = HashSet()
    for ((uri, artist) in current) {
        if (applied[uri] != artist) {
            changed.add(uri)
        }
    }
    for (uri in applied.keys) {
        if (uri !in current) {
            changed.add(uri)
        }
    }
    return changed
}
