package dev.monochromatic.musicplayer

import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.Session
import dev.monochromatic.musicplayer.core.TrackMeasure
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** M3 and S1 through the controller: what the engine and the media timeline see at the end of a track. */
class PlayerControllerPlaybackEndTest {
    /** Builds a library of [count] root-level tracks. */
    private fun tracks(count: Int): List<Track> =
        (0 until count).map { index -> Track(uri = "u$index", displayPath = "t$index.mp3") }

    /** Builds a controller on a fake engine with [mode] chosen and track 0 playing. */
    private fun playing(count: Int, mode: PlaybackMode, engine: FakeAudioEngine): PlayerController {
        val controller = PlayerController(engine)
        controller.applySettings(Session(playbackMode = mode))
        controller.beginLoad()
        controller.openLibrary(tracks(count))
        controller.playIndex(0)
        return controller
    }

    @Test
    fun inOrderStopsAtTheEndOfTheFolderAndReadiesTheFirstTrackPaused() {
        val engine = FakeAudioEngine()
        val controller = playing(2, PlaybackMode.IN_ORDER, engine)
        controller.handleNaturalEnd()
        assertEquals("u1", engine.loadedUri)
        assertTrue(engine.playWhenReadyValue)
        controller.handleNaturalEnd()
        assertEquals("u0", engine.loadedUri)
        assertFalse("In order must stop at the end of the folder", engine.playWhenReadyValue)
        assertEquals(0, controller.uiState.currentIndex)
    }

    @Test
    fun theStopAtTheEndOfTheFolderReportsTheReadiedFirstTrackAsLoaded() {
        val engine = FakeAudioEngine()
        val controller = playing(2, PlaybackMode.IN_ORDER, engine)
        /** URIs the controller reported as loaded, in report order. */
        val reported: MutableList<String> = mutableListOf()
        controller.onTrackLoaded = { uri -> reported += uri }
        controller.handleNaturalEnd()
        assertEquals("the natural advance reports the next track", listOf("u1"), reported)
        controller.handleNaturalEnd()
        assertEquals(
            "the stop reports the readied first track, so its artist tag is read",
            listOf("u1", "u0"),
            reported,
        )
    }

    @Test
    fun playAfterTheStopStartsTheFolderAgainFromItsFirstTrack() {
        val engine = FakeAudioEngine()
        val controller = playing(1, PlaybackMode.IN_ORDER, engine)
        val loadsBefore = engine.loadCount
        controller.handleNaturalEnd()
        assertFalse(engine.playWhenReadyValue)
        controller.togglePlay()
        assertTrue(engine.playWhenReadyValue)
        assertEquals("u0", engine.loadedUri)
        assertTrue(engine.loadCount > loadsBefore)
    }

    @Test
    fun aSingleTrackFolderInShuffleKeepsPlayingWithDistinctTimelineIds() {
        val engine = FakeAudioEngine()
        val controller = playing(1, PlaybackMode.SHUFFLE_PAGE, engine)
        repeat(4) { controller.handleNaturalEnd() }
        assertTrue(engine.playWhenReadyValue)
        val ids = controller.snapshot().items.map { item -> item.entryId }
        assertEquals(5, ids.size)
        assertEquals(5, ids.toSet().size)
    }

    @Test
    fun aTwoTrackFolderInShuffleSurvivesItsWrapWithDistinctTimelineIds() {
        val engine = FakeAudioEngine()
        val controller = playing(2, PlaybackMode.SHUFFLE_PAGE, engine)
        repeat(7) { controller.handleNaturalEnd() }
        val ids = controller.snapshot().items.map { item -> item.entryId }
        assertEquals(ids.size, ids.toSet().size)
        assertEquals(8, ids.size)
    }

    @Test
    fun repeatReplaysTheSameTrackWithoutGrowingTheTimeline() {
        val engine = FakeAudioEngine()
        val controller = playing(2, PlaybackMode.REPEAT, engine)
        repeat(3) { controller.handleNaturalEnd() }
        assertEquals("u0", engine.loadedUri)
        assertTrue(engine.playWhenReadyValue)
        val items = controller.snapshot().items
        assertEquals("Repeat never grows the timeline", 2, items.size)
        assertEquals(2, items.map { item -> item.entryId }.toSet().size)
        assertEquals(0, controller.snapshot().currentIndex)
    }

    @Test
    fun droppedMeasuresLeaveTheOtherTracksMeasuresAlone() {
        val controller = PlayerController(FakeAudioEngine())
        controller.mergeMeasures(
            mapOf(
                "u0" to TrackMeasure(durationSec = 9.0, peakDbtp = -21.1, peakExact = true),
                "u1" to TrackMeasure(durationSec = 11.0, peakDbtp = -27.1, peakExact = true),
            ),
        )
        controller.dropMeasures(setOf("u0"))
        assertEquals(setOf("u1"), controller.uiState.measures.keys)
    }
}
