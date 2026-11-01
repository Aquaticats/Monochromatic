// The progress model behind the true-peak analysis notification (decision D107). It replaces the in-app
// scan indicator: the same count formatting, but a notification-sized model with a pause flag.

// What:     `package ...core` places the progress model beside the other platform-independent models.
// Why:      The notification text and percent are pure functions, so JVM tests cover them without Android.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `const val ANALYSIS_NOTIFICATION_TITLE: String` is the fixed notification title.
// Why:      D107 names the title `Analyzing true peak`; one constant keeps the builder and the tests in step.
//
// In TS you'd write (pseudocode):
// ```ts
// export const ANALYSIS_NOTIFICATION_TITLE = "Analyzing true peak";
// ```
/** Title of the analysis notification, as D107 words it. */
const val ANALYSIS_NOTIFICATION_TITLE: String = "Analyzing true peak"

// What:     `private const val COUNT_GROUP_SIZE: Int = 3` is the digit count between two grouping commas.
// Why:      The count text groups digits in threes, such as `1,218`.
//
// In TS you'd write (pseudocode):
// ```ts
// const COUNT_GROUP_SIZE = 3;
// ```
/** Number of decimal digits between two grouping commas. */
private const val COUNT_GROUP_SIZE: Int = 3

// What:     `private const val PERCENT_SCALE: Long = 100L` converts a fraction to a whole percent.
// Why:      The percent is computed in `Long`, because `Int.MAX_VALUE * 100` overflows an `Int`.
//
// In TS you'd write (pseudocode):
// ```ts
// const PERCENT_SCALE = 100;
// ```
/** Scale from a completed fraction to a whole percent, kept as `Long` so the product cannot overflow. */
private const val PERCENT_SCALE: Long = 100L

// What:     `fun analysisCountText(value: Int): String` groups digits with ASCII commas.
// Why:      The notification text is the count such as `412 of 1,218`; this is the scan indicator's formatting,
//           moved here when the indicator was removed.
//
// In TS you'd write (pseudocode):
// ```ts
// function analysisCountText(value: number): string { return value.toLocaleString("en-US"); }
// ```
/**
 * Formats [value] with a comma between each group of three digits, using ASCII digits only.
 *
 * @param value non-negative count to format
 * @return the grouped decimal text
 */
fun analysisCountText(value: Int): String {
    /** Decimal digits of the count, least significant first after reversing. */
    val reversedDigits: String = value.toString().reversed()
    return reversedDigits.chunked(COUNT_GROUP_SIZE).joinToString(",").reversed()
}

// What:     `data class AnalysisNotificationContent(...)` is everything the notification shows that can change.
// Why:      The update pacer compares two contents to skip a post that would look the same, so the content holds
//           exactly the visible fields: the count text, the percent, the bar kind, and the pause state.
//
// In TS you'd write (pseudocode):
// ```ts
// type AnalysisNotificationContent = {
//   readonly text: string; readonly percent: number; readonly indeterminate: boolean; readonly paused: boolean;
// };
// ```
/** The visible state of the analysis notification; two equal values look the same on screen. */
data class AnalysisNotificationContent(
    /** Count line such as `412 of 1,218`, or an empty string while the total is not known. */
    val text: String,
    /** Whole percent from 0 to 100, or 0 while the bar is indeterminate. */
    val percent: Int,
    /** True while the total is not known, so the bar cannot show a fraction yet. */
    val indeterminate: Boolean,
    /** True while the user has paused the analysis, which flips the action button to Resume. */
    val paused: Boolean,
)

// What:     `fun analysisNotificationContent(...)` turns the sweep counts and the pause flag into the content.
// Why:      The sweep reports raw counts out of order and may overshoot, so the done count is clamped into the
//           total here; a total that is zero or negative means the library is still being read.
//
// In TS you'd write (pseudocode):
// ```ts
// function analysisNotificationContent(done, total, paused): AnalysisNotificationContent { /* clamp, format */ }
// ```
/**
 * Builds the notification content for one sweep report.
 *
 * @param done tracks visited so far, clamped into the total
 * @param total tracks the sweep will visit, or zero or less while unknown
 * @param paused whether the user has paused the analysis
 * @return the text, percent, bar kind and pause state to show
 */
fun analysisNotificationContent(done: Int, total: Int, paused: Boolean): AnalysisNotificationContent {
    if (total <= 0) {
        return AnalysisNotificationContent(text = "", percent = 0, indeterminate = true, paused = paused)
    }
    /** Visited count clamped into the total, so overshoot cannot pass 100 percent. */
    val clampedDone: Int = done.coerceIn(0, total)
    /** Whole percent rounded down, so the bar shows 100 only when every track has been visited. */
    val percent: Int = (clampedDone.toLong() * PERCENT_SCALE / total.toLong()).toInt()
    return AnalysisNotificationContent(
        text = "${analysisCountText(clampedDone)} of ${analysisCountText(total)}",
        percent = percent,
        indeterminate = false,
        paused = paused,
    )
}
