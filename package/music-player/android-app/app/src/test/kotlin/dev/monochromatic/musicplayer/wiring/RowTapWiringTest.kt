package dev.monochromatic.musicplayer.wiring

import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isNotSelected
import androidx.compose.ui.test.isSelected
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.pressKey
import androidx.test.core.app.ApplicationProvider
import dev.monochromatic.musicplayer.FeedbackHostState
import dev.monochromatic.musicplayer.PlayerController
import dev.monochromatic.musicplayer.ReportingAudioEngine
import dev.monochromatic.musicplayer.SessionStore
import dev.monochromatic.musicplayer.SweepProgress
import dev.monochromatic.musicplayer.Track
import dev.monochromatic.musicplayer.playerHost
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/**
 * Taps the rows of the real [playerHost] over a controller whose engine reports playing changes, and asserts the rule
 * of D124: a tap selects a row that is not current and leaves it paused, and a tap on the current row toggles play.
 */
@RunWith(RobolectricTestRunner::class)
@Config(qualifiers = "w411dp-h891dp-xxhdpi")
class RowTapWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Engine that reports playing changes the way the real one does. */
    private val engine = ReportingAudioEngine()

    /** Controller over [engine], so no native library or audio device is involved. */
    private val controller = PlayerController(engine)

    /** Clears the sweep progress and the Auto-play on select setting that earlier cases of the process may have left. */
    @Before
    fun resetProcessState() {
        SessionStore.saveAutoPlayOnSelect(ApplicationProvider.getApplicationContext(), enabled = false)
        SweepProgress.reset()
    }

    /** Invented tracks of one folder, `Alpha/Track NN.mp3`, with content URIs that name nothing real. */
    private fun alpha(count: Int): List<Track> = (0 until count).map { track ->
        Track(uri = "content://invented/alpha/$track", displayPath = "Alpha/${titleOf(track)}.mp3")
    }

    /** Shows the real host over [controller] and waits until its first frames are composed. */
    private fun showHost() {
        composeTestRule.setContent {
            MaterialTheme {
                playerHost(controller = controller, feedback = FeedbackHostState(), onChooseFolder = { })
            }
        }
        composeTestRule.waitForIdle()
    }

    /** The list row titled for [track], which is the only node with that text that carries a selected state. */
    private fun row(track: Int): SemanticsNodeInteraction =
        composeTestRule.onNode(hasText(titleOf(track)) and (isSelected() or isNotSelected()))

    /** Taps the row of [track] and waits for the frames the tap causes. */
    private fun tap(track: Int) {
        row(track).performClick()
        composeTestRule.waitForIdle()
    }

    /** Tapping a row of a quiet player selects it and does not start it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aTapOnAnUnselectedRowSelectsItAndPlaysNothing() {
        controller.openLibrary(alpha(count = 6))
        showHost()
        tap(2)
        row(2).assertIsSelected()
        assertEquals(2, controller.uiState.currentIndex)
        assertFalse(engine.playing)
        assertEquals(listOf("content://invented/alpha/2" to false), engine.loads)
    }

    /** Select, start, pause and resume by tapping one row, then a tap on another row selects it and stops playback. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theSameRowThenAnotherRowFollowTheRule() {
        controller.openLibrary(alpha(count = 6))
        showHost()
        tap(2)
        assertFalse(engine.playing)
        tap(2)
        assertTrue("the second tap starts the selected row", engine.playing)
        assertTrue(controller.uiState.playing)
        tap(2)
        assertFalse("the third tap pauses it", engine.playing)
        tap(2)
        assertTrue("the fourth tap resumes it", engine.playing)
        assertEquals("selection, start, pause and resume never reload the track", 1, engine.loads.size)
        tap(4)
        row(4).assertIsSelected()
        assertEquals(4, controller.uiState.currentIndex)
        assertFalse("selecting another row stops the playing track", engine.playing)
        assertFalse(controller.uiState.playing)
        assertEquals("content://invented/alpha/4" to false, engine.loads.last())
    }

    /** A tap on the row that plays pauses it and keeps it selected. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aTapOnThePlayingRowPausesIt() {
        controller.openLibrary(alpha(count = 6))
        controller.playIndex(3)
        showHost()
        tap(3)
        assertFalse(engine.playing)
        row(3).assertIsSelected()
        assertEquals(1, engine.loads.size)
    }

    /** Moves the ring onto the track list with Alt+2, then onto a row with Down, and presses Enter [times] times. */
    private fun enterOnTheRingedRow(times: Int) {
        composeTestRule.onRoot().performKeyInput {
            keyDown(Key.AltLeft)
            pressKey(Key.Two)
            keyUp(Key.AltLeft)
            pressKey(Key.DirectionDown)
        }
        composeTestRule.waitForIdle()
        repeat(times) {
            composeTestRule.onRoot().performKeyInput { pressKey(Key.Enter) }
            composeTestRule.waitForIdle()
        }
    }

    /** Enter on the ringed row selects it and leaves it paused, as a tap does. */
    @Test(timeout = CASE_LIMIT_MS)
    fun enterOnAnUnselectedRowSelectsItAndPlaysNothing() {
        controller.openLibrary(alpha(count = 6))
        showHost()
        enterOnTheRingedRow(times = 1)
        assertTrue("Enter selected a track", controller.uiState.currentIndex != null)
        assertFalse(engine.playing)
        assertEquals(false, engine.loads.single().second)
    }

    /** A second and a third Enter on the same row start it and pause it, with no reload. */
    @Test(timeout = CASE_LIMIT_MS)
    fun enterOnTheSelectedRowTogglesPlayAndPause() {
        controller.openLibrary(alpha(count = 6))
        showHost()
        enterOnTheRingedRow(times = 2)
        assertTrue("the second Enter started the selected row", engine.playing)
        composeTestRule.onRoot().performKeyInput { pressKey(Key.Enter) }
        composeTestRule.waitForIdle()
        assertFalse("the third Enter paused it", engine.playing)
        assertEquals(1, engine.loads.size)
    }
}
