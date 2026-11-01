// The pause gate of the true-peak sweep (decisions D26 and D84): a temporary pause, never a stop.

// What:     `package ...core` places the gate beside the other platform-independent sweep logic.
// Why:      The gate is plain coroutine code with no Android types, so JVM tests cover it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first

// What:     `class SweepPauseGate` holds one pause flag that sweep workers wait on before they claim a track.
// Why:      D84 reads the Pause control as a temporary pause that keeps the analysis non-optional, so pausing
//           stops new claims and resuming continues the same sweep. The wait is a cancellable suspension, so
//           cancelling or destroying the sweep while it is paused unwinds the workers instead of hanging.
//
// In TS you'd write (pseudocode):
// ```ts
// class SweepPauseGate {
//   readonly paused: Observable<boolean>;
//   pause(): void; resume(): void;
//   awaitResumed(signal: AbortSignal): Promise<void>; // rejects when the signal aborts
// }
// ```
/**
 * A pause flag that sweep workers pass before each track claim. A track already being decoded when the gate
 * closes finishes, because the gate only stops new claims.
 */
class SweepPauseGate {
    /** Backing flag; a state flow so waiters resume at once and observers can mirror the flag. */
    private val pausedFlag: MutableStateFlow<Boolean> = MutableStateFlow(false)

    /** Observable pause flag, which the notification publisher reads to flip its action button. */
    val paused: StateFlow<Boolean>
        get() = pausedFlag

    /** Stops workers from claiming further tracks until [resume] is called. Safe to call repeatedly. */
    fun pause() {
        pausedFlag.value = true
    }

    /** Lets waiting workers claim tracks again. Safe to call repeatedly or while not paused. */
    fun resume() {
        pausedFlag.value = false
    }

    // What:     `suspend fun awaitResumed()` returns at once when not paused and otherwise suspends.
    // Why:      The fast path is one volatile read per claim; the slow path is cancellable, so a cancelled
    //           scope throws `CancellationException` out of the wait and the sweep ends without hanging.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async awaitResumed(signal) { if (this.paused.value) await firstValue(this.paused, (p) => !p, signal); }
    // ```
    /**
     * Returns immediately while not paused, and otherwise suspends until [resume] is called.
     *
     * @throws kotlinx.coroutines.CancellationException when the calling coroutine is cancelled while paused
     */
    suspend fun awaitResumed() {
        if (pausedFlag.value) {
            pausedFlag.first { isPaused -> !isPaused }
        }
    }
}
