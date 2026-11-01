// Cases of `reportFirstVisibleTrack`: the list position becomes a library index, never the position itself.

// What:     `package ...core` places the test beside `FirstVisibleTrack.kt`, which declares the function.
// Why:      The test reaches the internal `reportFirstVisibleTrack` and `NO_VISIBLE_TRACK` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class FirstVisibleTrackTest` groups the cases of the first visible track report.
// Why:      The measure refresh reads from the library index the list reports, and a position written in its place
//           would start the reading at an unrelated track once the page holds headers or part of the library.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("reportFirstVisibleTrack", () => { /* cases */ });
// ```
/** Verifies what the list reports to the measure refresh for each first visible position. */
class FirstVisibleTrackTest {
    /** Builds one track row entry with the given library index. */
    private fun row(index: Int): TrackListEntry = TrackListEntry.RowEntry(
        TrackRowModel(indexInLibrary = index, title = "Track $index", supportingLine = null, isCurrent = false),
    )

    /** Builds one subfolder header entry. */
    private fun header(name: String): TrackListEntry = TrackListEntry.HeaderEntry(name = name, trackCount = 1)

    /** Collects what [reportFirstVisibleTrack] reports for [positions] over [entries]. */
    private fun reported(entries: List<TrackListEntry>, vararg positions: Int): List<Int> = runBlocking {
        /** Values reported so far, in report order. */
        val values: MutableList<Int> = mutableListOf()
        reportFirstVisibleTrack(positions = flowOf(*positions.toTypedArray()), entries = entries) { index ->
            values += index
        }
        return@runBlocking values
    }

    // What:     `@Test fun aPositionIsReportedAsTheLibraryIndexOfItsRow()` checks the conversion on rows and headers.
    // Why:      A page lists a subset of the library under headers, so a row's position and its library index differ.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("library index", () => {
    //   expect(reported([header, row(7), row(9), header, row(12)], 1, 2, 3)).toEqual([7, 9, 12]);
    // });
    // ```
    /** Positions one and two report library indexes seven and nine, and a header position reports the next row. */
    @Test
    fun aPositionIsReportedAsTheLibraryIndexOfItsRow() {
        /** A header, two rows, a header, and a row, whose library indexes differ from their positions. */
        val entries: List<TrackListEntry> = listOf(header("Live"), row(7), row(9), header("Studio"), row(12))
        assertEquals(listOf(7, 7, 9, 12), reported(entries, 0, 1, 2, 3))
    }

    // What:     `@Test fun everyPositionTheListReachesIsReportedInOrder()` checks the report keeps up with scrolling.
    // Why:      The refresh reads the cell between batches, so each scroll must update it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("reports each position", () => expect(reported(rows, 0, 3, 1)).toEqual([10, 40, 20]));
    // ```
    /** Three positions give three reports in the order the list reached them. */
    @Test
    fun everyPositionTheListReachesIsReportedInOrder() {
        /** Four rows with library indexes ten apart. */
        val entries: List<TrackListEntry> = listOf(row(10), row(20), row(30), row(40))
        assertEquals(listOf(10, 40, 20), reported(entries, 0, 3, 1))
    }

    // What:     `@Test fun aListWithoutRowsReportsNoVisibleTrack()` checks the value for a list that shows no track.
    // Why:      The cell must say that no track is on screen, not a stale or zero index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no rows", () => expect(reported([header("Live")], 0)).toEqual([NO_VISIBLE_TRACK]));
    // ```
    /** A list with only a header, and an empty list, report the none value. */
    @Test
    fun aListWithoutRowsReportsNoVisibleTrack() {
        assertEquals(listOf(NO_VISIBLE_TRACK), reported(listOf(header("Live")), 0))
        assertEquals(listOf(NO_VISIBLE_TRACK), reported(emptyList(), 0))
    }
}
