package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** M9 and D113: new, replaced, edited and failed files decide what the analysis does and when it starts. */
class SweepLedgerTest {
    /** Content URIs of [count] tracks. */
    private fun uris(count: Int): List<String> =
        (0 until count).map { index -> "content://media/external/audio/media/$index" }

    /** Signature function for libraries whose scan reported nothing. */
    private val noSignature: (String) -> Long = { 0L }

    /** Pairs each URI with signature [signature], the shape [SweepLedger.record] takes. */
    private fun pairs(list: List<String>, signature: Long = 0L): List<Pair<String, Long>> =
        list.map { uri -> uri to signature }

    @Test
    fun anEmptyLedgerHasEveryTrackToAnalyze() {
        val library = uris(5)
        val plan = SweepLedger.EMPTY.plan(library, noSignature, retryFailed = false)
        assertEquals(library, plan.todo)
        assertEquals(5, plan.fresh)
        assertTrue(shouldStartAnalysis(sourceReadable = true, plan = plan))
    }

    @Test
    fun aLibraryThatGainedTracksAfterTheFirstSweepStartsAgainForTheNewOnesOnly() {
        val first = uris(150)
        val afterFirstSweep = SweepLedger.EMPTY.record(pairs(first), emptyList()).ledger
        assertFalse(shouldStartAnalysis(true, afterFirstSweep.plan(first, noSignature, retryFailed = false)))
        val grown = first + uris(153).drop(150)
        val plan = afterFirstSweep.plan(grown, noSignature, retryFailed = false)
        assertEquals(uris(153).drop(150), plan.todo)
        assertTrue(shouldStartAnalysis(sourceReadable = true, plan = plan))
    }

    @Test
    fun aReplacedOrEditedFileKeepsItsUriButIsNewWorkBecauseItsSignatureChanged() {
        val library = uris(3)
        val ledger = SweepLedger.EMPTY.record(pairs(library, signature = 111L), emptyList()).ledger
        assertTrue(ledger.plan(library, { 111L }, retryFailed = false).todo.isEmpty())
        val edited = ledger.plan(library, { uri -> if (uri == library[1]) 222L else 111L }, retryFailed = false)
        assertEquals(listOf(library[1]), edited.todo)
        assertEquals(1, edited.fresh)
    }

    @Test
    fun aRemovedTrackNeverStartsASweep() {
        val ledger = SweepLedger.EMPTY.record(pairs(uris(10)), emptyList()).ledger
        assertTrue(ledger.plan(uris(9), noSignature, retryFailed = true).todo.isEmpty())
    }

    @Test
    fun nothingStartsWithoutAReadableSourceOrWithoutWork() {
        val work = SweepLedger.EMPTY.plan(uris(3), noSignature, retryFailed = false)
        assertFalse(shouldStartAnalysis(sourceReadable = false, plan = work))
        val none = SweepLedger.EMPTY.record(pairs(uris(3)), emptyList()).ledger.plan(uris(3), noSignature, true)
        assertFalse(shouldStartAnalysis(sourceReadable = true, plan = none))
    }

    @Test
    fun aFailedFileIsReportedOnceAndRetriedOnlyAtStartOrWhenTheLibraryChanges() {
        val library = uris(4)
        val bad = library[2]
        val good = library - bad
        val first = SweepLedger.EMPTY.record(pairs(good), listOf(bad to 0L))
        assertEquals(listOf(bad), first.newFailures)
        // The library did not change: no start, however often it is rescanned.
        assertTrue(first.ledger.plan(library, noSignature, retryFailed = false).todo.isEmpty())
        // The app starts again: the failure is tried again.
        val atStart = first.ledger.plan(library, noSignature, retryFailed = true)
        assertEquals(listOf(bad), atStart.todo)
        assertEquals(1, atStart.retried)
        // It fails again: it is not reported a second time.
        val second = first.ledger.record(emptyList(), listOf(bad to 0L))
        assertTrue(second.newFailures.isEmpty())
        // The library gains a file: the failure rides along with the new work.
        val grown = library + "content://media/external/audio/media/99"
        val whenChanged = second.ledger.plan(grown, noSignature, retryFailed = false)
        assertEquals(listOf("content://media/external/audio/media/99", bad), whenChanged.todo)
        assertEquals(1, whenChanged.fresh)
        assertEquals(1, whenChanged.retried)
    }

    @Test
    fun aFileThatDecodesOnRetryLeavesTheFailedSet() {
        val bad = "content://media/external/audio/media/7"
        val failed = SweepLedger.EMPTY.record(emptyList(), listOf(bad to 5L)).ledger
        val healed = failed.record(listOf(bad to 5L), emptyList()).ledger
        assertEquals(0, healed.failedCount)
        assertEquals(1, healed.sweptCount)
        assertTrue(healed.plan(listOf(bad), { 5L }, retryFailed = true).todo.isEmpty())
    }

    @Test
    fun aFailedFileThatIsEditedIsReportedAgainBecauseItIsAnotherFile() {
        val bad = "content://media/external/audio/media/7"
        val failed = SweepLedger.EMPTY.record(emptyList(), listOf(bad to 5L)).ledger
        val plan = failed.plan(listOf(bad), { 6L }, retryFailed = false)
        assertEquals(listOf(bad), plan.todo)
        assertEquals(1, plan.fresh)
        assertEquals(listOf(bad), failed.record(emptyList(), listOf(bad to 6L)).newFailures)
    }

    @Test
    fun theLedgerSurvivesTheRoundTripThroughItsBytes() {
        val ledger = SweepLedger.EMPTY.record(pairs(uris(1000), 9L), listOf("content://bad" to 1L)).ledger
        val restored = SweepLedger.fromBytes(ledger.toBytes())
        assertEquals(1000, restored.sweptCount)
        assertEquals(1, restored.failedCount)
        assertTrue(restored.plan(uris(1000), { 9L }, retryFailed = false).todo.isEmpty())
        assertEquals(listOf("content://other/1"), restored.plan(listOf("content://other/1"), { 9L }, false).todo)
    }

    @Test
    fun aDamagedOrOlderFileGivesAnEmptyLedgerSoEverythingIsAnalyzedAgain() {
        assertEquals(0, SweepLedger.fromBytes(ByteArray(13)).sweptCount)
        assertEquals(0, SweepLedger.fromBytes(ByteArray(16)).sweptCount)
        val good = SweepLedger.EMPTY.record(pairs(uris(5)), emptyList()).ledger.toBytes()
        assertEquals(0, SweepLedger.fromBytes(good.copyOf(good.size - 3)).sweptCount)
    }

    @Test
    fun addingTheSameTracksTwiceKeepsOneEntryEach() {
        val once = SweepLedger.EMPTY.record(pairs(uris(20)), emptyList()).ledger
        assertEquals(20, once.record(pairs(uris(20)), emptyList()).ledger.sweptCount)
    }
}
