// Hardware-keyboard binding for the player host, for decision D102. The host root takes key events through the
// Compose focus path, turns each key-down into the pure input that core/HostKeyboardModel.kt reads, and dispatches
// the command. Text fields and focused controls keep every key that the command rules leave unhandled.

// What:     `package dev.monochromatic.musicplayer` places the binding beside the host it serves.
// Why:      The host and the Search page read the composition locals declared here without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.util.Log` brings in the platform logger that the host uses for its branch decisions.
// Why:      Every key command that runs is logged, so a shortcut that does nothing can be traced in the log.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log
import android.view.KeyEvent as AndroidKeyEvent
import androidx.compose.foundation.focusable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isAltPressed
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.nativeKeyCode
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import dev.monochromatic.musicplayer.core.HostKeyCommand
import dev.monochromatic.musicplayer.core.HostKeyContext
import dev.monochromatic.musicplayer.core.HostKeyDispatcher
import dev.monochromatic.musicplayer.core.HostKeyInput
import dev.monochromatic.musicplayer.core.HostKeyPress
import dev.monochromatic.musicplayer.core.hostKeyCommandFor

// What:     `private const val HOST_KEY_LOG_TAG: String = "HostKeyboard"` names the log tag of this binding.
// Why:      The log filter can select only the keyboard decisions with this one tag.
//
// In TS you'd write (pseudocode):
// ```ts
// const HOST_KEY_LOG_TAG = "HostKeyboard";
// ```
/** Log tag of the keyboard binding. */
private const val HOST_KEY_LOG_TAG: String = "HostKeyboard"

// What:     `fun hostKeyNameOf(key: Key): String` gives the classifier's key text for each key the map uses.
// Why:      The classifier compares key text, and Compose names keys by code. Only the keys that the Windows and
//           Linux map uses are named; every other key returns an empty text, so no shortcut sees it.
//
// In TS you'd write (pseudocode):
// ```ts
// const keyName = (key: string): string => ({ KeyF: "f", Space: " ", Escape: "Escape", ... })[key] ?? "";
// ```
/**
 * Returns the classifier key text for a key that the map uses, or an empty text for any other key.
 *
 * Letters and digits give one lowercase character whatever Shift does, named keys give their DOM-style name, and the
 * keyboard's Home key is `MoveHome` (Compose `Key.Home` is the system Home button, which no shortcut uses).
 */
internal fun hostKeyNameOf(key: Key): String = when (key) {
    Key.Spacebar -> " "
    Key.Escape -> "Escape"
    Key.DirectionLeft -> "ArrowLeft"
    Key.DirectionRight -> "ArrowRight"
    Key.DirectionUp -> "ArrowUp"
    Key.DirectionDown -> "ArrowDown"
    Key.MoveHome -> "Home"
    Key.MoveEnd -> "End"
    Key.PageUp -> "PageUp"
    Key.PageDown -> "PageDown"
    Key.Enter, Key.NumPadEnter -> "Enter"
    Key.Delete -> "Delete"
    Key.Menu -> "ContextMenu"
    else -> letterOrDigitNameOf(key)
}

// What:     `private fun letterOrDigitNameOf(key: Key): String` names a letter or digit key by its character.
// Why:      The table lists letters and digits by lowercase character, and Android gives each its own key code, so
//           the code range maps to the character and Shift never changes the name.
//
// In TS you'd write (pseudocode):
// ```ts
// const letterOrDigitNameOf = (code: string) =>
//   /^(Key[A-Z]|Digit[0-9])$/.test(code) ? code.slice(-1).toLowerCase() : "";
// ```
/** Returns the lowercase letter or digit that a key types, or an empty text when the key is neither. */
private fun letterOrDigitNameOf(key: Key): String {
    /** Android key code of the key. */
    val code: Int = key.nativeKeyCode
    return when (code) {
        in AndroidKeyEvent.KEYCODE_A..AndroidKeyEvent.KEYCODE_Z ->
            ('a' + (code - AndroidKeyEvent.KEYCODE_A)).toString()
        in AndroidKeyEvent.KEYCODE_0..AndroidKeyEvent.KEYCODE_9 ->
            ('0' + (code - AndroidKeyEvent.KEYCODE_0)).toString()
        else -> ""
    }
}

// What:     `internal fun hostKeyInputOf(event: KeyEvent): HostKeyInput` reads one key-down into the pure input.
// Why:      The core rules take plain values, so the Compose event is read here once and never reaches them.
//           The repeat flag comes from the native event, because Compose does not expose it.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyInputOf = (e: KeyboardEvent): HostKeyInput => ({ keyName: keyName(e.key), ctrl: e.ctrlKey, ... });
// ```
/** Reads one key event into the keyboard input that the core rules use. */
internal fun hostKeyInputOf(event: KeyEvent): HostKeyInput = HostKeyInput(
    keyName = hostKeyNameOf(event.key),
    ctrl = event.isCtrlPressed,
    meta = event.isMetaPressed,
    alt = event.isAltPressed,
    shift = event.isShiftPressed,
    repeat = event.nativeKeyEvent.repeatCount > 0,
)

// What:     `data class HostKeyBinding(...)` joins the root's focus requester, its host state, and its dispatcher.
// Why:      The root modifier needs all three each frame, and one value keeps its signature to a single parameter.
//
// In TS you'd write (pseudocode):
// ```ts
// type HostKeyBinding = { focusRequester: FocusRequester; context: HostKeyContext; dispatcher: HostKeyDispatcher };
// ```
/** The focus requester, host state, and dispatcher that the root keyboard modifier uses. */
data class HostKeyBinding(
    // What:     `val focusRequester: FocusRequester` lets the host request focus for the root.
    // Why:      The root must hold focus on the player so that its keys reach the host.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // focusRequester: FocusRequester;
    // ```
    /** Requester that focuses the host root when no control holds focus. */
    val focusRequester: FocusRequester,

    // What:     `val context: HostKeyContext` is the host state for the current frame.
    // Why:      The owner and the command rules read the state at the moment each key arrives.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // context: HostKeyContext;
    // ```
    /** Host state that the rules read for each key-down. */
    val context: HostKeyContext,

    // What:     `val dispatcher: HostKeyDispatcher` runs the command that a key-down resolves to.
    // Why:      The host's actions are behind the dispatcher, so the root never calls them directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // dispatcher: HostKeyDispatcher;
    // ```
    /** Runs the host command for each consumed key-down. */
    val dispatcher: HostKeyDispatcher,
)

// What:     `@Composable fun Modifier.hostKeyboard(binding: HostKeyBinding): Modifier` makes the root focusable
//           and routes its key-downs through the keyboard rules.
// Why:      Compose delivers key events only along the focused path, so the root must be focusable and reached by
//           the focus path before any child. Preview handling sees each key first and leaves unhandled keys alone.
//
// In TS you'd write (pseudocode):
// ```ts
// root.addEventListener("keydown", handler, { capture: true });
// ```
/** Makes the root focusable and consumes the key-downs and key-ups that the keyboard rules handle. */
@Composable
internal fun Modifier.hostKeyboard(binding: HostKeyBinding): Modifier {
    /** Key codes of key-downs that the host consumed, so their key-ups are consumed too. */
    val consumedKeyUps: MutableSet<Long> = remember { mutableSetOf() }
    return this
        .focusRequester(binding.focusRequester)
        .focusable()
        .onPreviewKeyEvent { event ->
            if (event.type == KeyEventType.KeyUp) {
                return@onPreviewKeyEvent consumedKeyUps.remove(event.key.keyCode)
            }
            if (event.type != KeyEventType.KeyDown) {
                return@onPreviewKeyEvent false
            }
            /** Key text and modifiers of this key-down, read once from the event. */
            val input: HostKeyInput = hostKeyInputOf(event)
            /** Host command that this key-down resolves to, or unhandled when its owner keeps the key. */
            val command: HostKeyCommand = hostKeyCommandFor(HostKeyPress(key = input, context = binding.context))
            if (command == HostKeyCommand.Unhandled) {
                return@onPreviewKeyEvent false
            }
            Log.i(HOST_KEY_LOG_TAG, "host key command=$command keyName=${input.keyName} repeat=${input.repeat}")
            /** Whether the host consumed the key; a list command stays with the focused list. */
            val consumed: Boolean = binding.dispatcher.dispatch(command)
            if (consumed) {
                consumedKeyUps.add(event.key.keyCode)
            }
            consumed
        }
}

// What:     `class SearchFocusPort` hands the Search shortcut from the host's key handler to the open Search page.
// Why:      The shortcut must refocus the query field and put its caret at the end before the next key is read. A
//           counter that the page notices at its next composition and then reacts to in an effect needs two frames, and
//           a character typed in that gap landed at the old caret (D126). The page registers what to do, and the key
//           handler calls it directly, in the same event.
//
// In TS you'd write (pseudocode):
// ```ts
// class SearchFocusPort { onShortcut: (() => void) | null = null; press() { this.onShortcut?.(); } }
// ```
/** Synchronous link between the Search shortcut and the query field of the Search page that is open. */
class SearchFocusPort {
    /** What the open Search page does on the shortcut; null while no Search page is composed. */
    var onShortcut: (() -> Unit)? = null

    // What:     `fun press()` runs the registered shortcut action, if a Search page is open.
    // Why:      With no Search page there is nothing to refocus, so the shortcut does nothing here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // press(): void { this.onShortcut?.(); }
    // ```
    /** Runs the open Search page's shortcut action in the caller's event, or does nothing when none is open. */
    fun press() {
        onShortcut?.invoke()
    }
}

// What:     `val HostKeyLocalSearchFocus: ProvidableCompositionLocal<SearchFocusPort>` carries the shortcut port.
// Why:      The host presses the port on a same-visit Search shortcut, and the Search page registers its field with it.
//           Passing it through a local keeps the Search page's signature unchanged.
//
// In TS you'd write (pseudocode):
// ```ts
// const HostKeyLocalSearchFocus = createContext(new SearchFocusPort());
// ```
/** Port that the host presses on a same-visit Search shortcut, read by the Search page's query field. */
val HostKeyLocalSearchFocus: ProvidableCompositionLocal<SearchFocusPort> = compositionLocalOf { SearchFocusPort() }

// What:     `val HostKeyLocalReveal: ProvidableCompositionLocal<Int>` carries the count of reveal requests.
// Why:      The host bumps the count on the reveal chord, and the track list scrolls to and focuses the playing row.
//           Passing it through a local keeps the track list's signature unchanged.
//
// In TS you'd write (pseudocode):
// ```ts
// const HostKeyLocalReveal = createContext(0);
// ```
/** Count of reveal requests, read by the track list to focus the playing row. */
val HostKeyLocalReveal: ProvidableCompositionLocal<Int> = compositionLocalOf { 0 }

// What:     `val HostKeyLocalSearchField: ProvidableCompositionLocal<(Boolean) -> Unit>` reports field focus.
// Why:      The owner of a key depends on whether the query field has focus, and only the field knows that.
//
// In TS you'd write (pseudocode):
// ```ts
// const HostKeyLocalSearchField = createContext<(focused: boolean) => void>(() => {});
// ```
/** Callback that the Search query field calls with its focus state. */
val HostKeyLocalSearchField: ProvidableCompositionLocal<(Boolean) -> Unit> = compositionLocalOf { { _ -> } }
