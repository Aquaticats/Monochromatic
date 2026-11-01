// What:     `package dev.monochromatic.musicplayer` places this test beside the composables it covers.
// Why:      The test reaches the module-internal helpers without exposing them publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the rail builder.
// Why:      The lookup tests build a real rail from folder names, so they exercise the same model as the picker.
//
// In TS you'd write (pseudocode):
// ```ts
// import { railFor } from "core/FolderRail";
// ```
import dev.monochromatic.musicplayer.core.railFor

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each helper outcome is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

// What:     `private val SAMPLE_FOLDERS: List<String>` is the folder library the lookup tests share.
// Why:      It holds two Latin cells and one Japanese cell, so every lookup has a case in each section.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_FOLDERS = ["Camellia", "Adele", "C418", "あいみょん", "Casiopea"];
// ```
/** Folder names covering two Latin cells and one Japanese kana cell, in library order. */
private val SAMPLE_FOLDERS: List<String> = listOf("Camellia", "Adele", "C418", "あいみょん", "Casiopea")

// What:     `class PlayerTopBarsModelTest` groups the pure-helper tests for the top bars and the folder picker.
// Why:      The text and lookup helpers can be verified on the host JVM without a Compose runtime.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("PlayerTopBars helpers", () => { ... });
// ```
/** Verifies the spoken text and the rail lookups behind the top bars and the folder picker. */
class PlayerTopBarsModelTest {
    // What:     `openTriggerReportsExpanded` checks the open state text.
    // Why:      Screen readers must hear Expanded while the picker is open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("open trigger reports Expanded", () => { ... });
    // ```
    /** Confirms an open picker is described as Expanded. */
    @Test
    fun openTriggerReportsExpanded() {
        assertEquals("Expanded", folderTriggerStateDescription(pickerOpen = true))
    }

    // What:     `closedTriggerReportsCollapsed` checks the closed state text.
    // Why:      Screen readers must hear Collapsed while the picker is closed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("closed trigger reports Collapsed", () => { ... });
    // ```
    /** Confirms a closed picker is described as Collapsed. */
    @Test
    fun closedTriggerReportsCollapsed() {
        assertEquals("Collapsed", folderTriggerStateDescription(pickerOpen = false))
    }

    // What:     `triggerContentDescriptionNamesFolder` checks the spoken name for an ordinary title.
    // Why:      The trigger is announced as Folder followed by the current folder name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("trigger names the folder", () => { ... });
    // ```
    /** Confirms the trigger's spoken name is Folder followed by the title. */
    @Test
    fun triggerContentDescriptionNamesFolder() {
        assertEquals("Folder: Camellia", folderTriggerContentDescription(folderTitle = "Camellia"))
    }

    // What:     `triggerContentDescriptionKeepsEmptyTitle` checks an empty title.
    // Why:      A library folder with an empty name still produces a stable spoken label without a crash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty title keeps the prefix", () => { ... });
    // ```
    /** Confirms an empty title produces the prefix alone rather than failing. */
    @Test
    fun triggerContentDescriptionKeepsEmptyTitle() {
        assertEquals("Folder: ", folderTriggerContentDescription(folderTitle = ""))
    }

    // What:     `cellKeysListInRailOrder` checks the flat cell order across sections.
    // Why:      The names list item indexes follow this order, so every scroll target depends on it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("cells follow rail order", () => { ... });
    // ```
    /** Confirms the flat cell list runs Latin A, Latin C, then the Japanese kana cell. */
    @Test
    fun cellKeysListInRailOrder() {
        assertEquals(
            listOf("latinA", "latinC", "jpnあ"),
            cellsInOrder(railFor(SAMPLE_FOLDERS)).map { cell -> cell.key },
        )
    }

    // What:     `cellKeyForLatinFolder` checks the highlight lookup for a Latin folder.
    // Why:      The current folder's cell is the one the rail highlights.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("latin folder maps to its cell", () => { ... });
    // ```
    /** Confirms a Latin folder maps to the cell keyed by its section and letter. */
    @Test
    fun cellKeyForLatinFolder() {
        assertEquals("latinC", cellKeyForFolder(railFor(SAMPLE_FOLDERS), folder = "Camellia"))
    }

    // What:     `cellKeyForJapaneseFolder` checks the highlight lookup in a later section.
    // Why:      A folder in a later writing system must still highlight its own cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("japanese folder maps to its kana cell", () => { ... });
    // ```
    /** Confirms a Japanese folder maps to its kana row cell. */
    @Test
    fun cellKeyForJapaneseFolder() {
        assertEquals("jpnあ", cellKeyForFolder(railFor(SAMPLE_FOLDERS), folder = "あいみょん"))
    }

    // What:     `railHighlightPrefersTheTappedCell` checks that a tapped letter is the one circled.
    // Why:      Tapping V while W is current must circle V, not leave W circled.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tapped cell wins", () => expect(railHighlightKey("latinV", "latinW")).toBe("latinV"));
    // ```
    /** Confirms a pending tap overrides the current folder's cell. */
    @Test
    fun railHighlightPrefersTheTappedCell() {
        assertEquals("latinV", railHighlightKey(tappedKey = "latinV", currentKey = "latinW"))
    }

    // What:     `railHighlightFallsBackToTheCurrentCell` checks the highlight with no pending tap.
    // Why:      Before any tap, and after the current folder changes, the circle marks the current folder's cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no tap uses current", () => expect(railHighlightKey(null, "latinW")).toBe("latinW"));
    // ```
    /** Confirms the current folder's cell is circled when no tap is pending. */
    @Test
    fun railHighlightFallsBackToTheCurrentCell() {
        assertEquals("latinW", railHighlightKey(tappedKey = null, currentKey = "latinW"))
    }

    // What:     `cellKeyForNullFolderIsNull` checks the lookup when no folder is current.
    // Why:      A null current folder must highlight nothing rather than match an empty name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("null folder has no cell", () => { ... });
    // ```
    /** Confirms a null current folder has no cell. */
    @Test
    fun cellKeyForNullFolderIsNull() {
        assertNull(cellKeyForFolder(railFor(SAMPLE_FOLDERS), folder = null))
    }

    // What:     `cellKeyForUnknownFolderIsNull` checks the lookup for a name absent from the library.
    // Why:      A folder that left the library must not highlight a stale cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown folder has no cell", () => { ... });
    // ```
    /** Confirms a folder absent from the library has no cell. */
    @Test
    fun cellKeyForUnknownFolderIsNull() {
        assertNull(cellKeyForFolder(railFor(SAMPLE_FOLDERS), folder = "Missing"))
    }

    // What:     `firstNameForLatinCell` checks the first folder under a Latin cell.
    // Why:      A rail tap scrolls the names to this folder, so it must be the cell's first name in library order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("first name of latin C", () => { ... });
    // ```
    /** Confirms the first folder under the C cell is the first C name in library order. */
    @Test
    fun firstNameForLatinCell() {
        assertEquals("Camellia", firstNameForCell(railFor(SAMPLE_FOLDERS), cellKey = "latinC"))
    }

    // What:     `firstNameForUnknownCellIsNull` checks a rail tap on a cell that does not exist.
    // Why:      An unknown key must scroll nowhere instead of picking an arbitrary name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown cell has no first name", () => { ... });
    // ```
    /** Confirms an unknown cell key has no first name. */
    @Test
    fun firstNameForUnknownCellIsNull() {
        assertNull(firstNameForCell(railFor(SAMPLE_FOLDERS), cellKey = "latinZ"))
    }

    // What:     `itemIndexForJapaneseCell` checks the scroll index of the third cell.
    // Why:      The names list scrolls to this item index, so it must count every earlier cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("kana cell is third", () => { ... });
    // ```
    /** Confirms the kana cell sits at list position two after the two Latin cells. */
    @Test
    fun itemIndexForJapaneseCell() {
        assertEquals(2, itemIndexForCell(railFor(SAMPLE_FOLDERS), cellKey = "jpnあ"))
    }

    // What:     `itemIndexForFolderInSecondCell` checks that a folder maps to its own cell's index.
    // Why:      The picker opens on the current folder, so a folder inside a multi-name cell must resolve to that cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder resolves to its cell index", () => { ... });
    // ```
    /** Confirms a folder in the second Latin cell resolves to list position one. */
    @Test
    fun itemIndexForFolderInSecondCell() {
        assertEquals(1, itemIndexForFolder(railFor(SAMPLE_FOLDERS), folder = "C418"))
    }

    // What:     `itemIndexForFirstCellFolder` checks the opening index for a folder in the first cell.
    // Why:      A current folder in the first cell opens the list at position zero.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("first cell folder opens at zero", () => { ... });
    // ```
    /** Confirms a folder in the first cell resolves to list position zero. */
    @Test
    fun itemIndexForFirstCellFolder() {
        assertEquals(0, itemIndexForFolder(railFor(SAMPLE_FOLDERS), folder = "Adele"))
    }

    // What:     `itemIndexForNullCellIsNull` checks the opening index when no cell applies.
    // Why:      With no current cell the list opens at the top, so the lookup must return null.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("null cell has no index", () => { ... });
    // ```
    /** Confirms a null cell key has no list position. */
    @Test
    fun itemIndexForNullCellIsNull() {
        assertNull(itemIndexForCell(railFor(SAMPLE_FOLDERS), cellKey = null))
    }

    // What:     `emptyLibraryHasNoCells` checks the lookups on an empty library.
    // Why:      A library with no folders must produce no cells and no scroll target.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty library has no cells", () => { ... });
    // ```
    /** Confirms an empty library has no cells and no folder index. */
    @Test
    fun emptyLibraryHasNoCells() {
        assertNull(itemIndexForFolder(railFor(emptyList()), folder = "Camellia"))
    }
}
