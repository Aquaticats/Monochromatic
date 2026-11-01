// The menu's keyboard ring moves with Down and Up, wraps at the ends, and starts on no row.

// What:     `package ...core` matches the rule's package.
// Why:      The tests call the pure function directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** Checks where the ring lands after each key. */
class MenuRowFocusTest {
    /** The first Down lands on the first row and the first Up on the last. */
    @Test
    fun theFirstKeyStartsAtTheNearEnd() {
        assertEquals(0, menuRowAfter(current = null, count = 6, step = 1))
        assertEquals(5, menuRowAfter(current = null, count = 6, step = -1))
    }

    /** Down and Up move one row. */
    @Test
    fun keysMoveOneRow() {
        assertEquals(3, menuRowAfter(current = 2, count = 6, step = 1))
        assertEquals(1, menuRowAfter(current = 2, count = 6, step = -1))
    }

    /** Moving past either end wraps to the other. */
    @Test
    fun movingPastAnEndWraps() {
        assertEquals(0, menuRowAfter(current = 5, count = 6, step = 1))
        assertEquals(5, menuRowAfter(current = 0, count = 6, step = -1))
    }

    /** A menu with no rows has nothing to ring. */
    @Test
    fun anEmptyMenuHasNoRing() {
        assertNull(menuRowAfter(current = null, count = 0, step = 1))
        assertNull(menuRowAfter(current = 3, count = 0, step = -1))
    }
}
