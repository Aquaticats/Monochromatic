// The media notification shows an artist line under the title when the file carries an artist tag.

// What:     `package dev.monochromatic.musicplayer` places the tag reader beside the playback service that uses it.
// Why:      Reading a tag opens the file, so it lives in Android code and runs off the main thread.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.content.Context
import android.media.MediaMetadataRetriever
import android.net.Uri
import android.util.Log

// What:     `private const val TAGS_TAG: String = "TrackTags"` is the logcat tag of the tag reader.
// Why:      Every log line of one component carries the same tag.
//
// In TS you'd write (pseudocode):
// ```ts
// const TAGS_TAG = "TrackTags";
// ```
/** Logcat tag of the tag reader. */
private const val TAGS_TAG: String = "TrackTags"

// What:     `object TrackTags` reads tags from a track file.
// Why:      One entry point keeps the retriever's open and release in one place.
//
// In TS you'd write (pseudocode):
// ```ts
// export const TrackTags = { artistOf(context, uri) { /* retriever.extract(ARTIST) */ } };
// ```
/** Reads the tags that the media notification shows. */
object TrackTags {
    /**
     * Reads the artist of the track at [uri], preferring the track artist over the album artist.
     *
     * Blocks while the provider opens the file, so call it from a background thread.
     *
     * @param context any context of this app
     * @param uri content URI or absolute path of the track
     * @return the trimmed artist text, or null when the file has none or cannot be read
     */
    fun artistOf(context: Context, uri: String): String? {
        /** Reader of the container's metadata. */
        val retriever = MediaMetadataRetriever()
        return try {
            if (uri.startsWith("/")) {
                retriever.setDataSource(uri)
            } else {
                retriever.setDataSource(context, Uri.parse(uri))
            }
            /** The tag texts in the order of preference. */
            val candidates: List<String?> = listOf(
                retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_ARTIST),
                retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_ALBUMARTIST),
            )
            candidates.firstNotNullOfOrNull { text -> text?.trim()?.takeIf { trimmed -> trimmed.isNotEmpty() } }
        } catch (expectedFailure: RuntimeException) {
            Log.w(TAGS_TAG, "artist tag could not be read", expectedFailure)
            null
        } finally {
            retriever.release()
        }
    }
}
