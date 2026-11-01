// What:     `package dev.monochromatic.musicplayer.core` places this test beside the track menu actions.
// Why:      The test reaches the public action list and label function in the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each action outcome is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class TrackMenuActionsTest` groups the pure tests for the track menu action list.
// Why:      The order, labels, groups, and destructive marking can be verified on the host JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("track menu actions", () => { ... });
// ```
/** Verifies the accepted D7 action order, wording, grouping, and destructive marking. */
class TrackMenuActionsTest {
    // What:     `orderMatchesDecisionD7` compares the list with the enumerated D7 order.
    // Why:      The menu must show the seven actions in exactly the order the decision lists them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("order matches D7", () => { ... });
    // ```
    /** Confirms the action list follows the seven D7 actions in display order. */
    @Test
    fun orderMatchesDecisionD7() {
        assertEquals(
            listOf(
                TrackMenuAction.PLAY,
                TrackMenuAction.SHUFFLE_FROM_HERE,
                TrackMenuAction.DETAILS,
                TrackMenuAction.SHOW_IN_FILE_MANAGER,
                TrackMenuAction.COPY_FILENAME,
                TrackMenuAction.MOVE_TO_TRASH,
            ),
            trackMenuActionsFor(),
        )
    }

    // What:     `playLabelIsPlay` checks the first action's wording.
    // Why:      The first D7 row must read Play exactly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("play label", () => expect(label("play")).toBe("Play"));
    // ```
    /** Confirms the play action reads Play. */
    @Test
    fun playLabelIsPlay() {
        assertEquals("Play", label(TrackMenuAction.PLAY))
    }

    // What:     `shuffleLabelIsAccepted` checks the shuffle action's wording.
    // Why:      The accepted D7 wording must be reproduced exactly, including case and spacing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("shuffle label", () => expect(label("shuffleFromHere")).toBe("Start shuffle from here"));
    // ```
    /** Confirms the shuffle action reads Start shuffle from here. */
    @Test
    fun shuffleLabelIsAccepted() {
        assertEquals("Start shuffle from here", label(TrackMenuAction.SHUFFLE_FROM_HERE))
    }

    // What:     `detailsLabelIsAccepted` checks the file details action's wording.
    // Why:      The inline peak value is a separate caller concern, so this label is the bare D7 text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("details label", () => expect(label("details")).toBe("File details"));
    // ```
    /** Confirms the file details action reads File details. */
    @Test
    fun detailsLabelIsAccepted() {
        assertEquals("File details", label(TrackMenuAction.DETAILS))
    }

    /** The six actions each have one lowercase access letter, and no two actions share a letter (D114). */
    @Test
    fun accessLettersAreUniqueAndSitInTheirLabels() {
        val letters = TrackMenuAction.entries.map { action -> accessKey(action).letter }
        assertEquals(letters.size, letters.toSet().size)
        TrackMenuAction.entries.forEach { action ->
            val key = accessKey(action)
            assertEquals("lowercase letter for $action", key.letter.lowercaseChar(), key.letter)
            assertTrue("letter is a letter for $action", key.letter.isLetter())
            assertEquals(
                "label of $action has its letter at the underlined index",
                key.letter,
                label(action)[key.labelIndex].lowercaseChar(),
            )
        }
    }

    /** Pressing each action's letter with no modifier runs exactly that action. */
    @Test
    fun eachAccessLetterRunsItsRow() {
        TrackMenuAction.entries.forEach { action ->
            val pressed = HostKeyInput(accessKey(action).letter.toString(), ctrl = false, meta = false, alt = false, shift = false, repeat = false)
            assertEquals(action, trackMenuActionForAccessKey(pressed))
        }
    }

    /** Shift does not change the letter, but Control, Alt, Meta and a repeat never run a row. */
    @Test
    fun modifiersAndRepeatsAndOtherKeysDoNotRunARow() {
        val base = HostKeyInput("p", ctrl = false, meta = false, alt = false, shift = false, repeat = false)
        assertEquals(TrackMenuAction.PLAY, trackMenuActionForAccessKey(base.copy(shift = true)))
        assertNull(trackMenuActionForAccessKey(base.copy(ctrl = true)))
        assertNull(trackMenuActionForAccessKey(base.copy(alt = true)))
        assertNull(trackMenuActionForAccessKey(base.copy(meta = true)))
        assertNull(trackMenuActionForAccessKey(base.copy(repeat = true)))
        assertNull(trackMenuActionForAccessKey(base.copy(keyName = "x")))
        assertNull(trackMenuActionForAccessKey(base.copy(keyName = "Enter")))
    }

    /** The menu lists six rows and no re-analysis row exists anywhere (D113). */
    @Test
    fun noRowOffersReanalysis() {
        assertEquals(6, trackMenuActionsFor().size)
        assertTrue(trackMenuActionsFor().none { action -> label(action).contains("analy", ignoreCase = true) })
    }

    // What:     `showLabelIsAccepted` checks the file manager action's wording.
    // Why:      The accepted D7 wording names the file manager explicitly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("show label", () => expect(label("showInFileManager")).toBe("Show in file manager"));
    // ```
    /** Confirms the file manager action reads Show in file manager. */
    @Test
    fun showLabelIsAccepted() {
        assertEquals("Show in file manager", label(TrackMenuAction.SHOW_IN_FILE_MANAGER))
    }

    // What:     `copyLabelIsAccepted` checks the copy action's wording.
    // Why:      The accepted D7 wording uses the single word filename.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("copy label", () => expect(label("copyFilename")).toBe("Copy filename"));
    // ```
    /** Confirms the copy action reads Copy filename. */
    @Test
    fun copyLabelIsAccepted() {
        assertEquals("Copy filename", label(TrackMenuAction.COPY_FILENAME))
    }

    // What:     `trashLabelIsAccepted` checks the destructive action's wording.
    // Why:      Decision D8 names the action Move to trash, and the label must not hide that meaning.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("trash label", () => expect(label("moveToTrash")).toBe("Move to trash"));
    // ```
    /** Confirms the destructive action reads Move to trash. */
    @Test
    fun trashLabelIsAccepted() {
        assertEquals("Move to trash", label(TrackMenuAction.MOVE_TO_TRASH))
    }

    // What:     `onlyTrashIsDestructive` checks which actions carry the destructive flag.
    // Why:      Exactly one action removes a file, and the menu must mark only that one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("only trash is destructive", () => { ... });
    // ```
    /** Confirms that Move to trash is the only destructive action. */
    @Test
    fun onlyTrashIsDestructive() {
        /** Actions whose destructive flag is set, in display order. */
        val destructive: List<TrackMenuAction> = TrackMenuAction.entries.filter { action -> action.destructive }
        assertEquals(listOf(TrackMenuAction.MOVE_TO_TRASH), destructive)
    }

    // What:     `groupsNeverDecrease` checks that the display order keeps each group contiguous.
    // Why:      Dividers are drawn at group changes, so a group that reappears would split a section.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("groups never decrease", () => { ... });
    // ```
    /** Confirms the group numbers never go backwards along the display order. */
    @Test
    fun groupsNeverDecrease() {
        /** Divider group of each action in display order. */
        val groups: List<Int> = trackMenuActionsFor().map { action -> action.group }
        for ((index, group) in groups.withIndex()) {
            if (index > 0) {
                assertTrue(groups[index - 1] <= group)
            }
        }
    }

    // What:     `labelsAreDistinct` checks that no two actions share a visible label.
    // Why:      Identical labels would make two rows indistinguishable to a user and to TalkBack.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("labels are distinct", () => { ... });
    // ```
    /** Confirms every action has its own visible label. */
    @Test
    fun labelsAreDistinct() {
        /** Visible label of each action in display order. */
        val labels: List<String> = trackMenuActionsFor().map { action -> label(action) }
        assertEquals(labels.size, labels.toSet().size)
    }

    // What:     `nonDestructiveActionsAreNotMarked` checks that the first action is not destructive.
    // Why:      Play is the default action and must never carry the destructive marking.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("play is not destructive", () => expect(action("play").destructive).toBe(false));
    // ```
    /** Confirms the play action is not marked destructive. */
    @Test
    fun nonDestructiveActionsAreNotMarked() {
        assertFalse(TrackMenuAction.PLAY.destructive)
    }
}
