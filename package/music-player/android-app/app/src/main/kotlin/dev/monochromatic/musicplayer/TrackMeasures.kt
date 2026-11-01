// ============================================================================
// File summary:
//
// This file reads the per-track duration and true peak that the library rows and the track menu show.
// The duration comes from the system library's MediaStore row when the track is in the library, and
// from the file itself through `MediaMetadataRetriever` otherwise. A file's duration is cached by its
// fingerprint in a small text file, so a rescan does not open every document again. The peak comes from
// the native decision cache, read without measuring, so reading a page never decodes audio.
//
// A re-analysis clears one track's cached decision and measures that track again through the same
// sweep coordinator the library-wide sweep uses, with one item.
// ============================================================================

// What:     `package dev.monochromatic.musicplayer` puts the provider beside the sweep and the native bridge it reads.
// Why:      It calls `TrackFingerprint`, `NativeBridge`, `TruePeakGain`, and the sweep helpers without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword; the directory is the module.
// ```
package dev.monochromatic.musicplayer

// What:     Android imports for the context, the metadata retriever, the URI, the MediaStore columns, and the log.
// Why:      Each one names a platform type this file reads a duration or a peak through.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Context } from "android/content";
// ```
import android.content.Context
import android.media.MediaMetadataRetriever
import android.net.Uri
import android.provider.MediaStore
import android.util.Log
import androidx.core.net.toUri

// What:     Pure helpers from the core package for the batch choice, the peak reply, and the duration cache file.
// Why:      The decisions are unit-tested without Android, and this file only performs the reads and the writes.
//
// In TS you'd write (pseudocode):
// ```ts
// import { nextBatch, peakFromReply, encodeDurations, decodeDurations } from "./core/TrackMeasure";
// ```
import dev.monochromatic.musicplayer.core.MEASURE_DURATION_FILE_NAME
import dev.monochromatic.musicplayer.core.MeasurePeakReading
import dev.monochromatic.musicplayer.core.TrackMeasure
import dev.monochromatic.musicplayer.core.measureDecodeDurations
import dev.monochromatic.musicplayer.core.measureDeleteLegacyDurationFiles
import dev.monochromatic.musicplayer.core.measureEncodeDurations
import dev.monochromatic.musicplayer.core.measurePeakFromReply

// What:     `import java.io.File` and `IOException` name the file and the failure it raises.
// Why:      The duration cache is one file, and a failed write is an expected, logged outcome.
//
// In TS you'd write (pseudocode):
// ```ts
// import { readFile, writeFile } from "fs/promises";
// ```
import java.io.File
import java.io.IOException

// What:     Coroutine imports for the dispatcher and the context switch.
// Why:      The reads block on the native cache and the file system, so they run on the IO pool.
//
// In TS you'd write (pseudocode):
// ```ts
// import { withContext, Dispatchers } from "kotlinx/coroutines";
// ```
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

// What:     `private const val MEASURE_TAG: String = "TrackMeasures"` is the logcat tag of this file's lines.
// Why:      One tag lets a run be followed in logcat without the sweep's own lines.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_TAG = "TrackMeasures";
// ```
/** Logcat tag of the track measure log lines, which the page refresh in the host shares. */
internal const val MEASURE_TAG: String = "TrackMeasures"

// What:     `private const val MEASURE_MS_PER_SEC: Double = 1000.0` converts a stored duration to seconds.
// Why:      The retriever and MediaStore report milliseconds, and the template formats seconds.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEASURE_MS_PER_SEC = 1000;
// ```
/** Milliseconds in one second, to turn a stored duration into seconds. */
private const val MEASURE_MS_PER_SEC: Double = 1000.0

// What:     `object TrackMeasures { ... }` is the process-wide provider of the track measures.
// Why:      One owner holds the duration cache in memory and writes it to one file, so two readers never race on it.
//
// In TS you'd write (pseudocode):
// ```ts
// const TrackMeasures = { readMeasures };
// ```
/**
 * Reads the duration and true peak of library tracks, caching durations by fingerprint and never decoding for a peak.
 */
object TrackMeasures {
    // What:     `private val durationLock: Any = Any()` serializes the in-memory duration cache and its file writes.
    // Why:     The batch reader and a re-analysis may both record a duration, and the file must not be written twice at
    //           once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const durationLock = {};
    // ```
    /** Guards the in-memory duration cache and its dirty flag. */
    private val durationLock: Any = Any()

    // What:     `private var durations: MutableMap<Long, Long>?` holds the durations keyed by fingerprint, null until
    //           loaded.
    // Why:      The file is read once, on the first duration lookup, and then served from memory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let durations: Map<bigint, bigint> | null = null;
    // ```
    /** Durations read so far keyed by fingerprint, or null before the cache file is loaded. */
    private var durations: MutableMap<Long, Long>? = null

    // What:     `private var durationsDirty: Boolean = false` records whether the in-memory cache has entries the file
    //           lacks.
    // Why:      A batch that found no new durations must not rewrite the whole file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let durationsDirty = false;
    // ```
    /** True when the in-memory durations hold entries that the cache file does not yet record. */
    private var durationsDirty: Boolean = false

    // What:     `suspend fun readMeasures(context, tracks)` reads the measures of the given tracks on the IO pool.
    // Why:      The rows need the duration and the peak of every track they show, and this is the one read path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function readMeasures(tracks: Track[]): Promise<Map<string, TrackMeasure>> { ... }
    // ```
    /**
     * Reads the measure of each given track, keyed by its URI, and saves any newly read durations.
     */
    suspend fun readMeasures(context: Context, tracks: List<Track>): Map<String, TrackMeasure> =
        withContext(Dispatchers.IO) {
            Log.i(MEASURE_TAG, "track measures read count=${tracks.size}")
            /** Measure of each track keyed by its URI. */
            val measured: Map<String, TrackMeasure> = tracks.associate { track ->
                track.uri to measureOf(context, track)
            }
            saveDurations(context)
            return@withContext measured
        }

    // What:     `private suspend fun measureOf(context, track): TrackMeasure` reads one track's duration and peak.
    // Why:      The duration and the peak are read by different paths, and each one may be absent without the other.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function measureOf(track: Track): Promise<TrackMeasure> { ... }
    // ```
    /** Reads one track's duration and its peak from the decision cache, with either one left null when unknown. */
    private suspend fun measureOf(context: Context, track: Track): TrackMeasure {
        /** URI of the track, parsed from its stored string. */
        val uri: Uri = track.uri.toUri()
        /** Cache key of the track, or null when its size cannot be read. */
        val fingerprint: Long? = TrackFingerprint.of(context, uri)
        /** Duration in milliseconds, or null when the file's length is not known. */
        val durationMs: Long? = durationMsOf(context, uri, fingerprint)
        /** Peak read from the decision cache, or null when no decision is cached for the track. */
        val peak: MeasurePeakReading? = fingerprint?.let { key -> peakOf(context, key) }
        return TrackMeasure(
            durationSec = durationMs?.let { ms -> ms / MEASURE_MS_PER_SEC },
            peakDbtp = peak?.dbtp,
            peakExact = peak?.exact ?: false,
        )
    }

    // What:     `private fun durationMsOf(...)` reads a track's duration from the library or the file.
    // Why:      A library item has its length in MediaStore already, while a document needs its file opened once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function durationMsOf(uri: string, fingerprint: bigint | null): number | null { ... }
    // ```
    /** Duration of the track in milliseconds: from the library row, else the cache, else the file. */
    private fun durationMsOf(context: Context, uri: Uri, fingerprint: Long?): Long? {
        if (uri.authority == MediaStore.AUTHORITY) {
            return mediaStoreDurationMs(context, uri)
        }
        if (fingerprint != null) {
            /** Duration already cached for this fingerprint, or null when none is stored. */
            val cached: Long? = synchronized(durationLock) { loadDurations(context)[fingerprint] }
            if (cached != null) {
                return cached
            }
        }
        /** Duration read from the file, or null when the file's metadata cannot be read. */
        val measured: Long = retrieverDurationMs(context, uri) ?: return null
        if (fingerprint != null) {
            rememberDuration(context, fingerprint, measured)
        }
        return measured
    }

    // What:     `private fun mediaStoreDurationMs(...)` reads a library item's length from its MediaStore row.
    // Why:      The system library already stores the length, so no file is opened for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function mediaStoreDurationMs(uri: string): number | null { ... }
    // ```
    /** Duration of a library item in milliseconds, or null when its row has no length. */
    private fun mediaStoreDurationMs(context: Context, uri: Uri): Long? {
        try {
            return context.contentResolver
                .query(uri, arrayOf(MediaStore.Audio.Media.DURATION), null, null, null)
                ?.use { cursor ->
                    if (cursor.moveToFirst() && !cursor.isNull(0)) {
                        return@use cursor.getLong(0)
                    }
                    return@use null
                }
        } catch (expectedFailure: SecurityException) {
            Log.w(MEASURE_TAG, "library duration query was refused", expectedFailure)
            return null
        }
    }

    // What:     `private fun retrieverDurationMs(...)` reads a file's length through the platform metadata retriever.
    // Why:      A document in a picked folder has no library row, so its length must come from the file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function retrieverDurationMs(uri: string): number | null { ... }
    // ```
    /** Duration of a file in milliseconds, or null when the retriever cannot read it. */
    private fun retrieverDurationMs(context: Context, uri: Uri): Long? {
        /** Metadata retriever for this one file, released before the function returns. */
        val retriever = MediaMetadataRetriever()
        try {
            retriever.setDataSource(context, uri)
            return retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION)?.toLongOrNull()
        } catch (expectedFailure: Exception) {
            Log.w(MEASURE_TAG, "track duration could not be read from the file", expectedFailure)
            return null
        } finally {
            retriever.release()
        }
    }

    // What:     `private fun peakOf(context, fingerprint): MeasurePeakReading?` reads a cached peak without measuring.
    // Why:      Reading a page must never decode audio, so only the native peek runs here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function peakOf(fingerprint: bigint): MeasurePeakReading | null { ... }
    // ```
    /** Peak of one fingerprint read from the decision cache, or null when no decision is cached. */
    private fun peakOf(context: Context, fingerprint: Long): MeasurePeakReading? {
        /** Native reply of the peek: the peak in dBTP and the kind code, or null when no decision is cached. */
        val reply: FloatArray? = NativeBridge.nativePeekPeak(TruePeakGain.handle(context), fingerprint)
        return measurePeakFromReply(reply)
    }

    // What:     `private fun rememberDuration(...)` records one fingerprint's duration in memory and marks it dirty.
    // Why:      The file is written once per batch, not once per track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rememberDuration(fingerprint: bigint, durationMs: bigint): void { durations.set(fingerprint, durationMs); dirty =
    //   true; }
    // ```
    /** Stores one duration in memory and marks the cache as needing a write. */
    private fun rememberDuration(context: Context, fingerprint: Long, durationMs: Long) {
        synchronized(durationLock) {
            loadDurations(context)[fingerprint] = durationMs
            durationsDirty = true
        }
    }

    // What:     `private fun loadDurations(context): MutableMap<Long, Long>` returns the in-memory durations, loading
    //           the file once.
    // Why:      The lookups and the writes share one map, so the file is read at most once per process.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function loadDurations(): Map<bigint, bigint> { return durations ??= readFile(); }
    // ```
    /** Returns the in-memory durations, loading the cache file on first use. Callers hold [durationLock]. */
    private fun loadDurations(context: Context): MutableMap<Long, Long> {
        /** Durations already in memory, or null before the first load. */
        val current: MutableMap<Long, Long>? = durations
        if (current != null) {
            return current
        }
        /** Retired duration cache files (gxhash-era keys) deleted now, so they do not accumulate. */
        val deletedLegacy: Int = measureDeleteLegacyDurationFiles(context.applicationContext.filesDir)
        if (deletedLegacy > 0) {
            Log.i(MEASURE_TAG, "deleted $deletedLegacy retired duration cache file(s)")
        }
        /** Cache file under the app's private files directory. */
        val file = File(context.applicationContext.filesDir, MEASURE_DURATION_FILE_NAME)
        /** Durations decoded from the file, which is empty when the file is absent or unreadable. */
        val loaded: MutableMap<Long, Long> = readDurationFile(file).toMutableMap()
        durations = loaded
        return loaded
    }

    // What:     `private fun readDurationFile(file: File): Map<Long, Long>` decodes the cache file.
    // Why:      A missing or unreadable file starts the cache empty, which only costs a re-read of the durations.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function readDurationFile(path: string): Map<bigint, bigint> { ... }
    // ```
    /** Decodes the duration cache file, or returns no entries when the file is absent or cannot be read. */
    private fun readDurationFile(file: File): Map<Long, Long> {
        if (!file.exists()) {
            return emptyMap()
        }
        try {
            return measureDecodeDurations(file.readText())
        } catch (expectedFailure: IOException) {
            Log.w(MEASURE_TAG, "duration cache could not be read, starting empty", expectedFailure)
            return emptyMap()
        }
    }

    // What:     `private fun saveDurations(context)` writes the durations to a temporary file, then renames it over the
    //           cache.
    // Why:      A rename replaces the cache in one step, so a crash mid-write leaves the last complete file in place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function saveDurations(): Promise<void> { await writeFile(tmp, encode()); await rename(tmp, path); }
    // ```
    /** Writes the in-memory durations to the cache file when they changed since the last write. */
    private fun saveDurations(context: Context) {
        synchronized(durationLock) {
            /** Durations to write, or null when nothing has been loaded yet. */
            val current: MutableMap<Long, Long> = durations ?: return
            if (!durationsDirty) {
                return
            }
            /** Cache file under the app's private files directory. */
            val file = File(context.applicationContext.filesDir, MEASURE_DURATION_FILE_NAME)
            /** Temporary sibling the new contents are written to before the rename. */
            val temp = File(context.applicationContext.filesDir, "$MEASURE_DURATION_FILE_NAME.tmp")
            try {
                temp.writeText(measureEncodeDurations(current))
                if (temp.renameTo(file)) {
                    durationsDirty = false
                } else {
                    Log.w(MEASURE_TAG, "duration cache rename failed, will retry on the next batch")
                }
            } catch (expectedFailure: IOException) {
                Log.w(MEASURE_TAG, "duration cache could not be written", expectedFailure)
            }
        }
    }
}
