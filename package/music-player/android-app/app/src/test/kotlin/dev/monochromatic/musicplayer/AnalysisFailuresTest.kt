package dev.monochromatic.musicplayer

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** D113: the measures of freshly analyzed tracks reach the host once; failure reports are in the notification tests. */
class AnalysisFailuresTest {
    @Test
    fun measuredTracksAreHandedToTheHostOnceSoItReadsTheirNewMeasures() {
        MeasureInvalidation.take()
        MeasureInvalidation.report(listOf("content://a/1", "content://a/2"))
        MeasureInvalidation.report(listOf("content://a/2"))
        assertEquals(setOf("content://a/1", "content://a/2"), MeasureInvalidation.take())
        assertTrue(MeasureInvalidation.take().isEmpty())
        MeasureInvalidation.report(emptyList())
        assertTrue(MeasureInvalidation.pending.isEmpty())
    }
}
