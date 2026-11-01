// Wiring cases of the player host, rendered for real under Robolectric over a controller that drives a fake engine:
// the host provides the playing index and the line source to the track list and builds the rows of the page it shows.

// What:     `package ...wiring` groups the Robolectric Compose tests, which share the fixtures of this package.
// Why:      The cases reach `playerHost`, `PlayerController` and `SessionStore` of the app module.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import android.content.Context
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isNotSelected
import androidx.compose.ui.test.isSelected
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.core.app.ApplicationProvider
import dev.monochromatic.musicplayer.FeedbackHostState
import dev.monochromatic.musicplayer.PlayerController
import dev.monochromatic.musicplayer.ReportingAudioEngine
import dev.monochromatic.musicplayer.SearchHost
import dev.monochromatic.musicplayer.SessionStore
import dev.monochromatic.musicplayer.SweepProgress
import dev.monochromatic.musicplayer.Track
import dev.monochromatic.musicplayer.core.SEARCH_FIELD_LABEL
import dev.monochromatic.musicplayer.core.TemplateKind
import dev.monochromatic.musicplayer.playerHost
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

// What:     `class PlayerHostWiringTest` groups the cases of the real player host.
// Why:      The host passes `state.currentIndex` to the list through `LocalPlayingIndex`, builds the rows of the shown
//           page in a `remember` block with its own keys, and hands the list a line source; a mistake at any of those
//           sites keeps every pure-function test green, so each case here pins one through what the screen shows.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("PlayerHost wiring", () => { /* cases rendered with a test renderer */ });
// ```
/** Renders [playerHost] over invented tracks and asserts the playing marker, the opening scroll and the rows. */
@RunWith(RobolectricTestRunner::class)
@Config(qualifiers = "w411dp-h891dp-xxhdpi")
class PlayerHostWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Test engine, so no native library or audio device is involved; it records loads and reports play changes. */
    private val engine = ReportingAudioEngine()

    /** Controller over [engine]. */
    private val controller = PlayerController(engine)

    // What:     `@Before fun resetProcessState()` puts the process-wide state of the app into a known shape.
    // Why:      Robolectric keeps the app's classes loaded across the cases of a run, so the template cache of
    //           `SessionStore` and the sweep progress carry over; each case starts from a template that shows every
    //           track's file extension and from an idle sweep.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // beforeEach(() => { sessionStore.saveTemplate("track rows", "$mi(ext)$"); sweepProgress.reset(); });
    // ```
    /** Saves the extension template for the track rows and clears the sweep progress. */
    @Before
    fun resetProcessState() {
        /** Application context that the host reads the saved templates through. */
        val context: Context = ApplicationProvider.getApplicationContext()
        SessionStore.saveTemplate(context, TemplateKind.TRACK_ROWS, EXTENSION_TEMPLATE)
        SessionStore.saveAutoPlayOnSelect(context, enabled = false)
        SweepProgress.reset()
    }

    /** Invented tracks of one folder, `Alpha/Track NN.mp3`, with content URIs that name nothing real. */
    private fun alpha(count: Int): List<Track> = (0 until count).map { track ->
        Track(uri = "content://invented/alpha/$track", displayPath = "Alpha/${titleOf(track)}.mp3")
    }

    /** Invented tracks of a second folder, `Beta/Other NN.mp3`, whose titles differ from the first folder's. */
    private fun beta(count: Int): List<Track> = (0 until count).map { track ->
        Track(
            uri = "content://invented/beta/$track",
            displayPath = "Beta/Other " + track.toString().padStart(2, '0') + ".mp3",
        )
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

    /** Runs [change] on the UI thread, where the controller expects its calls, and waits for the frames it causes. */
    private fun change(change: () -> Unit) {
        composeTestRule.runOnIdle(change)
        composeTestRule.waitForIdle()
    }

    /** Count of nodes whose text list holds exactly [text], zero when no composed row shows it. */
    private fun countNodesWithText(text: String): Int =
        composeTestRule.onAllNodes(hasText(text)).fetchSemanticsNodes().size

    // What:     `@Test fun thePlayingRowIsTheOnlySelectedRow()` checks the host's playing index reaches the rows.
    // Why:      The host provides `LocalPlayingIndex provides state.currentIndex`; with any other value the rows get
    //           no marker, because the rows the host builds carry none of their own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one selected row", () => { render(host, { playing: 3 }); expect(selectedRows()).toEqual(["Track 03"]); });
    // ```
    /** With the fourth track playing, exactly the row titled `Track 03` is selected and its neighbours are not. */
    @Test(timeout = CASE_LIMIT_MS)
    fun thePlayingRowIsTheOnlySelectedRow() {
        controller.openLibrary(alpha(count = 8))
        controller.playIndex(3)
        showHost()
        composeTestRule.onAllNodes(hasText(titleOf(3)) and isSelected()).assertCountEquals(1)
        composeTestRule.onAllNodes(hasText(titleOf(2)) and isNotSelected()).assertCountEquals(1)
        composeTestRule.onAllNodes(hasText(titleOf(4)) and isNotSelected()).assertCountEquals(1)
        composeTestRule.onAllNodes(hasText("Track", substring = true) and isSelected()).assertCountEquals(1)
    }

    // What:     `@Test fun theMarkerFollowsNextAndPrevious()` checks the host updates the marker on a track change.
    // Why:      The rows of the page are not rebuilt on a track change, so the only way the marker moves is the playing
    //           index that the host re-provides each time the controller's state changes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("marker follows next", () => { render(host, { playing: 3 }); next(); expect(selectedRows()).toEqual(["Track 04"]); });
    // ```
    /** Next moves the selected row from `Track 03` to `Track 04`, and previous moves it back. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theMarkerFollowsNextAndPrevious() {
        controller.openLibrary(alpha(count = 8))
        controller.playIndex(3)
        showHost()
        change { controller.next() }
        composeTestRule.onAllNodes(hasText(titleOf(4)) and isSelected()).assertCountEquals(1)
        composeTestRule.onAllNodes(hasText(titleOf(3)) and isNotSelected()).assertCountEquals(1)
        change { controller.prev() }
        composeTestRule.onAllNodes(hasText(titleOf(3)) and isSelected()).assertCountEquals(1)
        composeTestRule.onAllNodes(hasText(titleOf(4)) and isNotSelected()).assertCountEquals(1)
    }

    // What:     `@Test fun theHostOpensTheListOnThePlayingRow()` checks the page entry scroll through the whole host.
    // Why:      The pane scrolls to the playing row on entry only if the host's playing index reaches it, so this case
    //           fails when the host provides no index even though the pane's own cases pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("opens on playing", () => { render(host, { rows: 60, playing: 40 }); expect(visible("Track 40")).toBe(true); });
    // ```
    /** A sixty-row folder with track 40 playing opens on that row, and the first row is not composed. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theHostOpensTheListOnThePlayingRow() {
        controller.openLibrary(alpha(count = 60))
        controller.playIndex(40)
        showHost()
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            composeTestRule.onAllNodes(hasText(titleOf(40)) and isSelected()).fetchSemanticsNodes().size == 1
        }
        assertEquals(0, countNodesWithText(titleOf(0)))
    }

    // What:     `@Test fun changingTheFolderMovesTheListToTheNewPage()` checks the page effect through the whole host.
    // Why:      A folder change builds new rows, which the pane must treat as a new page: the folder without the playing
    //           track opens at its top, and going back to the playing folder opens on the playing row again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("page change", () => { render(host, { playing: "Alpha 40" }); selectPage(1); expect(visible("Other 00")).toBe(true); });
    // ```
    /** From the playing folder scrolled to its row 40, folder two opens at its top and folder one reopens on row 40. */
    @Test(timeout = CASE_LIMIT_MS)
    fun changingTheFolderMovesTheListToTheNewPage() {
        controller.openLibrary(alpha(count = 60) + beta(count = 60))
        controller.playIndex(40)
        showHost()
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            composeTestRule.onAllNodes(hasText(titleOf(40)) and isSelected()).fetchSemanticsNodes().size == 1
        }
        change { controller.selectPage(1) }
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) { countNodesWithText("Other 00") == 1 }
        assertEquals(0, countNodesWithText("Other 40"))
        change { controller.selectPage(0) }
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            composeTestRule.onAllNodes(hasText(titleOf(40)) and isSelected()).fetchSemanticsNodes().size == 1
        }
        assertEquals(0, countNodesWithText(titleOf(0)))
    }

    // What:     `@Test fun theMeasureRefreshStartsAtTheFirstVisibleTrack()` checks the list reports its top row to the host.
    // Why:      The host provides a cell through `LocalFirstVisibleTrack`, the list writes the library index of its first
    //           visible track into it, and each batch of the measure refresh starts there; with no cell or a wrong
    //           value a pass starts at the top of a page of thousands of tracks and the rows on screen wait.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("refresh starts on screen", () => { render(host, { rows: 60, playing: 40 }); sweepStep(); expect(firstRead()).toBe("alpha/40"); });
    // ```
    /** After the list opens on track 40, the next pass reads that track first and not the top of the page. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theMeasureRefreshStartsAtTheFirstVisibleTrack() {
        /** Provider that records the order in which the refresh asks about the tracks' files. */
        val provider: RecordingProvider = Robolectric.setupContentProvider(RecordingProvider::class.java, "invented")
        controller.openLibrary(alpha(count = 60))
        controller.playIndex(40)
        showHost()
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            composeTestRule.onAllNodes(hasText(titleOf(40)) and isSelected()).fetchSemanticsNodes().size == 1
        }
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) { provider.queried.size >= 60 }
        provider.queried.clear()
        change { SweepProgress.report(done = 1, total = 60) }
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) { provider.queried.isNotEmpty() }
        assertEquals("content://invented/alpha/40", provider.queried.first())
    }

    // What:     `@Test fun aTrackAddedToTheShownFolderAppearsAsARow()` checks the rows are rebuilt when the page changes.
    // Why:      The host builds the rows in a `remember` block keyed on the page's entries; a rescan that adds a track
    //           to the folder keeps the folder's name, so only the entries key can make the new row appear.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("new row", () => { render(host, { rows: 5 }); rescan(6); expect(visible("Track 05")).toBe(true); });
    // ```
    /** After a rescan adds `Track 05` to a folder of five tracks, the list shows a row for it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aTrackAddedToTheShownFolderAppearsAsARow() {
        controller.openLibrary(alpha(count = 5))
        showHost()
        assertEquals(0, countNodesWithText(titleOf(5)))
        change { controller.reconcileLibrary(alpha(count = 6)) }
        assertEquals(1, countNodesWithText(titleOf(5)))
    }

    // What:     `@Test fun aTrackAddedToTheShownFolderShowsItsSupportingLine()` checks the line source is rebuilt too.
    // Why:      The host hands the rows a source built from the page's entries in a `remember` block; a source that
    //           kept the old entries finds no path for the new track and leaves its row without a line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("new row has its line", () => { render(host, { rows: 5 }); rescan(6); expect(rowText("Track 05")).toContain("mp3"); });
    // ```
    /** The row of the added track shows the extension line like every other row. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aTrackAddedToTheShownFolderShowsItsSupportingLine() {
        controller.openLibrary(alpha(count = 5))
        showHost()
        assertEquals(5, countNodesWithText(EXTENSION_LINE))
        change { controller.reconcileLibrary(alpha(count = 6)) }
        composeTestRule.onAllNodes(hasText(titleOf(5)) and hasText(EXTENSION_LINE)).assertCountEquals(1)
        assertEquals(6, countNodesWithText(EXTENSION_LINE))
    }

    /** Opens the Search page from the top bar, types [title], and taps the first result row that shows it. */
    private fun activateSearchResultTitled(title: String) {
        composeTestRule.onNodeWithContentDescription(SEARCH_BUTTON_LABEL).performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).performTextInput(title)
        /** Matches a tappable result row whose text holds [title]; the query field also holds it and is not a row. */
        val resultRow = hasClickAction() and hasText(title, substring = true) and !hasSetTextAction()
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            composeTestRule.onAllNodes(resultRow).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onAllNodes(resultRow)[0].performClick()
        composeTestRule.waitForIdle()
    }

    // What:     `@Test fun aSearchResultSelectsAnotherTrackAndTheSecondActivationStartsIt()` runs D127 via the host.
    // Why:      `openSearchResult` maps the activation to a controller call; a pure test of the decision cannot see a
    //           host that still starts the track at once, which is the behavior D127 removes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("search result selects", () => {
    //   searchAndTap("Track 05"); expect(paused(5));
    //   searchAndTap("Track 05"); expect(playing(5));
    // });
    // ```
    /** Tapping the result of another track loads it paused as the current track; tapping its result again plays it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aSearchResultSelectsAnotherTrackAndTheSecondActivationStartsIt() {
        controller.searchHost = SearchHost()
        controller.openLibrary(alpha(count = 8))
        controller.playIndex(1)
        showHost()
        assertTrue("the first track plays before the search", controller.uiState.playing)
        activateSearchResultTitled(titleOf(5))
        assertEquals("the result's track is the current track", 5, controller.uiState.currentIndex)
        assertFalse("selecting a result does not play it and stops the playing track", controller.uiState.playing)
        assertEquals(
            "the engine was cued with the track, paused",
            "content://invented/alpha/5" to false,
            engine.loads.last(),
        )
        assertFalse(engine.playing)
        activateSearchResultTitled(titleOf(5))
        assertEquals(5, controller.uiState.currentIndex)
        assertTrue("the result of the current track starts it", controller.uiState.playing)
        assertTrue(engine.playing)
    }

    /** Template of the track rows that shows only the file extension, so every row has a line without a measure. */
    private companion object {
        /** Template text, in the app's formula syntax, that prints the extension field. */
        const val EXTENSION_TEMPLATE: String = "\$mi(ext)\$"

        /** The line that template gives each of the invented `.mp3` tracks. */
        const val EXTENSION_LINE: String = "mp3"

        /** Spoken label of the top bar's Search button. */
        const val SEARCH_BUTTON_LABEL: String = "Search music"
    }
}
