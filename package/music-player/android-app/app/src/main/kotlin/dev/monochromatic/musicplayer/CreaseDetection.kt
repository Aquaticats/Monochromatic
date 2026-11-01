// Crease detection through Jetpack WindowManager (D110, E2). The window's `FoldingFeature` list says whether the screen
// has a vertical fold or hinge and how wide the platform reports it; `core/CreaseReading.kt` decides from those
// numbers.
// Until the first layout report arrives the hinge-angle system feature stands in, so the first frame is not wrong on a
// device that has a hinge. Settings and Search both read the one answer, which `PlayerHost` detects once.

// What:     `package dev.monochromatic.musicplayer` places the crease check beside the other player composables.
// Why:      The wide Settings page and the Search page need to know whether the window has a crease.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports bring in the platform layout tracker, Compose state, Android's context and log, and the pure rule.
// Why:      The tracker is a cold `Flow` of layout reports, which `produceState` turns into Compose state.
//
// In TS you'd write (pseudocode):
// ```ts
// import { WindowInfoTracker, FoldingFeature } from "androidx/window/layout";
// ```
import android.content.Context
import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.window.layout.DisplayFeature
import androidx.window.layout.FoldingFeature
import androidx.window.layout.WindowInfoTracker
import androidx.window.layout.WindowLayoutInfo
import dev.monochromatic.musicplayer.core.CreaseState
import dev.monochromatic.musicplayer.core.FoldReading
import dev.monochromatic.musicplayer.core.resolveCrease

// What:     `private const val CREASE_TAG: String = "PlayerCrease"` is the log tag of the crease detection.
// Why:      One tag lets a device check filter every crease line.
//
// In TS you'd write (pseudocode):
// ```ts
// const CREASE_TAG = "PlayerCrease";
// ```
/** Log tag of the crease detection. */
private const val CREASE_TAG: String = "PlayerCrease"

// What:     `private const val HINGE_ANGLE_FEATURE: String` names the system feature of a device with a hinge.
// Why:      Android declares `android.hardware.sensor.hinge_angle` (`PackageManager.FEATURE_SENSOR_HINGE_ANGLE`) on a
//           device whose body bends at a hinge. It is only the fallback before the first layout report.
//
// In TS you'd write (pseudocode):
// ```ts
// const HINGE_ANGLE_FEATURE = "android.hardware.sensor.hinge_angle";
// ```
/** Name of the system feature that a device with a hinge declares. */
private const val HINGE_ANGLE_FEATURE: String = "android.hardware.sensor.hinge_angle"

// What:     `internal fun deviceHasHingeSensor(context)` asks the platform whether the device declares a hinge.
// Why:      D110's source is the layout API; this answer covers only the time before it has reported.
//
// In TS you'd write (pseudocode):
// ```ts
// const deviceHasHingeSensor = (ctx) => ctx.packageManager.hasSystemFeature(HINGE_ANGLE_FEATURE);
// ```
/** Whether the device declares a hinge-angle sensor. */
internal fun deviceHasHingeSensor(context: Context): Boolean =
    context.packageManager.hasSystemFeature(HINGE_ANGLE_FEATURE)

// What:     `private fun WindowLayoutInfo.foldReadings()` reduces the layout's folding features to plain values.
// Why:      The pure rule cannot hold `Rect` or the library's enums, so the orientation, the flags and the horizontal
//           bounds are copied out here.
//
// In TS you'd write (pseudocode):
// ```ts
// const foldReadings = (info) => info.displayFeatures.filter(isFolding).map(toReading);
// ```
/** Lists every folding feature of this layout as plain values; other display features are not folds. */
private fun WindowLayoutInfo.foldReadings(): List<FoldReading> =
    displayFeatures.filterIsInstance<FoldingFeature>().map { feature ->
        FoldReading(
            vertical = feature.orientation == FoldingFeature.Orientation.VERTICAL,
            separating = feature.isSeparating,
            occluding = feature.occlusionType == FoldingFeature.OcclusionType.FULL,
            leftPx = feature.bounds.left,
            rightPx = feature.bounds.right,
        )
    }

// What:     `private fun DisplayFeature.describe()` formats a display feature for the log.
// Why:      The log must show everything the platform said about a fold, since the decision uses only some of it.
//
// In TS you'd write (pseudocode):
// ```ts
// const describe = (f) => f instanceof FoldingFeature ? `${f.state} ${f.orientation} ...` : String(f);
// ```
/** Describes one display feature for the log, with everything the platform reported about a fold. */
private fun DisplayFeature.describe(): String {
    if (this !is FoldingFeature) {
        return "${this::class.java.simpleName} bounds=$bounds"
    }
    return "FoldingFeature state=$state orientation=$orientation separating=$isSeparating " +
        "occlusion=$occlusionType bounds=$bounds"
}

// What:     `@Composable internal fun rememberCrease(): CreaseState` detects the crease of the window it is called in.
// Why:      `WindowInfoTracker.getOrCreate(context).windowLayoutInfo(context)` reports the window's folding features
//           and again whenever they change (folding, unfolding, resizing), so the answer follows the device; it starts
//           from the hinge-sensor fallback and logs every report.
//
// In TS you'd write (pseudocode):
// ```ts
// function useCrease(): CreaseState { return useLayoutInfo(window) ... }
// ```
/**
 * Detects the vertical crease of the current window through Jetpack WindowManager.
 *
 * The context must be a visual one, which the activity's `LocalContext` is.
 */
@Composable
internal fun rememberCrease(): CreaseState {
    /** Activity context whose window the tracker reports on. */
    val context: Context = LocalContext.current
    /** Whether the device declares a hinge, the answer used before the first layout report. */
    val hingeSensor: Boolean = remember(context) { deviceHasHingeSensor(context) }
    /** The crease state, updated on every layout report. */
    val state: State<CreaseState> = produceState(
        initialValue = resolveCrease(folds = null, hingeSensorPresent = hingeSensor),
        context,
        hingeSensor,
    ) {
        Log.i(CREASE_TAG, "waiting for the first window layout report; hingeSensor=$hingeSensor")
        WindowInfoTracker.getOrCreate(context).windowLayoutInfo(context).collect { info ->
            /** Crease state computed from the latest layout report. */
            val next: CreaseState = resolveCrease(folds = info.foldReadings(), hingeSensorPresent = hingeSensor)
            Log.i(
                CREASE_TAG,
                "crease present=${next.present} widthPx=${next.reportedWidthPx} source=${next.source} " +
                    "hingeSensor=$hingeSensor features=[${info.displayFeatures.joinToString("; ") { it.describe() }}]",
            )
            value = next
        }
    }
    return state.value
}
