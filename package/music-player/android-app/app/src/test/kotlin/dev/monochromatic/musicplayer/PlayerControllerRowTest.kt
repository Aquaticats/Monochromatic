package dev.monochromatic.musicplayer

import dev.monochromatic.musicplayer.core.Session
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Pins the rule for a tap or Enter on a track row (D124): the current row toggles play and pause, and any other row
 * is only selected, loaded paused, and stops what was playing. Every case reads the playing flag the deck and the
 * rows are drawn from, through an engine that reports playing changes the way the real one does. The cases where a
 * load fails live in `PlayerControllerRowFailureTest`, because the controller logs a failure through
 * `android.util.Log`.
 */
class PlayerControllerRowTest {
    /** Engine double that reports playing changes and load failures. */
    private val engine = ReportingAudioEngine()

    /** Controller under test, over [engine]. */
    private val controller = PlayerController(engine)

    /** Invented track of [folder] with the given number; its URI is `content://invented/folder/number`. */
    private fun trackOf(folder: String, number: Int): Track =
        Track(uri = "content://invented/${folder.lowercase()}/$number", displayPath = "$folder/Track $number.mp3")

    /** Opens a library of three tracks in `Alpha` and three in `Beta`, load-order indices 0 to 2 and 3 to 5. */
    private fun openTwoFolders() {
        /** The three tracks of each folder, in load order. */
        val tracks: List<Track> =
            (0 until 3).map { number -> trackOf("Alpha", number) } +
                (0 until 3).map { number -> trackOf("Beta", number) }
        controller.openLibrary(tracks)
    }

    /** URI of the invented track at [index] of the two-folder library. */
    private fun uriAt(index: Int): String = controller.trackAt(index)?.uri ?: error("No track at $index")

    /** A row that is not current is selected and left paused, with nothing loaded to play. */
    @Test
    fun anUnselectedRowIsSelectedAndStaysPaused() {
        openTwoFolders()
        controller.activateRow(1, autoPlayOnSelect = false)
        assertEquals(1, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertEquals(listOf(uriAt(1) to false), engine.loads)
    }

    /** Selecting another row while a track plays pauses the engine at once and loads the new row paused. */
    @Test
    fun selectingAnotherRowStopsThePlayingTrack() {
        openTwoFolders()
        controller.playIndex(0)
        assertTrue(controller.uiState.playing)
        /** Pause requests so far, to see that selecting asks the engine to stop before its asynchronous load. */
        val pausesBefore: Int = engine.pauseCount
        controller.activateRow(2, autoPlayOnSelect = false)
        assertTrue(engine.pauseCount > pausesBefore)
        assertEquals(2, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertFalse(engine.playing)
        assertEquals(uriAt(2) to false, engine.loads.last())
    }

    /** A second tap on the row just selected starts it, from the track already loaded, without a reload. */
    @Test
    fun theRowJustSelectedStartsOnTheSecondTapWithoutReloading() {
        openTwoFolders()
        controller.activateRow(1, autoPlayOnSelect = false)
        controller.activateRow(1, autoPlayOnSelect = false)
        assertTrue(controller.uiState.playing)
        assertEquals(1, engine.loads.size)
    }

    /** Selecting a row tells the engine owner which track was loaded, as playing it does. */
    @Test
    fun selectingARowReportsTheLoadedTrack() {
        openTwoFolders()
        /** URIs reported as loaded, in order. */
        val reported: MutableList<String> = mutableListOf()
        controller.onTrackLoaded = { uri -> reported += uri }
        controller.activateRow(4, autoPlayOnSelect = false)
        assertEquals(listOf(uriAt(4)), reported)
    }

    /** A row tapped while the first library load still runs stays the selected row, paused, when the load finishes. */
    @Test
    fun aRowSelectedDuringTheFirstLoadSurvivesFinishLoad() {
        /** The three invented tracks of the folder. */
        val tracks: List<Track> = (0 until 3).map { number -> trackOf("Alpha", number) }
        controller.beginLoad()
        controller.reconcileLibrary(tracks.take(2))
        controller.activateRow(0, autoPlayOnSelect = false)
        controller.finishLoad(tracks = tracks, session = Session(selected = uriAt(1)))
        assertEquals(0, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertEquals(1, engine.loads.size)
    }

    /** The playing row pauses on a tap and keeps its place, with no new load. */
    @Test
    fun thePlayingRowPausesOnATap() {
        openTwoFolders()
        controller.playIndex(1)
        controller.activateRow(1, autoPlayOnSelect = false)
        assertFalse(controller.uiState.playing)
        assertEquals(1, controller.uiState.currentIndex)
        assertEquals(1, engine.loads.size)
    }

    /** The paused current row resumes on a tap, with no new load. */
    @Test
    fun thePausedCurrentRowResumesOnATap() {
        openTwoFolders()
        controller.playIndex(1)
        controller.activateRow(1, autoPlayOnSelect = false)
        controller.activateRow(1, autoPlayOnSelect = false)
        assertTrue(controller.uiState.playing)
        assertEquals(1, engine.loads.size)
    }

    /** Tapping another row after pausing selects it and leaves it paused. */
    @Test
    fun anotherRowAfterAPauseIsSelectedPaused() {
        openTwoFolders()
        controller.playIndex(1)
        controller.activateRow(1, autoPlayOnSelect = false)
        controller.activateRow(2, autoPlayOnSelect = false)
        assertEquals(2, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertEquals(uriAt(2) to false, engine.loads.last())
    }

    /** The track restored from the saved session is current and loaded paused; one tap starts it without a reload. */
    @Test
    fun theRestoredRowStartsOnATapWithoutReloading() {
        controller.finishLoad(
            tracks = (0 until 3).map { number -> trackOf("Alpha", number) },
            session = Session(selected = "content://invented/alpha/2"),
        )
        assertEquals(2, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        controller.activateRow(2, autoPlayOnSelect = false)
        assertTrue(controller.uiState.playing)
        assertEquals(1, engine.loads.size)
    }

    /** With another folder shown, the current track's index still toggles, and a row of the shown folder selects. */
    @Test
    fun theRuleFollowsTheLibraryIndexWhateverFolderIsShown() {
        openTwoFolders()
        controller.playIndex(1)
        controller.selectPage(1)
        controller.activateRow(1, autoPlayOnSelect = false)
        assertFalse(controller.uiState.playing)
        assertEquals(1, engine.loads.size)
        controller.activateRow(4, autoPlayOnSelect = false)
        assertEquals(4, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertEquals(uriAt(4) to false, engine.loads.last())
        assertEquals(1, controller.uiState.selectedPage)
    }

    /** An index outside the library changes nothing: no load, no pause, no current track. */
    @Test
    fun anIndexOutsideTheLibraryIsIgnored() {
        openTwoFolders()
        controller.activateRow(99, autoPlayOnSelect = false)
        assertNull(controller.uiState.currentIndex)
        assertTrue(engine.loads.isEmpty())
        assertEquals(0, engine.pauseCount)
    }

    /** The explicit Play of the track menu and a Search result still start a row that is not current, at once. */
    @Test
    fun playIndexStillStartsAnotherRowAtOnce() {
        openTwoFolders()
        controller.playIndex(3)
        assertTrue(controller.uiState.playing)
        assertEquals(uriAt(3) to true, engine.loads.last())
    }
}
