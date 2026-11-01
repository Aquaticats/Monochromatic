package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.LoadFailure
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** M5 and M6 wording: File details shows name, size and folder; a failed open says what happened. */
class TrackDetailsMessagesTest {
    @Test
    fun sizesUseTheDesktopUnitsAndOneDecimal() {
        assertEquals("0.5 KB", humanFileSize(512))
        assertEquals("1.5 KB", humanFileSize(1536))
        assertEquals("2.0 MB", humanFileSize(2L * 1024 * 1024))
        assertEquals("1.0 GB", humanFileSize(1024L * 1024 * 1024))
    }

    @Test
    fun theFolderKeepsTheLastTwoNamesBehindAnEllipsis() {
        assertEquals("", shortFolderOf("song.mp3"))
        assertEquals("Artist", shortFolderOf("Artist/song.mp3"))
        assertEquals("Artist/Album", shortFolderOf("Artist/Album/song.mp3"))
        assertEquals("…/Album/Disc 1", shortFolderOf("Artist/Album/Disc 1/song.mp3"))
    }

    @Test
    fun detailsNameTheFileTheSizeAndTheFolderAsANotice() {
        val message = trackDetailsMessage("song.mp3", 3L * 1024 * 1024, "Artist/Album/song.mp3")
        assertEquals("song.mp3\n3.0 MB · Artist/Album", message.text)
        assertEquals(FeedbackKind.NOTICE, message.kind)
        assertEquals(TRACK_DETAILS_ID, message.id)
    }

    @Test
    fun detailsSaySizeUnknownAndLibraryRootWhenTheyAreMissing() {
        val message = trackDetailsMessage("song.mp3", null, "song.mp3")
        assertEquals("song.mp3\nsize unknown · library root", message.text)
    }

    @Test
    fun anUnreadableTrackAndAStalledProviderGetDifferentAdvice() {
        val unreadable = trackLoadFailureMessage(LoadFailure.UNREADABLE, "song.mp3")
        val stalled = trackLoadFailureMessage(LoadFailure.TIMED_OUT, "song.mp3")
        assertTrue(unreadable.text.contains("song.mp3"))
        assertTrue(stalled.text.contains("Try again"))
        assertEquals(FeedbackKind.ERROR, stalled.kind)
        assertEquals(TRACK_LOAD_ERROR_ID, stalled.id)
    }
}
