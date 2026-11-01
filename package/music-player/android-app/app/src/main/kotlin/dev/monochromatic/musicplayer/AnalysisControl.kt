// Process-wide controls of the true-peak analysis that more than one Android component must reach (D107, D26, D84).

// What:     `package dev.monochromatic.musicplayer` places the controls beside the sweep service that reads them.
// Why:      The notification button (a broadcast receiver), the sweep service and the activity share one process,
//           so one holder carries the pause gate and the known total between them without a binder.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import dev.monochromatic.musicplayer.core.SweepPauseGate
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

// What:     `object AnalysisControl` holds the sweep's pause gate and the total of the running analysis.
// Why:      The pause gate must be the same instance for the receiver that flips it and the sweep that waits on
//           it. The total is a plain state flow, not Compose state, so the activity can wait for the first
//           analysis without relying on a composition being alive.
//
// In TS you'd write (pseudocode):
// ```ts
// export const AnalysisControl = { pauseGate: new SweepPauseGate(), activeTotal: observable(0) };
// ```
/** Pause gate and running total of the true-peak analysis, shared by the service, receiver and activity. */
object AnalysisControl {
    /** Gate that the sweep workers pass before each track claim and that the notification button flips. */
    val pauseGate: SweepPauseGate = SweepPauseGate()

    /** Backing value of [activeTotal]. */
    private val activeTotalFlag: MutableStateFlow<Int> = MutableStateFlow(0)

    /** Tracks the running analysis will visit, or zero when none runs; written only through [setActiveTotal]. */
    val activeTotal: StateFlow<Int>
        get() = activeTotalFlag

    /**
     * Publishes the total of the analysis that just began, or zero when it ended.
     *
     * @param total tracks the analysis will visit, or zero for none
     */
    fun setActiveTotal(total: Int) {
        activeTotalFlag.value = total
    }
}

// What:     `class AnalysisActionReceiver : BroadcastReceiver()` turns a notification button press into a gate flip.
// Why:      A broadcast receiver starts no service and no sweep, so a press cannot launch a second sweep or hit the
//           background service start rules; it only flips the flag the running sweep already waits on.
//
// In TS you'd write (pseudocode):
// ```ts
// onBroadcast(action) { if (action === PAUSE) gate.pause(); else if (action === RESUME) gate.resume(); }
// ```
/** Receives the Pause and Resume presses of the analysis notification and flips [AnalysisControl.pauseGate]. */
class AnalysisActionReceiver : BroadcastReceiver() {
    /** Flips the pause gate for the pressed action and ignores any other action. */
    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            ACTION_ANALYSIS_PAUSE -> {
                Log.i(CONTROL_TAG, "analysis pause pressed")
                AnalysisControl.pauseGate.pause()
            }
            ACTION_ANALYSIS_RESUME -> {
                Log.i(CONTROL_TAG, "analysis resume pressed")
                AnalysisControl.pauseGate.resume()
            }
            else -> Log.w(CONTROL_TAG, "analysis receiver ignored action ${intent.action}")
        }
    }
}

/** Logcat tag of the analysis controls, shared with the sweep. */
private const val CONTROL_TAG: String = "PeakSweep"

/** Broadcast action of the notification's Pause button. */
internal const val ACTION_ANALYSIS_PAUSE: String = "dev.monochromatic.musicplayer.action.ANALYSIS_PAUSE"

/** Broadcast action of the notification's Resume button. */
internal const val ACTION_ANALYSIS_RESUME: String = "dev.monochromatic.musicplayer.action.ANALYSIS_RESUME"
