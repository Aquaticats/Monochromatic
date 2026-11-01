// Per-track measures for the library rows and the track menu: the duration and the true peak of one
// track, read off the device and the shared decision cache. Nothing here touches Android, so the
// formatting, the reply decoding, and the batch choice are plain JVM unit tests.

// What:     `package dev.monochromatic.musicplayer.core` places the measure model beside the template
//           fields it feeds.
// Why:      The pure rules (formatting, batching) share one namespace without any Android import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import kotlin.math.abs` brings in the absolute value for `Long`.
// Why:      A negative tenth of a decibel is formatted through its magnitude, with the sign written separately.
//
// In TS you'd write (pseudocode):
// ```ts
// import { abs } from "math";
// ```
import kotlin.math.abs

// What:     `import kotlin.math.roundToLong` brings in the rounding of a `Double` to the nearest `Long`.
// Why:      A peak is shown to one decimal place, so the value is rounded in tenths of a decibel.
//
// In TS you'd write (pseudocode):
// ```ts
// import { round } from "math";
// ```
import kotlin.math.roundToLong

// What:     `import dev.monochromatic.musicplayer.Track` brings in the library entry, which holds the URI and the
//           display path a measure is keyed by.
// Why:      The batch choice works on the same tracks the player shows.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "../Track";
// ```
import dev.monochromatic.musicplayer.Track

// What:     `const val MEASURE_BATCH_SIZE: Int = 24` is the number of tracks read in one pass.
// Why:      Small batches keep each pass short, so the rows fill in as they are read.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_BATCH_SIZE = 24;
// ```
/** Tracks read in one pass of the measure refresh. */
const val MEASURE_BATCH_SIZE: Int = 24

// What:     `const val MEASURE_NOT_MEASURED_TEXT: String` is the copy shown while a peak is not known.
// Why:      The details menu must say plainly that no peak exists yet, rather than show a blank.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_NOT_MEASURED_TEXT = "not measured yet";
// ```
/** Text shown in the details menu while a track has no known peak. */
const val MEASURE_NOT_MEASURED_TEXT: String = "not measured yet"

// What:     `const val MEASURE_PEAK_UNIT: String = "dBTP"` is the unit printed after every peak value.
// Why:      True peaks are written in dBTP, the unit the design and the template default use.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_PEAK_UNIT = "dBTP";
// ```
/** Unit printed after a peak value. */
const val MEASURE_PEAK_UNIT: String = "dBTP"

// What:     `const val MEASURE_MINUS_SIGN: String = "−"` is the Unicode minus sign, U+2212.
// Why:      The design writes negative peaks with this sign, not with a hyphen-minus.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_MINUS_SIGN = "−";
// ```
/** Minus sign drawn in front of a negative peak, U+2212. */
const val MEASURE_MINUS_SIGN: String = "−"

// What:     `const val MEASURE_PLUS_SIGN: String = "+"` is drawn in front of a positive peak.
// Why:      A true peak can rise above zero dBTP, and the sign makes that direction plain.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_PLUS_SIGN = "+";
// ```
/** Sign drawn in front of a positive peak. */
const val MEASURE_PLUS_SIGN: String = "+"

// What:     `const val MEASURE_TENTHS_PER_DB: Long = 10L` is the number of tenths in one decibel.
// Why:      A peak is rounded to one decimal place by scaling it by this factor.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_TENTHS_PER_DB = 10n;
// ```
/** Tenths of a decibel in one decibel, the scale of one decimal place. */
const val MEASURE_TENTHS_PER_DB: Long = 10L

// What:     `const val MEASURE_SAMPLED_SUFFIX: String = " (sampled)"` marks a sampled estimate in the details value.
// Why:      A sampled peak sits at or below the true peak, so the details value must say it is an estimate.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_SAMPLED_SUFFIX = " (sampled)";
// ```
/** Suffix that marks a sampled peak as an estimate. */
const val MEASURE_SAMPLED_SUFFIX: String = " (sampled)"

// What:     `const val MEASURE_EXACT_CODE: Float = 1.0f` is the kind code the native layer writes for an exact peak.
// Why:      The native reply carries the kind as a float, and this constant names the exact code once.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_EXACT_CODE = 1.0;
// ```
/** Kind code of an exact peak in the native reply. */
const val MEASURE_EXACT_CODE: Float = 1.0f

// What:     `const val MEASURE_REPLY_LENGTH: Int = 2` is the number of slots in a native peek reply.
// Why:      A reply shorter or longer than two slots is malformed, and is treated as no peak.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_REPLY_LENGTH = 2;
// ```
/** Slots in one native peek reply: the peak in dBTP, then the kind code. */
const val MEASURE_REPLY_LENGTH: Int = 2

// What:     `data class TrackMeasure(...)` holds the duration and the true peak of one track.
// Why:      The rows and the details menu read both values from one place, and an absent measure means not read yet.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrackMeasure = { durationSec: number | null; peakDbtp: number | null; peakExact: boolean };
// ```
/**
 * Duration and true peak of one track, each null while it is not known.
 */
data class TrackMeasure(
    /** Duration in seconds, or null when the file's length is not known. */
    val durationSec: Double?,
    /** True peak in dBTP, or null when no decision is cached for the track. */
    val peakDbtp: Double? = null,
    /** True when the peak came from a full scan, and false when it is a sampled estimate. */
    val peakExact: Boolean = false,
)

// What:     `data class MeasurePeakReading(...)` is one peak decoded from the native reply.
// Why:      The reply is two raw floats; this type names the dBTP value and whether it is exact.
//
// In TS you'd write (pseudocode):
// ```ts
// type MeasurePeakReading = { dbtp: number; exact: boolean };
// ```
/**
 * One peak decoded from the native reply, with its dBTP value and whether it came from a full scan.
 */
data class MeasurePeakReading(
    /** Peak in dBTP, which is negative-infinite for silence. */
    val dbtp: Double,
    /** True when the peak came from a full scan. */
    val exact: Boolean,
)

// What:     `fun measurePeakFromReply(reply: FloatArray?): MeasurePeakReading?` decodes the native reply.
// Why:      A null, short, long, or NaN reply means no usable peak, so the caller shows the pending copy.
//
// In TS you'd write (pseudocode):
// ```ts
// function measurePeakFromReply(reply: Float32Array | null): MeasurePeakReading | null { ... }
// ```
/**
 * Decodes the two-slot native peek reply, or returns null when the reply holds no usable peak.
 */
fun measurePeakFromReply(reply: FloatArray?): MeasurePeakReading? {
    if (reply == null || reply.size != MEASURE_REPLY_LENGTH) {
        return null
    }
    /** Peak in dBTP from the first slot, widened to a `Double`. */
    val dbtp: Double = reply[0].toDouble()
    if (dbtp.isNaN()) {
        return null
    }
    return MeasurePeakReading(dbtp = dbtp, exact = reply[1] == MEASURE_EXACT_CODE)
}

// What:     `private fun measureInfiniteText(dbtp: Double): String` writes an infinite peak, which silence produces.
// Why:      A silent track has a peak of negative infinity, and the value must read as infinity, not as a number.
//
// In TS you'd write (pseudocode):
// ```ts
// function infiniteText(dbtp: number): string { return `${dbtp < 0 ? "−" : "+"}∞ dBTP`; }
// ```
/** Text of an infinite peak, with its sign, the infinity mark, and the unit. */
private fun measureInfiniteText(dbtp: Double): String {
    /** Sign in front of the infinity mark. */
    val sign: String = if (dbtp < 0.0) MEASURE_MINUS_SIGN else MEASURE_PLUS_SIGN
    return "$sign∞ $MEASURE_PEAK_UNIT"
}

// What:     `fun measureFormatPeakDbtp(dbtp: Double): String` writes a peak with one decimal, a sign, and its unit.
// Why:     Every peak in the app reads the same way, such as `−1.2 dBTP`, with no `−0.0` for a peak that rounds to
//           zero.
//
// In TS you'd write (pseudocode):
// ```ts
// function formatPeakDbtp(dbtp: number): string { ... }
// ```
/**
 * Formats a peak in dBTP with one decimal place, the U+2212 minus for negatives, and the unit.
 */
fun measureFormatPeakDbtp(dbtp: Double): String {
    require(!dbtp.isNaN()) { "Peak in dBTP must be a number, was NaN." }
    if (dbtp.isInfinite()) {
        return measureInfiniteText(dbtp)
    }
    /** The peak in tenths of a decibel, rounded to the nearest whole tenth. */
    val tenths: Long = (dbtp * MEASURE_TENTHS_PER_DB).roundToLong()
    /** Sign drawn in front of the value, empty for a value that rounds to zero. */
    val sign: String = when {
        tenths < 0L -> MEASURE_MINUS_SIGN
        tenths > 0L -> MEASURE_PLUS_SIGN
        else -> ""
    }
    /** Magnitude in tenths, so the whole and fraction digits do not carry a sign. */
    val magnitude: Long = abs(tenths)
    return "$sign${magnitude / MEASURE_TENTHS_PER_DB}.${magnitude % MEASURE_TENTHS_PER_DB} $MEASURE_PEAK_UNIT"
}

// What:     `fun measurePeakText(measure: TrackMeasure?): String?` is the supporting-line text of a peak.
// Why:      Rows show only an exact peak, so a sampled estimate leaves the template in its pending form.
//
// In TS you'd write (pseudocode):
// ```ts
// function peakText(measure: TrackMeasure | null): string | null { ... }
// ```
/**
 * Text of an exact peak for the track row template, or null while the peak is unknown or only sampled.
 */
fun measurePeakText(measure: TrackMeasure?): String? {
    if (measure == null || !measure.peakExact) {
        return null
    }
    /** Peak in dBTP, or null when the measure carries no peak. */
    val dbtp: Double = measure.peakDbtp ?: return null
    return measureFormatPeakDbtp(dbtp)
}

// What:     `fun measureDetailsValue(measure: TrackMeasure?): String` is the value the details menu row shows.
// Why:     The row shows the peak when one is known, marks a sampled peak as an estimate, and says plainly when none
//           exists.
//
// In TS you'd write (pseudocode):
// ```ts
// function detailsValue(measure: TrackMeasure | null): string { ... }
// ```
/**
 * Value drawn after File details: the peak with its unit, a sampled marker when it is an estimate, or the pending copy.
 */
fun measureDetailsValue(measure: TrackMeasure?): String {
    if (measure == null) {
        return MEASURE_NOT_MEASURED_TEXT
    }
    /** Peak in dBTP, or null when the measure has no peak yet. */
    val dbtp: Double = measure.peakDbtp ?: return MEASURE_NOT_MEASURED_TEXT
    /** The peak text with its unit. */
    val text: String = measureFormatPeakDbtp(dbtp)
    if (measure.peakExact) {
        return text
    }
    return text + MEASURE_SAMPLED_SUFFIX
}

// What:     `fun measureNeedsRead(measure: TrackMeasure?): Boolean` says whether a track still needs a read.
// Why:     A track is read when it has no measure yet, or when its peak is still missing and a later sweep may have
//           filled it.
//
// In TS you'd write (pseudocode):
// ```ts
// function needsRead(measure: TrackMeasure | null): boolean { return measure === null || measure.peakDbtp === null; }
// ```
/** True when the track has no measure yet, or its peak is still unknown. */
fun measureNeedsRead(measure: TrackMeasure?): Boolean = measure == null || measure.peakDbtp == null

// What:     `fun measureNextBatch(...)` chooses the next tracks to read, starting at one position of the page and
//           wrapping around to its start, skipping those already attempted.
// Why:      A pass reads each track at most once, so a track that never gets a peak cannot loop the refresh forever.
//           Starting at the first visible track gives the rows the user is looking at their measures first, on a page
//           of thousands of tracks where reading from the top made a row deep in the page wait for every row above it.
//
// In TS you'd write (pseudocode):
// ```ts
// function nextBatch(
//   page: Track[], known: Map<string, TrackMeasure>, attempted: Set<string>, size: number, startAt = 0,
// ): Track[] {
//   const batch: Track[] = [];
//   for (let offset = 0; offset < page.length && batch.length < size; offset++) {
//     const track = page[(startAt + offset) % page.length];
//     if (!attempted.has(track.uri) && needsRead(known.get(track.uri))) batch.push(track);
//   }
//   return batch;
// }
// ```
/**
 * Returns up to [batchSize] tracks of [page] that need a read and were not attempted yet, in page order from [startAt]
 * to the end of the page and then from its start up to [startAt].
 *
 * @param startAt position in [page] where the scan begins, clamped into the page; a negative value or one past the
 *     end begins at the start of the page
 * @throws IllegalArgumentException when [batchSize] is below one
 */
fun measureNextBatch(
    page: List<Track>,
    known: Map<String, TrackMeasure>,
    attempted: Set<String>,
    batchSize: Int,
    startAt: Int = 0,
): List<Track> {
    require(batchSize >= 1) { "Measure batch size must be at least 1, was $batchSize." }
    /** Position where the scan begins, or the start of the page when [startAt] is outside it. */
    val start: Int = if (startAt in page.indices) startAt else 0
    /** Tracks chosen so far, in scan order. */
    val batch: MutableList<Track> = mutableListOf()
    /** Count of positions scanned since [start]. */
    var offset: Int = 0
    while (offset < page.size && batch.size < batchSize) {
        /** Track at this step of the scan, wrapping from the end of the page to its start. */
        val track: Track = page[(start + offset) % page.size]
        if (track.uri !in attempted && measureNeedsRead(known[track.uri])) {
            batch += track
        }
        offset += 1
    }
    return batch
}

// What:     `fun measuresByIndex(...)` keys the page's measures by load-order index.
// Why:      The rows name their tracks by index, and the URI lookup happens once here.
//
// In TS you'd write (pseudocode):
// ```ts
// function measuresByIndex(indices: number[], uriAt: (i: number) => string | null, measures: Map<string, TrackMeasure>)
//   { ... }
// ```
/**
 * Maps each page index to its measure, skipping indices with no URI or no measure yet.
 */
fun measuresByIndex(
    pageIndices: List<Int>,
    uriAt: (Int) -> String?,
    measures: Map<String, TrackMeasure>,
): Map<Int, TrackMeasure> {
    return pageIndices.mapNotNull { index ->
        /** URI of the track at this load-order index, or null when the index has no track. */
        val uri: String = uriAt(index) ?: return@mapNotNull null
        /** Measure of the track, or null when it has not been read yet. */
        val measure: TrackMeasure = measures[uri] ?: return@mapNotNull null
        index to measure
    }.toMap()
}
