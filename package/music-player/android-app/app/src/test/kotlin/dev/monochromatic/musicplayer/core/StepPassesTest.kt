// Cases of `runPassPerStep`: one pass per step, a running pass is never cancelled, steps that arrive meanwhile merge.

// What:     `package ...core` places the test beside `StepPasses.kt`, which declares the function.
// Why:      The test reaches `runPassPerStep` without an import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Job
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class StepPassesTest` groups the cases of the one-pass-per-step loop.
// Why:      The page measure refresh depends on three properties: a step never cancels a pass, steps during a pass
//           merge into one follow-up pass, and no step is lost.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("runPassPerStep", () => { /* cases */ });
// ```
/** Verifies the pass scheduling that keeps a sweep step from cancelling the page measure read in flight. */
class StepPassesTest {
    /** Longest wait for a condition in these cases, so a broken loop fails the case instead of hanging it. */
    private val waitLimitMs: Long = 5_000

    /** Waits until [condition] holds, polling in short sleeps, and fails the case when the limit passes first. */
    private suspend fun waitUntil(what: String, condition: () -> Boolean) {
        try {
            withTimeout(waitLimitMs) {
                while (!condition()) {
                    delay(5)
                }
            }
        } catch (timeout: TimeoutCancellationException) {
            throw AssertionError("Timed out waiting for $what.", timeout)
        }
    }

    /** Steps that arrive on demand, sent with [Channel.send], as a flow the loop collects. */
    private fun stepsOf(channel: Channel<Int>): Flow<Int> = channel.receiveAsFlow()

    // What:     `@Test fun aStepArrivingMidPassDoesNotCancelThePass()` checks the property the sweep fix rests on.
    // Why:      The old effect restarted on every sweep step and discarded the batch being read, so a pass that took
    //           longer than the gap between steps never published.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a mid-pass step does not cancel", async () => { /* start pass, send step, release, expect finished */ });
    // ```
    /** A pass that is waiting when a newer step arrives still runs to its end. */
    @Test
    fun aStepArrivingMidPassDoesNotCancelThePass() = runBlocking {
        /** Steps the loop collects. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        /** Opens when the first pass may finish. */
        val release = CompletableDeferred<Unit>()
        /** Count of passes that started. */
        val started = AtomicInteger(0)
        /** Count of passes that ran to their end. */
        val finished = AtomicInteger(0)
        /** The loop under test. */
        val loop: Job = launch {
            runPassPerStep(stepsOf(steps)) {
                started.incrementAndGet()
                release.await()
                finished.incrementAndGet()
            }
        }
        steps.send(1)
        waitUntil("the first pass to start") { started.get() == 1 }
        steps.send(2)
        delay(100)
        assertEquals("the pass was cancelled or restarted by the step", 1, started.get())
        assertEquals(0, finished.get())
        release.complete(Unit)
        waitUntil("the first pass to finish") { finished.get() >= 1 }
        steps.close()
        loop.join()
        assertEquals("the first pass must have finished, not been cancelled", 2, finished.get())
    }

    // What:     `@Test fun stepsDuringAPassMergeIntoOneFollowUpPass()` checks the conflation.
    // Why:      Ten sweep steps during one slow pass must cost one more pass, not ten, or the reads would never catch up.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("steps merge", async () => { /* send 2,3,4,5 during pass 1; expect exactly two passes */ });
    // ```
    /** Several steps that arrive during one pass produce exactly one pass after it. */
    @Test
    fun stepsDuringAPassMergeIntoOneFollowUpPass() = runBlocking {
        /** Steps the loop collects. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        /** Opens when the first pass may finish. */
        val release = CompletableDeferred<Unit>()
        /** Count of passes that started. */
        val started = AtomicInteger(0)
        /** The loop under test. */
        val loop: Job = launch {
            runPassPerStep(stepsOf(steps)) {
                started.incrementAndGet()
                release.await()
            }
        }
        steps.send(1)
        waitUntil("the first pass to start") { started.get() == 1 }
        for (step in 2..6) {
            steps.send(step)
        }
        delay(100)
        release.complete(Unit)
        delay(200)
        steps.close()
        loop.join()
        assertEquals("five steps during one pass must merge into one follow-up pass", 2, started.get())
    }

    // What:     `@Test fun aLoneStepRunsExactlyOnePass()` checks that nothing runs without a step.
    // Why:      The loop must not spin: a pass runs for a step, and an idle sweep means an idle loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one step one pass", async () => { /* send 1; wait; expect one pass */ });
    // ```
    /** One step gives one pass, and no further pass runs while no step arrives. */
    @Test
    fun aLoneStepRunsExactlyOnePass() = runBlocking {
        /** Steps the loop collects. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        /** Count of passes that ran to their end. */
        val finished = AtomicInteger(0)
        /** The loop under test. */
        val loop: Job = launch { runPassPerStep(stepsOf(steps)) { finished.incrementAndGet() } }
        steps.send(1)
        waitUntil("the pass to finish") { finished.get() == 1 }
        delay(150)
        assertEquals("a pass ran without a step asking for it", 1, finished.get())
        steps.close()
        loop.join()
    }

    // What:     `@Test fun stepsSeparatedByFinishedPassesEachRunAPass()` checks that no step is lost.
    // Why:      A sweep step after a finished pass must re-check the page, since the sweep may have filled new peaks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("each separated step runs a pass", async () => { /* send, wait, send, wait, send, wait; expect three */ });
    // ```
    /** Steps that each arrive after the previous pass finished each get their own pass. */
    @Test
    fun stepsSeparatedByFinishedPassesEachRunAPass() = runBlocking {
        /** Steps the loop collects. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        /** Count of passes that ran to their end. */
        val finished = AtomicInteger(0)
        /** The loop under test. */
        val loop: Job = launch { runPassPerStep(stepsOf(steps)) { finished.incrementAndGet() } }
        for (step in 1..3) {
            steps.send(step)
            waitUntil("pass number $step to finish") { finished.get() == step }
        }
        steps.close()
        loop.join()
        assertEquals(3, finished.get())
    }

    // What:     `@Test fun cancellingTheCallerCancelsTheRunningPass()` checks the cleanup path.
    // Why:      A page change cancels the effect, and the pass for the old page must stop, not run on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("abort stops the pass", async () => { /* start pass, abort, expect the pass saw the abort */ });
    // ```
    /** Cancelling the collector cancels the pass that is running. */
    @Test
    fun cancellingTheCallerCancelsTheRunningPass() = runBlocking {
        /** Steps the loop collects. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        /** Count of passes that started. */
        val started = AtomicInteger(0)
        /** Whether the running pass saw the cancellation. */
        val cancelled = AtomicInteger(0)
        /** The loop under test. */
        val loop: Job = launch {
            runPassPerStep(stepsOf(steps)) {
                started.incrementAndGet()
                try {
                    delay(60_000)
                } catch (cancellation: CancellationException) {
                    cancelled.incrementAndGet()
                    throw cancellation
                }
            }
        }
        steps.send(1)
        waitUntil("the pass to start") { started.get() == 1 }
        loop.cancelAndJoin()
        assertTrue("the pass did not see the cancellation", cancelled.get() == 1)
    }
}
