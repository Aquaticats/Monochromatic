package dev.monochromatic.musicplayer.core

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicInteger

/** Verifies the publisher loop with a recording sink and a short real rate limit window. */
class AnalysisNotificationPublisherTest {
    /** Recording sink for posts and removals, with the gate and pacer wired as the service wires them. */
    private class Rig(window: Long = WINDOW_MS) {
        /** Pause gate the publisher mirrors. */
        val gate = SweepPauseGate()

        /** Contents posted, in order. */
        val posts = CopyOnWriteArrayList<AnalysisNotificationContent>()

        /** Number of removals. */
        val removals = AtomicInteger(0)

        /** Pacer that knows the indeterminate first frame was posted by the service, as in production. */
        private val pacer = AnalysisUpdatePacer(minIntervalMs = window).also { started ->
            started.markPosted(
                content = analysisNotificationContent(done = 0, total = 0, paused = false),
                nowMs = System.nanoTime() / NANOS_PER_MS,
            )
        }

        /** Publisher under test. */
        val publisher = AnalysisNotificationPublisher(
            paused = gate.paused,
            pacer = pacer,
            clockMs = { System.nanoTime() / NANOS_PER_MS },
            post = { content -> posts.add(content) },
            remove = { removals.incrementAndGet() },
        )
    }

    /** Confirms a burst of reports inside the window is posted once, as the newest value, after the window. */
    @Test
    fun burstIsPostedOnceAsTheNewestValue() = runBlocking {
        /** Rig with a window long enough to hold the whole burst. */
        val rig = Rig()
        /** Loop under test. */
        val loop = launch(Dispatchers.Default) { rig.publisher.run() }
        repeat(BURST) { index -> rig.publisher.report(done = index + 1, total = TOTAL) }
        delay(WINDOW_MS * 2)
        rig.publisher.abandon()
        withTimeout(WAIT_MS) { loop.join() }
        assertEquals(listOf("$BURST of 100"), rig.posts.map { content -> content.text })
        assertEquals(1, rig.removals.get())
    }

    /** Confirms a report that arrives in the window is still posted after the window, without a later report. */
    @Test
    fun lastValueArrivesAfterTheWindowWithoutMoreReports() = runBlocking {
        /** Rig with the default window. */
        val rig = Rig()
        /** Loop under test. */
        val loop = launch(Dispatchers.Default) { rig.publisher.run() }
        rig.publisher.report(done = 1, total = TOTAL)
        delay(WINDOW_MS * 2)
        rig.publisher.report(done = 2, total = TOTAL)
        delay(WINDOW_MS * 2)
        assertEquals(listOf("1 of 100", "2 of 100"), rig.posts.map { content -> content.text })
        rig.publisher.abandon()
        withTimeout(WAIT_MS) { loop.join() }
    }

    /** Confirms out-of-order reports within one total never move the count backwards. */
    @Test
    fun lowerDoneNeverMovesTheCountBack() {
        /** Rig whose loop is not running; the content is read directly. */
        val rig = Rig()
        rig.publisher.report(done = 30, total = TOTAL)
        rig.publisher.report(done = 20, total = TOTAL)
        assertEquals("30 of 100", rig.publisher.currentContent().text)
        rig.publisher.report(done = 5, total = TOTAL + 1)
        assertEquals("5 of 101", rig.publisher.currentContent().text)
    }

    /** Confirms a pause is posted at once with the Resume state and a resume flips it back, with no progress. */
    @Test
    fun pauseAndResumeArePostedWithoutProgress() = runBlocking {
        /** Rig with a window long enough that only the pause bypass can explain a quick post. */
        val rig = Rig(window = LONG_WINDOW_MS)
        /** Loop under test. */
        val loop = launch(Dispatchers.Default) { rig.publisher.run() }
        rig.publisher.report(done = 10, total = TOTAL)
        delay(SETTLE_MS)
        rig.gate.pause()
        delay(SETTLE_MS)
        assertTrue("pause must be posted inside the window", rig.posts.last().paused)
        assertEquals("the pause post carries the newest count", "10 of 100", rig.posts.last().text)
        rig.gate.resume()
        delay(SETTLE_MS)
        assertFalse("resume must be posted inside the window", rig.posts.last().paused)
        rig.publisher.abandon()
        withTimeout(WAIT_MS) { loop.join() }
    }

    /** Confirms completion posts the final value even inside the window and then removes the notification. */
    @Test
    fun completionPostsFinalValueThenRemoves() = runBlocking {
        /** Rig with a window long enough to hold the final report. */
        val rig = Rig(window = LONG_WINDOW_MS)
        /** Loop under test. */
        val loop = launch(Dispatchers.Default) { rig.publisher.run() }
        rig.publisher.report(done = 10, total = TOTAL)
        delay(SETTLE_MS)
        rig.publisher.report(done = TOTAL, total = TOTAL)
        rig.publisher.complete()
        withTimeout(WAIT_MS) { loop.join() }
        assertEquals("100 of 100", rig.posts.last().text)
        assertEquals(100, rig.posts.last().percent)
        assertEquals(1, rig.removals.get())
    }

    /** Confirms abandoning removes the notification without a final post and never downgrades a completion. */
    @Test
    fun abandonRemovesAndDoesNotCancelACompletion() = runBlocking {
        /** Rig for an abandoned run, with a window that cannot close during the test. */
        val abandoned = Rig(window = LONG_WINDOW_MS)
        /** Loop under test for the abandoned run. */
        val loop = launch(Dispatchers.Default) { abandoned.publisher.run() }
        abandoned.publisher.report(done = 10, total = TOTAL)
        delay(SETTLE_MS)
        abandoned.publisher.abandon()
        withTimeout(WAIT_MS) { loop.join() }
        assertEquals("a value inside the window is not posted", emptyList<String>(), abandoned.posts.map { it.text })
        assertEquals(1, abandoned.removals.get())
        /** Rig for a run that completes and is then abandoned by a cleanup block. */
        val completed = Rig()
        completed.publisher.report(done = TOTAL, total = TOTAL)
        completed.publisher.complete()
        completed.publisher.abandon()
        withTimeout(WAIT_MS) { completed.publisher.run() }
        assertEquals("100 of 100", completed.posts.last().text)
    }

    /** Confirms reports only write a value: a sink that blocks cannot delay the caller of report. */
    @Test
    fun reportingDoesNotWaitForTheSink() = runBlocking {
        /** Release signal for a sink that blocks until released. */
        val release = java.util.concurrent.CountDownLatch(1)
        /** Publisher whose sink blocks. */
        val publisher = AnalysisNotificationPublisher(
            paused = SweepPauseGate().paused,
            pacer = AnalysisUpdatePacer(minIntervalMs = WINDOW_MS),
            clockMs = { System.nanoTime() / NANOS_PER_MS },
            post = { release.await() },
            remove = { },
        )
        /** Loop stuck inside the blocking sink after the first report. */
        val loop = launch(Dispatchers.IO) { publisher.run() }
        publisher.report(done = 1, total = TOTAL)
        delay(SETTLE_MS)
        /** Time spent reporting while the sink is blocked, in milliseconds. */
        val started = System.nanoTime()
        repeat(REPORTS_WHILE_BLOCKED) { index -> publisher.report(done = index + 2, total = TOTAL) }
        val elapsedMs = (System.nanoTime() - started) / NANOS_PER_MS
        assertTrue("reports took $elapsedMs ms while the sink blocked", elapsedMs < WAIT_MS)
        publisher.abandon()
        release.countDown()
        withTimeout(WAIT_MS) { loop.join() }
    }

    private companion object {
        /** Total used by the reports above. */
        private const val TOTAL = 100

        /** Number of reports in the burst. */
        private const val BURST = 20

        /** Number of reports sent while the sink blocks. */
        private const val REPORTS_WHILE_BLOCKED = 50

        /** Short rate limit window for tests, in milliseconds. */
        private const val WINDOW_MS = 150L

        /** Window long enough that only a bypass rule can post inside it, in milliseconds. */
        private const val LONG_WINDOW_MS = 60_000L

        /** Time for the loop to react to a change, in milliseconds. */
        private const val SETTLE_MS = 60L

        /** Upper bound for something that must finish, in milliseconds. */
        private const val WAIT_MS = 5_000L

        /** Nanoseconds in a millisecond. */
        private const val NANOS_PER_MS = 1_000_000L
    }
}
