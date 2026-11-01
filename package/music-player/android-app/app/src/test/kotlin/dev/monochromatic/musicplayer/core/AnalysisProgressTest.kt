package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the count text and the content of the analysis notification without Android. */
class AnalysisProgressTest {
    /** Confirms counts from zero through the largest Int group with ASCII commas. */
    @Test
    fun countTextGroupsThousands() {
        assertEquals("0", analysisCountText(0))
        assertEquals("999", analysisCountText(999))
        assertEquals("1,000", analysisCountText(1000))
        assertEquals("2,147,483,647", analysisCountText(Int.MAX_VALUE))
    }

    /** Confirms the accepted study counts give the accepted text and a determinate bar. */
    @Test
    fun contentShowsTheCountAndPercent() {
        /** Content for the counts named in decision D26. */
        val content: AnalysisNotificationContent = analysisNotificationContent(done = 412, total = 1218, paused = false)
        assertEquals("412 of 1,218", content.text)
        assertEquals(33, content.percent)
        assertFalse(content.indeterminate)
        assertFalse(content.paused)
    }

    /** Confirms a seven-digit count groups into millions and thousands. */
    @Test
    fun contentGroupsWideCounts() {
        /** Content for the widest study count. */
        val content: AnalysisNotificationContent =
            analysisNotificationContent(done = 9999998, total = 9999999, paused = false)
        assertEquals("9,999,998 of 9,999,999", content.text)
        assertEquals(99, content.percent)
    }

    /** Confirms an unknown total gives an indeterminate bar with no count text. */
    @Test
    fun unknownTotalIsIndeterminate() {
        for (total in listOf(0, -5)) {
            /** Content for a total that is not known yet. */
            val content: AnalysisNotificationContent =
                analysisNotificationContent(done = 3, total = total, paused = false)
            assertTrue(content.indeterminate)
            assertEquals("", content.text)
            assertEquals(0, content.percent)
        }
    }

    /** Confirms done counts below zero or above the total are clamped before they reach the text or percent. */
    @Test
    fun doneIsClampedIntoTheTotal() {
        /** Content for an overshooting count. */
        val over: AnalysisNotificationContent = analysisNotificationContent(done = 15, total = 10, paused = false)
        /** Content for a negative count. */
        val under: AnalysisNotificationContent = analysisNotificationContent(done = -3, total = 10, paused = false)
        assertEquals("10 of 10", over.text)
        assertEquals(100, over.percent)
        assertEquals("0 of 10", under.text)
        assertEquals(0, under.percent)
    }

    /** Confirms the percent rounds down and reaches 100 only when every track was visited. */
    @Test
    fun percentRoundsDownAndReachesOneHundredOnlyAtTheEnd() {
        assertEquals(99, analysisNotificationContent(done = 999, total = 1000, paused = false).percent)
        assertEquals(100, analysisNotificationContent(done = 1000, total = 1000, paused = false).percent)
        assertEquals(0, analysisNotificationContent(done = 0, total = 1000, paused = false).percent)
    }

    /** Confirms the percent of the largest counts does not overflow an Int. */
    @Test
    fun percentDoesNotOverflowForHugeCounts() {
        assertEquals(
            49,
            analysisNotificationContent(done = Int.MAX_VALUE / 2, total = Int.MAX_VALUE, paused = false).percent,
        )
    }

    /** Confirms the pause flag is carried into the content and makes it differ from the running content. */
    @Test
    fun pauseFlagIsPartOfTheContent() {
        /** Content while running. */
        val running: AnalysisNotificationContent = analysisNotificationContent(done = 5, total = 10, paused = false)
        /** Content while paused, with the same counts. */
        val paused: AnalysisNotificationContent = analysisNotificationContent(done = 5, total = 10, paused = true)
        assertTrue(paused.paused)
        assertFalse(running == paused)
    }

    /** Confirms the title is the one D107 names. */
    @Test
    fun titleIsAnalyzingTruePeak() {
        assertEquals("Analyzing true peak", ANALYSIS_NOTIFICATION_TITLE)
    }
}
