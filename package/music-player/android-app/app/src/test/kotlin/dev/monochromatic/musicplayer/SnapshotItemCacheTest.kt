package dev.monochromatic.musicplayer

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The rows of the playback snapshot are cached between media session refreshes, so a track change does not rebuild
 * one row per queue position. These tests pin what is reused and what is rebuilt.
 */
class SnapshotItemCacheTest {
    /** Playback URIs of [count] tracks. */
    private fun uris(count: Int): List<String> = (0 until count).map { index -> "u$index" }

    /** Display paths of [count] tracks. */
    private fun display(count: Int): List<String> = (0 until count).map { index -> "folder/t$index.mp3" }

    @Test
    fun theSameListsAndTagsGiveTheSameRowListInstance() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 1, 2)
        val display = display(3)
        val uris = uris(3)
        val first = cache.itemsFor(order = order, display = display, uris = uris, artists = emptyMap())
        val second = cache.itemsFor(order = order, display = display, uris = uris, artists = emptyMap())
        assertSame(first, second)
    }

    @Test
    fun aNewOrderRebuildsEveryRowWithTitleUriLoadIndexAndEntryId() {
        val cache = SnapshotItemCache()
        val display = display(3)
        val uris = uris(3)
        val before = cache.itemsFor(order = listOf(0, 1, 2), display = display, uris = uris, artists = emptyMap())
        val after = cache.itemsFor(order = listOf(2, 1, 0), display = display, uris = uris, artists = emptyMap())
        assertNotSame(before, after)
        assertEquals(listOf("u2", "u1", "u0"), after.map { item -> item.uri })
        assertEquals(listOf(2, 1, 0), after.map { item -> item.loadIndex })
        assertEquals(listOf("t2", "t1", "t0"), after.map { item -> item.title })
        assertEquals(listOf(2L, 1L, 0L), after.map { item -> item.entryId })
    }

    @Test
    fun aNewLibraryRebuildsEveryRowEvenWhenTheOrderIsTheSameList() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 1)
        val before = cache.itemsFor(order = order, display = display(2), uris = uris(2), artists = emptyMap())
        val renamed = listOf("other/a.mp3", "other/b.mp3")
        val after = cache.itemsFor(order = order, display = renamed, uris = listOf("x0", "x1"), artists = emptyMap())
        assertEquals(listOf("x0", "x1"), after.map { item -> item.uri })
        assertEquals(listOf("a", "b"), after.map { item -> item.title })
        assertTrue(before.indices.none { index -> before[index] === after[index] })
    }

    @Test
    fun anArtistTagPatchesOnlyTheRowsOfThatTrackAndSharesTheOthers() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 1, 2, 1)
        val display = display(3)
        val uris = uris(3)
        val before = cache.itemsFor(order = order, display = display, uris = uris, artists = emptyMap())
        val after = cache.itemsFor(order = order, display = display, uris = uris, artists = mapOf("u1" to "Ada"))
        assertNotSame(before, after)
        assertEquals(listOf(null, "Ada", null, "Ada"), after.map { item -> item.artist })
        assertSame(before[0], after[0])
        assertSame(before[2], after[2])
        assertNotSame(before[1], after[1])
        assertEquals(before[1].entryId, after[1].entryId)
        assertEquals(before[3].entryId, after[3].entryId)
        assertNotEquals(after[1].entryId, after[3].entryId)
    }

    @Test
    fun aReplacedTagPatchesAndAnEvictedTagClearsTheRows() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 1)
        val display = display(2)
        val uris = uris(2)
        val tags = LinkedHashMap<String, String>()
        tags["u0"] = "Ada"
        val first = cache.itemsFor(order = order, display = display, uris = uris, artists = tags)
        assertEquals("Ada", first[0].artist)
        tags["u0"] = "Bo"
        val replaced = cache.itemsFor(order = order, display = display, uris = uris, artists = tags)
        assertEquals("Bo", replaced[0].artist)
        tags.remove("u0")
        val evicted = cache.itemsFor(order = order, display = display, uris = uris, artists = tags)
        assertNull(evicted[0].artist)
        assertSame(first[1], evicted[1])
    }

    @Test
    fun aTagForATrackOutsideTheQueueChangesNothing() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 1)
        val display = display(2)
        val uris = uris(2)
        val before = cache.itemsFor(order = order, display = display, uris = uris, artists = emptyMap())
        val after = cache.itemsFor(order = order, display = display, uris = uris, artists = mapOf("gone" to "Ada"))
        assertSame(before, after)
    }

    @Test
    fun theCacheDoesNotWatchTheCallersMapAfterTheCall() {
        val cache = SnapshotItemCache()
        val order = listOf(0)
        val display = display(1)
        val uris = uris(1)
        val tags = LinkedHashMap<String, String>()
        cache.itemsFor(order = order, display = display, uris = uris, artists = tags)
        tags["u0"] = "Ada"
        val patched = cache.itemsFor(order = order, display = display, uris = uris, artists = tags)
        assertEquals("Ada", patched[0].artist)
        val again = cache.itemsFor(order = order, display = display, uris = uris, artists = tags)
        assertSame(patched, again)
    }

    @Test
    fun repeatedTracksInAHistoryKeepDistinctEntryIdsThroughAPatch() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 0, 0)
        val display = display(1)
        val uris = uris(1)
        cache.itemsFor(order = order, display = display, uris = uris, artists = emptyMap())
        val patched = cache.itemsFor(order = order, display = display, uris = uris, artists = mapOf("u0" to "Ada"))
        assertEquals(3, patched.map { item -> item.entryId }.toSet().size)
    }

    @Test
    fun changedTagUrisNamesAddedReplacedAndRemovedTagsOnly() {
        val applied = mapOf("kept" to "A", "replaced" to "B", "removed" to "C")
        val current = mapOf("kept" to "A", "replaced" to "D", "added" to "E")
        assertEquals(setOf("replaced", "removed", "added"), changedTagUris(applied = applied, current = current))
        assertTrue(changedTagUris(applied = applied, current = applied).isEmpty())
    }

    @Test
    fun newDisplayPathsAloneRebuildTheTitles() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 1)
        val uris = uris(2)
        cache.itemsFor(order = order, display = display(2), uris = uris, artists = emptyMap())
        val after = cache.itemsFor(
            order = order,
            display = listOf("x/first.mp3", "x/second.mp3"),
            uris = uris,
            artists = emptyMap(),
        )
        assertEquals(listOf("first", "second"), after.map { item -> item.title })
    }

    @Test
    fun newUrisAloneRebuildTheUris() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 1)
        val display = display(2)
        cache.itemsFor(order = order, display = display, uris = uris(2), artists = emptyMap())
        val after = cache.itemsFor(order = order, display = display, uris = listOf("n0", "n1"), artists = emptyMap())
        assertEquals(listOf("n0", "n1"), after.map { item -> item.uri })
    }

    @Test
    fun aRebuildAfterANewLibraryAppliesTheTagsOnRecord() {
        val cache = SnapshotItemCache()
        val order = listOf(0, 1)
        val display = display(2)
        cache.itemsFor(order = order, display = display, uris = uris(2), artists = emptyMap())
        val renamed = listOf("n0", "n1")
        val tags = mapOf("n1" to "Ada")
        val after = cache.itemsFor(order = order, display = display, uris = renamed, artists = tags)
        assertEquals(listOf(null, "Ada"), after.map { item -> item.artist })
        val same = cache.itemsFor(order = order, display = display, uris = renamed, artists = tags)
        assertSame(after, same)
    }

    /** Asserts [expected] and [actual] differ; JUnit's own assertNotEquals needs another import for one use. */
    private fun assertNotEquals(expected: Long, actual: Long) {
        assertTrue("both were $expected", expected != actual)
    }
}
