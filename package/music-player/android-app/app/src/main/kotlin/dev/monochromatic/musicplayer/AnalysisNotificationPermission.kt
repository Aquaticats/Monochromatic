// Asks for the notification permission once, when the first true-peak analysis starts (decision D107).

// What:     `package dev.monochromatic.musicplayer` places the asker beside the activity that hosts the prompt.
// Why:      The prompt needs an activity, and the Activity Result API registers its launcher on that activity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import dev.monochromatic.musicplayer.core.shouldAskForNotificationPermission
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch

// What:     `class AnalysisNotificationPermission(activity)` registers the permission launcher and waits for the
//           first analysis.
// Why:      D107 asks once, when the first analysis starts, so the prompt never competes with the music access
//           prompt of first run: the analysis only has a total after that access was granted. The answer is
//           not needed afterwards, because a denial simply leaves the analysis without an indicator. The ask is
//           recorded before the prompt opens, so a denial or a process death never asks again.
//
// In TS you'd write (pseudocode):
// ```ts
// class AnalysisNotificationPermission {
//   constructor(activity) { this.launcher = activity.registerForResult(RequestPermission, log); }
//   start() {
//     whenStarted(async () => { const total = await firstPositive(AnalysisControl.activeTotal); ask(total); });
//   }
// }
// ```
/**
 * Shows the Android notification permission prompt at most once, when the first analysis starts.
 *
 * Construct it as a field of the activity, because the launcher must be registered before the activity starts,
 * and call [start] from `onCreate`.
 *
 * @param activity activity that hosts the prompt and whose lifecycle bounds the wait
 */
class AnalysisNotificationPermission(private val activity: ComponentActivity) {
    /** Launcher of the permission prompt; the answer is only logged, since a denial changes nothing else. */
    private val launcher = activity.registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        Log.i(LOG_TAG, "notification permission granted=$granted")
    }

    /** Starts waiting for the first analysis while the activity is visible. */
    fun start() {
        activity.lifecycleScope.launch {
            activity.repeatOnLifecycle(Lifecycle.State.STARTED) {
                /** Total of the running analysis, taken when the first positive value arrives. */
                val total: Int = AnalysisControl.activeTotal.first { value -> value > 0 }
                askIfNeeded(total)
            }
        }
    }

    // What:     `private fun askIfNeeded(total)` applies the policy and opens the prompt when it says so.
    // Why:      The flag is saved before the launch, the same way the music access prompt records its ask.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function askIfNeeded(total) { if (shouldAsk(...)) { saveAsked(); launcher.launch(POST_NOTIFICATIONS); } }
    // ```
    /** Opens the permission prompt when the policy allows it for an analysis of [total] tracks. */
    private fun askIfNeeded(total: Int) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            // Below Android 13 notifications need no permission, so there is nothing to ask.
            return
        }
        /** Whether the app already holds the permission. */
        val granted: Boolean =
            ContextCompat.checkSelfPermission(activity, Manifest.permission.POST_NOTIFICATIONS) ==
                PackageManager.PERMISSION_GRANTED
        if (!shouldAskForNotificationPermission(
                sdkInt = Build.VERSION.SDK_INT,
                granted = granted,
                askedBefore = SessionStore.loadNotificationAccessRequested(activity),
                analysisTotal = total,
            )
        ) {
            return
        }
        Log.i(LOG_TAG, "asking for the notification permission at the first analysis total=$total")
        SessionStore.saveNotificationAccessRequested(activity, requested = true)
        launcher.launch(Manifest.permission.POST_NOTIFICATIONS)
    }
}
