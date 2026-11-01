// Unit tests for the pure rules of the track measures: peak text, the native reply, the details value,
// the batch choice, and the index mapping. No Android types are involved.

// What:     This package matches the measure model's package, so the tests reach its functions directly.
// Why:      The rules are plain functions over values, so they need no device.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports the library track type, which the batch choice takes as its page.
// Why:      The page is a list of tracks keyed by URI.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "../Track";
// ```
import dev.monochromatic.musicplayer.Track

// What:     JUnit assertions and the test annotation for the measure rules.
// Why:      Each expected string and each rejected input is checked exactly.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class TrackMeasureTest` groups the measure rule tests in one file.
// Why:      The formatting, the reply, the batch, and the mapping rules are one feature.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("TrackMeasure", () => { /* ... */ });
// ```
/** Verifies the formatting, decoding, and selection rules of the track measures. */
class TrackMeasureTest {
    // What:     `@Test fun formatsNegativePeakWithU2212Minus()` checks the usual negative true peak.
    // Why:      The design writes `−1.2 dBTP`, with U+2212 rather than a hyphen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("formats a negative peak", () => expect(formatPeakDbtp(-1.2)).toBe("−" + "1.2 dBTP"));
    // ```
    /** A negative peak prints its magnitude after the U+2212 minus and the unit. */
    @Test
    fun formatsNegativePeakWithU2212Minus() {
        assertEquals("−1.2 dBTP", measureFormatPeakDbtp(-1.2))
    }

    // What:     `@Test fun formatsPositivePeakWithPlusSign()` checks a peak above zero.
    // Why:      A true peak can rise above zero dBTP, and its sign must show the direction.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("formats a positive peak", () => expect(formatPeakDbtp(0.3)).toBe("+0.3 dBTP"));
    // ```
    /** A positive peak prints a plus sign before its value. */
    @Test
    fun formatsPositivePeakWithPlusSign() {
        assertEquals("+0.3 dBTP", measureFormatPeakDbtp(0.3))
    }

    // What:     `@Test fun roundsToOneDecimalPlace()` checks the rounding to tenths.
    // Why:      A peak with more digits than the display shows must round to the nearest tenth.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("rounds to a tenth", () => expect(formatPeakDbtp(-1.26)).toBe("−1.3 dBTP"));
    // ```
    /** A value between tenths rounds to the nearest one. */
    @Test
    fun roundsToOneDecimalPlace() {
        assertEquals("−1.3 dBTP", measureFormatPeakDbtp(-1.26))
    }

    // What:     `@Test fun neverPrintsNegativeZero()` checks peaks that round to zero.
    // Why:      A value of -0.04 must read `0.0 dBTP`, not `−0.0 dBTP`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no negative zero", () => expect(formatPeakDbtp(-0.04)).toBe("0.0 dBTP"));
    // ```
    /** A value that rounds to zero prints without a sign. */
    @Test
    fun neverPrintsNegativeZero() {
        assertEquals("0.0 dBTP", measureFormatPeakDbtp(-0.04))
        assertEquals("0.0 dBTP", measureFormatPeakDbtp(0.0))
    }

    // What:     `@Test fun formatsSilenceAsInfinity()` checks the negative-infinite peak of silence.
    // Why:      A silent track has no finite peak, and the row must say so rather than print a number.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("silence", () => expect(formatPeakDbtp(-Infinity)).toBe("−∞ dBTP"));
    // ```
    /** Negative infinity prints as a minus sign, the infinity mark, and the unit. */
    @Test
    fun formatsSilenceAsInfinity() {
        assertEquals("−∞ dBTP", measureFormatPeakDbtp(Double.NEGATIVE_INFINITY))
    }

    // What:     `@Test fun rejectsNotANumberPeak()` checks that NaN is refused rather than formatted.
    // Why:      Rounding NaN is an error, and a NaN peak means the reply was corrupt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("NaN throws", () => expect(() => formatPeakDbtp(NaN)).toThrow());
    // ```
    /** A NaN peak is rejected with an argument error. */
    @Test
    fun rejectsNotANumberPeak() {
        assertThrows(IllegalArgumentException::class.java) { measureFormatPeakDbtp(Double.NaN) }
    }

    // What:     `@Test fun decodesExactReply()` checks a reply whose kind code marks a full scan.
    // Why:      Only an exact reply may show as the track's peak in the rows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("exact reply", () => expect(peakFromReply([-1.2, 1])).toEqual({ dbtp: -1.2, exact: true }));
    // ```
    /** A reply with the exact code decodes to an exact peak. */
    @Test
    fun decodesExactReply() {
        assertEquals(MeasurePeakReading(dbtp = -1.5, exact = true), measurePeakFromReply(floatArrayOf(-1.5f, 1f)))
    }

    // What:     `@Test fun decodesSampledReplyAsNotExact()` checks a reply whose kind code marks a probe.
    // Why:      A sampled peak is an estimate and must not be treated as exact.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("sampled reply", () => expect(peakFromReply([-1.2, 0]).exact).toBe(false));
    // ```
    /** A reply with the sampled code decodes to a peak that is not exact. */
    @Test
    fun decodesSampledReplyAsNotExact() {
        assertEquals(MeasurePeakReading(dbtp = -1.5, exact = false), measurePeakFromReply(floatArrayOf(-1.5f, 0f)))
    }

    // What:     `@Test fun rejectsMalformedReplies()` checks null, short, long, and NaN replies.
    // Why:      A reply that is not two numbers means no peak is known, so the rows keep their pending form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("malformed replies", () => { expect(peakFromReply(null)).toBeNull(); });
    // ```
    /** Null, wrong-length, and not-a-number replies all decode to no peak. */
    @Test
    fun rejectsMalformedReplies() {
        assertNull(measurePeakFromReply(null))
        assertNull(measurePeakFromReply(floatArrayOf(-1.5f)))
        assertNull(measurePeakFromReply(floatArrayOf(-1.5f, 1f, 0f)))
        assertNull(measurePeakFromReply(floatArrayOf(Float.NaN, 1f)))
    }

    // What:     `@Test fun rowTextShowsOnlyExactPeak()` checks the row's peak text against the sampled and missing
    //           cases.
    // Why:      A sampled estimate must not appear as the track's peak in a row, while an exact one must.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("row text", () => { expect(peakText(exact)).toBe("−1.2 dBTP"); expect(peakText(sampled)).toBeNull(); });
    // ```
    /** Only an exact peak gives row text; a sampled or missing peak gives none. */
    @Test
    fun rowTextShowsOnlyExactPeak() {
        assertEquals("−1.2 dBTP", measurePeakText(TrackMeasure(durationSec = 275.0, peakDbtp = -1.2, peakExact = true)))
        assertNull(measurePeakText(TrackMeasure(durationSec = 275.0, peakDbtp = -1.2, peakExact = false)))
        assertNull(measurePeakText(TrackMeasure(durationSec = 275.0)))
        assertNull(measurePeakText(null))
    }

    // What:     `@Test fun detailsValueNamesThePeakOrThePendingCopy()` checks the menu value in each state.
    // Why:      The menu row must show the peak when it is known, mark a sample, and say plainly when none is read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("details value", () => expect(detailsValue(null)).toBe("not measured yet"));
    // ```
    /** The details value is the exact peak, the sampled peak with its marker, or the pending copy. */
    @Test
    fun detailsValueNamesThePeakOrThePendingCopy() {
        assertEquals("not measured yet", measureDetailsValue(null))
        assertEquals("not measured yet", measureDetailsValue(TrackMeasure(durationSec = 275.0)))
        assertEquals(
            "−1.2 dBTP",
            measureDetailsValue(TrackMeasure(durationSec = 275.0, peakDbtp = -1.2, peakExact = true)),
        )
        assertEquals(
            "−1.2 dBTP (sampled)",
            measureDetailsValue(TrackMeasure(durationSec = 275.0, peakDbtp = -1.2, peakExact = false)),
        )
    }

    // What:     `@Test fun needsReadWhenMeasureOrPeakIsMissing()` checks which states a track is read in.
    // Why:      A track with an exact peak is done, while a missing measure or peak is read again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("needs read", () => expect(needsRead(null)).toBe(true));
    // ```
    /** A track needs a read when it has no measure or no peak, and not otherwise. */
    @Test
    fun needsReadWhenMeasureOrPeakIsMissing() {
        assertTrue(measureNeedsRead(null))
        assertTrue(measureNeedsRead(TrackMeasure(durationSec = 275.0)))
        assertFalse(measureNeedsRead(TrackMeasure(durationSec = 275.0, peakDbtp = -1.2, peakExact = true)))
    }

    // What:     `@Test fun batchTakesPageOrderAndSkipsDoneAndAttempted()` checks the batch choice.
    // Why:      A pass reads the earliest pending tracks of the page, once each, up to the batch size.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("batch", () => expect(nextBatch(page, known, attempted, 2).map(t => t.uri)).toEqual(["b", "c"]));
    // ```
    /** The batch keeps page order, skips tracks with a peak or already attempted, and stops at its size. */
    @Test
    fun batchTakesPageOrderAndSkipsDoneAndAttempted() {
        val page = listOf(track("a"), track("b"), track("c"), track("d"))
        val known = mapOf(
            "a" to TrackMeasure(durationSec = 1.0, peakDbtp = -1.0, peakExact = true),
            "b" to TrackMeasure(durationSec = 2.0),
        )
        val batch: List<Track> = measureNextBatch(page, known, attempted = setOf("c"), batchSize = 2)
        assertEquals(listOf("b", "d"), batch.map { track -> track.uri })
    }

    // What:     `@Test fun batchStopsAtBatchSize()` checks the size cap.
    // Why:      A batch never exceeds the size, so the rows refresh in bounded steps.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("batch cap", () => expect(nextBatch(page, {}, new Set(), 1).length).toBe(1));
    // ```
    /** A batch of size one returns only the first pending track. */
    @Test
    fun batchStopsAtBatchSize() {
        val page = listOf(track("a"), track("b"))
        val batch: List<Track> = measureNextBatch(page, emptyMap(), emptySet(), batchSize = 1)
        assertEquals(listOf("a"), batch.map { track -> track.uri })
    }

    // What:     `@Test fun batchRejectsEmptySize()` checks the lower bound of the batch size.
    // Why:      A batch of zero would never finish a pass, so the choice refuses it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("batch size zero", () => expect(() => nextBatch(page, {}, new Set(), 0)).toThrow());
    // ```
    /** A batch size below one is rejected with an argument error. */
    @Test
    fun batchRejectsEmptySize() {
        assertThrows(IllegalArgumentException::class.java) {
            measureNextBatch(listOf(track("a")), emptyMap(), emptySet(), batchSize = 0)
        }
    }

    // What:     `@Test fun indexMappingSkipsMissingUriAndMeasure()` checks the index-to-measure mapping.
    // Why:      Rows name tracks by index, so a missing URI or an unread track must simply have no entry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("index mapping", () => expect(measuresByIndex([0, 1], uriAt, measures)).toEqual(new Map([[0, m]])));
    // ```
    /** Only indices whose URI has a measure appear in the mapping. */
    @Test
    fun indexMappingSkipsMissingUriAndMeasure() {
        val uris = listOf("a", "b")
        val measure = TrackMeasure(durationSec = 3.0, peakDbtp = -2.0, peakExact = true)
        val mapped: Map<Int, TrackMeasure> = measuresByIndex(
            pageIndices = listOf(0, 1, 5),
            uriAt = { index -> uris.getOrNull(index) },
            measures = mapOf("a" to measure),
        )
        assertEquals(mapOf(0 to measure), mapped)
    }

    // What:     `@Test fun batchStartsAtTheGivenPositionAndWrapsToThePageStart()` checks the start position.
    // Why:      The tracks on screen are read first, so a batch from the middle of a page must begin there and then
    //           continue from the page's start once the end is reached.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("start position", () => expect(nextBatch([a,b,c,d,e], {}, new Set(), 3, 3).map(uri)).toEqual(["d","e","a"]));
    // ```
    /** A batch begins at the start position, runs to the end of the page, and wraps to the first tracks. */
    @Test
    fun batchStartsAtTheGivenPositionAndWrapsToThePageStart() {
        val page: List<Track> = listOf(track("a"), track("b"), track("c"), track("d"), track("e"))
        val batch: List<Track> = measureNextBatch(page, emptyMap(), emptySet(), batchSize = 3, startAt = 3)
        assertEquals(listOf("d", "e", "a"), batch.map { track -> track.uri })
    }

    // What:     `@Test fun batchSkipsKnownAndAttemptedTracksFromTheStartPosition()` checks the skips from a start.
    // Why:      A measured or already attempted track must be passed over wherever the scan begins.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("skips from start", () => expect(nextBatch(page, {c: full}, new Set(["d"]), 2, 2).map(uri)).toEqual(["e","a"]));
    // ```
    /** Known measures and attempted tracks are skipped by a scan that begins in the middle of the page. */
    @Test
    fun batchSkipsKnownAndAttemptedTracksFromTheStartPosition() {
        val page: List<Track> = listOf(track("a"), track("b"), track("c"), track("d"), track("e"))
        val known: Map<String, TrackMeasure> = mapOf("c" to TrackMeasure(durationSec = 1.0, peakDbtp = -1.0, peakExact = true))
        val batch: List<Track> = measureNextBatch(page, known, attempted = setOf("d"), batchSize = 2, startAt = 2)
        assertEquals(listOf("e", "a"), batch.map { track -> track.uri })
    }

    // What:     `@Test fun startOutsideThePageBeginsAtItsStart()` checks a start that points nowhere.
    // Why:      The list has not reported a visible track yet (a negative start), or the page shrank under the start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("bad start", () => { expect(nextBatch(page, {}, new Set(), 2, -1)).toEqual(first2); expect(.. 9).toEqual(first2); });
    // ```
    /** A negative start, or one past the end, begins at the first track of the page. */
    @Test
    fun startOutsideThePageBeginsAtItsStart() {
        val page: List<Track> = listOf(track("a"), track("b"), track("c"))
        assertEquals(listOf("a", "b"), measureNextBatch(page, emptyMap(), emptySet(), batchSize = 2, startAt = -1).map { it.uri })
        assertEquals(listOf("a", "b"), measureNextBatch(page, emptyMap(), emptySet(), batchSize = 2, startAt = 9).map { it.uri })
    }

    // What:     `@Test fun emptyPageGivesNoBatchFromAnyStart()` checks an empty page.
    // Why:      The wrap-around arithmetic must never divide by the size of an empty page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty page", () => expect(nextBatch([], {}, new Set(), 2, 0)).toEqual([]));
    // ```
    /** An empty page gives an empty batch. */
    @Test
    fun emptyPageGivesNoBatchFromAnyStart() {
        assertTrue(measureNextBatch(emptyList(), emptyMap(), emptySet(), batchSize = 2, startAt = 0).isEmpty())
    }

    /** Builds a track whose display path is its URI, which is enough for the batch choice. */
    private fun track(uri: String): Track = Track(uri = uri, displayPath = uri)
}
