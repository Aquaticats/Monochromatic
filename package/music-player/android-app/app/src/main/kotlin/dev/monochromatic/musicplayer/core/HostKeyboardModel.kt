// Host keyboard glue for decision D102. It turns one key press plus the host's route and focus state into the
// command the host runs. The shortcut classifier in KeyboardMap.kt decides which action a key means; this file adds
// the owner, route, repeat, and product-exception rules that the host applies on top of it.
// Nothing here imports Compose or Android, so every rule runs in JVM unit tests.

// What:     `package dev.monochromatic.musicplayer.core` places the glue beside the classifier it wraps.
// Why:      The glue reads the classifier types and `PlaybackMode` from this package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `const val HOST_KEY_NAVIGATE_FROM_EDITOR: Boolean = false` decides whether global chords may leave the
//           template editor.
// Why:      The planning record leaves unsaved-template handling open, so the default keeps the editor in place.
//           Setting it to true lets Search, Settings, and the same-visit Search focus run from the editor.
//
// In TS you'd write (pseudocode):
// ```ts
// const HOST_KEY_NAVIGATE_FROM_EDITOR = false;
// ```
/** Whether the Search and Settings shortcuts may leave the template editor. */
const val HOST_KEY_NAVIGATE_FROM_EDITOR: Boolean = false

// What:     `enum class HostKeyRoute` names the four destinations that the shortcut rules tell apart.
// Why:      The host's full route type lives in the app package, so the pure rules use this smaller copy of its kinds.
//
// In TS you'd write (pseudocode):
// ```ts
// type HostKeyRoute = "player" | "search" | "settings" | "editor";
// ```
/** Destinations of the host that the keyboard rules distinguish. */
enum class HostKeyRoute {
    // What:     `PLAYER` is the player screen with its track list or folder picker.
    // Why:      Transport keys and the folder picker act only on this destination.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const PLAYER = "player";
    // ```
    /** The player screen with its track list or folder picker. */
    PLAYER,

    // What:     `SEARCH` is the Search page.
    // Why:      Escape leaves Search, and the Search shortcut refocuses it without a new visit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const SEARCH = "search";
    // ```
    /** The Search page. */
    SEARCH,

    // What:     `SETTINGS` is the Settings page.
    // Why:      The Settings shortcut does nothing here because the page is already open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const SETTINGS = "settings";
    // ```
    /** The Settings page that lists the two templates. */
    SETTINGS,

    // What:     `EDITOR` is the editor of one template.
    // Why:      Its text field keeps its keys, and the navigation chords stay out unless the policy allows them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const EDITOR = "editor";
    // ```
    /** The editor of one template, reached from Settings. */
    EDITOR,
}

// What:     `data class HostKeyInput(...)` is one key-down as the host reads it from the platform event.
// Why:      The classifier needs the key text and the four modifiers, and the repeat flag, as plain values.
//
// In TS you'd write (pseudocode):
// ```ts
// type HostKeyInput = { keyName: string; ctrl: boolean; meta: boolean; alt: boolean; shift: boolean; repeat: boolean };
// ```
/** One key-down event with the key text, the modifiers, and whether it repeats. */
data class HostKeyInput(
    // What:     `val keyName: String` is the key text the classifier compares, or an empty text for an unmapped key.
    // Why:      An empty text tells the glue that no shortcut uses the key, so it is never classified.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // keyName: string;
    // ```
    /** Key text such as `f`, a space, or `ArrowLeft`, or empty when no shortcut uses the key. */
    val keyName: String,

    // What:     `val ctrl: Boolean` is true while Control is held.
    // Why:      The Windows and Linux map uses Control for its primary chords.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // ctrl: boolean;
    // ```
    /** True while Control is held. */
    val ctrl: Boolean,

    // What:     `val meta: Boolean` is true while the Meta key is held.
    // Why:      The classifier rejects its letter chords when Meta is held, so the flag must reach it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // meta: boolean;
    // ```
    /** True while Meta is held. */
    val meta: Boolean,

    // What:     `val alt: Boolean` is true while Alt is held.
    // Why:      The Settings chord needs Alt, and every other chord must have it released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // alt: boolean;
    // ```
    /** True while Alt is held. */
    val alt: Boolean,

    // What:     `val shift: Boolean` is true while Shift is held.
    // Why:      Chords with Shift must not match the plain shortcuts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // shift: boolean;
    // ```
    /** True while Shift is held. */
    val shift: Boolean,

    // What:     `val repeat: Boolean` is true for an auto-repeated key-down.
    // Why:      A held key must not run a host command many times.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // repeat: boolean;
    // ```
    /** True when the key-down is an auto-repeat of a held key. */
    val repeat: Boolean,
)

// What:     `enum class HostKeyLayout` names the two layouts that the keyboard rules tell apart.
// Why:      The folder picker is part of the folded cover only, so its shortcut depends on which layout is drawn.
//
// In TS you'd write (pseudocode):
// ```ts
// type HostKeyLayout = "folded-cover" | "unfolded";
// ```
/** Layouts of the player screen that the keyboard rules distinguish. */
enum class HostKeyLayout {
    // What:     `FOLDED_COVER` is the single-column cover layout.
    // Why:      Only this layout shows the folder picker that Ctrl+O toggles.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // "folded-cover"
    // ```
    /** The single-column folded cover, which shows the folder picker. */
    FOLDED_COVER,

    // What:     `UNFOLDED` is the two-pane layout on wide screens.
    // Why:      The folder picker is not drawn here, so Ctrl+O has no meaning on this layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // "unfolded"
    // ```
    /** The two-pane unfolded layout, which draws no folder picker. */
    UNFOLDED,
}

// What:     `data class HostKeyContext(...)` is the host state that decides who owns a key and what a command may do.
// Why:      The owner, the route rules, and the mode cycle all read the same snapshot, so one value carries them.
//
// In TS you'd write (pseudocode):
// ```ts
// type HostKeyContext = {
//   route: HostKeyRoute; menuOpen: boolean; textFieldFocused: boolean; rootFocused: boolean;
//   playbackMode: PlaybackMode; layout: HostKeyLayout;
// };
// ```
/** The host state that the keyboard rules read for one key press. */
data class HostKeyContext(
    // What:     `val route: HostKeyRoute` is the destination the host shows.
    // Why:      The route decides whether transport keys and route-specific commands apply.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // route: HostKeyRoute;
    // ```
    /** The destination the host shows. */
    val route: HostKeyRoute,

    // What:     `val menuOpen: Boolean` is true while the track menu popup is open.
    // Why:      An open popup takes every key before any shortcut.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // menuOpen: boolean;
    // ```
    /** True while the track menu popup is open. */
    val menuOpen: Boolean,

    // What:     `val textFieldFocused: Boolean` is true while a text field holds focus.
    // Why:      A focused text field keeps its own keys, so the owner becomes the editor.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // textFieldFocused: boolean;
    // ```
    /** True while the Search query field or the template text field holds focus. */
    val textFieldFocused: Boolean,

    // What:     `val rootFocused: Boolean` is true while the host's own focusable root holds focus.
    // Why:      Only a focused playback area owns the transport keys, so this flag gates them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rootFocused: boolean;
    // ```
    /** True while the playback area, which is the host's own focusable root, holds focus. */
    val rootFocused: Boolean,

    // What:     `val playbackMode: PlaybackMode` is the end-of-track mode that the host shows now.
    // Why:      The mode cycle moves from this value to the next one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playbackMode: PlaybackMode;
    // ```
    /** The end-of-track mode that is in effect now. */
    val playbackMode: PlaybackMode,

    // What:     `val layout: HostKeyLayout` is the layout the screen draws at the current width.
    // Why:      The folder picker exists only on the folded cover, so its shortcut must know the layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // layout: HostKeyLayout;
    // ```
    /** The layout the screen draws now, which decides whether the folder picker shortcut applies. */
    val layout: HostKeyLayout,

    // What:     `val zone: HostKeyZone` names the list that holds keyboard focus, or none.
    // Why:      The list keys and the playback chords of D106 apply by focus zone, and a focused list keeps the
    //           playback keys that a focused button would keep for itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // zone: HostKeyZone;
    // ```
    /** The folder rail or the track list when one holds keyboard focus, else none. */
    val zone: HostKeyZone = HostKeyZone.NONE,

    // What:     `val pickerOpen: Boolean` is true while the folder picker covers the track list on the folded cover.
    // Why:      Escape closes the picker first, as the Back button does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // pickerOpen: boolean;
    // ```
    /** True while the folded cover shows the folder picker in place of the track list. */
    val pickerOpen: Boolean = false,
)

// What:     `data class HostKeyPress(...)` joins one key-down with the host state that it arrived in.
// Why:      The glue takes one value, so each rule function keeps a single parameter.
//
// In TS you'd write (pseudocode):
// ```ts
// type HostKeyPress = { key: HostKeyInput; context: HostKeyContext };
// ```
/** One key-down together with the host state at the moment it arrived. */
data class HostKeyPress(
    // What:     `val key: HostKeyInput` is the key-down.
    // Why:      The classifier request is built from its key text and modifiers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // key: HostKeyInput;
    // ```
    /** The key-down event. */
    val key: HostKeyInput,

    // What:     `val context: HostKeyContext` is the host state at the moment of the key-down.
    // Why:      The owner and the route rules both read it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // context: HostKeyContext;
    // ```
    /** The host state when the key-down arrived. */
    val context: HostKeyContext,
)

// What:     `sealed interface HostKeyCommand` lists what the host does with a key-down.
// Why:      The glue returns one of these, so the Compose side only dispatches and never decides.
//
// In TS you'd write (pseudocode):
// ```ts
// type HostKeyCommand = "unhandled" | "swallow-repeat" | "toggle" | ...;
// ```
/** The host command that one key-down resolves to. */
sealed interface HostKeyCommand {
    // What:     `data object Unhandled` means the host does nothing and the key goes on to its owner.
    // Why:      Text fields, focused controls, and unmapped keys keep their keys, so the host must not consume them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const unhandled = { consumed: false };
    // ```
    /** The host does nothing and does not consume the key. */
    data object Unhandled : HostKeyCommand

    // What:     `data object SwallowRepeat` means the host consumes a repeat without acting on it.
    // Why:      Holding a shortcut must not run its action many times, and the classifier leaves that to the host.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const swallowRepeat = { consumed: true, action: null };
    // ```
    /** The host consumes an auto-repeat of a shortcut and runs nothing. */
    data object SwallowRepeat : HostKeyCommand

    // What:     `data object TogglePlayback` plays or pauses the player.
    // Why:      Space toggles playback while the playback area has focus.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const togglePlayback = { action: "toggle" };
    // ```
    /** Plays or pauses the player. */
    data object TogglePlayback : HostKeyCommand

    // What:     `data object PreviousTrack` moves to the previous track.
    // Why:      It is the backward transport chord on the Windows and Linux map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const previousTrack = { action: "previous" };
    // ```
    /** Moves to the previous track. */
    data object PreviousTrack : HostKeyCommand

    // What:     `data object NextTrack` moves to the next track.
    // Why:      It is the forward transport chord on the Windows and Linux map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const nextTrack = { action: "next" };
    // ```
    /** Moves to the next track. */
    data object NextTrack : HostKeyCommand

    // What:     `data class CycleMode(val mode: PlaybackMode)` sets the end-of-track mode to the next one.
    // Why:      Control plus M cycles the mode, and the glue computes the target so the host only applies it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cycleMode = (mode: PlaybackMode) => ({ action: "mode", mode });
    // ```
    /** Sets the end-of-track mode to the given mode. */
    data class CycleMode(
        // What:     `val mode: PlaybackMode` is the mode to apply.
        // Why:      The host applies exactly this value, so the cycle is decided in one place.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // mode: PlaybackMode;
        // ```
        /** The end-of-track mode to apply. */
        val mode: PlaybackMode,
    ) : HostKeyCommand

    // What:     `data object OpenSearch` opens Search with an empty query.
    // Why:      A new Search visit starts empty, as D66 requires.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const openSearch = { query: "", route: "search" };
    // ```
    /** Opens Search as a new visit with an empty query. */
    data object OpenSearch : HostKeyCommand

    // What:     `data object FocusSearch` focuses the query field of the Search page that is already open.
    // Why:      A same-visit refocus keeps the query text, as D63 and D67 require.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const focusSearch = () => searchInput.focus();
    // ```
    /** Focuses the query field of the open Search page without clearing the query. */
    data object FocusSearch : HostKeyCommand

    // What:     `data object ToggleFolderPicker` opens or closes the folder picker on the player.
    // Why:      Control plus O toggles the picker for the current library.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const toggleFolderPicker = () => setPickerOpen(!pickerOpen);
    // ```
    /** Opens or closes the folder picker. */
    data object ToggleFolderPicker : HostKeyCommand

    // What:     `data object OpenSettings` opens the Settings page.
    // Why:      Control plus Alt plus S opens Settings from the Windows and Linux map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const openSettings = () => go("settings");
    // ```
    /** Opens the Settings page. */
    data object OpenSettings : HostKeyCommand

    // What:     `data object RevealCurrent` focuses the playing track's row without starting or changing a track.
    // Why:      Control plus G is the reveal chord, and it must not run a transport action.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const revealCurrent = () => focusRow(currentIndex);
    // ```
    /** Focuses the row of the playing track, scrolling it into view when needed. */
    data object RevealCurrent : HostKeyCommand

    // What:     `data object BackFromSearch` leaves Search by the same rule as the system Back button.
    // Why:      Escape and Back must agree, so the host applies the route rule that the Back handler uses.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const backFromSearch = () => go(backFrom("search"));
    // ```
    /** Leaves Search by the route rule that the Back handler uses. */
    data object BackFromSearch : HostKeyCommand

    // What:     `data object ClosePopup` closes the open track menu.
    // Why:      Escape closes the innermost popup before any other meaning applies.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const closePopup = () => setMenuTarget(null);
    // ```
    /** Closes the open track menu. */
    data object ClosePopup : HostKeyCommand
}

// What:     `interface HostKeyActions` is the set of host operations that the commands run.
// Why:      The glue names each operation once, and the Compose host and the tests supply their own implementations.
//
// In TS you'd write (pseudocode):
// ```ts
// interface HostKeyActions {
//   togglePlayback(): void; previousTrack(): void; nextTrack(): void; cycleMode(mode: PlaybackMode): void;
//   openSearch(): void; focusSearch(): void; toggleFolderPicker(): void; openSettings(): void;
//   revealCurrent(): void; backFromSearch(): void; closePopup(): void;
// }
// ```
/** Host operations that the keyboard commands run. */
interface HostKeyActions {
    // What:     `fun togglePlayback()` plays or pauses the player.
    // Why:      It is the one action behind Space.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // togglePlayback(): void;
    // ```
    /** Plays or pauses the player. */
    fun togglePlayback()

    // What:     `fun previousTrack()` moves to the previous track.
    // Why:      It is the one action behind the backward chord.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // previousTrack(): void;
    // ```
    /** Moves to the previous track. */
    fun previousTrack()

    // What:     `fun nextTrack()` moves to the next track.
    // Why:      It is the one action behind the forward chord.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // nextTrack(): void;
    // ```
    /** Moves to the next track. */
    fun nextTrack()

    // What:     `fun cycleMode(mode: PlaybackMode)` applies one end-of-track mode.
    // Why:      The glue decides the target mode, and the host only stores it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // cycleMode(mode: PlaybackMode): void;
    // ```
    /** Applies the given end-of-track mode. */
    fun cycleMode(mode: PlaybackMode)

    // What:     `fun openSearch()` opens Search as a new visit with an empty query.
    // Why:      A genuine new visit starts empty, as D66 requires.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // openSearch(): void;
    // ```
    /** Opens Search as a new visit with an empty query. */
    fun openSearch()

    // What:     `fun focusSearch()` refocuses the open Search query field.
    // Why:      A same-visit refocus keeps the query text, as D63 and D67 require.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // focusSearch(): void;
    // ```
    /** Refocuses the open Search query field without clearing its text. */
    fun focusSearch()

    // What:     `fun toggleFolderPicker()` opens or closes the folder picker.
    // Why:      The picker is the cover's list-replacing pane, and Control plus O toggles it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // toggleFolderPicker(): void;
    // ```
    /** Opens or closes the folder picker. */
    fun toggleFolderPicker()

    // What:     `fun openSettings()` opens the Settings page.
    // Why:      It is the one action behind the Settings chord.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // openSettings(): void;
    // ```
    /** Opens the Settings page. */
    fun openSettings()

    // What:     `fun revealCurrent()` focuses the playing track's row.
    // Why:      It is the one action behind the reveal chord, and it starts no track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // revealCurrent(): void;
    // ```
    /** Focuses the row of the playing track. */
    fun revealCurrent()

    // What:     `fun backFromSearch()` leaves Search by the route rule of the system Back button.
    // Why:      Escape and Back must lead to the same destination.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // backFromSearch(): void;
    // ```
    /** Leaves Search by the route rule of the system Back button. */
    fun backFromSearch()

    // What:     `fun closePopup()` closes the open track menu.
    // Why:      It is the one action behind Escape while a popup is open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // closePopup(): void;
    // ```
    /** Closes the open track menu. */
    fun closePopup()

    // What:     `fun seekBy(seconds: Int)` moves the playhead by a number of seconds.
    // Why:      It is the one action behind Alt+Left and Alt+Right, with and without Shift.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // seekBy(seconds: number): void;
    // ```
    /** Moves the playhead by [seconds], clamped to the track. */
    fun seekBy(seconds: Int)

    // What:     `fun restartTrack()` returns the playhead to the start.
    // Why:      It is the one action behind Ctrl+Home.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // restartTrack(): void;
    // ```
    /** Restarts the playing track from its beginning. */
    fun restartTrack()

    // What:     `fun stepFolder(step: Int)` selects the folder a number of places away.
    // Why:      It is the one action behind Alt+Up (minus one) and Alt+Down (plus one).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // stepFolder(step: number): void;
    // ```
    /** Selects the folder [step] places from the selected one, without starting a track. */
    fun stepFolder(step: Int)

    // What:     `fun revealPlayingTrackFolder()` shows the playing track's folder in the track list.
    // Why:      It is the one action behind Ctrl+Shift+G.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // revealPlayingTrackFolder(): void;
    // ```
    /** Selects the playing track's folder and scrolls the track list to it. */
    fun revealPlayingTrackFolder()

    // What:     `fun focusZone(zone: HostKeyZone)` moves keyboard focus to the folder rail or the track list.
    // Why:      It is the one action behind Alt+1 (folders) and Alt+2 (tracks).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // focusZone(zone: HostKeyZone): void;
    // ```
    /** Moves keyboard focus into the list that [zone] names, opening the folder picker on the cover when needed. */
    fun focusZone(zone: HostKeyZone)

    // What:     `fun openFolderDialog()` opens the system folder chooser.
    // Why:      It is the one action behind Ctrl+Shift+O, the same as the Open button.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // openFolderDialog(): void;
    // ```
    /** Opens the system folder chooser, as the Open button does. */
    fun openFolderDialog()

    // What:     `fun closePicker()` closes the folder picker on the folded cover.
    // Why:      It is the one action behind Escape while the picker covers the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // closePicker(): void;
    // ```
    /** Closes the folder picker, as the Back button does. */
    fun closePicker()
}

// What:     `fun hostKeyExecutes(command: HostKeyCommand): Boolean` is true when the command runs a host operation.
// Why:      Only executing commands run an action, and only `Unhandled` leaves the key to its owner.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyExecutes = (command: HostKeyCommand) => command !== "unhandled" && command !== "swallow-repeat";
// ```
/** Reports whether the command runs a host operation. */
fun hostKeyExecutes(command: HostKeyCommand): Boolean =
    command != HostKeyCommand.Unhandled && command != HostKeyCommand.SwallowRepeat && command !is ListKeyCommand

// What:     `fun hostKeyOwnerOf(context: HostKeyContext): ShortcutOwner` names the focus owner that receives the key.
// Why:      A focused text field keeps its keys, the playback area owns transport keys only on the player route, and
//           every other focus is treated as a control. Editors and controls behave the same in the classifier.
//
// In TS you'd write (pseudocode):
// ```ts
// function ownerOf(c: HostKeyContext): ShortcutOwner {
//   if (c.textFieldFocused) return "editor";
//   if (c.route === "player" && c.rootFocused) return "player";
//   return "control";
// }
// ```
/** Names the focus owner for one host state. */
fun hostKeyOwnerOf(context: HostKeyContext): ShortcutOwner = when {
    context.textFieldFocused -> ShortcutOwner.EDITOR
    context.route == HostKeyRoute.PLAYER && (context.rootFocused || context.zone != HostKeyZone.NONE) ->
        ShortcutOwner.PLAYER
    else -> ShortcutOwner.CONTROL
}

// What:     `fun hostKeyRequestOf(press: HostKeyPress): ShortcutRequest` builds the classifier request for one press.
// Why:      The host reads the Windows and Linux map only, because an Android device has no reliable platform signal.
//           Composition is not observable from Compose, so `composing` stays false and text keys stay with the editor.
//
// In TS you'd write (pseudocode):
// ```ts
// const request = { platform: "windows-linux", key: press.key.keyName, ...modifiers, owner: ownerOf(press.context) };
// ```
/** Builds the classifier request for one key-down on the Windows and Linux map. */
fun hostKeyRequestOf(press: HostKeyPress): ShortcutRequest = ShortcutRequest(
    platform = KeyboardPlatform.WINDOWS_LINUX,
    key = press.key.keyName,
    ctrl = press.key.ctrl,
    meta = press.key.meta,
    alt = press.key.alt,
    shift = press.key.shift,
    repeat = press.key.repeat,
    owner = hostKeyOwnerOf(press.context),
    inPopup = press.context.menuOpen,
    inSearch = press.context.route == HostKeyRoute.SEARCH,
    inWideSettings = (press.context.route == HostKeyRoute.SETTINGS || press.context.route == HostKeyRoute.EDITOR) &&
        press.context.layout == HostKeyLayout.UNFOLDED,
    composing = false,
)

// What:     `fun hostKeyNextPlaybackMode(mode: PlaybackMode): PlaybackMode` returns the end-of-track mode after the
//           given one.
// Why:      The cycle follows the order of the transport deck's mode segments, and it wraps around after the last one.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyNextPlaybackMode = (mode: PlaybackMode) => modes[(modes.indexOf(mode) + 1) % modes.length];
// ```
/** Returns the end-of-track mode that follows the given mode, wrapping after the last one. */
fun hostKeyNextPlaybackMode(mode: PlaybackMode): PlaybackMode {
    /** Every end-of-track mode in the order the transport deck shows them. */
    val modes: List<PlaybackMode> = PlaybackMode.entries
    return modes[(mode.ordinal + 1) % modes.size]
}
// What:     `private fun hostKeySearchCommandFor(press: HostKeyPress): HostKeyCommand` picks the Search command for
//           the route.
// Why:      On Search the shortcut only refocuses the field, so the query survives; elsewhere it opens a new visit.
//           The editor is left in place unless the navigation policy allows it.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeySearchCommandFor = (p) => p.route === "search" ? "focus-search"
//   : p.route === "editor" && !NAVIGATE_FROM_EDITOR ? "unhandled" : "open-search";
// ```
/** Chooses the Search command for the route that received the key. */
private fun hostKeySearchCommandFor(press: HostKeyPress): HostKeyCommand = when {
    press.context.route == HostKeyRoute.SEARCH -> HostKeyCommand.FocusSearch
    press.context.route == HostKeyRoute.EDITOR && !HOST_KEY_NAVIGATE_FROM_EDITOR -> HostKeyCommand.Unhandled
    else -> HostKeyCommand.OpenSearch
}

// What:     `private fun hostKeyPickerCommandFor(press: HostKeyPress): HostKeyCommand` allows the picker on the folded
//           cover player only.
// Why:      The picker is drawn only on the folded cover, so toggling it on the unfolded layout would change state
//           nobody can see.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyPickerCommandFor = (p) =>
//   p.route === "player" && p.layout === "folded-cover" ? "toggle-picker" : "unhandled";
// ```
/** Chooses the folder picker command for the route and layout that received the key. */
private fun hostKeyPickerCommandFor(press: HostKeyPress): HostKeyCommand =
    if (press.context.route == HostKeyRoute.PLAYER && press.context.layout == HostKeyLayout.FOLDED_COVER) {
        HostKeyCommand.ToggleFolderPicker
    } else {
        HostKeyCommand.Unhandled
    }

// What:     `private fun hostKeySettingsCommandFor(press: HostKeyPress): HostKeyCommand` opens Settings unless already
//           open.
// Why:      The Settings chord does nothing on the page it opens, and it leaves the editor only under the policy.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeySettingsCommandFor = (p) => p.route === "settings" ? "unhandled"
//   : p.route === "editor" && !NAVIGATE_FROM_EDITOR ? "unhandled" : "open-settings";
// ```
/** Chooses the Settings command for the route that received the key. */
private fun hostKeySettingsCommandFor(press: HostKeyPress): HostKeyCommand = when {
    press.context.route == HostKeyRoute.SETTINGS -> HostKeyCommand.Unhandled
    press.context.route == HostKeyRoute.EDITOR && !HOST_KEY_NAVIGATE_FROM_EDITOR -> HostKeyCommand.Unhandled
    else -> HostKeyCommand.OpenSettings
}

// What:     `private fun hostKeyRevealCommandFor(press: HostKeyPress): HostKeyCommand` reveals the playing row on the
//           player.
// Why:      Reveal works on the player route, and the host moves to the playing track's folder when it is elsewhere.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyRevealCommandFor = (p) => p.route === "player" ? "reveal" : "unhandled";
// ```
/** Chooses the reveal command for the route that received the key. */
private fun hostKeyRevealCommandFor(press: HostKeyPress): HostKeyCommand =
    if (press.context.route == HostKeyRoute.PLAYER) {
        HostKeyCommand.RevealCurrent
    } else {
        HostKeyCommand.Unhandled
    }

// What:     `private fun hostKeyRoutedOf(press: HostKeyPress): HostKeyCommand` maps the classifier action to a
//           command.
// Why:      Each action gets exactly one command, and the route and product-exception rules apply per action.
//
// In TS you'd write (pseudocode):
// ```ts
// function routedOf(press: HostKeyPress): HostKeyCommand { switch (classify(press)) { /* one case per action */ } }
// ```
/** Maps the classifier action for one press to a host command, before the repeat rule. */
private fun hostKeyRoutedOf(press: HostKeyPress): HostKeyCommand = when (shortcutAction(hostKeyRequestOf(press))) {
    ShortcutAction.OWNER, ShortcutAction.UNBOUND -> HostKeyCommand.Unhandled
    ShortcutAction.IGNORE_REPEAT -> HostKeyCommand.SwallowRepeat
    ShortcutAction.TOGGLE -> HostKeyCommand.TogglePlayback
    ShortcutAction.PREVIOUS -> HostKeyCommand.PreviousTrack
    ShortcutAction.NEXT -> HostKeyCommand.NextTrack
    ShortcutAction.MODE -> HostKeyCommand.CycleMode(hostKeyNextPlaybackMode(press.context.playbackMode))
    ShortcutAction.SEARCH -> hostKeySearchCommandFor(press)
    ShortcutAction.PICKER -> hostKeyPickerCommandFor(press)
    ShortcutAction.SETTINGS -> hostKeySettingsCommandFor(press)
    ShortcutAction.REVEAL -> hostKeyRevealCommandFor(press)
    ShortcutAction.BACK -> HostKeyCommand.BackFromSearch
    ShortcutAction.CLOSE_POPUP -> HostKeyCommand.ClosePopup
}

// What:     `fun hostKeyCommandFor(press: HostKeyPress): HostKeyCommand` resolves one key-down to its host command.
// Why:      This is the single entry the Compose side calls. Unmapped keys never reach the classifier, so a held
//           unmapped key is never swallowed, and a repeat of an executing command is swallowed without running it.
//
// In TS you'd write (pseudocode):
// ```ts
// function commandFor(press: HostKeyPress): HostKeyCommand {
//   if (press.key.keyName === "") return "unhandled";
//   const command = hostKeyRoutedOf(press);
//   return press.key.repeat && executes(command) ? "swallow-repeat" : command;
// }
// ```
/** Resolves one key-down to the command the host runs, or to `Unhandled` when the key goes on to its owner. */
fun hostKeyCommandFor(press: HostKeyPress): HostKeyCommand {
    if (press.key.keyName.isEmpty()) {
        return HostKeyCommand.Unhandled
    }
    /** Table row that the press hits, which decides before the classifier of the older rules. */
    val binding: KeyBinding? = bindingMatching(press)
    if (binding != null) {
        return if (press.key.repeat && !binding.repeats) HostKeyCommand.SwallowRepeat else binding.command
    }
    /** Jump to a typed letter in the folder rail, which the table does not list one row at a time. */
    val letterJump: KeyJumpToLetter? = letterJumpCommandFor(press)
    if (letterJump != null) {
        return if (press.key.repeat) HostKeyCommand.SwallowRepeat else letterJump
    }
    /** Command for the press before the repeat rule applies. */
    val routed: HostKeyCommand = hostKeyRoutedOf(press)
    return if (press.key.repeat && hostKeyExecutes(routed)) HostKeyCommand.SwallowRepeat else routed
}

// What:     `class HostKeyDispatcher(...)` runs one command against the host actions.
// Why:      The Compose side hands each command here, so the mapping from command to action is tested without Compose.
//
// In TS you'd write (pseudocode):
// ```ts
// class HostKeyDispatcher {
//   constructor(private readonly actions: HostKeyActions) {}
//   dispatch(command: HostKeyCommand): boolean { /* ... */ }
// }
// ```
/** Runs host key commands against one set of host actions and reports whether the key was consumed. */
class HostKeyDispatcher(
    // What:     `private val actions: HostKeyActions` is the host operations this dispatcher runs.
    // Why:      The dispatcher holds them so each command needs only its own value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly actions: HostKeyActions;
    // ```
    /** Host operations that the commands run. */
    private val actions: HostKeyActions,
) {
    // What:     `private fun runAction(command: HostKeyCommand)` calls the host operation for an executing command.
    // Why:      One `when` keeps every command paired with exactly one operation, and the compiler checks each branch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function runAction(command: HostKeyCommand): void { switch (command.kind) { /* one call per kind */ } }
    // ```
    /** Calls the host operation that belongs to one executing command. */
    private fun runAction(command: HostKeyCommand) {
        when (command) {
            HostKeyCommand.Unhandled -> Unit
            HostKeyCommand.SwallowRepeat -> Unit
            HostKeyCommand.TogglePlayback -> actions.togglePlayback()
            HostKeyCommand.PreviousTrack -> actions.previousTrack()
            HostKeyCommand.NextTrack -> actions.nextTrack()
            is HostKeyCommand.CycleMode -> actions.cycleMode(command.mode)
            HostKeyCommand.OpenSearch -> actions.openSearch()
            HostKeyCommand.FocusSearch -> actions.focusSearch()
            HostKeyCommand.ToggleFolderPicker -> actions.toggleFolderPicker()
            HostKeyCommand.OpenSettings -> actions.openSettings()
            HostKeyCommand.RevealCurrent -> actions.revealCurrent()
            HostKeyCommand.BackFromSearch -> actions.backFromSearch()
            HostKeyCommand.ClosePopup -> actions.closePopup()
            is KeySeekBy -> actions.seekBy(command.seconds)
            KeyRestartTrack -> actions.restartTrack()
            KeyPreviousFolder -> actions.stepFolder(-1)
            KeyNextFolder -> actions.stepFolder(1)
            KeyRevealFolder -> actions.revealPlayingTrackFolder()
            KeyFocusFolders -> actions.focusZone(HostKeyZone.NAMES)
            KeyFocusTracks -> actions.focusZone(HostKeyZone.TRACKS)
            KeyOpenFolderDialog -> actions.openFolderDialog()
            KeyClosePicker -> actions.closePicker()
            is ListKeyCommand -> Unit
        }
    }

    // What:     `fun dispatch(command: HostKeyCommand): Boolean` runs one command and reports whether it consumed it.
    // Why:      The Compose side consumes the key exactly when the command is not `Unhandled`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // dispatch(command: HostKeyCommand): boolean { if (command === "swallow-repeat") return true; ... }
    // ```
    /** Runs the command and returns true when the key is consumed, false when the owner keeps it. */
    fun dispatch(command: HostKeyCommand): Boolean {
        if (hostKeyExecutes(command)) {
            runAction(command)
        }
        return command != HostKeyCommand.Unhandled && command !is ListKeyCommand
    }
}
