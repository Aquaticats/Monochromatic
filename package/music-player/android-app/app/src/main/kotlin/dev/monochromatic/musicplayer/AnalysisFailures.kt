// A file that cannot be decoded is reported once, in plain words, as a system notification (D113 and D123).

// What:     `package dev.monochromatic.musicplayer` places the report beside the analysis service that makes it.
// Why:      The service and the notification poster live in one process, so the report is one function call.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

// What:     `object AnalysisFailures` is the entry point for files whose analysis just failed for the first time.
// Why:      D123: the report is a system notification and never an in-app message, so the object keeps no list for
//           a screen to read. "First time" is decided before the call, by the sweep ledger (`core/SweepLedger.kt`),
//           which is why each file is reported once. An empty report posts nothing, and a refused post only logs.
//
// In TS you'd write (pseudocode):
// ```ts
// export const AnalysisFailures = { report(context, names) { postNotification(context, names); } };
// ```
/** Reports files that failed to decode in the analysis, once per file, as one system notification. */
object AnalysisFailures {
    /**
     * Posts one notification for the files whose analysis failed for the first time, or logs one warning when the
     * notification cannot be posted. Nothing is shown in the app in either case.
     *
     * @param context any context of this app
     * @param fileNames bare file names, without their folders; an empty list changes nothing
     */
    fun report(context: Context, fileNames: List<String>) {
        if (fileNames.isEmpty()) {
            return
        }
        postAnalysisFailureNotification(context = context, fileNames = fileNames)
    }
}

// What:     `object MeasureInvalidation` is the in-process list of tracks whose measures the analysis just recomputed.
// Why:      A replaced or edited file keeps its URI, so a row that already shows the old duration and peak must read
//           them again. The player host drops the old measures of these tracks,
//           and the page refresh reads the new ones.
//
// In TS you'd write (pseudocode):
// ```ts
// export const MeasureInvalidation = { pending: observable<string[]>([]), report(uris) {}, take() {} };
// ```
/** URIs of tracks the analysis measured again and whose shown measures are therefore stale. */
object MeasureInvalidation {
    /** The stale URIs, observed by the player host. */
    var pending: Set<String> by mutableStateOf(emptySet())
        private set

    /**
     * Marks tracks whose measures were just recomputed.
     *
     * @param uris track URIs of the pass's fresh tracks
     */
    fun report(uris: Collection<String>) {
        if (uris.isEmpty()) {
            return
        }
        pending = pending + uris
    }

    /** Hands the pending URIs to the caller and empties the set. */
    fun take(): Set<String> {
        /** The URIs waiting to be dropped. */
        val uris: Set<String> = pending
        pending = emptySet()
        return uris
    }
}
