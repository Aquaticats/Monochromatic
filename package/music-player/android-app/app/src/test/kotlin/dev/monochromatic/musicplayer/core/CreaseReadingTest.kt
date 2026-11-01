// What:     `package dev.monochromatic.musicplayer.core` places this test beside the crease rule.
// Why:      The test reaches the internal pure function in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies how the folding features of a window layout become the crease the layout dodges (D110, E2). */
class CreaseReadingTest {
    /** A flat unfolded Fold as the platform reports it: vertical, not separating, no occlusion, a zero-width line. */
    private val flatFold =
        FoldReading(vertical = true, separating = false, occluding = false, leftPx = 1038, rightPx = 1038)

    /** Confirms a vertical feature that is not separating still counts as a crease, with its reported width. */
    @Test
    fun aFlatVerticalFoldIsACreaseEvenWhenNotSeparating() {
        val state: CreaseState = resolveCrease(listOf(flatFold), hingeSensorPresent = true)
        assertTrue(state.present)
        assertEquals(0f, state.reportedWidthPx, 0f)
        assertEquals(CreaseSource.FOLDING_FEATURE, state.source)
        assertEquals(0f, state.widthPxOrNull!!, 0f)
    }

    /** Confirms an occluding hinge reports the width of its bounds. */
    @Test
    fun anOccludingHingeReportsItsBoundsWidth() {
        val hinge = FoldReading(vertical = true, separating = true, occluding = true, leftPx = 980, rightPx = 1096)
        val state: CreaseState = resolveCrease(listOf(hinge), hingeSensorPresent = false)
        assertTrue(state.present)
        assertEquals(116f, state.reportedWidthPx, 0f)
    }

    /** Confirms the widest vertical fold wins when a layout lists several. */
    @Test
    fun theWidestVerticalFoldIsTheCrease() {
        val narrow = flatFold
        val wide = FoldReading(vertical = true, separating = true, occluding = true, leftPx = 900, rightPx = 1000)
        assertEquals(100f, resolveCrease(listOf(narrow, wide), hingeSensorPresent = false).reportedWidthPx, 0f)
    }

    /** Confirms a horizontal fold (tabletop posture) is not a vertical crease. */
    @Test
    fun aHorizontalFoldIsNotACrease() {
        val tabletop = FoldReading(vertical = false, separating = true, occluding = false, leftPx = 0, rightPx = 2076)
        val state: CreaseState = resolveCrease(listOf(tabletop), hingeSensorPresent = true)
        assertFalse(state.present)
        assertNull(state.widthPxOrNull)
    }

    /** Confirms a layout report with no features means no crease even on a device that declares a hinge sensor. */
    @Test
    fun anEmptyReportMeansNoCreaseDespiteTheHingeSensor() {
        val state: CreaseState = resolveCrease(emptyList(), hingeSensorPresent = true)
        assertFalse(state.present)
        assertEquals(CreaseSource.FOLDING_FEATURE, state.source)
        assertNull(state.fold)
    }

    /** Confirms the hinge-sensor fallback applies only before the first report. */
    @Test
    fun theHingeSensorStandsInOnlyBeforeTheFirstReport() {
        val withHinge: CreaseState = resolveCrease(null, hingeSensorPresent = true)
        assertTrue(withHinge.present)
        assertEquals(0f, withHinge.reportedWidthPx, 0f)
        assertEquals(CreaseSource.HINGE_SENSOR, withHinge.source)
        assertFalse(resolveCrease(null, hingeSensorPresent = false).present)
    }

    /** Confirms the fold width is the horizontal extent of its bounds. */
    @Test
    fun theWidthIsTheHorizontalExtent() {
        assertEquals(50, FoldReading(true, true, true, 10, 60).widthPx)
    }
}
