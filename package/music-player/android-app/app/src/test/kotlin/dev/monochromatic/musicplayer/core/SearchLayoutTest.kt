// Cases for the pure Search row and status geometry (D126): the sizes that the desktop port copies, and the rule that a
// rounded row shape starts before its content without moving the content (D58) or entering the crease clear zone (E2).

// What:     `package dev.monochromatic.musicplayer.core` places these tests beside the rules they cover.
// Why:      The rules are `internal` to the app module, which its tests may reach.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class SearchLayoutTest` groups the size constants and the row inset rule.
// Why:      A changed size or margin would move the glyph or the title off the header's columns, so each is pinned.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("Search layout", () => { /* cases */ });
// ```
/** Verifies the specified sizes and the inset arithmetic of a Search result row. */
class SearchLayoutTest {
    /** The sizes the desktop port copies are exactly the specified ones. */
    @Test
    fun specifiedSizesAreExact() {
        assertEquals(40f, SEARCH_ROW_ICON_CIRCLE_DP, 0f)
        assertEquals(48f, SEARCH_ROW_ICON_SLOT_DP, 0f)
        assertEquals(8f, SEARCH_ROW_SHAPE_MARGIN_DP, 0f)
        assertEquals(21.3f, SEARCH_ROW_RADIUS_DP, 0f)
        assertEquals(72f, SEARCH_STATUS_CIRCLE_DP, 0f)
        assertEquals(32f, SEARCH_STATUS_GLYPH_DP, 0f)
    }

    /** The circle fits its slot with room, so the slot (and with it the Back column of D58) is what centers it. */
    @Test
    fun circleFitsInsideItsSlot() {
        assertTrue(SEARCH_ROW_ICON_CIRCLE_DP < SEARCH_ROW_ICON_SLOT_DP)
        assertTrue(SEARCH_STATUS_GLYPH_DP < SEARCH_STATUS_CIRCLE_DP)
    }

    /** On the cover the shape starts 8 dp from the edges and the content keeps the 16 dp insets (D58). */
    @Test
    fun coverRowShapeStartsBeforeTheContentWithoutMovingIt() {
        /** Insets of a cover row. */
        val insets = searchRowInsets(startInsetDp = 16f, endInsetDp = 16f)
        assertEquals(8f, insets.outerStartDp, 0f)
        assertEquals(8f, insets.outerEndDp, 0f)
        assertEquals(8f, insets.innerDp, 0f)
        assertEquals(16f, insets.outerStartDp + insets.innerDp, 0f)
        assertEquals(16f, insets.outerEndDp + insets.innerDp, 0f)
    }

    /** On the inner panel the content keeps the crease half plus 16 dp, and the shape stays outside the clear zone. */
    @Test
    fun innerRowShapeStaysOutsideTheCreaseClearZone() {
        /** Half the crease clear zone in dp. */
        val halfCrease = 13.5f
        /** Insets of an inner-panel row. */
        val insets = searchRowInsets(startInsetDp = halfCrease + 16f, endInsetDp = 16f)
        assertEquals(halfCrease + 16f, insets.outerStartDp + insets.innerDp, 0f)
        assertTrue(insets.outerStartDp >= halfCrease)
        assertEquals(halfCrease + 8f, insets.outerStartDp, 0f)
    }

    /** A page inset smaller than the margin shrinks the margin, so the shape never leaves the page. */
    @Test
    fun smallInsetShrinksTheMargin() {
        /** Insets of a row whose start inset is only 3 dp. */
        val insets = searchRowInsets(startInsetDp = 3f, endInsetDp = 16f)
        assertEquals(0f, insets.outerStartDp, 0f)
        assertEquals(3f, insets.innerDp, 0f)
        assertEquals(13f, insets.outerEndDp, 0f)
        assertEquals(16f, insets.outerEndDp + insets.innerDp, 0f)
    }

    /** A zero inset gives a zero margin, and the content stays where the inset puts it. */
    @Test
    fun zeroInsetGivesZeroMargin() {
        /** Insets of a row with no start inset. */
        val insets = searchRowInsets(startInsetDp = 0f, endInsetDp = 0f)
        assertEquals(SearchRowInsets(outerStartDp = 0f, outerEndDp = 0f, innerDp = 0f), insets)
    }
}
