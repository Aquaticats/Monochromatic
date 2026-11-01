// What:     `package dev.monochromatic.musicplayer` puts this file beside `PlatformSessionCallback`, which uses it.
// Why:      The rule is plain Kotlin on integers, with no media3 types, so unit tests reach it directly (media3's
//           `Player.Commands` needs an Android runtime that the host JVM tests do not have).
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import androidx.media3.common.Player` pulls in `Player`, whose `COMMAND_*` constants are plain integer
//           codes. Kotlin copies a Java `static final int` into the calling code at compile time.
// Why:      The rule names the one command it removes. Because the constant is copied in, a host JVM test never loads
//           the `Player` class.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Player } from "media3/common";
// ```
import androidx.media3.common.Player

// What:     `fun keepsPlatformCommand(...)` says whether one player command code stays in the set a connecting
//           controller gets. `isMediaNotificationController` says whether the controller is the one media3 itself
//           creates for the system notification and the platform session. `code` is one command code from the set
//           the controller would get by default. The result is a `Boolean`.
// Why:      media3 publishes the whole playing queue to the platform session whenever the notification
//           controller may read the timeline (`COMMAND_GET_TIMELINE`), and that copy of a 10 000-track queue runs on
//           every track change. Taking only that command away from the notification controller makes media3 publish
//           an empty queue, while playback state, position, duration, metadata and every transport action keep
//           their own commands. Every other controller (an app that connects by itself, a car, a watch) keeps the full
//           offer.
// Gotcha:   Only `COMMAND_GET_TIMELINE` is removed; `COMMAND_GET_CURRENT_MEDIA_ITEM` stays, so the current item's
//           metadata and duration are still published.
//
// In TS you'd write (pseudocode):
// ```ts
// function keepsPlatformCommand(isMediaNotificationController: boolean, code: number): boolean {
//   if (!isMediaNotificationController) return true;
//   return code !== Player.COMMAND_GET_TIMELINE;
// }
// ```
/** True when command [code] stays in a controller's set; the notification controller loses the timeline read. */
fun keepsPlatformCommand(isMediaNotificationController: Boolean, code: Int): Boolean {
    if (!isMediaNotificationController) {
        return true
    }
    return code != Player.COMMAND_GET_TIMELINE
}
