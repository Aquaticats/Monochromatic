// The one rule for activating a track: a tap, Enter or a Search result on a track row (D124, D127, D129).

// What:     `package ...core` places this pure rule beside the other platform-independent rules.
// Why:      The rule takes two booleans, so host JVM tests exercise it without Android framework classes.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `enum class RowActivation` names the three things activating a track can do.
// Why:      The controller maps each value to one engine call, so a row tap, Enter and a Search result cannot differ.
//
// In TS you'd write (pseudocode):
// ```ts
// type RowActivation = "selectOnly" | "selectAndPlay" | "togglePlay";
// ```
/** What activating a track does to the player. */
enum class RowActivation {
    /** The track becomes current and loads paused at its start; whatever played stops (D124). */
    SELECT_ONLY,

    /** The track becomes current and starts playing at once (D129 with Auto-play on select on). */
    SELECT_AND_PLAY,

    /** The track is already current: it plays when paused and pauses when playing (D124). */
    TOGGLE_PLAY,
}

// What:     `fun rowActivationFor(autoPlayOnSelect: Boolean, isCurrent: Boolean): RowActivation` decides the action.
// Why:      The current track always toggles; any other track is only selected unless Auto-play on select is on, in
//           which case it also starts (the behavior before D124). Folder and letter rails do not use this rule.
//
// In TS you'd write (pseudocode):
// ```ts
// const rowActivationFor = (autoPlayOnSelect: boolean, isCurrent: boolean): RowActivation =>
//   isCurrent ? "togglePlay" : autoPlayOnSelect ? "selectAndPlay" : "selectOnly";
// ```
/** Decides what activating one track does, from the Auto-play on select setting and whether the track is current. */
fun rowActivationFor(autoPlayOnSelect: Boolean, isCurrent: Boolean): RowActivation {
    if (isCurrent) {
        return RowActivation.TOGGLE_PLAY
    }
    if (autoPlayOnSelect) {
        return RowActivation.SELECT_AND_PLAY
    }
    return RowActivation.SELECT_ONLY
}

// What:     `fun autoPlayOnSelectSupportingText(enabled: Boolean): String` is the line under the setting's title.
// Why:      The line states what the current choice does, so the text is a label of the state besides the switch
//           itself (D129). Both texts are short enough for one line on the 443 dp cover.
//
// In TS you'd write (pseudocode):
// ```ts
// const autoPlayOnSelectSupportingText = (enabled: boolean): string =>
//   enabled ? "Select and play at once" : "Select first, then Play";
// ```
/** Returns the one-line description of what selecting a track does with Auto-play on select [enabled] or not. */
fun autoPlayOnSelectSupportingText(enabled: Boolean): String =
    if (enabled) "Select and play at once" else "Select first, then Play"
