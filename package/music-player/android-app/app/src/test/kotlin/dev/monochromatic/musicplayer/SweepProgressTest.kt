// What:     `package dev.monochromatic.musicplayer` places this test beside the sweep progress it checks.
// Why:      Same-package tests call `sweepProgressMerge` and `SWEEP_PROGRESS_IDLE` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     JUnit assertions register each merge check.
// Why:      Every reported count is compared with the exact snapshot it must produce.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class SweepProgressTest` groups the merge checks for the sweep's reported counts.
// Why:      The parallel workers report out of order, so the merge rule decides what the scan indicator shows.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("sweepProgressMerge", () => { ... });
// ```
/** Verifies the merge rule for the sweep's reported counts: first report, order, clamping, and totals. */
class SweepProgressTest {
    // What:     `fun firstReportTakesItsCounts` checks the first report from the idle snapshot.
    // Why:      The service reports zero of the total before any flush, so the first report sets both counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("first", () => expect(merge(IDLE, 0, 5)).toEqual({ done: 0, total: 5 }));
    // ```
    /** Confirms that the first report from the idle snapshot sets the done and total counts. */
    @Test
    fun firstReportTakesItsCounts() {
        assertEquals(
            SweepProgressSnapshot(done = 0, total = 5),
            sweepProgressMerge(current = SWEEP_PROGRESS_IDLE, reportedDone = 0, reportedTotal = 5),
        )
    }

    // What:     `fun lowerDoneDoesNotMoveBarBack` checks a late report with a smaller done count.
    // Why:      Workers report out of order, and a late smaller count must not move the bar backwards.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("out of order", () => expect(merge({ done: 3, total: 5 }, 2, 5)).toEqual({ done: 3, total: 5 }));
    // ```
    /** Confirms that a report with a smaller done count keeps the larger count already shown. */
    @Test
    fun lowerDoneDoesNotMoveBarBack() {
        assertEquals(
            SweepProgressSnapshot(done = 3, total = 5),
            sweepProgressMerge(
                current = SweepProgressSnapshot(done = 3, total = 5),
                reportedDone = 2,
                reportedTotal = 5,
            ),
        )
    }

    // What:     `fun higherDoneAdvancesBar` checks a report with a larger done count.
    // Why:      A newer report with more visited tracks must move the bar forward.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("advance", () => expect(merge({ done: 3, total: 5 }, 4, 5)).toEqual({ done: 4, total: 5 }));
    // ```
    /** Confirms that a report with a larger done count advances the shown count. */
    @Test
    fun higherDoneAdvancesBar() {
        assertEquals(
            SweepProgressSnapshot(done = 4, total = 5),
            sweepProgressMerge(
                current = SweepProgressSnapshot(done = 3, total = 5),
                reportedDone = 4,
                reportedTotal = 5,
            ),
        )
    }

    // What:     `fun overshootClampsToTotal` checks a done count above the total.
    // Why:      The bar must never show more visited tracks than the total it counts against.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("clamp high", () => expect(merge(IDLE, 9, 5)).toEqual({ done: 5, total: 5 }));
    // ```
    /** Confirms that a done count above the total is clamped to the total. */
    @Test
    fun overshootClampsToTotal() {
        assertEquals(
            SweepProgressSnapshot(done = 5, total = 5),
            sweepProgressMerge(current = SWEEP_PROGRESS_IDLE, reportedDone = 9, reportedTotal = 5),
        )
    }

    // What:     `fun negativeDoneClampsToZero` checks a negative done count.
    // Why:      A negative visited count is impossible, so the bar must start at zero.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("clamp low", () => expect(merge(IDLE, -3, 5)).toEqual({ done: 0, total: 5 }));
    // ```
    /** Confirms that a negative done count is clamped to zero. */
    @Test
    fun negativeDoneClampsToZero() {
        assertEquals(
            SweepProgressSnapshot(done = 0, total = 5),
            sweepProgressMerge(current = SWEEP_PROGRESS_IDLE, reportedDone = -3, reportedTotal = 5),
        )
    }

    // What:     `fun negativeTotalBecomesZero` checks a negative total.
    // Why:      An unknown total must read as zero, which keeps the indicator hidden.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("negative total", () => expect(merge(IDLE, 2, -1)).toEqual({ done: 0, total: 0 }));
    // ```
    /** Confirms that a negative total becomes zero, with the done count clamped to it. */
    @Test
    fun negativeTotalBecomesZero() {
        assertEquals(
            SweepProgressSnapshot(done = 0, total = 0),
            sweepProgressMerge(current = SWEEP_PROGRESS_IDLE, reportedDone = 2, reportedTotal = -1),
        )
    }

    // What:     `fun newTotalStartsNewCount` checks a report whose total differs from the current one.
    // Why:      A new sweep has its own count, so a larger done count from the old total must not carry over.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("new total", () => expect(merge({ done: 4, total: 5 }, 1, 9)).toEqual({ done: 1, total: 9 }));
    // ```
    /** Confirms that a different total starts a new count without keeping the old done count. */
    @Test
    fun newTotalStartsNewCount() {
        assertEquals(
            SweepProgressSnapshot(done = 1, total = 9),
            sweepProgressMerge(
                current = SweepProgressSnapshot(done = 4, total = 5),
                reportedDone = 1,
                reportedTotal = 9,
            ),
        )
    }

    // What:     `fun zeroTotalIsIdle` checks a report with a zero total.
    // Why:      A zero total means the sweep has nothing to visit, which the indicator treats as idle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("zero total", () => expect(merge(IDLE, 0, 0)).toEqual(IDLE));
    // ```
    /** Confirms that a zero total gives the idle snapshot. */
    @Test
    fun zeroTotalIsIdle() {
        assertEquals(
            SWEEP_PROGRESS_IDLE,
            sweepProgressMerge(current = SWEEP_PROGRESS_IDLE, reportedDone = 0, reportedTotal = 0),
        )
    }

    // What:     `fun finishedSweepReadsDoneOfTotal` checks the report that the service makes when the sweep returns.
    // Why:      When the sweep returns, the done count equals the total, which the indicator reads as finished.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("finished", () => expect(merge({ done: 4, total: 5 }, 5, 5)).toEqual({ done: 5, total: 5 }));
    // ```
    /** Confirms that a finished sweep reports its done count equal to its total. */
    @Test
    fun finishedSweepReadsDoneOfTotal() {
        assertEquals(
            SweepProgressSnapshot(done = 5, total = 5),
            sweepProgressMerge(
                current = SweepProgressSnapshot(done = 4, total = 5),
                reportedDone = 5,
                reportedTotal = 5,
            ),
        )
    }
}
