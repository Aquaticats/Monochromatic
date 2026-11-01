// What:     `package dev.monochromatic.musicplayer.core` places this test beside the list focus rules.
// Why:      The test reaches the pure functions in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each ring position is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

// What:     `class ListFocusTest` groups the cases of the focus ring movement rules.
// Why:      Both one-stop lists move their ring with these functions, and the track list has headers it must skip.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("listFocus", () => { /* cases */ });
// ```
/** Verifies the ring movement over lists that mix selectable entries with headers. */
class ListFocusTest {
    /** A list of header, row, row, header, row: only positions 1, 2 and 4 can hold the ring. */
    private val mixed: List<Boolean> = listOf(false, true, true, false, true)

    /** Down skips a header and stops at the last row. */
    @Test
    fun downSkipsHeadersAndStopsAtTheEnd() {
        assertEquals(2, moveFocus(mixed, 1, ListMove.DOWN, 5))
        assertEquals(4, moveFocus(mixed, 2, ListMove.DOWN, 5))
        assertEquals(4, moveFocus(mixed, 4, ListMove.DOWN, 5))
    }

    /** Up skips a header and stops at the first row. */
    @Test
    fun upSkipsHeadersAndStopsAtTheStart() {
        assertEquals(2, moveFocus(mixed, 4, ListMove.UP, 5))
        assertEquals(1, moveFocus(mixed, 2, ListMove.UP, 5))
        assertEquals(1, moveFocus(mixed, 1, ListMove.UP, 5))
    }

    /** Home and End jump to the first and last row. */
    @Test
    fun homeAndEndJumpToTheEnds() {
        assertEquals(1, moveFocus(mixed, 4, ListMove.HOME, 5))
        assertEquals(4, moveFocus(mixed, 1, ListMove.END, 5))
    }

    /** A page key moves by rows, not by entries, and stops at the ends. */
    @Test
    fun pageKeysMoveByRows() {
        assertEquals(4, moveFocus(mixed, 1, ListMove.PAGE_DOWN, 2))
        assertEquals(1, moveFocus(mixed, 4, ListMove.PAGE_UP, 9))
        assertEquals(2, moveFocus(mixed, 1, ListMove.PAGE_DOWN, 1))
        assertEquals(2, moveFocus(mixed, 1, ListMove.PAGE_DOWN, 0))
    }

    /** A ring that is nowhere or on a header starts at the first row, or the last for End and Page down. */
    @Test
    fun aRingOnAHeaderStartsAtTheEdge() {
        assertEquals(1, moveFocus(mixed, null, ListMove.DOWN, 5))
        assertEquals(1, moveFocus(mixed, 0, ListMove.UP, 5))
        assertEquals(4, moveFocus(mixed, null, ListMove.END, 5))
        assertEquals(4, moveFocus(mixed, 3, ListMove.PAGE_DOWN, 5))
    }

    /** A list with no selectable entry has no ring. */
    @Test
    fun anEmptyListHasNoRing() {
        assertNull(moveFocus(emptyList(), null, ListMove.DOWN, 5))
        assertNull(moveFocus(listOf(false, false), 0, ListMove.HOME, 5))
        assertNull(startFocus(listOf(false), null))
    }

    /** The ring starts on the preferred entry when it is selectable, else on the first one. */
    @Test
    fun startFocusPrefersThePlayingEntry() {
        assertEquals(2, startFocus(mixed, 2))
        assertEquals(1, startFocus(mixed, 3))
        assertEquals(1, startFocus(mixed, null))
        assertEquals(1, startFocus(mixed, 99))
    }

    /** A letter finds the first label that starts with it, ignoring case, and skips unlabeled entries. */
    @Test
    fun aLetterFindsTheFirstLabel() {
        /** Labels with a heading that has none. */
        val labels: List<String?> = listOf(null, "Alpha", "bravo", "Beta")
        assertEquals(2, firstStartingWith(labels, 'b'))
        assertEquals(2, firstStartingWith(labels, 'B'))
        assertEquals(1, firstStartingWith(labels, 'a'))
        assertNull(firstStartingWith(labels, 'z'))
    }

    /** `ringAfter` moves for movement and letter commands and ignores the others. */
    @Test
    fun ringAfterMovesOnlyForMovementAndLetters() {
        /** Labels of three rail cells. */
        val labels: List<String?> = listOf("A", "B", "C")
        /** Every cell can hold the ring. */
        val all: List<Boolean> = listOf(true, true, true)
        assertEquals(1, ringAfter(KeyMoveList(ListMove.DOWN), all, labels, 0, 8))
        assertEquals(2, ringAfter(KeyJumpToLetter('c'), all, labels, 0, 8))
        assertNull(ringAfter(KeyJumpToLetter('z'), all, labels, 0, 8))
        assertNull(ringAfter(KeyActivateRow, all, labels, 0, 8))
    }
}
