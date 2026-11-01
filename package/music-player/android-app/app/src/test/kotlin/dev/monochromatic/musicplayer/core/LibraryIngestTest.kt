// The ingest publishes sorted snapshots of a growing scan off the main thread and feeds Search once the player shows
// each batch (D111). The cases use real threads: a slow scan thread reports batches, a stand-in main thread applies
// published libraries, and a real Search service answers.

// What:     `package ...core` places the cases beside the ingest.
// Why:      They call the ingest, the feed and the service through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.Executors
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Longest wait for any one event in these cases. */
private const val WAIT_SECONDS: Long = 30L

/** Orders tracks the way the sources sort them. */
private val BY_PATH: Comparator<Track> = Comparator { left, right -> compareByCodePoint(left.displayPath, right.displayPath) }

/** A Search answer reduced to what must not depend on how the tracks were numbered. */
private fun shape(result: SearchResult): List<Any?> = listOf(
    result.kind, result.name, result.parentContext, result.folderName, result.tier,
    result.nameHighlights, result.parentHighlights, result.trackUri,
)

/** Waits for the next answer of [inbox]. */
private fun LinkedBlockingQueue<SearchAnswer>.nextAnswer(): SearchAnswer =
    poll(WAIT_SECONDS, TimeUnit.SECONDS) ?: error("no answer in time")

/** Verifies publication, sorting, supersession and the hand-over to Search. */
class LibraryIngestTest {
    /** Everything the cases need around one ingest. */
    private class Rig(minIntervalNanos: Long, installResult: () -> Boolean = { true }) {
        /** Answers of the Search service. */
        val answers = LinkedBlockingQueue<SearchAnswer>()

        /** The service that the feed fills. */
        val service = SearchService(deliver = { answers.put(it) }, refreshIntervalMillis = 5L)

        /** Libraries applied by the stand-in main thread, in order. */
        val installed = CopyOnWriteArrayList<BuiltLibrary>()

        /** The single publishing thread. */
        val executor = Executors.newSingleThreadScheduledExecutor()

        /** The ingest under test. */
        val ingest = LibraryIngest(
            feed = LibraryFeed(service, service.replaceLibrary(emptyList(), null)),
            install = { built ->
                val applied = installResult()
                if (applied) installed.add(built)
                applied
            },
            executor = executor,
            minIntervalNanos = minIntervalNanos,
        )

        /** Stops the threads. */
        fun close() {
            ingest.cancel()
            service.close()
        }
    }

    /** Builds track [number] of a library whose folders and names make unique paths. */
    private fun track(number: Int, folder: String = "Artist ${number % 7}"): Track =
        Track("uri-$number", "$folder/Song ${(number * 7919) % 100_003}.flac")

    /** Every published library is sorted and extends the previous one, and finish returns the whole sorted library. */
    @Test
    fun publicationsAreSortedAndGrow() {
        val rig = Rig(minIntervalNanos = 1_000_000L)
        try {
            val all = List(3000) { track(it) }
            all.chunked(128).forEach { batch ->
                rig.ingest.accept(batch)
                Thread.sleep(2L)
            }
            val final = rig.ingest.finish()
            rig.ingest.complete(final)
            assertEquals(all.sortedWith(BY_PATH), final.built.tracks)
            assertTrue("several snapshots were published (${rig.installed.size})", rig.installed.size >= 2)
            var previous = 0
            for (built in rig.installed) {
                assertEquals(built.tracks.sortedWith(BY_PATH), built.tracks)
                assertTrue("each snapshot is larger", built.tracks.size > previous)
                previous = built.tracks.size
                assertEquals(built.displayPaths, relativeDisplayPaths(built.rawPaths))
                assertEquals(paginate(built.displayPaths), built.pages)
            }
            assertEquals(all.size, final.built.tracks.size)
        } finally {
            rig.close()
        }
    }

    /** Equal paths keep the order they were found in, as a stable sort of everything would. */
    @Test
    fun equalPathsKeepTheOrderFound() {
        val rig = Rig(minIntervalNanos = 0L)
        try {
            val all = List(500) { Track("uri-$it", "Same/${it % 3}.flac") }
            all.chunked(60).forEach { rig.ingest.accept(it) }
            val final = rig.ingest.finish()
            assertEquals(all.sortedWith(BY_PATH), final.built.tracks)
        } finally {
            rig.close()
        }
    }

    /** Publications are spaced: a burst of batches is published far fewer times than it has batches. */
    @Test
    fun aBurstIsPublishedLessOftenThanItArrives() {
        val rig = Rig(minIntervalNanos = 200_000_000L)
        try {
            List(40) { batch -> List(10) { track(batch * 10 + it) } }.forEach { rig.ingest.accept(it) }
            Thread.sleep(500L)
            val final = rig.ingest.finish()
            assertTrue("published ${rig.installed.size} times for 40 batches", rig.installed.size in 1..4)
            assertEquals(400, final.built.tracks.size)
        } finally {
            rig.close()
        }
    }

    /** Search is fed after the player shows a batch: a result can be located in the installed library. */
    @Test
    fun searchSeesOnlyTracksThePlayerShows() {
        val rig = Rig(minIntervalNanos = 0L)
        try {
            val all = List(900) { track(it) }
            all.chunked(100).forEach { rig.ingest.accept(it) }
            val final = rig.ingest.finish()
            rig.ingest.complete(final)
            val sequence = rig.service.query("song")
            var answer = rig.answers.nextAnswer()
            while (answer.sequence != sequence || !answer.complete) answer = rig.answers.nextAnswer()
            val uris = final.built.uris.toSet()
            val items = (answer.state as SearchState.Results).items
            assertTrue(items.isNotEmpty())
            assertTrue("every result is in the library", items.all { it.trackUri in uris })
            assertEquals(all.size, answer.knownTracks)
        } finally {
            rig.close()
        }
    }

    /** A superseded scan installs nothing and feeds Search nothing. */
    @Test
    fun aSupersededScanFeedsNothing() {
        val rig = Rig(minIntervalNanos = 0L, installResult = { false })
        try {
            List(5) { batch -> List(50) { track(batch * 50 + it) } }.forEach {
                rig.ingest.accept(it)
                Thread.sleep(5L)
            }
            rig.ingest.finish()
            assertTrue(rig.installed.isEmpty())
            assertEquals("Search was given nothing but the final tail later", 0, rig.service.trackCount)
        } finally {
            rig.close()
        }
    }

    /** When the shared prefix shrinks mid-scan, Search shows the final relative paths for all tracks. */
    @Test
    fun aShrinkingPrefixUpdatesSearchPaths() {
        val rig = Rig(minIntervalNanos = 0L)
        try {
            val music = List(300) { Track("m$it", "Music/Rock/Song $it.flac") }
            val podcasts = List(100) { Track("p$it", "Podcasts/Talk $it.mp3") }
            music.chunked(100).forEach {
                rig.ingest.accept(it)
                Thread.sleep(5L)
            }
            podcasts.chunked(50).forEach { rig.ingest.accept(it) }
            val final = rig.ingest.finish()
            rig.ingest.complete(final)
            val relative = final.built.displayPaths
            val oracleTracks = final.built.tracks.mapIndexed { index, track -> Track(track.uri, relative[index]) }
            for (query in listOf("song", "rock", "music", "talk", "pod", "s")) {
                val sequence = rig.service.query(query)
                var answer = rig.answers.nextAnswer()
                while (answer.sequence != sequence || !answer.complete) answer = rig.answers.nextAnswer()
                val expected = referenceSearchLibrary(query, oracleTracks, true)
                val got = answer.state
                assertEquals(query, expected::class, got::class)
                if (expected is SearchState.Results && got is SearchState.Results) {
                    assertEquals(query, expected.items.map(::shape), got.items.map(::shape))
                    assertEquals(query, expected.truncated, got.truncated)
                }
            }
        } finally {
            rig.close()
        }
    }

    /** The scan thread never waits for a publication: accepting a batch returns at once even while one runs. */
    @Test
    fun acceptingABatchNeverWaitsForAPublication() {
        val release = java.util.concurrent.CountDownLatch(1)
        val rig = Rig(minIntervalNanos = 0L, installResult = {
            release.await(WAIT_SECONDS, TimeUnit.SECONDS)
            true
        })
        try {
            val done = AtomicBoolean(false)
            val scan = Thread {
                List(30) { batch -> List(100) { track(batch * 100 + it) } }.forEach { rig.ingest.accept(it) }
                done.set(true)
            }
            scan.start()
            scan.join(10_000L)
            assertTrue("30 batches were accepted while the first publication was blocked", done.get())
            assertFalse(rig.installed.isNotEmpty())
            release.countDown()
            val final = rig.ingest.finish()
            assertEquals(3000, final.built.tracks.size)
        } finally {
            release.countDown()
            rig.close()
        }
    }
}
