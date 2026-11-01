// Cases of `runPassPerStep` when a pass fails: the failure reaches the caller and no later pass runs.

// What:     `package ...core` places the test beside `StepPasses.kt`, which declares the function.
// Why:      The test reaches `runPassPerStep` without an import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test

// What:     `class StepPassesFailureTest` groups the cases of a pass that throws.
// Why:      The page measure refresh must not hide a failing read: its caller (the host effect) decides what a failure
//           means, and a loop that swallowed it would leave the page unmeasured with nothing in the logs.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("runPassPerStep failures", () => { /* cases */ });
// ```
/** Verifies the failure contract of the one-pass-per-step loop: a pass's exception is thrown, never swallowed. */
class StepPassesFailureTest {
    /** Longest wait for the loop in these cases, so a loop that swallows the failure fails the case, not hangs it. */
    private val waitLimitMs: Long = 5_000

    // What:     `@Test fun aFailingPassThrowsOutOfTheLoop()` checks that the exception reaches the caller.
    // Why:      The steps channel stays open, so the only way out of the loop is the failure itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("pass failure propagates", async () => {
    //   await expect(runPassPerStep(steps, async () => { throw new Error("x"); })).rejects.toThrow("x");
    // });
    // ```
    /** The exception a pass throws is thrown by `runPassPerStep` itself, with its message intact. */
    @Test
    fun aFailingPassThrowsOutOfTheLoop() = runBlocking {
        /** Steps the loop collects; never closed, so only a failure can end the loop. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        steps.send(1)
        /** What the loop threw, or null when it returned or timed out first. */
        val thrown: IllegalStateException? = try {
            withTimeout(waitLimitMs) {
                runPassPerStep(steps.receiveAsFlow()) { throw IllegalStateException("pass failed") }
            }
            null
        } catch (failure: IllegalStateException) {
            failure
        }
        assertNotNull("the pass failure did not reach the caller", thrown)
        assertEquals("pass failed", thrown?.message)
    }

    // What:     `@Test fun aFailingPassStopsTheLaterSteps()` checks that no pass runs after a failure.
    // Why:      A failed read must end the effect, not retry silently on the next sweep step.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("failure stops the loop", async () => { /* send 1 and 2, pass throws, expect one call */ });
    // ```
    /** Two steps are waiting, the first pass throws, and the second step never gets a pass. */
    @Test
    fun aFailingPassStopsTheLaterSteps() = runBlocking {
        /** Steps the loop collects. */
        val steps = Channel<Int>(Channel.UNLIMITED)
        steps.send(1)
        steps.send(2)
        /** Count of passes that started. */
        val started = AtomicInteger(0)
        try {
            withTimeout(waitLimitMs) {
                runPassPerStep(steps.receiveAsFlow()) {
                    started.incrementAndGet()
                    throw IllegalStateException("pass failed")
                }
            }
        } catch (failure: IllegalStateException) {
            assertEquals("pass failed", failure.message)
        }
        assertEquals("a pass ran after the failing one", 1, started.get())
    }
}
