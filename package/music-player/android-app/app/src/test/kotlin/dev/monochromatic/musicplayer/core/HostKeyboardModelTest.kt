// Host keyboard glue cases for decision D102. Each case names the owner, key class, or boundary of the accepted map
// that it checks, so a failure points back to the planning record and the boundary evidence.

// What:     `package dev.monochromatic.musicplayer.core` places the tests beside the glue they call.
// Why:      The tests call the glue and its types directly, with no package prefix.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import org.junit.Assert.assertEquals` and `org.junit.Test` bring in the value check and the test marker.
// Why:      Every case compares one command or owner by exact value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `private class HostKeyRecordingActions` records each host operation in call order.
// Why:      The dispatcher cases check which operation ran, and in what order, without any Compose state.
//
// In TS you'd write (pseudocode):
// ```ts
// const recording: string[] = [];
// ```
/** Host operations that record their names so the dispatcher cases can read the call order. */
private class HostKeyRecordingActions : HostKeyActions {
    // What:     `private val calls: MutableList<String>` holds the names of the operations in call order.
    // Why:      Each override appends its name here, and the cases read the list back.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private calls: string[] = [];
    // ```
    /** Operation names in the order they were called. */
    private val calls: MutableList<String> = mutableListOf()

    // What:     `fun recorded(): List<String>` returns a copy of the call names so far.
    // Why:      A copy keeps the cases from reading a list that a later call would change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // recorded(): string[] { return [...this.calls]; }
    // ```
    /** Returns the operation names recorded so far, in call order. */
    fun recorded(): List<String> = calls.toList()

    // What:     `override fun togglePlayback()` records the toggle operation.
    // Why:      It proves Space reaches the toggle operation exactly once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // togglePlayback(): void { this.calls.push("toggle"); }
    // ```
    /** Records the play or pause operation. */
    override fun togglePlayback() {
        calls.add("toggle")
    }

    // What:     `override fun previousTrack()` records the previous operation.
    // Why:      It proves the backward chord reaches only the previous operation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // previousTrack(): void { this.calls.push("previous"); }
    // ```
    /** Records the previous-track operation. */
    override fun previousTrack() {
        calls.add("previous")
    }

    // What:     `override fun nextTrack()` records the next operation.
    // Why:      It proves the forward chord reaches only the next operation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // nextTrack(): void { this.calls.push("next"); }
    // ```
    /** Records the next-track operation. */
    override fun nextTrack() {
        calls.add("next")
    }

    // What:     `override fun cycleMode(mode: PlaybackMode)` records the mode that it was given.
    // Why:      It proves the cycle passes the computed mode through unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // cycleMode(mode: PlaybackMode): void { this.calls.push(`mode:${mode.storedName}`); }
    // ```
    /** Records the end-of-track mode that was applied. */
    override fun cycleMode(mode: PlaybackMode) {
        calls.add("mode:" + mode.name)
    }

    // What:     `override fun openSearch()` records the new-visit Search operation.
    // Why:      It proves a new visit is distinct from a same-visit refocus.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // openSearch(): void { this.calls.push("open-search"); }
    // ```
    /** Records the Search operation that starts a new visit. */
    override fun openSearch() {
        calls.add("open-search")
    }

    // What:     `override fun focusSearch()` records the same-visit Search focus operation.
    // Why:      It proves a repeated Search shortcut refocuses without starting a visit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // focusSearch(): void { this.calls.push("focus-search"); }
    // ```
    /** Records the Search refocus operation. */
    override fun focusSearch() {
        calls.add("focus-search")
    }

    // What:     `override fun toggleFolderPicker()` records the picker operation.
    // Why:      It proves Control plus O reaches the picker toggle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // toggleFolderPicker(): void { this.calls.push("picker"); }
    // ```
    /** Records the folder picker toggle. */
    override fun toggleFolderPicker() {
        calls.add("picker")
    }

    // What:     `override fun openSettings()` records the Settings operation.
    // Why:      It proves the Settings chord reaches the Settings operation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // openSettings(): void { this.calls.push("settings"); }
    // ```
    /** Records the Settings operation. */
    override fun openSettings() {
        calls.add("settings")
    }

    // What:     `override fun revealCurrent()` records the reveal operation.
    // Why:      It proves reveal runs only its own operation and no transport operation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // revealCurrent(): void { this.calls.push("reveal"); }
    // ```
    /** Records the reveal-current operation. */
    override fun revealCurrent() {
        calls.add("reveal")
    }

    // What:     `override fun backFromSearch()` records the Back operation.
    // Why:      It proves Escape in Search leaves through the Back operation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // backFromSearch(): void { this.calls.push("back"); }
    // ```
    /** Records the operation that leaves Search by the Back rule. */
    override fun backFromSearch() {
        calls.add("back")
    }

    // What:     `override fun closePopup()` records the popup close operation.
    // Why:      It proves Escape with an open menu closes that menu.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // closePopup(): void { this.calls.push("close-popup"); }
    // ```
    /** Records the operation that closes the open track menu. */
    override fun closePopup() {
        calls.add("close-popup")
    }

    // What:     The overrides below record the D106 operations by name, with their argument where they take one.
    // Why:      `KeyBindingsTest` and the dispatcher cases read the names back to prove which operation ran.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // seekBy(s: number): void { this.calls.push(`seek:${s}`); }
    // ```
    /** Records the seek operation with its step in seconds. */
    override fun seekBy(seconds: Int) {
        calls.add("seek:$seconds")
    }

    /** Records the restart operation. */
    override fun restartTrack() {
        calls.add("restart")
    }

    /** Records the folder step with its direction. */
    override fun stepFolder(step: Int) {
        calls.add("folder:$step")
    }

    /** Records the reveal of the playing track's folder. */
    override fun revealPlayingTrackFolder() {
        calls.add("reveal-folder")
    }

    /** Records the focus request with its zone. */
    override fun focusZone(zone: HostKeyZone) {
        calls.add("focus:${zone.name}")
    }

    /** Records the system folder chooser. */
    override fun openFolderDialog() {
        calls.add("folder-dialog")
    }

    /** Records the picker close. */
    override fun closePicker() {
        calls.add("close-picker")
    }
}

// What:     `private fun hostKeyPlayerContext(): HostKeyContext` is the player with its root focused and a playing row
//           shown.
// Why:      Most transport cases start from this state, and each case changes only the field it tests.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyPlayerContext = (): HostKeyContext => ({ route: "player", rootFocused: true, ... });
// ```
/** The player screen with the playback area focused, on the folded cover, and in the in-order mode. */
private fun hostKeyPlayerContext(): HostKeyContext = HostKeyContext(
    route = HostKeyRoute.PLAYER,
    menuOpen = false,
    textFieldFocused = false,
    rootFocused = true,
    playbackMode = PlaybackMode.IN_ORDER,
    layout = HostKeyLayout.FOLDED_COVER,
)

// What:     `private fun hostKeySearchContext(): HostKeyContext` is the Search page with focus on a result, not the
//           field.
// Why:      Cases that need a text field or a result focus start from this state and change one field.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeySearchContext = (): HostKeyContext => ({ route: "search", rootFocused: false, textFieldFocused: false
//           });
// ```
/** The Search page with no text field focused and the playback area not focused. */
private fun hostKeySearchContext(): HostKeyContext = hostKeyPlayerContext().copy(
    route = HostKeyRoute.SEARCH,
    rootFocused = false,
)

// What:     `private fun hostKeyEditorContext(): HostKeyContext` is the template editor with its text field focused.
// Why:      The editor cases prove that its text field keeps the keys it types with.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyEditorContext = (): HostKeyContext => ({ route: "editor", textFieldFocused: true, rootFocused: false });
// ```
/** The template editor with its text field focused. */
private fun hostKeyEditorContext(): HostKeyContext = hostKeyPlayerContext().copy(
    route = HostKeyRoute.EDITOR,
    textFieldFocused = true,
    rootFocused = false,
)

// What:     `private fun hostKeyPlainInput(name: String): HostKeyInput` is one key-down with no modifiers and no
//           repeat.
// Why:      Each case starts from a plain key and adds only the modifier or repeat flag that it tests.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyPlainInput = (keyName: string): HostKeyInput => ({ keyName, ctrl: false, meta: false, alt: false,
//           ...all false });
// ```
/** One plain key-down with the given key text and every modifier released. */
private fun hostKeyPlainInput(name: String): HostKeyInput = HostKeyInput(
    keyName = name,
    ctrl = false,
    meta = false,
    alt = false,
    shift = false,
    repeat = false,
)

// What:     `class HostKeyboardModelTest` groups the cases for the host keyboard glue.
// Why:      JUnit runs every `@Test` method inside this class.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("HostKeyboard", () => { /* cases */ });
// ```
/** Cases for the owner, key class, repeat, popup, Search, picker, Settings, reveal, and dispatch rules. */
class HostKeyboardModelTest {
    // What:     `fun ownerIsEditorWhenTextFieldHasFocus()` checks the editor owner.
    // Why:      A focused text field must keep its keys, so its owner is the editor.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(ownerOf({ textFieldFocused: true })).toBe("editor");
    // ```
    /** A focused text field makes the editor the owner, even on the player route. */
    @Test
    fun ownerIsEditorWhenTextFieldHasFocus() {
        assertEquals(ShortcutOwner.EDITOR, hostKeyOwnerOf(hostKeyPlayerContext().copy(textFieldFocused = true)))
    }

    // What:     `fun ownerIsPlayerOnlyOnPlayerRouteWithRootFocus()` checks the player owner and its route gate.
    // Why:      The transport keys belong to the playback area, and only on the player route.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(ownerOf({ route: "player", rootFocused: true })).toBe("player");
    // expect(ownerOf({ route: "settings", rootFocused: true })).toBe("control");
    // ```
    /** The playback area owns transport keys on the player route, and not on the other routes. */
    @Test
    fun ownerIsPlayerOnlyOnPlayerRouteWithRootFocus() {
        assertEquals(ShortcutOwner.PLAYER, hostKeyOwnerOf(hostKeyPlayerContext()))
        assertEquals(ShortcutOwner.CONTROL, hostKeyOwnerOf(hostKeyPlayerContext().copy(route = HostKeyRoute.SETTINGS)))
    }

    // What:     `fun ownerIsControlWhenRootIsNotFocused()` checks the control owner.
    // Why:      A focused button or row keeps Space and arrows, so its owner is a control.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(ownerOf({ route: "player", rootFocused: false })).toBe("control");
    // ```
    /** A focused control, with the playback area not focused, makes the control the owner. */
    @Test
    fun ownerIsControlWhenRootIsNotFocused() {
        assertEquals(ShortcutOwner.CONTROL, hostKeyOwnerOf(hostKeyPlayerContext().copy(rootFocused = false)))
    }

    // What:     `fun spaceTogglesPlaybackOnPlayer()` checks that Space plays or pauses the player.
    // Why:      The map gives Space to the playback area, so it toggles only when that area has focus.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor(" ", { owner: "player" })).toBe("toggle");
    // ```
    /** Space toggles playback while the playback area has focus. */
    @Test
    fun spaceTogglesPlaybackOnPlayer() {
        assertEquals(
            HostKeyCommand.TogglePlayback,
            hostKeyCommandFor(HostKeyPress(key = hostKeyPlainInput(" "), context = hostKeyPlayerContext())),
        )
    }

    // What:     `fun spaceIsNotConsumedOnControl()` checks that Space stays with a focused control.
    // Why:      A focused button keeps Space, and the press must not also toggle playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor(" ", { owner: "control" })).toBe("unhandled");
    // ```
    /** Space on a focused control is not consumed, so the control keeps it. */
    @Test
    fun spaceIsNotConsumedOnControl() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput(" "),
            context = hostKeyPlayerContext().copy(rootFocused = false),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun spaceIsNotConsumedInEditor()` checks that Space types in a text field.
    // Why:      Typing a space must not also start or pause playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor(" ", { owner: "editor" })).toBe("unhandled");
    // ```
    /** Space in a focused text field is not consumed, so the text keeps the space. */
    @Test
    fun spaceIsNotConsumedInEditor() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput(" "), context = hostKeyEditorContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlMCyclesModeOnPlayer()` checks that Control plus M moves to the next end-of-track mode.
    // Why:      The mode cycle is a product exception that the map gives to the playback area on both maps.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+M", { mode: "in_order" })).toEqual({ mode: "shuffle_page" });
    // ```
    /** Control plus M cycles the end-of-track mode to the one after the current mode. */
    @Test
    fun ctrlMCyclesModeOnPlayer() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("m").copy(ctrl = true), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.CycleMode(PlaybackMode.SHUFFLE_PAGE), hostKeyCommandFor(press))
    }

    // What:     `fun ctrlMIsNotConsumedInEditor()` checks that Control plus M is left to the text field.
    // Why:      The editor keeps its keys, and the mode cycle must not run while text is being edited.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+M", { owner: "editor" })).toBe("unhandled");
    // ```
    /** Control plus M in a focused text field is not consumed. */
    @Test
    fun ctrlMIsNotConsumedInEditor() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("m").copy(ctrl = true), context = hostKeyEditorContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlLeftPreviousOnPlayer()` checks that Control plus Left moves to the previous track.
    // Why:      It is the backward transport chord of the Windows and Linux map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+ArrowLeft", { owner: "player" })).toBe("previous");
    // ```
    /** Control plus Left moves to the previous track while the playback area has focus. */
    @Test
    fun ctrlLeftPreviousOnPlayer() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("ArrowLeft").copy(ctrl = true),
            context = hostKeyPlayerContext(),
        )
        assertEquals(HostKeyCommand.PreviousTrack, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlRightNextOnPlayer()` checks that Control plus Right moves to the next track.
    // Why:      It is the forward transport chord of the Windows and Linux map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+ArrowRight", { owner: "player" })).toBe("next");
    // ```
    /** Control plus Right moves to the next track while the playback area has focus. */
    @Test
    fun ctrlRightNextOnPlayer() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("ArrowRight").copy(ctrl = true),
            context = hostKeyPlayerContext(),
        )
        assertEquals(HostKeyCommand.NextTrack, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlLeftIsNotConsumedInEditor()` checks that Control plus Left keeps its word movement in a field.
    // Why:      The map says editing fields keep their own keys, so Control plus Left must not change the track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+ArrowLeft", { owner: "editor" })).toBe("unhandled");
    // ```
    /** Control plus Left in a focused text field is not consumed, so the field moves its caret. */
    @Test
    fun ctrlLeftIsNotConsumedInEditor() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("ArrowLeft").copy(ctrl = true),
            context = hostKeyEditorContext(),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlRightIsNotConsumedOnControl()` checks that Control plus Right stays with a focused control.
    // Why:      A focused control keeps its keys, so the transport chord must not fire from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+ArrowRight", { owner: "control" })).toBe("unhandled");
    // ```
    /** Control plus Right with a focused control is not consumed. */
    @Test
    fun ctrlRightIsNotConsumedOnControl() {
        /** Host state under test, built from a fixture with the one field this case changes. */
        val context = hostKeyPlayerContext().copy(rootFocused = false)
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("ArrowRight").copy(ctrl = true), context = context)
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun bareArrowLeftIsNotConsumedOnPlayer()` checks that a bare arrow keeps its native meaning.
    // Why:      The map gives bare arrows no seek or volume, so the host must not consume them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("ArrowLeft", { owner: "player" })).toBe("unhandled");
    // ```
    /** A bare Left arrow on the playback area is not consumed, because the map gives it no meaning. */
    @Test
    fun bareArrowLeftIsNotConsumedOnPlayer() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("ArrowLeft"), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun macChordsAreNotConsumedOnAndroid()` checks that the macOS track chords are not taken.
    // Why:      The host reads the Windows and Linux map only, so Meta plus Shift plus a bracket stays with the system.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Meta+Shift+]", { platform: "windows-linux" })).toBe("unhandled");
    // ```
    /** Meta plus Shift plus the right bracket is not consumed on the Windows and Linux map. */
    @Test
    fun macChordsAreNotConsumedOnAndroid() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("]").copy(meta = true, shift = true),
            context = hostKeyPlayerContext(),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlShiftFDoesNothing()` checks that Shift breaks the Search chord.
    // Why:      The classifier requires exactly the primary modifier, so Control plus Shift plus F is not Search.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+Shift+F", { owner: "player" })).toBe("unhandled");
    // ```
    /** Control plus Shift plus F is not consumed, because Shift changes the chord. */
    @Test
    fun ctrlShiftFDoesNothing() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("f").copy(ctrl = true, shift = true),
            context = hostKeyPlayerContext(),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun metaFIsRejected()` checks that a held Meta key rejects the Control chords.
    // Why:      The classifier rejects Meta on the Windows and Linux letter chords, so Meta plus F does nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Meta+f", { platform: "windows-linux" })).toBe("unhandled");
    // ```
    /** Meta plus F is not consumed on the Windows and Linux map. */
    @Test
    fun metaFIsRejected() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("f").copy(meta = true), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun repeatSpaceIsSwallowedOnPlayer()` checks that a held Space toggles playback once.
    // Why:      Auto-repeat must not toggle playback many times for one press.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor(" ", { repeat: true, owner: "player" })).toBe("swallow-repeat");
    // ```
    /** A repeated Space on the playback area is swallowed, so it runs nothing. */
    @Test
    fun repeatSpaceIsSwallowedOnPlayer() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput(" ").copy(repeat = true), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.SwallowRepeat, hostKeyCommandFor(press))
    }

    // What:     `fun repeatSpaceIsNotConsumedOnControl()` checks that a repeat stays with a focused control.
    // Why:      A focused button keeps its auto-repeat behavior, and the host must not swallow it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor(" ", { repeat: true, owner: "control" })).toBe("unhandled");
    // ```
    /** A repeated Space on a focused control is not consumed. */
    @Test
    fun repeatSpaceIsNotConsumedOnControl() {
        /** Host state under test, built from a fixture with the one field this case changes. */
        val context = hostKeyPlayerContext().copy(rootFocused = false)
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput(" ").copy(repeat = true), context = context)
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun repeatCtrlOIsSwallowedAndOpensNothing()` checks that a held picker chord toggles once.
    // Why:      The classifier ignores repeats only for player keys, so the host swallows repeats of its own commands.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+o", { repeat: true })).toBe("swallow-repeat");
    // ```
    /** A repeated Control plus O is swallowed, so the picker does not toggle on every repeat. */
    @Test
    fun repeatCtrlOIsSwallowedAndOpensNothing() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("o").copy(ctrl = true, repeat = true),
            context = hostKeyPlayerContext(),
        )
        assertEquals(HostKeyCommand.SwallowRepeat, hostKeyCommandFor(press))
    }

    // What:     `fun repeatCtrlMInEditorPassesThrough()` checks that a repeated chord in a field reaches the field.
    // Why:      Holding a key in a text field must repeat its text, so the host must not consume the repeat.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+m", { repeat: true, owner: "editor" })).toBe("unhandled");
    // ```
    /** A repeated Control plus M in a text field is not consumed. */
    @Test
    fun repeatCtrlMInEditorPassesThrough() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("m").copy(ctrl = true, repeat = true),
            context = hostKeyEditorContext(),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun repeatOfUnmappedKeyPassesThrough()` checks that a held key no shortcut uses is never swallowed.
    // Why:      An unmapped key must reach its owner even while it repeats, so arrows and letters keep working.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("", { repeat: true, owner: "player" })).toBe("unhandled");
    // ```
    /** A repeated key with no shortcut text is not consumed. */
    @Test
    fun repeatOfUnmappedKeyPassesThrough() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("").copy(repeat = true), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun typedLetterPassesThroughInEditorAndOnPlayer()` checks that an ordinary letter is never a command.
    // Why:      Letters belong to the focused text field, and on the player they are not bound to anything.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("a", { owner: "editor" })).toBe("unhandled");
    // expect(commandFor("a", { owner: "player" })).toBe("unhandled");
    // ```
    /** An unmapped letter is not consumed in a text field or on the player. */
    @Test
    fun typedLetterPassesThroughInEditorAndOnPlayer() {
        /** Key press that arrives while a text field holds focus. */
        val inEditor = HostKeyPress(key = hostKeyPlainInput("a"), context = hostKeyEditorContext())
        /** Key press that arrives while the playback area holds focus. */
        val onPlayer = HostKeyPress(key = hostKeyPlainInput("a"), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(inEditor))
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(onPlayer))
    }

    // What:     `fun escapeClosesOpenPopup()` checks that Escape closes the innermost popup first.
    // Why:      The map gives Escape to the popup before any other meaning.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Escape", { menuOpen: true })).toBe("close-popup");
    // ```
    /** Escape with a track menu open closes that menu. */
    @Test
    fun escapeClosesOpenPopup() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("Escape"),
            context = hostKeyPlayerContext().copy(menuOpen = true),
        )
        assertEquals(HostKeyCommand.ClosePopup, hostKeyCommandFor(press))
    }

    // What:     `fun spaceIsNotConsumedWhilePopupIsOpen()` checks that an open popup keeps every other key.
    // Why:      A popup must take every key, so no transport chord can change playback behind it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor(" ", { menuOpen: true })).toBe("unhandled");
    // ```
    /** Space with a track menu open is not consumed, so no playback change happens behind the popup. */
    @Test
    fun spaceIsNotConsumedWhilePopupIsOpen() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput(" "), context = hostKeyPlayerContext().copy(menuOpen = true))
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlFIsNotConsumedWhilePopupIsOpen()` checks that Search does not open from behind a popup.
    // Why:      The popup owns every key, so the Search chord must wait for the popup to close.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+f", { menuOpen: true })).toBe("unhandled");
    // ```
    /** Control plus F with a track menu open is not consumed. */
    @Test
    fun ctrlFIsNotConsumedWhilePopupIsOpen() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("f").copy(ctrl = true),
            context = hostKeyPlayerContext().copy(menuOpen = true),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun escapeInSearchWithEditorFocusGoesBack()` checks that Escape leaves Search with the field focused.
    // Why:      Escape leaves Search even though a text field would otherwise keep it, as the map requires.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Escape", { route: "search", owner: "editor" })).toBe("back");
    // ```
    /** Escape in Search with the query field focused leaves Search by the Back rule. */
    @Test
    fun escapeInSearchWithEditorFocusGoesBack() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("Escape"),
            context = hostKeySearchContext().copy(textFieldFocused = true),
        )
        assertEquals(HostKeyCommand.BackFromSearch, hostKeyCommandFor(press))
    }

    // What:     `fun escapeInWideSettingsGoesBack()` checks that Escape closes the two-pane Settings page.
    // Why:      D110 closes the wide Settings page on Escape, even with the editor pane's text field focused.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Escape", { route: "settings", layout: "unfolded", textFieldFocused: true })).toBe("back");
    // ```
    /** Escape on the unfolded Settings route goes back, and on the folded cover it does not. */
    @Test
    fun escapeInWideSettingsGoesBack() {
        /** Unfolded Settings with the editor field focused. */
        val wide: HostKeyContext = hostKeyPlayerContext().copy(
            route = HostKeyRoute.SETTINGS,
            layout = HostKeyLayout.UNFOLDED,
            textFieldFocused = true,
            rootFocused = false,
        )
        assertEquals(
            HostKeyCommand.BackFromSearch,
            hostKeyCommandFor(HostKeyPress(key = hostKeyPlainInput("Escape"), context = wide)),
        )
        assertEquals(
            HostKeyCommand.Unhandled,
            hostKeyCommandFor(
                HostKeyPress(
                    key = hostKeyPlainInput("Escape"),
                    context = wide.copy(layout = HostKeyLayout.FOLDED_COVER),
                ),
            ),
        )
    }

    // What:     `fun escapeInSearchWithControlFocusGoesBack()` checks that Escape leaves Search from a result.
    // Why:      Escape leaves Search from any focus inside it, not only from the query field.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Escape", { route: "search", owner: "control" })).toBe("back");
    // ```
    /** Escape in Search with a focused result leaves Search by the Back rule. */
    @Test
    fun escapeInSearchWithControlFocusGoesBack() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("Escape"), context = hostKeySearchContext())
        assertEquals(HostKeyCommand.BackFromSearch, hostKeyCommandFor(press))
    }

    // What:     `fun escapeOnSettingsIsNotConsumed()` checks that Escape does nothing on Settings.
    // Why:      The map gives Escape only the popup and Search, so Settings keeps the key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Escape", { route: "settings" })).toBe("unhandled");
    // ```
    /** Escape on the Settings page is not consumed. */
    @Test
    fun escapeOnSettingsIsNotConsumed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("Escape"),
            context = hostKeyPlayerContext().copy(route = HostKeyRoute.SETTINGS),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun escapeInEditorIsNotConsumed()` checks that Escape stays with the template editor.
    // Why:      The map gives Escape only the popup and Search, so the editor keeps the key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Escape", { route: "editor", owner: "editor" })).toBe("unhandled");
    // ```
    /** Escape in the template editor is not consumed. */
    @Test
    fun escapeInEditorIsNotConsumed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("Escape"), context = hostKeyEditorContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun escapeOnPlayerIsNotConsumed()` checks that Escape does nothing on the player with no popup.
    // Why:      The map gives Escape no meaning outside a popup and Search.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Escape", { route: "player" })).toBe("unhandled");
    // ```
    /** Escape on the player with no popup open is not consumed. */
    @Test
    fun escapeOnPlayerIsNotConsumed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("Escape"), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun repeatEscapeInSearchIsSwallowed()` checks that a held Escape leaves Search once.
    // Why:      A held Escape must not run the Back rule repeatedly, which would pop more than one destination.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Escape", { repeat: true, route: "search" })).toBe("swallow-repeat");
    // ```
    /** A repeated Escape in Search is swallowed and leaves Search only on the first press. */
    @Test
    fun repeatEscapeInSearchIsSwallowed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("Escape").copy(repeat = true),
            context = hostKeySearchContext(),
        )
        assertEquals(HostKeyCommand.SwallowRepeat, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlFOpensSearchFromPlayer()` checks that Search opens as a new visit from the player.
    // Why:      A new visit starts with an empty query, as D66 requires.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+f", { route: "player" })).toBe("open-search");
    // ```
    /** Control plus F from the player opens Search as a new visit. */
    @Test
    fun ctrlFOpensSearchFromPlayer() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("f").copy(ctrl = true), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.OpenSearch, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlFRefocusesSearchWithoutNewVisit()` checks that a repeated Search chord keeps the query.
    // Why:      D63 and D67 require a same-visit refocus that preserves the query text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+f", { route: "search" })).toBe("focus-search");
    // ```
    /** Control plus F on the Search page refocuses the field and does not start a new visit. */
    @Test
    fun ctrlFRefocusesSearchWithoutNewVisit() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("f").copy(ctrl = true), context = hostKeySearchContext())
        assertEquals(HostKeyCommand.FocusSearch, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlFInSearchFieldRefocusesWithoutNewVisit()` checks the same-visit refocus from the query field.
    // Why:      The field owns text keys, but the Search chord still refocuses the same visit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+f", { route: "search", owner: "editor" })).toBe("focus-search");
    // ```
    /** Control plus F with the query field focused refocuses the field without clearing it. */
    @Test
    fun ctrlFInSearchFieldRefocusesWithoutNewVisit() {
        /** Host state under test, built from a fixture with the one field this case changes. */
        val context = hostKeySearchContext().copy(textFieldFocused = true)
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("f").copy(ctrl = true), context = context)
        assertEquals(HostKeyCommand.FocusSearch, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlFFromEditorIsNotConsumed()` checks that the navigation policy keeps the editor in place.
    // Why:      The planning record leaves unsaved-template handling open, so Search does not leave the editor.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+f", { route: "editor" })).toBe("unhandled");
    // ```
    /** Control plus F in the template editor is not consumed while the navigation policy is off. */
    @Test
    fun ctrlFFromEditorIsNotConsumed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("f").copy(ctrl = true), context = hostKeyEditorContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlFFromSettingsOpensSearch()` checks that Search opens from Settings as a new visit.
    // Why:      Settings is not a text editor, so the Search chord may leave it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+f", { route: "settings" })).toBe("open-search");
    // ```
    /** Control plus F on the Settings page opens Search as a new visit. */
    @Test
    fun ctrlFFromSettingsOpensSearch() {
        /** Host state under test, built from a fixture with the one field this case changes. */
        val context = hostKeyPlayerContext().copy(route = HostKeyRoute.SETTINGS)
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("f").copy(ctrl = true), context = context)
        assertEquals(HostKeyCommand.OpenSearch, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlOTogglesPickerOnPlayer()` checks that the folder picker toggles from the player.
    // Why:      The picker is the cover pane for the current library, and Control plus O toggles it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+o", { route: "player" })).toBe("toggle-picker");
    // ```
    /** Control plus O on the player toggles the folder picker. */
    @Test
    fun ctrlOTogglesPickerOnPlayer() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("o").copy(ctrl = true), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.ToggleFolderPicker, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlOIsNotConsumedOnSearch()` checks that the picker chord does nothing on Search.
    // Why:      The picker is part of the cover layout, so toggling it on Search would change state nobody sees.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+o", { route: "search" })).toBe("unhandled");
    // ```
    /** Control plus O on the Search page is not consumed. */
    @Test
    fun ctrlOIsNotConsumedOnSearch() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("o").copy(ctrl = true), context = hostKeySearchContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlAltSOpensSettingsFromPlayer()` checks the Settings chord from the player.
    // Why:      D102 gives Settings to Control plus Alt plus S on the Windows and Linux map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+Alt+s", { route: "player" })).toBe("open-settings");
    // ```
    /** Control plus Alt plus S on the player opens Settings. */
    @Test
    fun ctrlAltSOpensSettingsFromPlayer() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("s").copy(ctrl = true, alt = true),
            context = hostKeyPlayerContext(),
        )
        assertEquals(HostKeyCommand.OpenSettings, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlAltSOnSettingsIsNotConsumed()` checks that the Settings chord does nothing on Settings.
    // Why:      The page is already open, so the chord must not push another destination.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+Alt+s", { route: "settings" })).toBe("unhandled");
    // ```
    /** Control plus Alt plus S on the Settings page is not consumed. */
    @Test
    fun ctrlAltSOnSettingsIsNotConsumed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("s").copy(ctrl = true, alt = true),
            context = hostKeyPlayerContext().copy(route = HostKeyRoute.SETTINGS),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlAltShiftSIsNotConsumed()` checks that Shift breaks the Settings chord.
    // Why:      The map's Settings chord takes exactly Control and Alt, so the extra Shift is not Settings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+Alt+Shift+s", { route: "player" })).toBe("unhandled");
    // ```
    /** Control plus Alt plus Shift plus S is not consumed. */
    @Test
    fun ctrlAltShiftSIsNotConsumed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("s").copy(ctrl = true, alt = true, shift = true),
            context = hostKeyPlayerContext(),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlAltSFromEditorIsNotConsumed()` checks that the editor keeps its place for Settings too.
    // Why:      The same navigation policy covers the Settings chord as the Search chord.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+Alt+s", { route: "editor" })).toBe("unhandled");
    // ```
    /** Control plus Alt plus S in the template editor is not consumed while the policy is off. */
    @Test
    fun ctrlAltSFromEditorIsNotConsumed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("s").copy(ctrl = true, alt = true),
            context = hostKeyEditorContext(),
        )
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlGRevealsCurrentRowWithoutTransport()` checks that reveal focuses the playing row only.
    // Why:      The map says reveal focuses the row and starts or changes no track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+g", { route: "player", layout: "folded-cover" })).toBe("reveal");
    // ```
    /** Control plus G on the player reveals the playing row and runs no transport command. */
    @Test
    fun ctrlGRevealsCurrentRowWithoutTransport() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("g").copy(ctrl = true), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.RevealCurrent, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlGRevealsPlayingTrackOnAnotherFolder()` checks that reveal is consumed when the playing track
    //           sits in another folder, because the host then shows that folder first.
    // Why:      Reveal used to do nothing off the shown folder; now it moves the folder and then focuses the row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+g", { route: "player", layout: "folded-cover" })).toBe("reveal");
    // ```
    /** Control plus G on the player is consumed for the playing track even when its folder is not on screen. */
    @Test
    fun ctrlGRevealsPlayingTrackOnAnotherFolder() {
        /** Key press under test, with the player state of a playing track on another folder. */
        val press = HostKeyPress(key = hostKeyPlainInput("g").copy(ctrl = true), context = hostKeyPlayerContext())
        assertEquals(HostKeyCommand.RevealCurrent, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlGRevealsOnUnfoldedLayout()` checks that reveal works on the unfolded layout too.
    // Why:      Reveal focuses a row in the track list, which both layouts draw, so only Ctrl+O depends on layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+g", { route: "player", layout: "unfolded" })).toBe("reveal");
    // ```
    /** Control plus G on the unfolded player reveals the playing row. */
    @Test
    fun ctrlGRevealsOnUnfoldedLayout() {
        /** Host state under test, built from the player fixture with the unfolded layout. */
        val context = hostKeyPlayerContext().copy(layout = HostKeyLayout.UNFOLDED)
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("g").copy(ctrl = true), context = context)
        assertEquals(HostKeyCommand.RevealCurrent, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlOIsNotConsumedOnUnfoldedLayout()` checks that the picker chord needs the folded cover.
    // Why:      The folder picker is drawn only on the folded cover, so Ctrl+O must not toggle hidden state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+o", { route: "player", layout: "unfolded" })).toBe("unhandled");
    // ```
    /** Control plus O on the unfolded player is not consumed. */
    @Test
    fun ctrlOIsNotConsumedOnUnfoldedLayout() {
        /** Host state under test, built from the player fixture with the unfolded layout. */
        val context = hostKeyPlayerContext().copy(layout = HostKeyLayout.UNFOLDED)
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("o").copy(ctrl = true), context = context)
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun ctrlGOnSearchIsNotConsumed()` checks that reveal needs the player route.
    // Why:      Search has no track list to reveal into, so the chord must not change the route.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+g", { route: "search" })).toBe("unhandled");
    // ```
    /** Control plus G on the Search page is not consumed. */
    @Test
    fun ctrlGOnSearchIsNotConsumed() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(key = hostKeyPlainInput("g").copy(ctrl = true), context = hostKeySearchContext())
        assertEquals(HostKeyCommand.Unhandled, hostKeyCommandFor(press))
    }

    // What:     `fun nextPlaybackModeFollowsDeckOrderAndWraps()` checks the cycle order and its wrap-around.
    // Why:      The cycle must match the transport deck's mode segments, and it must return to the first mode.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(nextMode("repeat")).toBe("in_order"); expect(nextMode("shuffle_all")).toBe("repeat");
    // ```
    /** The cycle runs repeat, in order, shuffle page, shuffle all, and wraps back to repeat. */
    @Test
    fun nextPlaybackModeFollowsDeckOrderAndWraps() {
        assertEquals(PlaybackMode.IN_ORDER, hostKeyNextPlaybackMode(PlaybackMode.REPEAT))
        assertEquals(PlaybackMode.SHUFFLE_PAGE, hostKeyNextPlaybackMode(PlaybackMode.IN_ORDER))
        assertEquals(PlaybackMode.SHUFFLE_ALL, hostKeyNextPlaybackMode(PlaybackMode.SHUFFLE_PAGE))
        assertEquals(PlaybackMode.REPEAT, hostKeyNextPlaybackMode(PlaybackMode.SHUFFLE_ALL))
    }

    // What:     `fun ctrlMFromShuffleAllWrapsToRepeat()` checks that the command carries the wrapped mode.
    // Why:      The host applies the mode the command carries, so the wrap must reach the command.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(commandFor("Ctrl+m", { mode: "shuffle_all" })).toEqual({ mode: "repeat" });
    // ```
    /** Control plus M in shuffle-all mode carries the repeat mode in its command. */
    @Test
    fun ctrlMFromShuffleAllWrapsToRepeat() {
        /** Key press under test, with its key and the host state it arrives in. */
        val press = HostKeyPress(
            key = hostKeyPlainInput("m").copy(ctrl = true),
            context = hostKeyPlayerContext().copy(playbackMode = PlaybackMode.SHUFFLE_ALL),
        )
        assertEquals(HostKeyCommand.CycleMode(PlaybackMode.REPEAT), hostKeyCommandFor(press))
    }

    // What:     `fun dispatchRunsOneOperationPerExecutingCommand()` checks that each command calls its own operation.
    // Why:      A command that called the wrong operation would fire the wrong action with no other check.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // dispatch("toggle"); dispatch({ mode: "repeat" }); expect(calls).toEqual(["toggle", "mode:REPEAT"]);
    // ```
    /** Each executing command runs exactly its own host operation, in order, and consumes the key. */
    @Test
    fun dispatchRunsOneOperationPerExecutingCommand() {
        /** Recording host actions that capture each operation the dispatcher calls. */
        val actions = HostKeyRecordingActions()
        /** Dispatcher under test, wired to the recording host actions. */
        val dispatcher = HostKeyDispatcher(actions = actions)
        /** Consumed flag or flags that the dispatcher returned for this case. */
        val consumed: List<Boolean> = listOf(
            dispatcher.dispatch(HostKeyCommand.TogglePlayback),
            dispatcher.dispatch(HostKeyCommand.CycleMode(PlaybackMode.REPEAT)),
            dispatcher.dispatch(HostKeyCommand.FocusSearch),
            dispatcher.dispatch(HostKeyCommand.BackFromSearch),
        )
        assertEquals(listOf("toggle", "mode:REPEAT", "focus-search", "back"), actions.recorded())
        assertEquals(listOf(true, true, true, true), consumed)
    }

    // What:     `fun dispatchCoversEveryNavigationAndTrackOperation()` checks the remaining command-to-operation pairs.
    // Why:      Every command must reach its own operation, so a wrong pairing is caught by name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // dispatch("previous"); dispatch("next"); dispatch("open-search"); dispatch("picker");
    // dispatch("settings"); dispatch("reveal"); dispatch("close-popup");
    // ```
    /** The previous, next, new-visit Search, picker, Settings, reveal, and close-popup commands each run once. */
    @Test
    fun dispatchCoversEveryNavigationAndTrackOperation() {
        /** Recording host actions that capture each operation the dispatcher calls. */
        val actions = HostKeyRecordingActions()
        /** Dispatcher under test, wired to the recording host actions. */
        val dispatcher = HostKeyDispatcher(actions = actions)
        dispatcher.dispatch(HostKeyCommand.PreviousTrack)
        dispatcher.dispatch(HostKeyCommand.NextTrack)
        dispatcher.dispatch(HostKeyCommand.OpenSearch)
        dispatcher.dispatch(HostKeyCommand.ToggleFolderPicker)
        dispatcher.dispatch(HostKeyCommand.OpenSettings)
        dispatcher.dispatch(HostKeyCommand.RevealCurrent)
        dispatcher.dispatch(HostKeyCommand.ClosePopup)
        assertEquals(
            listOf("previous", "next", "open-search", "picker", "settings", "reveal", "close-popup"),
            actions.recorded(),
        )
    }

    // What:     `fun dispatchSwallowRepeatRunsNothingAndConsumes()` checks that a swallowed repeat calls no operation.
    // Why:      A held key must be consumed without running its action again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(dispatch("swallow-repeat")).toBe(true); expect(calls).toEqual([]);
    // ```
    /** A swallowed repeat is consumed and calls no host operation. */
    @Test
    fun dispatchSwallowRepeatRunsNothingAndConsumes() {
        /** Recording host actions that capture each operation the dispatcher calls. */
        val actions = HostKeyRecordingActions()
        /** Consumed flag or flags that the dispatcher returned for this case. */
        val consumed: Boolean = HostKeyDispatcher(actions = actions).dispatch(HostKeyCommand.SwallowRepeat)
        assertEquals(true, consumed)
        assertEquals(emptyList<String>(), actions.recorded())
    }

    // What:     `fun dispatchUnhandledRunsNothingAndDoesNotConsume()` checks that an unhandled key stays put.
    // Why:      The owner must receive a key the host does not act on, so the host must report it as not consumed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(dispatch("unhandled")).toBe(false); expect(calls).toEqual([]);
    // ```
    /** An unhandled command is not consumed and calls no host operation. */
    @Test
    fun dispatchUnhandledRunsNothingAndDoesNotConsume() {
        /** Recording host actions that capture each operation the dispatcher calls. */
        val actions = HostKeyRecordingActions()
        /** Consumed flag or flags that the dispatcher returned for this case. */
        val consumed: Boolean = HostKeyDispatcher(actions = actions).dispatch(HostKeyCommand.Unhandled)
        assertEquals(false, consumed)
        assertEquals(emptyList<String>(), actions.recorded())
    }

    // What:     `fun hostKeyExecutesOnlyForRunningCommands()` checks which commands run a host operation.
    // Why:      The repeat rule and the dispatcher both depend on this split, so it is checked on its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(executes("toggle")).toBe(true); expect(executes("swallow-repeat")).toBe(false);
    // ```
    /** Only commands that run an operation count as executing. */
    @Test
    fun hostKeyExecutesOnlyForRunningCommands() {
        assertEquals(true, hostKeyExecutes(HostKeyCommand.TogglePlayback))
        assertEquals(false, hostKeyExecutes(HostKeyCommand.SwallowRepeat))
        assertEquals(false, hostKeyExecutes(HostKeyCommand.Unhandled))
    }
}
