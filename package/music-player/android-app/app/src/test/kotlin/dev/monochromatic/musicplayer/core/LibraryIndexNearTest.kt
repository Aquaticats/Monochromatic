// Cases of `libraryIndexNear` when it falls back to the rows before a position: the nearest row wins.

// What:     `package ...core` places the test beside `ListScrollRules.kt`, which declares the function.
// Why:      The test reaches the internal `libraryIndexNear` and the entry types without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class LibraryIndexNearTest` groups the nearest-row cases that need more than one row before a position.
// Why:      With a single row before the position, "nearest" and "first" name the same row, so the fallback order
//           needs lists with several rows in front of the header.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("libraryIndexNear nearest row", () => { /* cases */ });
// ```
/** Verifies the fallback picks the last row before the position, not the first row of the list. */
class LibraryIndexNearTest {
    /** Builds one track row entry with the given library index. */
    private fun row(index: Int): TrackListEntry = TrackListEntry.RowEntry(
        TrackRowModel(indexInLibrary = index, title = "Track $index", supportingLine = null, isCurrent = false),
    )

    /** Builds one subfolder header entry. */
    private fun header(name: String): TrackListEntry = TrackListEntry.HeaderEntry(name = name, trackCount = 1)

    // What:     `@Test fun aHeaderAtTheEndNamesTheLastRowBeforeIt()` checks the fallback from a header position.
    // Why:      The user is looking at the end of the list, so the rows just above the header are the ones on screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("nearest before", () => expect(libraryIndexNear([row(3), row(5), header("Live")], 2)).toBe(5));
    // ```
    /** With two rows then a header, the header's position names the second row's library index. */
    @Test
    fun aHeaderAtTheEndNamesTheLastRowBeforeIt() {
        /** Two rows with a header after them. */
        val entries: List<TrackListEntry> = listOf(row(3), row(5), header("Live"))
        assertEquals(5, libraryIndexNear(entries, position = 2))
    }

    // What:     `@Test fun aPositionPastTheEndNamesTheLastRow()` checks the fallback on a position outside the list.
    // Why:      A list that shrank under the remembered first visible position must still name the closest row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("past the end", () => expect(libraryIndexNear([row(3), row(5)], 9)).toBe(5));
    // ```
    /** A position past the end of a list of rows names the last row, not the first. */
    @Test
    fun aPositionPastTheEndNamesTheLastRow() {
        /** Two rows and nothing else. */
        val entries: List<TrackListEntry> = listOf(row(3), row(5))
        assertEquals(5, libraryIndexNear(entries, position = 9))
    }

    // What:     `@Test fun severalTrailingHeadersNameTheLastRowBeforeThem()` checks the fallback across headers.
    // Why:      Empty subfolders can leave several headers at the end, and the scan must walk back over all of them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("trailing headers", () => expect(libraryIndexNear([row(3), row(5), header("a"), header("b")], 3)).toBe(5));
    // ```
    /** Two rows then two headers: the position of the last header names the second row. */
    @Test
    fun severalTrailingHeadersNameTheLastRowBeforeThem() {
        /** Two rows with two headers after them. */
        val entries: List<TrackListEntry> = listOf(row(3), row(5), header("a"), header("b"))
        assertEquals(5, libraryIndexNear(entries, position = 3))
    }
}
