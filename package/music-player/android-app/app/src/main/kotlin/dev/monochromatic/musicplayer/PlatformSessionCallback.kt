// What:     `package dev.monochromatic.musicplayer` puts this class beside `PlaybackService`, which installs it.
// Why:      Same package as the rule it applies, `platformCommandCodes`.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.util.Log` pulls in Android's logger.
// Why:      The callback logs which controller got which timeline decision.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     `import androidx.annotation.OptIn` pulls in the annotation that acknowledges use of media3's
//           `@UnstableApi` members. Sibling: `kotlin.OptIn`, which is for Kotlin's own experimental markers.
// Why:      `onConnectAsync` and `isMediaNotificationController` are marked unstable, and lint rejects their use
//           without this acknowledgement.
//
// In TS you'd write (pseudocode):
// ```ts
// import { OptIn } from "androidx/annotation"; // a decorator
// ```
import androidx.annotation.OptIn

// What:     `import androidx.media3.common.Player` pulls in `Player`, which owns `Player.Commands`.
// Why:      The callback builds the command set it hands back.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Player } from "media3/common";
// ```
import androidx.media3.common.Player

// What:     `import androidx.media3.common.util.UnstableApi` pulls in the marker annotation for unstable media3 API.
// Why:      It is the argument of `@OptIn`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { UnstableApi } from "media3/common/util";
// ```
import androidx.media3.common.util.UnstableApi

// What:     `import androidx.media3.session.MediaSession` pulls in `MediaSession`, which owns the `Callback`
//           interface, `ControllerInfo` and `ConnectionResult` used below.
// Why:      This class implements `MediaSession.Callback`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { MediaSession } from "media3/session";
// ```
import androidx.media3.session.MediaSession

// What:     `import com.google.common.util.concurrent.Futures` pulls in Guava's `Futures` helpers.
//           `ListenableFuture` is Guava's promise type; `Futures.immediateFuture(x)` is an already-resolved one.
// Why:      `onConnectAsync` must return a `ListenableFuture`; the answer is known at once.
//
// In TS you'd write (pseudocode):
// ```ts
// // Promise.resolve(x)
// ```
import com.google.common.util.concurrent.Futures

// What:     `import com.google.common.util.concurrent.ListenableFuture` pulls in Guava's promise type.
// Why:      It is the declared return type of `onConnectAsync`.
//
// In TS you'd write (pseudocode):
// ```ts
// // Promise<ConnectionResult>
// ```
import com.google.common.util.concurrent.ListenableFuture

// What:     `private const val TAG` is a compile-time constant string, the log tag for this file.
// Why:      `Log.i(TAG, ...)` lines can be found with `adb logcat -s`.
//
// In TS you'd write (pseudocode):
// ```ts
// const TAG = "PlatformSessionCallback";
// ```
/** Log tag of this file. */
private const val TAG: String = "PlatformSessionCallback"

// What:     `class PlatformSessionCallback : MediaSession.Callback { ... }` declares a class that implements the
//           `MediaSession.Callback` interface (the `:` after the name means "extends or implements"). media3 calls
//           it when a controller connects. `@OptIn(UnstableApi::class)` acknowledges the unstable media3 members it
//           uses.
// Why:      The system notification controller, which media3 connects for the platform session, gets every command
//           except the timeline read. That makes media3 skip converting the whole playing queue to the platform
//           queue on each timeline change, the measured cost of a 10 000-track queue. Android Auto, Wear and other
//           controllers that read the platform queue see no queue.
// Gotcha:   Only `onConnectAsync` is overridden; the other callbacks keep media3's defaults, which accept every
//           command from every controller exactly as before.
//
// In TS you'd write (pseudocode):
// ```ts
// class PlatformSessionCallback implements MediaSessionCallback {
//   onConnectAsync(session, controller): Promise<ConnectionResult> { ... }
// }
// ```
/** Connection policy for the media session: the notification controller does not read the timeline. */
@OptIn(UnstableApi::class)
class PlatformSessionCallback : MediaSession.Callback {
    /** Answers a connecting controller with media3's default commands, minus the timeline read for the notification. */
    override fun onConnectAsync(
        session: MediaSession,
        controller: MediaSession.ControllerInfo,
    ): ListenableFuture<MediaSession.ConnectionResult> {
        // What:     `session.isMediaNotificationController(controller)` is true for the controller media3 connects
        //           for the system notification and the platform session.
        // Why:      Only that controller decides what the platform session publishes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const isNotification = session.isMediaNotificationController(controller);
        // ```
        /** Whether the connecting controller is the one that feeds the platform session. */
        val isNotification: Boolean = session.isMediaNotificationController(controller)
        // What:     `AcceptedResultBuilder(session, controller).build()` makes media3's default answer for this
        //           controller (every command for a trusted one, read-only for an untrusted one).
        // Why:      Keeps today's behaviour for every command except the one the rule removes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const defaults = new AcceptedResultBuilder(session, controller).build();
        // ```
        /** The answer media3 gives when no rule applies. */
        val defaults: MediaSession.ConnectionResult =
            MediaSession.ConnectionResult.AcceptedResultBuilder(session, controller).build()
        if (!isNotification) {
            Log.i(TAG, "onConnectAsync notification=false: default commands")
            return Futures.immediateFuture(defaults)
        }
        // What:     `Player.Commands.Builder()` collects command codes; `add(code)` adds one. The loop copies every
        //           default command except the ones the rule drops. `commands.get(index)` reads the code at a
        //           position and `size()` is the count.
        // Why:      Hand media3 the reduced command set for the notification controller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const builder = new Commands.Builder();
        // for (let i = 0; i < offered.size(); i++) {
        //   if (keepsPlatformCommand(true, offered.get(i))) builder.add(offered.get(i));
        // }
        // ```
        /** The default commands of the notification controller. */
        val offered: Player.Commands = defaults.availablePlayerCommands
        /** Collects the commands that stay. */
        val kept: Player.Commands.Builder = Player.Commands.Builder()
        for (index in 0 until offered.size()) {
            if (keepsPlatformCommand(isMediaNotificationController = true, code = offered.get(index))) {
                kept.add(offered.get(index))
            }
        }
        Log.i(TAG, "onConnectAsync notification=true: offered=${offered.size()}")
        /** The answer for the notification controller: defaults with the reduced player commands. */
        val reduced: MediaSession.ConnectionResult =
            MediaSession.ConnectionResult.AcceptedResultBuilder(session, controller)
                .setAvailablePlayerCommands(kept.build())
                .build()
        return Futures.immediateFuture(reduced)
    }
}
