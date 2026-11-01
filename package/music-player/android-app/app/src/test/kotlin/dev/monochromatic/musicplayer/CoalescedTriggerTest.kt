package dev.monochromatic.musicplayer

import org.junit.Assert.assertEquals
import org.junit.Test

/** Many brain changes in a row queue one media session refresh, and a change after the refresh queues the next. */
class CoalescedTriggerTest {
    /** Scheduler that holds tasks until the test runs them, like a looper that has not turned yet. */
    private class Held {
        val tasks: MutableList<() -> Unit> = ArrayList()

        fun schedule(task: () -> Unit) {
            tasks.add(task)
        }

        fun runAll() {
            val batch = tasks.toList()
            tasks.clear()
            batch.forEach { task -> task() }
        }
    }

    @Test
    fun aBurstOfRequestsSchedulesOneTaskAndRunsTheActionOnce() {
        val held = Held()
        var runs = 0
        val trigger = CoalescedTrigger(schedule = held::schedule, action = { runs += 1 })
        repeat(5) { trigger.request() }
        assertEquals(1, held.tasks.size)
        assertEquals(0, runs)
        held.runAll()
        assertEquals(1, runs)
    }

    @Test
    fun aRequestAfterTheRunSchedulesAgain() {
        val held = Held()
        var runs = 0
        val trigger = CoalescedTrigger(schedule = held::schedule, action = { runs += 1 })
        trigger.request()
        held.runAll()
        trigger.request()
        assertEquals(1, held.tasks.size)
        held.runAll()
        assertEquals(2, runs)
    }

    @Test
    fun aRequestMadeByTheActionItselfSchedulesAnotherRun() {
        val held = Held()
        var runs = 0
        lateinit var trigger: CoalescedTrigger
        trigger = CoalescedTrigger(
            schedule = held::schedule,
            action = {
                runs += 1
                if (runs == 1) {
                    trigger.request()
                }
            },
        )
        trigger.request()
        held.runAll()
        assertEquals(1, held.tasks.size)
        held.runAll()
        assertEquals(2, runs)
    }

    @Test
    fun aFailingActionDoesNotBlockLaterRequests() {
        val held = Held()
        var fail = true
        var runs = 0
        val trigger = CoalescedTrigger(
            schedule = held::schedule,
            action = {
                runs += 1
                if (fail) {
                    throw IllegalStateException("refresh failed")
                }
            },
        )
        trigger.request()
        try {
            held.runAll()
        } catch (expected: IllegalStateException) {
            fail = false
        }
        trigger.request()
        held.runAll()
        assertEquals(2, runs)
    }
}
