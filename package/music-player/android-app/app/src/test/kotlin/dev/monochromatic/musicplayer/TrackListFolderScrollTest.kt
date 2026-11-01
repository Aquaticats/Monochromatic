// Track list folder reveal cases: which list item the folder reveal scrolls to.

// What:     `package dev.monochromatic.musicplayer` places this test beside the track list pane.
// Why:      The test reaches the module-internal `folderScrollIndexFor` without exposing it publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the entry types the cases build.
// Why:      The cases build rows and headers the same way the core layer does.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { TrackListEntry, TrackRowModel } from "core/TrackListEntries";
// ```
import dev.monochromatic.musicplayer.core.TrackListEntry
import dev.monochromatic.musicplayer.core.TrackRowModel

// What:     JUnit's assertions and test annotation register each folder reveal case.
// Why:      Each case compares one expected list position with the function's result.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class TrackListFolderScrollTest` groups the folder reveal target cases.
// Why:      One class keeps the row and header builders next to the cases that use them.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("folderScrollIndexFor", () => { /* cases */ });
// ```
/** Verifies which list item the folder reveal scrolls to for each arrangement of playing row and headers. */
class TrackListFolderScrollTest {
    // What:     `private fun row(...)` builds one track row entry.
    // Why:      Each case states only the library index; the playing row is named to the function under test by its
    //           index, because the rows of a page carry no playing flag.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const row = (index: number): TrackListEntry => ({ kind: "row", row: { indexInLibrary: index } });
    // ```
    /** Builds one track row entry with the given library index. */
    private fun row(index: Int): TrackListEntry = TrackListEntry.RowEntry(
        TrackRowModel(indexInLibrary = index, title = "Track $index", supportingLine = null, isCurrent = false),
    )

    // What:     `private fun header(...)` builds one subfolder header entry.
    // Why:      Each case names the subfolder that its following rows belong to.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const header = (name: string): TrackListEntry => ({ kind: "header", name, trackCount: 1 });
    // ```
    /** Builds one subfolder header entry with the given name. */
    private fun header(name: String): TrackListEntry = TrackListEntry.HeaderEntry(name = name, trackCount = 1)

    // What:     `@Test fun playingRowInSubfolderScrollsToItsHeader()` checks a playing row under a subfolder header.
    // Why:      The header that sits directly above the row is the subfolder that holds it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing row in subfolder", () => {
    //   expect(folderScrollIndexFor([row(0), header("Live"), row(1)], 1)).toBe(1);
    // });
    // ```
    /** A playing row inside a subfolder scrolls to that subfolder's header. */
    @Test
    fun playingRowInSubfolderScrollsToItsHeader() {
        /** Own track, then the Live subfolder header with the playing track under it. */
        val entries: List<TrackListEntry> = listOf(row(0), header("Live"), row(1), row(2))
        assertEquals(1, folderScrollIndexFor(entries, playingIndex = 1))
    }

    // What:     `@Test fun playingRowDirectlyInFolderScrollsToTop()` checks a playing row with no header above it.
    // Why:      A track directly in the folder has no subfolder header, so the reveal goes to the top of the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing row in folder", () => {
    //   expect(folderScrollIndexFor([row(0), header("Live"), row(1)], 0)).toBe(0);
    // });
    // ```
    /** A playing row directly in the folder, with only subfolders after it, scrolls to the top of the list. */
    @Test
    fun playingRowDirectlyInFolderScrollsToTop() {
        /** Own playing track first, then a subfolder header with its own tracks after it. */
        val entries: List<TrackListEntry> = listOf(row(0), header("Live"), row(1))
        assertEquals(0, folderScrollIndexFor(entries, playingIndex = 0))
    }

    // What:     `@Test fun nearestHeaderAboveThePlayingRowWins()` checks a playing row under a later subfolder.
    // Why:      Several headers come before the row, and only the nearest one owns the row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("nearest header", () => {
    //   expect(folderScrollIndexFor([row(0), header("Live"), row(1), header("Remix"), row(2)], 2)).toBe(3);
    // });
    // ```
    /** With several subfolders above the playing row, the scroll goes to the nearest header, which is the row's own. */
    @Test
    fun nearestHeaderAboveThePlayingRowWins() {
        /** Two subfolders, with the playing track under the second one. */
        val entries: List<TrackListEntry> = listOf(
            row(0),
            header("Live"),
            row(1),
            header("Remix"),
            row(2),
        )
        assertEquals(3, folderScrollIndexFor(entries, playingIndex = 2))
    }

    // What:     `@Test fun playingOwnTrackBeforeAnyHeaderScrollsToTop()` checks an own track listed first.
    // Why:      Own tracks come before any header, so a playing own track must not be credited with a later header.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("own track before headers", () => {
    //   expect(folderScrollIndexFor([row(0), row(1), header("Live"), row(2)], 1)).toBe(0);
    // });
    // ```
    /** A playing own track listed before every subfolder header scrolls to the top of the list. */
    @Test
    fun playingOwnTrackBeforeAnyHeaderScrollsToTop() {
        /** Two own tracks, the second playing, then a subfolder header and its track. */
        val entries: List<TrackListEntry> = listOf(row(0), row(1), header("Live"), row(2))
        assertEquals(0, folderScrollIndexFor(entries, playingIndex = 1))
    }

    // What:     `@Test fun noPlayingRowScrollsToTop()` checks a list with no playing row.
    // Why:      With no playing row in this list there is no header to reveal, so the target is the top.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no playing row", () => expect(folderScrollIndexFor([header("Live"), row(1)], null)).toBe(0));
    // ```
    /** A list without a playing row scrolls to the top, even when it has subfolder headers. */
    @Test
    fun noPlayingRowScrollsToTop() {
        /** Subfolder header and its track, neither of them playing. */
        val entries: List<TrackListEntry> = listOf(header("Live"), row(1))
        assertEquals(0, folderScrollIndexFor(entries, playingIndex = null))
    }

    // What:     `@Test fun emptyListScrollsToTop()` checks an empty list.
    // Why:      A folder with no entries must still give a valid target, so the reveal never reads past the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty list", () => expect(folderScrollIndexFor([], null)).toBe(0));
    // ```
    /** An empty list gives the top of the list as the target. */
    @Test
    fun emptyListScrollsToTop() {
        assertEquals(0, folderScrollIndexFor(emptyList(), playingIndex = null))
    }
}
