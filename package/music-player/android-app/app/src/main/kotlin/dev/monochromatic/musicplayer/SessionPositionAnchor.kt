// What:     `package dev.monochromatic.musicplayer` puts this class beside `BrainPlayer`, which uses it.
// Why:      The rule is plain Kotlin with no Android types (the clock is passed in), so unit tests reach it
//           directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import kotlin.math.abs

// What:     `class SessionPositionAnchor(now)` remembers the playback position that `BrainPlayer.getState()` last
//           handed to media3 and answers whether the engine still agrees with it.
// Why:      media3 turns the position of a pulled state into a clock that runs on its own while playback is
//           advancing, and it re-reads the player only when something asks it to. The engine applies a seek on its
//           worker thread, after `getState()` has already read the old position, and a track change can be pulled
//           the same way. The session then extrapolates from a wrong anchor until the next unrelated pull, so the
//           notification and lock screen scrubber disagree with the app. Checking the anchor against the engine a
//           moment after each pull, and while playing, repairs that with one more pull.
// Gotcha:   Playback speed is always 1 in this app, so the expected position is the anchor plus elapsed time.
//
// In TS you'd write (pseudocode):
// ```ts
// class SessionPositionAnchor {
//   constructor(private now: () => number) {}
//   record(positionMs: number, advancing: boolean): void { ... }
//   check(actualMs: number): AnchorCheck { ... }
// }
// ```
/** Compares the position media3 extrapolates from the last pulled state with the engine's real position. */
class SessionPositionAnchor(
    /** Monotonic clock in milliseconds, the same clock media3 extrapolates with. */
    private val now: () -> Long,
) {
    /** What a check found, and what to do next. */
    sealed interface Check {
        /** The engine moved away from the anchor: pull the state again. */
        data object Resync : Check

        /** The anchor still holds: check again after [delayMs]. */
        data class CheckAgainIn(val delayMs: Long) : Check

        /** The anchor holds and the position is not advancing: nothing more to check until the next pull. */
        data object Settled : Check
    }

    /** Position in milliseconds that the last pulled state reported. */
    private var anchorMs: Long = 0L

    /** Clock reading when [anchorMs] was recorded. */
    private var anchoredAtMs: Long = 0L

    /** Whether media3 extrapolates the position, which it does while playing a loaded track. */
    private var advancing: Boolean = false

    /** Checks still owed after a pull when the position is not advancing. */
    private var settleChecksLeft: Int = 0

    /** Records the position and advancing flag that a pulled state reported, and restarts the settle checks. */
    fun record(positionMs: Long, advancing: Boolean) {
        anchorMs = positionMs
        anchoredAtMs = now()
        this.advancing = advancing
        settleChecksLeft = SETTLE_CHECKS
    }

    /** Position in milliseconds that media3 reports now for the recorded state. */
    fun expectedMs(): Long = if (advancing) anchorMs + (now() - anchoredAtMs) else anchorMs

    /** Compares [actualMs], the engine's position now, with [expectedMs] and says what to do next. */
    fun check(actualMs: Long): Check {
        if (abs(actualMs - expectedMs()) > DRIFT_THRESHOLD_MS) {
            return Check.Resync
        }
        settleChecksLeft = maxOf(0, settleChecksLeft - 1)
        return when {
            advancing -> Check.CheckAgainIn(ADVANCING_INTERVAL_MS)
            settleChecksLeft > 0 -> Check.CheckAgainIn(SETTLE_INTERVAL_MS)
            else -> Check.Settled
        }
    }

    /** Timings and limits of the checks. */
    companion object {
        /** Delay before the first check after a pull, long enough for the worker thread to apply a seek. */
        const val FIRST_CHECK_DELAY_MS: Long = 250L

        /** Disagreement in milliseconds that triggers a new pull; below it a seek is not worth repairing. */
        const val DRIFT_THRESHOLD_MS: Long = 500L

        /** Interval between checks while the position advances, so a stall or late seek is caught within a second. */
        const val ADVANCING_INTERVAL_MS: Long = 1000L

        /** Interval between the checks that follow a pull while paused, for a seek the worker applies late. */
        const val SETTLE_INTERVAL_MS: Long = 1000L

        /** Checks made after a pull while paused before the anchor is trusted until the next pull. */
        const val SETTLE_CHECKS: Int = 3
    }
}
