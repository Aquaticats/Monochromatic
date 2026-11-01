// Unit tests for the track menu placement rules (D7). Each case names the window position it checks,
// so a failure points at the rule that broke.

// What:     This package matches the placement rules' package, so the tests reach the functions directly.
// Why:      The rules are plain functions over integers, so they need no device.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit assertions and the test annotation for the placement rules.
// Why:      Each expected offset is checked exactly, in pixels.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class MenuAnchorTest` groups the placement cases in one file.
// Why:      The cases cover the vertical choices, the horizontal clamps, and the inset, which are one rule.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("menuAnchorOffset", () => { /* ... */ });
// ```
/** Verifies where the track menu lands for each row position and window width. */
class MenuAnchorTest {
    // What:     `private val rowHeight: Int` is the pixel height of one row used by the cases.
    // Why:      A shared height keeps each case about the position, not the row size.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rowHeight = 144;
    // ```
    /** Height of the row in pixels that every case anchors to. */
    private val rowHeight: Int = 144

    // What:     `private val menuHeight: Int` is the pixel height of the menu used by the cases.
    // Why:      The height decides when the menu overflows the bottom, so it is fixed across the cases.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const menuHeight = 600;
    // ```
    /** Height of the menu in pixels for every case. */
    private val menuHeight: Int = 600

    // What:     `private val menuWidth: Int` is the pixel width of the menu used by the cases.
    // Why:      The width decides the right-edge clamp, so it is fixed across the cases.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const menuWidth = 840;
    // ```
    /** Width of the menu in pixels for every case. */
    private val menuWidth: Int = 840

    // What:     `private fun rowAt(top: Int, left: Int, right: Int): MenuAnchorRect` builds a row at a position.
    // Why:      Every case states only the edges it varies, and the row height stays fixed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rowAt = (top, left, right) => ({ left, top, right, bottom: top + rowHeight });
    // ```
    /** Builds a row whose top edge and horizontal edges are given, with the shared row height. */
    private fun rowAt(top: Int, left: Int, right: Int): MenuAnchorRect =
        MenuAnchorRect(left = left, top = top, right = right, bottom = top + rowHeight)

    // What:     `private fun placeIn(anchor: MenuAnchorRect, windowWidth: Int, windowHeight: Int): MenuAnchorOffset`
    //           runs the placement for one row in one window.
    // Why:      Each case names only the window and the row, so the request is built in one place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const placeIn = (anchor, windowWidth, windowHeight) => menuAnchorOffset({ ... });
    // ```
    /** Places the shared menu for one row inside a window of the given size, with no start inset. */
    private fun placeIn(anchor: MenuAnchorRect, windowWidth: Int, windowHeight: Int): MenuAnchorOffset =
        menuAnchorOffset(
            MenuAnchorRequest(
                anchor = anchor,
                popupWidth = menuWidth,
                popupHeight = menuHeight,
                windowWidth = windowWidth,
                windowHeight = windowHeight,
                startInset = 0,
            ),
        )

    // What:     `@Test fun opensBelowRowNearTop()` checks a row near the top of a tall window.
    // Why:      With room below the row, the menu must sit directly under it at the row's left edge.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("below near top", () => expect(place(row(200, 40, 1000), 1080, 2400)).toEqual({ x: 40, y: 344 }));
    // ```
    /** A row near the top opens the menu directly below it. */
    @Test
    fun opensBelowRowNearTop() {
        /** Row placed near the top of a tall window. */
        val anchor: MenuAnchorRect = rowAt(top = 200, left = 40, right = 1000)
        assertEquals(MenuAnchorOffset(x = 40, y = 344), placeIn(anchor, windowWidth = 1080, windowHeight = 2400))
    }

    // What:     `@Test fun flipsAboveRowNearBottom()` checks a row whose menu would overflow the bottom.
    // Why:      When placing the menu below would leave the window, it must move above the row instead.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("flips above near bottom", () => expect(place(row(1700, 40, 1000), 1080, 2000)).toEqual(...));
    // ```
    /** A row near the bottom flips the menu above it. */
    @Test
    fun flipsAboveRowNearBottom() {
        /** Row placed near the bottom of a window that cannot hold the menu below it. */
        val anchor: MenuAnchorRect = rowAt(top = 1700, left = 40, right = 1000)
        assertEquals(MenuAnchorOffset(x = 40, y = 1100), placeIn(anchor, windowWidth = 1080, windowHeight = 2000))
    }

    // What:     `@Test fun clampsToWindowWhenNeitherSideFits()` checks a menu taller than the space above and below.
    // Why:      The menu must stay inside the window even when neither side has room for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("neither side fits", () => expect(place(row(100, 0, 500), 1080, 500)).toEqual({ x: 0, y: 0 }));
    // ```
    /** A menu taller than the window clamps to the top edge when neither side fits. */
    @Test
    fun clampsToWindowWhenNeitherSideFits() {
        /** Row in a window too short for the menu on either side. */
        val anchor: MenuAnchorRect = rowAt(top = 100, left = 0, right = 500)
        assertEquals(MenuAnchorOffset(x = 0, y = 0), placeIn(anchor, windowWidth = 1080, windowHeight = 500))
    }

    // What:     `@Test fun clampsLeftEdgeNearRightWindowEdge()` checks a row whose left edge is past the window's room.
    // Why:      The menu must slide left so its whole width stays visible on the right side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("right edge clamp", () => expect(place(row(200, 900, 1080), 1080, 2400).x).toBe(240));
    // ```
    /** A row near the right edge keeps the whole menu inside the window. */
    @Test
    fun clampsLeftEdgeNearRightWindowEdge() {
        /** Row whose left edge leaves less width than the menu needs. */
        val anchor: MenuAnchorRect = rowAt(top = 200, left = 900, right = 1080)
        assertEquals(240, placeIn(anchor, windowWidth = 1080, windowHeight = 2400).x)
    }

    // What:     `@Test fun coverAndInnerWidthsClampDifferently()` checks the same row in a cover and an inner window.
    // Why:      The narrow cover clamps the menu where the wide inner panel does not, so both widths are tested.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("cover vs inner", () => { expect(place(row(200, 300, 400), 360, 2400).x).toBe(0); });
    // ```
    /** The same row clamps on the narrow cover window and keeps its own left edge on the wide inner window. */
    @Test
    fun coverAndInnerWidthsClampDifferently() {
        /** Row that starts at the same place in both windows. */
        val anchor: MenuAnchorRect = rowAt(top = 200, left = 300, right = 400)
        assertEquals(0, placeIn(anchor, windowWidth = 360, windowHeight = 2400).x)
        assertEquals(300, placeIn(anchor, windowWidth = 1200, windowHeight = 2400).x)
    }

    // What:     `@Test fun honorsStartInsetAsLeftLimit()` checks a row at the window edge with an inset.
    // Why:      The menu must not start left of the inset that the pane leaves, even when the row does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("start inset", () => expect(menuAnchorOffset({ ...req, startInset: 36 }).x).toBe(36));
    // ```
    /** A row at the window's left edge starts the menu at the start inset. */
    @Test
    fun honorsStartInsetAsLeftLimit() {
        /** Row flush with the window's left edge. */
        val anchor: MenuAnchorRect = rowAt(top = 200, left = 0, right = 1000)
        /** Request with a 36 pixel start inset and room for the menu. */
        val request = MenuAnchorRequest(
            anchor = anchor,
            popupWidth = menuWidth,
            popupHeight = menuHeight,
            windowWidth = 1080,
            windowHeight = 2400,
            startInset = 36,
        )
        assertEquals(36, menuAnchorOffset(request).x)
    }

    // What:     `@Test fun menuWiderThanWindowStartsAtInset()` checks a menu wider than the window.
    // Why:      When the menu cannot fit at all, it starts at the inset rather than at a negative position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("wider than window", () => expect(place(row(200, 500, 900), 600, 2400).x).toBe(0));
    // ```
    /** A menu wider than the window starts at the start inset and never at a negative position. */
    @Test
    fun menuWiderThanWindowStartsAtInset() {
        /** Row in a window narrower than the menu. */
        val anchor: MenuAnchorRect = rowAt(top = 200, left = 500, right = 900)
        assertEquals(0, placeIn(anchor, windowWidth = 600, windowHeight = 2400).x)
    }
}
