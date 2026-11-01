// Cases of the queued-load ledger: which failure number the worker publishes leads to a report, and which does not.
// Every URI here is invented.

// What:     `package dev.monochromatic.musicplayer.core` places this test beside the ledger it verifies.
// Why:      The ledger has no Android type in it, so the test needs no device.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** Verifies that a decode failure of the latest queued load is reported once, and nothing else is. */
class QueuedLoadLedgerTest {
    /** Reports nothing while the worker has failed no load. */
    @Test
    fun noFailureReportsNothing() {
        /** Ledger with one queued load. */
        val ledger = QueuedLoadLedger().also { it.queued("u1") }
        assertNull(ledger.failureToReport(0L))
    }

    /** Reports the file of the latest load when the worker fails that load. */
    @Test
    fun theFailureOfTheLatestLoadIsReportedWithItsUri() {
        /** Ledger with two queued loads. */
        val ledger = QueuedLoadLedger().also {
            it.queued("u1")
            it.queued("u2")
        }
        assertEquals("u2", ledger.failureToReport(2L))
    }

    /** The worker keeps publishing the same failure number, so every poll after the first reports nothing. */
    @Test
    fun theSameFailureIsReportedOnce() {
        /** Ledger with one queued load. */
        val ledger = QueuedLoadLedger().also { it.queued("u1") }
        assertEquals("u1", ledger.failureToReport(1L))
        assertNull(ledger.failureToReport(1L))
    }

    /** A newer request replaced the failed load, so its failure is not the user's concern any more. */
    @Test
    fun theFailureOfAReplacedLoadIsDropped() {
        /** Ledger with two queued loads, the first of which the worker failed. */
        val ledger = QueuedLoadLedger().also {
            it.queued("u1")
            it.queued("u2")
        }
        assertNull(ledger.failureToReport(1L))
        assertNull("a dropped failure is not reported later either", ledger.failureToReport(1L))
    }

    /** A load queued after a reported failure can fail and be reported on its own. */
    @Test
    fun aLaterLoadThatFailsIsReportedAfterAnEarlierOne() {
        /** Ledger with one queued load that failed. */
        val ledger = QueuedLoadLedger().also { it.queued("u1") }
        assertEquals("u1", ledger.failureToReport(1L))
        ledger.queued("u2")
        assertEquals("u2", ledger.failureToReport(2L))
    }

    /**
     * Load 1 was queued and failed, the user then asked for load 2, and load 2 never reached the native engine (its
     * descriptor could not be opened). The next poll still reads failure number 1, which equals the queued count, but
     * the user has moved on to a request that was reported on its own, so load 1 is not reported again.
     */
    @Test
    fun theFailureOfAQueuedLoadIsDroppedWhenANewerRequestNeverReachedTheEngine() {
        /** Ledger with one queued load. */
        val ledger = QueuedLoadLedger().also { it.queued("u1") }
        ledger.requested()
        assertNull(ledger.failureToReport(1L))
        assertNull("a dropped failure is not reported later either", ledger.failureToReport(1L))
    }

    /** A request that does reach the native engine is queued, so its own failure is reported. */
    @Test
    fun aRequestThatReachesTheEngineIsReportedWhenItFails() {
        /** Ledger with one queued load and a second request that then reached the engine. */
        val ledger = QueuedLoadLedger().also { it.queued("u1") }
        ledger.requested()
        ledger.queued("u2")
        assertEquals("u2", ledger.failureToReport(2L))
    }

    /** After a request that never reached the engine, the next request that does is reported when it fails. */
    @Test
    fun aLaterQueuedLoadIsReportedAfterARequestThatNeverReachedTheEngine() {
        /** Ledger with a queued load, a request that stayed out and a third load that was queued. */
        val ledger = QueuedLoadLedger().also { it.queued("u1") }
        ledger.requested()
        ledger.requested()
        ledger.queued("u3")
        assertEquals("u3", ledger.failureToReport(2L))
    }
}
