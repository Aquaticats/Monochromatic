// Wiring cases of the flush track rows beside the crease (D58, D110, D120, D130), rendered for real under Robolectric
// with native graphics: the pane title, the subfolder heading and the row text share one column, the row's box spans
// the pane, and the keyboard ring lies inside that box with its gap to the text.

// What:     `package ...wiring` groups the Robolectric Compose tests, which share the fixtures of this package.
// Why:      The cases reach `trackListPane`, `unfoldedTrackHeader` and the composition locals of the app module.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.input.InputMode
import androidx.compose.ui.input.InputModeManager
import androidx.compose.ui.platform.LocalInputModeManager
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.getUnclippedBoundsInRoot
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.requestFocus
import androidx.compose.ui.unit.DpRect
import dev.monochromatic.musicplayer.LocalPlayingIndex
import dev.monochromatic.musicplayer.core.PaneClearance
import dev.monochromatic.musicplayer.core.TrackListEntry
import dev.monochromatic.musicplayer.core.TrackRowModel
import dev.monochromatic.musicplayer.core.leftPaneClearance
import dev.monochromatic.musicplayer.core.rightPaneClearance
import dev.monochromatic.musicplayer.paneClearanceProvider
import dev.monochromatic.musicplayer.trackListPane
import dev.monochromatic.musicplayer.unfoldedTrackHeader
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** Qualifier of a 400 dp wide pane at one physical pixel per dp, so a pixel is a dp. */
private const val PANE_QUALIFIER = "w400dp-h800dp-mdpi"

/** Width of the pane in dp, from the qualifier above. */
private const val PANE_WIDTH_DP: Float = 400f

/** Slack in dp for two sizes that must be equal; layout rounds each edge to a pixel, and a pixel is a dp here. */
private const val EDGE_SLACK_DP: Float = 0.6f

/** Title of the pane header, the invented folder whose tracks the pane lists. */
private const val PANE_TITLE = "Aster Vale"

/** Name of the invented subfolder heading, long enough to wrap across the pane so its end edge is the pane's. */
private const val LONG_HEADING = "Salt Garden with a very long invented name that wraps across the whole pane width"

/** Title of the invented long row, long enough to wrap across the pane so its end edge is the pane's. */
private const val LONG_ROW_TITLE = "A very long invented track title that goes on and on and wraps across the pane"

/** Library index of the playing row, the second row of the page. */
private const val PLAYING_INDEX: Int = 1001

/** Density of the Fold inner display at 390 dpi: 2.4375 physical pixels per dp, which sets the floor clearance. */
private const val INNER_DENSITY: Float = 2.4375f

/** Depth in dp of the two focus rings inside the box edge: the 2 dp near ring and the 1 dp outer ring. */
private const val RING_DEPTH_DP: Float = 3f

/** Least gap in dp from the ring's inner edge to the text (D120). */
private const val MIN_RING_TEXT_GAP_DP: Float = 8f

// What:     `class RowAlignmentWiringTest` groups the cases of the flush rows' box and text columns.
// Why:      The geometry rule is pure and tested on the JVM; these cases pin that the pane, the header and the rows
//           really use it, through the bounds and the pixels the screen shows.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("row alignment wiring", () => { /* cases rendered with a test renderer and read back as pixels */ });
// ```
/** Renders the right pane of the unfolded player with a pane clearance and asserts columns, fill and ring. */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(qualifiers = PANE_QUALIFIER)
class RowAlignmentWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Input mode manager of the composition, which the ring case switches to keyboard mode. */
    private lateinit var inputModes: InputModeManager

    /** Scheme with four loud roles, so a pixel names the layer that painted it. */
    private val scheme = lightColorScheme(
        primary = Color(0xFFFF0000),
        onPrimary = Color(0xFF0000FF),
        surface = Color(0xFFFFFFFF),
        surfaceContainerLow = Color(0xFF00FF00),
        onSurface = Color(0xFF000000),
    )

    /** Row model of the invented track [number], titled [title], with no supporting line. */
    private fun row(number: Int, title: String): TrackListEntry = TrackListEntry.RowEntry(
        TrackRowModel(
            indexInLibrary = libraryIndexOf(number),
            title = title,
            supportingLine = null,
            isCurrent = false,
        ),
    )

    /** Entries of the pane: two rows (the second plays), a long subfolder heading, a long row, and a last row. */
    private fun entries(): List<TrackListEntry> = listOf(
        row(0, titleOf(0)),
        row(1, titleOf(1)),
        TrackListEntry.HeaderEntry(name = LONG_HEADING, trackCount = 2),
        row(2, LONG_ROW_TITLE),
        row(3, titleOf(3)),
    )

    /** Shows the pane under [clearance] with the header and the list, and waits for its first frames. */
    private fun show(clearance: PaneClearance) {
        composeTestRule.setContent {
            inputModes = LocalInputModeManager.current
            MaterialTheme(colorScheme = scheme) {
                CompositionLocalProvider(LocalPlayingIndex provides PLAYING_INDEX) {
                    paneClearanceProvider(clearance = clearance) {
                        Column(modifier = Modifier.fillMaxSize()) {
                            unfoldedTrackHeader(folderTitle = PANE_TITLE, onSettings = { }, onSearch = { })
                            trackListPane(
                                entries = entries(),
                                onPlay = { },
                                onOpenMenu = null,
                                modifier = Modifier.weight(1f),
                            )
                        }
                    }
                }
            }
        }
        composeTestRule.waitForIdle()
    }

    /** Bounds in the root of the text node with exactly [text], read from the unmerged tree. */
    private fun textBounds(text: String): DpRect =
        composeTestRule.onNode(hasText(text), useUnmergedTree = true).getUnclippedBoundsInRoot()

    /** The clickable row that holds [title], whose bounds are the row's box. */
    private fun rowBox(title: String): SemanticsNodeInteraction =
        composeTestRule.onNode(hasClickAction().and(hasText(title)))

    /** Moves keyboard focus into the list, which starts the ring on the playing row. */
    private fun focusListByKeyboard() {
        composeTestRule.runOnUiThread { inputModes.requestInputMode(InputMode.Keyboard) }
        composeTestRule.waitForIdle()
        composeTestRule.onNode(hasScrollAction()).requestFocus()
        composeTestRule.waitForIdle()
    }

    /**
     * Asserts the title bar, the subfolder heading and the row text start at one x, which is the pane's text inset
     * for [clearance], and that every row's box starts at the pane edge.
     */
    private fun assertOneStartColumn(clearance: PaneClearance) {
        show(clearance)
        /** Start of the text for this clearance: the larger of the ordinary 16 dp and the clearance. */
        val column: Float = maxOf(16f, clearance.startDp)
        assertEquals("title bar", column, textBounds(PANE_TITLE).left.value, EDGE_SLACK_DP)
        assertEquals("subfolder heading", column, textBounds(LONG_HEADING).left.value, EDGE_SLACK_DP)
        assertEquals("row title", column, textBounds(titleOf(0)).left.value, EDGE_SLACK_DP)
        assertEquals("playing row title", column, textBounds(titleOf(1)).left.value, EDGE_SLACK_DP)
        assertEquals("row after the heading", column, textBounds(titleOf(3)).left.value, EDGE_SLACK_DP)
        assertEquals("row box start", 0f, rowBox(titleOf(1)).getUnclippedBoundsInRoot().left.value, EDGE_SLACK_DP)
    }

    /** Asserts the row text ends where the heading text ends and every row's box ends at the pane edge. */
    private fun assertOneEndColumn(clearance: PaneClearance) {
        show(clearance)
        /** End of the text for this clearance: the pane width less the larger of 16 dp and the clearance. */
        val columnEnd: Float = PANE_WIDTH_DP - maxOf(16f, clearance.endDp)
        assertEquals("subfolder heading", columnEnd, textBounds(LONG_HEADING).right.value, EDGE_SLACK_DP)
        assertEquals("long row title", columnEnd, textBounds(LONG_ROW_TITLE).right.value, EDGE_SLACK_DP)
        assertEquals(
            "row box end",
            PANE_WIDTH_DP,
            rowBox(titleOf(1)).getUnclippedBoundsInRoot().right.value,
            EDGE_SLACK_DP,
        )
    }

    /** Vertical middle of the playing row in root pixels, which are dp at this density. */
    private fun playingRowMiddle(): Int {
        /** Bounds of the playing row's box in the root. */
        val bounds: DpRect = rowBox(titleOf(1)).getUnclippedBoundsInRoot()
        return ((bounds.top.value + bounds.bottom.value) / 2f).toInt()
    }

    /** ARGB of the root pixel at ([x], [y]), counted from the pane's own edge, so a box that starts late shows. */
    private fun rootPixel(x: Int, y: Int): Int =
        composeTestRule.onRoot().captureToImage().toPixelMap()[x, y].toArgb()

    // What:     `coverRowsKeepTheOrdinaryColumn` checks the display without a crease.
    // Why:      With no clearance the title bar, the heading and the rows all start at 16 dp, as committed; the case
    //           passes before the change and guards that the cover stays as it was.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("cover columns", () => { expectOneColumn({ start: 0, end: 0 }, 16); });
    // ```
    /** Without a crease the three texts start at 16 dp and the box starts at the pane's start. */
    @Test(timeout = CASE_LIMIT_MS)
    fun coverRowsKeepTheOrdinaryColumn() {
        assertOneStartColumn(PaneClearance(startDp = 0f, endDp = 0f))
    }

    // What:     `coverRowsKeepTheOrdinaryEndColumn` checks the end side of the display without a crease.
    // Why:      The heading and the rows end 16 dp before the pane's end and the box reaches it, as committed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("cover end columns", () => { expectOneEndColumn({ start: 0, end: 0 }); });
    // ```
    /** Without a crease the texts end 16 dp before the pane's end. */
    @Test(timeout = CASE_LIMIT_MS)
    fun coverRowsKeepTheOrdinaryEndColumn() {
        assertOneEndColumn(PaneClearance(startDp = 0f, endDp = 0f))
    }

    // What:     `innerRightPaneRowsShareTheHeadingColumn` checks the pane right of the crease at the floor clearance.
    // Why:      The clearance there is 22.64 dp at the inner display's density; the title bar and the heading start
    //           there, and the row text must too, where it used to start 11 dp later (D130).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("inner right pane columns", () => { expectOneColumn(rightPaneClearance(0, 2.4375), 22.64); });
    // ```
    /** With the floor clearance of the inner display the row text starts at the title's x. */
    @Test(timeout = CASE_LIMIT_MS)
    fun innerRightPaneRowsShareTheHeadingColumn() {
        assertOneStartColumn(rightPaneClearance(creaseWidthPx = 0f, density = INNER_DENSITY))
    }

    // What:     `aClearanceBelowTheOrdinaryInsetKeepsTheOrdinaryColumn` checks a clearance of 10 dp.
    // Why:      The heading keeps its 16 dp; the old row rule put the text 11 dp after the clearance, at 21 dp.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("small clearance columns", () => { expectOneColumn({ start: 10, end: 0 }, 16); });
    // ```
    /** A 10 dp clearance leaves every start at 16 dp. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aClearanceBelowTheOrdinaryInsetKeepsTheOrdinaryColumn() {
        assertOneStartColumn(PaneClearance(startDp = 10f, endDp = 0f))
    }

    // What:     `aWideCreaseRowsShareTheHeadingColumn` checks a clearance of 60 dp.
    // Why:      A crease wider than the floor moves the heading further; the rows must follow it exactly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("wide crease columns", () => { expectOneColumn({ start: 60, end: 0 }, 60); });
    // ```
    /** A 60 dp clearance puts the title bar, the heading and the row text at 60 dp. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aWideCreaseRowsShareTheHeadingColumn() {
        assertOneStartColumn(PaneClearance(startDp = 60f, endDp = 0f))
    }

    // What:     `leftPaneRowsEndAtTheHeadingColumn` checks the end side, the pane left of the crease.
    // Why:      The end clearance is 22.64 dp there; heading and row text end at the same x, and the box reaches the
    //           pane's end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("left pane end column", () => { expectOneEndColumn(leftPaneClearance(0, 2.4375)); });
    // ```
    /** With the floor clearance at the end, the long heading and the long row end 22.64 dp before the pane's end. */
    @Test(timeout = CASE_LIMIT_MS)
    fun leftPaneRowsEndAtTheHeadingColumn() {
        assertOneEndColumn(leftPaneClearance(creaseWidthPx = 0f, density = INNER_DENSITY))
    }

    // What:     `theCurrentRowFillReachesThePaneEdgeBesideTheCrease` checks the fill by pixels.
    // Why:      A fill is not text, so the crease does not move it (D130); the playing row's fill starts at pane
    //           pixel 0 whatever the clearance, as on the cover.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("fill at the edge", () => { expect(pixel(playingRow, 0, mid)).toBe(fillColor); });
    // ```
    /** With the floor clearance the first pixels of the playing row, counted from the pane edge, are the fill color. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theCurrentRowFillReachesThePaneEdgeBesideTheCrease() {
        show(rightPaneClearance(creaseWidthPx = 0f, density = INNER_DENSITY))
        /** Vertical middle of the playing row. */
        val middle: Int = playingRowMiddle()
        assertEquals("first pixel", scheme.surfaceContainerLow.toArgb(), rootPixel(0, middle))
        assertEquals("just before the clearance", scheme.surfaceContainerLow.toArgb(), rootPixel(20, middle))
    }

    // What:     `theKeyboardRingLiesInsideTheBoxAndKeepsItsGapToTheText` checks the ring by pixels and bounds.
    // Why:      D120 draws the ring just inside the box edge; with the box at the pane edge the ring is at pixel 0,
    //           and the text, at the heading column, keeps at least 8 dp from the ring's inner edge.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("ring inside the box", () => {
    //   expect(pixel(0)).toBe(onPrimary);
    //   expect(textLeft - 3).toBeGreaterThan(8);
    // });
    // ```
    /** After Tab-like keyboard focus the first pixel is the outer ring, then the near ring, then the fill. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theKeyboardRingLiesInsideTheBoxAndKeepsItsGapToTheText() {
        show(rightPaneClearance(creaseWidthPx = 0f, density = INNER_DENSITY))
        focusListByKeyboard()
        /** Vertical middle of the playing row. */
        val middle: Int = playingRowMiddle()
        assertEquals("outer ring at the edge", scheme.onPrimary.toArgb(), rootPixel(0, middle))
        assertEquals("near ring just inside", scheme.primary.toArgb(), rootPixel(1, middle))
        assertEquals("near ring, second pixel", scheme.primary.toArgb(), rootPixel(2, middle))
        assertEquals("fill inside the rings", scheme.surfaceContainerLow.toArgb(), rootPixel(3, middle))
        /** Distance from the ring's inner edge (3 dp) to the start of the title. */
        val gap: Float = textBounds(titleOf(1)).left.value - RING_DEPTH_DP
        assertTrue("gap from ring to text is $gap dp", gap >= MIN_RING_TEXT_GAP_DP)
    }
}
