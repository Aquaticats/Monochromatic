package dev.monochromatic.musicplayer

import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Test

/** The timeline rows of the media session are reused between pulls except where the track or its data changed. */
class RowReuseTest {
    /** Item with identity: two instances with equal text are different items. */
    private class Item(val name: String)

    /** Row built for an item; counts as built when a new instance appears. */
    private class Row(val position: Int, val name: String)

    /** Records which positions were built. */
    private class Builds {
        val positions: MutableList<Int> = ArrayList()

        fun build(position: Int, item: Item): Row {
            positions.add(position)
            return Row(position, item.name)
        }
    }

    private fun items(count: Int): List<Item> = (0 until count).map { index -> Item("i$index") }

    private fun rowsFor(items: List<Item>): List<Row> = items.mapIndexed { position, item -> Row(position, item.name) }

    @Test
    fun theSameItemsWithNothingToRebuildReturnThePreviousRowListWithoutBuilding() {
        val items = items(5)
        val rows = rowsFor(items)
        val builds = Builds()
        val result = reuseRows(
            previousItems = items,
            previousRows = rows,
            items = items,
            rebuilt = emptySet(),
            build = builds::build,
        )
        assertSame(rows, result)
        assertEquals(emptyList<Int>(), builds.positions)
    }

    @Test
    fun theSameItemsBuildOnlyTheNamedPositions() {
        val items = items(5)
        val rows = rowsFor(items)
        val builds = Builds()
        val result = reuseRows(
            previousItems = items,
            previousRows = rows,
            items = items,
            rebuilt = setOf(1, 3),
            build = builds::build,
        )
        assertEquals(listOf(1, 3), builds.positions)
        assertSame(rows[0], result[0])
        assertSame(rows[2], result[2])
        assertSame(rows[4], result[4])
        assertEquals(5, result.size)
    }

    @Test
    fun aPositionOutsideTheItemsIsIgnored() {
        val items = items(2)
        val rows = rowsFor(items)
        val builds = Builds()
        val result = reuseRows(
            previousItems = items,
            previousRows = rows,
            items = items,
            rebuilt = setOf(7),
            build = builds::build,
        )
        assertEquals(emptyList<Int>(), builds.positions)
        assertEquals(2, result.size)
    }

    @Test
    fun aReplacedItemIsBuiltAndEveryOtherRowIsShared() {
        val items = items(4)
        val rows = rowsFor(items)
        val changed = items.toMutableList()
        changed[2] = Item("new")
        val builds = Builds()
        val result = reuseRows(
            previousItems = items,
            previousRows = rows,
            items = changed,
            rebuilt = emptySet(),
            build = builds::build,
        )
        assertEquals(listOf(2), builds.positions)
        assertEquals("new", result[2].name)
        assertSame(rows[0], result[0])
        assertSame(rows[1], result[1])
        assertSame(rows[3], result[3])
    }

    @Test
    fun aNamedPositionIsBuiltEvenWhenItsItemIsTheSameInstance() {
        val items = items(3)
        val rows = rowsFor(items)
        val changed = items.toList()
        val builds = Builds()
        val result = reuseRows(
            previousItems = items,
            previousRows = rows,
            items = changed,
            rebuilt = setOf(0),
            build = builds::build,
        )
        assertEquals(listOf(0), builds.positions)
        assertSame(rows[1], result[1])
    }

    @Test
    fun aLongerListBuildsTheNewPositionsAndAShorterListDropsTheTail() {
        val items = items(3)
        val rows = rowsFor(items)
        val longer = items + Item("tail")
        val builds = Builds()
        val grown = reuseRows(
            previousItems = items,
            previousRows = rows,
            items = longer,
            rebuilt = emptySet(),
            build = builds::build,
        )
        assertEquals(listOf(3), builds.positions)
        assertEquals(4, grown.size)
        val shorter = items.subList(0, 2)
        val shrunk = reuseRows(
            previousItems = items,
            previousRows = rows,
            items = shorter,
            rebuilt = emptySet(),
            build = builds::build,
        )
        assertEquals(2, shrunk.size)
        assertSame(rows[0], shrunk[0])
    }

    @Test
    fun anEmptyPreviousListBuildsEveryPosition() {
        val items = items(3)
        val builds = Builds()
        val result = reuseRows(
            previousItems = emptyList(),
            previousRows = emptyList<Row>(),
            items = items,
            rebuilt = emptySet(),
            build = builds::build,
        )
        assertEquals(listOf(0, 1, 2), builds.positions)
        assertEquals(3, result.size)
    }
}
