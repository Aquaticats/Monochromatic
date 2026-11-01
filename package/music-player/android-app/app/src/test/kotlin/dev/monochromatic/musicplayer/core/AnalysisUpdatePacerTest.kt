package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the rate limit, the duplicate skip, the pause rule and the end rules with an explicit clock. */
class AnalysisUpdatePacerTest {
    /** Builds a running snapshot. */
    private fun running(done: Int, total: Int = TOTAL, paused: Boolean = false): AnalysisSnapshot =
        AnalysisSnapshot(done = done, total = total, paused = paused, end = AnalysisEnd.RUNNING)

    /** Confirms the very first update is posted at once when nothing was posted before. */
    @Test
    fun firstUpdateIsPostedImmediately() {
        /** Pacer that has not seen any post. */
        val pacer = AnalysisUpdatePacer()
        /** Step for the first snapshot. */
        val step: AnalysisStep = pacer.next(running(done = 0), nowMs = 0L)
        assertTrue(step is AnalysisStep.Post)
    }

    /** Confirms a changed count inside the window waits for the rest of the window instead of posting. */
    @Test
    fun updateInsideTheWindowWaitsForTheRemainder() {
        /** Pacer that posted the first frame at time zero. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = 0, total = TOTAL, paused = false), nowMs = 0L)
        assertEquals(AnalysisStep.Wait(millis = 300L), pacer.next(running(done = 50), nowMs = 200L))
    }

    /** Confirms the same update posts once the window has passed, and that the post restarts the window. */
    @Test
    fun updateAfterTheWindowIsPostedAndRestartsTheWindow() {
        /** Pacer that posted the first frame at time zero. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = 0, total = TOTAL, paused = false), nowMs = 0L)
        assertTrue(pacer.next(running(done = 50), nowMs = 500L) is AnalysisStep.Post)
        assertEquals(AnalysisStep.Wait(millis = 400L), pacer.next(running(done = 90), nowMs = 600L))
    }

    /** Confirms a snapshot that renders the same text and percent as the last post is skipped. */
    @Test
    fun identicalContentIsSkipped() {
        /** Pacer that posted 50 of 100 at time zero. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = 50, total = TOTAL, paused = false), nowMs = 0L)
        assertEquals(AnalysisStep.Skip, pacer.next(running(done = 50), nowMs = 10_000L))
    }

    /** Confirms a pause tap is posted at once inside the window, because no further progress arrives while paused. */
    @Test
    fun pauseChangeBypassesTheWindow() {
        /** Pacer that posted 50 of 100 at time zero. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = 50, total = TOTAL, paused = false), nowMs = 0L)
        /** Step for the same counts with the pause flag set, 10 ms later. */
        val step: AnalysisStep = pacer.next(running(done = 50, paused = true), nowMs = 10L)
        assertEquals(true, (step as AnalysisStep.Post).content.paused)
    }

    /** Confirms a resume tap is posted at once too, and that the resumed content is the running one. */
    @Test
    fun resumeChangeBypassesTheWindow() {
        /** Pacer whose last post was the paused content. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = 50, total = TOTAL, paused = true), nowMs = 0L)
        /** Step for the same counts running again, 10 ms later. */
        val step: AnalysisStep = pacer.next(running(done = 50, paused = false), nowMs = 10L)
        assertEquals(false, (step as AnalysisStep.Post).content.paused)
    }

    /** Confirms a completed analysis posts the final value at once, ignoring the window, and then removes. */
    @Test
    fun completionPostsTheFinalValueThenRemoves() {
        /** Pacer that posted 90 of 100 at time zero. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = 90, total = TOTAL, paused = false), nowMs = 0L)
        /** Snapshot after the last track, only 5 ms after the last post. */
        val completed = AnalysisSnapshot(done = TOTAL, total = TOTAL, paused = false, end = AnalysisEnd.COMPLETED)
        /** Step for the completion. */
        val finalStep: AnalysisStep = pacer.next(completed, nowMs = 5L)
        assertEquals("100 of 100", (finalStep as AnalysisStep.Post).content.text)
        assertEquals(100, finalStep.content.percent)
        assertEquals(AnalysisStep.Remove, pacer.next(completed, nowMs = 6L))
    }

    /** Confirms a completion posts the final value even when the last report was paused, and shows it running. */
    @Test
    fun completionAfterPauseShowsTheRunningFinalValue() {
        /** Pacer whose last post was paused at 90. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = 90, total = TOTAL, paused = true), nowMs = 0L)
        /** Completed snapshot that still carries the pause flag. */
        val completed = AnalysisSnapshot(done = TOTAL, total = TOTAL, paused = true, end = AnalysisEnd.COMPLETED)
        assertEquals(false, ((pacer.next(completed, nowMs = 1L)) as AnalysisStep.Post).content.paused)
    }

    /** Confirms a completion whose final value was already posted is removed without a repeated post. */
    @Test
    fun completionWithFinalAlreadyPostedOnlyRemoves() {
        /** Pacer whose last post was the complete value. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = TOTAL, total = TOTAL, paused = false), nowMs = 0L)
        /** Completed snapshot. */
        val completed = AnalysisSnapshot(done = TOTAL, total = TOTAL, paused = false, end = AnalysisEnd.COMPLETED)
        assertEquals(AnalysisStep.Remove, pacer.next(completed, nowMs = 1L))
    }

    /** Confirms a completion with no known total has no final value to post and only removes. */
    @Test
    fun completionWithoutTotalOnlyRemoves() {
        /** Pacer that posted nothing. */
        val pacer = AnalysisUpdatePacer()
        /** Completed snapshot with an unknown total. */
        val completed = AnalysisSnapshot(done = 0, total = 0, paused = false, end = AnalysisEnd.COMPLETED)
        assertEquals(AnalysisStep.Remove, pacer.next(completed, nowMs = 0L))
    }

    /** Confirms an abandoned analysis removes the notification at once without posting a final value. */
    @Test
    fun abandonRemovesWithoutAFinalPost() {
        /** Pacer that posted 50 of 100 at time zero. */
        val pacer = AnalysisUpdatePacer()
        pacer.markPosted(analysisNotificationContent(done = 50, total = TOTAL, paused = false), nowMs = 0L)
        /** Snapshot of an analysis that was stopped midway. */
        val abandoned = AnalysisSnapshot(done = 50, total = TOTAL, paused = false, end = AnalysisEnd.ABANDONED)
        assertEquals(AnalysisStep.Remove, pacer.next(abandoned, nowMs = 1L))
    }

    private companion object {
        /** Total used by the snapshots above. */
        private const val TOTAL = 100
    }
}
