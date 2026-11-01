// Keyboard glue of the two one-stop lists, the folder rail and the track list (D106). Each list is one focus stop with
// its own focus ring that arrow keys move, instead of every entry being a Tab stop. This file holds what both lists
// share: the host state they read, the request to move focus into them, and the key handler that asks the D106 table
// what a key means inside a focused list.

// What:     `package dev.monochromatic.musicplayer` places the glue beside the two lists and the host root.
// Why:      The lists and the host share this package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports bring in Compose focus, key and effect types and the pure key rules.
// Why:      The glue wires a focusable modifier to the focus state, the host locals and the key table.
//
// In TS you'd write (pseudocode):
// ```ts
// import { useFocus, useKeys } from "compose";
// ```
import android.util.Log
import androidx.compose.foundation.focusable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import dev.monochromatic.musicplayer.core.HostKeyCommand
import dev.monochromatic.musicplayer.core.HostKeyContext
import dev.monochromatic.musicplayer.core.HostKeyPress
import dev.monochromatic.musicplayer.core.HostKeyZone
import dev.monochromatic.musicplayer.core.ListKeyCommand
import dev.monochromatic.musicplayer.core.TrackMenuAction
import dev.monochromatic.musicplayer.core.hostKeyCommandFor

// What:     `private const val LIST_KEY_TAG: String` is the log tag of the list keyboard glue.
// Why:      One tag lets a device check filter every list key line.
//
// In TS you'd write (pseudocode):
// ```ts
// const LIST_KEY_TAG = "ListKeyboard";
// ```
/** Log tag of the list keyboard glue. */
private const val LIST_KEY_TAG: String = "ListKeyboard"

// What:     `val HostKeyLocalContext: ProvidableCompositionLocal<HostKeyContext?>` carries the host's key state.
// Why:      A focused list asks the key table what a key means, and the table reads the same state as the host root.
//
// In TS you'd write (pseudocode):
// ```ts
// const HostKeyLocalContext = createContext<HostKeyContext | null>(null);
// ```
/** The host state that the key rules read, or null when no host provides one. */
val HostKeyLocalContext: ProvidableCompositionLocal<HostKeyContext?> = compositionLocalOf { null }

// What:     `val HostKeyLocalZone: ProvidableCompositionLocal<(HostKeyZone, Boolean) -> Unit>` reports list focus.
// Why:      The owner of a key depends on which list holds focus, and only the list knows that.
//
// In TS you'd write (pseudocode):
// ```ts
// const HostKeyLocalZone = createContext<(zone: HostKeyZone, focused: boolean) => void>(() => {});
// ```
/** Callback that a list calls when it gains or loses keyboard focus. */
val HostKeyLocalZone: ProvidableCompositionLocal<(HostKeyZone, Boolean) -> Unit> =
    compositionLocalOf { { _, _ -> } }

// What:     `val HostKeyLocalFocusRequest: ProvidableCompositionLocal<HostKeyZone?>` carries a pending focus request.
// Why:      The focus chords ask a list to take keyboard focus. The request stays set until that list handles it, so
//           a list that is not composed yet (the folder picker on the cover) can still act on it when it appears.
//
// In TS you'd write (pseudocode):
// ```ts
// const HostKeyLocalFocusRequest = createContext<HostKeyZone | null>(null);
// ```
/** The list that was asked to take keyboard focus and has not done so yet, or null. */
val HostKeyLocalFocusRequest: ProvidableCompositionLocal<HostKeyZone?> = compositionLocalOf { null }

// What:     `val HostKeyLocalFocusHandled: ProvidableCompositionLocal<() -> Unit>` clears the pending request.
// Why:      The list that took focus tells the host, so the request is not applied twice.
//
// In TS you'd write (pseudocode):
// ```ts
// const HostKeyLocalFocusHandled = createContext<() => void>(() => {});
// ```
/** Callback that a list calls after it took keyboard focus for a pending request. */
val HostKeyLocalFocusHandled: ProvidableCompositionLocal<() -> Unit> = compositionLocalOf { {} }

// What:     `val LocalTrackRowKeyActions` carries the host's runner for a menu action chosen by key.
// Why:      Delete, Alt+Enter and the other row chords run a context menu action on the row under the focus ring
//           without opening the menu, through the same host path as the menu rows.
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalTrackRowKeyActions = createContext<((a: TrackMenuAction, index: number) => void) | null>(null);
// ```
/** Runs a track menu action for a library index, or null when no host provides one. */
internal val LocalTrackRowKeyActions: ProvidableCompositionLocal<((TrackMenuAction, Int) -> Unit)?> =
    compositionLocalOf { null }

// What:     `class ListZoneState` holds a list's focus requester and whether the list is focused.
// Why:      The list draws its focus ring only while it holds focus, and the host asks it to take focus through the
//           requester.
//
// In TS you'd write (pseudocode):
// ```ts
// class ListZoneState { requester = new FocusRequester(); focused = false; }
// ```
/** Focus requester and focus flag of one one-stop list. */
internal class ListZoneState {
    /** Requester attached to the list, which a focus chord or a reveal asks to take focus. */
    val requester: FocusRequester = FocusRequester()

    /** Whether the list holds keyboard focus now. */
    var focused: Boolean by mutableStateOf(false)
}

// What:     `@Composable internal fun rememberListZone(zone)` creates a list's state and serves focus requests.
// Why:      The list takes focus when the host asks for its zone, once its node exists, and tells the host the
//           request is done; it also reports that it no longer has focus when it leaves composition.
//
// In TS you'd write (pseudocode):
// ```ts
// function useListZone(zone: HostKeyZone): ListZoneState { ... }
// ```
/** Remembers the focus state of one list and applies the host's pending focus request for [zone]. */
@Composable
internal fun rememberListZone(zone: HostKeyZone): ListZoneState {
    /** State of this list. */
    val state: ListZoneState = remember { ListZoneState() }
    /** Pending request to move focus into a list. */
    val request: HostKeyZone? = HostKeyLocalFocusRequest.current
    /** Clears the pending request once it was applied. */
    val handled: () -> Unit = HostKeyLocalFocusHandled.current
    /** Reports focus loss to the host when the list leaves composition. */
    val report: (HostKeyZone, Boolean) -> Unit = HostKeyLocalZone.current
    LaunchedEffect(request) {
        if (request == zone) {
            androidx.compose.runtime.withFrameNanos { }
            Log.i(LIST_KEY_TAG, "focus request applied zone=$zone")
            state.requester.requestFocus()
            handled()
        }
    }
    DisposableEffect(zone) {
        onDispose { report(zone, false) }
    }
    return state
}

// What:     `@Composable internal fun Modifier.listZone(zone, state, onCommand)` makes a list one focus stop.
// Why:      The list holds focus as a whole and handles its own keys: the key table decides what a key means for the
//           focused list, and the list runs the commands that concern it. Keys the list does not handle go on to the
//           parents, so the playback chords still reach the host root first.
//
// In TS you'd write (pseudocode):
// ```ts
// const listZone = (zone, state, onCommand) => focusable(state.requester, onFocus, onKey(onCommand));
// ```
/**
 * Makes the node one keyboard focus stop that reports its focus and runs the list commands of the key table.
 *
 * @param zone which list this is, reported to the host while it holds focus.
 * @param state focus requester and flag of the list.
 * @param onCommand runs a list command and returns true when it consumed the key.
 */
@Composable
internal fun Modifier.listZone(
    zone: HostKeyZone,
    state: ListZoneState,
    onCommand: (ListKeyCommand) -> Boolean,
): Modifier {
    /** Host state that the key table reads. */
    val context: HostKeyContext? = HostKeyLocalContext.current
    /** Reports focus changes to the host. */
    val report: (HostKeyZone, Boolean) -> Unit = HostKeyLocalZone.current
    return this
        .focusRequester(state.requester)
        .onFocusChanged { focus ->
            state.focused = focus.isFocused
            report(zone, focus.isFocused)
        }
        .focusable()
        .onKeyEvent { event -> listKeyHandled(event = event, context = context, onCommand = onCommand) }
}

// What:     `private fun listKeyHandled(event, context, onCommand)` asks the table about one key-down in a list.
// Why:      Only key-downs matter, and only the list commands of the table are the list's business.
//
// In TS you'd write (pseudocode):
// ```ts
// function listKeyHandled(e: KeyboardEvent, ctx, onCommand): boolean { ... }
// ```
/** Runs the list command that a key-down means, and reports whether the key was consumed. */
private fun listKeyHandled(
    event: KeyEvent,
    context: HostKeyContext?,
    onCommand: (ListKeyCommand) -> Boolean,
): Boolean {
    if (event.type != KeyEventType.KeyDown || context == null) {
        return false
    }
    /** Command the table gives this key in the current state. */
    val command: HostKeyCommand = hostKeyCommandFor(HostKeyPress(key = hostKeyInputOf(event), context = context))
    if (command !is ListKeyCommand) {
        return false
    }
    Log.i(LIST_KEY_TAG, "list key command=$command")
    return onCommand(command)
}
