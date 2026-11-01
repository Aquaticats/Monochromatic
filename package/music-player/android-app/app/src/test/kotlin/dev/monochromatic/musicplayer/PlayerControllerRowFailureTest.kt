package dev.monochromatic.musicplayer

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/**
 * Pins the row rule (D124) where a load fails: the controller logs a failure through `android.util.Log`, which a plain
 * JVM test cannot call, so these cases run under Robolectric.
 */
@RunWith(RobolectricTestRunner::class)
class PlayerControllerRowFailureTest {
    /** Engine double that reports playing changes and load failures. */
    private val engine = ReportingAudioEngine()

    /** Controller under test, over [engine]. */
    private val controller = PlayerController(engine)

    /** Opens a library of three invented tracks, load-order indices 0 to 2. */
    private fun openThree() {
        controller.openLibrary(
            (0 until 3).map { number ->
                Track(uri = "content://invented/alpha/$number", displayPath = "Alpha/Track $number.mp3")
            },
        )
    }

    /** URI of the invented track at [index]. */
    private fun uriAt(index: Int): String = controller.trackAt(index)?.uri ?: error("No track at $index")

    /** A current row whose file failed to open stays selected, and every further tap tries to open it again. */
    @Test
    fun aRowWhoseLoadFailedIsTriedAgainOnEachTap() {
        openThree()
        engine.unreadable += uriAt(1)
        controller.playIndex(1)
        assertNotNull(controller.loadFailure)
        assertEquals(1, controller.uiState.currentIndex)
        controller.activateRow(1, autoPlayOnSelect = false)
        assertEquals(2, engine.loads.size)
        assertEquals(uriAt(1) to true, engine.loads.last())
        assertEquals(2, controller.loadFailure?.serial)
        engine.unreadable.clear()
        controller.activateRow(1, autoPlayOnSelect = false)
        assertEquals(3, engine.loads.size)
        assertTrue(controller.uiState.playing)
        controller.activateRow(1, autoPlayOnSelect = false)
        assertFalse(controller.uiState.playing)
        assertEquals(3, engine.loads.size)
    }

    /** A row whose file fails to open on selection stays selected and does not claim to play. */
    @Test
    fun selectingARowThatCannotBeOpenedLeavesItSelectedAndPaused() {
        openThree()
        engine.unreadable += uriAt(2)
        controller.activateRow(2, autoPlayOnSelect = false)
        assertEquals(2, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertNotNull(controller.loadFailure)
        engine.unreadable.clear()
        controller.activateRow(2, autoPlayOnSelect = false)
        assertTrue(controller.uiState.playing)
        assertEquals(uriAt(2) to true, engine.loads.last())
    }

    /** With Auto-play on select on, a row whose file cannot be opened stays selected, reports once and does not start. */
    @Test
    fun anUnreadableRowStaysSelectedWhenAutoPlayIsOn() {
        openThree()
        engine.unreadable += uriAt(2)
        controller.activateRow(2, autoPlayOnSelect = true)
        assertEquals(2, controller.uiState.currentIndex)
        assertFalse(controller.uiState.playing)
        assertFalse(engine.playing)
        assertEquals(1, controller.loadFailure?.serial)
        engine.unreadable.clear()
        controller.activateRow(2, autoPlayOnSelect = true)
        assertTrue("the next activation of the current row retries the load", controller.uiState.playing)
        assertEquals(uriAt(2) to true, engine.loads.last())
    }
}
