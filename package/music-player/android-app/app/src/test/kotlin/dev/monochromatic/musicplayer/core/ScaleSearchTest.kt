// Search and page building at 1,000, 10,000, 100,000 and 1,000,000 tracks (D112, D47 to D74): the index must answer
// exactly like the old matcher at every size, and at a million tracks the answers, the memory and the cached page and
// view building must stay within bounds that the old code missed by orders of magnitude.

// What:     `package ...core` places the cases beside the index and the oracle.
// Why:      The cases call both implementations and the synthetic library through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Tracks the index is given per append, which is how a scan with large batches feeds it. */
private const val SCALE_APPEND_BATCH: Int = 10_000

/** Most milliseconds a million-track query may take; the old matcher took 15,000 to 47,000 per query. */
private const val MILLION_QUERY_LIMIT_MS: Double = 10_000.0

/** Most mebibytes the index of a million tracks may hold; measured at about 135. */
private const val MILLION_INDEX_LIMIT_MIB: Double = 250.0

/** Most milliseconds the cached view of one page may take at a million tracks. */
private const val VIEW_LIMIT_MS: Double = 100.0

/** Runs of the view build that only warm the compiler. */
private const val VIEW_WARMUP_RUNS: Int = 20

/** Runs of the view build whose fastest time is checked. */
private const val VIEW_MEASURED_RUNS: Int = 10

/** Bytes in a mebibyte. */
private const val BYTES_PER_MIB: Double = 1_048_576.0

/** Builds an index of [tracks] in batches the way a scan feeds it. */
private fun indexOf(tracks: List<dev.monochromatic.musicplayer.Track>): SearchIndex {
    val index = SearchIndex()
    var from = 0
    while (from < tracks.size) {
        val to: Int = minOf(tracks.size, from + SCALE_APPEND_BATCH)
        index.appendPaths(tracks.subList(from, to).map { it.displayPath }, tracks.subList(from, to).map { it.uri })
        from = to
    }
    return index
}

/** Runs each query on the index and the oracle and fails on the first difference. */
private fun assertParity(label: String, queries: List<String>, library: SyntheticLibrary, index: SearchIndex) {
    for (query in queries) {
        /** Answer of the index, timed. */
        val started: Long = System.nanoTime()
        val got = index.search(query)
        val millis: Double = (System.nanoTime() - started) / 1_000_000.0
        assertEquals("$label query '$query'", referenceSearchLibrary(query, library.tracks, true), got)
        println("SCALE $label query='$query' index_ms=$millis")
    }
}

/** Verifies parity, speed bounds and memory at each scale. */
class ScaleSearchTest {
    /** One artist: every query kind against the oracle. */
    @Test
    fun thousandTracksMatchTheOracle() {
        val library = buildSyntheticLibrary(1)
        assertParity("1k", listOf("e", "ka", "lin", "ashlinds", "ka lo", "q", "(", "  ", "夜"), library, indexOf(library.tracks))
    }

    /** Ten artists: every query kind against the oracle. */
    @Test
    fun tenThousandTracksMatchTheOracle() {
        val library = buildSyntheticLibrary(10)
        assertParity("10k", listOf("e", "ka", "lin", "ashlinds", "ka lo", "q", "(", "  ", "夜"), library, indexOf(library.tracks))
    }

    /** A hundred artists: queries that fill the cap, tie, and need verification against the oracle. */
    @Test
    fun hundredThousandTracksMatchTheOracle() {
        val library = buildSyntheticLibrary(100)
        assertParity("100k", listOf("ka", "ashlinds", "q"), library, indexOf(library.tracks))
    }

    /** A thousand artists: the index matches the oracle on a long term and is fast and small on every query kind. */
    @Test
    fun millionTracksAreFastSmallAndExact() {
        val library = buildSyntheticLibrary(1000)
        assertEquals(1_000_000, library.tracks.size)
        val index = indexOf(library.tracks)
        assertTrue("index heap ${index.heapBytes() / BYTES_PER_MIB} MiB", index.heapBytes() / BYTES_PER_MIB < MILLION_INDEX_LIMIT_MIB)
        for (query in listOf("e", "ka", "lin", "ashlinds", "ka lo", "q")) {
            val started: Long = System.nanoTime()
            index.search(query)
            val millis: Double = (System.nanoTime() - started) / 1_000_000.0
            println("SCALE 1M query='$query' index_ms=$millis")
            assertTrue("query '$query' took $millis ms", millis < MILLION_QUERY_LIMIT_MS)
        }
        assertParity("1M", listOf("ashlinds"), library, index)
    }

    /** Pages are built once; showing a page from the cache takes under 100 ms at a million tracks. */
    @Test
    fun millionTrackPagesBuildOnceAndShowQuickly() {
        val library = buildSyntheticLibrary(1000)
        val sorted = library.tracks.sortedWith { left, right -> compareByCodePoint(left.displayPath, right.displayPath) }
        val built = buildLibrary(sorted)
        assertEquals(1_000_000, built.uris.size)
        val labels: List<String> = built.pages.map { page -> page.label }
        val selected: Int = labels.size / 2
        val middle = built.pages[selected]
        /** Fastest of the measured runs; the first runs warm the compiler, which a loaded host starves. */
        var fastest = Double.MAX_VALUE
        repeat(VIEW_WARMUP_RUNS + VIEW_MEASURED_RUNS) { run ->
            val started: Long = System.nanoTime()
            buildPlayerContent(
                pageLabels = labels,
                selectedPage = selected,
                pageItems = middle.entries,
                playingItems = middle.entries,
                playingPageIndex = selected,
                currentIndex = middle.entries.first().index,
                playing = true,
                mode = PlaybackMode.IN_ORDER,
                positionSec = 0.0,
                durationSec = 0.0,
            )
            trackListEntries(pageFolderFor(middle.label, middle.entries), middle.entries.first().index)
            if (run >= VIEW_WARMUP_RUNS) {
                fastest = minOf(fastest, (System.nanoTime() - started) / 1_000_000.0)
            }
        }
        println("SCALE 1M view_ms=$fastest")
        assertTrue("view took $fastest ms", fastest < VIEW_LIMIT_MS)
    }
}
