// Cases of `measureNextBatch` that walk a whole lap of the page: every track once, from every kind of start.

// What:     `package ...core` places the test beside `TrackMeasure.kt`, which declares the function.
// Why:      The test reaches `measureNextBatch` and `TrackMeasure` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class MeasureNextBatchLapTest` groups the whole-lap and boundary-start cases of the batch choice.
// Why:      The tracks on screen are read first and the rest of the page follows, so a lap from the start position must
//           reach every track exactly once, and the boundary starts (last position, past the end) must be exact.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("measureNextBatch laps", () => { /* cases */ });
// ```
/** Verifies a batch large enough for the whole page visits each track once, and the edge start positions. */
class MeasureNextBatchLapTest {
    /** Builds a track whose display path is its URI, which is enough for the batch choice. */
    private fun track(uri: String): Track = Track(uri = uri, displayPath = uri)

    /** Five tracks to scan, none measured yet. */
    private val fivePage: List<Track> = listOf(track("a"), track("b"), track("c"), track("d"), track("e"))

    // What:     `@Test fun aFullLapFromTheMiddleVisitsEveryTrackOnce()` checks a batch larger than the page.
    // Why:      A lap that stopped one track short would leave the track before the start unread for the whole pass,
    //           and a lap that ran one track long would hand the same track to the reader twice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("full lap", () => expect(nextBatch(five, {}, new Set(), 8, 2).map(uri)).toEqual(["c","d","e","a","b"]));
    // ```
    /** A batch of eight from the third track of five holds the five tracks once each, starting at the third. */
    @Test
    fun aFullLapFromTheMiddleVisitsEveryTrackOnce() {
        /** Batch chosen with room for more tracks than the page has. */
        val batch: List<Track> = measureNextBatch(fivePage, emptyMap(), emptySet(), batchSize = 8, startAt = 2)
        assertEquals(listOf("c", "d", "e", "a", "b"), batch.map { track -> track.uri })
    }

    // What:     `@Test fun aFullLapFromTheFirstTrackVisitsEveryTrackOnce()` checks the same lap from the page start.
    // Why:      The default start of the first track must also stop after one lap.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("lap from the top", () => expect(nextBatch(five, {}, new Set(), 8, 0).map(uri)).toEqual(five.map(uri)));
    // ```
    /** A batch of eight from the first track of five holds the five tracks once each in page order. */
    @Test
    fun aFullLapFromTheFirstTrackVisitsEveryTrackOnce() {
        /** Batch chosen with room for more tracks than the page has. */
        val batch: List<Track> = measureNextBatch(fivePage, emptyMap(), emptySet(), batchSize = 8, startAt = 0)
        assertEquals(listOf("a", "b", "c", "d", "e"), batch.map { track -> track.uri })
    }

    // What:     `@Test fun aStartAtTheLastTrackBeginsThereAndWraps()` checks the highest valid start position.
    // Why:      The track on screen can be the last one of the page, and the scan must begin there, not at the top.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("last position", () => expect(nextBatch([a,b,c,d,e], {}, new Set(), 3, 4).map(uri)).toEqual(["e","a","b"]));
    // ```
    /** A start at the last position of five gives that track first, then the first two tracks. */
    @Test
    fun aStartAtTheLastTrackBeginsThereAndWraps() {
        /** Batch of three that begins at the last track. */
        val batch: List<Track> = measureNextBatch(fivePage, emptyMap(), emptySet(), batchSize = 3, startAt = 4)
        assertEquals(listOf("e", "a", "b"), batch.map { track -> track.uri })
    }

    // What:     `@Test fun aStartPastTheEndBeginsAtTheFirstTrackWhateverThePageSize()` checks an out-of-range start.
    // Why:      A page that shrank under a remembered start must restart at its top; wrapping the start by the page
    //           size instead would begin in the middle of the page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("past the end", () => expect(nextBatch([a,b,c], {}, new Set(), 3, 4).map(uri)).toEqual(["a","b","c"]));
    // ```
    /** A start of four on a page of three (not a multiple of the size) begins at the first track. */
    @Test
    fun aStartPastTheEndBeginsAtTheFirstTrackWhateverThePageSize() {
        /** Page of three, so a start of four would wrap to the second track if it were wrapped. */
        val page: List<Track> = listOf(track("a"), track("b"), track("c"))
        /** Batch of three from the out-of-range start. */
        val batch: List<Track> = measureNextBatch(page, emptyMap(), emptySet(), batchSize = 3, startAt = 4)
        assertEquals(listOf("a", "b", "c"), batch.map { track -> track.uri })
    }

    // What:     `@Test fun aSingleTrackPageGivesThatTrackOnce()` checks the smallest non-empty page.
    // Why:      The wrap arithmetic with a page of one must neither skip nor repeat the only track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("single track", () => expect(nextBatch([a], {}, new Set(), 4, 0).map(uri)).toEqual(["a"]));
    // ```
    /** A batch of four on a one-track page holds that track once. */
    @Test
    fun aSingleTrackPageGivesThatTrackOnce() {
        /** The only track of the page. */
        val page: List<Track> = listOf(track("a"))
        /** Batch with room for more tracks than the page has. */
        val batch: List<Track> = measureNextBatch(page, emptyMap(), emptySet(), batchSize = 4, startAt = 0)
        assertEquals(listOf("a"), batch.map { track -> track.uri })
    }
}
