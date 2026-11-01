package dev.monochromatic.musicplayer

import dev.monochromatic.musicplayer.SessionPositionAnchor.Check
import org.junit.Assert.assertEquals
import org.junit.Test

/** The session position is repaired when the engine moves away from it, and left alone while the two agree. */
class SessionPositionAnchorTest {
    /** Clock the test advances by hand, in milliseconds. */
    private var clockMs: Long = 10_000L

    private val anchor = SessionPositionAnchor(now = { clockMs })

    @Test
    fun anAdvancingAnchorExpectsThePositionToRunWithTheClock() {
        anchor.record(positionMs = 1_000L, advancing = true)
        clockMs += 3_000L
        assertEquals(4_000L, anchor.expectedMs())
    }

    @Test
    fun aPausedAnchorExpectsTheSamePositionHoweverMuchTimePasses() {
        anchor.record(positionMs = 1_000L, advancing = false)
        clockMs += 30_000L
        assertEquals(1_000L, anchor.expectedMs())
    }

    @Test
    fun anEngineThatAgreesWhilePlayingIsCheckedAgainEverySecond() {
        anchor.record(positionMs = 8_000L, advancing = true)
        clockMs += 250L
        assertEquals(Check.CheckAgainIn(1_000L), anchor.check(actualMs = 8_250L))
    }

    @Test
    fun aSeekTheWorkerAppliedAfterThePullIsRepairedWhilePlaying() {
        // getState() read 8 s, then the worker moved the playhead to 77 s.
        anchor.record(positionMs = 8_000L, advancing = true)
        clockMs += 250L
        assertEquals(Check.Resync, anchor.check(actualMs = 77_000L))
    }

    @Test
    fun aSeekTheWorkerAppliedAfterThePullIsRepairedWhilePaused() {
        anchor.record(positionMs = 8_000L, advancing = false)
        clockMs += 250L
        assertEquals(Check.Resync, anchor.check(actualMs = 77_000L))
    }

    @Test
    fun aSeekAppliedLateIsStillCaughtByALaterCheckWhilePaused() {
        anchor.record(positionMs = 8_000L, advancing = false)
        clockMs += 250L
        assertEquals(Check.CheckAgainIn(1_000L), anchor.check(actualMs = 8_000L))
        clockMs += 1_000L
        assertEquals(Check.Resync, anchor.check(actualMs = 77_000L))
    }

    @Test
    fun aPausedAnchorStopsBeingCheckedAfterTheSettleChecks() {
        anchor.record(positionMs = 8_000L, advancing = false)
        assertEquals(Check.CheckAgainIn(1_000L), anchor.check(actualMs = 8_000L))
        assertEquals(Check.CheckAgainIn(1_000L), anchor.check(actualMs = 8_000L))
        assertEquals(Check.Settled, anchor.check(actualMs = 8_000L))
    }

    @Test
    fun aNewRecordRestartsTheSettleChecks() {
        anchor.record(positionMs = 8_000L, advancing = false)
        repeat(SessionPositionAnchor.SETTLE_CHECKS) { anchor.check(actualMs = 8_000L) }
        anchor.record(positionMs = 9_000L, advancing = false)
        assertEquals(Check.CheckAgainIn(1_000L), anchor.check(actualMs = 9_000L))
    }

    @Test
    fun aStallWhilePlayingIsRepairedOnceTheEngineFallsBehindTheClock() {
        anchor.record(positionMs = 1_000L, advancing = true)
        clockMs += 3_000L
        // The engine stalled for 1.5 s: it reports 2.5 s where the clock says 4 s.
        assertEquals(Check.Resync, anchor.check(actualMs = 2_500L))
    }

    @Test
    fun aDifferenceUpToTheThresholdIsNotRepaired() {
        anchor.record(positionMs = 1_000L, advancing = true)
        clockMs += 3_000L
        val threshold = SessionPositionAnchor.DRIFT_THRESHOLD_MS
        assertEquals(Check.CheckAgainIn(1_000L), anchor.check(actualMs = 4_000L + threshold))
        assertEquals(Check.CheckAgainIn(1_000L), anchor.check(actualMs = 4_000L - threshold))
        assertEquals(Check.Resync, anchor.check(actualMs = 4_000L + threshold + 1L))
        assertEquals(Check.Resync, anchor.check(actualMs = 4_000L - threshold - 1L))
    }

    @Test
    fun aLoadingTrackThatHoldsTheEngineAtZeroIsRepairedAfterTheThresholdPasses() {
        anchor.record(positionMs = 0L, advancing = true)
        clockMs += 250L
        assertEquals(Check.CheckAgainIn(1_000L), anchor.check(actualMs = 0L))
        clockMs += 1_000L
        assertEquals(Check.Resync, anchor.check(actualMs = 0L))
    }
}
