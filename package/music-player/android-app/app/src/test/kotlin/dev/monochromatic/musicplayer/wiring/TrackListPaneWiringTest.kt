// Wiring cases of the track list pane, rendered for real under Robolectric with the Compose test rule: which row
// carries the playing marker, where the list opens, and what moves it. Pure functions behind these sites have their own
// JVM tests; these cases cover the Compose call sites that pass them their arguments.

// What:     `package ...wiring` groups the Robolectric Compose tests, which share the fixtures of this package.
// Why:      The cases reach `trackListPane` and the internal composition locals of the app module.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import dev.monochromatic.musicplayer.HostKeyLocalReveal
import dev.monochromatic.musicplayer.HostKeyLocalRevealFolder
import dev.monochromatic.musicplayer.LocalPlayingIndex
import dev.monochromatic.musicplayer.LocalTrackRowLines
import dev.monochromatic.musicplayer.core.TrackListEntry
import dev.monochromatic.musicplayer.core.TrackMeasure
import dev.monochromatic.musicplayer.core.TrackRowLineSource
import dev.monochromatic.musicplayer.trackListPane
import java.util.concurrent.ConcurrentHashMap
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

// What:     `class TrackListPaneWiringTest` groups the cases of the real track list pane.
// Why:      The pane reads the playing index, the reveal counts and the row-line source from composition locals and
//           hands them to the scroll rules and to every row; a wrong argument at any of those calls passes every
//           pure-function test, so each case here pins one of them through what the screen shows.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("TrackListPane wiring", () => { /* cases rendered with a test renderer */ });
// ```
/** Renders [trackListPane] with invented rows and asserts the playing marker and the scroll position it produces. */
@RunWith(RobolectricTestRunner::class)
@Config(qualifiers = "w400dp-h800dp-mdpi")
class TrackListPaneWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** The three counters the host passes to the pane through composition locals, changed by the cases. */
    private class HostInputs {
        /** Library index of the playing track, or null when none is loaded. */
        val playing: MutableState<Int?> = mutableStateOf(null)

        /** Count of reveal requests, which the host raises when the deck's track part is tapped. */
        val reveal: MutableState<Int> = mutableStateOf(0)

        /** Count of folder reveal requests, which the host raises when the deck's folder part is tapped. */
        val folderReveal: MutableState<Int> = mutableStateOf(0)
    }

    // What:     `@Composable private fun paneUnderTest(...)` draws the pane the way the host does.
    // Why:      The host provides the playing index, the reveal counts and the line source as composition locals around
    //           the player content, and gives the pane its entries; the cases do the same with invented values.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const PaneUnderTest = ({ entries, inputs, lines }) => (
    //   <Providers playing={inputs.playing} reveal={inputs.reveal} ...><TrackListPane entries={entries} /></Providers>
    // );
    // ```
    /** Draws [entries] in the real pane with the locals the host would provide, read from [inputs] on each frame. */
    @Composable
    private fun paneUnderTest(entries: List<TrackListEntry>, inputs: HostInputs, lines: TrackRowLineSource? = null) {
        MaterialTheme {
            CompositionLocalProvider(
                LocalPlayingIndex provides inputs.playing.value,
                HostKeyLocalReveal provides inputs.reveal.value,
                HostKeyLocalRevealFolder provides inputs.folderReveal.value,
                LocalTrackRowLines provides lines,
            ) {
                trackListPane(
                    entries = entries,
                    onPlay = { },
                    onOpenMenu = null,
                    modifier = Modifier.fillMaxSize(),
                )
            }
        }
    }

    /** Shows [entries] in the pane and waits until the first frame is composed. */
    private fun show(entries: List<TrackListEntry>, inputs: HostInputs, lines: TrackRowLineSource? = null) {
        composeTestRule.setContent { paneUnderTest(entries = entries, inputs = inputs, lines = lines) }
        composeTestRule.waitForIdle()
    }

    /** Changes the host's inputs on the UI thread and waits until the resulting frames are composed. */
    private fun change(change: () -> Unit) {
        composeTestRule.runOnIdle(change)
        composeTestRule.waitForIdle()
    }

    /** Waits, for at most [WAIT_LIMIT_MS], until the row titled [title] is on screen. */
    private fun awaitRow(title: String) {
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            countNodesWithText(title) > 0
        }
    }

    /** Count of nodes whose text is exactly [title], zero when the list has not composed that row. */
    private fun countNodesWithText(title: String): Int =
        composeTestRule.onAllNodes(hasText(title)).fetchSemanticsNodes().size

    // What:     `@Test fun onlyThePlayingRowIsMarkedSelected()` checks which row carries the playing marker.
    // Why:      The rows arrive unflagged, so the marker exists only if the pane passes the playing index to
    //           `rowWithPlayingFlag`; dropping the call, or its argument, leaves every row unmarked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one marked row", () => { render({ playing: 1003 }); expect(selected("Track 03")).toBe(true); });
    // ```
    /** With the track at position 3 playing, that row is selected and its neighbours are not. */
    @Test(timeout = CASE_LIMIT_MS)
    fun onlyThePlayingRowIsMarkedSelected() {
        /** Inputs of a host that is playing the track numbered 3. */
        val inputs = HostInputs()
        inputs.playing.value = libraryIndexOf(3)
        show(flatPage(count = 8), inputs)
        composeTestRule.onNodeWithText(titleOf(3)).assertIsSelected()
        composeTestRule.onNodeWithText(titleOf(2)).assertIsNotSelected()
        composeTestRule.onNodeWithText(titleOf(4)).assertIsNotSelected()
        composeTestRule.onNodeWithText(titleOf(0)).assertIsNotSelected()
    }

    // What:     `@Test fun theMarkerFollowsTheLibraryIndexNotThePosition()` checks the pane names the playing row by
    //           library index.
    // Why:      An index that equals no position (or the position of another row) must still mark the row that holds
    //           the index; a row is marked by `indexInLibrary`, never by its place in the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("marker by index", () => { render({ playing: 1005 }); expect(selected("Track 05")).toBe(true); });
    // ```
    /** The playing index 1005 marks the row of track 5, and the row at position 5 of a shifted page is not mistaken. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theMarkerFollowsTheLibraryIndexNotThePosition() {
        /** Inputs of a host that is playing the track numbered 5. */
        val inputs = HostInputs()
        inputs.playing.value = libraryIndexOf(5)
        /** A page whose first position holds a header, so every row sits one position after its number. */
        val shifted: List<TrackListEntry> =
            listOf(TrackListEntry.HeaderEntry(name = "Alpha", trackCount = 8)) + flatPage(count = 8)
        show(shifted, inputs)
        composeTestRule.onNodeWithText(titleOf(5)).assertIsSelected()
        composeTestRule.onNodeWithText(titleOf(6)).assertIsNotSelected()
        composeTestRule.onNodeWithText(titleOf(4)).assertIsNotSelected()
    }

    // What:     `@Test fun theMarkerMovesWhenThePlayingTrackChanges()` checks a track change moves the marker.
    // Why:      The pane reads the playing index from the local on every frame, so a new value must unmark the old
    //           row and mark the new one without a new set of entries.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("marker moves", () => { render({ playing: 1003 }); setPlaying(1005); expect(selected("Track 05")).toBe(true); });
    // ```
    /** Changing the playing index from track 3 to track 5 moves the selected state between those two rows. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theMarkerMovesWhenThePlayingTrackChanges() {
        /** Inputs of a host that starts on the track numbered 3. */
        val inputs = HostInputs()
        inputs.playing.value = libraryIndexOf(3)
        show(flatPage(count = 8), inputs)
        change { inputs.playing.value = libraryIndexOf(5) }
        composeTestRule.onNodeWithText(titleOf(5)).assertIsSelected()
        composeTestRule.onNodeWithText(titleOf(3)).assertIsNotSelected()
    }

    // What:     `@Test fun noPlayingTrackLeavesEveryRowUnmarked()` checks the marker clears when playback is gone.
    // Why:      A row may arrive flagged current from an older build of the entries; the playing index alone decides,
    //           so a null index must clear it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no marker", () => { render({ playing: null, rows: [flagged(2)] }); expect(selected("Track 02")).toBe(false); });
    // ```
    /** A row flagged current by its entry is drawn unselected while no track is playing. */
    @Test(timeout = CASE_LIMIT_MS)
    fun noPlayingTrackLeavesEveryRowUnmarked() {
        /** Inputs of a host with no track loaded. */
        val inputs = HostInputs()
        /** Rows in which the one at position 2 arrives flagged current. */
        val entries: List<TrackListEntry> =
            flatPage(count = 8).mapIndexed { at, entry -> if (at == 2) trackEntry(2, flaggedCurrent = true) else entry }
        show(entries, inputs)
        composeTestRule.onNodeWithText(titleOf(2)).assertIsNotSelected()
        composeTestRule.onNodeWithText(titleOf(0)).assertIsNotSelected()
    }

    // What:     `@Test fun thePageOpensOnThePlayingRow()` checks where a page starts when it holds the playing row.
    // Why:      The page effect scrolls to `pageEntryScrollTarget(entries, playingIndex)`; a wrong index or entry list
    //           there leaves the page at its top, which no pure-function test of the target can see.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("opens on the playing row", () => { render({ playing: 1040, rows: 60 }); expect(visible("Track 40")).toBe(true); });
    // ```
    /** A page of sixty rows entered with track 40 playing shows that row and not the first one. */
    @Test(timeout = CASE_LIMIT_MS)
    fun thePageOpensOnThePlayingRow() {
        /** Inputs of a host playing the track numbered 40. */
        val inputs = HostInputs()
        inputs.playing.value = libraryIndexOf(40)
        show(flatPage(count = 60), inputs)
        awaitRow(titleOf(40))
        composeTestRule.onNodeWithText(titleOf(40)).assertIsDisplayed().assertIsSelected()
        composeTestRule.onNodeWithText(titleOf(0)).assertDoesNotExist()
    }

    // What:     `@Test fun thePageOpensOnThePlayingRowBehindHeaders()` checks the target is a position, not an index.
    // Why:      Headers shift every later row, so the row of track 50 sits at position 52 of the folded page; a target
    //           built from the library index or from a header-blind count would land elsewhere.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("opens on a row behind headers", () => { render({ playing: 1050, rows: folded }); expect(visible("Track 50")).toBe(true); });
    // ```
    /** On the folded page, entering with track 50 playing shows its row and hides the rows near the top. */
    @Test(timeout = CASE_LIMIT_MS)
    fun thePageOpensOnThePlayingRowBehindHeaders() {
        /** Inputs of a host playing the track numbered 50, under the second subfolder. */
        val inputs = HostInputs()
        inputs.playing.value = libraryIndexOf(50)
        show(foldedPage(), inputs)
        awaitRow(titleOf(50))
        composeTestRule.onNodeWithText(titleOf(50)).assertIsDisplayed().assertIsSelected()
        composeTestRule.onNodeWithText(titleOf(0)).assertDoesNotExist()
    }

    // What:     `@Test fun aPageWithoutThePlayingRowOpensAtTheTop()` checks the fallback target.
    // Why:      A page that does not hold the playing track starts at its top; the target must not be the playing index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("opens at the top", () => { render({ playing: 5000, rows: 60 }); expect(visible("Track 00")).toBe(true); });
    // ```
    /** With a playing index that no row holds, the page shows its first row and not a late one. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aPageWithoutThePlayingRowOpensAtTheTop() {
        /** Inputs of a host playing a track from another folder. */
        val inputs = HostInputs()
        inputs.playing.value = libraryIndexOf(500)
        show(flatPage(count = 60), inputs)
        composeTestRule.onNodeWithText(titleOf(0)).assertIsDisplayed()
        composeTestRule.onNodeWithText(titleOf(40)).assertDoesNotExist()
    }

    // What:     `@Test fun aTrackChangeNeverMovesTheList()` checks that playing a new track leaves the list in place.
    // Why:      The list moves only on entry, on a page change and on a reveal request; the playing index feeds the
    //           reveal but must never scroll by itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no scroll on track change", () => { render({ playing: null }); setPlaying(1040); expect(visible("Track 40")).toBe(false); });
    // ```
    /** Starting playback of track 40 on a page shown from its top does not scroll to that row. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aTrackChangeNeverMovesTheList() {
        /** Inputs of a host with no track loaded at first. */
        val inputs = HostInputs()
        show(flatPage(count = 60), inputs)
        change { inputs.playing.value = libraryIndexOf(40) }
        composeTestRule.onNodeWithText(titleOf(0)).assertIsDisplayed()
        composeTestRule.onNodeWithText(titleOf(40)).assertDoesNotExist()
    }

    // What:     `@Test fun aRevealRequestScrollsToThePlayingRow()` checks the reveal reaches the playing row.
    // Why:      The reveal effect scrolls to the position that `currentPositionOf(entries, playingIndex)` gives; with
    //           the wrong index that position is null and the request does nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("reveal", () => { render({ playing: null }); setPlaying(1040); reveal(); expect(visible("Track 40")).toBe(true); });
    // ```
    /** After a track far below the screen starts, one reveal request brings its row into view. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aRevealRequestScrollsToThePlayingRow() {
        /** Inputs of a host that starts without a track, then plays track 40. */
        val inputs = HostInputs()
        show(flatPage(count = 60), inputs)
        change { inputs.playing.value = libraryIndexOf(40) }
        change { inputs.reveal.value += 1 }
        awaitRow(titleOf(40))
        composeTestRule.onNodeWithText(titleOf(40)).assertIsDisplayed().assertIsSelected()
        composeTestRule.onNodeWithText(titleOf(0)).assertDoesNotExist()
    }

    // What:     `@Test fun aRevealRequestWithoutAPlayingRowDoesNotMoveTheList()` checks the reveal guard.
    // Why:      With no row to reveal the request must leave the list where it is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("reveal without a row", () => { render({ playing: null }); reveal(); expect(visible("Track 00")).toBe(true); });
    // ```
    /** A reveal request while nothing plays keeps the first row on screen. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aRevealRequestWithoutAPlayingRowDoesNotMoveTheList() {
        /** Inputs of a host with no track loaded. */
        val inputs = HostInputs()
        show(flatPage(count = 60), inputs)
        change { inputs.reveal.value += 1 }
        composeTestRule.onNodeWithText(titleOf(0)).assertIsDisplayed()
    }

    // What:     `@Test fun aFolderRevealScrollsToTheHeaderOfThePlayingFolder()` checks the folder reveal target.
    // Why:      The folder reveal scrolls to `folderScrollIndexFor(entries, playingIndex)`; with the wrong index the
    //           target is the first item and the request leaves the list at its top.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder reveal", () => { render({ playing: null }); setPlaying(1050); revealFolder(); expect(visible("Beta")).toBe(true); });
    // ```
    /** With a track under "Beta" playing, one folder reveal request puts the "Beta" header on screen. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aFolderRevealScrollsToTheHeaderOfThePlayingFolder() {
        /** Inputs of a host that starts without a track, then plays track 50, under "Beta". */
        val inputs = HostInputs()
        show(foldedPage(), inputs)
        change { inputs.playing.value = libraryIndexOf(50) }
        change { inputs.folderReveal.value += 1 }
        awaitRow("Beta")
        composeTestRule.onNodeWithText("Beta").assertIsDisplayed()
        composeTestRule.onNodeWithText(titleOf(0)).assertDoesNotExist()
    }

    // What:     `@Test fun aTrackChangeDoesNotReadTheLinesOfOtherRows()` checks a track change rebuilds no other row.
    // Why:      Each row reads its supporting line through a state of its own, keyed by the row's library index. A
    //           playing change that rebuilt the rows would build those states again and read every row's line again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("lines are not read again", () => { render(); const before = reads(); setPlaying(1005); expect(reads()).toEqual(before); });
    // ```
    /** Moving the playing track between two rows reads no line a second time, for any row in view. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aTrackChangeDoesNotReadTheLinesOfOtherRows() {
        /** Count of line reads per library index, kept by the measure lookup the source calls. */
        val reads = ConcurrentHashMap<Int, Int>()
        /** Source whose every call for a row's measure is recorded. */
        val lines = TrackRowLineSource.forPage(
            pageItems = pageItemsOf(count = 8, folder = "Album"),
            template = "\$mi(ext)\$",
            measureOf = { index ->
                reads.merge(index, 1, Int::plus)
                TrackMeasure(durationSec = 1.0)
            },
        )
        /** Inputs of a host that starts on the track numbered 3. */
        val inputs = HostInputs()
        inputs.playing.value = libraryIndexOf(3)
        show(flatPage(count = 8), inputs, lines)
        /** Reads made while the first frames composed the eight rows. */
        val before: Map<Int, Int> = reads.toMap()
        assertEquals((0 until 8).map { track -> libraryIndexOf(track) }.toSet(), before.keys)
        change { inputs.playing.value = libraryIndexOf(5) }
        assertEquals(before, reads.toMap())
        composeTestRule.onNodeWithText(titleOf(5)).assertIsSelected()
    }

    // What:     `@Test fun everyRowShowsItsOwnSupportingLine()` checks the line source reaches the rows.
    // Why:      The host provides the source through a local and the pane hands it to each row; a source that never
    //           arrives leaves the rows with the entries' own (empty) lines.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("lines shown", () => { render({ lines: { 1003: "mp3" } }); expect(text("mp3")).toBeDefined(); });
    // ```
    /** With a source that gives only track 3 a line, exactly one row shows that line. */
    @Test(timeout = CASE_LIMIT_MS)
    fun everyRowShowsItsOwnSupportingLine() {
        /** Source whose line shows only for the track numbered 3, by giving the other rows no path. */
        val lines = TrackRowLineSource.forPage(
            pageItems = pageItemsOf(count = 8, folder = "Album").filter { entry -> entry.index == libraryIndexOf(3) },
            template = "\$mi(ext)\$",
            measureOf = { TrackMeasure(durationSec = 1.0) },
        )
        show(flatPage(count = 8), HostInputs(), lines)
        assertEquals(1, countNodesWithText("mp3"))
    }
}
