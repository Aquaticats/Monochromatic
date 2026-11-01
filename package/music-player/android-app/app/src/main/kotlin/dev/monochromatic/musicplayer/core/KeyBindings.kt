// The D106 keyboard table for an Android device with an attached keyboard. Each row is one chord with its scope; the
// table is plain data, so every row is read against D106 and tested one by one. It is the Android counterpart of
// `package/music-player/desktop-app/src/keymap_bindings.rs`, limited to the Windows and Linux map (an Android
// keyboard has no Command key) and to the actions that make sense on a phone: there is no Quit, and the key hint
// badges and tooltips of D106 are left out.

// What:     `package dev.monochromatic.musicplayer.core` places the table beside the host key rules that consult it.
// Why:      The table takes plain values only, so it shares the core package and is unit-tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `enum class HostKeyZone` names which list, if any, holds the keyboard focus on the player page.
// Why:      D106 gives the list keys (arrows, Home, End, pages, Enter) to a focused list only, and the playback
//           chords work while the playback area or a list has focus, so the host reports which one it is.
//
// In TS you'd write (pseudocode):
// ```ts
// type HostKeyZone = "none" | "folders" | "tracks";
// ```
/** The list that holds keyboard focus on the player page. */
enum class HostKeyZone {
    /** No list holds focus: the playback area or another control does. */
    NONE,

    /** The folder rail, which is one focus stop with its own focus ring. */
    FOLDERS,

    /** The list of folder names beside the rail, which is also one focus stop with its own focus ring. */
    NAMES,

    /** The track list, which is one focus stop with its own focus ring. */
    TRACKS,
}

// What:     `sealed interface ListKeyCommand` marks the commands that a focused list runs itself.
// Why:      The ring lives inside the list, so the host root leaves these keys unconsumed and the focused list asks
//           the same table what the key means.
//
// In TS you'd write (pseudocode):
// ```ts
// type ListKeyCommand = MoveList | ActivateRow | JumpToLetter | OpenRowMenu | RowAction;
// ```
/** A command that the focused list handles, not the host root. */
sealed interface ListKeyCommand : HostKeyCommand

/** Moves the focus ring of the focused list. */
data class KeyMoveList(
    /** Movement the key asked for. */
    val move: ListMove,
) : ListKeyCommand

/** Activates the entry under the focus ring: selects the row or toggles play, or scrolls to the rail cell. */
data object KeyActivateRow : ListKeyCommand

/** Moves the focus ring of the rail to the first cell whose label starts with a typed character. */
data class KeyJumpToLetter(
    /** The typed character. */
    val letter: Char,
) : ListKeyCommand

/** Opens the context menu of the track row under the focus ring. */
data object KeyOpenRowMenu : ListKeyCommand

/** Runs one context menu action on the track row under the focus ring, without opening the menu. */
data class KeyRowAction(
    /** The menu action to run. */
    val action: TrackMenuAction,
) : ListKeyCommand

/** Moves the playhead by a number of seconds, clamped to the track. */
data class KeySeekBy(
    /** Seconds to move; negative moves back. */
    val seconds: Int,
) : HostKeyCommand

/** Restarts the track from its beginning. */
data object KeyRestartTrack : HostKeyCommand

/** Selects the previous folder in the folder order. */
data object KeyPreviousFolder : HostKeyCommand

/** Selects the next folder in the folder order. */
data object KeyNextFolder : HostKeyCommand

/** Shows the folder of the playing track in the track list. */
data object KeyRevealFolder : HostKeyCommand

/** Moves keyboard focus to the list of folder names, the folder picker's main list. */
data object KeyFocusFolders : HostKeyCommand

/** Moves keyboard focus to the track list. */
data object KeyFocusTracks : HostKeyCommand

/** Opens the system folder chooser. */
data object KeyOpenFolderDialog : HostKeyCommand

/** Closes the folder picker that covers the track list on the folded cover. */
data object KeyClosePicker : HostKeyCommand

// What:     `enum class BindingScope` says where a table row applies.
// Why:      The same key means different things per focus zone, and the scope keeps each row inert elsewhere.
//
// In TS you'd write (pseudocode):
// ```ts
// type BindingScope = "player-page" | "playback-area" | "picker-overlay" | "list-zone" | "folder-list" | "track-list";
// ```
/** Where a binding row applies. */
enum class BindingScope {
    /** The player page, whatever holds focus except a text field, with no track menu open. */
    PLAYER_PAGE,

    /** The player page with focus on the playback area, the folder rail or the track list. */
    PLAYBACK_AREA,

    /** The folded cover while the folder picker covers the track list. */
    PICKER_OVERLAY,

    /** The player page with focus in the folder rail or the track list. */
    LIST_ZONE,

    /** The player page with focus in the folder rail or the folder name list. */
    FOLDER_LIST,

    /** The player page with focus in the track list. */
    TRACK_LIST,
}

// What:     `data class KeyBinding(...)` is one row of the table.
// Why:      Plain data keeps the table auditable: each row can be read against D106 line by line.
//
// In TS you'd write (pseudocode):
// ```ts
// type KeyBinding = { command: HostKeyCommand; keys: string[]; ctrl: boolean; alt: boolean; shift: boolean;
//   scope: BindingScope; repeats: boolean };
// ```
/** One chord of the table: the command, the key names, the exact modifiers, the scope and the repeat rule. */
data class KeyBinding(
    /** Command the chord runs. */
    val command: HostKeyCommand,
    /** Key names that trigger it, in the classifier's spelling (`ArrowLeft`, `Home`, a lowercase letter). */
    val keys: List<String>,
    /** Whether Control must be held, and nothing else may be held beyond the other flags. */
    val ctrl: Boolean = false,
    /** Whether Alt must be held. */
    val alt: Boolean = false,
    /** Whether Shift must be held. */
    val shift: Boolean = false,
    /** Where the row applies. */
    val scope: BindingScope,
    /** True when holding the key may repeat the command. */
    val repeats: Boolean = false,
)

// What:     `val KEY_BINDINGS: List<KeyBinding>` is the whole D106 table for Android in matching order.
// Why:      The first row that matches a key press and whose scope applies decides.
//
// In TS you'd write (pseudocode):
// ```ts
// const KEY_BINDINGS: readonly KeyBinding[] = [ /* one row per chord */ ];
// ```
/** Every D106 chord that Android implements, in matching order. */
val KEY_BINDINGS: List<KeyBinding> = listOf(
    // Escape closes the folder picker that covers the list on the folded cover.
    KeyBinding(KeyClosePicker, listOf("Escape"), scope = BindingScope.PICKER_OVERLAY),
    // Playback: seek 10 s and 60 s, pick a mode directly, restart the track.
    KeyBinding(
        KeySeekBy(-SEEK_SHORT_SECONDS), listOf("ArrowLeft"), alt = true,
        scope = BindingScope.PLAYER_PAGE, repeats = true,
    ),
    KeyBinding(
        KeySeekBy(SEEK_SHORT_SECONDS), listOf("ArrowRight"), alt = true,
        scope = BindingScope.PLAYER_PAGE, repeats = true,
    ),
    KeyBinding(
        KeySeekBy(-SEEK_LONG_SECONDS), listOf("ArrowLeft"), alt = true, shift = true,
        scope = BindingScope.PLAYER_PAGE, repeats = true,
    ),
    KeyBinding(
        KeySeekBy(SEEK_LONG_SECONDS), listOf("ArrowRight"), alt = true, shift = true,
        scope = BindingScope.PLAYER_PAGE, repeats = true,
    ),
    KeyBinding(
        HostKeyCommand.CycleMode(PlaybackMode.REPEAT), listOf("1"), ctrl = true, shift = true,
        scope = BindingScope.PLAYER_PAGE,
    ),
    KeyBinding(
        HostKeyCommand.CycleMode(PlaybackMode.IN_ORDER), listOf("2"), ctrl = true, shift = true,
        scope = BindingScope.PLAYER_PAGE,
    ),
    KeyBinding(
        HostKeyCommand.CycleMode(PlaybackMode.SHUFFLE_PAGE), listOf("3"), ctrl = true, shift = true,
        scope = BindingScope.PLAYER_PAGE,
    ),
    KeyBinding(
        HostKeyCommand.CycleMode(PlaybackMode.SHUFFLE_ALL), listOf("4"), ctrl = true, shift = true,
        scope = BindingScope.PLAYER_PAGE,
    ),
    KeyBinding(KeyRestartTrack, listOf("Home"), ctrl = true, scope = BindingScope.PLAYBACK_AREA),
    // Browsing: previous and next folder, the playing track's folder, the two focus zones, the system dialog.
    KeyBinding(KeyPreviousFolder, listOf("ArrowUp"), alt = true, scope = BindingScope.PLAYER_PAGE),
    KeyBinding(KeyNextFolder, listOf("ArrowDown"), alt = true, scope = BindingScope.PLAYER_PAGE),
    KeyBinding(KeyRevealFolder, listOf("g"), ctrl = true, shift = true, scope = BindingScope.PLAYER_PAGE),
    KeyBinding(KeyFocusFolders, listOf("1"), alt = true, scope = BindingScope.PLAYER_PAGE),
    KeyBinding(KeyFocusTracks, listOf("2"), alt = true, scope = BindingScope.PLAYER_PAGE),
    KeyBinding(KeyOpenFolderDialog, listOf("o"), ctrl = true, shift = true, scope = BindingScope.PLAYER_PAGE),
    // Inside a focused list: the arrows, Home, End, the page keys and Enter. A letter has its own rule below.
    KeyBinding(KeyMoveList(ListMove.UP), listOf("ArrowUp"), scope = BindingScope.LIST_ZONE, repeats = true),
    KeyBinding(KeyMoveList(ListMove.DOWN), listOf("ArrowDown"), scope = BindingScope.LIST_ZONE, repeats = true),
    KeyBinding(KeyMoveList(ListMove.UP), listOf("ArrowLeft"), scope = BindingScope.FOLDER_LIST, repeats = true),
    KeyBinding(KeyMoveList(ListMove.DOWN), listOf("ArrowRight"), scope = BindingScope.FOLDER_LIST, repeats = true),
    KeyBinding(KeyMoveList(ListMove.HOME), listOf("Home"), scope = BindingScope.LIST_ZONE),
    KeyBinding(KeyMoveList(ListMove.END), listOf("End"), scope = BindingScope.LIST_ZONE),
    KeyBinding(KeyMoveList(ListMove.PAGE_UP), listOf("PageUp"), scope = BindingScope.LIST_ZONE, repeats = true),
    KeyBinding(KeyMoveList(ListMove.PAGE_DOWN), listOf("PageDown"), scope = BindingScope.LIST_ZONE, repeats = true),
    KeyBinding(KeyActivateRow, listOf("Enter"), scope = BindingScope.LIST_ZONE),
    // The focused track row: its menu (Menu key, Alt+Enter), trash, copy and show in the file manager.
    KeyBinding(KeyOpenRowMenu, listOf("ContextMenu"), scope = BindingScope.TRACK_LIST),
    KeyBinding(KeyRowAction(TrackMenuAction.MOVE_TO_TRASH), listOf("Delete"), scope = BindingScope.TRACK_LIST),
    KeyBinding(
        KeyRowAction(TrackMenuAction.COPY_FILENAME), listOf("c"), ctrl = true, shift = true,
        scope = BindingScope.TRACK_LIST,
    ),
    KeyBinding(
        KeyRowAction(TrackMenuAction.SHOW_IN_FILE_MANAGER), listOf("e"), ctrl = true, shift = true,
        scope = BindingScope.TRACK_LIST,
    ),
    KeyBinding(KeyOpenRowMenu, listOf("Enter"), alt = true, scope = BindingScope.TRACK_LIST),
)

// What:     `const val SEEK_SHORT_SECONDS` and `SEEK_LONG_SECONDS` are the two seek steps of D106.
// Why:      Alt+Left and Alt+Right seek 10 seconds, and 60 seconds with Shift added.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEEK_SHORT_SECONDS = 10; const SEEK_LONG_SECONDS = 60;
// ```
/** Seconds that Alt+Left and Alt+Right seek. */
const val SEEK_SHORT_SECONDS: Int = 10

/** Seconds that Alt+Shift+Left and Alt+Shift+Right seek. */
const val SEEK_LONG_SECONDS: Int = 60

// What:     `fun bindingScopeAllows(scope, context)` tells whether a row's scope holds in a host state.
// Why:      Every row applies only on the player page with no text field focused and no menu open; the scope then
//           narrows it to the playback area or to one list.
//
// In TS you'd write (pseudocode):
// ```ts
// function bindingScopeAllows(scope: BindingScope, c: HostKeyContext): boolean { ... }
// ```
/** Reports whether [scope] holds for the host state [context]. */
fun bindingScopeAllows(scope: BindingScope, context: HostKeyContext): Boolean {
    /** Whether the player page is shown with no text field typing and no track menu open. */
    val onPlayer: Boolean = context.route == HostKeyRoute.PLAYER && !context.textFieldFocused && !context.menuOpen
    return when (scope) {
        BindingScope.PLAYER_PAGE -> onPlayer
        BindingScope.PLAYBACK_AREA -> onPlayer && (context.rootFocused || context.zone != HostKeyZone.NONE)
        BindingScope.PICKER_OVERLAY -> onPlayer && context.pickerOpen && context.layout == HostKeyLayout.FOLDED_COVER
        BindingScope.LIST_ZONE -> onPlayer && context.zone != HostKeyZone.NONE
        BindingScope.FOLDER_LIST ->
            onPlayer && (context.zone == HostKeyZone.FOLDERS || context.zone == HostKeyZone.NAMES)
        BindingScope.TRACK_LIST -> onPlayer && context.zone == HostKeyZone.TRACKS
    }
}

// What:     `fun bindingMatching(press)` finds the first row that the key press hits.
// Why:      A row hits when the key name is listed, the Control, Alt and Shift flags equal the row's flags exactly
//           (so Ctrl+Shift+G is not Ctrl+G), Meta is not held, and the scope holds.
//
// In TS you'd write (pseudocode):
// ```ts
// const bindingMatching = (p: HostKeyPress) => KEY_BINDINGS.find(b => hits(b, p) && scopeAllows(b.scope, p.context));
// ```
/** Returns the first table row that the press hits, or null when no row applies. */
fun bindingMatching(press: HostKeyPress): KeyBinding? = KEY_BINDINGS.firstOrNull { binding ->
    press.key.keyName in binding.keys &&
        press.key.ctrl == binding.ctrl &&
        press.key.alt == binding.alt &&
        press.key.shift == binding.shift &&
        !press.key.meta &&
        bindingScopeAllows(binding.scope, press.context)
}

// What:     `fun letterJumpCommandFor(press)` maps a typed letter or digit in the folder rail to a jump.
// Why:      D106 moves the ring to the first entry starting with the typed character, with no modifier except
//           Shift (which only changes the case); it is a rule, not 36 table rows.
//
// In TS you'd write (pseudocode):
// ```ts
// const letterJumpCommandFor = (p) => inFolderList(p) && /^[a-z0-9]$/.test(p.key.keyName) ? jump(p.key.keyName) : null;
// ```
/** Returns the jump command for a typed letter or digit in the folder rail, or null for any other press. */
fun letterJumpCommandFor(press: HostKeyPress): KeyJumpToLetter? {
    /** Name of the pressed key, which is one lowercase character for a letter or digit. */
    val name: String = press.key.keyName
    if (name.length != 1 || !name[0].isLetterOrDigit()) {
        return null
    }
    if (press.key.ctrl || press.key.alt || press.key.meta) {
        return null
    }
    if (!bindingScopeAllows(BindingScope.FOLDER_LIST, press.context)) {
        return null
    }
    return KeyJumpToLetter(name[0])
}

// What:     `fun seekTargetSec(positionSec, durationSec, deltaSec)` is where a seek chord moves the playhead.
// Why:      A seek never leaves the track: it stops at the start and, for an unknown duration, at the new position.
//
// In TS you'd write (pseudocode):
// ```ts
// const seekTargetSec = (pos, dur, delta) => dur > 0 ? clamp(pos + delta, 0, dur) : Math.max(0, pos + delta);
// ```
/** Returns the playhead position in seconds after seeking by [deltaSec], kept inside the track. */
fun seekTargetSec(positionSec: Double, durationSec: Double, deltaSec: Int): Double {
    /** The unclamped target. */
    val target: Double = positionSec + deltaSec
    if (durationSec <= 0.0) {
        return target.coerceAtLeast(0.0)
    }
    return target.coerceIn(0.0, durationSec)
}

// What:     `fun adjacentFolderLabel(labels, selected, step)` picks the folder that Alt+Up or Alt+Down moves to.
// Why:      The chord steps through the folders in their order and stops at the first and last one.
//
// In TS you'd write (pseudocode):
// ```ts
// const adjacentFolderLabel = (labels, selected, step) => labels[clamp(selected + step, 0, labels.length - 1)] ?? null;
// ```
/** Returns the label of the folder [step] places from [selected], or null when there is no folder to move to. */
fun adjacentFolderLabel(labels: List<String>, selected: Int, step: Int): String? {
    if (labels.isEmpty()) {
        return null
    }
    /** Position reached by the step, kept inside the list. */
    val target: Int = (selected + step).coerceIn(0, labels.lastIndex)
    if (target == selected) {
        return null
    }
    return labels[target]
}

// What:     `fun menuRowActionFor(input)` is the menu action that a row chord means while the track menu is open.
// Why:      D106 lets the row chords act on the row the menu is open on, and the open menu is a popup window that
//           receives its own keys, so it asks the same table with no list-focus scope.
//
// In TS you'd write (pseudocode):
// ```ts
// const menuRowActionFor = (key: HostKeyInput) =>
//   KEY_BINDINGS.find(b => b.command is RowAction && hits(b, key))?.action;
// ```
/** Returns the menu action of the row chord that [input] presses, or null when it presses none. */
fun menuRowActionFor(input: HostKeyInput): TrackMenuAction? = KEY_BINDINGS.firstOrNull { binding ->
    binding.command is KeyRowAction &&
        input.keyName in binding.keys &&
        input.ctrl == binding.ctrl &&
        input.alt == binding.alt &&
        input.shift == binding.shift &&
        !input.meta &&
        !input.repeat
}?.let { binding -> (binding.command as KeyRowAction).action }
