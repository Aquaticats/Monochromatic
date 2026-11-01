package dev.monochromatic.musicplayer

import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.Session
import dev.monochromatic.musicplayer.core.UnplayableReport
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/**
 * Pins what the player does when a track it moved to by itself cannot be opened (D122, D123, D128): the natural end,
 * Next, Previous, a Play with nothing selected, the readied first track and the restore walk past the file and report
 * it for a system notification, never as the in-app message; a tapped track keeps the in-app message. The controller
 * logs a failure through `android.util.Log`, which a plain JVM test cannot call, so these cases run under Robolectric.
 */
@RunWith(RobolectricTestRunner::class)
class PlayerControllerUnplayableWalkTest {
    /** Engine double that fails the load of every URI in its unreadable set, inside the load call. */
    private val engine = ReportingAudioEngine()

    /** Controller under test, over [engine]. */
    private val controller = PlayerController(engine)

    /** Every report the controller handed to the host, in order. */
    private val reports: MutableList<UnplayableReport> = mutableListOf()

    /** Opens [count] invented tracks in one folder and records the reports. */
    private fun open(count: Int, mode: PlaybackMode = PlaybackMode.IN_ORDER) {
        controller.onUnplayableFiles = { report -> reports += report }
        controller.applySettings(Session(playbackMode = mode))
        controller.openLibrary(
            (0 until count).map { number ->
                Track(uri = "content://invented/alpha/$number", displayPath = "Alpha/Track $number.mp3")
            },
        )
    }

    /** Builds [count] root-level tracks named `u0`, `u1` and so on. */
    private fun shortTracks(count: Int): List<Track> =
        (0 until count).map { number -> Track(uri = "u$number", displayPath = "t$number.mp3") }

    /** URI of the invented track at [index]. */
    private fun uriAt(index: Int): String = controller.trackAt(index)?.uri ?: error("No track at $index")

    /** The natural end onto an unreadable track plays the next one and reports the file, with no in-app message. */
    @Test
    fun aNaturalEndOntoAnUnreadableTrackPlaysTheNextOne() {
        open(4)
        controller.playIndex(0)
        engine.unreadable += uriAt(1)
        controller.handleNaturalEnd()
        assertEquals(uriAt(2) to true, engine.loads.last())
        assertEquals(2, controller.uiState.currentIndex)
        assertTrue(controller.uiState.playing)
        assertNull("a walk is never an in-app message", controller.loadFailure)
        assertEquals(listOf(UnplayableReport(listOf("Track 1.mp3"), timedOut = false)), reports)
    }

    /** Two unreadable tracks in a row are reported together, the later report holding both names. */
    @Test
    fun twoUnreadableTracksInARowAreReportedTogether() {
        open(4)
        controller.playIndex(0)
        engine.unreadable += listOf(uriAt(1), uriAt(2))
        controller.handleNaturalEnd()
        assertEquals(uriAt(3) to true, engine.loads.last())
        assertNull(controller.loadFailure)
        assertEquals(listOf("Track 1.mp3", "Track 2.mp3"), reports.last().fileNames)
    }

    /** When no track of the queue opens, the walk stops after one try each, paused, with a single growing report. */
    @Test
    fun aQueueOfUnreadableTracksStopsAfterOneTryEach() {
        open(3)
        controller.playIndex(0)
        engine.unreadable += listOf(uriAt(0), uriAt(1), uriAt(2))
        controller.handleNaturalEnd()
        assertEquals("the first load plus one try for each of the three tracks", 4, engine.loads.size)
        assertFalse(controller.uiState.playing)
        assertNull(controller.loadFailure)
        assertEquals(listOf("Track 1.mp3", "Track 2.mp3", "Track 0.mp3"), reports.last().fileNames)
    }

    /** The end of an In order folder with an unreadable first track readies the next playable track, paused. */
    @Test
    fun theReadiedFirstTrackOfAFolderIsWalkedPastPaused() {
        open(3)
        controller.playIndex(2)
        engine.unreadable += uriAt(0)
        controller.handleNaturalEnd()
        assertEquals("the readied track must not start playing", uriAt(1) to false, engine.loads.last())
        assertEquals(1, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertNull(controller.loadFailure)
        assertEquals(listOf("Track 0.mp3"), reports.last().fileNames)
    }

    /** An unreadable last track of an In order folder stops the folder instead of wrapping to its start playing. */
    @Test
    fun anUnreadableLastTrackOfAnInOrderFolderEndsTheFolder() {
        open(3)
        controller.playIndex(1)
        engine.unreadable += uriAt(2)
        controller.handleNaturalEnd()
        assertEquals("the folder is readied from its first track, paused", uriAt(0) to false, engine.loads.last())
        assertFalse(controller.uiState.playing)
        assertEquals(listOf("Track 2.mp3"), reports.last().fileNames)
    }

    /** Next skips an unreadable track and keeps playing. */
    @Test
    fun nextWalksPastAnUnreadableTrack() {
        open(4)
        controller.playIndex(0)
        engine.unreadable += uriAt(1)
        controller.next()
        assertEquals(uriAt(2) to true, engine.loads.last())
        assertNull(controller.loadFailure)
        assertEquals(listOf("Track 1.mp3"), reports.last().fileNames)
    }

    /** Previous walks backward past an unreadable track, not forward. */
    @Test
    fun previousWalksBackwardPastAnUnreadableTrack() {
        open(4)
        controller.playIndex(3)
        engine.unreadable += uriAt(2)
        controller.prev()
        assertEquals(uriAt(1) to true, engine.loads.last())
        assertNull(controller.loadFailure)
        assertEquals(listOf("Track 2.mp3"), reports.last().fileNames)
    }

    /** A repeat-one natural end meets the same unreadable track again, so the walk steps on to the next track. */
    @Test
    fun aRepeatedUnreadableTrackIsWalkedPast() {
        open(3, PlaybackMode.REPEAT)
        controller.playIndex(1)
        engine.unreadable += uriAt(1)
        controller.handleNaturalEnd()
        assertEquals(uriAt(2) to true, engine.loads.last())
        assertEquals(listOf("Track 1.mp3"), reports.last().fileNames)
    }

    /** A Play with nothing selected starts at the first track of the page and walks past it when it cannot open. */
    @Test
    fun playWithNothingSelectedWalksPastAnUnreadableFirstTrack() {
        open(3)
        engine.unreadable += uriAt(0)
        controller.togglePlay()
        assertEquals(uriAt(1) to true, engine.loads.last())
        assertNull(controller.loadFailure)
        assertEquals(listOf("Track 0.mp3"), reports.last().fileNames)
    }

    /** The restore of the last session with an unreadable saved track readies the next one, paused, and says so. */
    @Test
    fun theRestoreWalksPastAnUnreadableSavedTrack() {
        controller.onUnplayableFiles = { report -> reports += report }
        engine.unreadable += "content://invented/alpha/1"
        controller.finishLoad(
            tracks = (0 until 3).map { number ->
                Track(uri = "content://invented/alpha/$number", displayPath = "Alpha/Track $number.mp3")
            },
            session = Session(selected = "content://invented/alpha/1", positionSecs = 42.0),
        )
        assertEquals(uriAt(2) to false, engine.loads.last())
        assertEquals(2, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertNull(controller.loadFailure)
        assertEquals(listOf("Track 1.mp3"), reports.last().fileNames)
    }

    /** A tapped track that cannot be opened keeps the in-app message, stays selected and is not reported. */
    @Test
    fun aTappedUnreadableTrackKeepsTheInAppMessage() {
        open(3)
        engine.unreadable += uriAt(1)
        controller.activateRow(1, autoPlayOnSelect = true)
        assertNotNull(controller.loadFailure)
        assertEquals(1, controller.uiState.currentIndex)
        assertEquals(1, engine.loads.size)
        assertTrue(reports.isEmpty())
    }

    /** After a walk, a tap on an unreadable track is a tap again, not a continuation of the walk. */
    @Test
    fun aTapAfterAWalkKeepsTheInAppMessage() {
        open(4)
        controller.playIndex(0)
        engine.unreadable += uriAt(1)
        controller.handleNaturalEnd()
        engine.unreadable += uriAt(3)
        controller.playIndex(3)
        assertNotNull(controller.loadFailure)
        assertEquals(uriAt(3), controller.loadFailure?.uri)
        assertEquals("the tap adds no file to the earlier report", listOf("Track 1.mp3"), reports.last().fileNames)
    }

    /** A failure of a track other than the one just loaded is a leftover, so it neither walks nor reports. */
    @Test
    fun aLeftoverFailureOfAnEarlierTrackIsIgnored() {
        /** Engine double that lets the test deliver a failure by hand. */
        val manual = FakeAudioEngine()
        /** Controller whose engine reports no failure by itself. */
        val manualController = PlayerController(manual)
        manualController.onUnplayableFiles = { report -> reports += report }
        manualController.openLibrary(shortTracks(3))
        manualController.playIndex(0)
        manualController.handleNaturalEnd()
        /** Loads made before the leftover arrives. */
        val loadsBefore: Int = manual.loadCount
        manual.failLoad("u0", LoadFailure.UNREADABLE)
        assertEquals(loadsBefore, manual.loadCount)
        assertNull(manualController.loadFailure)
        assertTrue(reports.isEmpty())
    }

    /** A provider that does not answer in time stops the walk, because the next file would stall the same way. */
    @Test
    fun aTimedOutOpenOnAWalkStopsItAndSaysSo() {
        /** Engine double that lets the test deliver a failure by hand. */
        val manual = FakeAudioEngine()
        /** Controller whose engine reports no failure by itself. */
        val manualController = PlayerController(manual)
        manualController.onUnplayableFiles = { report -> reports += report }
        manualController.openLibrary(shortTracks(3))
        manualController.playIndex(0)
        manualController.handleNaturalEnd()
        manual.failLoad("u1", LoadFailure.TIMED_OUT)
        assertEquals("no further load after a timeout", 2, manual.loadCount)
        assertFalse(manual.playWhenReadyValue)
        assertNull(manualController.loadFailure)
        assertEquals(listOf(UnplayableReport(listOf("t1.mp3"), timedOut = true)), reports)
    }
}
