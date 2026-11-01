package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** D113: the library follows the disk by itself, and no change signal is lost to a running load. */
class RescanPlanTest {
    @Test
    fun aSignalWhileIdleStartsARescanAtOnce() {
        assertTrue(RescanPlan().signal())
    }

    @Test
    fun aSignalDuringALoadWaitsAndFiresOnceWhenTheLoadEnds() {
        val plan = RescanPlan()
        plan.loadStarted()
        assertFalse(plan.signal())
        assertFalse(plan.signal())
        assertTrue("one rescan is owed however many signals came", plan.loadFinished())
        assertFalse("and only one", plan.loadFinished())
    }

    @Test
    fun aLoadWithoutSignalsOwesNothing() {
        val plan = RescanPlan()
        plan.loadStarted()
        assertFalse(plan.loadFinished())
        assertTrue(plan.signal())
    }

    @Test
    fun theRescanThatIsOwedIsItselfALoadThatCanQueueAgain() {
        val plan = RescanPlan()
        plan.loadStarted()
        plan.signal()
        assertTrue(plan.loadFinished())
        plan.loadStarted()
        assertFalse(plan.signal())
        assertTrue(plan.loadFinished())
    }

    /** Returning to the front asks for a rescan once the library has loaded, and asks for none before. */
    @Test
    fun returningToTheFrontAsksForARescanOnlyAfterTheFirstLoad() {
        val plan = RescanPlan()
        assertFalse("the first load reads the disk itself", plan.foreground(libraryLoaded = false))
        assertTrue(plan.foreground(libraryLoaded = true))
    }

    /** A return that comes while a load runs is kept and answered once, like any other signal. */
    @Test
    fun aReturnDuringALoadIsAnsweredOnceWhenTheLoadEnds() {
        val plan = RescanPlan()
        plan.loadStarted()
        assertTrue(plan.foreground(libraryLoaded = true))
        assertFalse("the debounced signal waits for the load", plan.signal())
        assertTrue(plan.loadFinished())
        assertFalse(plan.loadFinished())
    }
}
