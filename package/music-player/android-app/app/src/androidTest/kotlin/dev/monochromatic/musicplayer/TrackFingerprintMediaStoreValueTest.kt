// Instrumented check that the fingerprint of a media store row keeps the value it had when every row asked the media
// provider for the document column `last_modified` and read the unknown time from the refusal.
//
// What:     `package dev.monochromatic.musicplayer` places the check beside `TrackFingerprint`.
// Why:      The check reads the fingerprint through its public entry point and the native helper it ends in.
//
// In TS you'd write (pseudocode):
// ```ts
// // The file path is the module.
// ```
package dev.monochromatic.musicplayer

import android.content.ContentUris
import android.content.Context
import android.net.Uri
import android.provider.MediaStore
import android.provider.OpenableColumns
import android.util.Log
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assume.assumeTrue
import org.junit.Test

/** Count of media store rows whose fingerprint the check compares. */
private const val ROWS_CHECKED: Int = 25

/** Logcat tag of the lines that list each row's fingerprint, so two builds can be compared line by line. */
private const val CONTINUITY_TAG: String = "FingerprintContinuity"

/**
 * A media store row has no modified-time column of the document kind, so its fingerprint is the native hash of the
 * URI, the byte size and a zero time. The value must not change when the app stops asking the provider for the column.
 */
class TrackFingerprintMediaStoreValueTest {
    /** Context of the app under test. */
    private val context: Context = InstrumentationRegistry.getInstrumentation().targetContext

    /** Up to [ROWS_CHECKED] audio rows of the device's media store, as URIs. */
    private fun audioUris(): List<Uri> {
        /** Audio collection of every volume. */
        val collection: Uri = MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL)
        /** URIs read so far. */
        val uris: MutableList<Uri> = mutableListOf()
        context.contentResolver.query(
            collection,
            arrayOf(MediaStore.Audio.Media._ID),
            "${MediaStore.Audio.Media.IS_MUSIC} != 0",
            null,
            "${MediaStore.Audio.Media._ID} ASC",
        )?.use { cursor ->
            while (cursor.moveToNext() && uris.size < ROWS_CHECKED) {
                uris.add(ContentUris.withAppendedId(collection, cursor.getLong(0)))
            }
        }
        return uris
    }

    /** Byte size the provider reports for [uri], or null when it reports none. */
    private fun sizeOf(uri: Uri): Long? =
        context.contentResolver.query(uri, arrayOf(OpenableColumns.SIZE), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst() && !cursor.isNull(0)) cursor.getLong(0) else null
        }

    /** Every media store row fingerprints as the native hash of its URI, size and a zero modified time. */
    @Test
    fun aMediaStoreRowFingerprintsWithAZeroModifiedTime() {
        /** Rows to check. */
        val uris: List<Uri> = audioUris()
        assumeTrue("no indexed media store audio (grant READ_MEDIA_AUDIO)", uris.isNotEmpty())
        for (uri in uris) {
            /** Size of the row's file, which the fingerprint also folds in. */
            val size: Long = requireNotNull(sizeOf(uri)) { "the media store reports no size for $uri" }
            /** Fingerprint the app computes. */
            val actual: Long? = runBlocking { TrackFingerprint.of(context, uri) }
            Log.i(CONTINUITY_TAG, "uri=$uri size=$size fingerprint=$actual")
            assertEquals("fingerprint of $uri", NativeBridge.nativeFingerprint(uri.toString(), size, 0L), actual)
        }
    }
}
