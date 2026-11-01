// The focus ring geometry of D117: two solid rings, 2 dp then 1 dp, flush against the element and hugging its shape,
// the same pair just inside the edge of an entry flush in a clipped list (D120), and the keyboard-only rule.

// What:     `package ...core` matches the geometry's package.
// Why:      The tests call the pure functions directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Checks that the ring bounds are the element bounds grown by the widths and that every shape keeps its kind. */
class FocusRingGeometryTest {
    /** The spec of the ring the app draws, which every rule below is checked for. */
    private val specs: List<FocusRingSpec> = listOf(FIREFOX_RING_SPEC)

    /** The rings are flush: bounds grown by 2 dp to the middle edge, then by 1 dp more to the outer edge. */
    @Test
    fun boundsGrowByTheNearWidthThenByTheOuterWidth() {
        for (spec in specs) {
            /** Bands of a 200 by 56 element with 8 dp corners. */
            val bands: FocusRingBands = focusRingBands(200f, 56f, RingCorners.uniform(8f), spec)
            assertEquals(0f, bands.inner.left, 0f)
            assertEquals(0f, bands.inner.top, 0f)
            assertEquals(200f, bands.inner.right, 0f)
            assertEquals(56f, bands.inner.bottom, 0f)
            assertEquals(-2f, bands.middle.left, 0f)
            assertEquals(-2f, bands.middle.top, 0f)
            assertEquals(202f, bands.middle.right, 0f)
            assertEquals(58f, bands.middle.bottom, 0f)
            assertEquals(-3f, bands.outer.left, 0f)
            assertEquals(-3f, bands.outer.top, 0f)
            assertEquals(203f, bands.outer.right, 0f)
            assertEquals(59f, bands.outer.bottom, 0f)
        }
    }

    /** Radii grow by the widths inside them: the element's radius, plus 2 dp, plus 3 dp at the outer edge. */
    @Test
    fun radiiGrowByTheWidthsInside() {
        for (spec in specs) {
            /** Bands of a 100 by 48 element with 12 dp corners. */
            val bands: FocusRingBands = focusRingBands(100f, 48f, RingCorners.uniform(12f), spec)
            assertEquals(RingCorners.uniform(12f), bands.inner.corners)
            assertEquals(RingCorners.uniform(14f), bands.middle.corners)
            assertEquals(RingCorners.uniform(15f), bands.outer.corners)
        }
    }

    /** The room a clipping parent must leave is the sum of the gap and both widths: 3 dp. */
    @Test
    fun outsetIsThreeDp() {
        assertEquals(3f, FIREFOX_RING_SPEC.outset, 0f)
    }

    /** A pill stays a pill: every edge's radius is still half of that edge's own height. */
    @Test
    fun aPillStaysAPill() {
        for (spec in specs) {
            /** Bands of a 160 by 40 pill. */
            val bands: FocusRingBands = focusRingBands(160f, 40f, RingCorners.uniform(20f), spec)
            for (edge in listOf(bands.inner, bands.middle, bands.outer)) {
                assertEquals(edge.height / 2f, edge.corners.topLeft, 0f)
                assertEquals(edge.height / 2f, edge.corners.bottomRight, 0f)
            }
        }
    }

    /** A circle stays a circle: every edge is square and its radius is half its side. */
    @Test
    fun aCircleStaysACircle() {
        for (spec in specs) {
            /** Bands of a 48 dp circle. */
            val bands: FocusRingBands = focusRingBands(48f, 48f, RingCorners.uniform(24f), spec)
            for (edge in listOf(bands.inner, bands.middle, bands.outer)) {
                assertEquals(edge.width, edge.height, 0f)
                assertEquals(edge.width / 2f, edge.corners.topLeft, 0f)
            }
        }
    }

    /** Firefox gives a square corner a 2 dp outer radius at the near ring and 3 dp at the outer ring. */
    @Test
    fun firefoxRoundsSquareCornersByTwoDp() {
        /** Bands of a plain rectangle. */
        val bands: FocusRingBands = focusRingBands(300f, 64f, RingCorners.uniform(0f), FIREFOX_RING_SPEC)
        assertEquals(RingCorners.uniform(0f), bands.inner.corners)
        assertEquals(RingCorners.uniform(2f), bands.middle.corners)
        assertEquals(RingCorners.uniform(3f), bands.outer.corners)
    }

    /** Only the square corner of a segment changes: the rounded corners grow by the widths. */
    @Test
    fun aSegmentRoundsOnlyItsOwnSquareCorners() {
        /** Corners of an end segment: rounded on the left, square on the right. */
        val corners = RingCorners(topLeft = 24f, topRight = 0f, bottomRight = 0f, bottomLeft = 24f)
        /** Bands of that segment under Firefox. */
        val bands: FocusRingBands = focusRingBands(90f, 48f, corners, FIREFOX_RING_SPEC)
        assertEquals(RingCorners(topLeft = 26f, topRight = 2f, bottomRight = 2f, bottomLeft = 26f), bands.middle.corners)
        assertEquals(RingCorners(topLeft = 27f, topRight = 3f, bottomRight = 3f, bottomLeft = 27f), bands.outer.corners)
    }

    /** A gap, when a spec has one, moves both rings out and grows every radius by it. */
    @Test
    fun aGapPushesBothRingsOutward() {
        /** A spec with a 2 dp gap. */
        val spec = FocusRingSpec(nearWidth = 2f, outerWidth = 1f, gap = 2f, squareCornerRadius = 0f)
        /** Bands of a 100 by 40 element with 10 dp corners. */
        val bands: FocusRingBands = focusRingBands(100f, 40f, RingCorners.uniform(10f), spec)
        assertEquals(-2f, bands.inner.left, 0f)
        assertEquals(RingCorners.uniform(12f), bands.inner.corners)
        assertEquals(-4f, bands.middle.left, 0f)
        assertEquals(-5f, bands.outer.left, 0f)
        assertEquals(5f, spec.outset, 0f)
    }

    /** Inside, the outer edge of the pair is the element's own bounds, so nothing reaches outside it (D120). */
    @Test
    fun insideRingsStayWithinTheElementAndTheirOuterEdgeIsItsEdge() {
        /** Bands of a 300 by 64 row with square corners, drawn inside. */
        val bands: FocusRingBands = focusRingBandsInside(300f, 64f, RingCorners.uniform(0f), FIREFOX_RING_SPEC)
        assertEquals(0f, bands.outer.left, 0f)
        assertEquals(0f, bands.outer.top, 0f)
        assertEquals(300f, bands.outer.right, 0f)
        assertEquals(64f, bands.outer.bottom, 0f)
        for (edge in listOf(bands.inner, bands.middle, bands.outer)) {
            assertTrue(edge.left >= 0f && edge.top >= 0f && edge.right <= 300f && edge.bottom <= 64f)
        }
    }

    /** Inside, the widths are the same 1 dp outer ring and 2 dp near ring, with the outer ring on the edge. */
    @Test
    fun insideRingsKeepTheTwoAndOneDpWidths() {
        /** Bands of a 200 by 56 row, drawn inside. */
        val bands: FocusRingBands = focusRingBandsInside(200f, 56f, RingCorners.uniform(0f), FIREFOX_RING_SPEC)
        assertEquals(1f, bands.middle.left - bands.outer.left, 0f)
        assertEquals(1f, bands.outer.right - bands.middle.right, 0f)
        assertEquals(1f, bands.middle.top - bands.outer.top, 0f)
        assertEquals(1f, bands.outer.bottom - bands.middle.bottom, 0f)
        assertEquals(2f, bands.inner.left - bands.middle.left, 0f)
        assertEquals(2f, bands.middle.right - bands.inner.right, 0f)
        assertEquals(2f, bands.inner.top - bands.middle.top, 0f)
        assertEquals(2f, bands.middle.bottom - bands.inner.bottom, 0f)
    }

    /** Inside, the pair is the outside pair of the row taken in by 3 dp, so the near ring's inner edge is 3 dp in. */
    @Test
    fun insideRingsAreTheOutsideRingsOfTheElementTakenInByTheOutset() {
        /** Bands of a 120 by 48 row, drawn inside. */
        val inside: FocusRingBands = focusRingBandsInside(120f, 48f, RingCorners.uniform(0f), FIREFOX_RING_SPEC)
        /** Bands of the same row taken in by 3 dp, drawn outside, moved back by 3 dp. */
        val outside: FocusRingBands = focusRingBands(114f, 42f, RingCorners.uniform(0f), FIREFOX_RING_SPEC)
        assertEquals(3f, inside.inner.left, 0f)
        assertEquals(3f, inside.inner.top, 0f)
        assertEquals(117f, inside.inner.right, 0f)
        assertEquals(45f, inside.inner.bottom, 0f)
        assertEquals(outside.middle.width, inside.middle.width, 0f)
        assertEquals(outside.outer.height, inside.outer.height, 0f)
        assertEquals(outside.outer.corners, inside.outer.corners)
    }

    /** A square corner keeps the 2 dp near-ring radius and the 3 dp outer-ring radius inside, as outside. */
    @Test
    fun insideSquareCornersKeepTheFirefoxRadii() {
        /** Bands of a plain rectangle, drawn inside. */
        val bands: FocusRingBands = focusRingBandsInside(300f, 64f, RingCorners.uniform(0f), FIREFOX_RING_SPEC)
        assertEquals(RingCorners.uniform(0f), bands.inner.corners)
        assertEquals(RingCorners.uniform(2f), bands.middle.corners)
        assertEquals(RingCorners.uniform(3f), bands.outer.corners)
    }

    /** A circle stays a circle inside: the outer edge is the element's own circle and every edge is square. */
    @Test
    fun anInsideRingOfACircleStaysACircleOnTheElementsEdge() {
        /** Bands of a 48 dp circle, drawn inside. */
        val bands: FocusRingBands = focusRingBandsInside(48f, 48f, RingCorners.uniform(24f), FIREFOX_RING_SPEC)
        assertEquals(RingCorners.uniform(24f), bands.outer.corners)
        for (edge in listOf(bands.inner, bands.middle, bands.outer)) {
            assertEquals(edge.width, edge.height, 0f)
            assertEquals(edge.width / 2f, edge.corners.topLeft, 0f)
        }
        assertEquals(48f, bands.outer.width, 0f)
        assertEquals(46f, bands.middle.width, 0f)
        assertEquals(42f, bands.inner.width, 0f)
    }

    /** A pill stays a pill inside: the outer edge keeps the element's radius, half its height. */
    @Test
    fun anInsideRingOfAPillKeepsTheElementsRadius() {
        /** Bands of a 160 by 40 pill, drawn inside. */
        val bands: FocusRingBands = focusRingBandsInside(160f, 40f, RingCorners.uniform(20f), FIREFOX_RING_SPEC)
        assertEquals(RingCorners.uniform(20f), bands.outer.corners)
        assertEquals(RingCorners.uniform(19f), bands.middle.corners)
        assertEquals(RingCorners.uniform(17f), bands.inner.corners)
    }

    /** An element smaller than the rings gets empty inner outlines instead of negative sizes. */
    @Test
    fun anElementTooSmallForTheRingsHasNoNegativeInnerSize() {
        /** Bands of a 4 by 4 element, drawn inside. */
        val bands: FocusRingBands = focusRingBandsInside(4f, 4f, RingCorners.uniform(0f), FIREFOX_RING_SPEC)
        assertTrue(bands.inner.width >= 0f && bands.inner.height >= 0f)
    }

    /** The placement picks the matching bands: outside reaches beyond the element, inside never does. */
    @Test
    fun placementPicksTheOutsideOrTheInsideBands() {
        /** Bands of a 100 by 40 element for each placement. */
        val square = RingCorners.uniform(0f)
        val outside = focusRingBandsAt(FocusRingPlacement.OUTSIDE, 100f, 40f, square, FIREFOX_RING_SPEC)
        val inside = focusRingBandsAt(FocusRingPlacement.INSIDE, 100f, 40f, square, FIREFOX_RING_SPEC)
        assertEquals(-3f, outside.outer.left, 0f)
        assertEquals(0f, inside.outer.left, 0f)
        assertEquals(focusRingBands(100f, 40f, square, FIREFOX_RING_SPEC), outside)
        assertEquals(focusRingBandsInside(100f, 40f, square, FIREFOX_RING_SPEC), inside)
    }

    /** The ring shows only when the element has focus and the last input was the keyboard (D117). */
    @Test
    fun theRingShowsOnlyForKeyboardInput() {
        assertTrue(focusRingShows(focused = true, lastInputWasKeyboard = true))
        assertFalse(focusRingShows(focused = true, lastInputWasKeyboard = false))
        assertFalse(focusRingShows(focused = false, lastInputWasKeyboard = true))
        assertFalse(focusRingShows(focused = false, lastInputWasKeyboard = false))
    }
}
