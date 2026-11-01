package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/**
 * M3 of the polish review: D2 says In order plays to the end of the folder and stops, but the queue wrapped to the
 * first track (and a one-track folder replayed itself forever).
 */
class QueueInOrderEndTest {
    /** Root-level track names, which all fall into one page. */
    private fun names(count: Int): List<String> = (0 until count).map { index -> "track$index.mp3" }

    /** Builds an In order queue of [count] tracks playing track 0. */
    private fun inOrder(count: Int): Queue {
        val queue = Queue.withRngSeed(3)
        queue.setTracks(names(count))
        queue.setPlaybackMode(PlaybackMode.IN_ORDER)
        queue.playIndex(0)
        return queue
    }

    @Test
    fun aNaturalEndOnTheLastTrackStopsAndKeepsTheCursor() {
        val queue = inOrder(3)
        assertEquals(1, queue.advance(natural = true))
        assertEquals(2, queue.advance(natural = true))
        assertNull(queue.advance(natural = true))
        assertEquals(2, queue.currentIndex())
    }

    @Test
    fun aSingleTrackFolderStopsAfterItsOneTrack() {
        val queue = inOrder(1)
        assertNull(queue.advance(natural = true))
        assertEquals(0, queue.currentIndex())
    }

    @Test
    fun aTwoTrackFolderStopsInsteadOfGoingBackToTheFirstTrack() {
        val queue = inOrder(2)
        assertEquals(1, queue.advance(natural = true))
        assertNull(queue.advance(natural = true))
        assertEquals(1, queue.currentIndex())
    }

    @Test
    fun aManualNextOnTheLastTrackStillWraps() {
        val queue = inOrder(2)
        queue.advance(natural = true)
        assertEquals(0, queue.advance(natural = false))
    }

    @Test
    fun rewindScopePutsTheCursorOnTheFirstTrackAfterTheStop() {
        val queue = inOrder(3)
        repeat(3) { queue.advance(natural = true) }
        assertEquals(0, queue.rewindScope())
        assertEquals(0, queue.currentIndex())
        assertEquals(1, queue.advance(natural = true))
    }

    @Test
    fun rewindScopeOfAnEmptyQueueIsNothing() {
        assertNull(Queue.withRngSeed(1).rewindScope())
    }

    @Test
    fun repeatStillReplaysOnANaturalEndAndWrapsOnManualNext() {
        val queue = Queue.withRngSeed(3)
        queue.setTracks(names(2))
        queue.setPlaybackMode(PlaybackMode.REPEAT)
        queue.playIndex(1)
        assertEquals(1, queue.advance(natural = true))
        assertEquals(0, queue.advance(natural = false))
    }

    @Test
    fun shuffleNeverStopsOnANaturalEnd() {
        val queue = Queue.withRngSeed(3)
        queue.setTracks(names(1))
        queue.setPlaybackMode(PlaybackMode.SHUFFLE_PAGE)
        queue.playIndex(0)
        repeat(4) { assertEquals(0, queue.advance(natural = true)) }
    }
}
