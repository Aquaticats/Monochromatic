package dev.monochromatic.musicplayer

import org.junit.Assert.assertEquals
import org.junit.Test

/** Which rows differ although their items are the same instances. */
class CurrentRowPositionsTest {
    @Test
    fun nothingChangedGivesNoPositions() {
        assertEquals(
            emptySet<Int>(),
            currentRowPositions(previousCurrent = 3, previousDurationMs = 200L, current = 3, durationMs = 200L),
        )
        assertEquals(
            emptySet<Int>(),
            currentRowPositions(previousCurrent = null, previousDurationMs = 0L, current = null, durationMs = 0L),
        )
    }

    @Test
    fun aNewDurationOfTheSameTrackNamesThatTrackOnly() {
        assertEquals(
            setOf(3),
            currentRowPositions(previousCurrent = 3, previousDurationMs = 0L, current = 3, durationMs = 200L),
        )
    }

    @Test
    fun aMovedCursorNamesTheOldAndTheNewTrack() {
        assertEquals(
            setOf(3, 4),
            currentRowPositions(previousCurrent = 3, previousDurationMs = 200L, current = 4, durationMs = 200L),
        )
    }

    @Test
    fun aMissingCurrentTrackIsLeftOut() {
        assertEquals(
            setOf(3),
            currentRowPositions(previousCurrent = 3, previousDurationMs = 200L, current = null, durationMs = 0L),
        )
        assertEquals(
            setOf(5),
            currentRowPositions(previousCurrent = null, previousDurationMs = 0L, current = 5, durationMs = 0L),
        )
    }
}
