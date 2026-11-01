// Cases of `pageMeasurePasses` and `startPositionFor`: where a pass starts, what each batch re-reads, and that a sweep
// step never cancels a pass that is reading.

// What:     `package ...core` places the test beside `PageMeasureRefresh.kt`, which declares the functions.
// Why:      The test reaches the internal `pageMeasurePasses` and `startPositionFor` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Job
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class PageMeasureRefreshTest` groups the cases of the page measure refresh.
// Why:      The host passes the sweep steps, the visible track, and the reads, and every decision between them is in
//           `pageMeasurePasses`: the start position, the per-batch re-reads, the attempt memory, and the scheduling.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("pageMeasurePasses", () => { /* cases */ });
// ```
/** Verifies the page measure refresh with fake reads, so no device, context, or Compose runtime is involved. */
class PageMeasureRefreshTest {
    /** Longest wait for a case, so a refresh that never ends fails the case instead of hanging it. */
    private val waitLimitMs: Long = 5_000

    /** Most batches one pass over the five tracks may read, so a pass that never ends fails the case at once. */
    private val readLimit: Int = 20

    /** Five tracks of one page, in page order. */
    private val page: List<Track> = listOf("a", "b", "c", "d", "e").map { uri -> Track(uri = uri, displayPath = uri) }

    /** A measure with a known peak, which a track needs no further read after. */
    private val measured: TrackMeasure = TrackMeasure(durationSec = 1.0, peakDbtp = -3.0, peakExact = true)

    /** A measure whose peak is unknown, which a track is read again for in the next pass. */
    private val unmeasured: TrackMeasure = TrackMeasure(durationSec = 1.0)

    /** Reads and publishes one pass the way the host does, recording what each step of it saw. */
    private class Recorder(val startUri: AtomicReference<String?>, val peak: Boolean) {
        /** Measures published so far, which the next batch choice reads as the known measures. */
        val known: AtomicReference<Map<String, TrackMeasure>> = AtomicReference(emptyMap())

        /** Batches handed to the read, as the URIs of their tracks, in read order. */
        val reads: MutableList<List<String>> = CopyOnWriteArrayList()

        /** Lines the refresh logged. */
        val lines: MutableList<String> = CopyOnWriteArrayList()
    }

    /** Builds a recorder whose visible track starts as [startUri] and whose reads give a peak when [peak] is set. */
    private fun recorder(startUri: String?, peak: Boolean = true): Recorder =
        Recorder(startUri = AtomicReference(startUri), peak = peak)

    /** Runs one pass over [page] with [batchSize] tracks per batch against [recorder] and returns when it ends. */
    private fun runOnePass(recorder: Recorder, batchSize: Int, afterRead: (List<String>) -> Unit = {}) = runBlocking {
        withTimeout(waitLimitMs) {
            pageMeasurePasses(
                steps = flowOf(0),
                tracks = page,
                known = { recorder.known.get() },
                firstVisibleUri = { recorder.startUri.get() },
                read = { batch ->
                    /** URIs of the batch being read. */
                    val uris: List<String> = batch.map { track -> track.uri }
                    recorder.reads += uris
                    check(recorder.reads.size <= readLimit) { "The pass read $readLimit batches without ending." }
                    afterRead(uris)
                    batch.associate { track -> track.uri to if (recorder.peak) measured else unmeasured }
                },
                publish = { updates -> recorder.known.updateAndGet { current -> current + updates } },
                log = { line -> recorder.lines += line },
                batchSize = batchSize,
            )
        }
    }

    // What:     `@Test fun aTrackUriNamesItsPositionInThePage()` checks `startPositionFor` on a present URI.
    // Why:      A start position of zero for every URI would read the page from the top and ignore the screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("position of a uri", () => expect(startPositionFor(page, "c")).toBe(2));
    // ```
    /** The position of "c" among a to e is two, and the first track is zero. */
    @Test
    fun aTrackUriNamesItsPositionInThePage() {
        assertEquals(2, startPositionFor(page, "c"))
        assertEquals(0, startPositionFor(page, "a"))
        assertEquals(4, startPositionFor(page, "e"))
    }

    // What:     `@Test fun anUnknownOrAbsentUriHasNoPosition()` checks `startPositionFor` on a URI the page lacks.
    // Why:      The list may not have reported yet, or the page may have changed; the batch choice reads the negative
    //           value as the start of the page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no position", () => {
    //   expect(startPositionFor(page, "z")).toBe(-1);
    //   expect(startPositionFor(page, null)).toBe(-1);
    // });
    // ```
    /** A URI outside the page and a null URI both give the negative value. */
    @Test
    fun anUnknownOrAbsentUriHasNoPosition() {
        assertEquals(-1, startPositionFor(page, "z"))
        assertEquals(-1, startPositionFor(page, null))
    }

    // What:     `@Test fun theFirstBatchStartsAtTheFirstVisibleTrack()` checks the rows on screen are read first.
    // Why:      On a page of thousands of tracks, a row deep in the page used to wait for every row above it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("starts at the visible track", () => {
    //   expect(firstBatch({ visible: "c", batchSize: 2 })).toEqual(["c", "d"]);
    // });
    // ```
    /** With "c" on screen and two tracks per batch, the first read holds "c" and "d", and the pass then wraps. */
    @Test
    fun theFirstBatchStartsAtTheFirstVisibleTrack() {
        /** Recorder of the pass. */
        val recorder: Recorder = recorder(startUri = "c")
        runOnePass(recorder, batchSize = 2)
        assertEquals(listOf(listOf("c", "d"), listOf("e", "a"), listOf("b")), recorder.reads.toList())
    }

    // What:     `@Test fun noVisibleTrackStartsAtTheTopOfThePage()` checks the start without a reported track.
    // Why:      Before the list reports, and for a track the page lacks, the pass reads the page from its first track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("starts at the top", () => expect(firstBatch({ visible: null, batchSize: 2 })).toEqual(["a", "b"]));
    // ```
    /** A null visible URI and an unknown one both begin at "a". */
    @Test
    fun noVisibleTrackStartsAtTheTopOfThePage() {
        /** Recorder with no visible track. */
        val nothing: Recorder = recorder(startUri = null)
        runOnePass(nothing, batchSize = 2)
        assertEquals(listOf("a", "b"), nothing.reads.first())
        /** Recorder whose visible track is not on the page. */
        val unknown: Recorder = recorder(startUri = "z")
        runOnePass(unknown, batchSize = 2)
        assertEquals(listOf("a", "b"), unknown.reads.first())
    }

    // What:     `@Test fun everyBatchRereadsTheVisibleTrackAndTheKnownMeasures()` checks the per-batch re-reads.
    // Why:      The user scrolls while a pass reads, so the next batch must start where the page is shown by then, and
    //           must skip what an earlier batch already published.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("re-reads per batch", () => { /* visible c then a: [c,d], [a,b], [e] */ });
    // ```
    /** Scrolling to "a" during the first read makes the second batch start at "a" and skip the published tracks. */
    @Test
    fun everyBatchRereadsTheVisibleTrackAndTheKnownMeasures() {
        /** Recorder that starts at "c". */
        val recorder: Recorder = recorder(startUri = "c")
        runOnePass(recorder, batchSize = 2) { uris ->
            if (uris == listOf("c", "d")) {
                recorder.startUri.set("a")
            }
        }
        assertEquals(listOf(listOf("c", "d"), listOf("a", "b"), listOf("e")), recorder.reads.toList())
    }

    // What:     `@Test fun aMeasureThatArrivesDuringAReadIsNotReadAgain()` checks the known measures are read anew
    //           for each batch.
    // Why:      The sweep publishes measures while a pass reads, and a track it already measured must not be read again
    //           by the batches that follow.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("skips what arrived", () => { /* c measured by the sweep during [a,b]: next batch is [d,e] */ });
    // ```
    /** A measure for "c" that appears during the first read makes the second batch skip "c". */
    @Test
    fun aMeasureThatArrivesDuringAReadIsNotReadAgain() {
        /** Recorder that starts at the top of the page. */
        val recorder: Recorder = recorder(startUri = null)
        runOnePass(recorder, batchSize = 2) { uris ->
            if (uris == listOf("a", "b")) {
                recorder.known.updateAndGet { current -> current + ("c" to measured) }
            }
        }
        assertEquals(listOf(listOf("a", "b"), listOf("d", "e")), recorder.reads.toList())
    }

    // What:     `@Test fun aTrackWithoutAPeakIsReadOncePerPass()` checks the attempt memory.
    // Why:      A track that never gets a peak stays unmeasured, and without the memory the pass would read it forever.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one attempt per pass", () => expect(reads({ peak: false })).toEqual([["a","b"],["c","d"],["e"]]));
    // ```
    /** With reads that never give a peak, each of the five tracks is still read exactly once and the pass ends. */
    @Test
    fun aTrackWithoutAPeakIsReadOncePerPass() {
        /** Recorder whose reads give no peak. */
        val recorder: Recorder = recorder(startUri = null, peak = false)
        runOnePass(recorder, batchSize = 2)
        assertEquals(listOf(listOf("a", "b"), listOf("c", "d"), listOf("e")), recorder.reads.toList())
    }

    // What:     `@Test fun everyBatchIsPublishedAndTheEndOfThePassIsLogged()` checks the outputs of a pass.
    // Why:      A batch must land on screen as soon as it is read, and the log line is the only trace of a finished
    //           pass in logcat.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("publishes and logs", () => { /* known holds all five; log says attempted=5 */ });
    // ```
    /** After one pass the published measures hold all five tracks and the log holds one completion line. */
    @Test
    fun everyBatchIsPublishedAndTheEndOfThePassIsLogged() {
        /** Recorder of the pass. */
        val recorder: Recorder = recorder(startUri = null)
        runOnePass(recorder, batchSize = 2)
        assertEquals(setOf("a", "b", "c", "d", "e"), recorder.known.get().keys)
        assertEquals(listOf("page measures are complete attempted=5"), recorder.lines.toList())
    }

    // What:     `@Test fun aStepArrivingDuringAReadDoesNotCancelIt()` checks the property the sweep fix rests on.
    // Why:      The sweep reports every few tracks. A refresh that restarted at each step cancelled the batch in flight
    //           and began again at the top, so a large page got its measures only after the sweep ended.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a step does not cancel a read", async () => { /* start read, send step, expect 1 start and 0 cancels */ });
    // ```
    /** A second step while the first read waits leaves that read running, and its batch is published once released. */
    @Test
    fun aStepArrivingDuringAReadDoesNotCancelIt() = runBlocking {
        /** Steps the refresh collects. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        /** Opens when the first read may finish. */
        val release = CompletableDeferred<Unit>()
        /** Count of reads that started. */
        val started = AtomicInteger(0)
        /** Count of reads that were cancelled. */
        val cancelled = AtomicInteger(0)
        /** Count of batches published. */
        val published = AtomicInteger(0)
        /** The refresh under test. */
        val refresh: Job = launch {
            pageMeasurePasses(
                steps = steps.receiveAsFlow(),
                tracks = page,
                known = { emptyMap() },
                firstVisibleUri = { null },
                read = { batch ->
                    check(started.incrementAndGet() <= readLimit) { "The refresh read $readLimit batches, endlessly." }
                    try {
                        release.await()
                    } catch (cancellation: CancellationException) {
                        cancelled.incrementAndGet()
                        throw cancellation
                    }
                    batch.associate { track -> track.uri to measured }
                },
                publish = { published.incrementAndGet() },
                log = {},
                batchSize = 24,
            )
        }
        steps.send(1)
        awaitCount("the first read to start") { started.get() == 1 }
        steps.send(2)
        delay(150)
        assertEquals("the step restarted the read", 1, started.get())
        assertEquals("the step cancelled the read", 0, cancelled.get())
        release.complete(Unit)
        awaitCount("the first batch to be published") { published.get() >= 1 }
        steps.close()
        refresh.join()
        assertEquals("the read in flight must have been published, not discarded", 0, cancelled.get())
    }

    // What:     `@Test fun everySweepStepAfterAFinishedPassStartsAnotherPass()` checks the steps drive the passes.
    // Why:      The sweep may fill peaks after a pass ended, so each later step must check the page again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a step per pass", async () => { /* send 1, wait for its log line, send 2, expect two lines */ });
    // ```
    /** Two steps, the second sent after the first pass ended, give two completed passes. */
    @Test
    fun everySweepStepAfterAFinishedPassStartsAnotherPass() = runBlocking {
        /** Steps the refresh collects. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        /** Lines the refresh logged, one per finished pass. */
        val lines: MutableList<String> = CopyOnWriteArrayList()
        /** The refresh under test. */
        val refresh: Job = launch {
            pageMeasurePasses(
                steps = steps.receiveAsFlow(),
                tracks = page,
                known = { page.associate { track -> track.uri to measured } },
                firstVisibleUri = { null },
                read = { emptyMap() },
                publish = {},
                log = { line -> lines += line },
                batchSize = 24,
            )
        }
        steps.send(1)
        awaitCount("the first pass to end") { lines.size == 1 }
        steps.send(2)
        awaitCount("the second pass to end") { lines.size == 2 }
        steps.close()
        refresh.join()
        assertEquals(2, lines.size)
    }

    /** Waits until [condition] holds, polling in short sleeps, and fails the case when the limit passes first. */
    private suspend fun awaitCount(what: String, condition: () -> Boolean) {
        try {
            withTimeout(waitLimitMs) {
                while (!condition()) {
                    delay(5)
                }
            }
        } catch (timeout: TimeoutCancellationException) {
            throw AssertionError("Timed out waiting for $what.", timeout)
        }
    }
}
