// The delta gate hands over only the tracks found since the last batch, and its tail leaves nothing behind.

// What:     `package ...core` places the cases beside the gate.
// Why:      The gate has no dependencies, so the cases construct it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** Verifies the thresholds, the order, and that the batches together carry every item exactly once. */
class LibraryDeltaGateTest {
    /** Nothing is handed over below the threshold, and the batch holds only the new items in order. */
    @Test
    fun aBatchHoldsOnlyTheNewItemsInOrder() {
        val gate = LibraryDeltaGate<Int>(3)
        val found = ArrayList<Int>()
        found.addAll(listOf(9, 1))
        assertNull(gate.nextDelta(found))
        found.add(5)
        assertEquals(listOf(9, 1, 5), gate.nextDelta(found))
        found.addAll(listOf(7, 2))
        assertNull(gate.nextDelta(found))
        found.add(4)
        assertEquals(listOf(7, 2, 4), gate.nextDelta(found))
    }

    /** The tail carries what no batch carried, and is null when nothing remains. */
    @Test
    fun theTailCarriesTheRemainder() {
        val gate = LibraryDeltaGate<Int>(3)
        val found = arrayListOf(1, 2, 3, 4)
        assertEquals(listOf(1, 2, 3, 4), gate.nextDelta(found))
        assertNull(gate.tail(found))
        found.add(5)
        assertEquals(listOf(5), gate.tail(found))
        assertNull(gate.tail(found))
    }

    /** A library smaller than the threshold arrives whole through the tail. */
    @Test
    fun aSmallLibraryArrivesThroughTheTail() {
        val gate = LibraryDeltaGate<String>(128)
        val found = arrayListOf("a", "b")
        assertNull(gate.nextDelta(found))
        assertEquals(listOf("a", "b"), gate.tail(found))
    }

    /** The batches together equal the list, for any threshold and any growth pattern. */
    @Test
    fun thePiecesTogetherAreTheWholeList() {
        val random = java.util.Random(3L)
        repeat(200) {
            val gate = LibraryDeltaGate<Int>(1 + random.nextInt(20))
            val found = ArrayList<Int>()
            val carried = ArrayList<Int>()
            repeat(random.nextInt(300)) {
                found.add(found.size)
                gate.nextDelta(found)?.let { carried.addAll(it) }
            }
            gate.tail(found)?.let { carried.addAll(it) }
            assertEquals(found, carried)
        }
    }
}
