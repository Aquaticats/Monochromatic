package dev.monochromatic.musicplayer

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Pins the Auto-play on select half of the row rule (D129): with the setting on, a row that is not current is selected
 * and starts at once, the current row still toggles, and an index past the library does nothing. The cases with the
 * setting off are in `PlayerControllerRowTest`; the case of a file that cannot be opened is in
 * `PlayerControllerRowFailureTest`, because it logs through `android.util.Log`.
 */
class PlayerControllerAutoPlayTest {
    /** Engine double that reports playing changes. */
    private val engine = ReportingAudioEngine()

    /** Controller under test, over [engine]. */
    private val controller = PlayerController(engine)

    /** Opens six invented tracks, load-order indices 0 to 5. */
    private fun openSix() {
        controller.openLibrary(
            (0 until 6).map { number ->
                Track(uri = "content://invented/alpha/$number", displayPath = "Alpha/Track $number.mp3")
            },
        )
    }

    /** URI of the invented track at [index]. */
    private fun uriAt(index: Int): String = controller.trackAt(index)?.uri ?: error("No track at $index")

    /** A row that is not current is selected and loaded to play at once. */
    @Test
    fun anUnselectedRowStartsPlayingAtOnce() {
        openSix()
        controller.activateRow(2, autoPlayOnSelect = true)
        assertEquals(2, controller.uiState.currentIndex)
        assertTrue(controller.uiState.playing)
        assertTrue(engine.playing)
        assertEquals(listOf(uriAt(2) to true), engine.loads)
    }

    /** With a track playing, another row replaces it and keeps playing. */
    @Test
    fun anotherRowReplacesThePlayingTrackAndKeepsPlaying() {
        openSix()
        controller.activateRow(1, autoPlayOnSelect = true)
        controller.activateRow(4, autoPlayOnSelect = true)
        assertEquals(4, controller.uiState.currentIndex)
        assertTrue(controller.uiState.playing)
        assertEquals(uriAt(4) to true, engine.loads.last())
    }

    /** The current row toggles with the setting on, without loading the track again. */
    @Test
    fun theCurrentRowStillTogglesPlayAndPause() {
        openSix()
        controller.activateRow(3, autoPlayOnSelect = true)
        controller.activateRow(3, autoPlayOnSelect = true)
        assertFalse(controller.uiState.playing)
        controller.activateRow(3, autoPlayOnSelect = true)
        assertTrue(controller.uiState.playing)
        assertEquals(1, engine.loads.size)
    }

    /** An index past the library leaves the current track and the engine alone. */
    @Test
    fun anIndexPastTheLibraryDoesNothing() {
        openSix()
        controller.activateRow(2, autoPlayOnSelect = true)
        /** Loads before the stray call. */
        val loadsBefore: Int = engine.loads.size
        controller.activateRow(99, autoPlayOnSelect = true)
        assertEquals(2, controller.uiState.currentIndex)
        assertEquals(loadsBefore, engine.loads.size)
        assertTrue(controller.uiState.playing)
    }

    /** With nothing selected, an index past the library does not start the first track by mistake. */
    @Test
    fun anIndexPastTheLibraryStartsNothingWhenNothingIsSelected() {
        openSix()
        controller.activateRow(99, autoPlayOnSelect = true)
        assertEquals(null, controller.uiState.currentIndex)
        assertTrue(engine.loads.isEmpty())
        assertFalse(controller.uiState.playing)
    }
}
