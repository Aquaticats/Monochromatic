// Keyboard shortcut classifier ported from the accepted keyboard-map prototype at
// package/music-player/design/questions/keyboard-map.prototype.html.
// The checks run in the prototype's order, so every request gets the action the prototype gives.
// The prototype's unknown-platform throw is replaced by the KeyboardPlatform enum, whose fromName
// parser rejects unknown platform names before a request can be built.

// What:     `package dev.monochromatic.musicplayer.core` places this file beside the other core
//           playback types.
// Why:      The classifier is plain Kotlin with no Android imports, so JVM unit tests can run it.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `enum class KeyboardPlatform(val storedName: String)` declares the two keyboard maps.
//           Each value owns an explicit text name rather than Kotlin's generated `.name`.
// Why:      The classifier branches on exactly these two maps, and the only way to choose one from
//           text is the `fromName` parser, so an unknown platform cannot reach the classifier.
//
// In TS you'd write (pseudocode):
// ```ts
// type KeyboardPlatform = "windows-linux" | "mac";
// ```
/** Names the keyboard maps that the shortcut classifier understands. */
enum class KeyboardPlatform(
    // What:     `val storedName: String` is a property holding the text for each entry.
    // Why:      The prototype and its tests refer to platforms by this exact text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly storedName: string;
    // ```
    /** Stable platform text that the prototype and its tests use. */
    val storedName: String,
) {
    // What:     `WINDOWS_LINUX("windows-linux")` is one enum entry, the Windows and Linux map.
    // Why:      On this map Control is the primary modifier and Alt+Control chords are Settings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const WINDOWS_LINUX = "windows-linux";
    // ```
    /** Windows and Linux map, where Control is the primary modifier. */
    WINDOWS_LINUX("windows-linux"),

    // What:     `MAC("mac")` is the second enum entry, the macOS map.
    // Why:      On this map Command is the primary modifier and Control is a separate key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const MAC = "mac";
    // ```
    /** macOS map, where Command is the primary modifier. */
    MAC("mac");

    // What:     `companion object` groups helpers that belong to the type but need no instance.
    // Why:      The `fromName` parser turns stored text into a platform, so it lives beside the values.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // namespace KeyboardPlatform { /* parsing helpers */ }
    // ```
    /** Holds the text parser for keyboard platform names. */
    companion object {
        // What:     `fun fromName(text: String): KeyboardPlatform` returns the entry whose stored
        //           name equals `text`, and throws `IllegalArgumentException` for any other text.
        // Why:      The prototype throws `Unknown keyboard-map platform.` for unknown names. An enum
        //           cannot hold an unknown value, so the rejection moves to this parser.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // function fromName(text: string): KeyboardPlatform {
        //   if (text === "windows-linux") return "windows-linux";
        //   if (text === "mac") return "mac";
        //   throw new Error("Unknown keyboard-map platform.");
        // }
        // ```
        /** Parses stored platform text and rejects every other name. */
        fun fromName(text: String): KeyboardPlatform {
            // What:     `for (platform in entries)` walks every KeyboardPlatform entry in
            //           declaration order.
            // Why:      Matching against the stored names keeps the list of accepted names in one place.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // for (const platform of KeyboardPlatform.entries) { /* compare storedName */ }
            // ```
            for (platform in entries) {
                // What:     `platform.storedName == text` compares two strings by value.
                // Why:      Only an exact, case-sensitive match counts, as in the prototype.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (platform.storedName === text) { return platform; }
                // ```
                if (platform.storedName == text) {
                    return platform
                }
            }
            // What:     `throw IllegalArgumentException(...)` stops the call with an error object.
            // Why:      Unknown text must not silently choose a map, so the caller is told the name is wrong.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw new Error("Unknown keyboard-map platform.");
            // ```
            throw IllegalArgumentException("Unknown keyboard-map platform.")
        }
    }
}

// What:     `enum class ShortcutOwner` declares the four focus owners that can receive a key.
// Why:      The prototype's `ownerOf` returns one of these four labels, and the classifier reads them.
//
// In TS you'd write (pseudocode):
// ```ts
// type ShortcutOwner = "player" | "editor" | "control" | "surface";
// ```
/** Names the focus owner that received a key press. */
enum class ShortcutOwner {
    // What:     `PLAYER` is the playback shortcut area itself, the only owner that gets transport keys.
    // Why:      Transport keys must act only while the playback area has focus.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const PLAYER = "player";
    // ```
    /** The playback shortcut area itself. */
    PLAYER,

    // What:     `EDITOR` is a text field or other editable surface.
    // Why:      Editable surfaces keep their own letters and arrow keys.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const EDITOR = "editor";
    // ```
    /** A text field or editable region that keeps its own keys. */
    EDITOR,

    // What:     `CONTROL` is a button, input, select or summary element.
    // Why:      Focused controls keep Space and arrows for their own behavior.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const CONTROL = "control";
    // ```
    /** A focused button, input, select or summary that keeps its own keys. */
    CONTROL,

    // What:     `SURFACE` is any other element that is not the player, an editor, or a control.
    // Why:      Keys on other surfaces stay unbound, so they cannot change playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const SURFACE = "surface";
    // ```
    /** Any other surface, whose keys are never bound to playback. */
    SURFACE
}

// What:     `enum class ShortcutAction(val wireName: String)` declares every result the classifier can return.
// Why:      Each value keeps the exact string the prototype returns, so tests compare named results.
//
// In TS you'd write (pseudocode):
// ```ts
// type ShortcutAction = "toggle" | "owner" | "ignore-repeat" | "search" | "picker" | "settings"
//   | "previous" | "next" | "close-popup" | "back" | "reveal" | "mode" | "unbound";
// ```
/** Lists the actions a key press can resolve to, in the prototype's own spelling. */
enum class ShortcutAction(
    // What:     `val wireName: String` is a property holding the prototype's action text.
    // Why:      Tests and any future bridge can compare the same spelling the prototype uses.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly wireName: string;
    // ```
    /** Action text exactly as the prototype returns it. */
    val wireName: String,
) {
    // What:     `OWNER("owner")` means the key belongs to the focused editor, control, or composition.
    // Why:      The app must not handle the key, so the focused widget keeps it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const OWNER = "owner";
    // ```
    /** The focused text, control, or composition keeps this key. */
    OWNER("owner"),

    // What:     `CLOSE_POPUP("close-popup")` means Escape closes the popup.
    // Why:      Escape dismisses the innermost popup before any other meaning applies.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const CLOSE_POPUP = "close-popup";
    // ```
    /** Escape closes the open popup. */
    CLOSE_POPUP("close-popup"),

    // What:     `SEARCH("search")` means the Search shortcut was pressed.
    // Why:      It opens Search, or focuses it when it is already open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const SEARCH = "search";
    // ```
    /** Opens or focuses Search. */
    SEARCH("search"),

    // What:     `PICKER("picker")` means the folder picker shortcut was pressed.
    // Why:      It opens the current-library folder picker.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const PICKER = "picker";
    // ```
    /** Opens the current-library folder picker. */
    PICKER("picker"),

    // What:     `SETTINGS("settings")` means the Settings shortcut was pressed.
    // Why:      It opens in-app Settings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const SETTINGS = "settings";
    // ```
    /** Opens in-app Settings. */
    SETTINGS("settings"),

    // What:     `REVEAL("reveal")` means the reveal-current shortcut was pressed.
    // Why:      It focuses the current track without starting or changing a track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const REVEAL = "reveal";
    // ```
    /** Focuses the current track row without a transport action. */
    REVEAL("reveal"),

    // What:     `BACK("back")` means Escape was pressed while Search is open.
    // Why:      It returns from Search to the player.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const BACK = "back";
    // ```
    /** Returns from Search to the player. */
    BACK("back"),

    // What:     `UNBOUND("unbound")` means no shortcut applies to this key.
    // Why:      The key is left to the platform or the focused widget.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const UNBOUND = "unbound";
    // ```
    /** No shortcut applies. */
    UNBOUND("unbound"),

    // What:     `IGNORE_REPEAT("ignore-repeat")` means a held key repeated while the player owns it.
    // Why:      Auto-repeat must not toggle playback many times for one press.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const IGNORE_REPEAT = "ignore-repeat";
    // ```
    /** A repeated key event that the player ignores. */
    IGNORE_REPEAT("ignore-repeat"),

    // What:     `TOGGLE("toggle")` means play or pause.
    // Why:      Space toggles playback while the playback area has focus.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const TOGGLE = "toggle";
    // ```
    /** Plays or pauses the authored player. */
    TOGGLE("toggle"),

    // What:     `MODE("mode")` means cycle the end-of-track mode.
    // Why:      Control+M changes what happens after a track ends.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const MODE = "mode";
    // ```
    /** Cycles the end-of-track mode. */
    MODE("mode"),

    // What:     `PREVIOUS("previous")` means go to the previous track.
    // Why:      It is the backward transport key on either map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const PREVIOUS = "previous";
    // ```
    /** Moves to the previous track. */
    PREVIOUS("previous"),

    // What:     `NEXT("next")` means go to the next track.
    // Why:      It is the forward transport key on either map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const NEXT = "next";
    // ```
    /** Moves to the next track. */
    NEXT("next");
}

// What:     `data class ShortcutRequest(...)` is one key press with the focus and popup state
//           that the classifier reads. It has eleven named fields, and `copy(...)` makes a changed copy.
// Why:      The classifier needs all of these inputs together, and the named fields keep call sites readable.
//
// In TS you'd write (pseudocode):
// ```ts
// type ShortcutRequest = {
//   platform: KeyboardPlatform; key: string; ctrl: boolean; meta: boolean; alt: boolean;
//   shift: boolean; repeat: boolean; owner: ShortcutOwner; inPopup: boolean;
//   inSearch: boolean; composing: boolean;
// };
// ```
/** One key press with the platform, focus owner, and popup or search state. */
data class ShortcutRequest(
    // What:     `val platform: KeyboardPlatform` is the keyboard map that reads this key.
    // Why:      The same key means different shortcuts on the two maps.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // platform: KeyboardPlatform;
    // ```
    /** Keyboard map that interprets the key. */
    val platform: KeyboardPlatform,

    // What:     `val key: String` is the key text the platform reports, such as `f`, ` `, or `ArrowLeft`.
    // Why:      The classifier compares this text against each shortcut's key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // key: string;
    // ```
    /** Key text as the platform reports it. */
    val key: String,

    // What:     `val ctrl: Boolean` is true while the Control key is held.
    // Why:      Windows and Linux shortcuts use Control.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // ctrl: boolean;
    // ```
    /** True while Control is held. */
    val ctrl: Boolean,

    // What:     `val meta: Boolean` is true while the Command (macOS) or Meta (Windows and Linux) key is held.
    // Why:      macOS shortcuts use Command, and some Windows and Linux checks reject Meta.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // meta: boolean;
    // ```
    /** True while Command or Meta is held. */
    val meta: Boolean,

    // What:     `val alt: Boolean` is true while the Alt or Option key is held.
    // Why:      Settings on Windows and Linux uses Alt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // alt: boolean;
    // ```
    /** True while Alt or Option is held. */
    val alt: Boolean,

    // What:     `val shift: Boolean` is true while Shift is held.
    // Why:      Some macOS transport keys require Shift.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // shift: boolean;
    // ```
    /** True while Shift is held. */
    val shift: Boolean,

    // What:     `val repeat: Boolean` is true for an auto-repeated key event.
    // Why:      Repeated events for player keys are ignored so they do not fire again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // repeat: boolean;
    // ```
    /** True when the event is an auto-repeat of a held key. */
    val repeat: Boolean,

    // What:     `val owner: ShortcutOwner` is the focus owner that received the key.
    // Why:      Owners decide whether the key belongs to the player or to a widget.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // owner: ShortcutOwner;
    // ```
    /** Focus owner that received the key. */
    val owner: ShortcutOwner,

    // What:     `val inPopup: Boolean` is true while a popup is open.
    // Why:      An open popup takes every key before any other shortcut.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // inPopup: boolean;
    // ```
    /** True while a popup is open. */
    val inPopup: Boolean,

    // What:     `val inSearch: Boolean` is true while Search is open.
    // Why:      Escape returns from Search only while Search is open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // inSearch: boolean;
    // ```
    /** True while Search is open. */
    val inSearch: Boolean,

    // What:     `val inWideSettings: Boolean` is true while the two-pane Settings page is open.
    // Why:      D110 closes the wide Settings page on Escape, even from the editor pane's text field.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // inWideSettings: boolean;
    // ```
    /** True while the two-pane Settings page, or its editor, is open on a wide window. */
    val inWideSettings: Boolean = false,

    // What:     `val composing: Boolean` is true while an input method is still composing text.
    // Why:      Composition keys belong to the input method, so the classifier leaves them alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // composing: boolean;
    // ```
    /** True while an input method is composing text. */
    val composing: Boolean,
)

// What:     `fun shortcutAction(request: ShortcutRequest): ShortcutAction` classifies one key press.
// Why:      It is the port of the prototype's classifier. It runs the same rule groups in the same
//           order, and the first group that returns an action decides the result.
//
// In TS you'd write (pseudocode):
// ```ts
// function shortcutAction(request: ShortcutRequest): ShortcutAction {
//   return composingOrPopupAction(request) ?? searchAndPickerAction(request) ?? /* ... */ ?? "unbound";
// }
// ```
/** Classifies one key press into the action the prototype would take. */
fun shortcutAction(request: ShortcutRequest): ShortcutAction {
    // What:     `?:` (Elvis) returns the value on its left unless that value is null, and
    //           otherwise evaluates the value on its right. Each group returns `ShortcutAction?`.
    // Why:      A null result means "not this group", so the next group is tried.
    // Gotcha:   `?:` checks only for null, not for false or empty values.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return composingOrPopupAction(request) ?? searchAndPickerAction(request) ?? "unbound";
    // ```
    return composingOrPopupAction(request)
        ?: searchAndPickerAction(request)
        ?: settingsAction(request)
        ?: revealAction(request)
        ?: ownerAction(request)
        ?: playerKeyAction(request)
        ?: ShortcutAction.UNBOUND
}

// What:     `private fun composingOrPopupAction(request: ShortcutRequest): ShortcutAction?` handles the
//           rules that run before any shortcut: an input method still composing, and an open popup.
// Why:      These rules come first in the prototype, so a composing key or a popup key never reaches a shortcut.
//
// In TS you'd write (pseudocode):
// ```ts
// function composingOrPopupAction(request: ShortcutRequest): ShortcutAction | null {
//   if (request.composing) return "owner";
//   if (request.inPopup) return request.key === "Escape" ? "close-popup" : "owner";
//   return null;
// }
// ```
/** Returns the composition or popup action, or null when neither rule applies. */
private fun composingOrPopupAction(request: ShortcutRequest): ShortcutAction? {
    // What:     `if (request.composing) { ... }` tests whether an input method is composing text.
    // Why:      Composition keys go to the input method before any other rule.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (request.composing) return "owner";
    // ```
    if (request.composing) {
        return ShortcutAction.OWNER
    }

    // What:     This block handles an open popup. Escape closes it, and every other key stays with it.
    // Why:      The popup must take all keys, so no shortcut can change playback behind it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (request.inPopup) return request.key === "Escape" ? "close-popup" : "owner";
    // ```
    if (request.inPopup) {
        if (request.key == "Escape") {
            return ShortcutAction.CLOSE_POPUP
        }
        return ShortcutAction.OWNER
    }

    // What:     `return null` means that none of the rules in this group matched.
    // Why:      The caller then tries the next group.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return null;
    // ```
    return null
}

// What:     `private fun searchAndPickerAction(request: ShortcutRequest): ShortcutAction?` handles the
//           primary plus F and primary plus O letter shortcuts.
// Why:      Search and the folder picker are the first letter shortcuts in the prototype's order.
//
// In TS you'd write (pseudocode):
// ```ts
// function searchAndPickerAction(request: ShortcutRequest): ShortcutAction | null {
//   if (isPrimaryLetter(request, "f")) return "search";
//   if (isPrimaryLetter(request, "o")) return "picker";
//   return null;
// }
// ```
/** Returns the Search or folder picker action, or null when neither letter shortcut applies. */
private fun searchAndPickerAction(request: ShortcutRequest): ShortcutAction? {
    // What:     `isPrimaryLetter(request, "f")` is true when the primary key plus F is pressed alone.
    // Why:      Search opens on the accepted Search chord.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (isPrimaryLetter(request, "f")) return "search";
    // ```
    if (isPrimaryLetter(request, "f")) {
        return ShortcutAction.SEARCH
    }

    // What:     `isPrimaryLetter(request, "o")` is true when the primary key plus O is pressed alone.
    // Why:      The folder picker opens on the accepted picker chord.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (isPrimaryLetter(request, "o")) return "picker";
    // ```
    if (isPrimaryLetter(request, "o")) {
        return ShortcutAction.PICKER
    }

    // What:     `return null` means that no letter shortcut in this group matched.
    // Why:      The caller then tries the settings group.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return null;
    // ```
    return null
}

// What:     `private fun settingsAction(request: ShortcutRequest): ShortcutAction?` handles the two
//           Settings chords, one for each keyboard map.
// Why:      Settings chords come after the letter shortcuts and before reveal in the prototype's order.
//
// In TS you'd write (pseudocode):
// ```ts
// function settingsAction(request: ShortcutRequest): ShortcutAction | null {
//   if (platform === "mac" && onlyMeta && key === ",") return "settings";
//   if (platform === "windows-linux" && onlyCtrlAlt && lowerKey === "s") return "settings";
//   return null;
// }
// ```
/** Returns the Settings action, or null when neither Settings chord applies. */
private fun settingsAction(request: ShortcutRequest): ShortcutAction? {
    // What:     On macOS, Command plus comma opens Settings. No other modifier may be held.
    // Why:      This is the macOS Settings convention used by the accepted map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (platform === "mac" && meta && !ctrl && !alt && !shift && key === ",") return "settings";
    // ```
    if (request.platform == KeyboardPlatform.MAC && onlyMeta(request) && request.key == ",") {
        return ShortcutAction.SETTINGS
    }

    // What:     On Windows and Linux, Control plus Alt plus S opens Settings. Meta and Shift must not be held.
    // Why:      This is the Windows and Linux Settings chord in the accepted map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (platform === "windows-linux" && ctrl && alt && !meta && !shift && lowerKey === "s") return "settings";
    // ```
    if (request.platform == KeyboardPlatform.WINDOWS_LINUX && onlyCtrlAlt(request) && hasLetterKey(request, "s")) {
        return ShortcutAction.SETTINGS
    }

    // What:     `return null` means that neither Settings chord matched.
    // Why:      The caller then tries the reveal group.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return null;
    // ```
    return null
}

// What:     `private fun revealAction(request: ShortcutRequest): ShortcutAction?` handles the reveal chord,
//           which uses a different letter on each keyboard map.
// Why:      Reveal focuses the current track without a transport action, so it has its own group.
//
// In TS you'd write (pseudocode):
// ```ts
// function revealAction(request: ShortcutRequest): ShortcutAction | null {
//   if (isPrimaryLetter(request, request.platform === "mac" ? "l" : "g")) return "reveal";
//   return null;
// }
// ```
/** Returns the reveal action, or null when the reveal chord does not apply. */
private fun revealAction(request: ShortcutRequest): ShortcutAction? {
    // What:     `revealKeyFor(request.platform)` gives the reveal letter for this map, G or L.
    // Why:      Each map uses its own go-to-location letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (isPrimaryLetter(request, request.platform === "mac" ? "l" : "g")) return "reveal";
    // ```
    if (isPrimaryLetter(request, revealKeyFor(request.platform))) {
        return ShortcutAction.REVEAL
    }

    // What:     `return null` means that the reveal chord did not match.
    // Why:      The caller then tries the owner group.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return null;
    // ```
    return null
}

// What:     `private fun ownerAction(request: ShortcutRequest): ShortcutAction?` decides keys by who
//           owns focus: Escape back from Search, editors and controls, and any non-player surface.
// Why:      Text and controls keep their own keys, and surfaces that are not the player stay unbound.
//
// In TS you'd write (pseudocode):
// ```ts
// function ownerAction(request: ShortcutRequest): ShortcutAction | null {
//   if (request.key === "Escape" && request.inSearch) return "back";
//   if (owner === "editor" || owner === "control") return "owner";
//   if (owner !== "player") return "unbound";
//   return null;
// }
// ```
/** Returns the owner-based action, or null when the player owns the key. */
private fun ownerAction(request: ShortcutRequest): ShortcutAction? {
    // What:     `if (request.key == "Escape" && (request.inSearch || request.inWideSettings)) { ... }` goes back.
    // Why:      Escape leaves Search, and closes the wide Settings page, even though a text field owns the key
    //           otherwise.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (request.key === "Escape" && request.inSearch) return "back";
    // ```
    if (request.key == "Escape" && (request.inSearch || request.inWideSettings)) {
        return ShortcutAction.BACK
    }

    // What:     An editor or a control keeps every key that no shortcut group in this classifier claimed.
    // Why:      Typing and focused buttons must not also change playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (owner === "editor" || owner === "control") return "owner";
    // ```
    if (request.owner == ShortcutOwner.EDITOR || request.owner == ShortcutOwner.CONTROL) {
        return ShortcutAction.OWNER
    }

    // What:     Any owner other than the player leaves the key unbound.
    // Why:      Keys on a plain surface must not reach playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (owner !== "player") return "unbound";
    // ```
    if (request.owner != ShortcutOwner.PLAYER) {
        return ShortcutAction.UNBOUND
    }

    // What:     `return null` means the player owns the key and the player rules must decide.
    // Why:      The caller then tries the player-key group.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return null;
    // ```
    return null
}

// What:     `private fun playerKeyAction(request: ShortcutRequest): ShortcutAction?` handles the keys the
//           player owns: repeat, Space, Control plus M, and the two track-change chords.
// Why:      These keys act only when the playback area has focus, which the owner group has already checked.
//
// In TS you'd write (pseudocode):
// ```ts
// function playerKeyAction(request: ShortcutRequest): ShortcutAction | null {
//   if (repeat) return "ignore-repeat";
//   if (noModifiers && key === " ") return "toggle";
//   if (onlyCtrl && lowerKey === "m") return "mode";
//   // ...the Windows and Linux arrows and the macOS brackets follow...
//   return null;
// }
// ```
/** Returns the player-key action, or null when no player key rule matches. */
private fun playerKeyAction(request: ShortcutRequest): ShortcutAction? {
    // What:     `if (request.repeat) { ... }` checks for an auto-repeated key event.
    // Why:      Holding a key must not fire the same transport action many times.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (request.repeat) return "ignore-repeat";
    // ```
    if (request.repeat) {
        return ShortcutAction.IGNORE_REPEAT
    }

    // What:     Plain Space, with no modifier, toggles playback.
    // Why:      Space toggles playback only while the playback area has focus.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (noModifiers && key === " ") return "toggle";
    // ```
    if (noModifiers(request) && request.key == " ") {
        return ShortcutAction.TOGGLE
    }

    // What:     Control plus M cycles the end-of-track mode on both maps.
    // Why:      This is an accepted product exception that applies on both maps.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (onlyCtrl && lowerKey === "m") return "mode";
    // ```
    if (onlyCtrl(request) && hasLetterKey(request, "m")) {
        return ShortcutAction.MODE
    }

    // What:     On Windows and Linux, Control plus Left or Right moves to the previous or next track.
    // Why:      These arrow chords are the transport keys for that map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (platform === "windows-linux" && onlyCtrl) {
    //   if (key === "ArrowLeft") return "previous";
    //   if (key === "ArrowRight") return "next";
    // }
    // ```
    if (request.platform == KeyboardPlatform.WINDOWS_LINUX && onlyCtrl(request)) {
        if (request.key == "ArrowLeft") {
            return ShortcutAction.PREVIOUS
        }
        if (request.key == "ArrowRight") {
            return ShortcutAction.NEXT
        }
    }

    // What:     On macOS, Command plus Shift plus a bracket moves to the previous or next track.
    // Why:      These chords follow the IDE previous and next tab convention in the accepted map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (platform === "mac" && onlyMetaShift) {
    //   if (key === "[" || key === "{") return "previous";
    //   if (key === "]" || key === "}") return "next";
    // }
    // ```
    if (request.platform == KeyboardPlatform.MAC && onlyMetaShift(request)) {
        if (request.key == "[" || request.key == "{") {
            return ShortcutAction.PREVIOUS
        }
        if (request.key == "]" || request.key == "}") {
            return ShortcutAction.NEXT
        }
    }

    // What:     `return null` means that no player key rule matched.
    // Why:      The caller falls back to the unbound action.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return null;
    // ```
    return null
}

// What:     `private fun isPrimaryLetter(request: ShortcutRequest, letter: String): Boolean` is true when
//           the primary key is held with no other modifier, and the key is the given letter.
// Why:      Search, the folder picker, and reveal all use this same combination with different letters.
//
// In TS you'd write (pseudocode):
// ```ts
// const isPrimaryLetter = (request: ShortcutRequest, letter: string) =>
//   isPlainPrimary(request) && request.key.toLowerCase() === letter;
// ```
/** Reports whether the primary key plus the given letter is pressed with no other modifier. */
private fun isPrimaryLetter(request: ShortcutRequest, letter: String): Boolean {
    // What:     `isPlainPrimary(request) && hasLetterKey(request, letter)` joins two checks with `&&`.
    // Why:      The primary check and the letter check must both pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return isPlainPrimary(request) && hasLetterKey(request, letter);
    // ```
    return isPlainPrimary(request) && hasLetterKey(request, letter)
}

// What:     `private fun isPlainPrimary(request: ShortcutRequest): Boolean` is true when the primary
//           modifier is held, the other one is not, and neither Alt nor Shift is held.
// Why:      Every letter shortcut needs exactly this modifier combination.
//
// In TS you'd write (pseudocode):
// ```ts
// const isPlainPrimary = (request: ShortcutRequest) =>
//   primary && !otherPrimary && !request.altKey && !request.shiftKey;
// ```
/** Reports whether only the primary modifier is held among the modifiers that matter for letters. */
private fun isPlainPrimary(request: ShortcutRequest): Boolean {
    // What:     `val primary: Boolean` holds whether this map's primary modifier is held.
    // Why:      The letter shortcuts need to know whether the primary modifier is pressed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const primary = request.platform === "mac" ? request.metaKey : request.ctrlKey;
    // ```
    /** Whether this keyboard map's primary modifier is held. */
    val primary: Boolean = primaryPressed(request)

    // What:     `val otherPrimary: Boolean` holds whether the modifier that is not primary is held.
    // Why:      A letter shortcut must not also have that other modifier pressed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const otherPrimary = request.platform === "mac" ? request.ctrlKey : request.metaKey;
    // ```
    /** Whether the non-primary modifier of this keyboard map is held. */
    val otherPrimary: Boolean = otherPrimaryPressed(request)

    // What:     `primary && !otherPrimary && noAltOrShift(request)` combines three checks with `&&`.
    // Why:      All three must hold for the plain primary combination.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return primary && !otherPrimary && !request.altKey && !request.shiftKey;
    // ```
    return primary && !otherPrimary && noAltOrShift(request)
}

// What:     `private fun hasLetterKey(request: ShortcutRequest, letter: String): Boolean` compares the
//           key text, ignoring letter case, with a lowercase letter.
// Why:      A shortcut must match the letter whether or not Shift changed its case.
//
// Gotcha:   `lowercase()` uses fixed rules in every locale, the same as JavaScript's `toLowerCase()`.
//
// In TS you'd write (pseudocode):
// ```ts
// const hasLetterKey = (request: ShortcutRequest, letter: string) => request.key.toLowerCase() === letter;
// ```
/** Reports whether the pressed key is the given lowercase letter, ignoring case. */
private fun hasLetterKey(request: ShortcutRequest, letter: String): Boolean {
    // What:     `val lowerKey: String` is the key text converted to lowercase.
    // Why:      The comparison below then ignores the case of the pressed letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lowerKey = request.key.toLowerCase();
    // ```
    /** The pressed key text in lowercase. */
    val lowerKey: String = request.key.lowercase()

    // What:     `lowerKey == letter` compares two strings by their text.
    // Why:      It is true only when the pressed key is the given letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return lowerKey === letter;
    // ```
    return lowerKey == letter
}

// What:     `private fun noModifiers(request: ShortcutRequest): Boolean` is true when Control, Command
//           or Meta, Alt, and Shift are all released.
// Why:      Plain keys such as Space must not fire while any modifier is held.
//
// In TS you'd write (pseudocode):
// ```ts
// const noModifiers = (r: ShortcutRequest) => !r.ctrlKey && !r.metaKey && !r.altKey && !r.shiftKey;
// ```
/** Reports whether no modifier key is held. */
private fun noModifiers(request: ShortcutRequest): Boolean {
    // What:     `!request.ctrl && !request.meta && noAltOrShift(request)` checks three conditions with `&&`.
    // Why:      Control and Meta are checked here, and Alt and Shift in the helper.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return !r.ctrlKey && !r.metaKey && !r.altKey && !r.shiftKey;
    // ```
    return !request.ctrl && !request.meta && noAltOrShift(request)
}

// What:     `private fun onlyCtrl(request: ShortcutRequest): Boolean` is true when Control is held and
//           Meta, Alt, and Shift are released.
// Why:      Control-only chords, such as Control plus M and the Windows and Linux track arrows, need this.
//
// In TS you'd write (pseudocode):
// ```ts
// const onlyCtrl = (r: ShortcutRequest) => r.ctrlKey && !r.metaKey && !r.altKey && !r.shiftKey;
// ```
/** Reports whether Control is the only modifier held. */
private fun onlyCtrl(request: ShortcutRequest): Boolean {
    // What:     `request.ctrl && !request.meta && noAltOrShift(request)` checks three conditions with `&&`.
    // Why:      Control must be held, and Meta, Alt, and Shift must be released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return r.ctrlKey && !r.metaKey && !r.altKey && !r.shiftKey;
    // ```
    return request.ctrl && !request.meta && noAltOrShift(request)
}

// What:     `private fun onlyMeta(request: ShortcutRequest): Boolean` is true when Meta (Command) is held
//           and Control, Alt, and Shift are released.
// Why:      The macOS Settings chord is Command with no other modifier.
//
// In TS you'd write (pseudocode):
// ```ts
// const onlyMeta = (r: ShortcutRequest) => r.metaKey && !r.ctrlKey && !r.altKey && !r.shiftKey;
// ```
/** Reports whether Meta is the only modifier held. */
private fun onlyMeta(request: ShortcutRequest): Boolean {
    // What:     `request.meta && !request.ctrl && noAltOrShift(request)` checks three conditions with `&&`.
    // Why:      Meta must be held, and Control, Alt, and Shift must be released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return r.metaKey && !r.ctrlKey && !r.altKey && !r.shiftKey;
    // ```
    return request.meta && !request.ctrl && noAltOrShift(request)
}

// What:     `private fun onlyCtrlAlt(request: ShortcutRequest): Boolean` is true when Control and Alt are
//           both held and Meta and Shift are released.
// Why:      The Windows and Linux Settings chord is Control plus Alt with no other modifier.
//
// In TS you'd write (pseudocode):
// ```ts
// const onlyCtrlAlt = (r: ShortcutRequest) => r.ctrlKey && r.altKey && !r.metaKey && !r.shiftKey;
// ```
/** Reports whether exactly Control and Alt are held among the four modifiers. */
private fun onlyCtrlAlt(request: ShortcutRequest): Boolean {
    // What:     `request.ctrl && request.alt && noMetaOrShift(request)` checks three conditions with `&&`.
    // Why:      Control and Alt must be held, and Meta and Shift must be released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return r.ctrlKey && r.altKey && !r.metaKey && !r.shiftKey;
    // ```
    return request.ctrl && request.alt && noMetaOrShift(request)
}

// What:     `private fun onlyMetaShift(request: ShortcutRequest): Boolean` is true when Meta and Shift are
//           both held and Control and Alt are released.
// Why:      The macOS track-change chords are Command plus Shift with no other modifier.
//
// In TS you'd write (pseudocode):
// ```ts
// const onlyMetaShift = (r: ShortcutRequest) => r.metaKey && r.shiftKey && !r.ctrlKey && !r.altKey;
// ```
/** Reports whether exactly Meta and Shift are held among the four modifiers. */
private fun onlyMetaShift(request: ShortcutRequest): Boolean {
    // What:     `request.meta && request.shift && noCtrlOrAlt(request)` checks three conditions with `&&`.
    // Why:      Meta and Shift must be held, and Control and Alt must be released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return r.metaKey && r.shiftKey && !r.ctrlKey && !r.altKey;
    // ```
    return request.meta && request.shift && noCtrlOrAlt(request)
}

// What:     `private fun noAltOrShift(request: ShortcutRequest): Boolean` is true when Alt and Shift are
//           both released.
// Why:      Most shortcut rules require Alt and Shift to be off, so the check has a single name.
//
// In TS you'd write (pseudocode):
// ```ts
// const noAltOrShift = (r: ShortcutRequest) => !r.altKey && !r.shiftKey;
// ```
/** Reports whether Alt and Shift are both released. */
private fun noAltOrShift(request: ShortcutRequest): Boolean {
    // What:     `!request.alt && !request.shift` checks two negated fields with `&&`.
    // Why:      Both modifiers must be released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return !r.altKey && !r.shiftKey;
    // ```
    return !request.alt && !request.shift
}

// What:     `private fun noMetaOrShift(request: ShortcutRequest): Boolean` is true when Meta and Shift are
//           both released.
// Why:      The Control-plus-Alt chord must not also have Meta or Shift held.
//
// In TS you'd write (pseudocode):
// ```ts
// const noMetaOrShift = (r: ShortcutRequest) => !r.metaKey && !r.shiftKey;
// ```
/** Reports whether Meta and Shift are both released. */
private fun noMetaOrShift(request: ShortcutRequest): Boolean {
    // What:     `!request.meta && !request.shift` checks two negated fields with `&&`.
    // Why:      Both modifiers must be released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return !r.metaKey && !r.shiftKey;
    // ```
    return !request.meta && !request.shift
}

// What:     `private fun noCtrlOrAlt(request: ShortcutRequest): Boolean` is true when Control and Alt are
//           both released.
// Why:      The macOS bracket chords must not also have Control or Alt held.
//
// In TS you'd write (pseudocode):
// ```ts
// const noCtrlOrAlt = (r: ShortcutRequest) => !r.ctrlKey && !r.altKey;
// ```
/** Reports whether Control and Alt are both released. */
private fun noCtrlOrAlt(request: ShortcutRequest): Boolean {
    // What:     `!request.ctrl && !request.alt` checks two negated fields with `&&`.
    // Why:      Both modifiers must be released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return !r.ctrlKey && !r.altKey;
    // ```
    return !request.ctrl && !request.alt
}

// What:     `private fun primaryPressed(request: ShortcutRequest): Boolean` reads the primary modifier.
// Why:      Control is primary on Windows and Linux, and Command is primary on macOS.
//
// In TS you'd write (pseudocode):
// ```ts
// const primaryPressed = (request: ShortcutRequest) => request.platform === "mac" ? request.meta : request.ctrl;
// ```
/** Reports whether this platform's primary modifier is held. */
private fun primaryPressed(request: ShortcutRequest): Boolean {
    // What:     `when (request.platform) { ... }` picks one branch per platform and lists both entries.
    // Why:      Listing every entry makes the compiler reject a future platform that has no branch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (request.platform) { case "windows-linux": ...; case "mac": ...; }
    // ```
    return when (request.platform) {
        KeyboardPlatform.WINDOWS_LINUX -> request.ctrl
        KeyboardPlatform.MAC -> request.meta
    }
}

// What:     `private fun otherPrimaryPressed(request: ShortcutRequest): Boolean` reads the non-primary modifier.
// Why:      Each shortcut checks that the other modifier is not also held.
//
// In TS you'd write (pseudocode):
// ```ts
// const otherPrimaryPressed = (request: ShortcutRequest) => request.platform === "mac" ? request.ctrl : request.meta;
// ```
/** Reports whether the modifier that is not primary on this platform is held. */
private fun otherPrimaryPressed(request: ShortcutRequest): Boolean {
    // What:     `when` lists both platforms again, this time for the non-primary modifier.
    // Why:      The same exhaustive shape keeps both helpers aligned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (request.platform) { case "windows-linux": ...; case "mac": ...; }
    // ```
    return when (request.platform) {
        KeyboardPlatform.WINDOWS_LINUX -> request.meta
        KeyboardPlatform.MAC -> request.ctrl
    }
}

// What:     `private fun revealKeyFor(platform: KeyboardPlatform): String` returns the reveal letter.
// Why:      Reveal uses G on Windows and Linux and L on macOS, and the classifier compares against it.
//
// In TS you'd write (pseudocode):
// ```ts
// const revealKeyFor = (platform: KeyboardPlatform): string => platform === "mac" ? "l" : "g";
// ```
/** Returns the lowercase letter that triggers reveal on this platform. */
private fun revealKeyFor(platform: KeyboardPlatform): String {
    // What:     `when (platform) { ... }` gives one letter for each platform.
    // Why:      Both entries are listed so every platform has an answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (platform) { case "windows-linux": return "g"; case "mac": return "l"; }
    // ```
    return when (platform) {
        KeyboardPlatform.WINDOWS_LINUX -> "g"
        KeyboardPlatform.MAC -> "l"
    }
}
