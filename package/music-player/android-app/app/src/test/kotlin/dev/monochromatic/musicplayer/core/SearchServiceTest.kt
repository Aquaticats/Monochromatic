// D111 on the JVM: Search answers while a deliberately slow scan keeps appending tracks in small batches. Typed
// queries get results for the tracks already known, the open query refreshes in place as batches land, keystroke
// latency stays inside the bar during ingest, and the final answer equals the old matcher's.

// What:     `package ...core` places the cases beside the service.
// Why:      They drive the service with real threads and latches.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Tracks per scan batch, as the sources emit them. */
private const val SCAN_BATCH: Int = 128

/** Longest wait for any one answer in these cases. */
private const val ANSWER_WAIT_SECONDS: Long = 30L

/** Collects the answers a service delivers, in order. */
private class Inbox {
    /** Answers in delivery order. */
    val answers = LinkedBlockingQueue<SearchAnswer>()

    /** Number of refresh answers that [until] and [next] have taken so far. */
    var refreshesTaken: Int = 0

    /** Takes the next answer or fails the case after the wait. */
    fun next(): SearchAnswer {
        val answer = answers.poll(ANSWER_WAIT_SECONDS, TimeUnit.SECONDS) ?: error("no answer in time")
        if (answer.refresh) refreshesTaken += 1
        return answer
    }

    /** Takes answers until one satisfies [wanted]. */
    fun until(wanted: (SearchAnswer) -> Boolean): SearchAnswer {
        while (true) {
            val answer = next()
            if (wanted(answer)) return answer
        }
    }
}

/** Verifies the service answers during ingest (D111) and agrees with the old matcher at rest. */
class SearchServiceTest {
    /** A query is answered from the tracks known, then re-answered in place when more arrive. */
    @Test
    fun anOpenQueryRefreshesWhenTracksArrive() {
        val inbox = Inbox()
        SearchService(deliver = { inbox.answers.put(it) }, refreshIntervalMillis = 10L).use { service ->
            val generation = service.generation
            service.append(generation, listOf("Artist/Camellia - one.flac"), listOf("u1"))
            val first = run {
                service.query("cam")
                inbox.until { !it.refresh }
            }
            assertEquals(1, (first.state as SearchState.Results).items.size)
            assertEquals(1, first.knownTracks)
            service.append(generation, listOf("Artist/Camellia - two.flac"), listOf("u2"))
            val refreshed = inbox.until { it.refresh && it.knownTracks == 2 }
            assertEquals(first.sequence, refreshed.sequence)
            assertEquals(2, (refreshed.state as SearchState.Results).items.size)
            assertFalse(refreshed.complete)
            service.setComplete(true)
            assertTrue(inbox.until { it.complete }.complete)
        }
    }

    /** A feeder of a replaced library is refused, and the new library answers. */
    @Test
    fun replacingTheLibraryRefusesTheOldFeeder() {
        val inbox = Inbox()
        SearchService(deliver = { inbox.answers.put(it) }).use { service ->
            val old = service.generation
            service.append(old, listOf("A/one.flac"), null)
            val fresh = service.replaceLibrary(listOf("B/two.flac"), listOf("u"))
            assertFalse(service.append(old, listOf("A/late.flac"), null))
            service.query("two")
            val answer = inbox.until { !it.refresh }
            assertEquals(fresh, answer.generation)
            assertEquals(1, (answer.state as SearchState.Results).items.size)
            assertEquals("u", (answer.state as SearchState.Results).items[0].trackUri)
        }
    }

    /** New display paths replace the folder table and keep the titles searchable. */
    @Test
    fun replacingPathsKeepsTitlesAndUpdatesFolders() {
        val inbox = Inbox()
        SearchService(deliver = { inbox.answers.put(it) }).use { service ->
            val generation = service.generation
            service.append(generation, listOf("Music/Rock/song.flac", "Music/Pop/tune.flac"), listOf("u1", "u2"))
            service.query("rock")
            val before = inbox.until { !it.refresh }
            assertTrue("no top-level folder or title is named rock yet", before.state is SearchState.NoMatch)
            assertTrue(service.replacePaths(generation, listOf("Rock/song.flac", "Pop/tune.flac")))
            service.query("rock")
            val after = inbox.until { !it.refresh && it.sequence > before.sequence }
            val folder = (after.state as SearchState.Results).items.single { it.kind == SearchResultKind.FOLDER }
            assertEquals("Rock", folder.name)
        }
    }

    /** The open query is re-answered in place, with the same sequence and a growing count, while a scan runs. */
    @Test
    fun theOpenQueryRefreshesInPlaceWhileAScanRuns() {
        val tracks = buildSyntheticLibrary(artists = 20).tracks
        val inbox = Inbox()
        SearchService(deliver = { inbox.answers.put(it) }, refreshIntervalMillis = 20L).use { service ->
            val generation = service.generation
            service.append(generation, tracks.take(SCAN_BATCH).map { it.displayPath }, tracks.take(SCAN_BATCH).map { it.uri })
            val sequence = service.query("ka")
            val first = inbox.until { it.sequence == sequence && !it.refresh }
            val scan = Thread {
                var from = SCAN_BATCH
                while (from < tracks.size) {
                    val to = minOf(tracks.size, from + SCAN_BATCH)
                    service.append(generation, tracks.subList(from, to).map { it.displayPath }, tracks.subList(from, to).map { it.uri })
                    from = to
                    Thread.sleep(5L)
                }
                service.setComplete(true)
            }
            scan.start()
            val counts = ArrayList<Int>()
            val sizes = ArrayList<Int>()
            while (true) {
                val answer = inbox.next()
                assertEquals("a refresh keeps the query's sequence", sequence, answer.sequence)
                assertTrue(answer.refresh)
                counts.add(answer.knownTracks)
                sizes.add((answer.state as SearchState.Results).items.size)
                if (answer.complete) break
            }
            scan.join()
            assertTrue("at least three refreshes arrived while the scan ran (${counts.size})", counts.size >= 3)
            assertEquals("counts never go backwards", counts.sorted(), counts)
            assertEquals("result lists never shrink for a growing library", sizes.sorted(), sizes)
            assertTrue("the first refresh followed the first answer", counts.first() > first.knownTracks)
            assertEquals(tracks.size, counts.last())
        }
    }

    /**
     * Types while a deliberately slow scan adds a hundred thousand tracks in 128-track batches: results exist for the
     * tracks known at each keystroke, refreshes arrive, latency stays inside the bar, the write lock is held briefly,
     * and the final answers equal the old matcher's.
     */
    @Test
    fun typingDuringASlowScanAnswersFromTheKnownTracks() {
        val library = buildSyntheticLibrary(artists = 100)
        val tracks = library.tracks
        val inbox = Inbox()
        SearchService(deliver = { inbox.answers.put(it) }, refreshIntervalMillis = 30L).use { service ->
            val generation = service.generation
            val scanned = java.util.concurrent.atomic.AtomicInteger(0)
            val scan = Thread {
                var from = 0
                while (from < tracks.size) {
                    val to = minOf(tracks.size, from + SCAN_BATCH)
                    service.append(generation, tracks.subList(from, to).map { it.displayPath }, tracks.subList(from, to).map { it.uri })
                    scanned.set(to)
                    from = to
                    Thread.sleep(1L)
                }
                service.setComplete(true)
            }
            scan.start()
            val latencies = ArrayList<Double>()
            var refreshes = 0
            var typed = 0
            val sizesSeen = ArrayList<Int>()
            while (scan.isAlive) {
                for (query in listOf("k", "ka", "kal", "kalo")) {
                    val knownBefore = scanned.get()
                    val started = System.nanoTime()
                    val sequence = service.query(query)
                    val answer = inbox.until { it.sequence == sequence && !it.refresh }
                    latencies.add((System.nanoTime() - started) / 1e6)
                    typed += 1
                    assertTrue("answer covers the tracks known when typing began", answer.knownTracks >= knownBefore - SCAN_BATCH)
                    sizesSeen.add(answer.knownTracks)
                    if (answer.state is SearchState.Results) {
                        assertTrue((answer.state as SearchState.Results).items.isNotEmpty() || knownBefore < SCAN_BATCH)
                    }
                    Thread.sleep(15L)
                }
            }
            scan.join()
            refreshes = inbox.refreshesTaken
            latencies.sort()
            val p95 = latencies[(latencies.size * 95 / 100).coerceAtMost(latencies.size - 1)]
            println("D111 typed=$typed refreshes_seen=$refreshes known_range=${sizesSeen.first()}..${sizesSeen.last()} " +
                "latency_ms median=${latencies[latencies.size / 2]} p95=$p95 max=${latencies.last()} " +
                "longest_push_ms=${service.longestPushMillis} mean_push_ms=${service.meanPushMillis}")
            assertTrue("several keystrokes were typed during the scan", typed >= 8)
            assertTrue("answers grew with the scan", sizesSeen.last() > sizesSeen.first())
            assertTrue("keystroke latency p95 $p95 ms inside the bar", p95 < 500.0)
            assertTrue("write lock held ${service.longestPushMillis} ms", service.longestPushMillis < 100.0)
            for (query in listOf("k", "ka", "kal", "kalo")) {
                val sequence = service.query(query)
                val answer = inbox.until { it.sequence == sequence && !it.refresh }
                assertEquals(tracks.size, answer.knownTracks)
                assertTrue(answer.complete)
                assertEquals("final answer for $query", referenceSearchLibrary(query, tracks, true), answer.state)
            }
        }
    }
}
