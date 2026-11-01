package dev.monochromatic.musicplayer.core

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.atomic.AtomicInteger

/** Verifies the pause gate on its own: fast path, blocking, resume, cancellation and the observable flag. */
class SweepPauseGateTest {
    /** Confirms a gate that was never paused lets a worker straight through. */
    @Test
    fun openGateReturnsImmediately() = runBlocking {
        /** Gate that was never paused. */
        val gate = SweepPauseGate()
        withTimeout(WAIT_MS) { gate.awaitResumed() }
        assertFalse(gate.paused.value)
    }

    /** Confirms a paused gate holds a worker until resume, and then lets it through. */
    @Test
    fun pausedGateHoldsUntilResume() = runBlocking {
        /** Gate that starts paused. */
        val gate = SweepPauseGate()
        gate.pause()
        /** Count of workers that passed the gate. */
        val passed = AtomicInteger(0)
        /** Worker waiting at the gate. */
        val worker = launch(Dispatchers.Default) {
            gate.awaitResumed()
            passed.incrementAndGet()
        }
        delay(SETTLE_MS)
        assertEquals("a paused gate must hold the worker", 0, passed.get())
        gate.resume()
        withTimeout(WAIT_MS) { worker.join() }
        assertEquals(1, passed.get())
    }

    /** Confirms cancelling a worker that waits at a paused gate ends it instead of hanging. */
    @Test
    fun cancelWhilePausedDoesNotHang() = runBlocking {
        /** Gate that stays paused for the whole test. */
        val gate = SweepPauseGate()
        gate.pause()
        /** Worker that waits at the gate. */
        val worker = launch(Dispatchers.Default) { gate.awaitResumed() }
        delay(SETTLE_MS)
        withTimeout(WAIT_MS) { worker.cancelAndJoin() }
        assertTrue(worker.isCancelled)
    }

    /** Confirms pause and resume are idempotent and the observable flag follows them. */
    @Test
    fun pauseAndResumeAreIdempotent() {
        /** Gate under test. */
        val gate = SweepPauseGate()
        gate.pause()
        gate.pause()
        assertTrue(gate.paused.value)
        gate.resume()
        gate.resume()
        assertFalse(gate.paused.value)
    }

    /** Confirms every one of several waiting workers is released by one resume. */
    @Test
    fun resumeReleasesEveryWaitingWorker() = runBlocking {
        /** Gate that starts paused. */
        val gate = SweepPauseGate()
        gate.pause()
        /** Workers that wait at the gate, started without delay so they all reach it. */
        val workers = List(WORKERS) { async(Dispatchers.Default) { gate.awaitResumed() } }
        delay(SETTLE_MS)
        gate.resume()
        withTimeout(WAIT_MS) { workers.forEach { worker -> worker.await() } }
    }

    private companion object {
        /** Number of waiting workers in the release test. */
        private const val WORKERS = 6

        /** Time for a worker to reach the gate, in milliseconds. */
        private const val SETTLE_MS = 80L

        /** Upper bound for something that must finish, in milliseconds. */
        private const val WAIT_MS = 5_000L
    }
}
