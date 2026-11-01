// Page identity, playing position, and page scroll target cases for the track list pane's scroll rules.

// What:     `package dev.monochromatic.musicplayer.core` places these tests beside the rules they exercise.
// Why:      The tests reach the internal rule functions through the same package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit's assertions and test annotation register each scroll rule case.
// Why:      Each case compares one identity, position, key, or target value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class ListScrollRulesTest` groups the identity, position, and scroll target cases.
// Why:      One class keeps the row and header builders next to the cases that use them.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("list scroll rules", () => { /* cases */ });
// ```
/** Verifies that page identity ignores play and measure changes, and that scroll targets pick the playing row. */
class ListScrollRulesTest {
    // What:     `private fun row(...)` builds one track row entry.
    // Why:      Each case states only the library index, the playing flag, the supporting line, and the title it needs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const row = (index: number, current = false, supportingLine: string | null = null, title = `Track ${index}`) =>
    //   ({ kind: "row", row: { indexInLibrary: index, current, supportingLine, title } });
    // ```
    /** Builds one track row entry with the given library index, playing flag, supporting line, and title. */
    private fun row(
        index: Int,
        current: Boolean = false,
        supportingLine: String? = null,
        title: String = "Track $index",
    ): TrackListEntry = TrackListEntry.RowEntry(
        TrackRowModel(
            indexInLibrary = index,
            title = title,
            supportingLine = supportingLine,
            isCurrent = current,
        ),
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

    // What:     `@Test fun identityIgnoresWhichRowIsPlaying()` checks that a play change keeps the page identity.
    // Why:      Next, previous, and taps move the playing flag, and that alone must never count as a page change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("identity ignores playing row", () => {
    //   expect(pageIdentityOf([row(0, true), row(1)])).toEqual(pageIdentityOf([row(0), row(1, true)]));
    // });
    // ```
    /** Moving the playing flag to another row of the same page leaves the page identity unchanged. */
    @Test
    fun identityIgnoresWhichRowIsPlaying() {
        /** Page with the first track playing. */
        val before: List<TrackListEntry> = listOf(row(0, current = true), header("Live"), row(1))
        /** Same page with the second track playing. */
        val after: List<TrackListEntry> = listOf(row(0), header("Live"), row(1, current = true))
        assertEquals(pageIdentityOf(before), pageIdentityOf(after))
    }

    // What:     `@Test fun identityIgnoresMeasureSupportingLines()` checks that a measure update keeps the identity.
    // Why:      Measures fill in supporting lines after the page is shown, and that must never count as a page change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("identity ignores measures", () => {
    //   expect(pageIdentityOf([row(0)])).toEqual(pageIdentityOf([row(0, false, "3:10")]));
    // });
    // ```
    /** A supporting line appearing or changing on a row leaves the page identity unchanged. */
    @Test
    fun identityIgnoresMeasureSupportingLines() {
        /** Page whose tracks have no measure yet. */
        val unmeasured: List<TrackListEntry> = listOf(row(0), row(1))
        /** Same page after its measures arrive, with a different value on the second track. */
        val measured: List<TrackListEntry> = listOf(row(0, supportingLine = "3:10"), row(1, supportingLine = "4:02"))
        assertEquals(pageIdentityOf(unmeasured), pageIdentityOf(measured))
    }

    // What:     `@Test fun identityChangesForAnotherFolder()` checks that another folder's tracks change the identity.
    // Why:      A folder change is the page change that must move the list to its playing row or its top.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("identity differs per folder", () => {
    //   expect(pageIdentityOf([row(0), row(1)])).not.toEqual(pageIdentityOf([row(2), row(3)]));
    // });
    // ```
    /** Entries of a different folder give a different page identity. */
    @Test
    fun identityChangesForAnotherFolder() {
        /** Tracks of the first folder. */
        val firstFolder: List<TrackListEntry> = listOf(row(0), row(1))
        /** Tracks of a second folder, with their own library indexes. */
        val secondFolder: List<TrackListEntry> = listOf(row(2), row(3))
        assertNotEquals(pageIdentityOf(firstFolder), pageIdentityOf(secondFolder))
    }

    // What:     `@Test fun identityIgnoresLibraryIndexes()` checks that shifted library indexes keep the identity.
    // Why:      A scan that sorts a new track ahead of this folder shifts every later index, and that must not change
    //           the page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("identity ignores indexes", () => {
    //   expect(pageIdentityOf([row(0, false, null, "Intro")])).toEqual(pageIdentityOf([row(7, false, null, "Intro")]));
    // });
    // ```
    /** The same tracks at shifted library indexes give the same page identity. */
    @Test
    fun identityIgnoresLibraryIndexes() {
        /** Two tracks at their indexes before a scan inserts a track ahead of them. */
        val beforeScan: List<TrackListEntry> = listOf(row(0, title = "Intro"), row(1, title = "Outro"))
        /** The same two tracks at shifted indexes after the scan. */
        val afterScan: List<TrackListEntry> = listOf(row(7, title = "Intro"), row(8, title = "Outro"))
        assertEquals(pageIdentityOf(beforeScan), pageIdentityOf(afterScan))
    }

    // What:     `@Test fun rowTitleAndHeaderNameStayApart()` checks that a track and a header with one text differ.
    // Why:      The prefixes keep a track titled like a subfolder from matching that subfolder's header.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("row and header tokens differ", () => {
    //   expect(pageIdentityOf([row(0, false, null, "Live")])).not.toEqual(pageIdentityOf([header("Live")]));
    // });
    // ```
    /** A track titled like a subfolder gives a different token from that subfolder's header. */
    @Test
    fun rowTitleAndHeaderNameStayApart() {
        /** One track titled "Live" and one subfolder header named "Live". */
        val track: List<TrackListEntry> = listOf(row(0, title = "Live"))
        /** Subfolder header named "Live" with no tracks. */
        val headerOnly: List<TrackListEntry> = listOf(header("Live"))
        assertNotEquals(pageIdentityOf(track), pageIdentityOf(headerOnly))
    }

    // What:     `@Test fun samePageWhenTrackIsAddedDuringScan()` checks that an added track keeps the page.
    // Why:      Batches of a streaming scan add tracks to the shown folder, and the list must stay where it is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("scan add keeps page", () => {
    //   expect(isSamePageUpdate(identity([A, C]), identity([A, B, C]))).toBe(true);
    // });
    // ```
    /** A track added between existing tracks of the shown folder leaves it the same page. */
    @Test
    fun samePageWhenTrackIsAddedDuringScan() {
        /** Shown folder before the scan adds its middle track. */
        val shown: List<String> = pageIdentityOf(listOf(row(0, title = "A"), row(2, title = "C")))
        /** Same folder after the scan adds a track between the two. */
        val grown: List<String> = pageIdentityOf(listOf(row(0, title = "A"), row(1, title = "B"), row(2, title = "C")))
        assertTrue(isSamePageUpdate(previous = shown, next = grown))
    }

    // What:     `@Test fun samePageWhenTrackIsRemoved()` checks that a dropped track keeps the page.
    // Why:      A rescan can drop a track from the shown folder, which must not scroll the list either.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("removal keeps page", () => {
    //   expect(isSamePageUpdate(identity([A, B, C]), identity([A, C]))).toBe(true);
    // });
    // ```
    /** A track removed from the shown folder leaves it the same page. */
    @Test
    fun samePageWhenTrackIsRemoved() {
        /** Shown folder before a track is dropped from it. */
        val shown: List<String> = pageIdentityOf(listOf(row(0, title = "A"), row(1, title = "B"), row(2, title = "C")))
        /** Same folder after the middle track is dropped. */
        val shrunk: List<String> = pageIdentityOf(listOf(row(0, title = "A"), row(2, title = "C")))
        assertTrue(isSamePageUpdate(previous = shown, next = shrunk))
    }

    // What:     `@Test fun differentFolderIsAPageChange()` checks that another folder's tracks are a page change.
    // Why:      Choosing another folder must move the list to its playing row or its top.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("other folder is page change", () => {
    //   expect(isSamePageUpdate(identity([A, B]), identity([C, D]))).toBe(false);
    // });
    // ```
    /** Tracks of a different folder, sharing no order with the shown ones, are a page change. */
    @Test
    fun differentFolderIsAPageChange() {
        /** Shown folder. */
        val shown: List<String> = pageIdentityOf(listOf(row(0, title = "A"), row(1, title = "B")))
        /** Another folder with other tracks. */
        val other: List<String> = pageIdentityOf(listOf(row(2, title = "C"), row(3, title = "D")))
        assertFalse(isSamePageUpdate(previous = shown, next = other))
    }

    // What:     `@Test fun firstPopulationIsAPageChange()` checks a list that had no page before.
    // Why:      The first scroll of a list has no shown page, and a page that fills from empty must scroll once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("first population", () => {
    //   expect(isSamePageUpdate(null, identity([A]))).toBe(false);
    //   expect(isSamePageUpdate([], identity([A]))).toBe(false);
    // });
    // ```
    /** A list with no previous page, or an empty one, treats the next page as a page change. */
    @Test
    fun firstPopulationIsAPageChange() {
        /** Page of one track, which is the next page in both cases. */
        val next: List<String> = pageIdentityOf(listOf(row(0, title = "A")))
        assertFalse(isSamePageUpdate(previous = null, next = next))
        assertFalse(isSamePageUpdate(previous = emptyList(), next = next))
    }

    // What:     `@Test fun emptiedPageIsAPageChange()` checks a page that loses all its entries.
    // Why:      An empty page has no playing row, so the list must go to its top rather than keep its offset.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("emptied page", () => expect(isSamePageUpdate(identity([A]), [])).toBe(false));
    // ```
    /** A page that becomes empty is a page change. */
    @Test
    fun emptiedPageIsAPageChange() {
        /** Page of one track before it empties. */
        val shown: List<String> = pageIdentityOf(listOf(row(0, title = "A")))
        assertFalse(isSamePageUpdate(previous = shown, next = emptyList()))
    }

    // What:     `@Test fun identityOfEmptyPageIsEmpty()` checks the identity of a page with no entries.
    // Why:      A page that has not loaded yet must still give a valid identity, so the first load counts as a change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty identity", () => expect(pageIdentityOf([])).toEqual([]));
    // ```
    /** A page with no entries has an empty identity. */
    @Test
    fun identityOfEmptyPageIsEmpty() {
        assertEquals(emptyList<String>(), pageIdentityOf(emptyList()))
    }

    // What:     `@Test fun rowAndHeaderKeysStayApart()` checks that a row and a header with one text get other keys.
    // Why:      The prefixes keep a header named like a library index from colliding with that track's row key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("row and header keys differ", () => {
    //   expect(entryKeyOf(row(5))).not.toEqual(entryKeyOf(header("5")));
    // });
    // ```
    /** A track row and a subfolder header named after the same number have different keys. */
    @Test
    fun rowAndHeaderKeysStayApart() {
        assertNotEquals(entryKeyOf(row(5)), entryKeyOf(header("5")))
    }

    // What:     `@Test fun currentPositionFollowsThePlayingIndexNotTheRowFlags()` checks how the playing row is found.
    // Why:      The rows of a page carry no playing flag that stays true, so the position must come from the library
    //           index alone, even when some row still holds a stale flag.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("position follows the index", () => {
    //   expect(currentPositionOf([row(0, true), header("Live"), row(1), row(2)], 2)).toBe(3);
    // });
    // ```
    /** The position of the row with the playing library index is returned, skipping headers and stale flags. */
    @Test
    fun currentPositionFollowsThePlayingIndexNotTheRowFlags() {
        /** A flagged first row, then a subfolder header with two rows under it. */
        val entries: List<TrackListEntry> = listOf(row(0, current = true), header("Live"), row(1), row(2))
        assertEquals(3, currentPositionOf(entries, playingIndex = 2))
    }

    // What:     `@Test fun currentPositionIsNullWhenThePlayingIndexIsNotOnThePage()` checks a page without that track.
    // Why:      A page without the playing track has no row position to scroll to.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("not on the page", () => expect(currentPositionOf([header("Live"), row(1)], 7)).toBeNull());
    // ```
    /** A playing index that no row of the page has gives a null position. */
    @Test
    fun currentPositionIsNullWhenThePlayingIndexIsNotOnThePage() {
        /** Subfolder header and its track, neither of which is library index 7. */
        val entries: List<TrackListEntry> = listOf(header("Live"), row(1))
        assertNull(currentPositionOf(entries, playingIndex = 7))
    }

    // What:     `@Test fun currentPositionIsNullWithoutAPlayingIndex()` checks that nothing playing gives no position.
    // Why:      With no track loaded, a row that still holds a stale flag must not be taken as the playing row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("nothing playing", () => expect(currentPositionOf([row(1, true)], null)).toBeNull());
    // ```
    /** A null playing index gives a null position, whatever the rows' own flags say. */
    @Test
    fun currentPositionIsNullWithoutAPlayingIndex() {
        /** One track whose own flag still says playing. */
        val entries: List<TrackListEntry> = listOf(row(1, current = true))
        assertNull(currentPositionOf(entries, playingIndex = null))
    }

    // What:     `@Test fun pageTargetIsPlayingRowWhenOnPage()` checks a page that holds the playing track.
    // Why:      Entry and page change put the playing row at the top, as the list did on entry before this rule.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("target is playing row", () => {
    //   expect(pageEntryScrollTarget([row(0), header("Live"), row(1)], 1)).toBe(2);
    // });
    // ```
    /** A page holding the playing row targets that row's position. */
    @Test
    fun pageTargetIsPlayingRowWhenOnPage() {
        /** First track, then a subfolder header with the playing track under it. */
        val entries: List<TrackListEntry> = listOf(row(0), header("Live"), row(1))
        assertEquals(2, pageEntryScrollTarget(entries, playingIndex = 1))
    }

    // What:     `@Test fun pageTargetIsTopWhenNoRowPlaying()` checks a page without the playing track.
    // Why:      A page change to a folder without the playing track must start at its top, not at the old offset.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("target is top", () => expect(pageEntryScrollTarget([row(0), row(1)], 9)).toBe(0));
    // ```
    /** A page without the playing track targets the top of the list. */
    @Test
    fun pageTargetIsTopWhenNoRowPlaying() {
        /** Two tracks, neither of which is library index 9. */
        val entries: List<TrackListEntry> = listOf(row(0), row(1))
        assertEquals(0, pageEntryScrollTarget(entries, playingIndex = 9))
    }

    // What:     `@Test fun pageTargetIsTopForEmptyPage()` checks a page with no entries.
    // Why:      A page that has not loaded yet still needs a valid target, so the scroll never reads past the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty target", () => expect(pageEntryScrollTarget([], 0)).toBe(0));
    // ```
    /** An empty page targets the top of the list. */
    @Test
    fun pageTargetIsTopForEmptyPage() {
        assertEquals(0, pageEntryScrollTarget(emptyList(), playingIndex = 0))
    }

    // What:     `@Test fun libraryIndexNearIsTheRowAtThePosition()` checks a position that holds a row.
    // Why:      The measure refresh starts at the library index of the first visible row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("row at position", () => expect(libraryIndexNear([row(5), header("Live"), row(8)], 2)).toBe(8));
    // ```
    /** A position on a row gives that row's library index. */
    @Test
    fun libraryIndexNearIsTheRowAtThePosition() {
        /** A row, a subfolder header, and a row. */
        val entries: List<TrackListEntry> = listOf(row(5), header("Live"), row(8))
        assertEquals(8, libraryIndexNear(entries, position = 2))
    }

    // What:     `@Test fun libraryIndexNearSkipsAHeaderToTheNextRow()` checks a position that holds a header.
    // Why:      A header has no library index, and the rows under it are what the user sees next.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("header skips down", () => expect(libraryIndexNear([row(5), header("Live"), row(8)], 1)).toBe(8));
    // ```
    /** A position on a header gives the library index of the next row. */
    @Test
    fun libraryIndexNearSkipsAHeaderToTheNextRow() {
        /** A row, a subfolder header, and a row. */
        val entries: List<TrackListEntry> = listOf(row(5), header("Live"), row(8))
        assertEquals(8, libraryIndexNear(entries, position = 1))
    }

    // What:     `@Test fun libraryIndexNearFallsBackToTheRowBefore()` checks a position with no row after it.
    // Why:      A list that ends in a header, or a position past the end, still names the closest row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("falls back", () => expect(libraryIndexNear([row(5), header("Live")], 1)).toBe(5));
    // ```
    /** With no row at or after the position, the last row before it is named. */
    @Test
    fun libraryIndexNearFallsBackToTheRowBefore() {
        /** A row followed by a header with no rows under it. */
        val entries: List<TrackListEntry> = listOf(row(5), header("Live"))
        assertEquals(5, libraryIndexNear(entries, position = 1))
        assertEquals(5, libraryIndexNear(entries, position = 7))
    }

    // What:     `@Test fun libraryIndexNearIsNullWithoutRows()` checks lists with no row.
    // Why:      An empty page or one with only headers has no track on screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no rows", () => { expect(libraryIndexNear([], 0)).toBeNull(); expect(libraryIndexNear([header("a")], 0)).toBeNull(); });
    // ```
    /** A list without rows gives null, and a negative position is treated as the start. */
    @Test
    fun libraryIndexNearIsNullWithoutRows() {
        assertNull(libraryIndexNear(emptyList(), position = 0))
        assertNull(libraryIndexNear(listOf(header("Live")), position = 0))
        assertEquals(5, libraryIndexNear(listOf(row(5)), position = -3))
    }
}
