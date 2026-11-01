// What:     `package dev.monochromatic.musicplayer.core` places this test beside the D106 key table.
// Why:      The test reaches the table and the host key rules in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each chord is compared with the exact command it must resolve to.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `private fun playerIn(zone, ...)` is the player page with one list or the root focused.
// Why:      Every case starts from a named focus state and changes one field.
//
// In TS you'd write (pseudocode):
// ```ts
// const playerIn = (zone: HostKeyZone, rootFocused: boolean): HostKeyContext => ({ ... });
// ```
/** The player page on the unfolded layout with the given list focused, or the root focused when [zone] is none. */
private fun playerIn(
    zone: HostKeyZone = HostKeyZone.NONE,
    rootFocused: Boolean = zone == HostKeyZone.NONE,
): HostKeyContext =
    HostKeyContext(
        route = HostKeyRoute.PLAYER,
        menuOpen = false,
        textFieldFocused = false,
        rootFocused = rootFocused,
        playbackMode = PlaybackMode.IN_ORDER,
        layout = HostKeyLayout.UNFOLDED,
        zone = zone,
    )

// What:     `private fun press(key, ...)` resolves one key-down to its command.
// Why:      The cases read as the chord written the way a person presses it.
//
// In TS you'd write (pseudocode):
// ```ts
// const press = (key, mods, ctx) => hostKeyCommandFor({ key: { keyName: key, ...mods }, context: ctx });
// ```
/** Resolves a key-down with the given modifiers in the given host state. */
private fun press(
    key: String,
    context: HostKeyContext,
    ctrl: Boolean = false,
    alt: Boolean = false,
    shift: Boolean = false,
    repeat: Boolean = false,
): HostKeyCommand = hostKeyCommandFor(
    HostKeyPress(
        key = HostKeyInput(keyName = key, ctrl = ctrl, meta = false, alt = alt, shift = shift, repeat = repeat),
        context = context,
    ),
)

// What:     `class KeyBindingsTest` groups one test per D106 chord of the Android table.
// Why:      A failing name says which chord broke, and a table row without a test would show as unchecked here.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("KEY_BINDINGS", () => { /* one test per chord */ });
// ```
/** One case per chord of the D106 table for Android, plus the scope rules that keep each chord inert elsewhere. */
class KeyBindingsTest {
    /** Alt+Left seeks back ten seconds. */
    @Test
    fun altLeftSeeksBackTenSeconds() {
        assertEquals(KeySeekBy(-10), press("ArrowLeft", playerIn(), alt = true))
    }

    /** Alt+Right seeks forward ten seconds. */
    @Test
    fun altRightSeeksForwardTenSeconds() {
        assertEquals(KeySeekBy(10), press("ArrowRight", playerIn(), alt = true))
    }

    /** Alt+Shift+Left seeks back sixty seconds. */
    @Test
    fun altShiftLeftSeeksBackSixtySeconds() {
        assertEquals(KeySeekBy(-60), press("ArrowLeft", playerIn(), alt = true, shift = true))
    }

    /** Alt+Shift+Right seeks forward sixty seconds. */
    @Test
    fun altShiftRightSeeksForwardSixtySeconds() {
        assertEquals(KeySeekBy(60), press("ArrowRight", playerIn(), alt = true, shift = true))
    }

    /** Seeking keeps going while the chord is held, and also works from a focused list or a focused control. */
    @Test
    fun seekRepeatsWhileHeldAndWorksFromAnyFocusOnThePlayerPage() {
        assertEquals(KeySeekBy(10), press("ArrowRight", playerIn(), alt = true, repeat = true))
        assertEquals(KeySeekBy(10), press("ArrowRight", playerIn(HostKeyZone.TRACKS), alt = true))
        assertEquals(KeySeekBy(10), press("ArrowRight", playerIn(rootFocused = false), alt = true))
    }

    /** Ctrl+Shift+1 picks Repeat. */
    @Test
    fun ctrlShiftOnePicksRepeat() {
        assertEquals(HostKeyCommand.CycleMode(PlaybackMode.REPEAT), press("1", playerIn(), ctrl = true, shift = true))
    }

    /** Ctrl+Shift+2 picks In order. */
    @Test
    fun ctrlShiftTwoPicksInOrder() {
        assertEquals(HostKeyCommand.CycleMode(PlaybackMode.IN_ORDER), press("2", playerIn(), ctrl = true, shift = true))
    }

    /** Ctrl+Shift+3 picks Shuffle the folder. */
    @Test
    fun ctrlShiftThreePicksShuffleTheFolder() {
        assertEquals(
            HostKeyCommand.CycleMode(PlaybackMode.SHUFFLE_PAGE),
            press("3", playerIn(), ctrl = true, shift = true),
        )
    }

    /** Ctrl+Shift+4 picks Shuffle all. */
    @Test
    fun ctrlShiftFourPicksShuffleAll() {
        assertEquals(
            HostKeyCommand.CycleMode(PlaybackMode.SHUFFLE_ALL),
            press("4", playerIn(), ctrl = true, shift = true),
        )
    }

    /** Ctrl+Home restarts the track from the playback area and from a list, but not from another control. */
    @Test
    fun ctrlHomeRestartsTheTrackInThePlaybackArea() {
        assertEquals(KeyRestartTrack, press("Home", playerIn(), ctrl = true))
        assertEquals(KeyRestartTrack, press("Home", playerIn(HostKeyZone.TRACKS), ctrl = true))
        assertEquals(HostKeyCommand.Unhandled, press("Home", playerIn(rootFocused = false), ctrl = true))
    }

    /** Ctrl+Home does not repeat while held. */
    @Test
    fun ctrlHomeDoesNotRepeat() {
        assertEquals(HostKeyCommand.SwallowRepeat, press("Home", playerIn(), ctrl = true, repeat = true))
    }

    /** Ctrl+Left and Ctrl+Right still change track, also from a focused list. */
    @Test
    fun ctrlArrowsStillChangeTrackFromAList() {
        assertEquals(HostKeyCommand.PreviousTrack, press("ArrowLeft", playerIn(HostKeyZone.TRACKS), ctrl = true))
        assertEquals(HostKeyCommand.NextTrack, press("ArrowRight", playerIn(HostKeyZone.FOLDERS), ctrl = true))
    }

    /** Space plays and pauses from a focused list, where D106 keeps it; Ctrl+M cycles the mode there too. */
    @Test
    fun spaceAndCtrlMStillWorkFromAList() {
        assertEquals(HostKeyCommand.TogglePlayback, press(" ", playerIn(HostKeyZone.TRACKS)))
        assertEquals(HostKeyCommand.TogglePlayback, press(" ", playerIn(HostKeyZone.FOLDERS)))
        assertEquals(
            HostKeyCommand.CycleMode(PlaybackMode.SHUFFLE_PAGE),
            press("m", playerIn(HostKeyZone.TRACKS), ctrl = true),
        )
    }

    /** Alt+Up selects the previous folder. */
    @Test
    fun altUpSelectsThePreviousFolder() {
        assertEquals(KeyPreviousFolder, press("ArrowUp", playerIn(), alt = true))
    }

    /** Alt+Down selects the next folder. */
    @Test
    fun altDownSelectsTheNextFolder() {
        assertEquals(KeyNextFolder, press("ArrowDown", playerIn(), alt = true))
    }

    /** Ctrl+Shift+G shows the playing track's folder, and Ctrl+G still jumps to the playing track. */
    @Test
    fun ctrlShiftGRevealsTheFolderAndCtrlGStillRevealsTheTrack() {
        assertEquals(KeyRevealFolder, press("g", playerIn(), ctrl = true, shift = true))
        assertEquals(HostKeyCommand.RevealCurrent, press("g", playerIn(), ctrl = true))
    }

    /** Alt+1 focuses the folder rail. */
    @Test
    fun altOneFocusesTheFolderRail() {
        assertEquals(KeyFocusFolders, press("1", playerIn(), alt = true))
    }

    /** Alt+2 focuses the track list. */
    @Test
    fun altTwoFocusesTheTrackList() {
        assertEquals(KeyFocusTracks, press("2", playerIn(), alt = true))
    }

    /** Ctrl+Shift+O opens the system folder chooser, and Ctrl+O still opens the picker on the cover. */
    @Test
    fun ctrlShiftOOpensTheFolderChooser() {
        assertEquals(KeyOpenFolderDialog, press("o", playerIn(), ctrl = true, shift = true))
        assertEquals(
            HostKeyCommand.ToggleFolderPicker,
            press("o", playerIn().copy(layout = HostKeyLayout.FOLDED_COVER), ctrl = true),
        )
    }

    /** F5 is not bound: rescanning is no longer a user action (D113). */
    @Test
    fun f5IsNotBound() {
        assertEquals(HostKeyCommand.Unhandled, press("F5", playerIn()))
        assertEquals(HostKeyCommand.Unhandled, press("F5", playerIn(HostKeyZone.TRACKS)))
    }

    /** Up and Down move the focus ring in either list, and repeat while held. */
    @Test
    fun upAndDownMoveTheRingInEitherList() {
        for (zone in listOf(HostKeyZone.FOLDERS, HostKeyZone.NAMES, HostKeyZone.TRACKS)) {
            assertEquals(KeyMoveList(ListMove.UP), press("ArrowUp", playerIn(zone)))
            assertEquals(KeyMoveList(ListMove.DOWN), press("ArrowDown", playerIn(zone), repeat = true))
        }
    }

    /** Left and Right move the ring in the folder rail only. */
    @Test
    fun leftAndRightMoveTheRingInTheRailOnly() {
        for (zone in listOf(HostKeyZone.FOLDERS, HostKeyZone.NAMES)) {
            assertEquals(KeyMoveList(ListMove.UP), press("ArrowLeft", playerIn(zone)))
            assertEquals(KeyMoveList(ListMove.DOWN), press("ArrowRight", playerIn(zone)))
        }
        assertEquals(HostKeyCommand.Unhandled, press("ArrowLeft", playerIn(HostKeyZone.TRACKS)))
    }

    /** Home, End, Page up and Page down move the ring to the ends and by a page. */
    @Test
    fun homeEndAndPageKeysMoveTheRing() {
        assertEquals(KeyMoveList(ListMove.HOME), press("Home", playerIn(HostKeyZone.TRACKS)))
        assertEquals(KeyMoveList(ListMove.END), press("End", playerIn(HostKeyZone.TRACKS)))
        assertEquals(KeyMoveList(ListMove.PAGE_UP), press("PageUp", playerIn(HostKeyZone.FOLDERS)))
        assertEquals(KeyMoveList(ListMove.PAGE_DOWN), press("PageDown", playerIn(HostKeyZone.NAMES)))
    }

    /** Enter activates the entry under the ring in either list. */
    @Test
    fun enterActivatesTheEntryUnderTheRing() {
        assertEquals(KeyActivateRow, press("Enter", playerIn(HostKeyZone.TRACKS)))
        assertEquals(KeyActivateRow, press("Enter", playerIn(HostKeyZone.FOLDERS)))
        assertEquals(KeyActivateRow, press("Enter", playerIn(HostKeyZone.NAMES)))
    }

    /** List keys mean nothing from the root or from another control. */
    @Test
    fun listKeysAreInertOutsideTheLists() {
        assertEquals(HostKeyCommand.Unhandled, press("ArrowUp", playerIn()))
        assertEquals(HostKeyCommand.Unhandled, press("Enter", playerIn(rootFocused = false)))
    }

    /** The Menu key and Alt+Enter open the menu of the ringed track row (D114); Shift+F10 is not bound. */
    @Test
    fun menuKeyAndAltEnterOpenTheRowMenu() {
        assertEquals(KeyOpenRowMenu, press("ContextMenu", playerIn(HostKeyZone.TRACKS)))
        assertEquals(KeyOpenRowMenu, press("Enter", playerIn(HostKeyZone.TRACKS), alt = true))
        assertEquals(HostKeyCommand.Unhandled, press("F10", playerIn(HostKeyZone.TRACKS), shift = true))
    }

    /** Delete moves the ringed track row to the trash. */
    @Test
    fun deleteMovesTheRowToTheTrash() {
        assertEquals(KeyRowAction(TrackMenuAction.MOVE_TO_TRASH), press("Delete", playerIn(HostKeyZone.TRACKS)))
    }

    /** Ctrl+Shift+C copies the file name of the ringed row. */
    @Test
    fun ctrlShiftCCopiesTheFileName() {
        assertEquals(
            KeyRowAction(TrackMenuAction.COPY_FILENAME),
            press("c", playerIn(HostKeyZone.TRACKS), ctrl = true, shift = true),
        )
    }

    /** Ctrl+Shift+E shows the ringed row in the file manager. */
    @Test
    fun ctrlShiftEShowsTheFileInTheFileManager() {
        assertEquals(
            KeyRowAction(TrackMenuAction.SHOW_IN_FILE_MANAGER),
            press("e", playerIn(HostKeyZone.TRACKS), ctrl = true, shift = true),
        )
    }

    /** The row chords mean nothing in the folder rail or on the root. */
    @Test
    fun rowChordsAreInertOutsideTheTrackList() {
        assertEquals(HostKeyCommand.Unhandled, press("Delete", playerIn(HostKeyZone.FOLDERS)))
        assertEquals(HostKeyCommand.Unhandled, press("Delete", playerIn()))
    }

    /** A typed letter or digit in the rail jumps to its cell, with or without Shift. */
    @Test
    fun aTypedLetterJumpsInTheRail() {
        assertEquals(KeyJumpToLetter('b'), press("b", playerIn(HostKeyZone.FOLDERS)))
        assertEquals(KeyJumpToLetter('b'), press("b", playerIn(HostKeyZone.FOLDERS), shift = true))
        assertEquals(KeyJumpToLetter('7'), press("7", playerIn(HostKeyZone.FOLDERS)))
        assertEquals(KeyJumpToLetter('k'), press("k", playerIn(HostKeyZone.NAMES)))
    }

    /** A typed letter does not jump in the track list, with Ctrl or Alt held, or while repeating. */
    @Test
    fun aTypedLetterDoesNotJumpElsewhere() {
        assertEquals(HostKeyCommand.Unhandled, press("b", playerIn(HostKeyZone.TRACKS)))
        assertEquals(HostKeyCommand.Unhandled, press("b", playerIn(HostKeyZone.FOLDERS), alt = true))
        assertEquals(HostKeyCommand.SwallowRepeat, press("b", playerIn(HostKeyZone.FOLDERS), repeat = true))
    }

    /** Escape closes the folder picker that covers the list on the folded cover. */
    @Test
    fun escapeClosesThePickerOnTheCover() {
        /** Folded cover with the picker open. */
        val cover: HostKeyContext = playerIn().copy(layout = HostKeyLayout.FOLDED_COVER, pickerOpen = true)
        assertEquals(KeyClosePicker, press("Escape", cover))
        assertEquals(HostKeyCommand.Unhandled, press("Escape", cover.copy(pickerOpen = false)))
    }

    /** Escape with the track menu open still closes the menu first. */
    @Test
    fun escapeStillClosesTheMenuFirst() {
        assertEquals(HostKeyCommand.ClosePopup, press("Escape", playerIn().copy(menuOpen = true, pickerOpen = true)))
    }

    /** No D106 chord fires while a text field has focus, while the menu is open, or on another page. */
    @Test
    fun chordsAreInertInTextFieldsMenusAndOtherPages() {
        /** Player page with a text field focused. */
        val typing: HostKeyContext = playerIn().copy(textFieldFocused = true, rootFocused = false)
        assertEquals(HostKeyCommand.Unhandled, press("ArrowLeft", typing, alt = true))
        assertEquals(HostKeyCommand.Unhandled, press("ArrowLeft", playerIn().copy(menuOpen = true), alt = true))
        assertEquals(
            HostKeyCommand.Unhandled,
            press("ArrowLeft", playerIn().copy(route = HostKeyRoute.SEARCH, rootFocused = false), alt = true),
        )
        assertEquals(
            HostKeyCommand.Unhandled,
            press("ArrowLeft", playerIn().copy(route = HostKeyRoute.SETTINGS, rootFocused = false), alt = true),
        )
    }

    /** A chord with an extra modifier is a different chord: Ctrl+Alt+Left is not a seek. */
    @Test
    fun anExtraModifierMakesADifferentChord() {
        assertEquals(HostKeyCommand.Unhandled, press("ArrowLeft", playerIn(), ctrl = true, alt = true))
    }

    /** The Meta key never completes a chord of the table. */
    @Test
    fun metaNeverCompletesAChord() {
        /** Alt+Left with Meta also held. */
        val withMeta = HostKeyPress(
            key = HostKeyInput(
                keyName = "ArrowLeft", ctrl = false, meta = true, alt = true, shift = false, repeat = false,
            ),
            context = playerIn(),
        )
        assertNull(bindingMatching(withMeta))
    }

    /** The table has no two rows that a key press could both hit. */
    @Test
    fun noTwoRowsShareAKeyModifiersAndScope() {
        /** Signatures of every key, modifier set and scope in the table. */
        val signatures: List<String> = KEY_BINDINGS.flatMap { binding ->
            binding.keys.map { key -> "$key ${binding.ctrl} ${binding.alt} ${binding.shift} ${binding.scope}" }
        }
        assertEquals(signatures.size, signatures.toSet().size)
    }

    /** The menu keeps the row chords for the row it is open on, and ignores everything else. */
    @Test
    fun theOpenMenuMapsEachRowChordToItsAction() {
        /** Builds one input from a key name and modifiers. */
        fun input(key: String, ctrl: Boolean = false, alt: Boolean = false, shift: Boolean = false) =
            HostKeyInput(keyName = key, ctrl = ctrl, meta = false, alt = alt, shift = shift, repeat = false)
        assertEquals(TrackMenuAction.MOVE_TO_TRASH, menuRowActionFor(input("Delete")))
        assertEquals(TrackMenuAction.COPY_FILENAME, menuRowActionFor(input("c", ctrl = true, shift = true)))
        assertEquals(TrackMenuAction.SHOW_IN_FILE_MANAGER, menuRowActionFor(input("e", ctrl = true, shift = true)))
        assertNull(menuRowActionFor(input("Enter", alt = true)))
        assertNull(menuRowActionFor(input("F5", ctrl = true, shift = true)))
        assertNull(menuRowActionFor(input("Enter")))
        assertNull(menuRowActionFor(input("Delete").copy(repeat = true)))
    }

    /** A seek stays inside the track and starts at zero. */
    @Test
    fun seekTargetStaysInsideTheTrack() {
        assertEquals(40.0, seekTargetSec(positionSec = 30.0, durationSec = 360.0, deltaSec = 10), 0.0)
        assertEquals(0.0, seekTargetSec(positionSec = 4.0, durationSec = 360.0, deltaSec = -60), 0.0)
        assertEquals(360.0, seekTargetSec(positionSec = 350.0, durationSec = 360.0, deltaSec = 60), 0.0)
        assertEquals(70.0, seekTargetSec(positionSec = 10.0, durationSec = 0.0, deltaSec = 60), 0.0)
    }

    /** Alt+Up and Alt+Down stop at the first and last folder and step by one in between. */
    @Test
    fun adjacentFolderStopsAtTheEnds() {
        /** Folder labels in order. */
        val labels: List<String> = listOf("A", "B", "C")
        assertEquals("C", adjacentFolderLabel(labels, selected = 1, step = 1))
        assertEquals("A", adjacentFolderLabel(labels, selected = 1, step = -1))
        assertNull(adjacentFolderLabel(labels, selected = 0, step = -1))
        assertNull(adjacentFolderLabel(labels, selected = 2, step = 1))
        assertNull(adjacentFolderLabel(emptyList(), selected = 0, step = 1))
    }

    /** Every command of the table that the host runs names an executing command, and list commands do not execute. */
    @Test
    fun listCommandsStayWithTheListAndHostCommandsExecute() {
        assertTrue(hostKeyExecutes(KeySeekBy(10)))
        assertTrue(hostKeyExecutes(KeyRestartTrack))
        assertEquals(false, hostKeyExecutes(KeyMoveList(ListMove.UP)))
        assertEquals(false, hostKeyExecutes(KeyActivateRow))
    }
}
