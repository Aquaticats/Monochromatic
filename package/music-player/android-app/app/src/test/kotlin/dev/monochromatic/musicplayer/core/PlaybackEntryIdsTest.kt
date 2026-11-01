package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * S1 of the polish review: a shuffle history repeats a track once every track has played, and the media timeline
 * crashed with `Duplicate MediaItemData UID in playlist` because the load-order index doubled as the entry id.
 * These tests drive the real [Queue] through the cases that reach a repeat.
 */
class PlaybackEntryIdsTest {
    /** Root-level track names, which all fall into one page. */
    private fun names(count: Int): List<String> = (0 until count).map { index -> "track$index.mp3" }

    /** Builds a queue in [mode] and plays track 0. */
    private fun started(count: Int, mode: PlaybackMode): Queue {
        val queue = Queue.withRngSeed(7)
        queue.setTracks(names(count))
        queue.setPlaybackMode(mode)
        queue.playIndex(0)
        return queue
    }

    /** Advances [steps] natural ends and asserts the entry ids stay distinct after every step. */
    private fun assertDistinctAfterEveryStep(queue: Queue, steps: Int) {
        repeat(steps) {
            queue.advance(natural = true)
            val ids = playbackEntryIds(queue.playbackOrder())
            assertEquals("ids of ${queue.playbackOrder()}", ids.size, ids.toSet().size)
        }
    }

    @Test
    fun emptyOrderHasNoIds() {
        assertTrue(playbackEntryIds(emptyList()).isEmpty())
    }

    @Test
    fun firstPlayKeepsTheLoadIndexAndRepeatsGetDistinctIds() {
        val ids = playbackEntryIds(listOf(3, 1, 3, 3, 1))
        assertEquals(3L, ids[0])
        assertEquals(1L, ids[1])
        assertEquals(5, ids.toSet().size)
    }

    @Test
    fun idsStayStableWhileTheHistoryOnlyGrows() {
        val before = playbackEntryIds(listOf(0, 1, 0))
        val after = playbackEntryIds(listOf(0, 1, 0, 1, 0))
        assertEquals(before, after.take(3))
    }

    @Test
    fun aSingleTrackFolderInShuffleRepeatsWithDistinctIds() {
        val queue = started(1, PlaybackMode.SHUFFLE_PAGE)
        assertDistinctAfterEveryStep(queue, 5)
        assertEquals(listOf(0, 0, 0, 0, 0, 0), queue.playbackOrder())
    }

    @Test
    fun aTwoTrackFolderInShuffleRepeatsWithDistinctIds() {
        val queue = started(2, PlaybackMode.SHUFFLE_PAGE)
        assertDistinctAfterEveryStep(queue, 9)
        assertEquals(10, queue.playbackOrder().size)
    }

    @Test
    fun theWrapOfAThreeTrackShuffleCycleHasDistinctIds() {
        val queue = started(3, PlaybackMode.SHUFFLE_PAGE)
        assertDistinctAfterEveryStep(queue, 10)
        val order = queue.playbackOrder()
        assertTrue("the history revisited a track", order.toSet().size < order.size)
    }

    @Test
    fun shuffleAllAcrossItsWrapHasDistinctIds() {
        val queue = started(4, PlaybackMode.SHUFFLE_ALL)
        assertDistinctAfterEveryStep(queue, 13)
    }

    @Test
    fun repeatTrackKeepsOneEntryAndNeverDuplicates() {
        val queue = started(1, PlaybackMode.REPEAT)
        repeat(4) { assertEquals(0, queue.advance(natural = true)) }
        assertEquals(listOf(0L), playbackEntryIds(queue.playbackOrder()))
    }

    @Test
    fun repeatTrackWithManualNextWrapsWithoutDuplicates() {
        val queue = started(2, PlaybackMode.REPEAT)
        repeat(5) { queue.advance(natural = false) }
        val ids = playbackEntryIds(queue.playbackOrder())
        assertEquals(ids.size, ids.toSet().size)
    }

    @Test
    fun backThroughAShuffleHistoryWithRepeatsKeepsIdsDistinct() {
        val queue = started(1, PlaybackMode.SHUFFLE_PAGE)
        repeat(3) { queue.advance(natural = true) }
        repeat(2) { queue.prev() }
        val ids = playbackEntryIds(queue.playbackOrder())
        assertEquals(ids.size, ids.toSet().size)
        assertEquals(1, queue.cursorPosition())
    }
}
