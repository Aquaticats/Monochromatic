// Cases of `playingRowOrderOf` and `playingPlaceIn`: the playing track's place found without indexing its folder again.

// What:     `package ...core` places the test beside `PlayerContentModel.kt`, which declares the functions.
// Why:      The test reaches them and `PageEntry` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class PlayingPlaceTest` groups the cases of the playing place built from a prepared row order.
// Why:      The host builds the row order once per folder and the place once per track change, and the two must agree
//           with the row order the track list itself shows.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("playingPlaceIn", () => { /* cases */ });
// ```
/** Verifies the playing place from a prepared row order matches the track list's order and the one-step builder. */
class PlayingPlaceTest {
    /** One folder in load order with own tracks after a subfolder's track, so the list order differs from load order. */
    private val items: List<PageEntry> = listOf(
        PageEntry(index = 10, name = "Album/Live/x.flac"),
        PageEntry(index = 11, name = "Album/b.flac"),
        PageEntry(index = 12, name = "Album/Live/y.flac"),
        PageEntry(index = 13, name = "Album/a.flac"),
    )

    // What:     `@Test fun rowOrderMatchesTheOrderTheTrackListShows()` checks the order against the list's own rows.
    // Why:      The deck's `N of M` must count rows the way the list lists them, own tracks before subfolders.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("order matches list", () => expect(playingRowOrderOf("Album", items)).toEqual(listRowIndices(items)));
    // ```
    /** The prepared order equals the library indices of the rows of the list built for the same folder. */
    @Test
    fun rowOrderMatchesTheOrderTheTrackListShows() {
        /** Library indices of the rows the track list builds for this folder, in list order, headers skipped. */
        val shown: List<Int> = trackListEntries(
            folder = pageFolderFor(folderTitle = "Album", pageItems = items),
            currentIndexInLibrary = null,
        )
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row.indexInLibrary }
        assertEquals(shown, playingRowOrderOf(folderTitle = "Album", playingItems = items))
        assertEquals(4, shown.size)
    }

    // What:     `@Test fun everyTrackGetsItsPositionInListOrder()` checks the position of each playing track.
    // Why:      A wrong position would show a wrong `N of M` line for that track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("position in list order", () => { for (const [i, index] of order.entries()) expect(place(index).rowPosition).toBe(i); });
    // ```
    /** For each row of the list, the place built from the prepared order reports that row's position and entry. */
    @Test
    fun everyTrackGetsItsPositionInListOrder() {
        /** Prepared order of the folder. */
        val order: List<Int> = playingRowOrderOf(folderTitle = "Album", playingItems = items)
        for ((position, libraryIndex) in order.withIndex()) {
            /** Place of the track at this position of the list. */
            val place: PlayingPlace = playingPlaceIn(playingItems = items, rowOrder = order, currentIndex = libraryIndex)
            assertEquals(position, place.rowPosition)
            assertEquals(libraryIndex, place.entry?.index)
            assertEquals(items.size, place.folderTrackCount)
        }
    }

    // What:     `@Test fun noPlayingTrackHasNoEntryAndNoPosition()` checks a null playing index.
    // Why:      With nothing loaded the deck must show no track and no position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("nothing playing", () => expect(playingPlaceIn(items, order, null)).toEqual({ entry: null, rowPosition: -1, count }));
    // ```
    /** A null playing index gives no entry and the position -1. */
    @Test
    fun noPlayingTrackHasNoEntryAndNoPosition() {
        /** Prepared order of the folder. */
        val order: List<Int> = playingRowOrderOf(folderTitle = "Album", playingItems = items)
        assertEquals(
            PlayingPlace(entry = null, rowPosition = -1, folderTrackCount = items.size),
            playingPlaceIn(playingItems = items, rowOrder = order, currentIndex = null),
        )
    }

    // What:     `@Test fun aTrackOutsideTheFolderHasNoEntryAndNoPosition()` checks an index of another folder.
    // Why:      The deck reads the playing folder only, and a track elsewhere must not borrow a position here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track elsewhere", () => expect(playingPlaceIn(items, order, 99).rowPosition).toBe(-1));
    // ```
    /** An index that no entry of the folder has gives no entry and the position -1. */
    @Test
    fun aTrackOutsideTheFolderHasNoEntryAndNoPosition() {
        /** Prepared order of the folder. */
        val order: List<Int> = playingRowOrderOf(folderTitle = "Album", playingItems = items)
        assertEquals(
            PlayingPlace(entry = null, rowPosition = -1, folderTrackCount = items.size),
            playingPlaceIn(playingItems = items, rowOrder = order, currentIndex = 99),
        )
    }

    // What:     `@Test fun oneStepBuilderAgreesWithThePreparedOrderBuilder()` checks `playingPlaceOf` against the split.
    // Why:      Other callers still use the one-step form, and it must give the same place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one step equals split", () => expect(playingPlaceOf("Album", items, 12)).toEqual(playingPlaceIn(items, order, 12)));
    // ```
    /** The one-step builder equals the prepared-order builder for a track in a subfolder. */
    @Test
    fun oneStepBuilderAgreesWithThePreparedOrderBuilder() {
        /** Prepared order of the folder. */
        val order: List<Int> = playingRowOrderOf(folderTitle = "Album", playingItems = items)
        assertEquals(
            playingPlaceIn(playingItems = items, rowOrder = order, currentIndex = 12),
            playingPlaceOf(folderTitle = "Album", playingItems = items, currentIndex = 12),
        )
    }
}
