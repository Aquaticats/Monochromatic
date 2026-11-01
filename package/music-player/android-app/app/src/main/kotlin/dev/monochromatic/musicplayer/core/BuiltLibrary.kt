// Everything the player derives from a list of tracks, computed without touching the player. Installing a library on
// the main thread used to rebuild the URI list, the relative paths and the pages in place, which froze the screen for
// seconds at a million tracks and ran again for every scan batch. Building is pure, so it runs on a worker thread, and
// the main thread only swaps in the finished result.

// What:     `package dev.monochromatic.musicplayer.core` places the build beside the queue and the pagination it uses.
// Why:      The build is pure logic over `Track` values, so the JVM tests time it at every library size.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track

// What:     `class BuiltLibrary(...)` is a library in the forms the player reads.
// Why:      The queue stores raw paths, the pages and the rows read relative paths, and playback reads URIs; deriving
//           each once, off the main thread, lets installing a library be a handful of assignments.
//
// In TS you'd write (pseudocode):
// ```ts
// type BuiltLibrary = { tracks: Track[]; uris: string[]; rawPaths: string[]; displayPaths: string[];
//   pages: Page[]; pageOfIndex: Int32Array };
// ```
/** A library in load order with its derived lists, ready to install. */
class BuiltLibrary(
    /** Tracks in load order. */
    val tracks: List<Track>,
    /** URI of each track by load-order index. */
    val uris: List<String>,
    /** Raw display path of each track by load-order index, which the queue stores. */
    val rawPaths: List<String>,
    /** Relative display path of each track, without the segments that all paths share. */
    val displayPaths: List<String>,
    /** Pages built from [displayPaths], in tab order. */
    val pages: List<Page>,
    /** Tab-order page of each load-order index, so finding a track's page is a lookup and not a scan. */
    val pageOfIndex: IntArray,
    /** URI the build was asked to locate, or null when it was asked for none. */
    val hintUri: String? = null,
    /** Load-order index of [hintUri], or -1 when it is absent or no URI was asked for. */
    val hintIndex: Int = -1,
)

// What:     `fun buildLibrary(tracks: List<Track>): BuiltLibrary` derives every list the player reads from the tracks.
// Why:      It is the only place that pays for relative paths and pagination, so callers can run it where they like.
//
// In TS you'd write (pseudocode):
// ```ts
// function buildLibrary(tracks: Track[]): BuiltLibrary { /* uris, raw paths, relative paths, pages, page lookup */ }
// ```
/**
 * Derives the URIs, raw and relative paths, pages and page lookup of [tracks], which are in load order.
 *
 * When [locate] is given, the build also finds its position in the library, which would cost the main thread a scan.
 */
fun buildLibrary(tracks: List<Track>, locate: String? = null): BuiltLibrary {
    /** Raw display path of each track. */
    val rawPaths: List<String> = tracks.map { track -> track.displayPath }
    /** Segments of the library root that the paths are relative to, from the source of the first track. */
    val root: List<String>? = tracks.firstOrNull()?.let { track -> displayRootOf(track.uri) }
    /** Relative display path of each track, cut at the library root. */
    val displayPaths: List<String> = relativeDisplayPaths(rawPaths, root)
    /** Pages built from the relative paths. */
    val pages: List<Page> = paginate(displayPaths)
    /** URI of each track. */
    val uris: List<String> = tracks.map { track -> track.uri }
    return BuiltLibrary(
        tracks = tracks,
        uris = uris,
        rawPaths = rawPaths,
        displayPaths = displayPaths,
        pages = pages,
        pageOfIndex = pageLookupOf(pages, tracks.size),
        hintUri = locate,
        hintIndex = if (locate == null) -1 else uris.indexOf(locate),
    )
}

// What:     `fun pageLookupOf(pages: List<Page>, size: Int): IntArray` records the page of every load-order index.
// Why:      `pageOfIndex` scans every page until it finds the index, which costs milliseconds at a million tracks and
//           runs on every refresh; a table answers in one read.
//
// In TS you'd write (pseudocode):
// ```ts
// const pageLookupOf = (pages: Page[], size: number) => { const t = new Int32Array(size).fill(-1); return t; };
// ```
/** Returns, for each of [size] load-order indices, the tab-order page that holds it, or -1 for none. */
fun pageLookupOf(pages: List<Page>, size: Int): IntArray {
    /** Page of each index, -1 until a page claims it. */
    val table = IntArray(size) { -1 }
    for ((position, page) in pages.withIndex()) {
        for (entry in page.entries) {
            if (entry.index in 0 until size) {
                table[entry.index] = position
            }
        }
    }
    return table
}
