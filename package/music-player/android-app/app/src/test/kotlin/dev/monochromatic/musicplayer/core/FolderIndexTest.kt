// Folder index cases for the flat per-folder list (D5) and playback order (D6).

// What:     `package ...core` places these tests beside the folder index they exercise.
// Why:      The tests reach `folderIndex` and its row types through the same package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import dev.monochromatic.musicplayer.Track` brings in the library track type used by the fixtures.
// Why:      `Track` lives in the parent package, so the test imports it by absolute name.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "dev.monochromatic.musicplayer/Track";
// ```
import dev.monochromatic.musicplayer.Track

// What:     JUnit's value assertion and test annotation register each folder case.
// Why:      Each case compares full entry lists, so the flat order and counts are checked together.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

/** Verifies folder grouping, flat per-folder ordering, headers, counts, and the Unfiled entry. */
class FolderIndexTest {
    // What:     `private fun track(displayPath: String): Track` builds a fixture track from its display path.
    // Why:      Only the display path affects grouping, so the URI can be derived from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const track = (displayPath: string): Track => ({ uri: `content://${displayPath}`, displayPath });
    // ```
    /** Builds a library track whose URI is derived from its display path. */
    private fun track(displayPath: String): Track = Track("content://$displayPath", displayPath)

    // What:     `private fun trackItem(tracks: List<Track>, index: Int): FolderListItem.TrackItem` wraps one fixture
    //           track as the row the index should produce for it.
    // Why:      Expected lists are written with the library index, which keeps the test readable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const trackItem = (tracks: Track[], index: number) => ({ kind: "track", row: { track: tracks[index], indexInLibrary: index } });
    // ```
    /** Builds the expected track row for the library track at one index. */
    private fun trackItem(tracks: List<Track>, index: Int): FolderListItem.TrackItem =
        FolderListItem.TrackItem(FolderTrackRow(tracks[index], index))

    // What:     `@Test fun groupsAndSortsTopLevelFolders()` checks the folder order.
    // Why:      The picker lists top-level folders by name, so the entries must be sorted that way.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("groups and sorts top-level folders", () => { /* ... */ });
    // ```
    /** Verifies top-level folders come out sorted by name with one entry each. */
    @Test
    fun groupsAndSortsTopLevelFolders() {
        /** Library with one track in each of three folders, supplied out of name order. */
        val entries = folderIndex(listOf(track("Zed/z.flac"), track("Alpha/a.flac"), track("Mid/m.flac")))
        assertEquals(listOf("Alpha", "Mid", "Zed"), entries.map { entry -> entry.name })
        assertEquals(listOf(1, 1, 1), entries.map { entry -> entry.trackCount })
    }

    // What:     `@Test fun tracksComeBeforeSubfolderHeadersWithCounts()` checks the D5 layout.
    // Why:      D5 puts a folder's own tracks first, then each subfolder as a header with its count and tracks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tracks come before subfolder headers with counts", () => { /* ... */ });
    // ```
    /** Verifies own tracks lead, then each subfolder shows a header followed by its tracks. */
    @Test
    fun tracksComeBeforeSubfolderHeadersWithCounts() {
        /** Camellia folder with two own tracks, two Album tracks, and one Live track. */
        val tracks = listOf(
            track("Camellia/01.flac"),
            track("Camellia/Album/02.flac"),
            track("Camellia/Album/03.flac"),
            track("Camellia/Live/04.flac"),
            track("Camellia/05.flac"),
        )
        /** The single folder entry built from the library. */
        val entry = folderIndex(tracks).single()
        assertEquals(
            listOf(
                trackItem(tracks, 0),
                trackItem(tracks, 4),
                FolderListItem.SubfolderHeader("Album", 2),
                trackItem(tracks, 1),
                trackItem(tracks, 2),
                FolderListItem.SubfolderHeader("Live", 1),
                trackItem(tracks, 3),
            ),
            entry.items,
        )
        assertEquals(5, entry.trackCount)
    }

    // What:     `@Test fun subfolderNamesSortCaseInsensitively()` checks the header order.
    // Why:      Subfolders are listed in name order, so case must not change where a header lands.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("subfolder names sort case-insensitively", () => { /* ... */ });
    // ```
    /** Verifies subfolder headers sort by name regardless of letter case. */
    @Test
    fun subfolderNamesSortCaseInsensitively() {
        /** Camellia folder whose three subfolders are supplied out of name order. */
        val tracks = listOf(
            track("Camellia/beta/1.flac"),
            track("Camellia/Alpha/2.flac"),
            track("Camellia/gamma/3.flac"),
        )
        assertEquals(
            listOf(
                FolderListItem.SubfolderHeader("Alpha", 1),
                trackItem(tracks, 1),
                FolderListItem.SubfolderHeader("beta", 1),
                trackItem(tracks, 0),
                FolderListItem.SubfolderHeader("gamma", 1),
                trackItem(tracks, 2),
            ),
            folderIndex(tracks).single().items,
        )
    }

    // What:     `@Test fun deeperNestingIsFlattenedUnderNearestNamedSubfolderPath()` checks deep paths.
    // Why:      A deeper folder keeps its full path under the header, so nesting never adds a layer (D5).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deeper nesting is flattened under the nearest named subfolder path", () => { /* ... */ });
    // ```
    /** Verifies a track two folders below the top level sits under its full subfolder path header. */
    @Test
    fun deeperNestingIsFlattenedUnderNearestNamedSubfolderPath() {
        /** Camellia folder with one track in Album and two in the deeper Album/Disc 1 folder. */
        val tracks = listOf(
            track("Camellia/Album/01.flac"),
            track("Camellia/Album/Disc 1/02.flac"),
            track("Camellia/Album/Disc 1/03.flac"),
        )
        /** The single folder entry built from the library. */
        val entry = folderIndex(tracks).single()
        assertEquals(
            listOf(
                FolderListItem.SubfolderHeader("Album", 1),
                trackItem(tracks, 0),
                FolderListItem.SubfolderHeader("Album/Disc 1", 2),
                trackItem(tracks, 1),
                trackItem(tracks, 2),
            ),
            entry.items,
        )
        assertEquals(3, entry.trackCount)
    }

    // What:     `@Test fun rootLevelTracksGoToUnfiledLast()` checks the Unfiled entry.
    // Why:      Tracks with no folder form the Unfiled entry, which the picker lists after every real folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("root-level tracks go to Unfiled last", () => { /* ... */ });
    // ```
    /** Verifies root-level tracks form one virtual letter entry per initial, listed after the named folders. */
    @Test
    fun rootLevelTracksGoToTheirLetterEntriesLast() {
        /** Two root-level tracks interleaved with two folders, one of which sorts after the letters. */
        val tracks = listOf(
            track("Loose.flac"),
            track("Zed/z.flac"),
            track("Camellia/c.flac"),
            track("Another loose.flac"),
        )
        /** Folder entries built from the mixed library. */
        val entries = folderIndex(tracks)
        assertEquals(listOf("Camellia", "Zed", "A", "L"), entries.map { entry -> entry.name })
        assertEquals(listOf(false, false, true, true), entries.map { entry -> entry.loose })
        assertEquals(listOf(trackItem(tracks, 3)), entries[2].items)
        assertEquals(listOf(trackItem(tracks, 0)), entries[3].items)
        assertEquals(1, entries.last().trackCount)
    }

    /** Verifies loose tracks share the letter entry of their base letter, and anything else lands on the # entry. */
    @Test
    fun looseTracksFoldAccentsAndKeepNonLatinInitialsOnTheHashEntry() {
        /** Loose tracks whose initials are an accented capital, its plain twin, a lowercase letter, CJK and a digit. */
        val tracks = listOf(
            track("Émilie.flac"),
            track("Eno.flac"),
            track("echo.flac"),
            track("初音.flac"),
            track("1 song.flac"),
        )
        /** Entries of the loose tracks. */
        val entries = folderIndex(tracks)
        assertEquals(listOf("E", "#"), entries.map { entry -> entry.name })
        assertEquals(listOf(trackItem(tracks, 0), trackItem(tracks, 1), trackItem(tracks, 2)), entries[0].items)
        assertEquals(listOf(trackItem(tracks, 3), trackItem(tracks, 4)), entries[1].items)
    }

    // What:     `@Test fun noUnfiledEntryWhenNoTrackSitsAtRoot()` checks the absent case.
    // Why:      The Unfiled entry exists only when a root-level track does, so an all-folder library gets none.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no Unfiled entry when no track sits at the root", () => { /* ... */ });
    // ```
    /** Verifies no letter entry appears when every track sits inside a folder. */
    @Test
    fun noLetterEntryWhenNoTrackSitsAtRoot() {
        assertEquals(listOf("Camellia"), folderIndex(listOf(track("Camellia/c.flac"))).map { entry -> entry.name })
    }

    // What:     `@Test fun indexInLibraryIsPreserved()` checks the library positions of every row.
    // Why:      Playback addresses tracks by library index, so each row must keep the index it had in the input.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("indexInLibrary is preserved", () => { /* ... */ });
    // ```
    /** Verifies every row carries the library position of its own track after reordering. */
    @Test
    fun indexInLibraryIsPreserved() {
        /** Library whose folder rows come out in a different order from the input. */
        val tracks = listOf(
            track("Camellia/Album/a.flac"),
            track("Loose.flac"),
            track("Camellia/b.flac"),
            track("Zed/z.flac"),
        )
        /** Every row in the flat order the index produces, across all entries. */
        val rows = folderIndex(tracks).flatMap { entry ->
            entry.items.filterIsInstance<FolderListItem.TrackItem>().map { item -> item.row }
        }
        assertEquals(listOf(0, 1, 2, 3), rows.map { row -> row.indexInLibrary }.sorted())
        rows.forEach { row -> assertEquals(tracks[row.indexInLibrary], row.track) }
    }

    // What:     `@Test fun emptyLibraryGivesEmptyList()` checks an empty input.
    // Why:      With no tracks there are no folders and no Unfiled entry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty library gives an empty list", () => { expect(folderIndex([])).toEqual([]); });
    // ```
    /** Verifies an empty library produces no folder entries. */
    @Test
    fun emptyLibraryGivesEmptyList() {
        assertEquals(emptyList<FolderEntry>(), folderIndex(emptyList()))
    }

    // What:     `@Test fun folderNamesWithDifferentCaseSortCaseInsensitively()` checks the top-level order.
    // Why:      Folder names sort case-insensitively, so `Camellia` comes before `capsule` and `beta` before both.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder names with different case sort case-insensitively", () => { /* ... */ });
    // ```
    /** Verifies top-level folder names sort without regard to letter case. */
    @Test
    fun folderNamesWithDifferentCaseSortCaseInsensitively() {
        /** One track in each of five folders whose names differ only in case and order. */
        val tracks = listOf(
            track("beta/1.flac"),
            track("Alpha/2.flac"),
            track("gamma/3.flac"),
            track("capsule/4.flac"),
            track("Camellia/5.flac"),
        )
        assertEquals(
            listOf("Alpha", "beta", "Camellia", "capsule", "gamma"),
            folderIndex(tracks).map { entry -> entry.name },
        )
    }

    // What:     `@Test fun foldersThatTieIgnoringCaseSortByCodePoint()` checks the tie-break.
    // Why:      `Camellia` and `camellia` tie at primary strength, so code point order decides and keeps the
    //           picker order stable across runs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folders that tie ignoring case sort by code point", () => { /* ... */ });
    // ```
    /** Verifies two folder names that differ only in case come out in a fixed code point order. */
    @Test
    fun foldersThatTieIgnoringCaseSortByCodePoint() {
        assertEquals(
            listOf("Camellia", "camellia"),
            folderIndex(listOf(track("camellia/x.flac"), track("Camellia/y.flac"))).map { entry -> entry.name },
        )
    }

    // What:     `@Test fun folderWithOnlyOwnTracksHasNoHeaders()` checks a folder without subfolders.
    // Why:      A folder whose tracks all sit directly inside it gets no header rows at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder with only own tracks has no headers", () => { /* ... */ });
    // ```
    /** Verifies a folder with no subfolders lists only its own track rows. */
    @Test
    fun folderWithOnlyOwnTracksHasNoHeaders() {
        /** Two tracks directly inside one folder. */
        val tracks = listOf(track("Camellia/a.flac"), track("Camellia/b.flac"))
        assertEquals(listOf(trackItem(tracks, 0), trackItem(tracks, 1)), folderIndex(tracks).single().items)
    }

    // What:     `@Test fun sameSubfolderNameInTwoFoldersStaysSeparate()` checks headers across folders.
    // Why:      Headers belong to their own top-level folder, so equal subfolder names must not merge.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("same subfolder name in two folders stays separate", () => { /* ... */ });
    // ```
    /** Verifies two folders that each contain a subfolder named Live keep separate headers. */
    @Test
    fun sameSubfolderNameInTwoFoldersStaysSeparate() {
        /** One Live subfolder track in each of two folders. */
        val tracks = listOf(track("A/Live/1.flac"), track("B/Live/2.flac"))
        assertEquals(
            listOf(FolderListItem.SubfolderHeader("Live", 1), trackItem(tracks, 0)),
            folderIndex(tracks)[0].items,
        )
        assertEquals(
            listOf(FolderListItem.SubfolderHeader("Live", 1), trackItem(tracks, 1)),
            folderIndex(tracks)[1].items,
        )
    }
}
