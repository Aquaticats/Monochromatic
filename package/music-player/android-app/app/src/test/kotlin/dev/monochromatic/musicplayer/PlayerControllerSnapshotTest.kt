package dev.monochromatic.musicplayer

import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.Session
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test

/** What the media session sees of the queue through the controller, and what stays shared between refreshes. */
class PlayerControllerSnapshotTest {
    /** Builds a library of [count] root-level tracks. */
    private fun tracks(count: Int): List<Track> =
        (0 until count).map { index -> Track(uri = "u$index", displayPath = "t$index.mp3") }

    /** Builds a controller with [mode] chosen and track 0 playing. */
    private fun playing(count: Int, mode: PlaybackMode): PlayerController {
        val controller = PlayerController(FakeAudioEngine())
        controller.applySettings(Session(playbackMode = mode))
        controller.beginLoad()
        controller.openLibrary(tracks(count))
        controller.playIndex(0)
        return controller
    }

    @Test
    fun nextInOrderMovesTheCursorAndSharesEveryRowWithThePreviousSnapshot() {
        val controller = playing(50, PlaybackMode.IN_ORDER)
        val before = controller.snapshot()
        controller.next()
        val after = controller.snapshot()
        assertEquals(before.currentIndex?.plus(1), after.currentIndex)
        assertSame("a track change must not rebuild the queue rows", before.items, after.items)
    }

    @Test
    fun aLibraryChangeGivesNewRows() {
        val controller = playing(5, PlaybackMode.IN_ORDER)
        val before = controller.snapshot()
        controller.openLibrary(
            (0 until 5).map { index -> Track(uri = "v$index", displayPath = "s$index.mp3") },
        )
        val after = controller.snapshot()
        assertNotSame(before.items, after.items)
        assertTrue(after.items.all { item -> item.uri.startsWith("v") })
    }

    @Test
    fun anArtistTagReachesTheRowsOfThatTrackOnly() {
        val controller = playing(4, PlaybackMode.IN_ORDER)
        val before = controller.snapshot()
        controller.setTrackArtist("u2", "Ada")
        val after = controller.snapshot()
        assertEquals(listOf(null, null, "Ada", null), after.items.map { item -> item.artist })
        assertSame(before.items[0], after.items[0])
        assertSame(before.items[3], after.items[3])
    }

    @Test
    fun theThirtyThirdTagEvictsTheFirstFromTheRows() {
        val controller = playing(40, PlaybackMode.IN_ORDER)
        for (index in 0 until 33) {
            controller.setTrackArtist("u$index", "Artist $index")
            controller.snapshot()
        }
        val items = controller.snapshot().items
        assertEquals(null, items[0].artist)
        assertEquals("Artist 1", items[1].artist)
        assertEquals("Artist 32", items[32].artist)
    }

    @Test
    fun aShuffleHistoryKeepsDistinctEntryIdsAcrossSnapshots() {
        val controller = playing(2, PlaybackMode.SHUFFLE_PAGE)
        repeat(6) {
            controller.next()
            val ids = controller.snapshot().items.map { item -> item.entryId }
            assertEquals(ids.size, ids.toSet().size)
        }
    }
}
