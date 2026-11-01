// The primitive lists of the Search index: growth that never changes what readers see, the keyed lists that group a
// batch without sorting, and the path helpers that find titles and folders in place.

// What:     `package ...core` places the cases beside the lists.
// Why:      They call the internal list classes through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.Random
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the lists and the in-place path helpers. */
class IndexListsTest {
    /** A list grown for more entries keeps its entries and reports the new room only when it needed it. */
    @Test
    fun growthKeepsEntriesAndIsOnlyMadeWhenNeeded() {
        val list = IntList(IntArray(4), 0)
        repeat(4) { list.add(it * 2) }
        assertNull(list.grownFor(0))
        val grown = list.grownFor(5)!!
        assertEquals(listOf(0, 2, 4, 6), grown.data.take(grown.size))
        assertTrue(grown.data.size >= 9)
        assertEquals(4, list.size)
    }

    /** Counting entries below a limit is a binary search over the ascending list. */
    @Test
    fun countBelowFindsTheCut() {
        val list = IntList(IntArray(16), 0)
        listOf(1, 3, 3, 8, 20).forEach { list.add(it) }
        assertEquals(0, list.countBelow(1))
        assertEquals(1, list.countBelow(2))
        assertEquals(3, list.countBelow(4))
        assertEquals(5, list.countBelow(100))
    }

    /** The keyed lists group ascending track numbers by key and ignore a repeat within one track. */
    @Test
    fun keyedListsGroupWithoutSorting() {
        val random = Random(9L)
        val lists = KeyedLists()
        val expected = HashMap<Long, MutableList<Int>>()
        repeat(5000) {
            val local = it / 7
            val key = random.nextInt(300).toLong() * 0x1_0000_0001L
            lists.add(key, local)
            val bucket = expected.getOrPut(key) { ArrayList() }
            if (bucket.isEmpty() || bucket.last() != local) bucket.add(local)
        }
        assertEquals(expected.size, lists.size)
        var seen = 0
        lists.forEach { key, list ->
            seen += 1
            assertEquals(expected[key], list.data.take(list.size))
        }
        assertEquals(expected.size, seen)
    }

    /** The title range is the file name without the last extension, as `trackTitleFor` defines it. */
    @Test
    fun titleRangeMatchesTheTitleFunction() {
        val random = Random(4L)
        repeat(3000) {
            val path = randomPath(random)
            val range = titleRangeOf(path)
            val from = (range ushr KEY_SHIFT).toInt()
            val to = (range and LOW_BITS_MASK).toInt()
            assertEquals(path, trackTitleFor(path), path.substring(from, to))
        }
    }

    /** The top-level folder and the segment count equal the folder index's own splitting. */
    @Test
    fun folderOfMatchesTheFolderIndex() {
        val random = Random(8L)
        repeat(3000) {
            val path = randomPath(random)
            val segments = path.split("/").filter { it.isNotEmpty() && it != "." && it != ".." }
            val found = folderOf(path)
            assertEquals(path, segments.firstOrNull(), found.first)
            assertEquals(path, minOf(segments.size, 3), found.segments)
        }
    }
}
