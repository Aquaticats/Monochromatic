// Pure cases of the flush track row's box and text insets (D58, D110, D120, D130), from the clearance inputs of the
// cover display, the right pane of the creased inner display, and the left pane of it.

// What:     `package dev.monochromatic.musicplayer.core` places this test beside the rule it checks.
// Why:      The test reaches the internal pure function in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each inset is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class TrackRowGeometryTest` groups the cases of the flush row's box and text insets.
// Why:      The rule decides where the fill, the ring and the text of every track row begin on every display, so each
//           pane position is pinned with its own clearance input.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("trackRowColumns", () => { ... });
// ```
/** Verifies that the box spans the pane and the text takes the heading column, with a ring gap that never shrinks. */
class TrackRowGeometryTest {
    /** Density of the Fold inner display at 390 dpi, which is 2.4375 physical pixels per dp. */
    private val foldDensity: Float = 2.4375f

    /** Depth of the focus ring inside the box edge: the near ring plus the outer ring. */
    private val ringDepthDp: Float = FIREFOX_RING_SPEC.outset

    /** Row insets in a pane that has the given clearance. */
    private fun columnsIn(clearance: PaneClearance): TrackRowColumns =
        columnsFor(startClearanceDp = clearance.startDp, endClearanceDp = clearance.endDp)

    /** Row insets for the given clearances with the 16 dp ordinary inset and the real ring depth and gap. */
    private fun columnsFor(startClearanceDp: Float, endClearanceDp: Float): TrackRowColumns = trackRowColumns(
        startClearanceDp = startClearanceDp,
        endClearanceDp = endClearanceDp,
        ordinaryInsetDp = 16f,
        ringDepthDp = ringDepthDp,
        ringTextGapDp = TRACK_ROW_RING_TEXT_GAP_DP,
    )

    // What:     `coverRowKeepsTheOrdinaryInsetAndAFullWidthBox` checks the display without a crease.
    // Why:      With no clearance the text stays 16 dp from both edges and the box fills the pane, as committed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("cover row", () => { expect(trackRowColumns({ start: 0, end: 0 })).toEqual({ 0, 0, 16, 16 }); });
    // ```
    /** With zero clearance on both sides the box starts and ends at the edges and the text insets are 16 dp. */
    @Test
    fun coverRowKeepsTheOrdinaryInsetAndAFullWidthBox() {
        /** Insets of a row in a pane next to no crease. */
        val columns: TrackRowColumns = columnsFor(startClearanceDp = 0f, endClearanceDp = 0f)
        assertEquals(TrackRowColumns(boxStartDp = 0f, boxEndDp = 0f, textStartDp = 16f, textEndDp = 16f), columns)
    }

    // What:     `rightPaneRowTextStartsAtTheHeadingColumnAndTheBoxAtTheEdge` checks the pane right of the crease.
    // Why:      The floor clearance there is about 22.64 dp; the heading starts at `paneTextInsetDp` of that, so
    //           the row text starts at the same dp, and the box starts at the pane edge, not at the clearance.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("right pane row", () => { expect(row.textStartDp).toBe(headingStartDp); expect(row.boxStartDp).toBe(0); });
    // ```
    /** Beside the crease the text starts where the heading starts, the box at 0, and the ring keeps its gap. */
    @Test
    fun rightPaneRowTextStartsAtTheHeadingColumnAndTheBoxAtTheEdge() {
        /** Clearance of the right pane on the inner display for a crease that reports no width. */
        val clearance: PaneClearance = rightPaneClearance(creaseWidthPx = 0f, density = foldDensity)
        /** Insets of a row in that pane. */
        val columns: TrackRowColumns = columnsIn(clearance)
        /** Start of the subfolder headings and the title bar in that pane, the column the text must share. */
        val headingStartDp: Float = paneTextInsetDp(ordinaryDp = 16f, clearanceDp = clearance.startDp)
        assertEquals(22.64f, headingStartDp, 0.01f)
        assertEquals(headingStartDp, columns.textStartDp, 0f)
        assertEquals(0f, columns.boxStartDp, 0f)
        assertEquals(0f, columns.boxEndDp, 0f)
        assertEquals(16f, columns.textEndDp, 0f)
        assertTrue(columns.textStartDp - ringDepthDp >= TRACK_ROW_RING_TEXT_GAP_DP)
    }

    // What:     `leftPaneRowTextEndsAtTheHeadingColumnAndTheBoxAtTheEdge` checks the pane left of the crease.
    // Why:      The end side is the mirror: the clearance is at the pane's end and the box still reaches the edge.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("left pane row", () => { expect(row.textEndDp).toBe(headingEndDp); expect(row.boxEndDp).toBe(0); });
    // ```
    /** At the end side the text ends where the heading ends, the box at 0, and the ring keeps its gap. */
    @Test
    fun leftPaneRowTextEndsAtTheHeadingColumnAndTheBoxAtTheEdge() {
        /** Clearance of the left pane on the inner display for a crease that reports no width. */
        val clearance: PaneClearance = leftPaneClearance(creaseWidthPx = 0f, density = foldDensity)
        /** Insets of a row in that pane. */
        val columns: TrackRowColumns = columnsIn(clearance)
        /** End of the headings in that pane. */
        val headingEndDp: Float = paneTextInsetDp(ordinaryDp = 16f, clearanceDp = clearance.endDp)
        assertEquals(headingEndDp, columns.textEndDp, 0f)
        assertEquals(0f, columns.boxEndDp, 0f)
        assertEquals(0f, columns.boxStartDp, 0f)
        assertEquals(16f, columns.textStartDp, 0f)
        assertTrue(columns.textEndDp - ringDepthDp >= TRACK_ROW_RING_TEXT_GAP_DP)
    }

    // What:     `aWiderCreaseMovesTheTextAndNeverTheBox` checks a crease wider than the floor.
    // Why:      A 400 px crease gives 200 px to each pane; the text moves with it, the box does not.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("wide crease", () => {
    //   expect(row.textStartDp).toBeCloseTo(200 / density);
    //   expect(row.boxStartDp).toBe(0);
    // });
    // ```
    /** A wide crease moves only the text start. */
    @Test
    fun aWiderCreaseMovesTheTextAndNeverTheBox() {
        /** Clearance of the right pane beside a 400 px crease. */
        val clearance: PaneClearance = rightPaneClearance(creaseWidthPx = 400f, density = foldDensity)
        /** Insets of a row in that pane. */
        val columns: TrackRowColumns = columnsIn(clearance)
        assertEquals(200f / foldDensity, columns.textStartDp, 0.001f)
        assertEquals(0f, columns.boxStartDp, 0f)
    }

    // What:     `aClearanceBelowTheOrdinaryInsetLeavesTheOrdinaryColumn` checks a small clearance.
    // Why:      A 10 dp clearance is less than the 16 dp ordinary inset, which the heading keeps, so the row text keeps
    //           it too; the old rule put the text at 10 plus the ring floor of 11, which is 21 dp.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("small clearance", () => { expect(row.textStartDp).toBe(16); });
    // ```
    /** The text column is the larger of the ordinary inset and the clearance, not their sum with the ring floor. */
    @Test
    fun aClearanceBelowTheOrdinaryInsetLeavesTheOrdinaryColumn() {
        /** Insets of a row beside a 10 dp clearance. */
        val columns: TrackRowColumns = columnsFor(startClearanceDp = 10f, endClearanceDp = 10f)
        assertEquals(16f, columns.textStartDp, 0f)
        assertEquals(16f, columns.textEndDp, 0f)
    }

    // What:     `theRingFloorHoldsWhenTheOrdinaryInsetIsSmall` checks the floor of ring depth plus gap.
    // Why:      An ordinary inset under that floor would put the text inside the ring's gap; the floor lifts the
    //           text to the ring depth plus the least gap, 11 dp, so the gap equals the minimum and not less.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("ring floor", () => { expect(row.textStartDp - ringDepth).toBe(MIN_GAP); });
    // ```
    /** With a 4 dp ordinary inset and no crease the text starts at the ring floor, a gap of 8 dp from the ring. */
    @Test
    fun theRingFloorHoldsWhenTheOrdinaryInsetIsSmall() {
        /** Insets of a row whose ordinary inset is smaller than the ring floor. */
        val columns: TrackRowColumns = trackRowColumns(
            startClearanceDp = 0f,
            endClearanceDp = 0f,
            ordinaryInsetDp = 4f,
            ringDepthDp = ringDepthDp,
            ringTextGapDp = TRACK_ROW_RING_TEXT_GAP_DP,
        )
        assertEquals(TRACK_ROW_RING_TEXT_GAP_DP, columns.textStartDp - ringDepthDp, 0f)
        assertEquals(TRACK_ROW_RING_TEXT_GAP_DP, columns.textEndDp - ringDepthDp, 0f)
        assertEquals(11f, columns.textStartDp, 0f)
    }
}
