// What:     `package dev.monochromatic.musicplayer.core` places this test beside the pane clearance rule.
// Why:      The test reaches the internal pure functions in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each clearance is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

// What:     `class PaneClearanceTest` groups the tests of the crease clearance of the player's two panes.
// Why:      The rule decides where text may start and end on the unfolded screen, so each case is pinned.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("paneClearance", () => { ... });
// ```
/** Verifies the crease clearance that the unfolded player's panes give their text. */
class PaneClearanceTest {
    /** Density of the Fold inner display at 390 dpi, which is 2.4375 physical pixels per dp. */
    private val foldDensity: Float = 2.4375f

    // What:     `leftPaneKeepsHalfTheFloorAtItsEnd` checks the physical 7.5 mm floor on the inner display.
    // Why:      A device that reports a zero-width crease still keeps 7.5 mm, half of it in each pane, so the left
    //           pane keeps 55 px (about 22.6 dp) before the shared edge and nothing at its start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("left pane keeps half the floor at its end", () => { ... });
    // ```
    /** Confirms the left pane clears half of the 7.5 mm floor at its end edge when the crease reports no width. */
    @Test
    fun leftPaneKeepsHalfTheFloorAtItsEnd() {
        /** Clearance of the left pane for a crease that reports zero width. */
        val clearance: PaneClearance = leftPaneClearance(creaseWidthPx = 0f, density = foldDensity)
        assertEquals(0f, clearance.startDp, 0f)
        assertEquals(CREASE_FLOOR_WIDTH_PX / 2f / foldDensity, clearance.endDp, 0.001f)
        assertEquals(22.64f, clearance.endDp, 0.01f)
    }

    // What:     `rightPaneKeepsHalfTheFloorAtItsStart` checks the mirror case for the right pane.
    // Why:      The right pane's text starts 55 px after the shared edge, which is exactly x = 1093 on 2076 px.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("right pane keeps half the floor at its start", () => { ... });
    // ```
    /** Confirms the right pane clears half of the 7.5 mm floor at its start edge. */
    @Test
    fun rightPaneKeepsHalfTheFloorAtItsStart() {
        /** Clearance of the right pane for a crease that reports zero width. */
        val clearance: PaneClearance = rightPaneClearance(creaseWidthPx = 0f, density = foldDensity)
        assertEquals(0f, clearance.endDp, 0f)
        assertEquals(22.64f, clearance.startDp, 0.01f)
        assertEquals(1038f + 55.18f, 1038f + clearance.startDp * foldDensity, 0.1f)
    }

    // What:     `widerReportedCreaseWins` checks that a crease wider than the floor sets the clearance.
    // Why:      E2 asks for `max(7.5 mm, crease width)`, so a 200 px crease gives 100 px to each side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a wider crease wins", () => { ... });
    // ```
    /** Confirms a 200 px crease gives each pane 100 px of clearance. */
    @Test
    fun widerReportedCreaseWins() {
        /** Clearance of the left pane for a 200 px crease. */
        val clearance: PaneClearance = leftPaneClearance(creaseWidthPx = 200f, density = foldDensity)
        assertEquals(100f / foldDensity, clearance.endDp, 0.001f)
    }

    // What:     `noCreaseKeepsNothing` checks a window without a crease.
    // Why:      D110 adds no clearance on a desktop, a tablet or the folded cover, where the API reports no crease.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no crease means no clearance", () => { ... });
    // ```
    /** Confirms both panes keep nothing extra when the window has no crease. */
    @Test
    fun noCreaseKeepsNothing() {
        assertEquals(NO_PANE_CLEARANCE, leftPaneClearance(creaseWidthPx = null, density = foldDensity))
        assertEquals(NO_PANE_CLEARANCE, rightPaneClearance(creaseWidthPx = null, density = foldDensity))
    }

    // What:     `clearanceIsPhysicalNotAFixedDp` checks the clearance at the largest display size.
    // Why:      E2 forbids a stored dp value, so a denser display (the largest display size) needs fewer dp for the
    //           same millimeters, and the pixel clearance stays the same.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("the clearance is the same in pixels at every density", () => { ... });
    // ```
    /** Confirms the pixel clearance does not change with the display density. */
    @Test
    fun clearanceIsPhysicalNotAFixedDp() {
        /** Density of the largest display size step, which is higher than the default. */
        val denseDensity = 3.6f
        /** Clearance of the left pane at the default density. */
        val atDefault: PaneClearance = leftPaneClearance(creaseWidthPx = 0f, density = foldDensity)
        /** Clearance of the left pane at the dense display size. */
        val atDense: PaneClearance = leftPaneClearance(creaseWidthPx = 0f, density = denseDensity)
        assertEquals(atDefault.endDp * foldDensity, atDense.endDp * denseDensity, 0.01f)
    }

    // What:     `textInsetKeepsTheLargerOfOrdinaryAndClearance` checks the inset rule.
    // Why:      A 16 dp ordinary inset wins where the crease is far away, and the clearance wins beside the crease.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("the larger inset wins", () => { ... });
    // ```
    /** Confirms the text inset is the larger of the ordinary inset and the crease clearance. */
    @Test
    fun textInsetKeepsTheLargerOfOrdinaryAndClearance() {
        assertEquals(16f, paneTextInsetDp(ordinaryDp = 16f, clearanceDp = 0f), 0f)
        assertEquals(22.64f, paneTextInsetDp(ordinaryDp = 16f, clearanceDp = 22.64f), 0f)
        assertEquals(30f, paneTextInsetDp(ordinaryDp = 30f, clearanceDp = 22.64f), 0f)
    }

    // What:     `nonPositiveDensityThrows` checks the guard of both functions.
    // Why:      A zero or negative density cannot convert pixels to dp, so the rule must refuse it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a non-positive density throws", () => { ... });
    // ```
    /** Confirms a non-positive density is refused by both pane rules. */
    @Test
    fun nonPositiveDensityThrows() {
        assertThrows(PaneClearanceException::class.java) { leftPaneClearance(creaseWidthPx = 0f, density = 0f) }
        assertThrows(PaneClearanceException::class.java) { rightPaneClearance(creaseWidthPx = 0f, density = -1f) }
    }
}
