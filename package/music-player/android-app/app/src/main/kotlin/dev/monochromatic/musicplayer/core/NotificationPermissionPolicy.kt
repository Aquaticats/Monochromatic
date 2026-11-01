// When the app asks for the notification permission that the analysis notification needs (decision D107).

// What:     `package ...core` places the policy beside the other platform-independent decisions.
// Why:      The rule is a pure function of four facts, so JVM tests cover every combination.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `const val NOTIFICATION_PERMISSION_MIN_SDK: Int = 33` is the first Android version with the prompt.
// Why:      Android 13 (API 33) introduced the runtime `POST_NOTIFICATIONS` permission; earlier versions allow
//           notifications without asking, so the app never asks there.
//
// In TS you'd write (pseudocode):
// ```ts
// export const NOTIFICATION_PERMISSION_MIN_SDK = 33;
// ```
/** First Android API level that makes the notification permission a runtime prompt. */
const val NOTIFICATION_PERMISSION_MIN_SDK: Int = 33

// What:     `fun shouldAskForNotificationPermission(...)` says whether to show the permission prompt now.
// Why:      D107 asks once, when the first analysis starts: the analysis must have a known total (which also
//           means audio access was already granted, so the two prompts never overlap), the platform must need
//           the prompt, it must not be granted already, and an earlier ask, answered either way, ends asking.
//
// In TS you'd write (pseudocode):
// ```ts
// function shouldAskForNotificationPermission(a): boolean {
//   return a.sdkInt >= 33 && !a.granted && !a.askedBefore && a.analysisTotal > 0;
// }
// ```
/**
 * Decides whether to show the notification permission prompt.
 *
 * @param sdkInt Android API level of the device
 * @param granted whether the notification permission is already held
 * @param askedBefore whether the app showed the prompt on an earlier run or earlier in this one
 * @param analysisTotal tracks the running analysis will visit, or zero while no analysis has started
 * @return true only for the first analysis on a device that needs the prompt and has not been asked
 */
fun shouldAskForNotificationPermission(
    sdkInt: Int,
    granted: Boolean,
    askedBefore: Boolean,
    analysisTotal: Int,
): Boolean = sdkInt >= NOTIFICATION_PERMISSION_MIN_SDK && !granted && !askedBefore && analysisTotal > 0
